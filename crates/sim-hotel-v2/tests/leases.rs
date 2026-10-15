// Najmovi (obnova v2, O-2 korak b′; obnova-o1-review „Prijedlog formule najmova"): (a) stanje parkinga je treći
// sastojak osjećaja sobe gosta Smještaja, (b) najamnina lokala pada kad stanje padne ispod praga.
mod common;

use common::{catalog_with_renewal, hold_plan};
use sim_hotel::LevelEffect;
use sim_hotel_v2::calendar::{PeriodKind, month_period, season_periods};
use sim_hotel_v2::department::*;
use sim_hotel_v2::{
    Catalog, CatalogError, HotelState, Maintenance, Month, MonthTable, Profile, resolve_period,
};

fn parking() -> DeptCode {
    DeptCode("parking".into())
}

/// Najam lokala 6.000 € mjesečno, trošenje 0,5 boda, redovno 300 €/mj (+0,38), pojačano 800 €/mj (+2).
fn lease(rule: Option<RentCondition>) -> DeptSpec {
    DeptSpec {
        name: "Najam lokala".into(),
        class: UsaliClass::Rentals,
        revenue: RevenueModel::Fixed {
            monthly_cents: 600_000,
            per_guest_night_cents: 0,
        },
        cost_of_sales_bp: 0,
        direct_per_use_cents: 0,
        wage_cents: 0,
        capacity_tenths_per_staff_day: 0,
        wear_pts_per_month: 50,
        maintenance: [
            LevelEffect {
                cost_cents: 0,
                equipment_pts: 0,
            },
            LevelEffect {
                cost_cents: 30_000,
                equipment_pts: 38,
            },
            LevelEffect {
                cost_cents: 80_000,
                equipment_pts: 200,
            },
        ],
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max: 0,
        seasonal_plan: MonthTable([0; 4]),
        can_close: false,
        staffing: Default::default(),
        renewal: Default::default(),
        guest_share_bp: 0,
        rent_condition: rule,
    }
}

/// Leonova odluka (anketa 13.10. (3)): prag = početno stanje, 2 % po bodu, najviše 30 %.
fn leon_rule() -> RentCondition {
    RentCondition {
        threshold_pts: None,
        cut_bp_per_pt: 200,
        max_cut_bp: 3_000,
    }
}

fn lease_state(start: i32, now: i32) -> DeptState {
    DeptState {
        class: UsaliClass::Rentals,
        open: true,
        permanent: 0,
        seasonal: 0,
        condition_pts: now,
        satisfaction_pts: 7_000,
        season_departures: 0,
        start_condition_pts: start,
        book: Default::default(),
    }
}

fn lease_june(spec: &DeptSpec, st: &DeptState, m: Maintenance) -> DeptOutcome {
    let p = month_period(Month::Jun);
    dept_step(&DeptInput {
        code: DeptCode("retail_lease".into()),
        period: &p,
        spec,
        state: st,
        plan: &DeptPlan {
            open: true,
            seasonal_target: 0,
            price_level_pct: 100,
            maintenance: m,
            renewal_cents: 0,
        },
        guest_nights: 4_000,
        rooms_revenue_cents: 30_000_000,
        experience_pts: 6_000,
        morale_pts: 6_000,
        wage_pct: 100,
        satisfaction_step_bp: 5_000,
        renewal: Default::default(),
        building_pts: 0,
        building_share_bp: 0,
        deferred_wear_bp: 10_000,
        situation_pts: 0,
    })
    .unwrap()
}

// ---------------------------------------------------------------- (a) osjećaj sobe

#[test]
fn felt_mix_with_zero_share_is_felt_condition() {
    for c in [0, 3_500, 7_500, 10_000] {
        for (b, s) in [(10_000, 0), (8_000, 2_000), (4_500, 2_000)] {
            let one = felt_condition(c, b, s).unwrap();
            assert_eq!(felt_condition_mix(c, &[(b, s)]).unwrap(), one);
            for parking in [0, 6_000, 10_000] {
                assert_eq!(felt_condition_mix(c, &[(b, s), (parking, 0)]).unwrap(), one);
            }
        }
    }
}

/// Ručni primjer: soba 75, zgrada 80 (20 %), parking 60 (5 %) →
/// div_round(7 500·7 500 + 8 000·2 000 + 6 000·500, 10 000) = 75 250 000 / 10 000 = 7 525; bez parkinga 7 600.
#[test]
fn felt_mix_by_hand() {
    assert_eq!(
        felt_condition_mix(7_500, &[(8_000, 2_000), (6_000, 500)]).unwrap(),
        7_525
    );
    assert_eq!(felt_condition(7_500, 8_000, 2_000).unwrap(), 7_600);
}

fn with_parking_share(share: u16) -> Catalog {
    let mut c = catalog_with_renewal();
    c.departments.get_mut(&parking()).unwrap().guest_share_bp = share;
    c
}

/// Prvi tjedan profila A: stanje sobe koje gost osjeća je mješavina sobe, zgrade (20 %) i parkinga (5 %) nakon
/// kruga.
#[test]
fn parking_condition_enters_rooms_felt_condition() {
    let p = season_periods(PeriodKind::Week)[0];
    let run = |c: &Catalog| {
        let s = HotelState::from_profile(c, Profile::A).unwrap();
        resolve_period(&s, &p, &hold_plan(&s, &p, c), &[], c).unwrap()
    };
    let with = run(&with_parking_share(500));
    let without = run(&with_parking_share(0));
    let a = &with.accounts;
    let rooms = &a.departments[&DeptCode::rooms()];
    let park = &a.departments[&parking()];
    let want = (i64::from(rooms.condition_after_pts) * 7_500
        + i64::from(a.building.after_pts) * 2_000
        + i64::from(park.condition_after_pts) * 500
        + 5_000)
        / 10_000;
    assert_eq!(i64::from(rooms.felt_condition_pts), want);
    // Parking se razlikuje od sobe, pa njegov udio pomiče osjećaj sobe prema stanju parkinga.
    let felt_without = without.accounts.departments[&DeptCode::rooms()].felt_condition_pts;
    assert_ne!(park.condition_after_pts, rooms.condition_after_pts);
    assert_ne!(rooms.felt_condition_pts, felt_without);
    assert_eq!(
        rooms.felt_condition_pts > felt_without,
        park.condition_after_pts > rooms.condition_after_pts
    );
    // Ostali odjeli osjećaju samo zgradu.
    assert_eq!(
        a.departments[&DeptCode("restaurant".into())],
        without.accounts.departments[&DeptCode("restaurant".into())]
    );
}

/// Nula = bez učinka: udio parkinga 0 i pravilo najamnine s popustom 0 daju istu sezonu kao katalog bez njih.
#[test]
fn zero_share_and_zero_cut_change_nothing() {
    let base = catalog_with_renewal();
    let mut zero = with_parking_share(0);
    // U testnom katalogu parking ima stalni prihod (`Fixed`), pa smije nositi pravilo najamnine.
    zero.departments.get_mut(&parking()).unwrap().rent_condition = Some(RentCondition {
        threshold_pts: Some(10_000),
        cut_bp_per_pt: 0,
        max_cut_bp: 0,
    });
    zero.validate().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in Profile::ALL {
            let run = |c: &Catalog| {
                let refs = sim_hotel_v2::RefTable::compute(
                    c,
                    c.base.profile(profile).unwrap().category,
                    &season_periods(kind),
                )
                .unwrap();
                sim_hotel_v2::run_season(
                    profile,
                    sim_hotel_v2::Strategy::Caring,
                    3,
                    kind,
                    c,
                    &[],
                    &refs,
                )
                .unwrap()
            };
            assert_eq!(run(&zero), run(&base), "{profile:?} {kind:?}");
        }
    }
}

// ---------------------------------------------------------------- (b) najamnina lokala

/// Ručni primjeri pravila: lokal 6.000 €, početno stanje 60, 2 % po bodu, najviše 30 %.
#[test]
fn rent_cut_by_hand() {
    let rule = leon_rule();
    let rent = |now: i32| rent_after_condition(600_000, Some(&rule), 6_000, now).unwrap();
    assert_eq!(rent(7_000), 600_000); // iznad praga
    assert_eq!(rent(6_000), 600_000); // na pragu
    assert_eq!(rent(5_950), 594_000); // −0,5 boda → −1 %
    assert_eq!(rent(5_800), 576_000); // −2 boda → −4 %
    assert_eq!(rent(4_000), 420_000); // −20 bodova → −40 %, najviše 30 %
    assert_eq!(
        rent_after_condition(600_000, None, 6_000, 0).unwrap(),
        600_000
    );
    let fixed_threshold = RentCondition {
        threshold_pts: Some(6_500),
        ..rule
    };
    assert_eq!(
        rent_after_condition(600_000, Some(&fixed_threshold), 9_000, 6_000).unwrap(),
        540_000
    );
}

/// Lipanj, lokal B (početno 60), odgođeno održavanje: stanje 60 → 59,5, najamnina 6.000 € − 1 % = 5.940 €; redovno
/// održavanje (+0,38) drži 59,88 → −0,12 boda → −0,24 % ≈ 5.986 €. Bez pravila uvijek 6.000 €.
#[test]
fn lease_revenue_in_june_by_hand() {
    let st = lease_state(6_000, 6_000);
    let deferred = lease_june(&lease(Some(leon_rule())), &st, Maintenance::Deferred);
    assert_eq!(deferred.condition_after_pts, 5_950);
    assert_eq!(deferred.revenue_cents, 594_000);
    assert_eq!(deferred.profit_cents, 594_000);
    let regular = lease_june(&lease(Some(leon_rule())), &st, Maintenance::Regular);
    assert_eq!(regular.condition_after_pts, 5_988);
    assert_eq!(regular.revenue_cents, 600_000 - 1_440);
    for m in [Maintenance::Deferred, Maintenance::Regular] {
        let o = lease_june(&lease(None), &st, m);
        assert_eq!(o.revenue_cents, 600_000);
    }
}

// ---------------------------------------------------------------- provjera kataloga

fn out_of_range(c: &Catalog, field: &str) {
    match c.validate() {
        Err(CatalogError::OutOfRange(f)) => assert_eq!(f, field),
        other => panic!("očekivano {field}, dobiveno {other:?}"),
    }
}

#[test]
fn validate_rejects_bad_lease_rules() {
    let mut c = with_parking_share(500);
    c.validate().unwrap();
    c.departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .guest_share_bp = 100;
    out_of_range(&c, "v2.departments.rooms.guest_share_bp");

    let mut c = with_parking_share(8_001); // zgrada 2 000 + parking 8 001 > 10 000
    out_of_range(&c, "v2.departments.guest_share_bp");
    c.departments.get_mut(&parking()).unwrap().guest_share_bp = 8_000;
    c.validate().unwrap();

    let mut c = catalog_with_renewal();
    c.departments
        .get_mut(&DeptCode("restaurant".into()))
        .unwrap()
        .rent_condition = Some(leon_rule());
    out_of_range(&c, "v2.departments.restaurant.rent_condition");

    let mut c = catalog_with_renewal();
    c.departments.get_mut(&parking()).unwrap().rent_condition = Some(RentCondition {
        max_cut_bp: 10_001,
        ..leon_rule()
    });
    out_of_range(&c, "v2.departments.parking.rent_condition");
    c.departments.get_mut(&parking()).unwrap().rent_condition = Some(RentCondition {
        threshold_pts: Some(10_001),
        ..leon_rule()
    });
    out_of_range(&c, "v2.departments.parking.rent_condition");
}
