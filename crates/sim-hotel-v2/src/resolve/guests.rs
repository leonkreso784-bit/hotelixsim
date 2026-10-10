//! Faza opreme i gostiju (5): trošenje i održavanje opreme, kvaliteta, vrijednost, zadovoljstvo i ocjena.

use engine::calc::{self as c, Calc};
use sim_hotel::{GuestOutcome, Pending};

use super::{maintenance_effect, step_toward, w};
use crate::calendar::{Period, prorate};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

pub(crate) struct GuestStep {
    pub(crate) out: GuestOutcome,
    pub(crate) equipment: i128,
    pub(crate) satisfaction: i128,
    pub(crate) rating: i128,
    pub(crate) maintenance_eff: i128,
    pub(crate) equipment_load: i128,
    pub(crate) renovation_equipment: i128,
    pub(crate) renovation_satisfaction: i128,
    pub(crate) reply_rating: i128,
    pub(crate) satisfaction_gain: i128,
    pub(crate) rating_gain: i128,
    /// Trošenje opreme u ovom krugu (mjesečnih -400 razmjerno).
    pub(crate) wear: i128,
}

pub(super) struct GuestIn<'a> {
    pub(super) state: &'a HotelState,
    pub(super) plan: &'a PeriodPlan,
    pub(super) pend: &'a Pending,
    pub(super) period: &'a Period,
    pub(super) satisfaction_step_bp: u16,
    pub(super) rating_step_bp: u16,
    pub(super) price_ref_cents: i64,
    pub(super) load: i128,
    pub(super) penalty: i128,
    pub(super) morale: i128,
    pub(super) experience: i128,
    pub(super) sit_equipment: i128,
    pub(super) sit_satisfaction: i128,
    pub(super) sit_rating: i128,
}

pub(super) fn guests(i: &GuestIn, cat: &sim_hotel::Catalog) -> Calc<GuestStep> {
    let p = i.period;
    let maintenance_eff = w(maintenance_effect(i.plan.maintenance, cat, p)?.equipment_pts);
    // v1 mjesečna konstanta -400, razmjerno krugu.
    let wear = w(prorate(-400, p)?);
    let equipment_load = if i.load > 180 {
        w(prorate(-100, p)?)
    } else {
        0
    };
    let (renovation_equipment, renovation_satisfaction) = if i.pend.renovation {
        (
            w(cat.actions.renovation_equipment_pts),
            w(cat.actions.renovation_satisfaction_pts),
        )
    } else {
        (0, 0)
    };
    let equipment = c::sum(&[
        w(i.state.equipment_pts),
        wear,
        equipment_load,
        maintenance_eff,
        renovation_equipment,
        i.sit_equipment,
    ])?
    .clamp(0, 10_000);
    let quality = c::div_round(
        c::sum(&[
            c::mul(4_000, i.experience)?,
            c::mul(3_000, i.morale)?,
            c::mul(3_000, c::sub(10_000, i.penalty)?)?,
        ])?,
        10_000,
    )?;
    let reference = w(i.price_ref_cents);
    let off_ref = c::div_round(
        c::mul(10_000, c::sub(w(i.plan.price_cents), reference)?)?,
        reference,
    )?;
    let value = c::sub(7_000, off_ref)?.clamp(0, 10_000);
    let sat_target = c::div_round(
        c::sum(&[
            c::mul(4_500, quality)?,
            c::mul(3_000, equipment)?,
            c::mul(2_500, value)?,
        ])?,
        10_000,
    )?;
    let sat_now = w(i.state.satisfaction_pts);
    let satisfaction_gain = step_toward(sat_now, sat_target, i.satisfaction_step_bp, p)?;
    let satisfaction = c::sum(&[
        sat_now,
        satisfaction_gain,
        renovation_satisfaction,
        i.sit_satisfaction,
    ])?
    .clamp(0, 10_000);
    let rating_target = c::div_round(c::add(14_000, satisfaction)?, 25)?;
    let rating_now = w(i.state.rating);
    let rating_gain = step_toward(rating_now, rating_target, i.rating_step_bp, p)?;
    let reply_rating = if i.plan.reply_reviews {
        w(prorate(i64::from(cat.actions.reply_rating), p)?)
    } else {
        0
    };
    let rating = c::sum(&[rating_now, rating_gain, reply_rating, i.sit_rating])?.clamp(100, 1_000);
    Ok(GuestStep {
        out: GuestOutcome {
            quality_pts: c::narrow(quality)?,
            value_pts: c::narrow(value)?,
            satisfaction_target_pts: c::narrow(sat_target)?,
            rating_target: c::narrow(rating_target)?,
        },
        equipment,
        satisfaction,
        rating,
        maintenance_eff,
        equipment_load,
        renovation_equipment,
        renovation_satisfaction,
        reply_rating,
        satisfaction_gain,
        rating_gain,
        wear,
    })
}
