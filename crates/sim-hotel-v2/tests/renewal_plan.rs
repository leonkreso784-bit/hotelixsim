// Obnova v2 (O-1): plan, zaključavanje i greške, granice, roboti, degenerirani katalog.
mod common;

use engine::{Rng, label};
use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code, season_periods};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::renewal::gain_pts;
use sim_hotel_v2::{
    Catalog, HotelState, Month, Period, PeriodPlan, PlanError, Profile, RefTable, Strategy,
    resolve_period, robot_plan, run_season,
};

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

fn eur(x: i64) -> i64 {
    x * 100
}

fn week(c: &str) -> Period {
    period_by_code(PeriodKind::Week, c).unwrap()
}

fn set(plan: &mut PeriodPlan, d: &str, a: i64) {
    plan.departments.get_mut(&code(d)).unwrap().renewal_cents = eur(a);
}

/// Stanje profila C nakon kruga `p` s obnovom restorana `a` €.
fn after_round(c: &Catalog, s: &HotelState, p: &Period, a: i64) -> HotelState {
    let mut plan = common::hold_plan(s, p, c);
    set(&mut plan, "restaurant", a);
    resolve_period(s, p, &plan, &[], c).unwrap().after
}

// ------------------------------------------------------------------ zaključavanje (§3.3, §4.2)

#[test]
fn renewal_is_locked_for_the_rest_of_the_month() {
    let c = common::catalog_with_renewal();
    let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
    // lipanj t01–t05 bez obnove
    for w in ["t01", "t02", "t03", "t04", "t05"] {
        s = after_round(&c, &s, &week(w), 0);
    }
    s = after_round(&c, &s, &week("t06"), 40_000);
    let t07 = week("t07");
    let mut other = common::hold_plan(&s, &t07, &c);
    set(&mut other, "restaurant", 30_000);
    assert_eq!(other.validate(&s, &t07, &c), Err(PlanError::RenewalLocked));
    set(&mut other, "restaurant", 0);
    assert_eq!(other.validate(&s, &t07, &c), Err(PlanError::RenewalLocked));
    set(&mut other, "restaurant", 40_000);
    assert_eq!(other.validate(&s, &t07, &c), Ok(()));
    // zadani plan i preneseni plan nose zaključani iznos
    let default = PeriodPlan::default_for(&s, &t07, &c).unwrap();
    assert_eq!(
        default.departments[&code("restaurant")].renewal_cents,
        eur(40_000)
    );
    let mut t06_plan = common::hold_plan(&s, &week("t06"), &c);
    set(&mut t06_plan, "restaurant", 40_000);
    let carried = t06_plan.carry_into(&s, &t07, &c).unwrap();
    assert_eq!(
        carried.departments[&code("restaurant")].renewal_cents,
        eur(40_000)
    );
    assert_eq!(carried.validate(&s, &t07, &c), Ok(()));
    // granice pokazuju zaključano
    let l = PeriodPlan::limits(&s, &t07, &c).unwrap();
    let rl = l.departments[&code("restaurant")].renewal;
    assert_eq!((rl.month, rl.rounds_left), (Month::Jul, 3));
    let locked = rl.locked.unwrap();
    assert_eq!(
        (locked.total_cents, locked.paid_cents),
        (eur(40_000), eur(10_000))
    );
    // ostatak srpnja
    for w in ["t07", "t08", "t09"] {
        s = after_round(&c, &s, &week(w), 40_000);
    }
    // kolovoz: nova odluka slobodna, preneseni plan kreće od 0
    let t10 = week("t10");
    let mut t09_plan = common::hold_plan(&s, &week("t09"), &c);
    set(&mut t09_plan, "restaurant", 40_000);
    let carried = t09_plan.carry_into(&s, &t10, &c).unwrap();
    assert_eq!(carried.departments[&code("restaurant")].renewal_cents, 0);
    assert_eq!(
        PeriodPlan::default_for(&s, &t10, &c).unwrap().departments[&code("restaurant")]
            .renewal_cents,
        0
    );
    let mut aug = common::hold_plan(&s, &t10, &c);
    set(&mut aug, "restaurant", 20_000);
    assert_eq!(aug.validate(&s, &t10, &c), Ok(()));
    // amount 0 ne stvara obnovu: odluka smije i u kasnijem tjednu istog mjeseca
    let s10 = after_round(&c, &s, &t10, 0);
    let mut t11 = common::hold_plan(&s10, &week("t11"), &c);
    set(&mut t11, "restaurant", 25_000);
    assert_eq!(t11.validate(&s10, &week("t11"), &c), Ok(()));
}

#[test]
fn building_renewal_is_locked_too() {
    let c = common::catalog_with_renewal();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let t01 = week("t01");
    let mut plan = common::hold_plan(&s, &t01, &c);
    plan.building_renewal_cents = eur(100_000);
    let s = resolve_period(&s, &t01, &plan, &[], &c).unwrap().after;
    let t02 = week("t02");
    let mut next = common::hold_plan(&s, &t02, &c);
    next.building_renewal_cents = eur(50_000);
    assert_eq!(next.validate(&s, &t02, &c), Err(PlanError::RenewalLocked));
    assert_eq!(
        PeriodPlan::default_for(&s, &t02, &c)
            .unwrap()
            .building_renewal_cents,
        eur(100_000)
    );
    assert_eq!(
        plan.carry_into(&s, &t02, &c)
            .unwrap()
            .building_renewal_cents,
        eur(100_000)
    );
    let l = PeriodPlan::limits(&s, &t02, &c).unwrap();
    assert_eq!(l.building_renewal.max_monthly_cents, eur(200_000));
    assert_eq!(l.building_renewal.locked.unwrap().paid_cents, eur(20_000));
}

#[test]
fn amounts_outside_the_rules_are_rejected() {
    let c = common::catalog_with_renewal();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let check = |f: &dyn Fn(&mut PeriodPlan)| {
        let mut plan = common::hold_plan(&s, &p, &c);
        f(&mut plan);
        plan.validate(&s, &p, &c)
    };
    let out = Err(PlanError::RenewalOutOfRange);
    assert_eq!(check(&|pl| set(pl, "restaurant", 55_000)), out);
    assert_eq!(check(&|pl| set(pl, "restaurant", 12_345)), out);
    assert_eq!(check(&|pl| set(pl, "restaurant", -5_000)), out);
    assert_eq!(check(&|pl| pl.building_renewal_cents = eur(210_000)), out);
    assert_eq!(check(&|pl| pl.building_renewal_cents = eur(15_000)), out);
    assert_eq!(check(&|pl| set(pl, "restaurant", 50_000)), Ok(()));
    assert_eq!(check(&|pl| set(pl, "rooms", 150_000)), Ok(()));
    assert_eq!(
        check(&|pl| pl.building_renewal_cents = eur(200_000)),
        Ok(())
    );
    let unavailable = Err(PlanError::RenewalUnavailable);
    // neraspoređeni odjel
    assert_eq!(check(&|pl| set(pl, "admin", 5_000)), unavailable);
    // stara kvačica u katalogu s obnovom
    assert_eq!(check(&|pl| pl.renovation = true), unavailable);
    assert!(!PeriodPlan::limits(&s, &p, &c).unwrap().renovation_available);
    // odjel bez obnove (kao lokal): parking bez krivulje
    let mut lease = c.clone();
    lease.departments.get_mut(&code("parking")).unwrap().renewal = Default::default();
    let s2 = HotelState::from_profile(&lease, Profile::C).unwrap();
    let mut plan = common::hold_plan(&s2, &p, &lease);
    set(&mut plan, "parking", 5_000);
    assert_eq!(plan.validate(&s2, &p, &lease), unavailable);
    // zatvoren odjel smije obnovu
    let mut closed = common::hold_plan(&s, &p, &c);
    let lobby = closed.departments.get_mut(&code("lobby_bar")).unwrap();
    lobby.open = false;
    lobby.seasonal_target = 0;
    lobby.renewal_cents = eur(10_000);
    let moved: u16 = s.departments[&code("lobby_bar")].seasonal;
    if moved <= c.coeffs(PeriodKind::Month).unwrap().seasonal_step {
        assert_eq!(closed.validate(&s, &p, &c), Ok(()));
    }
}

#[test]
fn old_checkbox_in_degenerate_catalog_works_as_before() {
    let c = sim_hotel_v2::degenerate(&common::test_catalog());
    assert!(!c.renewal_enabled());
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let l = PeriodPlan::limits(&s, &p, &c).unwrap();
    assert!(l.renovation_available);
    assert_eq!(l.building_renewal.max_monthly_cents, 0);
    assert!(
        l.departments
            .values()
            .all(|d| d.renewal.max_monthly_cents == 0)
    );
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.renovation = true;
    let after = resolve_period(&s, &p, &plan, &[], &c).unwrap().after;
    let jul = month_period(Month::Jul);
    let mut again = PeriodPlan::default_for(&after, &jul, &c).unwrap();
    again.renovation = true;
    assert_eq!(
        again.validate(&after, &jul, &c),
        Err(PlanError::RenovationTwice)
    );
    assert!(
        !PeriodPlan::limits(&after, &jul, &c)
            .unwrap()
            .renovation_available
    );
}

// ------------------------------------------------------------------ degenerirani katalog

#[test]
fn degenerate_has_no_renewal_building_or_ebit_effect() {
    let c = sim_hotel_v2::degenerate(&common::test_catalog());
    assert!(!c.renewal_enabled());
    assert_eq!(c.economy.building.satisfaction_share_bp, 0);
    assert_eq!(c.economy.building.wear_pts_per_month, 0);
    assert!(
        c.departments
            .values()
            .all(|d| d.renewal.max_monthly_cents == 0)
    );
    assert!(c.profiles.values().all(|p| p.building_pts == 10_000));
    for kind in PeriodKind::ALL {
        let periods = season_periods(kind);
        for profile in Profile::ALL {
            let category = c.base.profile(profile).unwrap().category;
            let refs = RefTable::compute(&c, category, &periods).unwrap();
            assert_eq!(refs.start_ebitpar_cents, refs.start.goppar_cents);
            for (pt, e) in refs.points.iter().zip(&refs.ebitpar_cents) {
                assert_eq!(pt.goppar_cents, *e);
            }
            for strategy in Strategy::ALL {
                let r = run_season(profile, strategy, 7, kind, &c, &[], &refs).unwrap();
                for (rep, _) in &r.periods {
                    let a = &rep.accounts;
                    assert_eq!(
                        (
                            a.depreciation_equipment_cents,
                            a.depreciation_building_cents
                        ),
                        (0, 0)
                    );
                    assert_eq!(rep.finance.ebitpar_cents, rep.summary.money.goppar_cents);
                    assert_eq!(rep.finance.ebit_cents, rep.summary.money.gop_cents);
                    assert_eq!(a.building.after_pts, 10_000);
                    assert!(
                        a.departments.values().all(|d| d.renewal_cents == 0
                            && d.felt_condition_pts == d.condition_after_pts)
                    );
                }
            }
        }
    }
}

// ------------------------------------------------------------------ roboti (§5.1)

fn plan_of(strategy: Strategy, s: &HotelState, p: &Period, c: &Catalog) -> PeriodPlan {
    let mut rng = Rng::derive(7, &[u64::from(p.index), label("robot:plan")]);
    robot_plan(strategy, s, p, c, &mut rng).unwrap()
}

fn renewals(plan: &PeriodPlan) -> Vec<(String, i64)> {
    plan.departments
        .iter()
        .filter(|(_, d)| d.renewal_cents != 0)
        .map(|(k, d)| (k.as_str().to_string(), d.renewal_cents))
        .collect()
}

#[test]
fn caring_renews_below_60_with_the_smallest_step() {
    let c = common::catalog_with_renewal();
    // B: Smještaj 40 → cilj 60, treba 20 bodova: najmanji iznos u koraku 5.000 € je 100.000 € (E = 2000).
    let s = HotelState::from_profile(&c, Profile::B).unwrap();
    let p = week("t01");
    let plan = plan_of(Strategy::Caring, &s, &p, &c);
    assert_eq!(renewals(&plan), [("rooms".to_string(), eur(100_000))]);
    let rooms = c.dept(&DeptCode::rooms()).unwrap();
    assert!(gain_pts(&rooms.renewal, eur(100_000)).unwrap() >= 2_000);
    assert!(gain_pts(&rooms.renewal, eur(95_000)).unwrap() < 2_000);
    assert_eq!(plan.building_renewal_cents, 0);
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
    // nosi zaključano kroz mjesec, pa više nikad (jednom po odjelu u sezoni)
    let s1 = resolve_period(&s, &p, &plan, &[], &c).unwrap().after;
    let t02 = week("t02");
    let next = plan_of(Strategy::Caring, &s1, &t02, &c);
    assert_eq!(renewals(&next), [("rooms".to_string(), eur(100_000))]);
    let mut s2 = s1;
    for w in ["t02", "t03", "t04", "t05"] {
        let q = week(w);
        let pl = plan_of(Strategy::Caring, &s2, &q, &c);
        s2 = resolve_period(&s2, &q, &pl, &[], &c).unwrap().after;
    }
    let mut low = s2.clone();
    low.departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .condition_pts = 3_000;
    assert!(renewals(&plan_of(Strategy::Caring, &low, &week("t06"), &c)).is_empty());
}

#[test]
fn caring_renews_five_points_below_start_and_hunter_only_rooms() {
    let c = common::catalog_with_renewal();
    let s = HotelState::from_profile(&c, Profile::A).unwrap();
    let p = week("t03");
    let rest = code("restaurant");
    assert_eq!(s.departments[&rest].start_condition_pts, 7_800);
    let mut just = s.clone();
    just.departments.get_mut(&rest).unwrap().condition_pts = 7_300;
    assert!(renewals(&plan_of(Strategy::Caring, &just, &p, &c)).is_empty());
    let mut below = s.clone();
    below.departments.get_mut(&rest).unwrap().condition_pts = 7_200;
    // cilj 78, treba 6 bodova: 5.000 € daje 4,17, 10.000 € daje 7,69
    assert_eq!(
        renewals(&plan_of(Strategy::Caring, &below, &p, &c)),
        [("restaurant".to_string(), eur(10_000))]
    );
    assert!(renewals(&plan_of(Strategy::OccupancyHunter, &below, &p, &c)).is_empty());
    below
        .departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .condition_pts = 5_000;
    assert_eq!(
        renewals(&plan_of(Strategy::OccupancyHunter, &below, &p, &c)),
        [("rooms".to_string(), eur(40_000))] // cilj 60, treba 10: 35.000 € daje 9,46, 40.000 € 10,53
    );
    // već uloženo u sezoni → ne opet
    let mut done = below.clone();
    done.departments
        .get_mut(&rest)
        .unwrap()
        .book
        .capitalized_cents = eur(5_000);
    assert!(
        !renewals(&plan_of(Strategy::Caring, &done, &p, &c))
            .iter()
            .any(|(k, _)| k == "restaurant")
    );
}

#[test]
fn saver_and_random_never_renew_and_random_plan_is_unchanged() {
    let with = common::catalog_with_renewal();
    let without = common::catalog_with_departments();
    for profile in Profile::ALL {
        let mut low = HotelState::from_profile(&with, profile).unwrap();
        let plain = HotelState::from_profile(&without, profile).unwrap();
        for d in low.departments.values_mut() {
            d.condition_pts = 2_000;
        }
        for p in season_periods(PeriodKind::Week) {
            for strategy in [Strategy::Saver, Strategy::Random] {
                let plan = plan_of(strategy, &low, &p, &with);
                assert!(renewals(&plan).is_empty() && plan.building_renewal_cents == 0);
            }
            assert_eq!(
                plan_of(
                    Strategy::Random,
                    &HotelState::from_profile(&with, profile).unwrap(),
                    &p,
                    &with
                ),
                plan_of(Strategy::Random, &plain, &p, &without),
                "{profile:?} {}",
                p.code()
            );
        }
    }
}
