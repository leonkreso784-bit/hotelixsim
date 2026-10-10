//! Tržište (kopija v1, DEMO §3.1, P9–P15): potražnja, kapacitet osoblja i prodaja u jednom krugu.
//! Izlaz je v1 `MarketOutcome` (ulazi u v1 sažetak izvješća).

use engine::calc::{self as c, Calc};
use sim_hotel::{MarketOutcome, Month};

use crate::plan::PeriodPlan;
use crate::state::HotelState;

fn f_price_bp(price: i64, reference: i64, elasticity_bp: i32) -> Calc<i32> {
    let delta = c::sub(i128::from(price), i128::from(reference))?;
    let shift = c::div_round(
        c::mul(i128::from(elasticity_bp), delta)?,
        i128::from(reference),
    )?;
    c::narrow(c::add(10_000, shift)?.clamp(3_000, 16_000))
}

fn f_reputation_bp(rating: u16) -> Calc<i32> {
    let gap = c::sub(i128::from(rating), 840)?;
    let shift = c::div_round(c::mul(54, gap)?, 10)?;
    c::narrow(c::add(10_000, shift)?.max(0))
}

fn f_marketing_bp(marketing_cents: i64, max_bp: u16, half_cents: i64) -> Calc<i32> {
    let m = i128::from(marketing_cents);
    let lift = c::mul_div(i128::from(max_bp), m, c::add(m, i128::from(half_cents))?)?;
    c::narrow(c::add(10_000, lift)?)
}

pub fn market(
    state: &HotelState,
    month: Month,
    plan: &PeriodPlan,
    staff: u16,
    cat: &sim_hotel::Catalog,
) -> Calc<MarketOutcome> {
    let spec = cat
        .category(state.category)
        .map_err(|_| c::CalcError::BadDivisor)?;
    let days = i128::from(month.days());
    let rooms = i128::from(state.rooms);

    let out_raw = c::div_round(c::sub(4_000, i128::from(state.equipment_pts))?, 400)?;
    let rooms_out = out_raw.clamp(0, rooms);
    let available = c::mul(c::sub(rooms, rooms_out)?, days)?;

    let f_price = f_price_bp(
        plan.price_cents,
        spec.price_ref_cents.get(month),
        spec.elasticity_bp,
    )?;
    let f_rep = f_reputation_bp(state.rating)?;
    let f_mkt = f_marketing_bp(
        plan.marketing_cents,
        cat.economy.marketing_max_bp,
        cat.economy.marketing_half_cents,
    )?;

    let room_days = c::mul(rooms, days)?;
    let base_bp = i128::from(spec.base_occupancy_bp.get(month));
    let base_product = c::mul(room_days, base_bp)?;
    let base_demand = c::div_round(base_product, 10_000)?;
    let mut product = base_product;
    for f in [f_price, f_rep, f_mkt] {
        product = c::mul(product, i128::from(f))?;
    }
    let demand = c::div_round(product, 10_000_000_000_000_000)?.max(0);

    let staff_capacity = c::div_floor(c::mul(c::mul(i128::from(staff), days)?, 24)?, 10)?;

    let mut group_nights: i128 = 0;
    for g in state.groups.iter().filter(|g| g.month == month) {
        group_nights = c::add(group_nights, i128::from(g.room_nights))?;
    }

    let sold = c::add(group_nights, demand)?
        .min(available)
        .min(staff_capacity);
    let group_sold = group_nights.min(sold);
    let individual = c::sub(sold, group_sold)?;

    let mut remaining = group_sold;
    let mut revenue: i128 = 0;
    for g in state.groups.iter().filter(|g| g.month == month) {
        let take = i128::from(g.room_nights).min(remaining);
        revenue = c::add(revenue, c::mul(take, i128::from(g.price_cents))?)?;
        remaining = c::sub(remaining, take)?;
    }

    Ok(MarketOutcome {
        rooms_out: c::narrow(rooms_out)?,
        available: c::narrow(available)?,
        f_price_bp: f_price,
        f_reputation_bp: f_rep,
        f_marketing_bp: f_mkt,
        base_demand: c::narrow(base_demand)?,
        demand: c::narrow(demand)?,
        staff_capacity: c::narrow(staff_capacity)?,
        group_room_nights: c::narrow(group_nights)?,
        sold: c::narrow(sold)?,
        group_sold: c::narrow(group_sold)?,
        individual_sold: c::narrow(individual)?,
        group_revenue_cents: c::narrow(revenue)?,
    })
}
