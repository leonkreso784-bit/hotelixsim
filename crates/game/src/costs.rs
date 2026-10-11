//! Cijene odluka plana za ekran (Leon 12.10.: obnova je koštala 150.000 € a kvačica to nije govorila). Brojke su iz
//! kataloga, za krug kako ih obračun knjiži: v1 mjesečni iznosi; v2 razmjerno krugu (`prorate`), obnova u punom
//! iznosu. Uz to procjena tjednih plaća hotela (upozorenje uz obnovu). Ništa se ne računa drugačije nego u obračunu.

use std::collections::BTreeMap;

use sim_hotel::{LevelEffect, Month, TrainingEffect};
use sim_hotel_v2::Period;
use sim_hotel_v2::calendar::prorate;
use sim_hotel_v2::department::DeptCode;

use crate::error::{GameError, corrupt};

/// Trošak tri razine održavanja u krugu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelCosts {
    pub deferred_cents: i64,
    pub regular_cents: i64,
    pub intensive_cents: i64,
}

/// Trošak obuke u krugu i njezin odgođeni učinak (bodovi 0–10 000, stižu idući krug).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingCost {
    pub cost_cents: i64,
    pub experience_pts: i32,
    pub morale_pts: i32,
}

/// Cijene odluka plana hotela u krugu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanCosts {
    /// Obnova: jednokratno, puni iznos, i učinak na opremu i zadovoljstvo (bodovi 0–10 000).
    pub renovation_cents: i64,
    pub renovation_equipment_pts: i32,
    pub renovation_satisfaction_pts: i32,
    /// Odgovaranje na recenzije u krugu.
    pub reply_cents: i64,
    pub training_basic: TrainingCost,
    pub training_full: TrainingCost,
    /// Održavanje hotela (v1) odnosno Smještaja (v2) u krugu.
    pub maintenance: LevelCosts,
    /// Plaće cijelog hotela za 7 dana pri sadašnjem broju ljudi i postotku plaće (procjena za upozorenje).
    pub payroll_week_cents: i64,
}

fn calc<T, E: std::fmt::Debug>(r: Result<T, E>) -> Result<T, GameError> {
    r.map_err(|e| corrupt(format!("cijene plana: {e:?}")))
}

/// `monthly · 7 / dani mjeseca`, zaokruženo (procjena tjedna iz mjesečnog iznosa).
fn per_week(monthly: i128, month_days: u8) -> Result<i64, GameError> {
    let days = i128::from(month_days.max(1));
    let week = monthly
        .checked_mul(7)
        .and_then(|x| x.checked_add(days / 2))
        .and_then(|x| x.checked_div(days))
        .ok_or_else(|| corrupt("tjedne plaće"))?;
    i64::try_from(week).map_err(|_| corrupt("tjedne plaće"))
}

/// Mjesečne plaće `staff` ljudi: `staff · ⌊plaća/100⌋ · postotak` (isto pravilo kao obračun).
fn monthly_payroll(staff: u16, wage_cents: i64, wage_pct: u16) -> Result<i128, GameError> {
    i128::from(staff)
        .checked_mul(i128::from(wage_cents).div_euclid(100))
        .and_then(|x| x.checked_mul(i128::from(wage_pct)))
        .ok_or_else(|| corrupt("mjesečne plaće"))
}

fn levels(
    l: [&LevelEffect; 3],
    scale: impl Fn(i64) -> Result<i64, GameError>,
) -> Result<LevelCosts, GameError> {
    Ok(LevelCosts {
        deferred_cents: scale(l[0].cost_cents)?,
        regular_cents: scale(l[1].cost_cents)?,
        intensive_cents: scale(l[2].cost_cents)?,
    })
}

fn training(
    t: &TrainingEffect,
    scale: &impl Fn(i64) -> Result<i64, GameError>,
) -> Result<TrainingCost, GameError> {
    let pts = |x: i32| -> Result<i32, GameError> {
        i32::try_from(scale(i64::from(x))?).map_err(|_| corrupt("bodovi obuke"))
    };
    Ok(TrainingCost {
        cost_cents: scale(t.cost_cents)?,
        experience_pts: pts(t.experience_pts)?,
        morale_pts: pts(t.morale_pts)?,
    })
}

fn hotel_costs(
    a: &sim_hotel::ActionSpec,
    scale: impl Fn(i64) -> Result<i64, GameError>,
    payroll_week_cents: i64,
) -> Result<PlanCosts, GameError> {
    Ok(PlanCosts {
        renovation_cents: a.renovation_cents,
        renovation_equipment_pts: a.renovation_equipment_pts,
        renovation_satisfaction_pts: a.renovation_satisfaction_pts,
        reply_cents: scale(a.reply_cents)?,
        training_basic: training(&a.training_basic, &scale)?,
        training_full: training(&a.training_full, &scale)?,
        maintenance: levels(
            [
                &a.maintenance_deferred,
                &a.maintenance_regular,
                &a.maintenance_intensive,
            ],
            &scale,
        )?,
        payroll_week_cents,
    })
}

/// v1: mjesečni iznosi kataloga (mjesec je krug).
pub fn v1_costs(
    s: &sim_hotel::HotelState,
    m: Month,
    cat: &sim_hotel::Catalog,
) -> Result<PlanCosts, GameError> {
    let spec = calc(cat.category(s.category))?;
    let staff = s.permanent.saturating_add(s.seasonal);
    let monthly = monthly_payroll(staff, spec.wage_cents, s.wage_pct)?
        .checked_add(i128::from(s.extra_wage_cents))
        .ok_or_else(|| corrupt("mjesečne plaće"))?;
    hotel_costs(&cat.actions, Ok, per_week(monthly, m.days())?)
}

/// v2: iznosi razmjerno krugu `p`; uz to održavanje po odjelu (Smještaj iz kataloga radnji, kao obračun).
pub fn v2_costs(
    s: &sim_hotel_v2::HotelState,
    p: &Period,
    cat: &sim_hotel_v2::Catalog,
) -> Result<(PlanCosts, BTreeMap<String, LevelCosts>), GameError> {
    let scale = |x: i64| calc(prorate(x, p));
    let mut monthly = i128::from(s.extra_wage_cents);
    let mut depts = BTreeMap::new();
    for (code, d) in &s.departments {
        let spec = calc(cat.dept(code))?;
        let staff = d
            .permanent
            .saturating_add(if d.open { d.seasonal } else { 0 });
        monthly = monthly
            .checked_add(monthly_payroll(staff, spec.wage_cents, s.wage_pct)?)
            .ok_or_else(|| corrupt("mjesečne plaće"))?;
        let m = &spec.maintenance;
        let own = if *code == DeptCode::rooms() {
            let a = &cat.base.actions;
            levels(
                [
                    &a.maintenance_deferred,
                    &a.maintenance_regular,
                    &a.maintenance_intensive,
                ],
                scale,
            )?
        } else {
            levels([&m[0], &m[1], &m[2]], scale)?
        };
        depts.insert(code.as_str().to_owned(), own);
    }
    let costs = hotel_costs(
        &cat.base.actions,
        scale,
        per_week(monthly, p.month_days_at_start())?,
    )?;
    Ok((costs, depts))
}
