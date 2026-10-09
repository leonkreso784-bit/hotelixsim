mod common;
use common::*;
use game::command::*;
use game::dto::*;
use game::*;
use sim_hotel::{Month, MonthPlan, PlanError};

fn me() -> HotelId {
    hotel_ids()[0]
}

#[test]
fn new_game_writes_created_then_robot_moves_for_june() {
    let log = created(42);
    let kinds: Vec<(&str, String)> = log
        .iter()
        .map(|r| (r.event.kind(), r.actor.as_db()))
        .collect();
    assert_eq!(kinds[0], ("GameCreated", "system".into()));
    assert_eq!(
        log.len(),
        1 + 3 * 3,
        "svaki robot: PlanSet + odgovor na S1 i S3"
    );
    for (i, robot) in ["robot:stedisa", "robot:brizni", "robot:lovac"]
        .iter()
        .enumerate()
    {
        let k: Vec<&str> = log[1 + i * 3..4 + i * 3]
            .iter()
            .map(|r| r.event.kind())
            .collect();
        assert_eq!(k, ["PlanSet", "AnswerChosen", "AnswerChosen"], "{robot}");
        assert!(
            log[1 + i * 3..4 + i * 3]
                .iter()
                .all(|r| r.actor.as_db() == *robot)
        );
    }
    let GameEvent::GameCreated(c) = &log[0].event else {
        panic!()
    };
    assert_eq!(c.ruleset_id, content::fingerprint_value(&c.snapshot));
    assert_eq!(c.months, [MonthCode::Jun, MonthCode::Jul]);
    let mut robot_profiles: Vec<&str> = c.hotels[1..].iter().map(|h| h.profile.as_str()).collect();
    robot_profiles.sort_unstable();
    assert_eq!(
        robot_profiles,
        ["A", "B", "C"],
        "roboti pokrivaju sva tri profila"
    );
    assert!(["A", "B", "C"].contains(&c.hotels[0].profile.as_str()));
    assert_eq!(
        c.hotels[0].controller,
        Controller::Human { guest_id: GUEST }
    );
}

#[test]
fn statuses_follow_the_calendar() {
    let g = Game::fold(&created(42)).unwrap();
    assert_eq!(g.current_month(), Some(Month::Jun));
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Open));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Scheduled));
    assert_eq!(g.status(Month::Aug), Err(GameError::UnknownMonth));
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(2));
    assert_eq!(g.unanswered_count(hotel_ids()[1], Month::Jun), Ok(0));
}

#[test]
fn untouched_plan_is_default_and_last_plan_wins() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let default = MonthPlan::default_for(&s, Month::Jun, &g.content.catalog).unwrap();
    assert_eq!(g.effective_plan(me(), Month::Jun).unwrap(), default);
    let mut p1 = default.clone();
    p1.marketing_cents = 500_000;
    let ev = g.set_plan(me(), Month::Jun, p1.clone()).unwrap();
    push(&mut log, Actor::Guest(GUEST), ev);
    let mut p2 = default.clone();
    p2.wage_pct = 105;
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.set_plan(me(), Month::Jun, p2.clone()).unwrap(),
    );
    assert_eq!(
        Game::fold(&log)
            .unwrap()
            .effective_plan(me(), Month::Jun)
            .unwrap(),
        p2
    );
}

#[test]
fn invalid_plan_is_rejected_with_plan_error() {
    let g = Game::fold(&created(42)).unwrap();
    let mut p = g.effective_plan(me(), Month::Jun).unwrap();
    p.price_cents += 50;
    assert_eq!(
        g.set_plan(me(), Month::Jun, p),
        Err(GameError::Plan(PlanError::PriceNotWholeEuro))
    );
}

#[test]
fn answers_once_per_situation() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(
        &mut log,
        Actor::Guest(GUEST),
        g.choose_answer(me(), "S1", 0).unwrap(),
    );
    let g = Game::fold(&log).unwrap();
    assert_eq!(
        g.choose_answer(me(), "S1", 1),
        Err(GameError::AlreadyAnswered)
    );
    assert_eq!(
        g.choose_answer(me(), "S3", 9),
        Err(GameError::UnknownAnswer)
    );
    assert_eq!(
        g.choose_answer(me(), "S2", 0),
        Err(GameError::RoundNotOpen),
        "S2 je srpanj"
    );
    assert_eq!(
        g.choose_answer(me(), "S4", 0),
        Err(GameError::UnknownSituation),
        "kolovoz nije u igri"
    );
    assert_eq!(
        g.choose_answer(me(), "S9", 0),
        Err(GameError::UnknownSituation)
    );
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(1));
}

#[test]
fn stale_commands_are_rejected() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let ev = g
        .close_round(Month::Jun)
        .unwrap()
        .expect("otvorena runda se zatvara");
    push(&mut log, Actor::Guest(GUEST), ev);
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Closing));
    assert_eq!(
        g.close_round(Month::Jun),
        Ok(None),
        "drugi klik ne piše ništa"
    );
    let p = g.effective_plan(me(), Month::Jun).unwrap();
    assert_eq!(
        g.set_plan(me(), Month::Jun, p),
        Err(GameError::RoundNotOpen)
    );
    assert_eq!(g.choose_answer(me(), "S1", 0), Err(GameError::RoundNotOpen));
    assert_eq!(
        g.close_round(Month::Jul),
        Err(GameError::RoundNotOpen),
        "srpanj još nije otvoren"
    );
}

#[test]
fn fold_rejects_broken_logs() {
    let mut log = created(42);
    log[3].seq = 99;
    assert!(matches!(Game::fold(&log), Err(GameError::Corrupt(_))));
    let mut log = created(42);
    let GameEvent::GameCreated(c) = &mut log[0].event else {
        panic!()
    };
    let mut v = serde_json::to_value(&c.snapshot).unwrap();
    v["catalog"]["economy"]["hire_cents"] = serde_json::json!(1);
    c.snapshot = serde_json::from_value(v).unwrap(); // ruleset_id ostaje stari
    match Game::fold(&log) {
        Err(GameError::Corrupt(m)) => assert!(m.contains("otisak"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn plan_limits_are_exactly_the_validation_bounds() {
    let g = Game::fold(&created(42)).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let cat = &g.content.catalog;
    let l = plan_limits(&s, Month::Jun, cat).unwrap();
    let base = MonthPlan::default_for(&s, Month::Jun, cat).unwrap();
    let with = |f: &dyn Fn(&mut MonthPlan)| {
        let mut p = base.clone();
        f(&mut p);
        p.validate(&s, Month::Jun, cat)
    };
    assert_eq!(with(&|p| p.price_cents = l.price_min_cents), Ok(()));
    assert_eq!(
        with(&|p| p.price_cents = l.price_min_cents - 100),
        Err(PlanError::PriceOutOfRange)
    );
    assert_eq!(with(&|p| p.price_cents = l.price_max_cents), Ok(()));
    assert_eq!(
        with(&|p| p.price_cents = l.price_max_cents + 100),
        Err(PlanError::PriceOutOfRange)
    );
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_min), Ok(()));
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_max), Ok(()));
    assert!(with(&|p| p.seasonal_target = l.seasonal_max + 1).is_err());
    assert_eq!(with(&|p| p.wage_pct = l.wage_max_pct), Ok(()));
    assert_eq!(
        with(&|p| p.wage_pct = l.wage_max_pct + 1),
        Err(PlanError::WageOutOfRange)
    );
    assert_eq!(with(&|p| p.marketing_cents = l.marketing_max_cents), Ok(()));
    assert!(l.renovation_available);
}

#[test]
fn fold_rejects_unknown_profile_and_strategy_codes() {
    let mut log = created(42);
    let GameEvent::GameCreated(c) = &mut log[0].event else {
        panic!()
    };
    c.hotels[0].profile = "Z".into();
    match Game::fold(&log) {
        Err(GameError::Corrupt(m)) => assert!(m.contains("profil"), "{m}"),
        other => panic!("{other:?}"),
    }
    let mut log = created(42);
    let GameEvent::GameCreated(c) = &mut log[0].event else {
        panic!()
    };
    c.hotels[1].controller = Controller::Robot {
        strategy: "nepoznata".into(),
    };
    match Game::fold(&log) {
        Err(GameError::Corrupt(m)) => assert!(m.contains("strategija"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn resolve_refuses_other_core_version() {
    let mut g = closed_june_game();
    g.core_version = "0.0.0-drugi".into();
    assert!(matches!(
        g.resolve_job(Month::Jun, false),
        Err(GameError::CoreMismatch(_))
    ));
}

#[test]
fn resolved_round_is_done_even_with_other_core_version() {
    let mut g = closed_june_game();
    let ResolveStep::Done(evs) = g.resolve_job(Month::Jun, true).unwrap() else {
        panic!("očekivano Done")
    };
    for (actor, event) in evs {
        let seq = g.last_seq + 1;
        g.apply(Recorded { seq, actor, event }).unwrap();
    }
    g.core_version = "0.0.0-drugi".into();
    assert_eq!(
        g.resolve_job(Month::Jun, false),
        Ok(ResolveStep::Done(Vec::new()))
    );
}
