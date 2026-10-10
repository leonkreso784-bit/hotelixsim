//! Obračun kruga u fazama (DEMO §3, ADR-013 t. 4): osoblje, tržište, ljudi, novac, oprema i gosti. Svaka faza je
//! privatna funkcija s vlastitim ulazom i izlazom; `resolve_period` ih samo slaže, piše objašnjenje (`lines`) i uz
//! v2 stanje daje v1 sažetak (`PeriodReport::summary`). Krug je tjedan ili mjesec: mjesečni iznosi (plaće, fiksno,
//! održavanje, obuka, trošenje opreme, rast iskustva, odgovaranje) idu razmjerno (`prorate`), jednokratni (obnova,
//! situacije, najam zaposlenika) u punom iznosu; u mjesečnom krugu svako pravilo daje točno v1 vrijednost.
//!
//! Faze žive u podmodulima: `situations` (učinci situacija, grupe), `people` (osoblje i ljudi), `money` (novac),
//! `guests` (oprema i gosti); tržište je `crate::market`. Ovdje su zajednički tipovi i redoslijed faza.

mod guests;
mod money;
mod people;
mod situations;

pub(crate) use guests::GuestStep;
pub(crate) use money::MoneyStep;
pub(crate) use people::PeopleStep;

use guests::{GuestIn, guests};
use money::{MoneyIn, money};
use people::{PeopleIn, people, staffing};
use situations::{group_done, situation_effects};

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{
    CatalogError, LevelEffect, MechanicError, MonthReport, Pending, SituationOutcome,
    TrainingEffect,
};
use sim_hotel::{Maintenance, Training};

use crate::calendar::{Period, avg_i64, prorate};
use crate::catalog::Catalog;
use crate::explain::{Explain, Lines, explain};
use crate::market::market;
use crate::plan::{PeriodPlan, PlanError};
use crate::state::HotelState;

/// Izvješće kruga: v2 stanje prije i poslije te v1 sažetak (isti oblik kao v1 `MonthReport`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodReport {
    pub period: Period,
    pub before: HotelState,
    pub after: HotelState,
    pub summary: MonthReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    Calc(CalcError),
    Plan(PlanError),
    Catalog(CatalogError),
    Mechanic(MechanicError),
    /// Izbor odgovora ne postoji (prazan popis ili indeks izvan popisa).
    BadChoice,
}

impl From<CalcError> for ResolveError {
    fn from(e: CalcError) -> Self {
        ResolveError::Calc(e)
    }
}
impl From<PlanError> for ResolveError {
    fn from(e: PlanError) -> Self {
        ResolveError::Plan(e)
    }
}
impl From<CatalogError> for ResolveError {
    fn from(e: CatalogError) -> Self {
        ResolveError::Catalog(e)
    }
}
impl From<MechanicError> for ResolveError {
    fn from(e: MechanicError) -> Self {
        ResolveError::Mechanic(e)
    }
}
/// v1 funkcije koje v2 ponovno koristi (`Strategy::choose`, `base_labor_cents`) vraćaju v1 grešku.
impl From<sim_hotel::ResolveError> for ResolveError {
    fn from(e: sim_hotel::ResolveError) -> Self {
        match e {
            sim_hotel::ResolveError::Calc(e) => ResolveError::Calc(e),
            sim_hotel::ResolveError::Plan(e) => ResolveError::Plan(e.into()),
            sim_hotel::ResolveError::Catalog(e) => ResolveError::Catalog(e),
            sim_hotel::ResolveError::Mechanic(e) => ResolveError::Mechanic(e),
            sim_hotel::ResolveError::BadChoice => ResolveError::BadChoice,
        }
    }
}

pub(crate) fn w(x: impl Into<i128>) -> i128 {
    x.into()
}

pub(crate) fn neg(x: i128) -> Calc<i128> {
    c::sub(0, x)
}

/// Pomak stanja prema cilju u krugu: `round(Δ·step_bp/10 000)` (moral, zadovoljstvo, ocjena). Tjedni koeficijenti su
/// manji od mjesečnih, pa bi se mali razmak zaokružio na 0 i stanje bi zauvijek stalo ispod cilja; zato je u krugu
/// koji nije cijeli kalendarski mjesec najmanji pomak 1 jedinica prema cilju (nikad preko cilja; 🟨 H1 korak 4).
/// Mjesečni krug je nepromijenjen (v1).
pub(super) fn step_toward(now: i128, target: i128, step_bp: u16, p: &Period) -> Calc<i128> {
    let gap = c::sub(target, now)?;
    let step = c::div_round(c::mul(w(step_bp), gap)?, 10_000)?;
    if step == 0 && gap != 0 && !p.is_whole_month() {
        return Ok(gap.signum());
    }
    Ok(step)
}

/// Obuka za krug: trošak i oba odgođena učinka razmjerno krugu (mjesečni iznosi iz kataloga).
fn training_effect(t: Training, cat: &sim_hotel::Catalog, p: &Period) -> Calc<TrainingEffect> {
    let monthly = match t {
        Training::None => TrainingEffect {
            cost_cents: 0,
            experience_pts: 0,
            morale_pts: 0,
        },
        Training::Basic => cat.actions.training_basic.clone(),
        Training::Full => cat.actions.training_full.clone(),
    };
    Ok(TrainingEffect {
        cost_cents: prorate(monthly.cost_cents, p)?,
        experience_pts: c::narrow(w(prorate(i64::from(monthly.experience_pts), p)?))?,
        morale_pts: c::narrow(w(prorate(i64::from(monthly.morale_pts), p)?))?,
    })
}

/// Održavanje za krug: trošak i bodovi razine razmjerno krugu.
fn maintenance_effect(m: Maintenance, cat: &sim_hotel::Catalog, p: &Period) -> Calc<LevelEffect> {
    let monthly = match m {
        Maintenance::Deferred => cat.actions.maintenance_deferred.clone(),
        Maintenance::Regular => cat.actions.maintenance_regular.clone(),
        Maintenance::Intensive => cat.actions.maintenance_intensive.clone(),
    };
    Ok(LevelEffect {
        cost_cents: prorate(monthly.cost_cents, p)?,
        equipment_pts: c::narrow(w(prorate(i64::from(monthly.equipment_pts), p)?))?,
    })
}

pub fn resolve_period(
    state: &HotelState,
    p: &Period,
    plan: &PeriodPlan,
    situations: &[SituationOutcome],
    cat: &Catalog,
) -> Result<PeriodReport, ResolveError> {
    plan.validate(state, p, cat)?;
    let base = &cat.base;
    let coeffs = cat.coeffs(p.kind)?;
    let pend = state.pending.clone();
    let spec = base.category(state.category)?;

    let sit = situation_effects(situations, p, base, state.category)?;

    let st = staffing(state, plan)?;
    let mut s = st.state;
    s.groups.extend(sit.groups);
    s.extra_wage_cents = c::narrow(c::add(w(s.extra_wage_cents), sit.extra_wage)?)?;
    let mkt = market(&s, p, plan, st.staff, base)?;
    s.groups.retain(|g| !group_done(g.month, p));

    let pe = people(
        &PeopleIn {
            state: &s,
            staff: st.staff,
            hires: st.hires,
            released: st.released,
            sold: mkt.sold,
            period: p,
            morale_step_bp: coeffs.morale_step_bp,
            wage_pct: plan.wage_pct,
            pend: &pend,
            sit_morale: sit.morale,
        },
        base,
    )?;
    let mo = money(
        &MoneyIn {
            spec,
            state: &s,
            market: &mkt,
            plan,
            staff: st.staff,
            hires: st.hires,
            departures: pe.out.departures,
            period: p,
            sit_cost: sit.cost,
        },
        base,
    )?;
    let gu = guests(
        &GuestIn {
            state: &s,
            plan,
            pend: &pend,
            period: p,
            satisfaction_step_bp: coeffs.satisfaction_step_bp,
            rating_step_bp: coeffs.rating_step_bp,
            price_ref_cents: avg_i64(&spec.price_ref_cents, p)?,
            load: pe.load,
            penalty: pe.penalty,
            morale: pe.morale,
            experience: pe.experience,
            sit_equipment: sit.equipment,
            sit_satisfaction: sit.satisfaction,
            sit_rating: sit.rating,
        },
        base,
    )?;

    let mut lines = Lines::default();
    explain(
        &mut lines,
        &Explain {
            before: state,
            pend: &pend,
            market: &mkt,
            plan,
            people: &pe,
            money: &mo,
            guests: &gu,
            situations: &sit.shown,
            sit_extra_wage: sit.shown_extra_wage,
        },
    )?;

    let mut after = s;
    after.morale_pts = c::narrow(pe.morale)?;
    after.experience_pts = c::narrow(pe.experience)?;
    after.equipment_pts = c::narrow(gu.equipment)?;
    after.satisfaction_pts = c::narrow(gu.satisfaction)?;
    after.rating = c::narrow(gu.rating)?;
    after.cash_cents = c::narrow(mo.cash)?;
    after.load_hundredths = pe.out.load_hundredths;
    after.turnover_bp = pe.out.turnover_bp;
    after.renovated = after.renovated || plan.renovation;
    let training = training_effect(plan.training, base, p)?;
    after.pending = Pending {
        experience_pts: training.experience_pts,
        morale_pts: training.morale_pts,
        renovation: plan.renovation,
    };

    let summary = MonthReport {
        month: p.start_month(),
        before: state.summary(),
        after: after.summary(),
        market: mkt,
        people: pe.out,
        money: mo.out,
        guests: gu.out,
        lines: lines.0,
    };
    Ok(PeriodReport {
        period: *p,
        before: state.clone(),
        after,
        summary,
    })
}
