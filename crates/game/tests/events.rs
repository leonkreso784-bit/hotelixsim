mod common;
use content::{Snapshot, repo_dir};
use game::dto::*;
use game::*;
use serde_json::json;
use sim_hotel::{HotelState, Month, MonthPlan, Profile, Strategy, resolve_month};
use uuid::Uuid;

fn ids() -> (GameId, HotelId, GuestId) {
    (
        GameId(Uuid::from_u128(1)),
        HotelId(Uuid::from_u128(2)),
        GuestId(Uuid::from_u128(3)),
    )
}

fn plan() -> PlanDto {
    PlanDto {
        price_cents: 14_500,
        marketing_cents: 500_000,
        seasonal_target: 18,
        wage_pct: 103,
        training: TrainingCode::Basic,
        maintenance: MaintenanceCode::Regular,
        renovation: false,
        reply_reviews: true,
    }
}

fn june_report() -> (HotelState, sim_hotel::MonthReport) {
    let c = Snapshot::read(&repo_dir()).unwrap().build().unwrap();
    let s = HotelState::from_profile(c.catalog.profile(Profile::C).unwrap()).unwrap();
    let p = MonthPlan::default_for(&s, Month::Jun, &c.catalog).unwrap();
    let r = resolve_month(&s, Month::Jun, &p, &[], &c.catalog).unwrap();
    (s, r)
}

fn samples() -> Vec<GameEvent> {
    let (g, h, guest) = ids();
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let (_, r) = june_report();
    vec![
        GameEvent::GameCreated(GameCreated {
            game_id: g,
            seed: u64::MAX,
            ruleset_id: snap.fingerprint(),
            core_version: sim_hotel::CORE_VERSION.into(),
            snapshot: Box::new(snap.to_value()),
            months: vec![MonthCode::Jun, MonthCode::Jul],
            hotels: vec![
                HotelSetup {
                    hotel_id: h,
                    slot: 1,
                    name: "Tvoj hotel".into(),
                    profile: "C".into(),
                    controller: Controller::Human { guest_id: guest },
                },
                HotelSetup {
                    hotel_id: HotelId(Uuid::from_u128(4)),
                    slot: 2,
                    name: "Štediša".into(),
                    profile: "C".into(),
                    controller: Controller::Robot {
                        strategy: "stedisa".into(),
                    },
                },
            ],
        }),
        GameEvent::PlanSet(PlanSet {
            hotel_id: h,
            month: MonthCode::Jun,
            plan: plan(),
        }),
        GameEvent::AnswerChosen(AnswerChosen {
            hotel_id: h,
            month: MonthCode::Jun,
            situation: "S1".into(),
            answer: 2,
        }),
        GameEvent::RoundClosed(RoundClosed {
            month: MonthCode::Jun,
        }),
        GameEvent::RoundResolved(RoundResolved {
            month: MonthCode::Jun,
            hotels: vec![HotelResolution {
                hotel_id: h,
                plan: plan(),
                situations: vec![
                    SituationResult {
                        code: "S1".into(),
                        answer: Some(2),
                        quality: 1,
                        free_text: false,
                    },
                    SituationResult {
                        code: "S3".into(),
                        answer: None,
                        quality: 1,
                        free_text: false,
                    },
                ],
                report: MonthReportDto::from(&r),
                indices: [5000, 5100, 4900, 5000, 5200],
                progress_hundredths: -37,
            }],
        }),
    ]
}

#[test]
fn every_event_roundtrips_through_record() {
    for e in samples() {
        let back = GameEvent::from_record(e.kind(), SCHEMA_VER, e.to_payload()).unwrap();
        assert_eq!(back, e, "{}", e.kind());
    }
}

#[test]
fn record_kinds_are_stable_names() {
    let kinds: Vec<&str> = samples().iter().map(GameEvent::kind).collect();
    assert_eq!(
        kinds,
        [
            "GameCreated",
            "PlanSet",
            "AnswerChosen",
            "RoundClosed",
            "RoundResolved"
        ]
    );
}

#[test]
fn bad_records_are_rejected() {
    let e = &samples()[1];
    assert!(matches!(
        GameEvent::from_record("Nepoznat", 1, e.to_payload()),
        Err(EventError::UnknownKind(_))
    ));
    assert!(matches!(
        GameEvent::from_record("PlanSet", 2, e.to_payload()),
        Err(EventError::UnsupportedVersion { .. })
    ));
    let mut p = e.to_payload();
    p["visak"] = json!(1);
    assert!(matches!(
        GameEvent::from_record("PlanSet", 1, p),
        Err(EventError::Payload { .. })
    ));
}

#[test]
fn actor_roundtrip_and_rejects() {
    let (_, _, guest) = ids();
    let mut all = vec![Actor::Guest(guest), Actor::System];
    all.extend(Strategy::ALL.iter().map(|s| Actor::Robot(*s)));
    for a in all {
        assert_eq!(Actor::parse(&a.as_db()), Some(a.clone()), "{}", a.as_db());
    }
    assert_eq!(
        Actor::Guest(guest).as_db(),
        "guest:00000000-0000-0000-0000-000000000003"
    );
    assert_eq!(Actor::Robot(Strategy::Saver).as_db(), "robot:stedisa");
    for bad in ["", "guest:xyz", "robot:nepoznat", "System", "guest:"] {
        assert_eq!(Actor::parse(bad), None, "{bad}");
    }
}

#[test]
fn state_and_plan_dtos_roundtrip_to_core() {
    let (s, r) = june_report();
    assert_eq!(HotelState::try_from(&HotelStateDto::from(&s)).unwrap(), s);
    assert_eq!(
        HotelState::try_from(&HotelStateDto::from(&r.after)).unwrap(),
        r.after
    );
    let p = MonthPlan::from(&plan());
    assert_eq!(PlanDto::from(&p), plan());
}

#[test]
fn report_dto_survives_json() {
    let (_, r) = june_report();
    let d = MonthReportDto::from(&r);
    let back: MonthReportDto = serde_json::from_value(serde_json::to_value(&d).unwrap()).unwrap();
    assert_eq!(back, d);
    assert_eq!(
        serde_json::to_value(CauseDto::Situation("S1".into())).unwrap(),
        json!({"situation": "S1"})
    );
    assert_eq!(
        serde_json::to_value(CauseDto::Price).unwrap(),
        json!("price")
    );
}

#[test]
fn legacy_game_still_folds() {
    // Igra iz koraka 3: ruleset_id je stari otisak tipiziranog snimka.
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let mut events = common::new_game_events(&snap);
    let GameEvent::GameCreated(c) = &mut events[0].event else {
        panic!()
    };
    c.ruleset_id = snap.legacy_fingerprint();
    assert!(Game::fold(&events).is_ok());
}

#[test]
fn shuffled_jsonb_keys_still_fold() {
    // JSONB vraća ključeve drugim redom; otisak nad Value to ne smije primijetiti.
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let events = common::new_game_events(&snap);
    let payload = events[0].event.to_payload();
    let text = serde_json::to_string(&payload).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let ev = GameEvent::from_record("GameCreated", 1, reparsed).unwrap();
    let mut again = events.clone();
    again[0].event = ev;
    assert!(Game::fold(&again).is_ok());
}

#[test]
fn tampered_snapshot_is_corrupt() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let mut events = common::new_game_events(&snap);
    let GameEvent::GameCreated(c) = &mut events[0].event else {
        panic!()
    };
    c.snapshot["catalog"]["economy"]["hire_cents"] = json!(1);
    assert!(matches!(Game::fold(&events), Err(GameError::Corrupt(_))));
}

#[test]
fn old_snapshot_without_ai_fields_still_folds() {
    // Snimak iz koraka 3 nema rubric/character/free_text/criteria; mora se čitati i dati stari otisak.
    let snap = content::Snapshot::read(&content::repo_dir()).unwrap();
    let mut v = snap.to_value();
    for s in v["situations"].as_array_mut().unwrap() {
        let o = s.as_object_mut().unwrap();
        o.remove("rubric");
        o.remove("character");
        o.remove("free_text");
        for a in o["answers"].as_array_mut().unwrap() {
            a.as_object_mut().unwrap().remove("criteria");
        }
    }
    let old = content::Snapshot::from_value(&v).unwrap();
    let mut events = common::new_game_events(&old);
    let GameEvent::GameCreated(c) = &mut events[0].event else {
        panic!()
    };
    c.ruleset_id = old.legacy_fingerprint();
    let g = Game::fold(&events).unwrap();
    assert!(!g.content.ai["S1"].free_text);
}
