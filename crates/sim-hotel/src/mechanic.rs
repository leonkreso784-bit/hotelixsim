//! Mehanike situacija (DEMO §6.2, ADR-013 t. 6–7): kvaliteta odgovora i mehanike daju učinke na stanje.

use engine::calc::{self as c, Calc, CalcError};

use crate::catalog::{Catalog, Category};
use crate::month::Month;
use crate::resolve::{neg, w};
use crate::state::GroupBlock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mechanic {
    ServiceRecovery {
        compensation_cents: i64,
    },
    EquipmentRepair {
        urgent: bool,
    },
    StaffRetention {
        raise_one_pct: u16,
        bonus_cents: i64,
    },
    GroupBooking {
        accept: bool,
        rooms: u16,
        nights: u16,
        discount_bp: u16,
        month: Month,
    },
    CrisisCommunication {
        backup_cents: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationOutcome {
    pub code: String,
    pub mechanics: Vec<Mechanic>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedAnswer {
    pub label: String,
    pub mechanics: Vec<Mechanic>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationTemplate {
    pub code: String,
    pub month: Month,
    pub title: String,
    pub answers: Vec<OfferedAnswer>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub satisfaction_pts: i32,
    pub morale_pts: i32,
    pub rating: i32,
    pub equipment_pts: i32,
    pub cost_cents: i64,
    pub extra_wage_cents: i64,
    pub groups: Vec<GroupBlock>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanicError {
    BadQuality(u8),
    GroupInPast,
    BadGroup,
    Calc(CalcError),
}

impl From<CalcError> for MechanicError {
    fn from(e: CalcError) -> Self {
        MechanicError::Calc(e)
    }
}

fn clamp_bound(v: i128, bound: i32) -> Calc<i128> {
    Ok(v.clamp(neg(w(bound))?, w(bound)))
}

impl Mechanic {
    /// Izravni trošak mehanike u prvom mjesecu (za robote i usporedbu odgovora).
    pub fn direct_cost_cents(&self, cat: &Catalog, category: Category) -> Calc<i64> {
        let m = &cat.mechanics;
        match self {
            Mechanic::ServiceRecovery { compensation_cents } => Ok(*compensation_cents),
            Mechanic::EquipmentRepair { urgent } => Ok(if *urgent {
                m.repair_urgent_cents
            } else {
                m.repair_temporary_cents
            }),
            Mechanic::StaffRetention {
                raise_one_pct,
                bonus_cents,
            } => {
                let spec = cat.category(category).map_err(|_| CalcError::BadDivisor)?;
                let raise = c::div_round(c::mul(w(spec.wage_cents), w(*raise_one_pct))?, 100)?;
                c::narrow(c::add(w(*bonus_cents), raise)?)
            }
            Mechanic::GroupBooking { .. } => Ok(0),
            Mechanic::CrisisCommunication { backup_cents } => Ok(*backup_cents),
        }
    }
}

impl SituationOutcome {
    pub fn from_answer(code: &str, a: &OfferedAnswer) -> SituationOutcome {
        SituationOutcome {
            code: code.to_string(),
            mechanics: a.mechanics.clone(),
            quality: a.quality,
        }
    }
}

/// Učinci jedne situacije u mjesecu `month`; zbroj mehanika ograničen je granicama kataloga.
pub fn effects(
    outcome: &SituationOutcome,
    month: Month,
    cat: &Catalog,
    category: Category,
) -> Result<Effects, MechanicError> {
    if !(1..=5).contains(&outcome.quality) {
        return Err(MechanicError::BadQuality(outcome.quality));
    }
    let m = &cat.mechanics;
    let q3 = c::sub(w(outcome.quality), 3)?;
    let (mut sat, mut morale, mut equipment, mut cost, mut extra): (i128, i128, i128, i128, i128) =
        (0, 0, 0, 0, 0);
    // Nijedna mehanika još ne mijenja ocjenu izravno; granica vrijedi za buduće.
    let rating: i128 = 0;
    let mut groups = Vec::new();
    for mech in &outcome.mechanics {
        match mech {
            Mechanic::ServiceRecovery { compensation_cents } => {
                sat = c::add(sat, c::mul(w(m.recovery_pts_per_q), q3)?)?;
                cost = c::add(cost, w(*compensation_cents))?;
            }
            Mechanic::EquipmentRepair { urgent } => {
                if *urgent {
                    cost = c::add(cost, w(m.repair_urgent_cents))?;
                } else {
                    cost = c::add(cost, w(m.repair_temporary_cents))?;
                    equipment = c::add(equipment, w(m.repair_temporary_equipment_pts))?;
                }
            }
            Mechanic::StaffRetention {
                raise_one_pct,
                bonus_cents,
            } => {
                let spec = cat
                    .category(category)
                    .map_err(|_| MechanicError::Calc(CalcError::BadDivisor))?;
                morale = c::add(morale, c::mul(w(m.retention_pts_per_q), q3)?)?;
                extra = c::add(
                    extra,
                    c::div_round(c::mul(w(spec.wage_cents), w(*raise_one_pct))?, 100)?,
                )?;
                cost = c::add(cost, w(*bonus_cents))?;
            }
            Mechanic::GroupBooking {
                accept,
                rooms,
                nights,
                discount_bp,
                month: target,
            } => {
                if !*accept {
                    continue;
                }
                let room_nights = c::mul(w(*rooms), w(*nights))?;
                if room_nights <= 0 {
                    return Err(MechanicError::BadGroup);
                }
                if *target < month {
                    return Err(MechanicError::GroupInPast);
                }
                let cut = if outcome.quality >= 4 {
                    w(m.group_good_quality_cut_bp)
                } else {
                    0
                };
                let discount = c::sub(w(*discount_bp), cut)?.max(0);
                let spec = cat
                    .category(category)
                    .map_err(|_| MechanicError::BadGroup)?;
                let price = c::div_round(
                    c::mul(
                        w(spec.price_ref_cents.get(*target)),
                        c::sub(10_000, discount)?,
                    )?,
                    10_000,
                )?;
                groups.push(GroupBlock {
                    month: *target,
                    room_nights: c::narrow(room_nights)?,
                    price_cents: c::narrow(price)?,
                });
            }
            Mechanic::CrisisCommunication { backup_cents } => {
                sat = c::add(sat, c::mul(w(m.crisis_pts_per_q), q3)?)?;
                cost = c::add(cost, w(*backup_cents))?;
            }
        }
    }
    Ok(Effects {
        satisfaction_pts: c::narrow(clamp_bound(sat, m.satisfaction_bound_pts)?)?,
        morale_pts: c::narrow(clamp_bound(morale, m.morale_bound_pts)?)?,
        rating: c::narrow(clamp_bound(rating, m.rating_bound)?)?,
        equipment_pts: c::narrow(equipment)?,
        cost_cents: c::narrow(cost)?,
        extra_wage_cents: c::narrow(extra)?,
        groups,
    })
}
