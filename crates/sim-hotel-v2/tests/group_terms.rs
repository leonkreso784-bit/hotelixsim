//! J-3: uvjeti grupnih rezervacija (`EconomyV2::groups`). M1 provizija posrednika, M2 grupa ne iskoristi cijeli
//! blok, M3 opterećenje gostiju i osoblja, M5 istiskivanje pojedinačne potražnje; sve 0 = grupa kao u v1.
//! (M4 i granica grupa po sezoni su pravila generatora: `crates/content/tests/v2_situation_groups.rs`.)

mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::*;

fn cat(terms: GroupTerms) -> Catalog {
    let mut c = degenerate(&common::test_catalog());
    c.economy.groups = terms;
    c
}

/// Hotel C u lipnju s prihvaćenom grupom od `room_nights` noćenja po 80 € za mjesec `month`.
fn run_with_group(c: &Catalog, p: &Period, month: Month, room_nights: u32) -> PeriodReport {
    let mut s = HotelState::from_profile(c, Profile::C).unwrap();
    s.groups.push(GroupBlock {
        month,
        room_nights,
        price_cents: 8_000,
    });
    let plan = PeriodPlan::default_for(&s, p, c).unwrap();
    resolve_period(&s, p, &plan, &[], c).unwrap()
}

fn june() -> Period {
    month_period(Month::Jun)
}

#[test]
fn commission_cuts_group_revenue_only() {
    let zero = run_with_group(&cat(GroupTerms::default()), &june(), Month::Jun, 200);
    let m1 = run_with_group(
        &cat(GroupTerms {
            commission_bp: 1_500,
            ..GroupTerms::default()
        }),
        &june(),
        Month::Jun,
        200,
    );
    // 200 noćenja × 80 € = 16.000 €; provizija 15 % → 13.600 €.
    assert_eq!(zero.summary.market.group_sold, 200);
    assert_eq!(zero.summary.market.group_revenue_cents, 1_600_000);
    assert_eq!(m1.summary.market.group_sold, 200);
    assert_eq!(m1.summary.market.group_revenue_cents, 1_360_000);
    assert_eq!(
        m1.summary.market.individual_sold,
        zero.summary.market.individual_sold
    );
    common::assert_lines_explain(&m1.summary);
}

#[test]
fn attrition_fills_part_of_the_block_and_holds_the_rest() {
    let m2 = cat(GroupTerms {
        attrition_bp: 1_500,
        ..GroupTerms::default()
    });
    // Mali blok: grupa iskoristi 85 % (170 od 200), plaća samo iskorištena noćenja.
    let small = run_with_group(&m2, &june(), Month::Jun, 200);
    assert_eq!(small.summary.market.group_sold, 170);
    assert_eq!(small.summary.market.group_revenue_cents, 1_360_000);
    // Blok od svih soba mjeseca: 15 % bloka ostaje prazno i ne prodaje se pojedincima.
    let zero = run_with_group(&cat(GroupTerms::default()), &june(), Month::Jun, 1);
    let all = zero.summary.market.available;
    let full = run_with_group(&m2, &june(), Month::Jun, all);
    let m = &full.summary.market;
    let held = all - (all * 85 + 50) / 100;
    assert!(m.sold + held <= m.available, "{m:?}");
    assert_eq!(m.individual_sold, 0, "{m:?}");
    common::assert_lines_explain(&full.summary);
}

#[test]
fn displacement_turns_away_individual_demand() {
    let zero = run_with_group(&cat(GroupTerms::default()), &june(), Month::Jun, 200);
    // Hotel nije pun, pa se istisnuta potražnja vidi u prodanim noćenjima pojedinaca.
    assert!(zero.summary.market.sold < zero.summary.market.available);
    let m5 = run_with_group(
        &cat(GroupTerms {
            displacement_bp: 3_000,
            ..GroupTerms::default()
        }),
        &june(),
        Month::Jun,
        200,
    );
    // 30 % od 200 noćenja bloka = 60 pojedinačnih noćenja manje.
    assert_eq!(
        m5.summary.market.individual_sold,
        zero.summary.market.individual_sold - 60
    );
    assert_eq!(m5.summary.market.group_sold, 200);
    common::assert_lines_explain(&m5.summary);
}

/// Pad cilja = `pts` × udio grupnih noćenja u prodanim noćenjima kruga (zaokruženo).
fn share_of(pts: u64, m: &MarketOutcome) -> i32 {
    let (g, sold) = (u64::from(m.group_sold), u64::from(m.sold));
    i32::try_from((pts * g * 2 + sold) / (2 * sold)).unwrap()
}

#[test]
fn group_load_lowers_satisfaction_and_morale_targets_by_group_share() {
    let sat = GroupTerms {
        load_satisfaction_pts: 4_000,
        ..GroupTerms::default()
    };
    let morale = GroupTerms {
        load_morale_pts: 4_000,
        ..GroupTerms::default()
    };
    // Učinak je na ciljevima, pa je isti u tjednu i u mjesecu.
    for p in [june(), period_by_code(PeriodKind::Week, "t02").unwrap()] {
        let zero = run_with_group(&cat(GroupTerms::default()), &p, Month::Jun, 200);
        let s = run_with_group(&cat(sat.clone()), &p, Month::Jun, 200);
        let m = run_with_group(&cat(morale.clone()), &p, Month::Jun, 200);
        assert!(zero.summary.market.group_sold > 0);
        let drop = share_of(4_000, &zero.summary.market);
        assert!(drop > 0);
        assert_eq!(
            s.summary.guests.satisfaction_target_pts,
            zero.summary.guests.satisfaction_target_pts - drop,
            "{}",
            p.code()
        );
        assert!(s.after.satisfaction_pts < zero.after.satisfaction_pts);
        assert_eq!(
            m.summary.people.morale_target_pts,
            zero.summary.people.morale_target_pts - drop,
            "{}",
            p.code()
        );
        assert!(m.after.morale_pts < zero.after.morale_pts);
        // Tržište i novac isti: opterećenje je samo na ciljevima.
        for r in [&s, &m] {
            assert_eq!(r.summary.market, zero.summary.market);
            assert_eq!(r.summary.money.costs_cents, zero.summary.money.costs_cents);
            common::assert_lines_explain(&r.summary);
        }
    }
}

#[test]
fn group_load_without_a_group_changes_nothing() {
    let p = june();
    let terms = GroupTerms {
        commission_bp: 1_500,
        attrition_bp: 1_500,
        displacement_bp: 3_000,
        load_satisfaction_pts: 4_000,
        load_morale_pts: 4_000,
    };
    let c0 = cat(GroupTerms::default());
    let c1 = cat(terms);
    let s = HotelState::from_profile(&c0, Profile::C).unwrap();
    let plan = PeriodPlan::default_for(&s, &p, &c0).unwrap();
    // Bez grupe u stanju uvjeti grupe ne mijenjaju ništa (isti izvještaj do bajta).
    assert_eq!(
        resolve_period(&s, &p, &plan, &[], &c1).unwrap(),
        resolve_period(&s, &p, &plan, &[], &c0).unwrap()
    );
}

#[test]
fn group_of_another_month_is_not_touched() {
    // Grupa za kolovoz ne puni lipanj, pa ni uvjeti grupe ne diraju lipanj.
    let terms = GroupTerms {
        commission_bp: 1_500,
        attrition_bp: 1_500,
        displacement_bp: 3_000,
        load_satisfaction_pts: 4_000,
        load_morale_pts: 4_000,
    };
    let zero = run_with_group(&cat(GroupTerms::default()), &june(), Month::Aug, 200);
    let all = run_with_group(&cat(terms), &june(), Month::Aug, 200);
    assert_eq!(all.summary.market.group_sold, 0);
    assert_eq!(all.summary, zero.summary);
}
