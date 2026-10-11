mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

fn v2_events(kind: PeriodKind) -> Vec<Recorded> {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings { kind },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect()
}

fn play_all(mut g: Game) -> Game {
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("čeka")
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

#[test]
fn v2_weekly_game_has_17_rounds_and_resolves_all_with_robots() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    assert_eq!(g.model(), Model::V2);
    assert_eq!(g.rounds.len(), 17);
    assert_eq!(g.rounds_info()[4].period.days, 7);
    let g = play_all(g);
    assert_eq!(g.resolved.len(), 17);
    let last = g.resolved.values().last().unwrap();
    assert!(last.hotels.iter().all(|h| h.detail.is_some()));
}

#[test]
fn v2_monthly_game_has_four_rounds() {
    let g = play_all(Game::fold(&v2_events(PeriodKind::Month)).unwrap());
    assert_eq!(g.resolved.len(), 4);
}

#[test]
fn v1_with_weeks_is_invalid_settings() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let r = new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind: PeriodKind::Week,
        },
    );
    assert_eq!(r.unwrap_err(), GameError::InvalidSettings);
}

#[test]
fn v2_plan_carries_over_without_renovation() {
    let mut g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r1 = g.current_round().unwrap();
    let CorePlan::V2(mut p) = g.effective_plan(h, r1).unwrap() else {
        panic!("v2")
    };
    assert_eq!(p.marketing_cents, 0); // zadani plan: marketing 0 — pa 300 000 u t02 može doći samo prenošenjem
    p.marketing_cents = 300_000; // ≤ najviše za tjedan: prorate(4 000 000, t01) = 933 333
    p.renovation = true;
    let ev = g.set_plan(h, r1, CorePlan::V2(p.clone())).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(r1).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r1, false).unwrap() else {
        panic!()
    };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let r2 = g.current_round().unwrap();
    let CorePlan::V2(carried) = g.effective_plan(h, r2).unwrap() else {
        panic!("v2")
    };
    assert_eq!(carried.marketing_cents, 300_000); // preneseno iz t01 (zadani plan bi dao 0)
    assert!(!carried.renovation);
}

#[test]
fn v1_round_without_plan_still_uses_the_default_plan() {
    let (mut g, h) = common::open_june_game();
    let mut p = match g.effective_plan(h, sim_hotel::Month::Jun).unwrap() {
        CorePlan::V1(p) => p,
        CorePlan::V2(_) => panic!("v1"),
    };
    // Marketing, ne plaća: v1 zadani plan kopira `state.wage_pct` (crates/sim-hotel/src/plan.rs:75), koji obračun lipnja
    // postavi iz plana (resolve.rs:134), pa plaća ne razlikuje „zadano" od „preneseno". Zadani marketing je uvijek 0.
    p.marketing_cents = 500_000;
    let ev = g.set_plan(h, sim_hotel::Month::Jun, p).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(sim_hotel::Month::Jun, false).unwrap() else {
        panic!()
    };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let CorePlan::V1(jul) = g.effective_plan(h, sim_hotel::Month::Jul).unwrap() else {
        panic!()
    };
    assert_eq!(jul.marketing_cents, 0); // v1: zadani plan, ne prošlomjesečni (koji bi dao 500 000)
}

#[test]
fn v2_game_from_other_core_version_is_outdated() {
    let mut log = v2_events(PeriodKind::Week);
    let GameEvent::GameCreated(c) = &mut log[0].event else {
        panic!()
    };
    c.core_version = "2.0.0-staro".into();
    assert!(matches!(Game::fold(&log), Err(GameError::Outdated { .. })));
}

#[test]
fn v1_game_is_never_outdated_by_v2_version() {
    let (g, _) = common::open_june_game();
    assert_eq!(g.model(), Model::V1);
    assert_eq!(g.core_version, sim_hotel::CORE_VERSION);
}

#[test]
fn v2_situations_appear_in_the_round_containing_the_first_of_the_month() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let in_round = |code: &str| g.situations_in(RoundKey::parse(code).unwrap()).len();
    assert!(in_round("t01") > 0); // lipanj
    assert_eq!(in_round("t02"), 0);
    assert!(in_round("t05") > 0); // srpanj
}

#[test]
fn v2_weekly_situation_is_answered_in_its_round() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let code = g.situations_in(RoundKey::Week(1))[0].code.clone();
    let (round, _) = g.situation_round(&code).unwrap();
    assert_eq!(round, RoundKey::Week(1));
    let GameEvent::AnswerChosen(a) = g.choose_answer(h, &code, 0).unwrap() else {
        panic!("AnswerChosen")
    };
    assert_eq!(a.month, RoundKey::Week(1)); // JSON ime polja ostaje `month`, vrijednost "t01"
    assert!(matches!(
        g.situation_round("NEMA"),
        Err(GameError::UnknownSituation)
    ));
}

#[test]
fn dispatch_lives_in_one_place() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap() == "model.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for needle in [
            "Model::V1",
            "Model::V2",
            "CoreState::V1",
            "CoreState::V2",
            "CorePlan::V1",
            "CorePlan::V2",
            "CoreRefs::",
        ] {
            assert!(
                !text.contains(needle),
                "{} sadrži {needle} — grananje ide u model.rs",
                path.display()
            );
        }
    }
    // ZP-7: ni app ni api ne granaju na model — ni na `content::Model` (grane `=>`) ni na kod modela.
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut dirs = vec![crates.join("app/src"), crates.join("api/src")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for needle in [
                "Model::V1 =>",
                "Model::V2 =>",
                "Model::V1 |",
                "Model::V2 |",
                "code() == \"v1\"",
                "code() == \"v2\"",
                "code() != \"v1\"",
                "code() != \"v2\"",
            ] {
                assert!(
                    !text.contains(needle),
                    "{} sadrži {needle} — grananje ide u game/src/model.rs",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn v2_plan_dto_without_departments_takes_them_from_the_effective_plan() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let CorePlan::V2(eff) = g.effective_plan(h, r).unwrap() else {
        panic!("v2")
    };
    let mut dto = CorePlan::V2(eff.clone()).to_dto();
    assert!(!dto.departments.is_empty());
    dto.departments.clear();
    let CorePlan::V2(p) = g.plan_from_dto(h, r, &dto).unwrap() else {
        panic!("v2")
    };
    assert_eq!(p.departments, eff.departments);
}

/// v2 tijelo plana bez odjela (API do Taska 15): ukupni sezonci i održavanje iz tijela se ne gube (popravak 1).
fn v2_round_one() -> (Game, HotelId, RoundKey, sim_hotel_v2::PeriodPlan) {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let CorePlan::V2(eff) = g.effective_plan(h, r).unwrap() else {
        panic!("v2")
    };
    (g, h, r, eff)
}

fn without_departments(p: &sim_hotel_v2::PeriodPlan) -> dto::PlanDto {
    let mut dto = CorePlan::V2(p.clone()).to_dto();
    dto.departments.clear();
    dto
}

#[test]
fn v2_plan_dto_without_departments_keeps_the_seasonal_total() {
    let (g, h, r, eff) = v2_round_one();
    let mut dto = without_departments(&eff);
    dto.seasonal_target = eff.seasonal_total() + 1;
    let plan = g.plan_from_dto(h, r, &dto).unwrap();
    let CorePlan::V2(p) = plan.clone() else {
        panic!("v2")
    };
    assert_eq!(p.seasonal_total(), eff.seasonal_total() + 1);
    for (k, d) in &p.departments {
        assert_eq!(d.open, eff.departments[k].open, "{}", k.as_str());
        assert_eq!(d.price_level_pct, eff.departments[k].price_level_pct);
    }
    g.set_plan(h, r, plan).unwrap();
}

#[test]
fn v2_plan_dto_without_departments_keeps_the_maintenance_level() {
    let (g, h, r, eff) = v2_round_one();
    let mut dto = without_departments(&eff);
    assert_eq!(dto.maintenance, dto::MaintenanceCode::Regular);
    dto.maintenance = dto::MaintenanceCode::Intensive;
    let CorePlan::V2(p) = g.plan_from_dto(h, r, &dto).unwrap() else {
        panic!("v2")
    };
    assert!(
        p.departments
            .values()
            .all(|d| d.maintenance == sim_hotel::Maintenance::Intensive)
    );
    assert_eq!(p.seasonal_total(), eff.seasonal_total());
}

#[test]
fn v2_plan_dto_without_departments_beyond_the_step_is_the_step_error() {
    let (g, h, r, eff) = v2_round_one();
    let CoreState::V2(s) = g.state_at(h, r).unwrap() else {
        panic!("v2")
    };
    let now: u16 = s.departments.values().map(|d| d.seasonal).sum();
    let mut dto = without_departments(&eff);
    dto.seasonal_target = now + 6; // tjedni korak je 5
    let plan = g.plan_from_dto(h, r, &dto).unwrap();
    assert_eq!(
        g.set_plan(h, r, plan),
        Err(GameError::PlanV2(
            sim_hotel_v2::PlanError::SeasonalStepTooBig
        ))
    );
}
