// Zapušten prostor propada brže (O-2 korak e, Leonova odluka): dok je održavanje odjela „odgođeno", mjesečno trošenje
// tog kruga množi se faktorom `deferred_wear_bp` iz gospodarstva v2 (10 000 = kao prije).
mod common;

use common::{catalog_with_renewal, hold_plan};
use sim_hotel_v2::calendar::{PeriodKind, month_period, season_periods};
use sim_hotel_v2::department::{DeptCode, wear_pts};
use sim_hotel_v2::{
    Catalog, CatalogError, HotelState, Maintenance, Month, Profile, RefTable, Strategy,
    resolve_period, run_season,
};

#[test]
fn wear_factor_by_hand() {
    let june = month_period(Month::Jun);
    // Smještaj troši 4 boda mjesečno: odgođeno uz 15 000 bp → 6 bodova; redovno i pojačano bez faktora.
    assert_eq!(
        wear_pts(400, Maintenance::Deferred, 15_000, &june).unwrap(),
        -600
    );
    assert_eq!(
        wear_pts(400, Maintenance::Regular, 15_000, &june).unwrap(),
        -400
    );
    assert_eq!(
        wear_pts(400, Maintenance::Intensive, 15_000, &june).unwrap(),
        -400
    );
    assert_eq!(
        wear_pts(400, Maintenance::Deferred, 10_000, &june).unwrap(),
        -400
    );
    // Lokal 0,5 boda uz 20 000 bp → 1 bod; tjedan od 7 dana u lipnju (30 dana) dobiva razmjerni dio.
    assert_eq!(
        wear_pts(50, Maintenance::Deferred, 20_000, &june).unwrap(),
        -100
    );
    let t01 = season_periods(PeriodKind::Week)[0];
    assert_eq!(
        wear_pts(400, Maintenance::Deferred, 15_000, &t01).unwrap(),
        i32::try_from(sim_hotel_v2::calendar::prorate(-600, &t01).unwrap()).unwrap()
    );
}

fn with_factor(bp: u16) -> Catalog {
    let mut c = catalog_with_renewal();
    c.economy.deferred_wear_bp = bp;
    c
}

/// Lipanj, profil C, svi odjeli na odgođenom: trošenje svakog odjela je mjesečno trošenje · 2 (20 000 bp).
#[test]
fn deferred_departments_wear_faster_in_resolve() {
    let p = month_period(Month::Jun);
    let run = |c: &Catalog| {
        let s = HotelState::from_profile(c, Profile::C).unwrap();
        let mut plan = hold_plan(&s, &p, c);
        for d in plan.departments.values_mut() {
            d.maintenance = Maintenance::Deferred;
        }
        resolve_period(&s, &p, &plan, &[], c).unwrap()
    };
    let fast = run(&with_factor(20_000));
    let c = with_factor(20_000);
    for (k, o) in &fast.accounts.departments {
        let monthly = c.dept(k).unwrap().wear_pts_per_month;
        assert_eq!(o.wear_pts, -2 * monthly, "{}", k.as_str());
    }
    let normal = run(&with_factor(10_000));
    let rooms = |r: &sim_hotel_v2::PeriodReport| r.accounts.departments[&DeptCode::rooms()].clone();
    assert_eq!(rooms(&normal).wear_pts, -400);
    assert!(rooms(&fast).condition_after_pts < rooms(&normal).condition_after_pts);
}

/// Faktor djeluje samo na odgođeno održavanje: robot koji nikad ne odgađa (brižni) igra istu sezonu, a faktor
/// 10 000 je točno današnji račun za svaku strategiju.
#[test]
fn factor_touches_only_deferred_upkeep() {
    let base = catalog_with_renewal();
    assert_eq!(base.economy.deferred_wear_bp, 10_000);
    let fast = with_factor(20_000);
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let run = |c: &Catalog, s: Strategy| {
            let refs = RefTable::compute(
                c,
                c.base.profile(Profile::A).unwrap().category,
                &season_periods(kind),
            )
            .unwrap();
            run_season(Profile::A, s, 5, kind, c, &[], &refs).unwrap()
        };
        assert_eq!(
            run(&fast, Strategy::Caring),
            run(&base, Strategy::Caring),
            "{kind:?}"
        );
        assert_ne!(
            run(&fast, Strategy::Saver),
            run(&base, Strategy::Saver),
            "{kind:?}"
        );
    }
}

#[test]
fn validate_keeps_factor_between_one_and_five_times() {
    for bp in [10_000, 12_500, 50_000] {
        with_factor(bp).validate().unwrap();
    }
    for bp in [0, 9_999, 50_001] {
        match with_factor(bp).validate() {
            Err(CatalogError::OutOfRange(f)) => assert_eq!(f, "v2.economy.deferred_wear_bp"),
            other => panic!("{bp}: {other:?}"),
        }
    }
}
