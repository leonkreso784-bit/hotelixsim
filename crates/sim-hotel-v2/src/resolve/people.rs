//! Faze osoblja (1) i ljudi (3): sezonci po odjelima na cilj, opterećenje hotela (ponderirano osobljem odjela),
//! moral, odlasci i iskustvo.

use std::collections::BTreeMap;

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{Pending, PeopleOutcome, load_penalty, turnover_bp};

use super::{neg, step_toward, w, weighted};
use crate::calendar::{Period, prorate};
use crate::department::{DeptCode, split_season_departures};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

// ---------------------------------------------------------------- faza 1: osoblje

pub(super) struct Staffing {
    pub(super) state: HotelState,
    pub(super) hires: u16,
    pub(super) released: u16,
    /// Svi zaposleni hotela (odlasci, iskustvo, sažetak).
    pub(super) staff: u16,
    /// Zaposleni Smještaja (kapacitet tržišta i opterećenje).
    pub(super) rooms_staff: u16,
}

/// Sezonci svakog odjela se dovode na cilj iz plana (najam i puštanje po odjelu), otvorenost odjela se prepisuje iz
/// plana, plaća se postavlja; `pending` se troši ovaj krug (zato ga ovdje čistimo).
pub(super) fn staffing(state: &HotelState, plan: &PeriodPlan) -> Calc<Staffing> {
    let mut s = state.clone();
    let (mut hires, mut released) = (0_u16, 0_u16);
    for (code, d) in s.departments.iter_mut() {
        // `validate` jamči da plan ima točno odjele stanja.
        let Some(pl) = plan.departments.get(code) else {
            continue;
        };
        hires = hires
            .checked_add(pl.seasonal_target.saturating_sub(d.seasonal))
            .ok_or(CalcError::Overflow)?;
        released = released
            .checked_add(d.seasonal.saturating_sub(pl.seasonal_target))
            .ok_or(CalcError::Overflow)?;
        d.seasonal = pl.seasonal_target;
        d.open = pl.open;
    }
    s.wage_pct = plan.wage_pct;
    let mut staff: u16 = 0;
    for d in s.departments.values() {
        staff = staff
            .checked_add(d.permanent)
            .and_then(|x| x.checked_add(d.seasonal))
            .ok_or(CalcError::Overflow)?;
    }
    let rooms_staff = s
        .departments
        .get(&DeptCode::rooms())
        .map_or(Some(0), |d| d.permanent.checked_add(d.seasonal))
        .ok_or(CalcError::Overflow)?;
    s.pending = Pending::default();
    Ok(Staffing {
        state: s,
        hires,
        released,
        staff,
        rooms_staff,
    })
}

// ---------------------------------------------------------------- faza 3: ljudi

/// Hotelski odlasci kruga po odjelima prema ljudima odjela nakon faze osoblja (stalni + sezonci; zbroj je `staff`
/// hotela), razmjerno kroz cijelu sezonu (`split_season_departures` s dosadašnjim odlascima odjela). Broj, trošak
/// zamjene (Uprava) i iskustvo ostaju hotelski; ovo samo kaže iz kojeg su odjela otišli.
pub(super) fn department_departures(
    state: &HotelState,
    departures: u16,
) -> Calc<BTreeMap<DeptCode, u16>> {
    let mut staff = BTreeMap::new();
    let mut so_far = BTreeMap::new();
    for (code, d) in &state.departments {
        let n = d
            .permanent
            .checked_add(d.seasonal)
            .ok_or(CalcError::Overflow)?;
        staff.insert(code.clone(), n);
        so_far.insert(code.clone(), d.season_departures);
    }
    split_season_departures(departures, &staff, &so_far)
}

pub(crate) struct PeopleStep {
    pub(crate) out: PeopleOutcome,
    pub(crate) morale: i128,
    pub(crate) experience: i128,
    pub(crate) penalty: i128,
    pub(crate) release_morale: i128,
    /// Rast iskustva u ovom krugu (mjesečnih +300 razmjerno).
    pub(crate) growth: i128,
}

/// Opterećenje Smještaja (stotinke): pri punom kapacitetu 240 (kao `dept_step`); uz 2,4 noći po zaposlenom na dan to
/// je točno v1 `sold·100/(staff·days)`. Bez osoblja ili kapaciteta 300 (v1).
pub(super) fn rooms_load(sold: u32, staff: u16, capacity_tenths: u16, p: &Period) -> Calc<i128> {
    let denom = c::mul(c::mul(w(staff), w(p.days))?, w(capacity_tenths))?;
    if denom == 0 {
        return Ok(300);
    }
    Ok(c::div_round(c::mul(w(sold), 2_400)?, denom)?.clamp(0, 300))
}

/// Opterećenje jednog odjela s ljudima koji služe goste: otvorenost, zaposleni, kapacitet (desetinke po zaposlenom na
/// dan) i opterećenje u stotinkama.
pub(super) struct LoadUnit {
    pub(super) open: bool,
    pub(super) staff: u16,
    pub(super) capacity_tenths: u16,
    pub(super) load: i128,
}

/// Opterećenje i kazna hotela: prosjek opterećenja i kazni otvorenih odjela s kapacitetom i osobljem
/// (`capacity_tenths > 0`, `staff > 0`), ponderiran brojem zaposlenih; zatvoreni odjel (stalni bez gostiju,
/// opterećenje 0) ne razrjeđuje prosjek. Bez takvog odjela vrijedi opterećenje Smještaja `fallback` (v1: 300 bez
/// osoblja). U degeneriranom katalogu jedini takav odjel je Smještaj, pa je to točno v1.
pub(super) fn hotel_load(units: &[LoadUnit], fallback: i128) -> Calc<(i128, i128)> {
    let mut loads: Vec<(u32, i128)> = Vec::with_capacity(units.len());
    let mut penalties: Vec<(u32, i128)> = Vec::with_capacity(units.len());
    for u in units
        .iter()
        .filter(|u| u.open && u.capacity_tenths > 0 && u.staff > 0)
    {
        loads.push((u32::from(u.staff), u.load));
        penalties.push((u32::from(u.staff), w(load_penalty(c::narrow(u.load)?)?)));
    }
    let fallback_penalty = w(load_penalty(c::narrow(fallback)?)?);
    Ok((
        weighted(&loads, fallback)?,
        weighted(&penalties, fallback_penalty)?,
    ))
}

pub(super) struct PeopleIn<'a> {
    pub(super) state: &'a HotelState,
    pub(super) staff: u16,
    /// Opterećenje i kazna hotela (`hotel_load`).
    pub(super) load: i128,
    pub(super) penalty: i128,
    pub(super) hires: u16,
    pub(super) released: u16,
    pub(super) period: &'a Period,
    pub(super) morale_step_bp: u16,
    pub(super) wage_pct: u16,
    pub(super) pend: &'a Pending,
    pub(super) sit_morale: i128,
}

pub(super) fn people(i: &PeopleIn, cat: &sim_hotel::Catalog) -> Calc<PeopleStep> {
    let staff = w(i.staff);
    let (load, penalty) = (i.load, i.penalty);
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
    let morale_gain = step_toward(morale_now, morale_target, i.morale_step_bp, i.period)?;
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
        release_morale,
        growth,
    })
}
