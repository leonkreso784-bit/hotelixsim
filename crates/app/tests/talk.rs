use std::sync::Arc;

use app::memory::FakeAi;
use app::{Caller, Deps, ErrorCode, GameId, HotelId, ResponseStatus};
use game::RoundStatus;
use game::dto::MonthCode;
use sim_hotel::Month;

fn snapshot() -> content::Snapshot {
    content::Snapshot::read(&content::repo_dir()).unwrap()
}

async fn start(deps: &Deps) -> (Caller, HotelId, GameId) {
    let d = app::start_demo(deps, None).await.unwrap();
    let caller = app::caller(deps, d.new_token.as_deref()).await.unwrap();
    (caller, d.hotel_id, d.game_id)
}

/// Vrti radnika dok red nije prazan; greške poslova se ignoriraju (red ih sam ponavlja), najviše 60 koraka.
async fn run_all_jobs(deps: &Deps) {
    for _ in 0..60 {
        if let Ok(false) = app::run_next_job(deps).await {
            return;
        }
    }
    panic!("red se nije ispraznio u 60 koraka");
}

fn s1_response(s: &app::SituationsView) -> app::ResponseView {
    s.items
        .iter()
        .find(|i| i.code == "S1")
        .unwrap()
        .response
        .clone()
        .unwrap()
}

/// Jedno pravilo „bez odgovora": pogled situacija i `current.unanswered` igre moraju se slagati.
async fn assert_unanswered(deps: &Deps, caller: &Caller, hotel: HotelId, game: GameId, n: u8) {
    let s = app::situations(deps, caller, hotel).await.unwrap();
    let g = app::game_view(deps, caller, game).await.unwrap();
    assert_eq!(s.unanswered, n);
    assert_eq!(g.current.unwrap().unanswered, n);
}

#[tokio::test]
async fn submitted_free_response_counts_as_answered_in_situations() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, game) = start(&deps).await;
    assert_unanswered(&deps, &caller, hotel, game, 2).await;
    // slobodan odgovor (predan, presuda još čeka) računa se kao odgovor
    app::submit_response(&deps, &caller, hotel, "S1", "Ispričam se.".into())
        .await
        .unwrap();
    assert_unanswered(&deps, &caller, hotel, game, 1).await;
    // ponuđeni odgovor 0 na S3
    app::answer(&deps, &caller, hotel, "S3", 0).await.unwrap();
    assert_unanswered(&deps, &caller, hotel, game, 0).await;
}

#[tokio::test]
async fn unsubmitted_free_response_stays_unanswered() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, game) = start(&deps).await;
    // samo razgovor s likom nije odgovor
    app::send_message(&deps, &caller, hotel, "S1", "Halo?".into())
        .await
        .unwrap();
    app::answer(&deps, &caller, hotel, "S3", 0).await.unwrap();
    assert_unanswered(&deps, &caller, hotel, game, 1).await;
}

#[tokio::test]
async fn chat_message_gets_character_reply_through_job() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, _) = start(&deps).await;
    let v = app::send_message(
        &deps,
        &caller,
        hotel,
        "S1",
        "Ispričavam se, odmah dolazim.".into(),
    )
    .await
    .unwrap();
    assert!(v.awaiting_reply);
    assert_eq!(v.remaining, 5);
    run_all_jobs(&deps).await;
    let v = app::conversation(&deps, &caller, hotel, "S1")
        .await
        .unwrap();
    assert!(!v.awaiting_reply);
    assert_eq!(v.lines.len(), 2);
    assert_eq!(v.lines[1].text, "Hvala, to mi pomaže.");
}

#[tokio::test]
async fn free_response_is_judged_and_shown_with_criteria() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, _) = start(&deps).await;
    let s = app::submit_response(
        &deps,
        &caller,
        hotel,
        "S1",
        "Ispričam se i premjestim obitelj.".into(),
    )
    .await
    .unwrap();
    assert_eq!(s1_response(&s).status, ResponseStatus::Pending);
    run_all_jobs(&deps).await;
    let r = s1_response(&app::situations(&deps, &caller, hotel).await.unwrap());
    assert_eq!((r.status, r.quality), (ResponseStatus::Judged, Some(4)));
    assert_eq!(r.criteria.len(), 5);
    assert!(r.criteria[0].text.starts_with("Brza isprika"));
}

#[tokio::test]
async fn failing_ai_falls_back_on_last_attempt() {
    let (deps, _mem) = app::memory::deps_with_ai(snapshot(), Arc::new(FakeAi::failing()));
    let (caller, hotel, _) = start(&deps).await;
    app::send_message(&deps, &caller, hotel, "S1", "Halo?".into())
        .await
        .unwrap();
    // razgovor ne blokira slobodan odgovor: student smije razgovarati pa odgovoriti
    app::submit_response(
        &deps,
        &caller,
        hotel,
        "S1",
        "Premjestit ću vas odmah.".into(),
    )
    .await
    .unwrap();
    // S2 je srpanj: šifra postoji (jedinstvena po igri, ADR-017), ali runda nije otvorena
    let e = app::submit_response(&deps, &caller, hotel, "S2", "Dobit ćeš povišicu.".into())
        .await
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::RoundNotOpen);
    run_all_jobs(&deps).await; // memorija nema sat: neuspio posao odmah se vraća; peti pokušaj piše rezervu
    let v = app::conversation(&deps, &caller, hotel, "S1")
        .await
        .unwrap();
    assert_eq!(
        v.lines.last().unwrap().text,
        "Gošća je previše ljuta da bi nastavila razgovor. Odluči što ćeš poduzeti."
    );
    let r = s1_response(&app::situations(&deps, &caller, hotel).await.unwrap());
    assert_eq!((r.status, r.quality), (ResponseStatus::Neutral, Some(3)));
}

#[tokio::test]
async fn close_waits_for_verdict_then_neutral_and_late_verdict_is_kept() {
    let (deps, mem) = app::memory::deps(snapshot());
    let (caller, hotel, game) = start(&deps).await;
    app::submit_response(
        &deps,
        &caller,
        hotel,
        "S1",
        "Ispričam se i premjestim obitelj.".into(),
    )
    .await
    .unwrap();
    // posao procjene preuzmi ručno i drži ga „u radu" (kao spor AI)
    let assess = deps.jobs.claim(app::LEASE_SECS).await.unwrap().unwrap();
    assert_eq!(assess.kind, app::ASSESS_KIND);
    app::close_round(&deps, &caller, game, MonthCode::Jun)
        .await
        .unwrap();
    // obračun vidi presudu na čekanju i odgađa se
    assert!(app::run_next_job(&deps).await.unwrap());
    let r = app::round_status(&deps, &caller, game, MonthCode::Jun)
        .await
        .unwrap();
    assert_eq!(r.status, RoundStatus::Closing);
    // rok je prošao → neutralna presuda i obračun
    mem.set_job_age(
        &app::resolve_key(game, Month::Jun),
        app::VERDICT_DEADLINE_SECS + 1,
    );
    assert!(app::run_next_job(&deps).await.unwrap());
    let r = app::round_status(&deps, &caller, game, MonthCode::Jun)
        .await
        .unwrap();
    assert_eq!(r.status, RoundStatus::Resolved);
    let s = app::situations_for_month(&deps, &caller, hotel, MonthCode::Jun)
        .await
        .unwrap();
    assert_eq!(s1_response(&s).status, ResponseStatus::Neutral);
    // spori AI napokon završi: presuda se zapiše kao zakašnjela, obračun se ne mijenja
    app::run_job_for_test(&deps, &assess).await.unwrap();
    let feed = app::feed(&deps, &caller, hotel).await.unwrap();
    assert!(
        feed.iter()
            .any(|i| matches!(i, game::feed::FeedItem::Verdict { late: true, .. }))
    );
}

#[tokio::test]
async fn resolve_enqueues_debrief_for_human_only_and_mentor_appears() {
    let ai = Arc::new(FakeAi::good());
    let (deps, _mem) = app::memory::deps_with_ai(snapshot(), ai.clone());
    let (caller, hotel, game) = start(&deps).await;
    app::close_round(&deps, &caller, game, MonthCode::Jun)
        .await
        .unwrap();
    run_all_jobs(&deps).await;
    let h = app::hotel_view(&deps, &caller, hotel).await.unwrap();
    let last = h.private.unwrap().last_month.unwrap();
    assert_eq!(last.mentor.as_deref(), Some("Mentor: dobar mjesec."));
    assert_eq!(ai.calls().2, 1, "mentor samo za hotel s čovjekom");
}

#[tokio::test]
async fn other_guest_cannot_chat_or_respond_in_my_hotel() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (_, hotel, _) = start(&deps).await;
    let (other, _, _) = start(&deps).await; // drugi gost u drugoj igri
    let e = app::send_message(&deps, &other, hotel, "S1", "bok".into())
        .await
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
    let e = app::submit_response(&deps, &other, hotel, "S1", "bok".into())
        .await
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
    let e = app::conversation(&deps, &other, hotel, "S1")
        .await
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
}

#[tokio::test]
async fn refused_reply_falls_back_after_single_attempt() {
    let ai = Arc::new(FakeAi::refusing());
    let (deps, _mem) = app::memory::deps_with_ai(snapshot(), ai.clone());
    let (caller, hotel, _) = start(&deps).await;
    app::send_message(&deps, &caller, hotel, "S1", "Halo?".into())
        .await
        .unwrap();
    run_all_jobs(&deps).await;
    assert_eq!(ai.calls().1, 1, "odbijanje se ne ponavlja");
    let v = app::conversation(&deps, &caller, hotel, "S1")
        .await
        .unwrap();
    assert_eq!(
        v.lines.last().unwrap().text,
        "Gošća je previše ljuta da bi nastavila razgovor. Odluči što ćeš poduzeti."
    );
}

#[tokio::test]
async fn closed_round_is_not_awaiting_reply() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, game) = start(&deps).await;
    app::send_message(&deps, &caller, hotel, "S1", "Halo?".into())
        .await
        .unwrap();
    app::close_round(&deps, &caller, game, MonthCode::Jun)
        .await
        .unwrap();
    let v = app::conversation(&deps, &caller, hotel, "S1")
        .await
        .unwrap();
    assert!(!v.open);
    assert!(!v.awaiting_reply, "zatvorena runda ne čeka odgovor lika");
}
