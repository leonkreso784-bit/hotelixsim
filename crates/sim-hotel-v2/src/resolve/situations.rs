//! Faza situacija (0): zbroj učinaka odgovora na situacije kruga i nove grupe; kraj grupe po razdoblju.

use engine::calc::{self as c};
use sim_hotel::{Category, Effects, GroupBlock, SituationOutcome, effects};

use super::{ResolveError, w};
use crate::calendar::{Period, prorate};

/// Zbroj učinaka svih situacija kruga; `shown` su učinci kako ih objašnjenje prikazuje (povišice razmjerno).
pub(super) struct SituationSum {
    pub(super) shown: Vec<(String, Effects)>,
    pub(super) groups: Vec<GroupBlock>,
    pub(super) satisfaction: i128,
    pub(super) morale: i128,
    pub(super) rating: i128,
    pub(super) equipment: i128,
    pub(super) cost: i128,
    pub(super) extra_wage: i128,
    pub(super) shown_extra_wage: i128,
}

pub(super) fn situation_effects(
    situations: &[SituationOutcome],
    p: &Period,
    base: &sim_hotel::Catalog,
    category: Category,
) -> Result<SituationSum, ResolveError> {
    let mut sits: Vec<(String, Effects)> = Vec::with_capacity(situations.len());
    let mut new_groups = Vec::new();
    let (mut sat, mut morale, mut rating, mut equip, mut cost, mut extra): (
        i128,
        i128,
        i128,
        i128,
        i128,
        i128,
    ) = (0, 0, 0, 0, 0, 0);
    let mut shown_extra: i128 = 0;
    // Situacija pripada svom mjesecu: krug nosi situacije mjeseca čiji je 1. dan u njemu (`situations_for`), pa
    // t05 (29. 6. – 5. 7.) obračunava srpanjske situacije sa srpnjem (`GroupInPast`). Krug bez 1. dana mjeseca
    // nema situacija; zamjena je početni mjesec (u mjesečnom krugu je to uvijek isti mjesec, kao u v1).
    let month = p.month_starting_in().unwrap_or(p.start_month());
    for o in situations {
        let e = effects(o, month, base, category)?;
        sat = c::add(sat, w(e.satisfaction_pts))?;
        morale = c::add(morale, w(e.morale_pts))?;
        rating = c::add(rating, w(e.rating))?;
        equip = c::add(equip, w(e.equipment_pts))?;
        cost = c::add(cost, w(e.cost_cents))?;
        extra = c::add(extra, w(e.extra_wage_cents))?;
        new_groups.extend(e.groups.iter().cloned());
        // Trajne povišice plaća razmjerno krugu (kao i ostale plaće); ostali učinci situacija su jednokratni.
        let mut shown = e;
        shown.extra_wage_cents = prorate(shown.extra_wage_cents, p)?;
        shown_extra = c::add(shown_extra, w(shown.extra_wage_cents))?;
        sits.push((o.code.clone(), shown));
    }
    Ok(SituationSum {
        shown: sits,
        groups: new_groups,
        satisfaction: sat,
        morale,
        rating,
        equipment: equip,
        cost,
        extra_wage: extra,
        shown_extra_wage: shown_extra,
    })
}

/// Grupa za mjesec `m` je obračunana kad krug završi taj mjesec ili kasniji (blokada se iz stanja briše tada).
pub(super) fn group_done(m: sim_hotel::Month, p: &Period) -> bool {
    p.parts()
        .last()
        .is_some_and(|last| m < last.month || (m == last.month && p.ends_month()))
}
