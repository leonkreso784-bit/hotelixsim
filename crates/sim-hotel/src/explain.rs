//! Objašnjenje mjeseca (DEMO §11.6): svaka promjena varijable dobiva uzroke s iznosima.

use engine::calc::{self, Calc};

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
