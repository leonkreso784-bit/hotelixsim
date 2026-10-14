//! Faza situacija (0): zbroj učinaka odgovora na situacije kruga i nove grupe; kraj grupe po razdoblju.

use engine::calc::{self as c};
use sim_hotel::{Category, Effects, GroupBlock, SituationOutcome, effects};

use super::{ResolveError, w};
use crate::calendar::{Period, prorate};
use crate::catalog::PeriodCoefficients;

/// Zbroj učinaka svih situacija kruga; `shown` su učinci kako ih objašnjenje prikazuje (bodovi s faktorom i
/// granicom kruga, povišice razmjerno); zbrojevi bodova su zbrojevi `shown`.
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
    coeffs: &PeriodCoefficients,
) -> Result<SituationSum, ResolveError> {
    // Situacija pripada svom mjesecu: krug nosi situacije mjeseca čiji je 1. dan u njemu (`situations_for`), pa
    // t05 (29. 6. – 5. 7.) obračunava srpanjske situacije sa srpnjem (`GroupInPast`). Krug bez 1. dana mjeseca
    // nosi mjesec početka (u mjesečnom krugu je to uvijek isti mjesec, kao u v1).
    let month = p.situation_month();
    let mut sits: Vec<(String, Effects)> = Vec::with_capacity(situations.len());
    let mut new_groups = Vec::new();
    let (mut cost, mut extra): (i128, i128) = (0, 0);
    for o in situations {
        let e = effects(o, month, base, category)?;
        cost = c::add(cost, w(e.cost_cents))?;
        extra = c::add(extra, w(e.extra_wage_cents))?;
        new_groups.extend(e.groups.iter().cloned());
        // Bodovi s faktorom kruga (J-3); troškovi, povišice i grupe puni. Trajne povišice plaća razmjerno krugu
        // (kao i ostale plaće); ostali učinci situacija su jednokratni.
        let mut shown = e;
        shown.satisfaction_pts = scaled(shown.satisfaction_pts, coeffs)?;
        shown.morale_pts = scaled(shown.morale_pts, coeffs)?;
        shown.rating = scaled(shown.rating, coeffs)?;
        shown.equipment_pts = scaled(shown.equipment_pts, coeffs)?;
        shown.extra_wage_cents = prorate(shown.extra_wage_cents, p)?;
        sits.push((o.code.clone(), shown));
    }
    if let Some(bound) = coeffs.situation_round_bound_pts {
        cap_round(&mut sits, bound, |e| &mut e.satisfaction_pts)?;
        cap_round(&mut sits, bound, |e| &mut e.morale_pts)?;
    }
    let total = |f: fn(&Effects) -> i128| -> Result<i128, ResolveError> {
        Ok(sits.iter().try_fold(0_i128, |a, (_, e)| c::add(a, f(e)))?)
    };
    Ok(SituationSum {
        satisfaction: total(|e| w(e.satisfaction_pts))?,
        morale: total(|e| w(e.morale_pts))?,
        rating: total(|e| w(e.rating))?,
        equipment: total(|e| w(e.equipment_pts))?,
        shown_extra_wage: total(|e| w(e.extra_wage_cents))?,
        shown: sits,
        groups: new_groups,
        cost,
        extra_wage: extra,
    })
}

/// Bodovi situacije uz faktor kruga (`situation_effect_bp`), zaokruženo na cijeli bod.
fn scaled(pts: i32, coeffs: &PeriodCoefficients) -> Result<i32, ResolveError> {
    Ok(c::narrow(c::div_round(
        c::mul(w(pts), w(coeffs.situation_effect_bp))?,
        10_000,
    )?)?)
}

/// Granica zbroja kruga (±`bound`) za jedno polje: višak se skida od zadnje situacije prema prvoj, samo sa
/// situacija istog predznaka kao zbroj, pa svaka ostaje između 0 i svog učinka i retci objašnjenja daju zbroj.
fn cap_round(
    sits: &mut [(String, Effects)],
    bound: u16,
    field: fn(&mut Effects) -> &mut i32,
) -> Result<(), ResolveError> {
    let mut sum: i128 = 0;
    for (_, e) in sits.iter_mut() {
        sum = c::add(sum, w(*field(e)))?;
    }
    let b = w(bound);
    let capped = sum.clamp(c::sub(0, b)?, b);
    let mut excess = c::sub(sum, capped)?;
    for (_, e) in sits.iter_mut().rev() {
        if excess == 0 {
            break;
        }
        let v = field(e);
        let x = w(*v);
        // Uzmi s ove situacije najviše njezin učinak, i to samo ako je istog predznaka kao višak.
        let take = if excess > 0 {
            x.clamp(0, excess)
        } else {
            x.clamp(excess, 0)
        };
        *v = c::narrow(c::sub(x, take)?)?;
        excess = c::sub(excess, take)?;
    }
    Ok(())
}

/// Grupa za mjesec `m` je obračunana kad krug završi taj mjesec ili kasniji (blokada se iz stanja briše tada).
pub(super) fn group_done(m: sim_hotel::Month, p: &Period) -> bool {
    p.parts()
        .last()
        .is_some_and(|last| m < last.month || (m == last.month && p.ends_month()))
}
