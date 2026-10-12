use app::memory::{JobStatus, Memory, deps};
use app::*;
use content::{Snapshot, repo_dir};
use game::RoundKey;
use game::RoundStatus;
use sim_hotel::Month;
use std::sync::Arc;

fn setup() -> (Deps, Arc<Memory>) {
    deps(Snapshot::read(&repo_dir()).unwrap())
}

async fn guest(d: &Deps) -> (Caller, DemoStarted) {
    let s = start_demo(d, None, NewGameSettings::default())
        .await
        .unwrap();
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
        set_plan(
            &d,
            &me,
            s.hotel_id,
            RoundKey::Month(Month::Jun),
            plan.clone()
        )
        .await
        .unwrap(),
        plan
    );
    answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap();
    assert_eq!(
        round_status(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
            .await
            .unwrap()
            .unanswered,
        1
    );
    let c = close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    assert!(c.accepted);
    assert_eq!(c.round.status, RoundStatus::Closing);
    assert!(run_next_job(&d).await.unwrap());
    assert!(
        run_next_job(&d).await.unwrap(),
        "mentorov posao za igračev hotel"
    );
    assert!(!run_next_job(&d).await.unwrap(), "red je prazan");
    assert_eq!(
        round_status(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
            .await
            .unwrap()
            .status,
        RoundStatus::Resolved
    );
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current.unwrap().round.key, RoundKey::Month(Month::Jul));
    assert!(g.hotels.iter().all(|h| h.months.len() == 1));
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved"), 1);
}

#[tokio::test]
async fn double_close_is_idempotent() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    assert!(
        close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
            .await
            .unwrap()
            .accepted
    );
    let again = close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    assert!(!again.accepted);
    assert_eq!(again.round.status, RoundStatus::Closing);
    assert_eq!(mem.jobs().len(), 1, "jedan posao po ključu");
    run_next_job(&d).await.unwrap();
    let after = close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
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
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    assert_eq!(
        set_plan(&d, &me, s.hotel_id, RoundKey::Month(Month::Jun), plan)
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
        close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jul))
            .await
            .unwrap_err()
            .code,
        ErrorCode::RoundNotOpen
    );
    assert_eq!(
        close_round(&d, &me, s.game_id, RoundKey::Month(Month::Aug))
            .await
            .unwrap_err()
            .code,
        ErrorCode::UnknownMonth
    );
}

#[tokio::test]
async fn crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    let claimed = d.jobs.claim(30).await.unwrap().unwrap(); // radnik preuzme i „padne"
    assert_eq!(claimed.attempts, 1);
    assert!(!run_next_job(&d).await.unwrap(), "zakup još traje");
    mem.expire_leases();
    assert!(run_next_job(&d).await.unwrap());
    while run_next_job(&d).await.unwrap() {} // mentorov posao nastao obračunom
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
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    run_next_job(&d).await.unwrap();
    let claim = JobClaim {
        id: mem.jobs()[0].id,
        attempts: 1,
    };
    d.jobs
        .fail(claim, "zakašnjela greška".into())
        .await
        .unwrap();
    assert_eq!(mem.jobs()[0].status, JobStatus::Done);
    assert!(run_next_job(&d).await.unwrap(), "mentorov posao");
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
        set_plan(&d, &me, robot, RoundKey::Month(Month::Jun), plan.clone())
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
        close_round(&d, &other, s.game_id, RoundKey::Month(Month::Jun))
            .await
            .unwrap_err()
            .code,
        ErrorCode::NotInGame
    );
    let anon = Caller { guest: None };
    assert_eq!(
        close_round(&d, &anon, s.game_id, RoundKey::Month(Month::Jun))
            .await
            .unwrap_err()
            .code,
        ErrorCode::GuestRequired
    );
    let mut bad = plan.clone();
    bad.price_cents += 50;
    assert_eq!(
        set_plan(&d, &me, s.hotel_id, RoundKey::Month(Month::Jun), bad)
            .await
            .unwrap_err()
            .code,
        ErrorCode::PriceNotWholeEuro
    );
    let unknown = HotelId(uuid::Uuid::from_u128(9999));
    assert_eq!(
        set_plan(&d, &me, unknown, RoundKey::Month(Month::Jun), plan)
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
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    while run_next_job(&d).await.unwrap() {}
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert_eq!(
        v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
        ["S2"]
    );
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jul))
        .await
        .unwrap();
    while run_next_job(&d).await.unwrap() {}
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current, None, "igra je gotova nakon srpnja");
    assert!(g.hotels.iter().all(|h| h.months.len() == 2));
    assert!(
        !close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jul))
            .await
            .unwrap()
            .accepted
    );
}

#[tokio::test]
async fn stale_worker_cannot_clobber_reclaimed_job() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    let a = d.jobs.claim(30).await.unwrap().unwrap();
    mem.expire_leases();
    let b = d.jobs.claim(30).await.unwrap().unwrap();
    assert_eq!((a.id, a.attempts, b.attempts), (b.id, 1, 2));
    d.jobs.fail(a.claim(), "kasni A".into()).await.unwrap();
    let j = &mem.jobs()[0];
    assert_eq!((j.status, j.attempts), (JobStatus::Running, 2));
    // A-ov kasni završetak ne zatvara B-ov posao
    let stale = a.claim();
    d.log
        .decide(
            s.game_id,
            Box::new(move |_| {
                Ok(Decision {
                    finish_job: Some(stale),
                    ..Decision::default()
                })
            }),
        )
        .await
        .unwrap();
    assert_eq!(mem.jobs()[0].status, JobStatus::Running);
    // B normalno završi
    let own = b.claim();
    d.log
        .decide(
            s.game_id,
            Box::new(move |_| {
                Ok(Decision {
                    finish_job: Some(own),
                    ..Decision::default()
                })
            }),
        )
        .await
        .unwrap();
    assert_eq!(mem.jobs()[0].status, JobStatus::Done);
}

#[tokio::test]
async fn expired_lease_on_last_attempt_fails_job() {
    let (d, mem) = setup();
    let (_, s) = guest(&d).await;
    mem.push_job(NewJob {
        key: "zadnji".into(),
        kind: RESOLVE_KIND.into(),
        game_id: s.game_id,
        payload: serde_json::json!({"month": "jun"}),
    });
    for _ in 0..MAX_ATTEMPTS {
        assert!(d.jobs.claim(30).await.unwrap().is_some());
        mem.expire_leases();
    }
    assert_eq!(mem.jobs()[0].status, JobStatus::Failed);
    assert!(d.jobs.claim(30).await.unwrap().is_none());
}

#[tokio::test]
async fn weekly_debrief_calls_ai_only_at_month_end() {
    let ai = Arc::new(app::memory::FakeAi::good());
    let (d, mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai.clone());
    let settings = NewGameSettings {
        model: content::Model::V2,
        kind: Some(sim_hotel_v2::PeriodKind::Week),
    };
    let s = start_demo(&d, None, settings).await.unwrap();
    let me = caller(&d, s.new_token.as_deref()).await.unwrap();
    for n in 1..=5 {
        assert!(
            close_round(&d, &me, s.game_id, game::RoundKey::Week(n))
                .await
                .unwrap()
                .accepted,
            "t0{n}"
        );
        while run_next_job(&d).await.unwrap() {}
    }
    let debriefs: Vec<_> = mem
        .events(s.game_id)
        .into_iter()
        .filter(|e| e.kind == "DebriefRecorded")
        .collect();
    assert_eq!(debriefs.len(), 5);
    assert_eq!(ai.calls().2, 1); // samo t05 (završava lipanj)
    for e in &debriefs[..4] {
        assert_eq!(e.payload["source"], "fallback");
        assert!(e.payload["text"].as_str().unwrap().contains("Tri uzroka"));
        assert!(
            !e.payload["text"]
                .as_str()
                .unwrap()
                .contains("Što je išlo dobro")
        );
    }
    assert_eq!(debriefs[4].payload["month"], "t05");
    assert_eq!(debriefs[4].payload["source"], "ai");
}

/// v2 mjesečna igra s danim AI-jem: (deps, memorija, pozivatelj, igra).
async fn monthly_v2_with(ai: Arc<dyn AiPort>) -> (Deps, Arc<Memory>, Caller, DemoStarted) {
    let (d, mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai);
    let settings = NewGameSettings {
        model: content::Model::V2,
        kind: Some(sim_hotel_v2::PeriodKind::Month),
    };
    let s = start_demo(&d, None, settings).await.unwrap();
    let me = caller(&d, s.new_token.as_deref()).await.unwrap();
    (d, mem, me, s)
}

async fn close_and_work(d: &Deps, me: &Caller, g: GameId, m: Month) {
    assert!(
        close_round(d, me, g, RoundKey::Month(m))
            .await
            .unwrap()
            .accepted
    );
    while run_next_job(d).await.unwrap() {}
}

fn season_jobs(mem: &Memory) -> usize {
    mem.jobs()
        .iter()
        .filter(|j| j.kind == SEASON_DEBRIEF_KIND)
        .count()
}

/// Kraj-sezone-spec §6.3: posao sezonskog mentora stavlja se u red samo uz obračun zadnjeg kruga v2 igre.
#[tokio::test]
async fn last_v2_round_queues_season_debrief_and_ai_writes_it() {
    let ai = Arc::new(app::memory::FakeAi::good());
    let (d, mem, me, s) = monthly_v2_with(ai.clone()).await;
    for m in [Month::Jun, Month::Jul, Month::Aug] {
        close_and_work(&d, &me, s.game_id, m).await;
        assert_eq!(season_jobs(&mem), 0, "{m:?}");
    }
    close_and_work(&d, &me, s.game_id, Month::Sep).await;
    assert_eq!(season_jobs(&mem), 1);
    assert_eq!(ai.season_calls(), 1);
    let evs: Vec<_> = mem
        .events(s.game_id)
        .into_iter()
        .filter(|e| e.kind == "SeasonDebriefRecorded")
        .collect();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].payload["source"], "ai");
    assert_eq!(evs[0].payload["hotel_id"], s.hotel_id.0.to_string());
    let view = season_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(
        view.season.mentor.as_deref(),
        evs[0].payload["text"].as_str()
    );

    // Drugi prolaz istog posla ne dodaje ništa (mentor je već zapisan) i ne zove AI.
    mem.push_job(NewJob {
        key: "ponovno".into(),
        kind: SEASON_DEBRIEF_KIND.into(),
        game_id: s.game_id,
        payload: serde_json::json!({ "hotel_id": s.hotel_id }),
    });
    assert!(run_next_job(&d).await.unwrap());
    assert_eq!(events_of(&mem, s.game_id, "SeasonDebriefRecorded"), 1);
    assert_eq!(ai.season_calls(), 1);
}

#[tokio::test]
async fn without_ai_key_season_debrief_is_fallback_from_numbers() {
    let (d, mem, me, s) = monthly_v2_with(Arc::new(DisabledAi)).await;
    for m in [Month::Jun, Month::Jul, Month::Aug, Month::Sep] {
        close_and_work(&d, &me, s.game_id, m).await;
    }
    let evs: Vec<_> = mem
        .events(s.game_id)
        .into_iter()
        .filter(|e| e.kind == "SeasonDebriefRecorded")
        .collect();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].payload["source"], "fallback");
    assert!(evs[0].payload.get("meta").is_none());
    let text = evs[0].payload["text"].as_str().unwrap();
    assert!(text.starts_with("Kako je prošla sezona\n"), "{text}");
    assert!(text.ends_with("ovo je sažetak iz brojki."), "{text}");
}

#[tokio::test]
async fn v1_game_never_queues_season_debrief() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    for m in [Month::Jun, Month::Jul] {
        close_and_work(&d, &me, s.game_id, m).await;
    }
    assert_eq!(
        round_status(&d, &me, s.game_id, RoundKey::Month(Month::Jul))
            .await
            .unwrap()
            .status,
        RoundStatus::Resolved
    );
    assert_eq!(season_jobs(&mem), 0);
    assert_eq!(events_of(&mem, s.game_id, "SeasonDebriefRecorded"), 0);
}
