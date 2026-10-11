//! Roboti v2 (v1 `Strategy::plan` za krug, DEMO §10): plan kruga za svaku strategiju. v1 strategija daje ukupne
//! sezonce i razinu održavanja; sezonci se raspodijele po odjelima prema planovima odjela (`allocate_seasonal`),
//! održavanje je isto u svim odjelima, razina cijene 100 (Nasumični: jednoliko u granicama, u koracima od 5, samo
//! operativni odjeli). Izbor odgovora na situacije ostaje v1 `Strategy::choose`. Sva slučajnost dolazi iz predanog
//! `Rng`-a; razine cijene Nasumičnog izvlače se nakon svih v1 izvlačenja, pa v1 dio plana ostaje isti.

use engine::Rng;
use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{Maintenance, Strategy, Training, euro_bounds};

use crate::calendar::{Period, avg_i64, avg_u16, prorate};
use crate::catalog::Catalog;
use crate::department::UsaliClass;
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

/// Stalni i sezonci cijelog hotela (zbroj odjela).
fn staff_totals(state: &HotelState) -> (u16, u16) {
    state
        .departments
        .values()
        .fold((0_u16, 0_u16), |(p, s), d| {
            (p.saturating_add(d.permanent), s.saturating_add(d.seasonal))
        })
}

/// Dopušteni raspon ukupnog broja sezonaca za sljedeći plan (korak je koeficijent vrste kruga).
fn seasonal_range(seasonal: u16, step: u16, seasonal_max: u16) -> (u16, u16) {
    let lo = seasonal.saturating_sub(step);
    let hi = seasonal.saturating_add(step).min(seasonal_max);
    (lo, hi)
}

/// v1 oblik plana robota (ukupni sezonci, jedna razina održavanja); odjeli se dodaju na kraju.
struct Draft {
    price_cents: i64,
    marketing_cents: i64,
    seasonal_target: u16,
    wage_pct: u16,
    training: Training,
    maintenance: Maintenance,
    renovation: bool,
    reply_reviews: bool,
}

/// Razina cijene iz `[min, max]` u koracima od 5 (jednoliko).
fn random_level(rng: &mut Rng, min: u16, max: u16) -> Result<u16, ResolveError> {
    let steps = max.saturating_sub(min).checked_div(5).unwrap_or(0);
    let k = uniform(rng, 0, u64::from(steps))?;
    let k = u16::try_from(k).map_err(|_| overflow())?;
    k.checked_mul(5)
        .and_then(|x| x.checked_add(min))
        .ok_or_else(overflow)
}

pub fn robot_plan(
    strategy: Strategy,
    s: &HotelState,
    p: &Period,
    cat: &Catalog,
    rng: &mut Rng,
) -> Result<PeriodPlan, ResolveError> {
    let spec = cat.base.category(s.category)?;
    let a = &cat.base.actions;
    let reference = i128::from(avg_i64(&spec.price_ref_cents, p)?);
    let (min_pct, max_pct) = (i128::from(a.price_min_pct), i128::from(a.price_max_pct));
    let fixed_price = |pct: i128| price_for_pct(pct, reference, min_pct, max_pct);
    let wage = |w: u16| w.max(a.wage_min_pct).min(a.wage_max_pct);
    // Marketing iz kataloga i strategija su mjesečni iznosi; plan je za krug, pa idu razmjerno.
    let marketing_max = prorate(a.marketing_max_cents, p)?;
    let market = |v: i64| v.max(0).min(marketing_max);
    let (permanent, seasonal) = staff_totals(s);
    let (s_lo, s_hi) = seasonal_range(seasonal, cat.coeffs(p.kind)?.seasonal_step, a.seasonal_max);

    let mut plan = match strategy {
        Strategy::Saver => Draft {
            price_cents: fixed_price(100)?,
            marketing_cents: 0,
            seasonal_target: avg_u16(&spec.seasonal_plan, p)?.saturating_sub(5),
            wage_pct: wage(95),
            training: Training::None,
            maintenance: Maintenance::Deferred,
            renovation: false,
            reply_reviews: false,
        },
        Strategy::Caring => {
            let rooms_bp = c::mul(
                i128::from(s.rooms),
                i128::from(avg_u16(&spec.base_occupancy_bp, p)?),
            )?;
            let need = c::div_floor(c::add(rooms_bp, 17_999)?, 18_000)?;
            let target = c::sub(need, i128::from(permanent))?.max(0);
            Draft {
                price_cents: fixed_price(105)?,
                marketing_cents: market(prorate(500_000, p)?),
                seasonal_target: u16::try_from(target).unwrap_or(u16::MAX),
                wage_pct: wage(103),
                training: Training::Basic,
                maintenance: Maintenance::Regular,
                renovation: false,
                reply_reviews: false,
            }
        }
        Strategy::OccupancyHunter => Draft {
            price_cents: fixed_price(85)?,
            marketing_cents: market(prorate(2_500_000, p)?),
            seasonal_target: seasonal,
            wage_pct: wage(100),
            training: Training::None,
            maintenance: Maintenance::Regular,
            renovation: false,
            reply_reviews: false,
        },
        Strategy::Random => {
            let (lo, hi) = euro_bounds(reference, min_pct, max_pct)?;
            let euros = uniform(rng, to_u64(lo)?, to_u64(hi)?)?;
            let steps = marketing_max.checked_div(100_000).unwrap_or(0);
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
            Draft {
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
    let mut departments =
        PeriodPlan::departments_for(s, p, cat, plan.seasonal_target, plan.maintenance)?;
    if matches!(strategy, Strategy::Random) {
        let e = &cat.economy;
        for (code, d) in departments.iter_mut() {
            let operated = s.departments.get(code).is_some_and(|st| {
                matches!(
                    st.class,
                    UsaliClass::FoodBeverage | UsaliClass::OtherOperated
                )
            });
            if operated {
                d.price_level_pct =
                    random_level(rng, e.price_level_min_pct, e.price_level_max_pct)?;
            }
        }
    }
    Ok(PeriodPlan {
        price_cents: plan.price_cents,
        marketing_cents: plan.marketing_cents,
        wage_pct: plan.wage_pct,
        training: plan.training,
        renovation: plan.renovation,
        reply_reviews: plan.reply_reviews,
        departments,
    })
}
