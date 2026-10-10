//! Roboti v2 (kopija v1 `Strategy::plan`, DEMO §10): plan kruga za svaku strategiju. Izbor odgovora na
//! situacije ostaje v1 `Strategy::choose`. Sva slučajnost dolazi iz predanog `Rng`-a.

use engine::Rng;
use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{Maintenance, Month, Strategy, Training, euro_bounds};

use crate::catalog::Catalog;
use crate::plan::PeriodPlan;
use crate::resolve::ResolveError;
use crate::state::HotelState;

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

/// Cijena u centima: `ref · pct %`, zaokruženo na cijeli euro (`div_round(x, 100)·100`),
/// pa stisnuto u granice kataloga izražene u cijelim eurima.
fn price_for_pct(pct: i128, reference: i128, min_pct: i128, max_pct: i128) -> Calc<i64> {
    let x = c::div_round(c::mul(reference, pct)?, 100)?;
    let euros = c::div_round(x, 100)?;
    let (lo, hi) = euro_bounds(reference, min_pct, max_pct)?;
    c::narrow(c::mul(euros.max(lo).min(hi.max(lo)), 100)?)
}

/// Dopušteni raspon sezonaca za sljedeći plan (v1 `seasonal_range` nad v2 stanjem).
fn seasonal_range(state: &HotelState, cat: &Catalog) -> (u16, u16) {
    let a = &cat.base.actions;
    let lo = state.seasonal.saturating_sub(a.seasonal_step);
    let hi = state
        .seasonal
        .saturating_add(a.seasonal_step)
        .min(a.seasonal_max);
    (lo, hi)
}

pub fn robot_plan(
    strategy: Strategy,
    s: &HotelState,
    m: Month,
    cat: &Catalog,
    rng: &mut Rng,
) -> Result<PeriodPlan, ResolveError> {
    let spec = cat.base.category(s.category)?;
    let a = &cat.base.actions;
    let reference = i128::from(spec.price_ref_cents.get(m));
    let (min_pct, max_pct) = (i128::from(a.price_min_pct), i128::from(a.price_max_pct));
    let fixed_price = |pct: i128| price_for_pct(pct, reference, min_pct, max_pct);
    let wage = |w: u16| w.max(a.wage_min_pct).min(a.wage_max_pct);
    let market = |v: i64| v.max(0).min(a.marketing_max_cents);
    let (s_lo, s_hi) = seasonal_range(s, cat);

    let mut plan = match strategy {
        Strategy::Saver => PeriodPlan {
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
            PeriodPlan {
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
        Strategy::OccupancyHunter => PeriodPlan {
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
            PeriodPlan {
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
