//! Serde-zrcala tipova jezgre v2 (jezgra nema serde). Polje za polje, kao `dto.rs`; v2 dijelovi koje v1 nema
//! (odjeli) dolaze ovamo zajedno sa stanjem.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sim_hotel::{Category, GroupBlock, Pending};
use sim_hotel_v2::department::{DeptCode, DeptPlan, DeptState, UsaliClass};
use sim_hotel_v2::{HotelState, PeriodPlan};

use crate::dto::{GroupBlockDto, MaintenanceCode, PendingDto, PlanDto};
use crate::error::{GameError, corrupt};

/// v2 dio obračuna kruga uz v1 sažetak (`HotelResolution.report`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailDto {
    /// Stanje v2 nakon kruga; iz njega kreće idući krug.
    pub after: HotelStateV2Dto,
}

/// Serde-zrcalo `sim_hotel_v2::department::DeptState`; `class` je kod `UsaliClass::code`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeptStateDto {
    pub class: String,
    pub open: bool,
    pub permanent: u16,
    pub seasonal: u16,
    pub condition_pts: i32,
    pub satisfaction_pts: i32,
}

/// Serde-zrcalo `sim_hotel_v2::department::DeptPlan` (postavke jednog odjela u planu kruga).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeptPlanDto {
    pub open: bool,
    pub seasonal_target: u16,
    pub price_level_pct: u16,
    pub maintenance: MaintenanceCode,
}

/// Serde-zrcalo `sim_hotel_v2::HotelState` (polje za polje): odjeli umjesto v1 agregata osoblja, opreme i
/// stalnog troška (v1 oblik daje `HotelState::summary`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelStateV2Dto {
    pub category: String,
    pub rooms: u16,
    pub wage_pct: u16,
    pub extra_wage_cents: i64,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub load_hundredths: u16,
    pub turnover_bp: u16,
    pub renovated: bool,
    pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
    pub departments: BTreeMap<String, DeptStateDto>,
    pub undistributed_fixed_cents: BTreeMap<String, i64>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
}

impl From<&DeptState> for DeptStateDto {
    fn from(d: &DeptState) -> Self {
        DeptStateDto {
            class: d.class.code().to_owned(),
            open: d.open,
            permanent: d.permanent,
            seasonal: d.seasonal,
            condition_pts: d.condition_pts,
            satisfaction_pts: d.satisfaction_pts,
        }
    }
}

impl TryFrom<&DeptStateDto> for DeptState {
    type Error = GameError;

    fn try_from(d: &DeptStateDto) -> Result<Self, GameError> {
        Ok(DeptState {
            class: UsaliClass::from_code(&d.class)
                .ok_or_else(|| corrupt(format!("razred odjela {}", d.class)))?,
            open: d.open,
            permanent: d.permanent,
            seasonal: d.seasonal,
            condition_pts: d.condition_pts,
            satisfaction_pts: d.satisfaction_pts,
        })
    }
}

impl From<&DeptPlan> for DeptPlanDto {
    fn from(d: &DeptPlan) -> Self {
        DeptPlanDto {
            open: d.open,
            seasonal_target: d.seasonal_target,
            price_level_pct: d.price_level_pct,
            maintenance: d.maintenance.into(),
        }
    }
}

impl From<&DeptPlanDto> for DeptPlan {
    fn from(d: &DeptPlanDto) -> Self {
        DeptPlan {
            open: d.open,
            seasonal_target: d.seasonal_target,
            price_level_pct: d.price_level_pct,
            maintenance: d.maintenance.into(),
        }
    }
}

fn by_string<V, W>(m: &BTreeMap<DeptCode, V>, f: impl Fn(&V) -> W) -> BTreeMap<String, W> {
    m.iter()
        .map(|(k, v)| (k.as_str().to_owned(), f(v)))
        .collect()
}

impl From<&HotelState> for HotelStateV2Dto {
    fn from(s: &HotelState) -> Self {
        HotelStateV2Dto {
            category: s.category.code().to_owned(),
            rooms: s.rooms,
            wage_pct: s.wage_pct,
            extra_wage_cents: s.extra_wage_cents,
            morale_pts: s.morale_pts,
            experience_pts: s.experience_pts,
            satisfaction_pts: s.satisfaction_pts,
            rating: s.rating,
            cash_cents: s.cash_cents,
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
            departments: by_string(&s.departments, |d| DeptStateDto::from(d)),
            undistributed_fixed_cents: by_string(&s.undistributed_fixed_cents, |v| *v),
            fixed_charges_cents: s.fixed_charges_cents.clone(),
        }
    }
}

impl TryFrom<&HotelStateV2Dto> for HotelState {
    type Error = GameError;

    fn try_from(d: &HotelStateV2Dto) -> Result<Self, GameError> {
        let category = Category::from_code(&d.category)
            .ok_or_else(|| corrupt(format!("kategorija {}", d.category)))?;
        let mut departments = BTreeMap::new();
        for (k, v) in &d.departments {
            departments.insert(DeptCode(k.clone()), DeptState::try_from(v)?);
        }
        Ok(HotelState {
            category,
            rooms: d.rooms,
            wage_pct: d.wage_pct,
            extra_wage_cents: d.extra_wage_cents,
            morale_pts: d.morale_pts,
            experience_pts: d.experience_pts,
            satisfaction_pts: d.satisfaction_pts,
            rating: d.rating,
            cash_cents: d.cash_cents,
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
            departments,
            undistributed_fixed_cents: d
                .undistributed_fixed_cents
                .iter()
                .map(|(k, v)| (DeptCode(k.clone()), *v))
                .collect(),
            fixed_charges_cents: d.fixed_charges_cents.clone(),
        })
    }
}

/// v2 plan u v1 obliku tijela plana: `seasonal_target` je zbroj odjela, `maintenance` razina Smještaja (za v1
/// klijente i prikaz); postavke odjela idu u `departments`.
impl From<&PeriodPlan> for PlanDto {
    fn from(p: &PeriodPlan) -> Self {
        PlanDto {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_total(),
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.rooms_maintenance().into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
            departments: by_string(&p.departments, |d| DeptPlanDto::from(d)),
        }
    }
}

/// v2 plan traži postavke odjela; `seasonal_target` i `maintenance` iz DTO-a se ne čitaju (izvode se iz odjela).
/// Prazni `departments` ovdje je pokvaren dnevnik: zahtjev bez odjela dopunjuje `Game::plan_from_dto`.
impl TryFrom<&PlanDto> for PeriodPlan {
    type Error = GameError;

    fn try_from(p: &PlanDto) -> Result<Self, GameError> {
        if p.departments.is_empty() {
            return Err(corrupt("v2 plan bez odjela"));
        }
        Ok(PeriodPlan {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
            departments: p
                .departments
                .iter()
                .map(|(k, v)| (DeptCode(k.clone()), DeptPlan::from(v)))
                .collect(),
        })
    }
}
