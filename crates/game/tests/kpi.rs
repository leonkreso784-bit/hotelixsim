//! Sloj 0 — pokazatelji iz dnevnika: samo obračunati mjeseci, iste brojke kao jezgra i kao „Moj hotel".
mod common;
use common::*;
use game::dto::MonthReportDto;
use game::kpi::report_kpi;
use game::*;
use sim_hotel::{Month, MonthPlan, month_kpi, resolve_month};
use uuid::Uuid;

#[test]
fn open_june_has_no_indicators_yet() {
    let (g, h) = open_june_game();
    assert_eq!(g.hotel_kpi(h).unwrap(), Vec::new());
}

#[test]
fn only_resolved_months_in_season_order() {
    let g = june_resolved_game();
    let months: Vec<RoundKey> = g
        .hotel_kpi(my_hotel(&g))
        .unwrap()
        .iter()
        .map(|k| k.round)
        .collect();
    assert_eq!(months, [RoundKey::Month(Month::Jun)]);

    let (g, h) = july_resolved_bad_chef_game();
    let ks = g.hotel_kpi(h).unwrap();
    let months: Vec<RoundKey> = ks.iter().map(|k| k.round).collect();
    assert_eq!(
        months,
        [RoundKey::Month(Month::Jun), RoundKey::Month(Month::Jul)]
    );
    for k in &ks {
        let rep = &g.resolved[&k.round].hotels[0].report;
        assert_eq!(k.kpi.goppar_cents, Some(rep.money.goppar_cents));
        assert_eq!(k.kpi.sold_nights, rep.market.sold);
    }
}

#[test]
fn unknown_hotel_is_an_error() {
    let g = june_resolved_game();
    assert_eq!(
        g.hotel_kpi(HotelId(Uuid::from_u128(777))),
        Err(GameError::UnknownHotel)
    );
}

#[test]
fn log_form_gives_the_same_numbers_as_core() {
    let (g, h) = open_june_game();
    let cat = &g.content.catalog;
    let state = g.state_at(h, Month::Jun).unwrap();
    for price_pct in [60_i64, 100, 150] {
        let mut plan = MonthPlan::default_for(&state, Month::Jun, cat).unwrap();
        plan.price_cents = plan.price_cents * price_pct / 100 / 100 * 100;
        let report = resolve_month(&state, Month::Jun, &plan, &[], cat).unwrap();
        let from_log = report_kpi(&MonthReportDto::from(&report), cat).unwrap();
        assert_eq!(from_log, month_kpi(&report, cat).unwrap(), "{price_pct} %");
    }
}

#[test]
fn last_month_on_my_hotel_matches_indicators() {
    let (g, h) = july_resolved_bad_chef_game();
    let last = g.private_hotel(h).unwrap().last_month.unwrap();
    let k = g.hotel_kpi(h).unwrap().pop().unwrap();
    assert_eq!(last.round, k.round);
    assert_eq!(last.occupancy_bp, k.kpi.occupancy_bp);
    assert_eq!(last.adr_cents, k.kpi.adr_cents);
    assert_eq!(last.revpar_cents, k.kpi.revpar_cents);
}
