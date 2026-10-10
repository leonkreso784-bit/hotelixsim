//! Faze osoblja (1) i ljudi (3): sezonci na cilj, opterećenje, moral, odlasci i iskustvo.

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{Pending, PeopleOutcome, turnover_bp};

use super::{neg, w};
use crate::calendar::{Period, prorate};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

// ---------------------------------------------------------------- faza 1: osoblje

pub(super) struct Staffing {
    pub(super) state: HotelState,
    pub(super) hires: u16,
    pub(super) released: u16,
    pub(super) staff: u16,
}

/// Sezonci se dovode na cilj, plaća se postavlja; `pending` se troši ovaj mjesec (zato ga ovdje čistimo).
pub(super) fn staffing(state: &HotelState, plan: &PeriodPlan) -> Calc<Staffing> {
    let mut s = state.clone();
    let hires = plan.seasonal_target.saturating_sub(state.seasonal);
    let released = state.seasonal.saturating_sub(plan.seasonal_target);
    s.seasonal = plan.seasonal_target;
    s.wage_pct = plan.wage_pct;
    let staff = s
        .permanent
        .checked_add(s.seasonal)
        .ok_or(CalcError::Overflow)?;
    s.pending = Pending::default();
    Ok(Staffing {
        state: s,
        hires,
        released,
        staff,
    })
}

// ---------------------------------------------------------------- faza 3: ljudi

pub(crate) struct PeopleStep {
    pub(crate) out: PeopleOutcome,
    pub(crate) morale: i128,
    pub(crate) experience: i128,
    pub(crate) penalty: i128,
    pub(crate) load: i128,
    pub(crate) release_morale: i128,
    /// Rast iskustva u ovom krugu (mjesečnih +300 razmjerno).
    pub(crate) growth: i128,
}

pub(super) struct PeopleIn<'a> {
    pub(super) state: &'a HotelState,
    pub(super) staff: u16,
    pub(super) hires: u16,
    pub(super) released: u16,
    pub(super) sold: u32,
    pub(super) period: &'a Period,
    pub(super) morale_step_bp: u16,
    pub(super) wage_pct: u16,
    pub(super) pend: &'a Pending,
    pub(super) sit_morale: i128,
}

pub(super) fn people(i: &PeopleIn, cat: &sim_hotel::Catalog) -> Calc<PeopleStep> {
    let days = w(i.period.days);
    let staff = w(i.staff);
    let load = if staff == 0 {
        300
    } else {
        c::div_round(c::mul(w(i.sold), 100)?, c::mul(staff, days)?)?.clamp(0, 300)
    };
    let penalty = c::mul(100, c::sub(load, 180)?)?.clamp(0, 6_000);
    let release_morale = if i.released > 0 {
        w(cat.actions.release_morale_pts)
    } else {
        0
    };
    let wage_gain = c::mul(200, c::sub(w(i.wage_pct), 100)?)?;
    let morale_target = c::sum(&[
        5_500,
        wage_gain,
        neg(c::div_round(penalty, 2)?)?,
        w(i.pend.morale_pts),
        i.sit_morale,
        release_morale,
    ])?
    .clamp(0, 10_000);
    let morale_now = w(i.state.morale_pts);
    let morale_gain = c::div_round(
        c::mul(w(i.morale_step_bp), c::sub(morale_target, morale_now)?)?,
        10_000,
    )?;
    let morale = c::add(morale_now, morale_gain)?.clamp(0, 10_000);
    let turnover = turnover_bp(c::narrow(morale)?)?;
    // Odlasci: mjesečni broj odlazaka raspoređen po danima kruga (`prorate`; u cijelom mjesecu je identitet), da
    // tjedni krug ne zaokružuje svaki tjedan zasebno na 0.
    let monthly_departures = c::div_round(c::mul(staff, w(turnover))?, 10_000)?;
    let departures = w(prorate(c::narrow(monthly_departures)?, i.period)?).clamp(0, staff);
    let replaced = c::add(departures, w(i.hires))?.min(staff);
    let exp_now = w(i.state.experience_pts);
    let exp_mix = if staff == 0 {
        exp_now
    } else {
        c::div_round(
            c::add(
                c::mul(exp_now, c::sub(staff, replaced)?)?,
                c::mul(2_000, replaced)?,
            )?,
            staff,
        )?
    };
    // v1 mjesečna konstanta +300, razmjerno krugu.
    let growth = w(prorate(300, i.period)?);
    let experience = c::sum(&[exp_mix, growth, w(i.pend.experience_pts)])?.clamp(0, 10_000);
    Ok(PeopleStep {
        out: PeopleOutcome {
            hires: i.hires,
            released: i.released,
            staff: i.staff,
            load_hundredths: c::narrow(load)?,
            load_penalty_pts: c::narrow(penalty)?,
            morale_target_pts: c::narrow(morale_target)?,
            turnover_bp: turnover,
            departures: c::narrow(departures)?,
            experience_mix_pts: c::narrow(exp_mix)?,
        },
        morale,
        experience,
        penalty,
        load,
        release_morale,
        growth,
    })
}
