//! Roboti (DEMO §10): četiri strategije koje igraju sezonu umjesto studenta; služe za balans.
//! Sva slučajnost dolazi iz predanog `Rng`-a, pa je svaka sezona ponovljiva.

use engine::Rng;
use engine::calc::{self as c, Calc, CalcError};

use crate::catalog::{Catalog, Category};
use crate::mechanic::{Mechanic, OfferedAnswer};
use crate::month::Month;
use crate::plan::{Maintenance, MonthPlan, Training};
use crate::resolve::ResolveError;
use crate::state::HotelState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strategy {
    Saver,
    Caring,
    OccupancyHunter,
    Random,
}

fn overflow() -> ResolveError {
    ResolveError::Calc(CalcError::Overflow)
}

fn to_u64(x: i128) -> Result<u64, ResolveError> {
    u64::try_from(x).map_err(|_| overflow())
}

/// Jednoliko iz `[lo, hi]` (oba uključena); `hi < lo` daje `lo`.
fn uniform(rng: &mut Rng, lo: u64, hi: u64) -> Result<u64, ResolveError> {
    let top = hi.max(lo).checked_add(1).ok_or_else(overflow)?;
    Ok(rng.range(lo, top))
}

/// Najmanji i najveći cijeli euro koji prolaze provjeru granica plana (koristi `game` za granice plana).
pub fn euro_bounds(reference: i128, min_pct: i128, max_pct: i128) -> Calc<(i128, i128)> {
    let lo = c::div_floor(c::add(c::mul(min_pct, reference)?, 9_999)?, 10_000)?;
    let hi = c::div_floor(c::mul(max_pct, reference)?, 10_000)?;
    Ok((lo, hi))
}

/// Cijena u centima: `ref · pct %`, zaokruženo na cijeli euro (`div_round(x, 100)·100`),
/// pa stisnuto u granice kataloga izražene u cijelim eurima.
fn price_for_pct(pct: i128, reference: i128, min_pct: i128, max_pct: i128) -> Calc<i64> {
    let x = c::div_round(c::mul(reference, pct)?, 100)?;
    let euros = c::div_round(x, 100)?;
    let (lo, hi) = euro_bounds(reference, min_pct, max_pct)?;
    c::narrow(c::mul(euros.max(lo).min(hi.max(lo)), 100)?)
}

/// Dopušteni raspon sezonaca za sljedeći plan (koristi `game` za granice plana).
pub fn seasonal_range(state: &HotelState, cat: &Catalog) -> (u16, u16) {
    let a = &cat.actions;
    let lo = state.seasonal.saturating_sub(a.seasonal_step);
    let hi = state
        .seasonal
        .saturating_add(a.seasonal_step)
        .min(a.seasonal_max);
    (lo, hi)
}

fn direct_cost(a: &OfferedAnswer, cat: &Catalog, category: Category) -> Calc<i128> {
    let mut total: i128 = 0;
    for m in &a.mechanics {
        total = c::add(total, i128::from(m.direct_cost_cents(cat, category)?))?;
    }
    Ok(total)
}

/// Najveći popust među prihvaćenim grupama odgovora; `None` ako odgovor ne prihvaća grupu.
fn accepted_discount(a: &OfferedAnswer) -> Option<u16> {
    a.mechanics
        .iter()
        .filter_map(|m| match m {
            Mechanic::GroupBooking {
                accept: true,
                discount_bp,
                ..
            } => Some(*discount_bp),
            _ => None,
        })
        .max()
}

impl Strategy {
    pub const ALL: [Strategy; 4] = [
        Strategy::Saver,
        Strategy::Caring,
        Strategy::OccupancyHunter,
        Strategy::Random,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Strategy::Saver => "stedisa",
            Strategy::Caring => "brizni",
            Strategy::OccupancyHunter => "lovac",
            Strategy::Random => "nasumicni",
        }
    }

    pub fn plan(
        self,
        s: &HotelState,
        m: Month,
        cat: &Catalog,
        rng: &mut Rng,
    ) -> Result<MonthPlan, ResolveError> {
        let spec = cat.category(s.category)?;
        let a = &cat.actions;
        let reference = i128::from(spec.price_ref_cents.get(m));
        let (min_pct, max_pct) = (i128::from(a.price_min_pct), i128::from(a.price_max_pct));
        let fixed_price = |pct: i128| price_for_pct(pct, reference, min_pct, max_pct);
        let wage = |w: u16| w.max(a.wage_min_pct).min(a.wage_max_pct);
        let market = |v: i64| v.max(0).min(a.marketing_max_cents);
        let (s_lo, s_hi) = seasonal_range(s, cat);

        let mut plan = match self {
            Strategy::Saver => MonthPlan {
                price_cents: fixed_price(100)?,
                marketing_cents: 0,
                seasonal_target: spec.seasonal_plan.get(m).saturating_sub(5),
                wage_pct: wage(95),
                training: Training::None,
                maintenance: Maintenance::Deferred,
                renovation: false,
                reply_reviews: false,
            },
            Strategy::Caring => {
                let rooms_bp = c::mul(
                    i128::from(s.rooms),
                    i128::from(spec.base_occupancy_bp.get(m)),
                )?;
                let need = c::div_floor(c::add(rooms_bp, 17_999)?, 18_000)?;
                let target = c::sub(need, i128::from(s.permanent))?.max(0);
                MonthPlan {
                    price_cents: fixed_price(105)?,
                    marketing_cents: market(500_000),
                    seasonal_target: u16::try_from(target).unwrap_or(u16::MAX),
                    wage_pct: wage(103),
                    training: Training::Basic,
                    maintenance: Maintenance::Regular,
                    renovation: false,
                    reply_reviews: false,
                }
            }
            Strategy::OccupancyHunter => MonthPlan {
                price_cents: fixed_price(85)?,
                marketing_cents: market(2_500_000),
                seasonal_target: s.seasonal,
                wage_pct: wage(100),
                training: Training::None,
                maintenance: Maintenance::Regular,
                renovation: false,
                reply_reviews: false,
            },
            Strategy::Random => {
                let (lo, hi) = euro_bounds(reference, min_pct, max_pct)?;
                let euros = uniform(rng, to_u64(lo)?, to_u64(hi)?)?;
                let steps = a.marketing_max_cents.checked_div(100_000).unwrap_or(0);
                let mkt_steps = uniform(rng, 0, to_u64(i128::from(steps))?)?;
                let seasonal = uniform(rng, u64::from(s_lo), u64::from(s_hi))?;
                let wage_pct = uniform(rng, u64::from(a.wage_min_pct), u64::from(a.wage_max_pct))?;
                let training = match rng.range(0, 3) {
                    0 => Training::None,
                    1 => Training::Basic,
                    _ => Training::Full,
                };
                let maintenance = match rng.range(0, 3) {
                    0 => Maintenance::Deferred,
                    1 => Maintenance::Regular,
                    _ => Maintenance::Intensive,
                };
                MonthPlan {
                    price_cents: c::narrow(c::mul(i128::from(euros), 100)?)?,
                    marketing_cents: c::narrow(c::mul(i128::from(mkt_steps), 100_000)?)?,
                    seasonal_target: u16::try_from(seasonal).unwrap_or(s_lo),
                    wage_pct: u16::try_from(wage_pct).unwrap_or(a.wage_min_pct),
                    training,
                    maintenance,
                    renovation: false,
                    reply_reviews: rng.chance_bp(5_000),
                }
            }
        };
        plan.seasonal_target = plan.seasonal_target.max(s_lo).min(s_hi);
        Ok(plan)
    }

    /// Indeks odabranog odgovora u `answers`.
    pub fn choose(
        self,
        answers: &[OfferedAnswer],
        cat: &Catalog,
        category: Category,
        rng: &mut Rng,
    ) -> Result<usize, ResolveError> {
        if answers.is_empty() {
            return Err(ResolveError::BadChoice);
        }
        match self {
            Strategy::Saver => {
                let mut best: Option<(usize, i128, u8)> = None;
                for (i, a) in answers.iter().enumerate() {
                    let cost = direct_cost(a, cat, category)?;
                    if best.is_none_or(|(_, bc, bq)| (cost, a.quality) < (bc, bq)) {
                        best = Some((i, cost, a.quality));
                    }
                }
                best.map(|(i, _, _)| i).ok_or(ResolveError::BadChoice)
            }
            Strategy::Caring => {
                let mut best: Option<(usize, u8)> = None;
                for (i, a) in answers.iter().enumerate() {
                    if best.is_none_or(|(_, bq)| a.quality > bq) {
                        best = Some((i, a.quality));
                    }
                }
                best.map(|(i, _)| i).ok_or(ResolveError::BadChoice)
            }
            Strategy::OccupancyHunter => {
                let mut best: Option<(usize, u16)> = None;
                for (i, a) in answers.iter().enumerate() {
                    if let Some(d) = accepted_discount(a)
                        && best.is_none_or(|(_, bd)| d > bd)
                    {
                        best = Some((i, d));
                    }
                }
                if let Some((i, _)) = best {
                    return Ok(i);
                }
                let mut order: Vec<usize> = (0..answers.len()).collect();
                order.sort_by_key(|&i| answers.get(i).map_or(0, |a| a.quality));
                let mid = order.len().checked_div(2).unwrap_or(0);
                order.get(mid).copied().ok_or(ResolveError::BadChoice)
            }
            Strategy::Random => {
                let n = u64::try_from(answers.len()).map_err(|_| overflow())?;
                usize::try_from(rng.range(0, n)).map_err(|_| overflow())
            }
        }
    }
}
