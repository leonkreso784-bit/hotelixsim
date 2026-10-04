//! Obračun mjeseca u fazama (DEMO §3, ADR-013 t. 4): osoblje, tržište, ljudi, novac, oprema i gosti.
//! Svaka faza je privatna funkcija s vlastitim ulazom i izlazom; `resolve_month` ih samo slaže i
//! piše objašnjenje (`lines`).

use engine::calc::{self as c, Calc, CalcError};

use crate::catalog::{Catalog, CatalogError, CategorySpec, LevelEffect, TrainingEffect};
use crate::explain::{Explain, Line, Lines, explain};
use crate::market::{MarketOutcome, market};
use crate::mechanic::{Effects, MechanicError, SituationOutcome, effects};
use crate::month::Month;
use crate::plan::{Maintenance, MonthPlan, PlanError, Training};
use crate::state::{HotelState, Pending, turnover_bp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleOutcome {
    pub hires: u16,
    pub released: u16,
    pub staff: u16,
    pub load_hundredths: u16,
    pub load_penalty_pts: i32,
    pub morale_target_pts: i32,
    pub turnover_bp: u16,
    pub departures: u16,
    pub experience_mix_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoneyOutcome {
    pub rooms_revenue_cents: i64,
    pub other_revenue_cents: i64,
    pub labor_cents: i64,
    pub costs_cents: i64,
    pub gop_cents: i64,
    pub capex_cents: i64,
    pub interest_cents: i64,
    pub goppar_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestOutcome {
    pub quality_pts: i32,
    pub value_pts: i32,
    pub satisfaction_target_pts: i32,
    pub rating_target: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthReport {
    pub month: Month,
    pub before: HotelState,
    pub after: HotelState,
    pub market: MarketOutcome,
    pub people: PeopleOutcome,
    pub money: MoneyOutcome,
    pub guests: GuestOutcome,
    pub lines: Vec<Line>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    Calc(CalcError),
    Plan(PlanError),
    Catalog(CatalogError),
    Mechanic(MechanicError),
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

pub(crate) fn w(x: impl Into<i128>) -> i128 {
    x.into()
}

pub(crate) fn neg(x: i128) -> Calc<i128> {
    c::sub(0, x)
}

fn training_effect(t: Training, cat: &Catalog) -> TrainingEffect {
    match t {
        Training::None => TrainingEffect {
            cost_cents: 0,
            experience_pts: 0,
            morale_pts: 0,
        },
        Training::Basic => cat.actions.training_basic.clone(),
        Training::Full => cat.actions.training_full.clone(),
    }
}

fn maintenance_effect(m: Maintenance, cat: &Catalog) -> LevelEffect {
    match m {
        Maintenance::Deferred => cat.actions.maintenance_deferred.clone(),
        Maintenance::Regular => cat.actions.maintenance_regular.clone(),
        Maintenance::Intensive => cat.actions.maintenance_intensive.clone(),
    }
}

// ---------------------------------------------------------------- faza 1: osoblje

struct Staffing {
    state: HotelState,
    hires: u16,
    released: u16,
    staff: u16,
}

/// Sezonci se dovode na cilj, plaća se postavlja; `pending` se troši ovaj mjesec (zato ga ovdje čistimo).
fn staffing(state: &HotelState, plan: &MonthPlan) -> Calc<Staffing> {
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
}

struct PeopleIn<'a> {
    state: &'a HotelState,
    staff: u16,
    hires: u16,
    released: u16,
    sold: u32,
    days: u8,
    wage_pct: u16,
    pend: &'a Pending,
    sit_morale: i128,
}

fn people(i: &PeopleIn, cat: &Catalog) -> Calc<PeopleStep> {
    let staff = w(i.staff);
    let load = if staff == 0 {
        300
    } else {
        c::div_round(c::mul(w(i.sold), 100)?, c::mul(staff, w(i.days))?)?.clamp(0, 300)
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
    let morale_gain = c::div_round(c::mul(3_000, c::sub(morale_target, morale_now)?)?, 10_000)?;
    let morale = c::add(morale_now, morale_gain)?.clamp(0, 10_000);
    let turnover = turnover_bp(c::narrow(morale)?)?;
    let departures = c::div_round(c::mul(staff, w(turnover))?, 10_000)?.clamp(0, staff);
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
    let experience = c::sum(&[exp_mix, 300, w(i.pend.experience_pts)])?.clamp(0, 10_000);
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
}

struct MoneyIn<'a> {
    spec: &'a CategorySpec,
    state: &'a HotelState,
    market: &'a MarketOutcome,
    plan: &'a MonthPlan,
    staff: u16,
    hires: u16,
    departures: u16,
    days: u8,
    sit_cost: i128,
}

fn money(i: &MoneyIn, cat: &Catalog) -> Calc<MoneyStep> {
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
    let wage = c::add(
        c::mul(c::mul(w(i.staff), wage_unit)?, w(i.plan.wage_pct))?,
        w(i.state.extra_wage_cents),
    )?;
    let hire_cost = c::mul(
        c::add(w(i.hires), w(i.departures))?,
        w(cat.economy.hire_cents),
    )?;
    let training_cost = w(training_effect(i.plan.training, cat).cost_cents);
    let labor = c::sum(&[wage, hire_cost, training_cost])?;
    let cpor_total = c::mul(w(spec.cpor_cents), w(m.sold))?;
    let maintenance_cost = w(maintenance_effect(i.plan.maintenance, cat).cost_cents);
    let reply_cost = if i.plan.reply_reviews {
        w(cat.actions.reply_cents)
    } else {
        0
    };
    let costs = c::sum(&[
        labor,
        cpor_total,
        maintenance_cost,
        w(i.plan.marketing_cents),
        w(i.state.fixed_cents),
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
        interest = c::mul_div(neg(cash)?, w(cat.economy.interest_bp), 10_000)?;
        cash = c::sub(cash, interest)?;
    }
    let goppar = c::div_round(gop, c::mul(w(i.state.rooms), w(i.days))?)?;
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
}

struct GuestIn<'a> {
    state: &'a HotelState,
    plan: &'a MonthPlan,
    pend: &'a Pending,
    price_ref_cents: i64,
    load: i128,
    penalty: i128,
    morale: i128,
    experience: i128,
    sit_equipment: i128,
    sit_satisfaction: i128,
    sit_rating: i128,
}

fn guests(i: &GuestIn, cat: &Catalog) -> Calc<GuestStep> {
    let maintenance_eff = w(maintenance_effect(i.plan.maintenance, cat).equipment_pts);
    let equipment_load = if i.load > 180 { -100 } else { 0 };
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
        -400,
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
    let satisfaction_gain = c::div_round(c::mul(5_000, c::sub(sat_target, sat_now)?)?, 10_000)?;
    let satisfaction = c::sum(&[
        sat_now,
        satisfaction_gain,
        renovation_satisfaction,
        i.sit_satisfaction,
    ])?
    .clamp(0, 10_000);
    let rating_target = c::div_round(c::add(14_000, satisfaction)?, 25)?;
    let rating_now = w(i.state.rating);
    let rating_gain = c::div_round(c::mul(2_500, c::sub(rating_target, rating_now)?)?, 10_000)?;
    let reply_rating = if i.plan.reply_reviews {
        w(cat.actions.reply_rating)
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
    })
}

// ---------------------------------------------------------------- slaganje

pub fn resolve_month(
    state: &HotelState,
    month: Month,
    plan: &MonthPlan,
    situations: &[SituationOutcome],
    cat: &Catalog,
) -> Result<MonthReport, ResolveError> {
    plan.validate(state, month, cat)?;
    let days = month.days();
    let pend = state.pending.clone();
    let spec = cat.category(state.category)?;

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
    for o in situations {
        let e = effects(o, month, cat, state.category)?;
        sat = c::add(sat, w(e.satisfaction_pts))?;
        morale = c::add(morale, w(e.morale_pts))?;
        rating = c::add(rating, w(e.rating))?;
        equip = c::add(equip, w(e.equipment_pts))?;
        cost = c::add(cost, w(e.cost_cents))?;
        extra = c::add(extra, w(e.extra_wage_cents))?;
        new_groups.extend(e.groups.iter().cloned());
        sits.push((o.code.clone(), e));
    }

    let st = staffing(state, plan)?;
    let mut s = st.state;
    s.groups.extend(new_groups);
    s.extra_wage_cents = c::narrow(c::add(w(s.extra_wage_cents), extra)?)?;
    let mkt = market(&s, month, plan, st.staff, cat)?;
    s.groups.retain(|g| g.month != month);

    let pe = people(
        &PeopleIn {
            state: &s,
            staff: st.staff,
            hires: st.hires,
            released: st.released,
            sold: mkt.sold,
            days,
            wage_pct: plan.wage_pct,
            pend: &pend,
            sit_morale: morale,
        },
        cat,
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
            days,
            sit_cost: cost,
        },
        cat,
    )?;
    let gu = guests(
        &GuestIn {
            state: &s,
            plan,
            pend: &pend,
            price_ref_cents: spec.price_ref_cents.get(month),
            load: pe.load,
            penalty: pe.penalty,
            morale: pe.morale,
            experience: pe.experience,
            sit_equipment: equip,
            sit_satisfaction: sat,
            sit_rating: rating,
        },
        cat,
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
            sit_extra_wage: extra,
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
    let training = training_effect(plan.training, cat);
    after.pending = Pending {
        experience_pts: training.experience_pts,
        morale_pts: training.morale_pts,
        renovation: plan.renovation,
    };

    Ok(MonthReport {
        month,
        before: state.clone(),
        after,
        market: mkt,
        people: pe.out,
        money: mo.out,
        guests: gu.out,
        lines: lines.0,
    })
}
