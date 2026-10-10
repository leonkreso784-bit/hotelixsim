//! DTO-i odjela, profila i gospodarstva modela v2 (`content/hotel/v2/{departments,profiles,economy}.json`) i
//! pretvorbe u tipove jezgre i natrag.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sim_hotel::{LevelEffect, MonthTable};
use sim_hotel_v2::department::{DeptCode, DeptSpec, RevenueModel, UsaliClass};
use sim_hotel_v2::{DeptStart, EconomyV2, FixedChargeSpec, ProfileV2};

use crate::ContentError;

/// Jedna vrijednost po mjesecu sezone (lipanj–rujan).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MonthsDto<T> {
    jun: T,
    jul: T,
    aug: T,
    sep: T,
}

impl<T: Copy> MonthsDto<T> {
    fn table(&self) -> MonthTable<T> {
        MonthTable([self.jun, self.jul, self.aug, self.sep])
    }

    fn from_table(t: &MonthTable<T>) -> Self {
        Self {
            jun: t.0[0],
            jul: t.0[1],
            aug: t.0[2],
            sep: t.0[3],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LevelDto {
    cost_cents: i64,
    equipment_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaintenanceDto {
    deferred: LevelDto,
    regular: LevelDto,
    intensive: LevelDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RevenueDto {
    Rooms,
    PerGuest {
        usage_bp: MonthsDto<u16>,
        check_cents: i64,
        price_elasticity_bp: i32,
        satisfaction_elasticity_bp: i32,
    },
    Fixed {
        monthly_cents: i64,
        per_guest_night_cents: i64,
    },
    ShareOfRooms {
        bp: u16,
    },
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeptDto {
    name: String,
    class: String,
    revenue: RevenueDto,
    cost_of_sales_bp: u16,
    direct_per_use_cents: i64,
    wage_cents: i64,
    capacity_tenths_per_staff_day: u16,
    wear_pts_per_month: i32,
    maintenance: MaintenanceDto,
    fixed_monthly_cents: i64,
    per_occupied_room_cents: i64,
    seasonal_max: u16,
    seasonal_plan: MonthsDto<u16>,
    can_close: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeptStartDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    open: bool,
    permanent: u16,
    seasonal: u16,
    condition_pts: i32,
    satisfaction_pts: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fixed_monthly_cents: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileDto {
    departments: BTreeMap<String, DeptStartDto>,
    fixed_charges_cents: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixedChargeDto {
    name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EconomyDto {
    guests_per_room_hundredths: u16,
    price_level_min_pct: u16,
    price_level_max_pct: u16,
    fixed_charges: BTreeMap<String, FixedChargeDto>,
}

fn level_to_dto(l: &LevelEffect) -> LevelDto {
    LevelDto {
        cost_cents: l.cost_cents,
        equipment_pts: l.equipment_pts,
    }
}

fn level(l: &LevelDto) -> LevelEffect {
    LevelEffect {
        cost_cents: l.cost_cents,
        equipment_pts: l.equipment_pts,
    }
}

fn revenue_to_dto(r: &RevenueModel) -> RevenueDto {
    match r {
        RevenueModel::Rooms => RevenueDto::Rooms,
        RevenueModel::PerGuest {
            usage_bp,
            check_cents,
            price_elasticity_bp,
            satisfaction_elasticity_bp,
        } => RevenueDto::PerGuest {
            usage_bp: MonthsDto::from_table(usage_bp),
            check_cents: *check_cents,
            price_elasticity_bp: *price_elasticity_bp,
            satisfaction_elasticity_bp: *satisfaction_elasticity_bp,
        },
        RevenueModel::Fixed {
            monthly_cents,
            per_guest_night_cents,
        } => RevenueDto::Fixed {
            monthly_cents: *monthly_cents,
            per_guest_night_cents: *per_guest_night_cents,
        },
        RevenueModel::ShareOfRooms { bp } => RevenueDto::ShareOfRooms { bp: *bp },
        RevenueModel::None => RevenueDto::None,
    }
}

fn revenue(r: &RevenueDto) -> RevenueModel {
    match r {
        RevenueDto::Rooms => RevenueModel::Rooms,
        RevenueDto::PerGuest {
            usage_bp,
            check_cents,
            price_elasticity_bp,
            satisfaction_elasticity_bp,
        } => RevenueModel::PerGuest {
            usage_bp: usage_bp.table(),
            check_cents: *check_cents,
            price_elasticity_bp: *price_elasticity_bp,
            satisfaction_elasticity_bp: *satisfaction_elasticity_bp,
        },
        RevenueDto::Fixed {
            monthly_cents,
            per_guest_night_cents,
        } => RevenueModel::Fixed {
            monthly_cents: *monthly_cents,
            per_guest_night_cents: *per_guest_night_cents,
        },
        RevenueDto::ShareOfRooms { bp } => RevenueModel::ShareOfRooms { bp: *bp },
        RevenueDto::None => RevenueModel::None,
    }
}

pub(crate) fn dept_to_dto(d: &DeptSpec) -> DeptDto {
    DeptDto {
        name: d.name.clone(),
        class: d.class.code().to_string(),
        revenue: revenue_to_dto(&d.revenue),
        cost_of_sales_bp: d.cost_of_sales_bp,
        direct_per_use_cents: d.direct_per_use_cents,
        wage_cents: d.wage_cents,
        capacity_tenths_per_staff_day: d.capacity_tenths_per_staff_day,
        wear_pts_per_month: d.wear_pts_per_month,
        maintenance: MaintenanceDto {
            deferred: level_to_dto(&d.maintenance[0]),
            regular: level_to_dto(&d.maintenance[1]),
            intensive: level_to_dto(&d.maintenance[2]),
        },
        fixed_monthly_cents: d.fixed_monthly_cents,
        per_occupied_room_cents: d.per_occupied_room_cents,
        seasonal_max: d.seasonal_max,
        seasonal_plan: MonthsDto::from_table(&d.seasonal_plan),
        can_close: d.can_close,
    }
}

/// Odjel iz DTO-a; nepoznat razred je `ContentError::Invalid` s imenom polja.
pub(crate) fn build_dept(code: &str, d: &DeptDto) -> Result<DeptSpec, ContentError> {
    let Some(class) = UsaliClass::from_code(&d.class) else {
        return Err(ContentError::Invalid(format!(
            "v2.departments.{code}.class: nepoznat razred '{}'",
            d.class
        )));
    };
    Ok(DeptSpec {
        name: d.name.clone(),
        class,
        revenue: revenue(&d.revenue),
        cost_of_sales_bp: d.cost_of_sales_bp,
        direct_per_use_cents: d.direct_per_use_cents,
        wage_cents: d.wage_cents,
        capacity_tenths_per_staff_day: d.capacity_tenths_per_staff_day,
        wear_pts_per_month: d.wear_pts_per_month,
        maintenance: [
            level(&d.maintenance.deferred),
            level(&d.maintenance.regular),
            level(&d.maintenance.intensive),
        ],
        fixed_monthly_cents: d.fixed_monthly_cents,
        per_occupied_room_cents: d.per_occupied_room_cents,
        seasonal_max: d.seasonal_max,
        seasonal_plan: d.seasonal_plan.table(),
        can_close: d.can_close,
    })
}

pub(crate) fn profile_to_dto(p: &ProfileV2) -> ProfileDto {
    ProfileDto {
        departments: p
            .departments
            .iter()
            .map(|(k, s)| {
                (
                    k.as_str().to_string(),
                    DeptStartDto {
                        name: s.name.clone(),
                        open: s.open,
                        permanent: s.permanent,
                        seasonal: s.seasonal,
                        condition_pts: s.condition_pts,
                        satisfaction_pts: s.satisfaction_pts,
                        fixed_monthly_cents: s.fixed_monthly_cents,
                    },
                )
            })
            .collect(),
        fixed_charges_cents: p.fixed_charges_cents.clone(),
    }
}

pub(crate) fn build_profile(p: &ProfileDto) -> ProfileV2 {
    ProfileV2 {
        departments: p
            .departments
            .iter()
            .map(|(k, s)| {
                (
                    DeptCode(k.clone()),
                    DeptStart {
                        name: s.name.clone(),
                        open: s.open,
                        permanent: s.permanent,
                        seasonal: s.seasonal,
                        condition_pts: s.condition_pts,
                        satisfaction_pts: s.satisfaction_pts,
                        fixed_monthly_cents: s.fixed_monthly_cents,
                    },
                )
            })
            .collect(),
        fixed_charges_cents: p.fixed_charges_cents.clone(),
    }
}

pub(crate) fn economy_to_dto(e: &EconomyV2) -> EconomyDto {
    EconomyDto {
        guests_per_room_hundredths: e.guests_per_room_hundredths,
        price_level_min_pct: e.price_level_min_pct,
        price_level_max_pct: e.price_level_max_pct,
        fixed_charges: e
            .fixed_charges
            .iter()
            .map(|(k, f)| {
                (
                    k.clone(),
                    FixedChargeDto {
                        name: f.name.clone(),
                    },
                )
            })
            .collect(),
    }
}

pub(crate) fn build_economy(e: &EconomyDto) -> EconomyV2 {
    EconomyV2 {
        guests_per_room_hundredths: e.guests_per_room_hundredths,
        price_level_min_pct: e.price_level_min_pct,
        price_level_max_pct: e.price_level_max_pct,
        fixed_charges: e
            .fixed_charges
            .iter()
            .map(|(k, f)| {
                (
                    k.clone(),
                    FixedChargeSpec {
                        name: f.name.clone(),
                    },
                )
            })
            .collect(),
    }
}
