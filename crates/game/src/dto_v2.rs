//! Serde-zrcala tipova jezgre v2 (jezgra nema serde). Polje za polje, kao `dto.rs`; v2 dijelovi koje v1 nema
//! (Task 11: odjeli) dolaze ovamo zajedno sa stanjem.

use serde::{Deserialize, Serialize};
use sim_hotel::{Category, GroupBlock, Pending};
use sim_hotel_v2::{HotelState, PeriodPlan};

use crate::dto::{GroupBlockDto, PendingDto, PlanDto};
use crate::error::{GameError, corrupt};

/// v2 dio obračuna kruga uz v1 sažetak (`HotelResolution.report`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailDto {
    /// Stanje v2 nakon kruga; iz njega kreće idući krug.
    pub after: HotelStateV2Dto,
}

/// Serde-zrcalo `sim_hotel_v2::HotelState` (polje za polje; Task 11 ga mijenja zajedno sa stanjem).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelStateV2Dto {
    pub category: String,
    pub rooms: u16,
    pub permanent: u16,
    pub seasonal: u16,
    pub wage_pct: u16,
    pub extra_wage_cents: i64,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub equipment_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub fixed_cents: i64,
    pub load_hundredths: u16,
    pub turnover_bp: u16,
    pub renovated: bool,
    pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
}

impl From<&HotelState> for HotelStateV2Dto {
    fn from(s: &HotelState) -> Self {
        HotelStateV2Dto {
            category: s.category.code().to_owned(),
            rooms: s.rooms,
            permanent: s.permanent,
            seasonal: s.seasonal,
            wage_pct: s.wage_pct,
            extra_wage_cents: s.extra_wage_cents,
            morale_pts: s.morale_pts,
            experience_pts: s.experience_pts,
            equipment_pts: s.equipment_pts,
            satisfaction_pts: s.satisfaction_pts,
            rating: s.rating,
            cash_cents: s.cash_cents,
            fixed_cents: s.fixed_cents,
            load_hundredths: s.load_hundredths,
            turnover_bp: s.turnover_bp,
            renovated: s.renovated,
            pending: PendingDto {
                experience_pts: s.pending.experience_pts,
                morale_pts: s.pending.morale_pts,
                renovation: s.pending.renovation,
            },
            groups: s
                .groups
                .iter()
                .map(|g| GroupBlockDto {
                    month: g.month.into(),
                    room_nights: g.room_nights,
                    price_cents: g.price_cents,
                })
                .collect(),
        }
    }
}

impl TryFrom<&HotelStateV2Dto> for HotelState {
    type Error = GameError;

    fn try_from(d: &HotelStateV2Dto) -> Result<Self, GameError> {
        let category = Category::from_code(&d.category)
            .ok_or_else(|| corrupt(format!("kategorija {}", d.category)))?;
        Ok(HotelState {
            category,
            rooms: d.rooms,
            permanent: d.permanent,
            seasonal: d.seasonal,
            wage_pct: d.wage_pct,
            extra_wage_cents: d.extra_wage_cents,
            morale_pts: d.morale_pts,
            experience_pts: d.experience_pts,
            equipment_pts: d.equipment_pts,
            satisfaction_pts: d.satisfaction_pts,
            rating: d.rating,
            cash_cents: d.cash_cents,
            fixed_cents: d.fixed_cents,
            load_hundredths: d.load_hundredths,
            turnover_bp: d.turnover_bp,
            renovated: d.renovated,
            pending: Pending {
                experience_pts: d.pending.experience_pts,
                morale_pts: d.pending.morale_pts,
                renovation: d.pending.renovation,
            },
            groups: d
                .groups
                .iter()
                .map(|g| GroupBlock {
                    month: g.month.into(),
                    room_nights: g.room_nights,
                    price_cents: g.price_cents,
                })
                .collect(),
        })
    }
}

impl From<&PeriodPlan> for PlanDto {
    fn from(p: &PeriodPlan) -> Self {
        PlanDto {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
        }
    }
}

impl From<&PlanDto> for PeriodPlan {
    fn from(p: &PlanDto) -> Self {
        PeriodPlan {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
        }
    }
}
