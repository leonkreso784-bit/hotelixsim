//! Faza situacija (0): zbroj učinaka odgovora na situacije kruga i nove grupe; kraj grupe po razdoblju. Učinak na
//! prostor (oprema) ide odjelu situacije (ADR-042 t. 1): hitan popravak diže stanje tog odjela bez faktora kruga.

use std::collections::BTreeMap;

use engine::calc::{self as c};
use sim_hotel::{Category, Effects, GroupBlock, Mechanic, effects};

use super::{ResolveError, w};
use crate::calendar::{Period, prorate};
use crate::catalog::{Catalog, PeriodCoefficients};
use crate::department::DeptCode;
use crate::situations::{FAULT_FIXED_MIN_QUALITY, RoundSituation};

/// Učinak jedne situacije u krugu: `effects` kao u obračunu (bodovi s faktorom kruga, troškovi i povišice puni) i
/// odjel čiji prostor mijenja (`effects.equipment_pts`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationEffect {
    pub effects: Effects,
    pub space: DeptCode,
}

/// Učinak jedne situacije u krugu `p`, isti račun kao obračun (bez granice zbroja kruga). Bodovi imaju faktor kruga,
/// troškovi su puni. `effects.equipment_pts` = promjena stanja prostora `space` = privremeni popravak s faktorom +
/// broj hitnih popravaka × `economy.repair_urgent_condition_pts` bez faktora (popravak je fizički, ADR-042 t. 1).
/// Kvar prostora (`s.fault`, L1-13-1): −`economy.fault_condition_pts`, a uz hitni popravak u odgovoru i ocjenu
/// od `FAULT_FIXED_MIN_QUALITY` naviše još +`economy.fault_fixed_condition_pts` (jednom po situaciji). Oba ključa su
/// bodovi situacije (Leon 18.10., K1): množe se faktorom kruga i razine kao ostali bodovi, svaki zaokružen za sebe.
/// Klik (`s.click`, L1-13-1b): zatim se pozitivni dijelovi zadovoljstva, morala, ocjene i opreme (i noćenja grupa, najmanje 1)
/// množe s `economy.click_gain_bp / 10 000`; kvar se skida tek poslije, bez umanjenja klikom.
/// Situacija bez odjela i neraspoređeni odjel djeluju na Smještaj; nepoznat odjel je greška kataloga.
pub fn situation_effect(
    s: &RoundSituation,
    p: &Period,
    cat: &Catalog,
    category: Category,
) -> Result<SituationEffect, ResolveError> {
    let coeffs = cat.coeffs(p.kind)?;
    // Situacija pripada svom mjesecu: krug nosi situacije mjeseca čiji je 1. dan u njemu (`situations_for`), pa
    // t05 (29. 6. – 5. 7.) obračunava srpanjske situacije sa srpnjem (`GroupInPast`). Krug bez 1. dana mjeseca
    // nosi mjesec početka (u mjesečnom krugu je to uvijek isti mjesec, kao u v1).
    let mut e = effects(&s.outcome, p.situation_month(), &cat.base, category)?;
    let sig = match s.significance {
        crate::situations::Significance::Key => cat.economy.significance_bp.key,
        crate::situations::Significance::Important => cat.economy.significance_bp.important,
        crate::situations::Significance::Everyday => cat.economy.significance_bp.everyday,
    };
    e.satisfaction_pts = scaled(e.satisfaction_pts, coeffs, sig)?;
    e.morale_pts = scaled(e.morale_pts, coeffs, sig)?;
    e.rating = scaled(e.rating, coeffs, sig)?;
    let urgent = s
        .outcome
        .mechanics
        .iter()
        .filter(|m| matches!(m, Mechanic::EquipmentRepair { urgent: true }))
        .count();
    let repair = c::mul(
        i128::try_from(urgent).map_err(|_| c::CalcError::Overflow)?,
        w(cat.economy.repair_urgent_condition_pts),
    )?;
    let mut equipment = c::add(w(scaled(e.equipment_pts, coeffs, sig)?), repair)?;
    if s.fault && urgent > 0 && s.outcome.quality >= FAULT_FIXED_MIN_QUALITY {
        equipment = c::add(
            equipment,
            w(scaled(cat.economy.fault_fixed_condition_pts, coeffs, sig)?),
        )?;
    }
    if s.click {
        // Klik (L1-13-1b): samo DOBIT se umanjuje; trošak, povišice i negativni dijelovi ostaju puni.
        let bp = cat.economy.click_gain_bp;
        e.satisfaction_pts = click_gain(e.satisfaction_pts, bp)?;
        e.morale_pts = click_gain(e.morale_pts, bp)?;
        e.rating = click_gain(e.rating, bp)?;
        equipment = if equipment > 0 {
            c::div_round(c::mul(equipment, w(bp))?, 10_000)?
        } else {
            equipment
        };
        for g in &mut e.groups {
            let nights = c::div_round(c::mul(w(g.room_nights), w(bp))?, 10_000)?;
            g.room_nights = u32::try_from(nights.max(1)).map_err(|_| c::CalcError::Overflow)?;
        }
    }
    if s.fault {
        equipment = c::sub(
            equipment,
            w(scaled(cat.economy.fault_condition_pts, coeffs, sig)?),
        )?;
    }
    e.equipment_pts = c::narrow(equipment)?;
    let space = match &s.space {
        Some(d) if !cat.dept(d)?.class.is_undistributed() => d.clone(),
        _ => DeptCode::rooms(),
    };
    Ok(SituationEffect { effects: e, space })
}

/// Zbroj učinaka svih situacija kruga; `shown` su učinci kako ih objašnjenje prikazuje (bodovi s faktorom i
/// granicom kruga, povišice razmjerno) uz odjel prostora; zbrojevi bodova su zbrojevi `shown`, a oprema po odjelu.
pub(super) struct SituationSum {
    pub(super) shown: Vec<(String, DeptCode, Effects)>,
    pub(super) groups: Vec<GroupBlock>,
    pub(super) satisfaction: i128,
    pub(super) morale: i128,
    pub(super) rating: i128,
    /// Promjena stanja prostora po odjelu (Smještaj kroz goste, ostali kroz račun odjela).
    pub(super) equipment: BTreeMap<DeptCode, i128>,
    pub(super) cost: i128,
    pub(super) extra_wage: i128,
    pub(super) shown_extra_wage: i128,
}

pub(super) fn situation_effects(
    situations: &[RoundSituation],
    p: &Period,
    cat: &Catalog,
    category: Category,
) -> Result<SituationSum, ResolveError> {
    let coeffs = cat.coeffs(p.kind)?;
    let mut sits: Vec<(String, DeptCode, Effects)> = Vec::with_capacity(situations.len());
    let mut new_groups = Vec::new();
    let (mut cost, mut extra): (i128, i128) = (0, 0);
    for s in situations {
        let SituationEffect { effects: e, space } = situation_effect(s, p, cat, category)?;
        cost = c::add(cost, w(e.cost_cents))?;
        extra = c::add(extra, w(e.extra_wage_cents))?;
        new_groups.extend(e.groups.iter().cloned());
        // Bodovi s faktorom kruga (J-3); troškovi, povišice i grupe puni. Trajne povišice plaća razmjerno krugu
        // (kao i ostale plaće); ostali učinci situacija su jednokratni.
        let mut shown = e;
        shown.extra_wage_cents = prorate(shown.extra_wage_cents, p)?;
        sits.push((s.outcome.code.clone(), space, shown));
    }
    if let Some(bound) = coeffs.situation_round_bound_pts {
        cap_round(&mut sits, bound, |e| &mut e.satisfaction_pts)?;
        cap_round(&mut sits, bound, |e| &mut e.morale_pts)?;
    }
    let total = |f: fn(&Effects) -> i128| -> Result<i128, ResolveError> {
        Ok(sits
            .iter()
            .try_fold(0_i128, |a, (_, _, e)| c::add(a, f(e)))?)
    };
    let mut equipment: BTreeMap<DeptCode, i128> = BTreeMap::new();
    for (_, space, e) in &sits {
        let v = equipment.entry(space.clone()).or_insert(0);
        *v = c::add(*v, w(e.equipment_pts))?;
    }
    Ok(SituationSum {
        satisfaction: total(|e| w(e.satisfaction_pts))?,
        morale: total(|e| w(e.morale_pts))?,
        rating: total(|e| w(e.rating))?,
        equipment,
        shown_extra_wage: total(|e| w(e.extra_wage_cents))?,
        shown: sits,
        groups: new_groups,
        cost,
        extra_wage: extra,
    })
}

/// Bodovi situacije uz faktor kruga (`situation_effect_bp`), zaokruženo na cijeli bod.
fn scaled(pts: i32, coeffs: &PeriodCoefficients, sig: u16) -> Result<i32, ResolveError> {
    Ok(c::narrow(c::div_round(
        c::mul(c::mul(w(pts), w(coeffs.situation_effect_bp))?, w(sig))?,
        100_000_000,
    )?)?)
}

/// Klik: pozitivan dio × `click_gain_bp / 10 000` (zaokruženo); nula i negativni dijelovi ostaju.
fn click_gain(pts: i32, bp: u16) -> Result<i32, ResolveError> {
    if pts <= 0_i32 {
        return Ok(pts);
    }
    Ok(c::narrow(c::div_round(c::mul(w(pts), w(bp))?, 10_000)?)?)
}

/// Granica zbroja kruga (±`bound`) za jedno polje: višak se skida od zadnje situacije prema prvoj, samo sa
/// situacija istog predznaka kao zbroj, pa svaka ostaje između 0 i svog učinka i retci objašnjenja daju zbroj.
fn cap_round(
    sits: &mut [(String, DeptCode, Effects)],
    bound: u16,
    field: fn(&mut Effects) -> &mut i32,
) -> Result<(), ResolveError> {
    let mut sum: i128 = 0;
    for (_, _, e) in sits.iter_mut() {
        sum = c::add(sum, w(*field(e)))?;
    }
    let b = w(bound);
    let capped = sum.clamp(c::sub(0, b)?, b);
    let mut excess = c::sub(sum, capped)?;
    for (_, _, e) in sits.iter_mut().rev() {
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
