//! Obračun kruga u fazama (DEMO §3, ADR-013 t. 4): osoblje, tržište, ljudi, novac, oprema i gosti. Svaka faza je
//! privatna funkcija s vlastitim ulazom i izlazom; `resolve_period` ih samo slaže, piše objašnjenje (`lines`) i uz
//! v2 stanje daje v1 sažetak (`PeriodReport::summary`). Krug je tjedan ili mjesec: mjesečni iznosi (plaće, fiksno,
//! održavanje, obuka, trošenje opreme, rast iskustva, odgovaranje) idu razmjerno (`prorate`), jednokratni (obnova,
//! situacije, najam zaposlenika) u punom iznosu; u mjesečnom krugu svako pravilo daje točno v1 vrijednost.

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{
    CatalogError, CategorySpec, Effects, GuestOutcome, LevelEffect, MechanicError, MoneyOutcome,
    MonthReport, Pending, PeopleOutcome, SituationOutcome, TrainingEffect, effects, turnover_bp,
};
use sim_hotel::{Maintenance, MarketOutcome, Training};

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

// ---------------------------------------------------------------- faza 1: osoblje

struct Staffing {
    state: HotelState,
    hires: u16,
    released: u16,
    staff: u16,
}

/// Sezonci se dovode na cilj, plaća se postavlja; `pending` se troši ovaj mjesec (zato ga ovdje čistimo).
fn staffing(state: &HotelState, plan: &PeriodPlan) -> Calc<Staffing> {
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

struct PeopleIn<'a> {
    state: &'a HotelState,
    staff: u16,
    hires: u16,
    released: u16,
    sold: u32,
    period: &'a Period,
    morale_step_bp: u16,
    wage_pct: u16,
    pend: &'a Pending,
    sit_morale: i128,
}

fn people(i: &PeopleIn, cat: &sim_hotel::Catalog) -> Calc<PeopleStep> {
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

// ---------------------------------------------------------------- faza 4: novac

pub(crate) struct MoneyStep {
    pub(crate) out: MoneyOutcome,
    pub(crate) cash: i128,
    pub(crate) wage: i128,
    pub(crate) hire_cost: i128,
    pub(crate) training_cost: i128,
    pub(crate) cpor_total: i128,
    pub(crate) maintenance_cost: i128,
    pub(crate) reply_cost: i128,
    pub(crate) other: i128,
    pub(crate) fixed: i128,
}

struct MoneyIn<'a> {
    spec: &'a CategorySpec,
    state: &'a HotelState,
    market: &'a MarketOutcome,
    plan: &'a PeriodPlan,
    staff: u16,
    hires: u16,
    departures: u16,
    period: &'a Period,
    sit_cost: i128,
}

fn money(i: &MoneyIn, cat: &sim_hotel::Catalog) -> Calc<MoneyStep> {
    let p = i.period;
    let spec = i.spec;
    let m = i.market;
    let rooms_revenue = c::add(
        c::mul(w(m.individual_sold), w(i.plan.price_cents))?,
        w(m.group_revenue_cents),
    )?;
    let other = c::div_round(
        c::mul(rooms_revenue, w(cat.economy.other_revenue_bp))?,
        10_000,
    )?;
    let wage_unit = c::div_floor(w(spec.wage_cents), 100)?;
    let monthly_wage = c::add(
        c::mul(c::mul(w(i.staff), wage_unit)?, w(i.plan.wage_pct))?,
        w(i.state.extra_wage_cents),
    )?;
    let wage = w(prorate(c::narrow(monthly_wage)?, p)?);
    let hire_cost = c::mul(
        c::add(w(i.hires), w(i.departures))?,
        w(cat.economy.hire_cents),
    )?;
    let training_cost = w(training_effect(i.plan.training, cat, p)?.cost_cents);
    let labor = c::sum(&[wage, hire_cost, training_cost])?;
    let cpor_total = c::mul(w(spec.cpor_cents), w(m.sold))?;
    let maintenance_cost = w(maintenance_effect(i.plan.maintenance, cat, p)?.cost_cents);
    let fixed = w(prorate(i.state.fixed_cents, p)?);
    let reply_cost = if i.plan.reply_reviews {
        w(prorate(cat.actions.reply_cents, p)?)
    } else {
        0
    };
    let costs = c::sum(&[
        labor,
        cpor_total,
        maintenance_cost,
        w(i.plan.marketing_cents),
        fixed,
        reply_cost,
        i.sit_cost,
    ])?;
    let gop = c::sub(c::add(rooms_revenue, other)?, costs)?;
    let capex = if i.plan.renovation {
        w(cat.actions.renovation_cents)
    } else {
        0
    };
    let mut cash = c::sub(c::add(w(i.state.cash_cents), gop)?, capex)?;
    let mut interest = 0;
    if cash < 0 {
        // Mjesečna kamata razmjerno danima kruga (nazivnik su dani mjeseca u kojem krug počinje).
        interest = c::mul_div(
            c::mul(neg(cash)?, w(cat.economy.interest_bp))?,
            w(p.days),
            c::mul(10_000, w(p.month_days_at_start()))?,
        )?;
        cash = c::sub(cash, interest)?;
    }
    let goppar = c::div_round(gop, c::mul(w(i.state.rooms), w(p.days))?)?;
    Ok(MoneyStep {
        out: MoneyOutcome {
            rooms_revenue_cents: c::narrow(rooms_revenue)?,
            other_revenue_cents: c::narrow(other)?,
            labor_cents: c::narrow(labor)?,
            costs_cents: c::narrow(costs)?,
            gop_cents: c::narrow(gop)?,
            capex_cents: c::narrow(capex)?,
            interest_cents: c::narrow(interest)?,
            goppar_cents: c::narrow(goppar)?,
        },
        cash,
        wage,
        hire_cost,
        training_cost,
        cpor_total,
        maintenance_cost,
        reply_cost,
        other,
        fixed,
    })
}

// ---------------------------------------------------------------- faza 5: oprema i gosti

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

struct GuestIn<'a> {
    state: &'a HotelState,
    plan: &'a PeriodPlan,
    pend: &'a Pending,
    period: &'a Period,
    satisfaction_step_bp: u16,
    rating_step_bp: u16,
    price_ref_cents: i64,
    load: i128,
    penalty: i128,
    morale: i128,
    experience: i128,
    sit_equipment: i128,
    sit_satisfaction: i128,
    sit_rating: i128,
}

fn guests(i: &GuestIn, cat: &sim_hotel::Catalog) -> Calc<GuestStep> {
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
    let satisfaction_gain = c::div_round(
        c::mul(w(i.satisfaction_step_bp), c::sub(sat_target, sat_now)?)?,
        10_000,
    )?;
    let satisfaction = c::sum(&[
        sat_now,
        satisfaction_gain,
        renovation_satisfaction,
        i.sit_satisfaction,
    ])?
    .clamp(0, 10_000);
    let rating_target = c::div_round(c::add(14_000, satisfaction)?, 25)?;
    let rating_now = w(i.state.rating);
    let rating_gain = c::div_round(
        c::mul(w(i.rating_step_bp), c::sub(rating_target, rating_now)?)?,
        10_000,
    )?;
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

// ---------------------------------------------------------------- slaganje

/// Grupa za mjesec `m` je obračunana kad krug završi taj mjesec ili kasniji (blokada se iz stanja briše tada).
fn group_done(m: sim_hotel::Month, p: &Period) -> bool {
    p.parts()
        .last()
        .is_some_and(|last| m < last.month || (m == last.month && p.ends_month()))
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
    for o in situations {
        let e = effects(o, p.start_month(), base, state.category)?;
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

    let st = staffing(state, plan)?;
    let mut s = st.state;
    s.groups.extend(new_groups);
    s.extra_wage_cents = c::narrow(c::add(w(s.extra_wage_cents), extra)?)?;
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
            sit_morale: morale,
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
            sit_cost: cost,
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
            sit_equipment: equip,
            sit_satisfaction: sat,
            sit_rating: rating,
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
            situations: &sits,
            sit_extra_wage: shown_extra,
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
