//! Sloj 0 — hotelski pokazatelji (MODEL_V2 §5, PROSIRENJA P2): čiste funkcije nad `MonthReport`.
mod common;
use common::*;
use sim_hotel::*;

/// Ručno izračunat mjesec: 100 soba × 30 dana, 2 sobe izvan funkcije, prodano 2 400 noći po 100 €.
fn hand_input() -> KpiInput {
    KpiInput {
        rooms: 100,
        days: 30,
        available: 2_940,
        sold: 2_400,
        rooms_revenue_cents: 24_000_000,
        other_revenue_cents: 7_200_000,
        labor_cents: 9_000_000,
        // 2 400 noći × 20 € izravnog troška + 165 000 € troškova koji ne ovise o prodaji.
        costs_cents: 21_300_000,
        gop_cents: 9_900_000,
        cpor_cents: 2_000,
    }
}

#[test]
fn hand_example_every_indicator() {
    let k = kpi(&hand_input()).unwrap();
    assert_eq!(k.room_nights, 3_000);
    assert_eq!(k.sold_nights, 2_400);
    assert_eq!(k.lost_nights, 60);
    // Najviša moguća popunjenost: sobe u funkciji 2 940 ÷ 3 000.
    assert_eq!(k.sellable_occupancy_bp, Some(9_800));
    assert_eq!(k.occupancy_bp, Some(8_000));
    assert_eq!(k.adr_cents, Some(10_000));
    assert_eq!(k.revpar_cents, Some(8_000));
    assert_eq!(k.total_revenue_cents, 31_200_000);
    assert_eq!(k.trevpar_cents, Some(10_400));
    assert_eq!(k.gop_cents, 9_900_000);
    // 9 900 000 / 31 200 000 = 31,73 %
    assert_eq!(k.gop_margin_bp, Some(3_173));
    assert_eq!(k.goppar_cents, Some(3_300));
    // 9 000 000 / 31 200 000 = 28,846 % → 2 885 bp
    assert_eq!(k.labor_share_bp, Some(2_885));
    assert_eq!(k.cpor_cents, Some(2_000));
}

#[test]
fn break_even_on_hand_example() {
    // Doprinos po prodanoj noći: 100 € sobe + 30 € ostalog − 20 € izravnog troška = 110 €.
    // Troškovi koji ne ovise o prodaji: 21 300 000 − 2 400 × 2 000 = 16 500 000 centi.
    // GOP = 0 pri 16 500 000 / 11 000 = 1 500 noći, od 3 000 → 50,00 %.
    let k = kpi(&hand_input()).unwrap();
    assert_eq!(k.break_even_occupancy_bp, Some(5_000));
    // Provjera: pri 1 500 noći GOP je točno nula.
    // (1 500 × 11 000 − 16 500 000 = 0.)
}

#[test]
fn break_even_is_zero_when_costs_do_not_need_sales() {
    let i = KpiInput {
        costs_cents: 2_400 * 2_000,
        gop_cents: 31_200_000 - 4_800_000,
        ..hand_input()
    };
    assert_eq!(kpi(&i).unwrap().break_even_occupancy_bp, Some(0));
}

#[test]
fn break_even_can_exceed_full_house() {
    // Fiksni troškovi veći od doprinosa pune kuće: točka pokrića iznad 100 %.
    let i = KpiInput {
        costs_cents: 4_800_000 + 40_000_000,
        gop_cents: 31_200_000 - 44_800_000,
        ..hand_input()
    };
    let k = kpi(&i).unwrap();
    // 40 000 000 / 11 000 = 3 636,36 noći od 3 000 → 121,212 %; prag se zaokružuje NAVIŠE (12 121,2 → 12 122),
    // da GOP na pragu nikad nije negativan.
    assert_eq!(k.break_even_occupancy_bp, Some(12_122));
    assert!(k.gop_margin_bp.unwrap() < 0);
}

#[test]
fn month_without_sales_gives_none_not_panic() {
    let i = KpiInput {
        sold: 0,
        rooms_revenue_cents: 0,
        other_revenue_cents: 0,
        costs_cents: 16_500_000,
        gop_cents: -16_500_000,
        ..hand_input()
    };
    let k = kpi(&i).unwrap();
    assert_eq!(k.occupancy_bp, Some(0));
    assert_eq!(k.adr_cents, None);
    assert_eq!(k.revpar_cents, Some(0));
    assert_eq!(k.trevpar_cents, Some(0));
    assert_eq!(k.gop_margin_bp, None);
    assert_eq!(k.labor_share_bp, None);
    assert_eq!(k.cpor_cents, None);
    assert_eq!(k.break_even_occupancy_bp, None);
    assert_eq!(k.goppar_cents, Some(-5_500));
}

#[test]
fn hotel_without_rooms_gives_none_not_panic() {
    let i = KpiInput {
        rooms: 0,
        available: 0,
        sold: 0,
        rooms_revenue_cents: 0,
        other_revenue_cents: 0,
        ..hand_input()
    };
    let k = kpi(&i).unwrap();
    assert_eq!(k.room_nights, 0);
    assert_eq!(k.occupancy_bp, None);
    assert_eq!(k.sellable_occupancy_bp, None);
    assert_eq!(k.revpar_cents, None);
    assert_eq!(k.trevpar_cents, None);
    assert_eq!(k.goppar_cents, None);
    assert_eq!(k.break_even_occupancy_bp, None);
}

/// Profil C kroz cijelu sezonu s nekoliko cijena: pravi izvještaji jezgre.
fn real_reports() -> Vec<MonthReport> {
    let cat = test_catalog();
    let mut out = Vec::new();
    for price_pct in [60_i64, 100, 150] {
        let mut s = state_c();
        for m in Month::SEASON {
            let mut plan = MonthPlan::default_for(&s, m, &cat).unwrap();
            plan.price_cents = plan.price_cents * price_pct / 100 / 100 * 100;
            let r = resolve_month(&s, m, &plan, &[], &cat).unwrap();
            s = r.after.clone();
            out.push(r);
        }
    }
    out
}

#[test]
fn revpar_is_adr_times_occupancy_within_rounding() {
    let cat = test_catalog();
    for r in real_reports() {
        let k = month_kpi(&r, &cat).unwrap();
        let adr = i128::from(k.adr_cents.unwrap());
        let occ = i128::from(k.occupancy_bp.unwrap());
        let revpar = i128::from(k.revpar_cents.unwrap());
        // Svaka od tri brojke je zaokružena; zajedno griješe najviše za 1 cent + ADR / 10 000.
        let gap = (adr * occ - revpar * 10_000).abs();
        assert!(
            gap <= 10_000 + adr,
            "{:?}: {adr} × {occ} vs {revpar}",
            r.month
        );
    }
}

#[test]
fn goppar_is_the_same_as_resolve() {
    let cat = test_catalog();
    for r in real_reports() {
        let k = month_kpi(&r, &cat).unwrap();
        assert_eq!(k.goppar_cents, Some(r.money.goppar_cents), "{:?}", r.month);
        assert_eq!(k.gop_cents, r.money.gop_cents);
        assert_eq!(k.sold_nights, r.market.sold);
    }
}

#[test]
fn occupancy_counts_all_rooms_and_lost_nights_separately() {
    let cat = test_catalog();
    let mut s = state_c();
    // Loša oprema: dio soba izvan funkcije.
    s.equipment_pts = 2_000;
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let r = resolve_month(&s, Month::Jun, &plan, &[], &cat).unwrap();
    let k = month_kpi(&r, &cat).unwrap();
    assert!(r.market.rooms_out > 0);
    assert_eq!(k.room_nights, 120 * 30);
    assert_eq!(k.lost_nights, u32::from(r.market.rooms_out) * 30);
    assert_eq!(k.lost_nights + r.market.available, k.room_nights);
    let expected = (i64::from(r.market.sold) * 10_000 * 2 + 3_600) / (2 * 3_600);
    assert_eq!(k.occupancy_bp, Some(expected));
}

#[test]
fn break_even_on_real_report_is_the_first_threshold_with_gop_not_negative() {
    let cat = test_catalog();
    let cpor = i128::from(cat.category(Category::Four).unwrap().cpor_cents);
    let mut checked = 0;
    for r in real_reports() {
        let k = month_kpi(&r, &cat).unwrap();
        let Some(be) = k.break_even_occupancy_bp else {
            continue;
        };
        // GOP pri popunjenosti p (bp), uz isti ADR i iste ostale troškove, pomnožen s prodano × 10 000.
        let sold = i128::from(r.market.sold);
        let n = i128::from(k.room_nights);
        let contribution = i128::from(k.total_revenue_cents) - sold * cpor;
        let fixed = i128::from(r.money.costs_cents) - sold * cpor;
        let gop_x = |p: i128| contribution * p * n - fixed * sold * 10_000;
        // Prag je zaokružen naviše: na njemu GOP nije negativan, bazni bod niže jest.
        assert!(gop_x(i128::from(be)) >= 0, "{:?}", r.month);
        assert!(gop_x(i128::from(be) - 1) < 0, "{:?}", r.month);
        checked += 1;
    }
    assert_eq!(checked, 12, "svaki mjesec ima točku pokrića");
}
