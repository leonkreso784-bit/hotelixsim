//! Obnova v2 u igri (obnova-v2-spec §4, §8 O-3): plan i događaj nose iznose (i robotov plan), tijelo bez iznosa
//! zadržava zaključanu obnovu, granice nude opcije iz jezgre, smjernica FF&E, prikaz hotela i odjela, račun dobiti s
//! amortizacijom i EBIT-om, kraj sezone, stari JSON bez novih polja i zastarjela igra 2.0.0.
mod common;

use common::{GUEST, apply_ev, hotel_ids, play_through, renewal_snapshot, v2_game_from};
use content::{Snapshot, repo_dir};
use game::dto::PlanDto;
use game::dto_v2::{HotelStateV2Dto, PeriodAccountsDto};
use game::renewal::{BuildingView, RenewalStatus, SeasonRenewal};
use game::*;
use serde_json::Value;
use sim_hotel::{Month, Profile};
use sim_hotel_v2::department::{DeptCode, PeriodAccounts};
use sim_hotel_v2::renewal::{RenewalSpec, gain_pts, monthly_depreciation};
use sim_hotel_v2::usali::Section;
use sim_hotel_v2::{HotelState, PeriodKind, PeriodPlan, PlanError};

fn me() -> HotelId {
    hotel_ids()[0]
}

fn week(n: u8) -> RoundKey {
    RoundKey::Week(n)
}

fn plan_v2(g: &Game, h: HotelId, r: RoundKey) -> PeriodPlan {
    let CorePlan::V2(p) = g.effective_plan(h, r).unwrap() else {
        panic!("v2 plan")
    };
    p
}

/// Plan igrača za krug `r` s obnovom odjela (euri) i zgrade (euri), zapisan kao `PlanSet`.
fn renew(g: &mut Game, r: RoundKey, depts: &[(&str, i64)], building_eur: i64) {
    let mut p = plan_v2(g, me(), r);
    for (code, eur) in depts {
        p.departments
            .get_mut(&DeptCode((*code).into()))
            .unwrap()
            .renewal_cents = eur * 100;
    }
    p.building_renewal_cents = building_eur * 100;
    let ev = g.set_plan(me(), r, CorePlan::V2(p)).unwrap();
    apply_ev(g, Actor::Guest(GUEST), ev);
}

fn spec(half_eur: i64, max_eur: i64, step_eur: i64, life: u16) -> RenewalSpec {
    RenewalSpec {
        gain_pts: 5_000,
        half_cents: half_eur * 100,
        max_monthly_cents: max_eur * 100,
        step_cents: step_eur * 100,
        life_months: life,
    }
}

/// Igra s obnovom u kojoj je brižni robot hotel profila B (Smještaj 40 < 60, pa obnavlja u prvom krugu).
fn caring_b_game(kind: PeriodKind) -> (Game, HotelId) {
    let snap = renewal_snapshot();
    for seed in 0..60 {
        let g = v2_game_from(&snap, seed, kind);
        let found = g.hotels.iter().find(|h| {
            h.profile == Profile::B
                && matches!(&h.setup.controller, Controller::Robot { strategy } if strategy == "brizni")
        });
        if let Some(h) = found {
            let id = h.setup.hotel_id;
            return (g, id);
        }
    }
    panic!("nijedno sjeme ne daje brižnog robota s profilom B");
}

// ------------------------------------------------------------------ Important 1: robotov plan kroz događaj

#[test]
fn caring_robot_plan_carries_its_renewal_through_the_event() {
    let (g, robot) = caring_b_game(PeriodKind::Week);
    let first = g.current_round().unwrap();
    // Plan je u igru stigao kroz `PlanSet` (PlanDto) → fold; iznos obnove mora preživjeti.
    let rec = g.hotels.iter().find(|h| h.setup.hotel_id == robot).unwrap();
    let CorePlan::V2(p) = &rec.plans[&first] else {
        panic!("v2 plan")
    };
    let rooms = p.departments[&DeptCode::rooms()].renewal_cents;
    assert!(rooms > 0, "brižni u B obnavlja Smještaj u prvom krugu");
    let dto = rec.plans[&first].to_dto();
    assert_eq!(dto.departments["rooms"].renewal_cents, Some(rooms));
    // DTO → JSON → DTO → plan: isti plan.
    let json = serde_json::to_value(&dto).unwrap();
    assert_eq!(json["departments"]["rooms"]["renewal_cents"], rooms);
    let back: PlanDto = serde_json::from_value(json).unwrap();
    assert_eq!(PeriodPlan::try_from(&back).unwrap(), *p);
}

#[test]
fn weekly_game_with_caring_robot_renews_and_resolves_every_round() {
    let (g, robot) = caring_b_game(PeriodKind::Week);
    let first = g.current_round().unwrap();
    let rec = g.hotels.iter().find(|h| h.setup.hotel_id == robot).unwrap();
    let CorePlan::V2(p) = &rec.plans[&first] else {
        panic!("v2 plan")
    };
    let decided = p.departments[&DeptCode::rooms()].renewal_cents;
    // Cijela sezona se obračuna (zaključana obnova bez iznosa u spremljenom planu bila bi RenewalLocked → pokvaren dnevnik).
    let g = play_through(g, "all");
    assert_eq!(g.resolved.len(), 17);
    let rooms = g.department(robot, "rooms").unwrap();
    let paid: i64 = rooms.history.iter().map(|(_, o)| o.renewal_cents).sum();
    assert_eq!(paid, decided, "plaćeno je točno odlučeno, jednom u sezoni");
    let june: i64 = rooms
        .history
        .iter()
        .filter(|(r, _)| matches!(r, RoundKey::Week(1..=5)))
        .map(|(_, o)| o.renewal_cents)
        .sum();
    assert_eq!(june, decided, "sve plaćeno u lipnju");
    assert!(rooms.history.iter().any(|(_, o)| o.renewal_pts > 0));
    let pl = g.profit_and_loss(robot, None, PlScope::Season).unwrap();
    assert!(pl.actual.capex_cents >= decided);
    assert!(
        pl.actual.depreciation_cents > 0,
        "lipanjska obnova amortizira se od srpnja"
    );
}

// ------------------------------------------------------------------ tijelo plana: None = zadrži

#[test]
fn plan_body_without_renewal_keeps_the_locked_amount_and_another_is_locked() {
    let snap = renewal_snapshot();
    let mut g = v2_game_from(&snap, 42, PeriodKind::Week);
    renew(&mut g, week(1), &[("restaurant", 40_000)], 100_000);
    let mut g = play_through(g, "t01");
    let base = plan_v2(&g, me(), week(2));
    // Preneseni plan nosi zaključane iznose.
    assert_eq!(
        base.departments[&DeptCode("restaurant".into())].renewal_cents,
        4_000_000
    );
    assert_eq!(base.building_renewal_cents, 10_000_000);

    // Tijelo s odjelima, ali bez iznosa obnove → zadrži važeći.
    let mut body = CorePlan::V2(base.clone()).to_dto();
    for d in body.departments.values_mut() {
        d.renewal_cents = None;
    }
    body.building_renewal_cents = None;
    let plan = g.plan_from_dto(me(), week(2), &body).unwrap();
    let CorePlan::V2(p) = &plan else { panic!() };
    assert_eq!(
        p.departments[&DeptCode("restaurant".into())].renewal_cents,
        4_000_000
    );
    assert_eq!(p.building_renewal_cents, 10_000_000);
    let ev = g.set_plan(me(), week(2), plan).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);

    // Tijelo bez odjela i bez zgrade → isto.
    let mut bare = body.clone();
    bare.departments.clear();
    let CorePlan::V2(p) = g.plan_from_dto(me(), week(2), &bare).unwrap() else {
        panic!()
    };
    assert_eq!(
        p.departments[&DeptCode("restaurant".into())].renewal_cents,
        4_000_000
    );
    assert_eq!(p.building_renewal_cents, 10_000_000);

    // Drugi iznos u istom mjesecu → RenewalLocked.
    let mut other = body.clone();
    other
        .departments
        .get_mut("restaurant")
        .unwrap()
        .renewal_cents = Some(500_000);
    let plan = g.plan_from_dto(me(), week(2), &other).unwrap();
    assert_eq!(
        g.set_plan(me(), week(2), plan).unwrap_err(),
        GameError::PlanV2(PlanError::RenewalLocked)
    );
    let mut other = body;
    other.building_renewal_cents = Some(0);
    let plan = g.plan_from_dto(me(), week(2), &other).unwrap();
    assert_eq!(
        g.set_plan(me(), week(2), plan).unwrap_err(),
        GameError::PlanV2(PlanError::RenewalLocked)
    );
}

#[test]
fn v1_plan_body_with_building_renewal_is_invalid() {
    let (g, h) = common::open_june_game();
    let mut body = g.effective_plan(h, Month::Jun).unwrap().to_dto();
    assert_eq!(body.building_renewal_cents, None);
    body.building_renewal_cents = Some(0);
    assert_eq!(
        g.plan_from_dto(h, Month::Jun, &body).unwrap_err(),
        GameError::PlanBody
    );
}

// ------------------------------------------------------------------ granice: opcije iz jezgre, zaključano

#[test]
fn limits_offer_renewal_options_computed_by_the_core() {
    let g = play_through(
        v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week),
        "t05",
    );
    assert_eq!(g.current_round(), Some(week(6)));
    let l = g.private_hotel(me()).unwrap().plan.unwrap().limits;
    let depts = g.departments(me()).unwrap();
    let condition = |code: &str| {
        depts
            .departments
            .iter()
            .find(|d| d.code == code)
            .unwrap()
            .condition_pts
    };
    let check = |code: &str,
                 choice: &game::renewal::RenewalChoice,
                 s: &RenewalSpec,
                 cond: i32,
                 n: usize| {
        assert_eq!(choice.month, Month::Jul, "{code}");
        assert_eq!(choice.rounds_left, 4, "{code}: srpanj ima t06–t09");
        assert_eq!(
            (
                choice.max_monthly_cents,
                choice.step_cents,
                choice.life_months
            ),
            (s.max_monthly_cents, s.step_cents, s.life_months)
        );
        assert_eq!(choice.condition_pts, cond);
        assert_eq!(choice.locked, None);
        assert_eq!(choice.options.len(), n, "{code}");
        for (k, o) in choice.options.iter().enumerate() {
            let a = s.step_cents * i64::try_from(k).unwrap();
            assert_eq!(o.amount_cents, a);
            assert_eq!(o.installment_cents, a / 4);
            assert_eq!(o.last_installment_cents, a - 3 * (a / 4));
            let e = gain_pts(s, a).unwrap();
            assert_eq!(o.gain_pts, e);
            assert_eq!(o.gain_capped_pts, e.min(10_000 - cond));
            assert_eq!(
                o.depreciation_monthly_cents,
                monthly_depreciation(a, s.life_months).unwrap()
            );
        }
    };
    let rest = l.departments["restaurant"].renewal.as_ref().unwrap();
    check(
        "restaurant",
        rest,
        &spec(55_000, 50_000, 5_000, 84),
        condition("restaurant"),
        11,
    );
    let rooms = l.departments["rooms"].renewal.as_ref().unwrap();
    check(
        "rooms",
        rooms,
        &spec(150_000, 150_000, 5_000, 84),
        condition("rooms"),
        31,
    );
    let building = l.costs.building_renewal.as_ref().unwrap();
    let b_cond = g
        .private_hotel(me())
        .unwrap()
        .building
        .unwrap()
        .condition_pts;
    check(
        "building",
        building,
        &spec(150_000, 200_000, 10_000, 480),
        b_cond,
        21,
    );
    // Primjer iz tablice §0: Restoran 10.000 € → +7,69; 40.000 € u t06 → 4 × 10.000 €.
    assert_eq!(rest.options[2].gain_pts, 769);
    assert_eq!(rest.options[8].installment_cents, 1_000_000);
    // Bez obnove: neraspoređeni i lokal.
    assert!(l.departments["admin"].renewal.is_none());
    if let Some(lease) = l.departments.get("retail_lease") {
        assert!(lease.renewal.is_none());
    }
}

#[test]
fn locked_renewal_shows_in_limits_departments_and_hotel() {
    let mut g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week);
    renew(&mut g, week(1), &[("restaurant", 40_000)], 100_000);
    let g = play_through(g, "t01");
    // Restoran 40.000 € odlučeno u t01 (lipanj ima 5 krugova): rata 8.000 €, E = 21,05 boda, nakon prve rate 4,21.
    let restaurant = RenewalStatus {
        month: Month::Jun,
        total_cents: 4_000_000,
        installment_cents: 800_000,
        paid_cents: 800_000,
        rounds_left: 4,
        gain_pts: 2_105,
        gained_pts: 421,
    };
    // Zgrada 100.000 €: rata 20.000 €, E = 20 bodova, nakon prve rate 4.
    let building = RenewalStatus {
        month: Month::Jun,
        total_cents: 10_000_000,
        installment_cents: 2_000_000,
        paid_cents: 2_000_000,
        rounds_left: 4,
        gain_pts: 2_000,
        gained_pts: 400,
    };
    let hotel = g.private_hotel(me()).unwrap();
    let l = hotel.plan.unwrap().limits;
    assert_eq!(
        l.departments["restaurant"].renewal.as_ref().unwrap().locked,
        Some(restaurant)
    );
    assert_eq!(
        l.costs.building_renewal.as_ref().unwrap().locked,
        Some(building)
    );
    assert_eq!(
        l.departments["rooms"].renewal.as_ref().unwrap().locked,
        None
    );

    let d = g.department(me(), "restaurant").unwrap().department;
    assert_eq!(d.renewal, Some(restaurant));
    assert_eq!(d.invested_cents, 800_000);
    let start = g.hotel(me()).unwrap().profile;
    let profile_start = content_start_condition(start, "restaurant");
    assert_eq!(d.start_condition_pts, profile_start);
    let rooms = g.department(me(), "rooms").unwrap().department;
    assert_eq!((rooms.renewal, rooms.invested_cents), (None, 0));

    let b = hotel.building.unwrap();
    assert_eq!(b.locked, Some(building));
    assert_eq!(b.invested_cents, 2_000_000);
    assert_eq!(b.wear_pts_per_month, 50);
    let pts = match start {
        Profile::A => 8_000,
        Profile::B => 4_500,
        Profile::C => 6_500,
    };
    assert_eq!(b.start_pts, pts);
}

fn content_start_condition(p: Profile, code: &str) -> i32 {
    let c = renewal_snapshot().build().unwrap().v2.unwrap();
    let s = HotelState::from_profile(&c, p).unwrap();
    s.departments[&DeptCode(code.into())].condition_pts
}

#[test]
fn renewal_guide_is_four_percent_of_the_season_budget() {
    let snap = renewal_snapshot();
    let g = v2_game_from(&snap, 42, PeriodKind::Week);
    let guide = g
        .private_hotel(me())
        .unwrap()
        .plan
        .unwrap()
        .limits
        .costs
        .renewal_guide
        .unwrap();
    let end = play_through(v2_game_from(&snap, 42, PeriodKind::Week), "all");
    let budget = end
        .profit_and_loss(me(), None, PlScope::Season)
        .unwrap()
        .budget
        .total_revenue_cents;
    assert_eq!(guide.reserve_bp, 400);
    assert_eq!(guide.season_revenue_budget_cents, budget);
    assert_eq!(guide.guide_cents, (budget * 400 + 5_000) / 10_000);
    assert_eq!(guide.invested_cents, 0);
    // Nakon ulaganja smjernica broji uloženo (plaćeno do sada).
    let mut g = g;
    renew(&mut g, week(1), &[("restaurant", 40_000)], 100_000);
    let g = play_through(g, "t01");
    let guide = g
        .private_hotel(me())
        .unwrap()
        .plan
        .unwrap()
        .limits
        .costs
        .renewal_guide
        .unwrap();
    assert_eq!(guide.invested_cents, 800_000 + 2_000_000);
}

#[test]
fn content_without_renewal_has_no_renewal_choices_but_a_building() {
    let g = v2_game_from(&common::no_renewal_snapshot(), 42, PeriodKind::Week);
    let hotel = g.private_hotel(me()).unwrap();
    let l = hotel.plan.unwrap().limits;
    assert!(l.departments.values().all(|d| d.renewal.is_none()));
    assert!(l.costs.building_renewal.is_none());
    assert!(l.costs.renewal_guide.is_none());
    assert_eq!(
        hotel.building,
        Some(BuildingView {
            condition_pts: 10_000,
            start_pts: 10_000,
            wear_pts_per_month: 0,
            locked: None,
            invested_cents: 0,
        })
    );
    let (v1, h) = common::open_june_game();
    let v1_hotel = v1.private_hotel(h).unwrap();
    assert_eq!(v1_hotel.building, None);
    assert!(v1_hotel.plan.unwrap().limits.costs.renewal_guide.is_none());
}

#[test]
fn rooms_maintenance_costs_come_from_the_rooms_department() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["departments"]["rooms"]["maintenance"]["regular"]["cost_cents"] = 2_000_000.into();
    let snap = Snapshot::from_value(&v).unwrap();
    let g = v2_game_from(&snap, 42, PeriodKind::Week);
    let l = g.private_hotel(me()).unwrap().plan.unwrap().limits;
    // t01 = 7 od 30 lipanjskih dana: ⌊2.000.000 · 7 / 30⌋.
    assert_eq!(l.costs.maintenance.regular_cents, 466_666);
    assert_eq!(l.departments["rooms"].maintenance, l.costs.maintenance);
}

// ------------------------------------------------------------------ račun dobiti: amortizacija i EBIT

#[test]
fn pl_has_depreciation_and_ebit_after_ebitda_and_season_sums_them() {
    let mut g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week);
    renew(&mut g, week(1), &[("rooms", 150_000)], 0);
    let g = play_through(g, "t06");
    let t06 = g
        .profit_and_loss(me(), Some(week(6)), PlScope::Round)
        .unwrap();
    let sections: Vec<Section> = t06.lines.iter().map(|l| l.section).collect();
    let tail: Vec<(Section, &str)> = t06
        .lines
        .iter()
        .skip_while(|l| l.section != Section::Ebitda)
        .map(|l| (l.section, l.code.as_str()))
        .collect();
    assert_eq!(
        tail,
        [
            (Section::Ebitda, "ebitda"),
            (Section::Depreciation, "equipment"),
            (Section::Depreciation, "building"),
            (Section::Depreciation, "total"),
            (Section::Ebit, "ebit"),
        ]
    );
    assert!(sections.windows(2).all(|w| w[0] <= w[1]), "sekcije redom");
    // Smještaj 150.000 € u lipnju: 178.571 € / 100 mjesečno od srpnja; t06 = 6.–12. srpnja (7 od 31 dana).
    let monthly = 178_571_i64;
    let t06_dep = monthly * 12 / 31 - monthly * 5 / 31;
    let line = |pl: &PlView, s: Section, code: &str| {
        pl.lines
            .iter()
            .find(|l| l.section == s && l.code == code)
            .cloned()
            .unwrap()
    };
    let eq = line(&t06, Section::Depreciation, "equipment");
    assert_eq!((eq.actual_cents, eq.budget_cents), (t06_dep, 0));
    assert!(
        !eq.favorable || eq.actual_cents == 0,
        "amortizacija iznad budžeta je trošak"
    );
    assert_eq!(
        line(&t06, Section::Depreciation, "building").actual_cents,
        0
    );
    let total = line(&t06, Section::Depreciation, "total");
    assert!(total.is_total);
    assert_eq!((total.actual_cents, total.budget_cents), (t06_dep, 0));
    let ebit = line(&t06, Section::Ebit, "ebit");
    assert!(!ebit.is_total);
    assert_eq!(ebit.actual_cents, t06.actual.ebitda_cents - t06_dep);
    assert_eq!(ebit.budget_cents, t06.budget.ebitda_cents);

    // Sezona do t06 zbraja: t05 nosi srpanjski dio (1.–5. srpnja), t06 ostatak.
    let season = g
        .profit_and_loss(me(), Some(week(6)), PlScope::Season)
        .unwrap();
    let t05 = g
        .profit_and_loss(me(), Some(week(5)), PlScope::Round)
        .unwrap();
    assert_eq!(t05.actual.depreciation_cents, monthly * 5 / 31);
    assert_eq!(season.actual.depreciation_cents, monthly * 12 / 31);
    assert_eq!(season.budget.depreciation_cents, 0);
    assert_eq!(
        line(&season, Section::Ebit, "ebit").actual_cents,
        season.actual.ebitda_cents - monthly * 12 / 31
    );
    assert_eq!(season.actual.capex_cents, 15_000_000);
}

// ------------------------------------------------------------------ kraj sezone

#[test]
fn season_end_lists_renewals_and_depreciation() {
    let mut g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Month);
    renew(
        &mut g,
        RoundKey::Month(Month::Jun),
        &[("rooms", 150_000)],
        0,
    );
    let mut g = play_through(g, "jun");
    renew(&mut g, RoundKey::Month(Month::Jul), &[], 200_000);
    let g = play_through(g, "all");
    let season = g.season(me()).unwrap();
    let mine = season.hotels.iter().find(|h| h.mine).unwrap();
    let d = mine.decisions.as_ref().unwrap();
    assert_eq!(
        d.renewals,
        [
            SeasonRenewal {
                asset_code: "rooms".into(),
                asset_name: "Smještaj".into(),
                month: Month::Jun,
                amount_cents: 15_000_000,
            },
            SeasonRenewal {
                asset_code: "building".into(),
                asset_name: "Zgrada".into(),
                month: Month::Jul,
                amount_cents: 20_000_000,
            },
        ]
    );
    assert_eq!(d.renovation_round, None);
    // Smještaj 150.000 € / 84 = 1.785,71 € mjesečno (srpanj–rujan), zgrada 200.000 € / 480 = 416,67 € (kolovoz, rujan).
    let f = mine.finances.as_ref().unwrap();
    assert_eq!(f.depreciation_cents, 3 * 178_571 + 2 * 41_667);
    let pl = g.profit_and_loss(me(), None, PlScope::Season).unwrap();
    assert_eq!(f.ebit_cents, pl.actual.ebitda_cents - f.depreciation_cents);
    assert_eq!(f.ebit_cents, pl.actual.ebit_cents);
}

// ------------------------------------------------------------------ JSON: stari bez novih polja, novi round-trip

fn strip(v: &mut Value, keys: &[&str]) {
    if let Some(o) = v.as_object_mut() {
        for k in keys {
            o.remove(*k);
        }
    }
}

#[test]
fn old_v2_json_without_renewal_fields_is_read_with_defaults() {
    let mut g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week);
    renew(&mut g, week(1), &[("restaurant", 40_000)], 100_000);
    let g = play_through(g, "t01");
    let detail = g.resolved[&week(1)].hotels[0].detail.clone().unwrap();

    let mut state = serde_json::to_value(&detail.after).unwrap();
    assert!(state["building"]["book"]["renewal"].is_object());
    assert!(state["departments"]["restaurant"]["book"]["renewal"].is_object());
    strip(&mut state, &["building"]);
    for d in state["departments"].as_object_mut().unwrap().values_mut() {
        strip(d, &["start_condition_pts", "book"]);
    }
    let old: HotelStateV2Dto = serde_json::from_value(state).unwrap();
    let s = HotelState::try_from(&old).unwrap();
    assert_eq!(
        (s.building.condition_pts, s.building.start_pts),
        (10_000, 10_000)
    );
    assert_eq!(s.building.book, Default::default());
    for d in s.departments.values() {
        assert_eq!(d.start_condition_pts, d.condition_pts);
        assert_eq!(d.book, Default::default());
    }

    let mut accounts = serde_json::to_value(detail.accounts.as_ref().unwrap()).unwrap();
    strip(
        &mut accounts,
        &[
            "building",
            "depreciation_equipment_cents",
            "depreciation_building_cents",
        ],
    );
    for d in accounts["departments"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        strip(d, &["renewal_cents", "renewal_pts", "felt_condition_pts"]);
    }
    let old: PeriodAccountsDto = serde_json::from_value(accounts).unwrap();
    let a = PeriodAccounts::try_from(&old).unwrap();
    assert_eq!(a.building, Default::default());
    assert_eq!(a.depreciation_equipment_cents, 0);
    for d in a.departments.values() {
        assert_eq!((d.renewal_cents, d.renewal_pts), (0, 0));
        assert_eq!(d.felt_condition_pts, d.condition_after_pts);
    }

    let mut plan = serde_json::to_value(&g.resolved[&week(1)].hotels[0].plan).unwrap();
    assert_eq!(plan["building_renewal_cents"], 10_000_000);
    assert_eq!(
        plan["departments"]["restaurant"]["renewal_cents"],
        4_000_000
    );
    strip(&mut plan, &["building_renewal_cents"]);
    for d in plan["departments"].as_object_mut().unwrap().values_mut() {
        strip(d, &["renewal_cents"]);
    }
    let old: PlanDto = serde_json::from_value(plan).unwrap();
    let p = PeriodPlan::try_from(&old).unwrap();
    assert_eq!(p.building_renewal_cents, 0);
    assert!(p.departments.values().all(|d| d.renewal_cents == 0));
}

#[test]
fn new_v2_json_with_renewal_round_trips() {
    let mut g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week);
    renew(
        &mut g,
        week(1),
        &[("restaurant", 40_000), ("rooms", 10_000)],
        100_000,
    );
    let g = play_through(g, "t02");
    let res = &g.resolved[&week(2)].hotels[0];
    let detail = res.detail.clone().unwrap();
    let json = serde_json::to_string(&detail).unwrap();
    let back: game::dto_v2::DetailDto = serde_json::from_str(&json).unwrap();
    assert_eq!(back, detail);
    let s = HotelState::try_from(&detail.after).unwrap();
    assert_eq!(HotelStateV2Dto::from(&s), detail.after);
    let a = PeriodAccounts::try_from(detail.accounts.as_ref().unwrap()).unwrap();
    assert_eq!(
        PeriodAccountsDto::from(&a),
        *detail.accounts.as_ref().unwrap()
    );
    assert_eq!(
        a.departments[&DeptCode("restaurant".into())].renewal_cents,
        800_000
    );
    assert_eq!(a.building.renewal_cents, 2_000_000);
    let plan_json = serde_json::to_string(&res.plan).unwrap();
    let plan: PlanDto = serde_json::from_str(&plan_json).unwrap();
    assert_eq!(plan, res.plan);
}

// ------------------------------------------------------------------ verzija jezgre

#[test]
fn game_created_with_core_2_0_0_is_outdated() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let log: Vec<Recorded> = game::command::new_game_with(
        GameId(uuid::Uuid::from_u128(1)),
        42,
        &snap,
        GUEST,
        hotel_ids(),
        game::command::GameSettings {
            kind: PeriodKind::Week,
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    let mut log = log;
    let GameEvent::GameCreated(c) = &mut log[0].event else {
        panic!()
    };
    c.core_version = "2.0.0".into();
    assert_eq!(
        Game::fold(&log).unwrap_err(),
        GameError::Outdated {
            created: "2.0.0".into(),
            current: sim_hotel_v2::CORE_VERSION.into(),
        }
    );
}
