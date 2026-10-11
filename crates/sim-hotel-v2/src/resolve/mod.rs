//! Obračun kruga u fazama (DEMO §3, ADR-013 t. 4): osoblje, tržište, ljudi, novac, oprema i gosti. Svaka faza je
//! privatna funkcija s vlastitim ulazom i izlazom; `resolve_period` ih samo slaže, piše objašnjenje (`lines`) i uz
//! v2 stanje daje v1 sažetak (`PeriodReport::summary`). Krug je tjedan ili mjesec: mjesečni iznosi (plaće, fiksno,
//! održavanje, obuka, trošenje opreme, rast iskustva, odgovaranje) idu razmjerno (`prorate`), jednokratni (obnova,
//! situacije, najam zaposlenika) u punom iznosu; u mjesečnom krugu svako pravilo daje točno v1 vrijednost.
//!
//! Faze žive u podmodulima: `situations` (učinci situacija, grupe), `people` (osoblje i ljudi), `departments`
//! (računi odjela po USALI-ju, neraspoređeni odjeli, fiksne naknade), `money` (novac), `guests` (oprema i gosti);
//! tržište je `crate::market`. Ovdje su zajednički tipovi i redoslijed faza. v1 sažetak se slaže iz računa odjela.

mod departments;
mod guests;
mod money;
mod people;
mod situations;

pub(crate) use guests::GuestStep;
pub(crate) use money::MoneyStep;
pub(crate) use people::PeopleStep;

use departments::{
    OperatedIn, RoomsIn, fixed_charges, operated, rooms_outcome, rooms_payroll, undistributed,
};
use guests::{GuestIn, guests};
use money::{MoneyIn, decisions, money, rooms_maintenance_cost};
use people::{LoadUnit, PeopleIn, hotel_load, people, rooms_load, staffing};
use situations::{group_done, situation_effects};

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{
    CatalogError, LevelEffect, MechanicError, MonthReport, Pending, SituationOutcome,
    TrainingEffect,
};
use sim_hotel::{Maintenance, Training};

use crate::calendar::{Period, avg_i64, prorate};
use crate::catalog::Catalog;
use crate::department::{DeptCode, PeriodAccounts};
use crate::explain::{Explain, Lines, explain};
use crate::market::market;
use crate::plan::{PeriodPlan, PlanError};
use crate::state::HotelState;

/// Izvješće kruga: v2 stanje prije i poslije, v1 sažetak (isti oblik kao v1 `MonthReport`, složen iz odjela) i
/// računi odjela kruga (USALI).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodReport {
    pub period: Period,
    pub before: HotelState,
    pub after: HotelState,
    pub summary: MonthReport,
    pub accounts: PeriodAccounts,
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

/// Ponderirani prosjek `div_round(Σ težina·vrijednost, Σ težina)`; bez ikakve težine `fallback`.
pub(super) fn weighted(items: &[(u32, i128)], fallback: i128) -> Calc<i128> {
    let (mut num, mut den): (i128, i128) = (0, 0);
    for (weight, v) in items {
        num = c::add(num, c::mul(w(*weight), *v)?)?;
        den = c::add(den, w(*weight))?;
    }
    if den == 0 {
        return Ok(fallback);
    }
    c::div_round(num, den)
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

/// Obračun kruga. Redoslijed: osoblje → tržište i Smještaj (prodane noći, noćenja gostiju) → ljudi (opterećenje
/// hotela ponderirano osobljem odjela) → računi operativnih odjela → oprema i gosti (kvaliteta i zadovoljstvo
/// hotela ponderirani gostima) → račun Smještaja → neraspoređeni odjeli s troškovima odluka → fiksne naknade →
/// v1 sažetak novca iz računa → objašnjenje.
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

    // 1. osoblje, tržište i Smještaj
    let rooms_capacity = cat.dept(&DeptCode::rooms())?.capacity_tenths_per_staff_day;
    let rooms_before = state.rooms_state()?.clone();
    let st = staffing(state, plan)?;
    let mut s = st.state;
    s.groups.extend(sit.groups);
    s.extra_wage_cents = c::narrow(c::add(w(s.extra_wage_cents), sit.extra_wage)?)?;
    let mkt = market(&s, p, plan, st.rooms_staff, rooms_capacity, base)?;
    s.groups.retain(|g| !group_done(g.month, p));
    let rooms_revenue: i64 = c::narrow(c::add(
        c::mul(w(mkt.individual_sold), w(plan.price_cents))?,
        w(mkt.group_revenue_cents),
    )?)?;
    let guest_nights: u32 = c::narrow(c::div_round(
        c::mul(w(mkt.sold), w(cat.economy.guests_per_room_hundredths))?,
        100,
    )?)?;
    let ops_in = |state: &HotelState, experience_pts: i32, morale_pts: i32| {
        operated(&OperatedIn {
            state,
            period: p,
            plan,
            cat,
            guest_nights,
            rooms_revenue_cents: rooms_revenue,
            experience_pts,
            morale_pts,
            satisfaction_step_bp: coeffs.satisfaction_step_bp,
        })
    };

    // 2. ljudi: opterećenje odjela ne ovisi o iskustvu i moralu, pa se uzima iz računa s početnim vrijednostima
    let r_load = rooms_load(mkt.sold, st.rooms_staff, rooms_capacity, p)?;
    let mut units = vec![LoadUnit {
        staff: st.rooms_staff,
        capacity_tenths: rooms_capacity,
        load: r_load,
    }];
    for o in ops_in(&s, s.experience_pts, s.morale_pts)?.values() {
        units.push(LoadUnit {
            staff: o.staff,
            capacity_tenths: cat.dept(&o.code)?.capacity_tenths_per_staff_day,
            load: w(o.load_hundredths),
        });
    }
    let (load, penalty) = hotel_load(&units, r_load)?;
    let pe = people(
        &PeopleIn {
            state: &s,
            staff: st.staff,
            load,
            penalty,
            hires: st.hires,
            released: st.released,
            period: p,
            morale_step_bp: coeffs.morale_step_bp,
            wage_pct: plan.wage_pct,
            pend: &pend,
            sit_morale: sit.morale,
        },
        base,
    )?;

    // 3. operativni odjeli s iskustvom i moralom nakon faze ljudi
    let mut depts = ops_in(&s, c::narrow(pe.experience)?, c::narrow(pe.morale)?)?;

    // 4. oprema i gosti, račun Smještaja
    let r_penalty = w(sim_hotel::load_penalty(c::narrow(r_load)?)?);
    let others: Vec<(u32, i32, i32)> = depts
        .values()
        .map(|o| (o.guest_weight, o.quality_pts, o.satisfaction_target_pts))
        .collect();
    let gu = guests(
        &GuestIn {
            state: &s,
            plan,
            equipment_pts: rooms_before.condition_pts,
            pend: &pend,
            period: p,
            satisfaction_step_bp: coeffs.satisfaction_step_bp,
            rating_step_bp: coeffs.rating_step_bp,
            price_ref_cents: avg_i64(&spec.price_ref_cents, p)?,
            rooms_load: r_load,
            rooms_penalty: r_penalty,
            rooms_satisfaction_pts: rooms_before.satisfaction_pts,
            guest_nights,
            others: &others,
            morale: pe.morale,
            experience: pe.experience,
            sit_equipment: sit.equipment,
            sit_satisfaction: sit.satisfaction,
            sit_rating: sit.rating,
        },
        base,
    )?;
    let rooms_plan = plan
        .departments
        .get(&DeptCode::rooms())
        .ok_or_else(|| CatalogError::Missing("plan.departments.rooms".to_string()))?;
    let rooms = rooms_outcome(&RoomsIn {
        market: &mkt,
        guests: &gu,
        open: rooms_plan.open,
        staff: st.rooms_staff,
        price_level_pct: rooms_plan.price_level_pct,
        revenue_cents: rooms_revenue,
        payroll_cents: rooms_payroll(
            st.rooms_staff,
            cat.dept(&DeptCode::rooms())?.wage_cents,
            plan.wage_pct,
            s.extra_wage_cents,
            p,
        )?,
        cpor_cents: spec.cpor_cents,
        load: r_load,
        penalty: r_penalty,
        condition_before_pts: rooms_before.condition_pts,
        satisfaction_before_pts: rooms_before.satisfaction_pts,
        maintenance_cents: rooms_maintenance_cost(plan, p, base)?,
        guest_nights,
    })?;
    depts.insert(DeptCode::rooms(), rooms);

    // 5. neraspoređeni odjeli s troškovima odluka, 6. fiksne naknade
    let mut maintenance: i128 = 0;
    for o in depts.values() {
        maintenance = c::add(maintenance, w(o.maintenance_cents))?;
    }
    let dec = decisions(
        plan,
        (st.hires, pe.out.departures),
        c::narrow(maintenance)?,
        sit.cost,
        p,
        base,
    )?;
    let mut accounts = PeriodAccounts {
        days: p.days,
        rooms: s.rooms,
        undistributed: undistributed(&s, p, cat, mkt.sold, plan.wage_pct, &dec)?,
        departments: depts,
        fixed_charges: fixed_charges(&s, p)?,
        interest_cents: 0,
        capex_cents: 0,
    };

    // 7. v1 sažetak novca iz računa
    let mo = money(
        &MoneyIn {
            state: &s,
            plan,
            accounts: &accounts,
            decisions: &dec,
            period: p,
        },
        base,
    )?;
    accounts.interest_cents = mo.out.interest_cents;
    accounts.capex_cents = mo.out.capex_cents;

    let mut after = s;
    after.morale_pts = c::narrow(pe.morale)?;
    after.experience_pts = c::narrow(pe.experience)?;
    for (code, o) in &accounts.departments {
        let d = after
            .departments
            .get_mut(code)
            .ok_or_else(|| CatalogError::Missing(format!("state.departments.{}", code.as_str())))?;
        d.condition_pts = o.condition_after_pts;
        d.satisfaction_pts = o.satisfaction_after_pts;
    }
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

    // 8. objašnjenje: „oprema" sažetka je 70/30 (Smještaj i ostali operativni); promjenu Smještaja objašnjavaju v1
    // uzroci, ostatak ide u redak održavanja
    let (summary_before, summary_after) = (state.summary(), after.summary());
    let equipment_others = c::sub(
        c::sub(
            w(summary_after.equipment_pts),
            w(summary_before.equipment_pts),
        )?,
        c::sub(gu.equipment, w(rooms_before.condition_pts))?,
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
            equipment_others,
        },
    )?;

    let summary = MonthReport {
        month: p.start_month(),
        before: summary_before,
        after: summary_after,
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
        accounts,
    })
}
