//! Zajednički ulazi testova AI sloja (dijele ih `request.rs` i `live.rs`).
#![allow(dead_code)]

use game::dto::MonthCode;
use game::view::Seat;
use game::{
    CurvePoint, HotelId, RoundKey, SeasonDebriefInput, SeasonDecisions, SeasonFinances,
    SeasonHotel, SeasonMentorHotel, SeasonRound,
};
use sim_hotel::Month;
use uuid::Uuid;

fn decisions(wage: u16, training: u8, marketing_cents: i64) -> SeasonDecisions {
    SeasonDecisions {
        avg_price_cents: 13_500,
        marketing_total_cents: marketing_cents,
        wage_pct_min: wage,
        wage_pct_max: wage,
        seasonal_peak: 28,
        training_rounds: training,
        maintenance_deferred_rounds: 0,
        renovation_round: None,
        reply_reviews_rounds: 0,
        situations: 4,
        unanswered: 1,
        avg_quality_hundredths: Some(275),
    }
}

fn hotel(
    n: u128,
    name: &str,
    seat: Seat,
    rank: u8,
    progress: i32,
    d: SeasonDecisions,
) -> SeasonMentorHotel {
    let months = [Month::Jun, Month::Jul, Month::Aug, Month::Sep];
    let curve = months
        .iter()
        .zip([progress / 4, progress / 2, progress - 100, progress])
        .map(|(&m, p)| CurvePoint {
            round: RoundKey::Month(m),
            progress_hundredths: p,
        })
        .collect();
    SeasonMentorHotel {
        hotel: SeasonHotel {
            hotel_id: HotelId(Uuid::from_u128(n)),
            slot: u8::try_from(n).unwrap(),
            name: name.into(),
            mine: matches!(seat, Seat::Human),
            seat,
            rank,
            winner: rank == 1,
            progress_hundredths: progress,
            start: [6_000, 7_500, 3_000, 5_000, 3_500],
            end: [5_300, 5_800, 4_500, 5_800, 2_800],
            curve,
            finances: Some(SeasonFinances {
                total_revenue_cents: 237_214_600,
                gop_cents: 71_075_800,
                gop_margin_bp: Some(2_996),
                cash_start_cents: 3_700_000,
                cash_end_cents: 49_975_800,
            }),
            decisions: Some(d),
        },
        price_ref_cents: 15_300,
    }
}

/// Mjesečna sezona: Tvoj hotel drugi, Brižni pobjednik (plaće i ostalo nepromijenjeno od početka).
pub fn sample_season() -> SeasonDebriefInput {
    let robot = |s: &str| Seat::Robot { strategy: s.into() };
    let rounds = [
        (Month::Jun, MonthCode::Jun, 30),
        (Month::Jul, MonthCode::Jul, 31),
        (Month::Aug, MonthCode::Aug, 31),
        (Month::Sep, MonthCode::Sep, 30),
    ]
    .into_iter()
    .map(|(m, c, last)| SeasonRound {
        key: RoundKey::Month(m),
        first_date: (1, c),
        last_date: (last, c),
    })
    .collect();
    SeasonDebriefInput {
        rounds,
        me: hotel(1, "Tvoj hotel", Seat::Human, 2, -420, decisions(100, 0, 0)),
        others: vec![
            hotel(
                2,
                "Brižni",
                robot("brizni"),
                1,
                910,
                decisions(103, 4, 2_000_000),
            ),
            hotel(
                3,
                "Lovac na popunjenost",
                robot("lovac"),
                3,
                -480,
                decisions(100, 0, 10_000_000),
            ),
            hotel(4, "Štediša", robot("stedisa"), 4, -600, decisions(95, 0, 0)),
        ],
        wage_start_pct: 100,
        own_plan_rounds: Vec::new(),
    }
}
