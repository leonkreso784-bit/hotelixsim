use app::memory::{JobStatus, Memory, deps};
use app::*;
use content::{Snapshot, repo_dir};
use game::RoundStatus;
use game::dto::MonthCode::{Aug, Jul, Jun};
use std::sync::Arc;

fn setup() -> (Deps, Arc<Memory>) {
    deps(Snapshot::read(&repo_dir()).unwrap())
}

async fn guest(d: &Deps) -> (Caller, DemoStarted) {
    let s = start_demo(d, None).await.unwrap();
    let c = caller(d, s.new_token.as_deref()).await.unwrap();
    (c, s)
}

fn robot_hotel(mem: &Memory, game: GameId) -> HotelId {
    let created = &mem.events(game)[0];
    HotelId(
        created.payload["hotels"][1]["hotel_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
    )
}

fn events_of(mem: &Memory, g: GameId, kind: &str) -> usize {
    mem.events(g).iter().filter(|e| e.kind == kind).count()
}

#[tokio::test]
async fn plan_answer_close_then_worker_resolves_june() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let mut plan = my_hotel(&d, &me)
        .await
        .unwrap()
        .private
        .unwrap()
        .plan
        .unwrap()
        .plan;
    plan.marketing_cents = 500_000;
    assert_eq!(
        set_plan(&d, &me, s.hotel_id, Jun, plan.clone())
            .await
            .unwrap(),
        plan
    );
    answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap();
    assert_eq!(
        round_status(&d, &me, s.game_id, Jun)
            .await
            .unwrap()
            .unanswered,
        1
    );
    let c = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(c.accepted);
    assert_eq!(c.round.status, RoundStatus::Closing);
    assert!(run_next_job(&d).await.unwrap());
    assert!(!run_next_job(&d).await.unwrap(), "red je prazan");
    assert_eq!(
        round_status(&d, &me, s.game_id, Jun).await.unwrap().status,
        RoundStatus::Resolved
    );
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current.unwrap().month, Jul);
    assert!(g.hotels.iter().all(|h| h.months.len() == 1));
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved"), 1);
}

#[tokio::test]
async fn double_close_is_idempotent() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    assert!(close_round(&d, &me, s.game_id, Jun).await.unwrap().accepted);
    let again = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(!again.accepted);
    assert_eq!(again.round.status, RoundStatus::Closing);
    assert_eq!(mem.jobs().len(), 1, "jedan posao po ključu");
    run_next_job(&d).await.unwrap();
    let after = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!(
        (after.accepted, after.round.status),
        (false, RoundStatus::Resolved)
    );
    assert_eq!(events_of(&mem, s.game_id, "RoundClosed"), 1);
}

#[tokio::test]
async fn plan_for_closed_month_rejected() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let plan = my_hotel(&d, &me)
        .await
        .unwrap()
        .private
        .unwrap()
        .plan
        .unwrap()
        .plan;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!(
        set_plan(&d, &me, s.hotel_id, Jun, plan)
            .await
            .unwrap_err()
            .code,
        ErrorCode::RoundNotOpen
    );
    assert_eq!(
        answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap_err().code,
        ErrorCode::RoundNotOpen
    );
    assert_eq!(
        close_round(&d, &me, s.game_id, Jul).await.unwrap_err().code,
        ErrorCode::RoundNotOpen
    );
    assert_eq!(
        close_round(&d, &me, s.game_id, Aug).await.unwrap_err().code,
        ErrorCode::UnknownMonth
    );
}

#[tokio::test]
async fn crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    let claimed = d.jobs.claim(30).await.unwrap().unwrap(); // radnik preuzme i „padne"
    assert_eq!(claimed.attempts, 1);
    assert!(!run_next_job(&d).await.unwrap(), "zakup još traje");
    mem.expire_leases();
    assert!(run_next_job(&d).await.unwrap());
    mem.push_job(NewJob {
        key: "dupli".into(),
        kind: RESOLVE_KIND.into(),
        game_id: s.game_id,
        payload: serde_json::json!({"month": "jun"}),
    });
    assert!(run_next_job(&d).await.unwrap());
    assert_eq!(
        events_of(&mem, s.game_id, "RoundResolved"),
        1,
        "drugi posao ne obračunava ponovno"
    );
    assert!(mem.jobs().iter().all(|j| j.status == JobStatus::Done));
}

#[tokio::test]
async fn failing_job_is_recorded_and_eventually_failed() {
    let (d, mem) = setup();
    let (_, s) = guest(&d).await;
    mem.push_job(NewJob {
        key: "los".into(),
        kind: "nepoznat".into(),
        game_id: s.game_id,
        payload: serde_json::json!({}),
    });
    for _ in 0..MAX_ATTEMPTS {
        assert!(run_next_job(&d).await.is_err());
    }
    assert_eq!(mem.jobs()[0].status, JobStatus::Failed);
    assert!(!run_next_job(&d).await.unwrap());
}

#[tokio::test]
async fn done_job_is_never_resurrected_by_fail() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    run_next_job(&d).await.unwrap();
    let id = mem.jobs()[0].id;
    d.jobs.fail(id, "zakašnjela greška".into()).await.unwrap();
    assert_eq!(mem.jobs()[0].status, JobStatus::Done);
    assert!(!run_next_job(&d).await.unwrap());
}

#[tokio::test]
async fn rights_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let plan = my_hotel(&d, &me)
        .await
        .unwrap()
        .private
        .unwrap()
        .plan
        .unwrap()
        .plan;
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(
        set_plan(&d, &me, robot, Jun, plan.clone())
            .await
            .unwrap_err()
            .code,
        ErrorCode::NotYourHotel
    );
    assert_eq!(
        answer(&d, &me, robot, "S1", 0).await.unwrap_err().code,
        ErrorCode::NotYourHotel
    );
    assert_eq!(
        close_round(&d, &other, s.game_id, Jun)
            .await
            .unwrap_err()
            .code,
        ErrorCode::NotInGame
    );
    let anon = Caller { guest: None };
    assert_eq!(
        close_round(&d, &anon, s.game_id, Jun)
            .await
            .unwrap_err()
            .code,
        ErrorCode::GuestRequired
    );
    let mut bad = plan.clone();
    bad.price_cents += 50;
    assert_eq!(
        set_plan(&d, &me, s.hotel_id, Jun, bad)
            .await
            .unwrap_err()
            .code,
        ErrorCode::PriceNotWholeEuro
    );
    let unknown = HotelId(uuid::Uuid::from_u128(9999));
    assert_eq!(
        set_plan(&d, &me, unknown, Jun, plan)
            .await
            .unwrap_err()
            .code,
        ErrorCode::UnknownHotel
    );
}

#[tokio::test]
async fn full_demo_season_june_and_july() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    run_next_job(&d).await.unwrap();
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert_eq!(
        v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
        ["S2"]
    );
    close_round(&d, &me, s.game_id, Jul).await.unwrap();
    run_next_job(&d).await.unwrap();
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current, None, "igra je gotova nakon srpnja");
    assert!(g.hotels.iter().all(|h| h.months.len() == 2));
    assert!(!close_round(&d, &me, s.game_id, Jul).await.unwrap().accepted);
}
