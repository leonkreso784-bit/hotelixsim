//! Faza novca (4): prihodi, troškovi, obnova, kamata i GOPPAR kruga.

use engine::calc::{self as c, Calc};
use sim_hotel::{CategorySpec, MarketOutcome, MoneyOutcome};

use super::{maintenance_effect, neg, training_effect, w};
use crate::calendar::{Period, prorate};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

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

pub(super) struct MoneyIn<'a> {
    pub(super) spec: &'a CategorySpec,
    pub(super) state: &'a HotelState,
    pub(super) market: &'a MarketOutcome,
    pub(super) plan: &'a PeriodPlan,
    pub(super) staff: u16,
    pub(super) hires: u16,
    pub(super) departures: u16,
    pub(super) period: &'a Period,
    pub(super) sit_cost: i128,
}

pub(super) fn money(i: &MoneyIn, cat: &sim_hotel::Catalog) -> Calc<MoneyStep> {
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
