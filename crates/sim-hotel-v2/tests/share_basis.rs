//! SB-1 (ADR-040 t. 2): osnovica prihoda odjela s modelom „udio prihoda soba" je prihod soba PRIJE provizije (bruto);
//! provizija posrednika je trošak Smještaja i ne umanjuje goste u restoranu. Katalog je degenerirani (odjel „ostalo"
//! ima `ShareOfRooms` s `economy.other_revenue_bp`), isti kao u `group_terms.rs`.

mod common;

use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::department::{DeptCode, RevenueModel};
use sim_hotel_v2::*;

fn cat(commission_bp: u16) -> Catalog {
    let mut c = degenerate(&common::test_catalog());
    c.economy.groups = GroupTerms {
        commission_bp,
        ..GroupTerms::default()
    };
    c
}

fn run(c: &Catalog, group_nights: u32) -> PeriodReport {
    let p = month_period(Month::Jun);
    let mut s = HotelState::from_profile(c, Profile::C).unwrap();
    if group_nights > 0 {
        s.groups.push(GroupBlock {
            month: Month::Jun,
            room_nights: group_nights,
            price_cents: 8_000,
        });
    }
    let plan = PeriodPlan::default_for(&s, &p, c).unwrap();
    resolve_period(&s, &p, &plan, &[], c).unwrap()
}

fn share_bp(c: &Catalog) -> i128 {
    match c.departments[&DeptCode("ostalo".to_string())].revenue {
        RevenueModel::ShareOfRooms { bp } => i128::from(bp),
        _ => panic!("odjel „ostalo” mora imati model udjela prihoda soba"),
    }
}

fn other_revenue(r: &PeriodReport) -> i64 {
    r.accounts.departments[&DeptCode("ostalo".to_string())].revenue_cents
}

/// Zaokruženo na najbliže (polovina gore), kao `c::div_round` za pozitivne iznose.
fn share_of(gross: i64, bp: i128) -> i64 {
    ((i128::from(gross) * bp + 5_000) / 10_000) as i64
}

#[test]
fn share_of_rooms_revenue_is_taken_from_gross_before_commission() {
    let c = cat(1_500);
    let bp = share_bp(&c);
    assert!(bp > 0);
    let r = run(&c, 200);
    // 200 noćenja × 80 € = 16.000 € grupnog prihoda; provizija 15 % = 2.400 € (trošak Smještaja).
    assert_eq!(r.summary.market.group_revenue_cents, 1_600_000);
    let gross = r.summary.money.rooms_revenue_cents;
    assert!(gross >= 1_600_000);
    assert_eq!(
        other_revenue(&r),
        share_of(gross, bp),
        "udio {bp} bp mora se računati od bruto prihoda soba {gross}, ne od {} nakon provizije",
        gross - 240_000
    );
}

/// Kontrola: bez grupe (provizija 0) osnovica je ista bruto i prije i poslije izmjene.
#[test]
fn share_of_rooms_revenue_without_group_is_unchanged() {
    let c = cat(1_500);
    let bp = share_bp(&c);
    let r = run(&c, 0);
    assert_eq!(r.summary.market.group_revenue_cents, 0);
    let gross = r.summary.money.rooms_revenue_cents;
    assert!(gross > 0);
    assert_eq!(other_revenue(&r), share_of(gross, bp));
}
