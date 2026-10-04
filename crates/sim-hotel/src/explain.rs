//! Objašnjenje mjeseca (DEMO §11.6): svaka promjena varijable dobiva uzroke s iznosima.

use engine::calc::{self, self as c, Calc};

use crate::market::MarketOutcome;
use crate::plan::MonthPlan;
use crate::resolve::{GuestStep, MoneyStep, PeopleStep, neg, w};
use crate::state::{HotelState, Pending};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Var {
    Sold,
    Revenue,
    Costs,
    Cash,
    Staff,
    Morale,
    Experience,
    Equipment,
    Satisfaction,
    Rating,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cause {
    Baseline,
    Price,
    Reputation,
    Marketing,
    Groups,
    Capacity,
    OtherRevenue,
    Wage,
    Load,
    Training,
    Hires,
    Release,
    Turnover,
    Maintenance,
    Renovation,
    ReplyReviews,
    FixedCosts,
    Interest,
    Situation(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    RoomNights,
    Cents,
    People,
    Points,
    RatingHundredths,
    Bp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub var: Var,
    pub cause: Cause,
    pub amount: i64,
    pub unit: Unit,
}

/// Sakupljač redaka: nula se ne piše, osim kad je uzrok `Baseline`.
#[derive(Debug, Default)]
pub(crate) struct Lines(pub Vec<Line>);

impl Lines {
    pub(crate) fn push(&mut self, var: Var, cause: Cause, amount: i128, unit: Unit) -> Calc<()> {
        if amount != 0 || cause == Cause::Baseline {
            self.0.push(Line {
                var,
                cause,
                amount: calc::narrow(amount)?,
                unit,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- zapis objašnjenja

pub(crate) struct Explain<'a> {
    pub(crate) before: &'a HotelState,
    pub(crate) pend: &'a Pending,
    pub(crate) market: &'a MarketOutcome,
    pub(crate) plan: &'a MonthPlan,
    pub(crate) people: &'a PeopleStep,
    pub(crate) money: &'a MoneyStep,
    pub(crate) guests: &'a GuestStep,
}

pub(crate) fn explain(l: &mut Lines, e: &Explain) -> Calc<()> {
    let (m, p, mo, g, po) = (e.market, e.people, e.money, e.guests, &e.people.out);
    let before = e.before;
    // Prodane sobe-noći
    l.push(
        Var::Sold,
        Cause::Baseline,
        w(m.base_demand),
        Unit::RoomNights,
    )?;
    for (cause, f) in [
        (Cause::Price, m.f_price_bp),
        (Cause::Reputation, m.f_reputation_bp),
        (Cause::Marketing, m.f_marketing_bp),
    ] {
        l.push(Var::Sold, cause, c::sub(w(f), 10_000)?, Unit::Bp)?;
    }
    l.push(Var::Sold, Cause::Groups, w(m.group_sold), Unit::RoomNights)?;
    let wanted = c::add(w(m.group_room_nights), w(m.demand))?;
    let shortfall = c::sub(w(m.sold), wanted)?;
    l.push(
        Var::Sold,
        Cause::Capacity,
        shortfall.min(0),
        Unit::RoomNights,
    )?;
    // Prihod
    let cents = Unit::Cents;
    l.push(
        Var::Revenue,
        Cause::Baseline,
        c::mul(w(m.individual_sold), w(e.plan.price_cents))?,
        cents,
    )?;
    l.push(Var::Revenue, Cause::Groups, w(m.group_revenue_cents), cents)?;
    l.push(Var::Revenue, Cause::OtherRevenue, mo.other, cents)?;
    // Troškovi
    l.push(Var::Costs, Cause::Wage, mo.wage, cents)?;
    l.push(Var::Costs, Cause::Hires, mo.hire_cost, cents)?;
    l.push(Var::Costs, Cause::Training, mo.training_cost, cents)?;
    l.push(Var::Costs, Cause::Baseline, mo.cpor_total, cents)?;
    l.push(Var::Costs, Cause::Maintenance, mo.maintenance_cost, cents)?;
    l.push(
        Var::Costs,
        Cause::Marketing,
        w(e.plan.marketing_cents),
        cents,
    )?;
    l.push(Var::Costs, Cause::FixedCosts, w(before.fixed_cents), cents)?;
    l.push(Var::Costs, Cause::ReplyReviews, mo.reply_cost, cents)?;
    // Novac
    l.push(Var::Cash, Cause::Baseline, w(mo.out.gop_cents), cents)?;
    l.push(
        Var::Cash,
        Cause::Renovation,
        neg(w(mo.out.capex_cents))?,
        cents,
    )?;
    l.push(
        Var::Cash,
        Cause::Interest,
        neg(w(mo.out.interest_cents))?,
        cents,
    )?;
    // Osoblje
    l.push(Var::Staff, Cause::Hires, w(po.hires), Unit::People)?;
    l.push(
        Var::Staff,
        Cause::Release,
        neg(w(po.released))?,
        Unit::People,
    )?;
    l.push(Var::Staff, Cause::Turnover, w(po.departures), Unit::People)?;
    // Moral: promjena, a uzroci su doprinosi cilju
    let pts = Unit::Points;
    l.push(
        Var::Morale,
        Cause::Baseline,
        c::sub(p.morale, w(before.morale_pts))?,
        pts,
    )?;
    l.push(
        Var::Morale,
        Cause::Wage,
        c::mul(200, c::sub(w(e.plan.wage_pct), 100)?)?,
        pts,
    )?;
    l.push(
        Var::Morale,
        Cause::Load,
        neg(c::div_round(p.penalty, 2)?)?,
        pts,
    )?;
    l.push(Var::Morale, Cause::Training, w(e.pend.morale_pts), pts)?;
    l.push(Var::Morale, Cause::Release, p.release_morale, pts)?;
    // Iskustvo
    l.push(Var::Experience, Cause::Baseline, 300, pts)?;
    l.push(
        Var::Experience,
        Cause::Turnover,
        c::sub(w(po.experience_mix_pts), w(before.experience_pts))?,
        pts,
    )?;
    l.push(
        Var::Experience,
        Cause::Training,
        w(e.pend.experience_pts),
        pts,
    )?;
    // Oprema
    l.push(Var::Equipment, Cause::Baseline, -400, pts)?;
    l.push(Var::Equipment, Cause::Load, g.equipment_load, pts)?;
    l.push(Var::Equipment, Cause::Maintenance, g.maintenance_eff, pts)?;
    l.push(
        Var::Equipment,
        Cause::Renovation,
        g.renovation_equipment,
        pts,
    )?;
    // Zadovoljstvo i ocjena
    l.push(Var::Satisfaction, Cause::Baseline, g.satisfaction_gain, pts)?;
    l.push(
        Var::Satisfaction,
        Cause::Renovation,
        g.renovation_satisfaction,
        pts,
    )?;
    l.push(
        Var::Rating,
        Cause::Baseline,
        g.rating_gain,
        Unit::RatingHundredths,
    )?;
    l.push(
        Var::Rating,
        Cause::ReplyReviews,
        g.reply_rating,
        Unit::RatingHundredths,
    )?;
    Ok(())
}
