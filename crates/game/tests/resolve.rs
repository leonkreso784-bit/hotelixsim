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
    for (a, e) in Game::fold(log).unwrap().resolve_job(m).unwrap() {
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
            quality: 5
        }
    );
    assert_eq!(
        mine.situations[1],
        SituationResult {
            code: "S3".into(),
            answer: None,
            quality: 1
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
    assert_eq!(g.current_month(), Some(Month::Jul));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Open));
    let GameEvent::RoundResolved(rr) = &log[before].event else {
        panic!()
    };
    assert_eq!(
        HotelStateDto::from(&g.state_at(me(), Month::Jul).unwrap()),
        rr.hotels[0].report.after
    );
}

#[test]
fn resolve_job_twice_is_noop() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    assert_eq!(
        Game::fold(&log).unwrap().resolve_job(Month::Jun).unwrap(),
        vec![]
    );
}

#[test]
fn resolve_job_for_open_round_is_corrupt() {
    let g = Game::fold(&created(42)).unwrap();
    assert!(matches!(
        g.resolve_job(Month::Jun),
        Err(GameError::Corrupt(_))
    ));
    assert!(matches!(
        g.resolve_job(Month::Jul),
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
    assert_eq!(g.current_month(), None);
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
            month: MonthCode::Jun,
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
        | FeedItem::Change { seq, .. } => *seq,
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
    assert_eq!(mine.last_month.as_ref().unwrap().month, MonthCode::Jun);
    let plan = mine.plan.expect("srpanj je otvoren");
    assert_eq!(plan.month, MonthCode::Jul);
}
