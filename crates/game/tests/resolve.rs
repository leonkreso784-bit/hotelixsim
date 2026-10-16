mod common;
use common::*;
use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::dto::*;
use game::feed::FeedItem;
use game::*;
use sim_hotel::Month;
use uuid::Uuid;

fn me() -> HotelId {
    hotel_ids()[0]
}

/// Lipanj: igrač odgovori na S1 (indeks 0), S3 ostavi; zatvori.
fn june_closed(seed: u64) -> Vec<Recorded> {
    let mut log = created(seed);
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.choose_answer(me(), "S1", 0).unwrap(),
    );
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.close_round(Month::Jun).unwrap().unwrap(),
    );
    log
}

fn run_job(log: &mut Vec<Recorded>, m: Month) {
    let ResolveStep::Done(evs) = Game::fold(log).unwrap().resolve_job(m, false).unwrap() else {
        panic!("presude ne čekaju");
    };
    for (a, e) in evs {
        push(log, a, e);
    }
}

#[test]
fn june_resolution_covers_four_hotels_and_unanswered_is_quality_one() {
    let g = Game::fold(&june_closed(42)).unwrap();
    let rr = g.resolve_round(Month::Jun).unwrap();
    assert_eq!(
        rr.hotels.iter().map(|h| h.hotel_id).collect::<Vec<_>>(),
        hotel_ids()
    );
    let mine = &rr.hotels[0];
    assert_eq!(
        mine.situations[0],
        SituationResult {
            code: "S1".into(),
            answer: Some(0),
            quality: 5,
            free_text: false,
            delegated: false,
            capped: false,
        }
    );
    assert_eq!(
        mine.situations[1],
        SituationResult {
            code: "S3".into(),
            answer: None,
            quality: 1,
            free_text: false,
            delegated: false,
            capped: false,
        }
    );
    assert!(
        rr.hotels[1..]
            .iter()
            .all(|h| h.situations.iter().all(|s| s.answer.is_some()))
    );
}

#[test]
fn unanswered_s1_costs_satisfaction() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.close_round(Month::Jun).unwrap().unwrap(),
    );
    let rr = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    let line = rr.hotels[0]
        .report
        .lines
        .iter()
        .find(|l| l.var == VarCode::Satisfaction && l.cause == CauseDto::Situation("S1".into()))
        .expect("neodgovorena S1 ima redak zadovoljstva");
    assert_eq!(line.amount, -400);
}

#[test]
fn replay_gives_byte_identical_round_resolved() {
    let log = june_closed(42);
    let first = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    // kao iz baze: JSON → tipizirano → JSON
    let stored: RoundResolved =
        serde_json::from_value(serde_json::to_value(&first).unwrap()).unwrap();
    let again = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    assert_eq!(
        serde_json::to_vec(&again).unwrap(),
        serde_json::to_vec(&first).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&stored).unwrap(),
        serde_json::to_vec(&first).unwrap()
    );
    let other = Game::fold(&june_closed(42))
        .unwrap()
        .resolve_round(Month::Jun)
        .unwrap();
    assert_eq!(other, first, "isto sjeme, iste naredbe → isti obračun");
}

#[test]
fn resolution_uses_snapshot_not_files() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let mut v = serde_json::to_value(&snap).unwrap();
    for p in ["A", "B", "C"] {
        let f = v["profiles"][p]["fixed_cents"].as_i64().unwrap();
        v["profiles"][p]["fixed_cents"] = serde_json::json!(f + 100_000);
    }
    let changed: Snapshot = serde_json::from_value(v).unwrap();
    let log_of = |s: &Snapshot| {
        let mut log: Vec<Recorded> =
            new_game(GameId(Uuid::from_u128(1)), 42, s, GUEST, hotel_ids())
                .unwrap()
                .into_iter()
                .zip(1..)
                .map(|((actor, event), seq)| Recorded { seq, actor, event })
                .collect();
        let g = Game::fold(&log).unwrap();
        push(
            &mut log,
            Actor::Guest(GUEST),
            g.close_round(Month::Jun).unwrap().unwrap(),
        );
        Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap()
    };
    let a = log_of(&snap);
    let b = log_of(&changed);
    // fiksni troškovi dolaze iz snimka igre, ne iz datoteka u content/hotel
    assert_eq!(
        b.hotels[0].report.before.fixed_cents,
        a.hotels[0].report.before.fixed_cents + 100_000
    );
    assert_ne!(
        b.hotels[0].report.money.gop_cents,
        a.hotels[0].report.money.gop_cents
    );
}

#[test]
fn june_resolution_opens_july_with_robot_moves() {
    let mut log = june_closed(42);
    let before = log.len();
    run_job(&mut log, Month::Jun);
    let kinds: Vec<&str> = log[before..].iter().map(|r| r.event.kind()).collect();
    assert_eq!(kinds[0], "RoundResolved");
    assert_eq!(kinds.len(), 1 + 3 * 2, "svaki robot: PlanSet + S2");
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_round(), Some(RoundKey::Month(Month::Jul)));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Open));
    let GameEvent::RoundResolved(rr) = &log[before].event else {
        panic!()
    };
    assert_eq!(
        HotelStateDto::from(&g.state_at(me(), Month::Jul).unwrap().summary()),
        rr.hotels[0].report.after
    );
}

#[test]
fn resolve_job_twice_is_noop() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    assert_eq!(
        Game::fold(&log)
            .unwrap()
            .resolve_job(Month::Jun, false)
            .unwrap(),
        ResolveStep::Done(vec![])
    );
}

#[test]
fn resolve_job_for_open_round_is_corrupt() {
    let g = Game::fold(&created(42)).unwrap();
    assert!(matches!(
        g.resolve_job(Month::Jun, false),
        Err(GameError::Corrupt(_))
    ));
    assert!(matches!(
        g.resolve_job(Month::Jul, false),
        Err(GameError::Corrupt(_))
    ));
}

#[test]
fn july_resolution_finishes_game() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.close_round(Month::Jul).unwrap().unwrap(),
    );
    let before = log.len();
    run_job(&mut log, Month::Jul);
    assert_eq!(
        log.len(),
        before + 1,
        "nakon srpnja nema novih poteza robota"
    );
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_round(), None);
    assert_eq!(g.close_round(Month::Jul), Ok(None));
    assert_eq!(g.public_hotel(me()).unwrap().months.len(), 2);
    assert_eq!(g.private_hotel(me()).unwrap().plan, None, "igra je gotova");
}

#[test]
fn feed_is_chronological_and_only_mine() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let feed = Game::fold(&log).unwrap().feed(me()).unwrap();
    assert!(matches!(feed[0], FeedItem::HotelReceived { seq: 1, .. }));
    assert!(feed.windows(2).all(|w| seq(&w[0]) <= seq(&w[1])));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::AnswerChosen { situation, by: Actor::Guest(_), .. } if situation == "S1")));
    assert!(
        feed.iter().any(
            |f| matches!(f, FeedItem::SituationIgnored { situation, .. } if situation == "S3")
        )
    );
    assert!(feed.iter().any(|f| matches!(
        f,
        FeedItem::MonthResolved {
            round: RoundKey::Month(Month::Jun),
            ..
        }
    )));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::Change { line, .. } if line.cause == CauseDto::Situation("S1".into()))));
    assert!(
        !feed.iter().any(|f| matches!(
            f,
            FeedItem::PlanChanged {
                by: Actor::Robot(_),
                ..
            }
        )),
        "tuđi potezi nisu u mom feedu"
    );
}

fn seq(f: &FeedItem) -> i64 {
    match f {
        FeedItem::HotelReceived { seq, .. }
        | FeedItem::PlanChanged { seq, .. }
        | FeedItem::AnswerChosen { seq, .. }
        | FeedItem::RoundClosed { seq, .. }
        | FeedItem::SituationIgnored { seq, .. }
        | FeedItem::MonthResolved { seq, .. }
        | FeedItem::Change { seq, .. }
        | FeedItem::Response { seq, .. }
        | FeedItem::Verdict { seq, .. }
        | FeedItem::Mentor { seq, .. } => *seq,
    }
}

#[test]
fn views_split_public_and_private() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    let p = g.public_hotel(hotel_ids()[1]).unwrap();
    assert_eq!(
        p.seat,
        game::view::Seat::Robot {
            strategy: "stedisa".into()
        }
    );
    assert_eq!(p.months.len(), 1);
    let mine = g.private_hotel(me()).unwrap();
    let GameEvent::RoundResolved(rr) = &g
        .history
        .iter()
        .rev()
        .find(|r| r.event.kind() == "RoundResolved")
        .unwrap()
        .event
    else {
        panic!()
    };
    assert_eq!(mine.now.cash_cents, rr.hotels[0].report.after.cash_cents);
    assert_eq!(
        mine.last_month.as_ref().unwrap().round,
        RoundKey::Month(Month::Jun)
    );
    let plan = mine.plan.expect("srpanj je otvoren");
    assert_eq!(plan.round, RoundKey::Month(Month::Jul));
}

#[test]
fn verdict_drives_resolution_with_nearest_answer_mechanics() {
    // Slobodan odgovor s presudom (q=3, najbliži odgovor 1) daje ISTI obračun hotela kao ponuđeni odgovor 1 (q=3).
    let free = common::june_resolved_with_free_verdict(3, 1);
    let offered = common::june_resolved_with_offered(1);
    let a = common::my_resolution(&free);
    let b = common::my_resolution(&offered);
    assert_eq!(a.report, b.report);
    assert_eq!(a.indices, b.indices);
    let s1 = a.situations.iter().find(|s| s.code == "S1").unwrap();
    assert!(s1.free_text);
    assert_eq!(s1.quality, 3);
}

#[test]
fn closing_with_pending_verdict_waits_then_goes_neutral() {
    let g = common::june_closed_with_pending_response();
    assert_eq!(g.pending_verdicts(Month::Jun).len(), 1);
    assert!(matches!(
        g.resolve_job(Month::Jun, false).unwrap(),
        ResolveStep::WaitForVerdicts(1)
    ));
    let ResolveStep::Done(evs) = g.resolve_job(Month::Jun, true).unwrap() else {
        panic!()
    };
    let GameEvent::VerdictRecorded(v) = &evs[0].1 else {
        panic!("prvo neutralna presuda")
    };
    assert_eq!(
        (v.quality, v.nearest_answer, v.source, v.review),
        (3, None, AiSource::Fallback, true)
    );
    assert!(matches!(evs[1].1, GameEvent::RoundResolved(_)));
}

#[test]
fn verdict_after_neutral_is_late_and_changes_nothing() {
    let mut g = common::june_closed_with_pending_response();
    let ResolveStep::Done(evs) = g.resolve_job(Month::Jun, true).unwrap() else {
        panic!()
    };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let before = g.resolved[&RoundKey::Month(Month::Jun)].clone();
    let h = common::my_hotel(&g);
    let v = g.validate_verdict("S1", common::draft_all_met(0)).unwrap();
    let ev = g.record_verdict(h, "S1", v, common::meta()).unwrap();
    assert!(matches!(ev, GameEvent::VerdictLate(_)));
    common::apply_ev(&mut g, Actor::System, ev);
    assert_eq!(g.resolved[&RoundKey::Month(Month::Jun)], before);
}

#[test]
fn replay_with_verdicts_is_byte_identical() {
    // Ponovljivost (DEMO §11.5) i uz presude: dnevnik pročitan kroz JSON (kao iz baze) obračuna lipanj bajt-isto.
    let g = common::june_resolved_with_free_verdict_game(4, 0);
    let reread: Vec<Recorded> = g
        .history
        .iter()
        .map(|r| {
            let text = serde_json::to_string(&r.event.to_payload()).unwrap();
            let payload: serde_json::Value = serde_json::from_str(&text).unwrap();
            Recorded {
                seq: r.seq,
                actor: r.actor.clone(),
                event: GameEvent::from_record(r.event.kind(), 1, payload).unwrap(),
            }
        })
        .collect();
    let pos = reread
        .iter()
        .position(|r| matches!(r.event, GameEvent::RoundResolved(_)))
        .unwrap();
    let before = Game::fold(&reread[..pos]).unwrap();
    let again = before.resolve_round(Month::Jun).unwrap();
    let stored = Game::fold(&reread).unwrap().resolved[&RoundKey::Month(Month::Jun)].clone();
    assert_eq!(
        serde_json::to_vec(&again).unwrap(),
        serde_json::to_vec(&stored).unwrap()
    );
}

#[test]
fn fallback_debrief_mentions_progress_and_situations() {
    let g = common::june_resolved_game();
    let input = g.debrief_input(common::my_hotel(&g), Month::Jun).unwrap();
    let text = fallback_debrief(&input);
    assert!(text.contains("lipanj") || text.contains("lipnju"));
    assert!(!text.is_empty() && text.chars().count() < 1200);
}
