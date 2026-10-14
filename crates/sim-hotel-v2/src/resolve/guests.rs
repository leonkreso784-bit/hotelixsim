//! Faza opreme i gostiju (5): trošenje i održavanje stanja prostora Smještaja (trošenje i razine održavanja iz
//! odjela Smještaj; u degeneriranom katalogu to su v1 brojke), obnova v2 Smještaja, kvaliteta, vrijednost i
//! zadovoljstvo Smještaja (stanje koje gost osjeća uključuje zgradu), pa kvaliteta, zadovoljstvo i ocjena hotela.
//! Kvaliteta i cilj zadovoljstva hotela su prosjeci odjela ponderirani gostima (Smještaj: noćenja; ostali:
//! usluženi); u degeneriranom katalogu goste ima samo Smještaj, pa je to točno v1.

use engine::calc::{self as c, Calc};
use sim_hotel::{GuestOutcome, Pending};

use super::{step_toward, w, weighted};
use crate::calendar::{Period, prorate};
use crate::department::{DeptSpec, after_renewal, felt_condition_mix, wear_pts};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

pub(crate) struct GuestStep {
    /// Kvaliteta i cilj zadovoljstva u `out` su hotelski (ponderirani); vrijednost je Smještaja.
    pub(crate) out: GuestOutcome,
    /// Stanje prostora Smještaja nakon kruga (v1 „oprema").
    pub(crate) equipment: i128,
    /// Kvaliteta, cilj zadovoljstva i zadovoljstvo Smještaja nakon kruga (račun odjela).
    pub(crate) rooms_quality: i128,
    pub(crate) rooms_target: i128,
    pub(crate) rooms_satisfaction: i128,
    pub(crate) satisfaction: i128,
    pub(crate) rating: i128,
    pub(crate) maintenance_eff: i128,
    pub(crate) equipment_load: i128,
    pub(crate) renovation_equipment: i128,
    pub(crate) renovation_satisfaction: i128,
    pub(crate) reply_rating: i128,
    pub(crate) satisfaction_gain: i128,
    pub(crate) rating_gain: i128,
    /// Trošenje opreme u ovom krugu (mjesečno trošenje Smještaja razmjerno).
    pub(crate) wear: i128,
    /// Bodovi koje je obnova v2 Smještaja stvarno donijela (nakon granice 100).
    pub(crate) renewal_pts: i128,
    /// Stanje Smještaja koje gost osjeća (sa zgradom, F8).
    pub(crate) felt: i32,
}

pub(super) struct GuestIn<'a> {
    pub(super) state: &'a HotelState,
    pub(super) plan: &'a PeriodPlan,
    /// Stanje prostora Smještaja na početku kruga (v1 „oprema").
    pub(super) equipment_pts: i32,
    pub(super) pend: &'a Pending,
    pub(super) period: &'a Period,
    pub(super) satisfaction_step_bp: u16,
    pub(super) rating_step_bp: u16,
    pub(super) price_ref_cents: i64,
    /// Opterećenje i kazna Smještaja (trošenje prostora i kvaliteta Smještaja).
    pub(super) rooms_load: i128,
    pub(super) rooms_penalty: i128,
    /// Zadovoljstvo gostiju Smještajem na početku kruga.
    pub(super) rooms_satisfaction_pts: i32,
    /// Noćenja gostiju: težina Smještaja u kvaliteti i zadovoljstvu hotela.
    pub(super) guest_nights: u32,
    /// Ostali odjeli s gostima: (težina, kvaliteta, cilj zadovoljstva).
    pub(super) others: &'a [(u32, i32, i32)],
    pub(super) morale: i128,
    pub(super) experience: i128,
    pub(super) sit_equipment: i128,
    pub(super) sit_satisfaction: i128,
    pub(super) sit_rating: i128,
    /// Odjel Smještaj iz kataloga (trošenje i razine održavanja).
    pub(super) rooms_spec: &'a DeptSpec,
    /// Bodovi obnove v2 Smještaja u krugu (prije granice 100).
    pub(super) renewal_delta_pts: i32,
    /// Stanje zgrade nakon kruga i njen udio u stanju koje gost osjeća.
    pub(super) building_pts: i32,
    pub(super) building_share_bp: u16,
    /// Ostali odjeli koje gost Smještaja osjeća (parking): (stanje nakon kruga, udio u bp); prazno = samo zgrada.
    pub(super) felt_parts: &'a [(i32, u16)],
    /// Faktor trošenja dok je održavanje odgođeno (bp).
    pub(super) deferred_wear_bp: u16,
}

pub(super) fn guests(i: &GuestIn, cat: &sim_hotel::Catalog) -> Calc<GuestStep> {
    let p = i.period;
    let level = i.rooms_spec.maintenance_level(i.plan.rooms_maintenance());
    let maintenance_eff = w(prorate(i64::from(level.equipment_pts), p)?);
    // Mjesečno trošenje Smještaja (v1: 400; uz odgođeno održavanje puta faktor), razmjerno krugu.
    let wear = w(wear_pts(
        i.rooms_spec.wear_pts_per_month,
        i.plan.rooms_maintenance(),
        i.deferred_wear_bp,
        p,
    )?);
    let equipment_load = if i.rooms_load > 180 {
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
    let base = c::sum(&[
        w(i.equipment_pts),
        wear,
        equipment_load,
        maintenance_eff,
        renovation_equipment,
        i.sit_equipment,
    ])?
    .clamp(0, 10_000);
    let equipment = after_renewal(base, i.renewal_delta_pts)?;
    let renewal_pts = c::sub(equipment, base)?;
    let mut parts = vec![(i.building_pts, i.building_share_bp)];
    parts.extend_from_slice(i.felt_parts);
    let felt = felt_condition_mix(c::narrow(equipment)?, &parts)?;
    let quality = c::div_round(
        c::sum(&[
            c::mul(4_000, i.experience)?,
            c::mul(3_000, i.morale)?,
            c::mul(3_000, c::sub(10_000, i.rooms_penalty)?)?,
        ])?,
        10_000,
    )?;
    let reference = w(i.price_ref_cents);
    let off_ref = c::div_round(
        c::mul(10_000, c::sub(w(i.plan.price_cents), reference)?)?,
        reference,
    )?;
    let value = c::sub(7_000, off_ref)?.clamp(0, 10_000);
    let rooms_target = c::div_round(
        c::sum(&[
            c::mul(4_500, quality)?,
            c::mul(3_000, w(felt))?,
            c::mul(2_500, value)?,
        ])?,
        10_000,
    )?;
    let rooms_sat_now = w(i.rooms_satisfaction_pts);
    let rooms_satisfaction = c::add(
        rooms_sat_now,
        step_toward(rooms_sat_now, rooms_target, i.satisfaction_step_bp, p)?,
    )?
    .clamp(0, 10_000);
    // Hotel: prosjek odjela ponderiran gostima; bez gostiju vrijedi Smještaj (v1).
    let mut qualities = vec![(i.guest_nights, quality)];
    let mut targets = vec![(i.guest_nights, rooms_target)];
    for (weight, q, t) in i.others {
        qualities.push((*weight, w(*q)));
        targets.push((*weight, w(*t)));
    }
    let hotel_quality = weighted(&qualities, quality)?;
    let sat_target = weighted(&targets, rooms_target)?;
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
            quality_pts: c::narrow(hotel_quality)?,
            value_pts: c::narrow(value)?,
            satisfaction_target_pts: c::narrow(sat_target)?,
            rating_target: c::narrow(rating_target)?,
        },
        equipment,
        rooms_quality: quality,
        rooms_target,
        rooms_satisfaction,
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
        renewal_pts,
        felt,
    })
}
