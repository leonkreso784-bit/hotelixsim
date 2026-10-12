//! Računi odjela (USALI, MODEL_V2 §3.2): operativni odjeli i odjeli ostalih prihoda kroz `dept_step`, Smještaj iz
//! tržišta i faze gostiju, neraspoređeni odjeli s troškovima odluka (`undistributed_step`) i fiksne naknade ispod
//! GOP-a. Formule jednog odjela su u `crate::department`; ovdje se samo slažu ulazi.

use std::collections::BTreeMap;

use engine::calc::{self as c, Calc};
use sim_hotel::{CatalogError, MarketOutcome};

use super::{GuestStep, ResolveError, w};
use crate::calendar::{Period, prorate};
use crate::catalog::Catalog;
use crate::department::{
    AssignedCosts, DeptCode, DeptInput, DeptOutcome, UndistributedKind, UndistributedOutcome,
    UsaliClass, dept_step, undistributed_step,
};
use crate::plan::PeriodPlan;
use crate::renewal::RoundRenewal;
use crate::state::HotelState;

pub(super) struct OperatedIn<'a> {
    /// Obnova v2 po odjelu u krugu (plaćeno, bodovi); odjel bez zapisa nema obnovu.
    pub(super) renewals: &'a BTreeMap<DeptCode, RoundRenewal>,
    /// Stanje zgrade nakon kruga i njen udio u stanju koje gost osjeća.
    pub(super) building_pts: i32,
    pub(super) building_share_bp: u16,
    /// Stanje nakon faze osoblja (sezonci na cilju, otvorenost iz plana).
    pub(super) state: &'a HotelState,
    pub(super) period: &'a Period,
    pub(super) plan: &'a PeriodPlan,
    pub(super) cat: &'a Catalog,
    pub(super) guest_nights: u32,
    pub(super) rooms_revenue_cents: i64,
    pub(super) experience_pts: i32,
    pub(super) morale_pts: i32,
    pub(super) satisfaction_step_bp: u16,
}

/// `dept_step` za svaki odjel osim Smještaja i neraspoređenih, redom koda. Opterećenje odjela ne ovisi o iskustvu i
/// moralu, pa ga faza ljudi može uzeti iz poziva s početnim vrijednostima; račun odjela (kvaliteta, zadovoljstvo)
/// koristi iskustvo i moral nakon faze ljudi, kao Smještaj (v1).
pub(super) fn operated(i: &OperatedIn) -> Result<BTreeMap<DeptCode, DeptOutcome>, ResolveError> {
    let mut out = BTreeMap::new();
    for (code, st) in &i.state.departments {
        let spec = i.cat.dept(code)?;
        if *code == DeptCode::rooms() || spec.class.is_undistributed() {
            continue;
        }
        // `validate` jamči da plan ima točno odjele stanja.
        let pl =
            i.plan.departments.get(code).ok_or_else(|| {
                CatalogError::Missing(format!("plan.departments.{}", code.as_str()))
            })?;
        let o = dept_step(&DeptInput {
            code: code.clone(),
            period: i.period,
            spec,
            state: st,
            plan: pl,
            guest_nights: i.guest_nights,
            rooms_revenue_cents: i.rooms_revenue_cents,
            experience_pts: i.experience_pts,
            morale_pts: i.morale_pts,
            wage_pct: i.plan.wage_pct,
            satisfaction_step_bp: i.satisfaction_step_bp,
            renewal: i.renewals.get(code).copied().unwrap_or_default(),
            building_pts: i.building_pts,
            building_share_bp: i.building_share_bp,
        })?;
        out.insert(code.clone(), o);
    }
    Ok(out)
}

/// Plaće Smještaja: zaposleni · ⌊plaća odjela Smještaj / 100⌋ · postotak plaće (degenerirani katalog: plaća v1
/// kategorije), plus trajne povišice iz situacija
/// (🟨 P12: u sloju 1 sve idu Smještaju), razmjerno krugu.
pub(super) fn rooms_payroll(
    staff: u16,
    wage_cents: i64,
    wage_pct: u16,
    extra_wage_cents: i64,
    p: &Period,
) -> Calc<i64> {
    let monthly = c::add(
        c::mul(
            c::mul(w(staff), c::div_floor(w(wage_cents), 100)?)?,
            w(wage_pct),
        )?,
        w(extra_wage_cents),
    )?;
    prorate(c::narrow(monthly)?, p)
}

pub(super) struct RoomsIn<'a> {
    pub(super) market: &'a MarketOutcome,
    pub(super) guests: &'a GuestStep,
    pub(super) open: bool,
    pub(super) staff: u16,
    pub(super) price_level_pct: u16,
    pub(super) revenue_cents: i64,
    pub(super) payroll_cents: i64,
    /// Trošak po prodanoj sobi (rublje, potrošni materijal) kategorije hotela.
    pub(super) cpor_cents: i64,
    pub(super) load: i128,
    pub(super) penalty: i128,
    pub(super) condition_before_pts: i32,
    pub(super) satisfaction_before_pts: i32,
    pub(super) maintenance_cents: i64,
    pub(super) guest_nights: u32,
    /// Obnova v2 Smještaja plaćena u krugu.
    pub(super) renewal_cents: i64,
}

/// Račun Smještaja iz tržišta i faze gostiju: kapacitet je manji od kapaciteta osoblja i raspoloživih soba-noći,
/// korištenje je popunjenost raspoloživih soba, ostali izravni trošak je trošak po prodanoj sobi. Stanje prostora
/// nakon kruga uključuje i obnovu i situacije (v1 „oprema").
pub(super) fn rooms_outcome(i: &RoomsIn) -> Calc<DeptOutcome> {
    let m = i.market;
    let g = i.guests;
    let other_direct = c::mul(w(i.cpor_cents), w(m.sold))?;
    let profit = c::sub(
        c::sub(w(i.revenue_cents), w(i.payroll_cents))?,
        other_direct,
    )?;
    let usage = if m.available == 0 {
        0
    } else {
        c::div_round(c::mul(w(m.sold), 10_000)?, w(m.available))?.clamp(0, 10_000)
    };
    Ok(DeptOutcome {
        code: DeptCode::rooms(),
        class: UsaliClass::Rooms,
        open: i.open,
        staff: i.staff,
        capacity_uses: m.staff_capacity.min(m.available),
        demand_uses: c::narrow(c::add(w(m.group_room_nights), w(m.demand))?)?,
        served_uses: m.sold,
        usage_bp: c::narrow(usage)?,
        load_hundredths: c::narrow(i.load)?,
        load_penalty_pts: c::narrow(i.penalty)?,
        quality_pts: c::narrow(g.rooms_quality)?,
        price_level_pct: i.price_level_pct,
        revenue_cents: i.revenue_cents,
        payroll_cents: i.payroll_cents,
        cost_of_sales_cents: 0,
        other_direct_cents: c::narrow(other_direct)?,
        profit_cents: c::narrow(profit)?,
        condition_before_pts: i.condition_before_pts,
        wear_pts: c::narrow(g.wear)?,
        load_wear_pts: c::narrow(g.equipment_load)?,
        maintenance_pts: c::narrow(g.maintenance_eff)?,
        condition_after_pts: c::narrow(g.equipment)?,
        maintenance_cents: i.maintenance_cents,
        value_pts: g.out.value_pts,
        satisfaction_before_pts: i.satisfaction_before_pts,
        satisfaction_target_pts: c::narrow(g.rooms_target)?,
        satisfaction_after_pts: c::narrow(g.rooms_satisfaction)?,
        guest_weight: i.guest_nights,
        departures: 0,
        renewal_cents: i.renewal_cents,
        renewal_pts: c::narrow(g.renewal_pts)?,
        felt_condition_pts: g.felt,
    })
}

/// Troškovi odluka i situacija kruga koji se knjiže u neraspoređene odjele.
pub(super) struct Decisions {
    pub(super) marketing: i64,
    pub(super) reply: i64,
    pub(super) maintenance: i64,
    pub(super) training: i64,
    pub(super) hiring: i64,
    pub(super) situation: i64,
}

/// Komu pripada koja odluka: Prodaja i marketing ← marketing i odgovaranje na recenzije; Održavanje ← održavanje
/// svih odjela; Uprava ← obuka, zapošljavanje (najam i odlasci) i troškovi situacija; Energija ← ništa.
fn assigned(kind: UndistributedKind, d: &Decisions) -> AssignedCosts {
    match kind {
        UndistributedKind::SalesMarketing => AssignedCosts {
            marketing_cents: d.marketing,
            reply_cents: d.reply,
            ..AssignedCosts::default()
        },
        UndistributedKind::Maintenance => AssignedCosts {
            maintenance_cents: d.maintenance,
            ..AssignedCosts::default()
        },
        UndistributedKind::AdminGeneral => AssignedCosts {
            training_cents: d.training,
            hiring_cents: d.hiring,
            situation_cents: d.situation,
            ..AssignedCosts::default()
        },
        UndistributedKind::Utilities => AssignedCosts::default(),
    }
}

/// Neraspoređeni odjeli (Uprava, Prodaja i marketing, Održavanje, Energija), redom koda: plaće, stalni dio iz
/// stanja, trošak po prodanoj sobi i troškovi odluka (`assigned`). Katalog ima točno jedan odjel svake vrste
/// (`Catalog::validate`), pa se svaki trošak odluke knjiži točno jednom.
pub(super) fn undistributed(
    state: &HotelState,
    p: &Period,
    cat: &Catalog,
    sold: u32,
    wage_pct: u16,
    d: &Decisions,
) -> Result<BTreeMap<DeptCode, UndistributedOutcome>, ResolveError> {
    let mut out = BTreeMap::new();
    for (code, st) in &state.departments {
        let spec = cat.dept(code)?;
        let UsaliClass::Undistributed(kind) = spec.class else {
            continue;
        };
        let fixed = state
            .undistributed_fixed_cents
            .get(code)
            .copied()
            .unwrap_or(0);
        let o = undistributed_step(code, spec, st, fixed, p, sold, wage_pct, assigned(kind, d))?;
        out.insert(code.clone(), o);
    }
    Ok(out)
}

/// Fiksne naknade (osiguranje, najam, porez) razmjerno krugu, po kodu.
pub(super) fn fixed_charges(state: &HotelState, p: &Period) -> Calc<BTreeMap<String, i64>> {
    let mut out = BTreeMap::new();
    for (name, monthly) in &state.fixed_charges_cents {
        out.insert(name.clone(), prorate(*monthly, p)?);
    }
    Ok(out)
}
