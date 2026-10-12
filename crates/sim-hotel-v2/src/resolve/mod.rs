//! Obračun kruga u fazama (DEMO §3, ADR-013 t. 4): osoblje, tržište, ljudi, novac, oprema i gosti. Svaka faza je
//! privatna funkcija s vlastitim ulazom i izlazom; `resolve_period` ih samo slaže, piše objašnjenje (`lines`) i uz
//! v2 stanje daje v1 sažetak (`PeriodReport::summary`). Krug je tjedan ili mjesec: mjesečni iznosi (plaće, fiksno,
//! održavanje, obuka, trošenje opreme, rast iskustva, odgovaranje) idu razmjerno (`prorate`), jednokratni (obnova,
//! situacije, najam zaposlenika) u punom iznosu; u mjesečnom krugu svako pravilo daje točno v1 vrijednost.
//!
//! Faze žive u podmodulima: `situations` (učinci situacija, grupe), `people` (osoblje i ljudi), `departments`
//! (računi odjela po USALI-ju, neraspoređeni odjeli, fiksne naknade), `money` (novac), `guests` (oprema i gosti);
//! tržište je `crate::market`. Ovdje su zajednički tipovi i redoslijed faza. v1 sažetak se slaže iz računa odjela.

mod departments;
mod guests;
mod money;
mod people;
mod situations;

pub(crate) use guests::GuestStep;
pub use money::FinanceOutcome;
pub(crate) use money::MoneyStep;
pub(crate) use people::PeopleStep;

use departments::{
    OperatedIn, RoomsIn, fixed_charges, operated, rooms_outcome, rooms_payroll, undistributed,
};
use guests::{GuestIn, guests};
use money::{MoneyIn, decisions, finance, money, rooms_maintenance_cost};
use people::{LoadUnit, PeopleIn, department_departures, hotel_load, people, rooms_load, staffing};
use situations::{group_done, situation_effects};

use std::collections::BTreeMap;

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::Training;
use sim_hotel::{
    CatalogError, MechanicError, MonthReport, Pending, SituationOutcome, TrainingEffect,
};

use crate::calendar::{Period, avg_i64, prorate};
use crate::catalog::Catalog;
use crate::department::{BuildingOutcome, DeptCode, PeriodAccounts, after_renewal};
use crate::explain::{Explain, Lines, explain};
use crate::market::market;
use crate::plan::{PeriodPlan, PlanError};
use crate::renewal::{RoundRenewal, depreciation, renew_round};
use crate::state::HotelState;

/// Izvješće kruga: v2 stanje prije i poslije, v1 sažetak (isti oblik kao v1 `MonthReport`, složen iz odjela) i
/// računi odjela kruga (USALI), uz EBITDA, amortizaciju i EBIT kruga.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodReport {
    pub period: Period,
    pub before: HotelState,
    pub after: HotelState,
    pub summary: MonthReport,
    pub accounts: PeriodAccounts,
    pub finance: FinanceOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    Calc(CalcError),
    Plan(PlanError),
    Catalog(CatalogError),
    Mechanic(MechanicError),
    /// Izbor odgovora ne postoji (prazan popis ili indeks izvan popisa).
    BadChoice,
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
/// v1 funkcija koju v2 ponovno koristi (`Strategy::choose`) vraća v1 grešku. (Početni trošak rada v2 računa sam,
/// `indices::start_labor_cents`, umjesto v1 `base_labor_cents`.)
impl From<sim_hotel::ResolveError> for ResolveError {
    fn from(e: sim_hotel::ResolveError) -> Self {
        match e {
            sim_hotel::ResolveError::Calc(e) => ResolveError::Calc(e),
            sim_hotel::ResolveError::Plan(e) => ResolveError::Plan(e.into()),
            sim_hotel::ResolveError::Catalog(e) => ResolveError::Catalog(e),
            sim_hotel::ResolveError::Mechanic(e) => ResolveError::Mechanic(e),
            sim_hotel::ResolveError::BadChoice => ResolveError::BadChoice,
        }
    }
}

pub(crate) fn w(x: impl Into<i128>) -> i128 {
    x.into()
}

pub(crate) fn neg(x: i128) -> Calc<i128> {
    c::sub(0, x)
}

/// Ponderirani prosjek `div_round(Σ težina·vrijednost, Σ težina)`; bez ikakve težine `fallback`.
pub(super) fn weighted(items: &[(u32, i128)], fallback: i128) -> Calc<i128> {
    let (mut num, mut den): (i128, i128) = (0, 0);
    for (weight, v) in items {
        num = c::add(num, c::mul(w(*weight), *v)?)?;
        den = c::add(den, w(*weight))?;
    }
    if den == 0 {
        return Ok(fallback);
    }
    c::div_round(num, den)
}

/// Pomak stanja prema cilju u krugu: `round(Δ·step_bp/10 000)` (moral, zadovoljstvo, ocjena). Tjedni koeficijenti su
/// manji od mjesečnih, pa bi se mali razmak zaokružio na 0 i stanje bi zauvijek stalo ispod cilja; zato je u krugu
/// koji nije cijeli kalendarski mjesec najmanji pomak 1 jedinica prema cilju (nikad preko cilja; 🟨 H1 korak 4).
/// Mjesečni krug je nepromijenjen (v1).
pub(super) fn step_toward(now: i128, target: i128, step_bp: u16, p: &Period) -> Calc<i128> {
    let gap = c::sub(target, now)?;
    let step = c::div_round(c::mul(w(step_bp), gap)?, 10_000)?;
    if step == 0 && gap != 0 && !p.is_whole_month() {
        return Ok(gap.signum());
    }
    Ok(step)
}

/// Obuka za krug: trošak i oba odgođena učinka razmjerno krugu (mjesečni iznosi iz kataloga).
fn training_effect(t: Training, cat: &sim_hotel::Catalog, p: &Period) -> Calc<TrainingEffect> {
    let monthly = match t {
        Training::None => TrainingEffect {
            cost_cents: 0,
            experience_pts: 0,
            morale_pts: 0,
        },
        Training::Basic => cat.actions.training_basic.clone(),
        Training::Full => cat.actions.training_full.clone(),
    };
    Ok(TrainingEffect {
        cost_cents: prorate(monthly.cost_cents, p)?,
        experience_pts: c::narrow(w(prorate(i64::from(monthly.experience_pts), p)?))?,
        morale_pts: c::narrow(w(prorate(i64::from(monthly.morale_pts), p)?))?,
    })
}

/// Obnova v2 kruga (obnova-v2-spec §3.3, §3.5) nad knjigama u stanju `s`: svaka knjiga se zarola u mjesec kruga,
/// plaća se rata planiranog iznosa; zgrada se troši i dobiva bodove obnove prije odjela (treba je mješavina).
fn renewals(
    s: &mut HotelState,
    p: &Period,
    plan: &PeriodPlan,
    cat: &Catalog,
) -> Result<(BTreeMap<DeptCode, RoundRenewal>, BuildingOutcome), ResolveError> {
    let mut out = BTreeMap::new();
    for (code, d) in s.departments.iter_mut() {
        let amount = plan.departments.get(code).map_or(0, |pl| pl.renewal_cents);
        let r = renew_round(&mut d.book, &cat.dept(code)?.renewal, amount, p)?;
        out.insert(code.clone(), r);
    }
    let b = &cat.economy.building;
    let r = renew_round(
        &mut s.building.book,
        &b.renewal,
        plan.building_renewal_cents,
        p,
    )?;
    let before = s.building.condition_pts;
    let wear: i32 = c::narrow(w(prorate(
        c::narrow(c::sub(0, w(b.wear_pts_per_month))?)?,
        p,
    )?))?;
    let base = c::add(w(before), w(wear))?.clamp(0, 10_000);
    let after = after_renewal(base, r.delta_pts)?;
    Ok((
        out,
        BuildingOutcome {
            before_pts: before,
            wear_pts: wear,
            renewal_pts: c::narrow(c::sub(after, base)?)?,
            after_pts: c::narrow(after)?,
            renewal_cents: r.cents,
        },
    ))
}

/// Obračun kruga. Redoslijed: obnova v2 (knjige, plaćanja, zgrada) → osoblje → tržište i Smještaj (prodane noći,
/// noćenja gostiju) → ljudi (opterećenje hotela ponderirano osobljem odjela) → računi operativnih odjela (sa
/// zgradom u cilju zadovoljstva) → oprema i gosti (kvaliteta i zadovoljstvo hotela ponderirani gostima) → račun
/// Smještaja → neraspoređeni odjeli s troškovima odluka → fiksne naknade → v1 sažetak novca iz računa (capex) →
/// amortizacija i EBIT → objašnjenje.
pub fn resolve_period(
    state: &HotelState,
    p: &Period,
    plan: &PeriodPlan,
    situations: &[SituationOutcome],
    cat: &Catalog,
) -> Result<PeriodReport, ResolveError> {
    plan.validate(state, p, cat)?;
    let base = &cat.base;
    let coeffs = cat.coeffs(p.kind)?;
    let pend = state.pending.clone();
    let spec = base.category(state.category)?;

    let sit = situation_effects(situations, p, base, state.category)?;

    // 1. osoblje, tržište i Smještaj
    let rooms_capacity = cat.dept(&DeptCode::rooms())?.capacity_tenths_per_staff_day;
    let rooms_before = state.rooms_state()?.clone();
    let st = staffing(state, plan)?;
    let mut s = st.state;
    // 0. obnova v2: knjige u `s` (pa i u stanju nakon kruga), plaćanja kruga, zgrada
    let (round_renewals, building) = renewals(&mut s, p, plan, cat)?;
    let share_bp = cat.economy.building.satisfaction_share_bp;
    let rooms_spec = cat.dept(&DeptCode::rooms())?;
    let rooms_renewal = round_renewals
        .get(&DeptCode::rooms())
        .copied()
        .unwrap_or_default();
    s.groups.extend(sit.groups);
    s.extra_wage_cents = c::narrow(c::add(w(s.extra_wage_cents), sit.extra_wage)?)?;
    let mkt = market(&s, p, plan, st.rooms_staff, rooms_capacity, base)?;
    s.groups.retain(|g| !group_done(g.month, p));
    let rooms_revenue: i64 = c::narrow(c::add(
        c::mul(w(mkt.individual_sold), w(plan.price_cents))?,
        w(mkt.group_revenue_cents),
    )?)?;
    let guest_nights: u32 = c::narrow(c::div_round(
        c::mul(w(mkt.sold), w(cat.economy.guests_per_room_hundredths))?,
        100,
    )?)?;
    let ops_in = |state: &HotelState, experience_pts: i32, morale_pts: i32| {
        operated(&OperatedIn {
            state,
            period: p,
            plan,
            cat,
            guest_nights,
            rooms_revenue_cents: rooms_revenue,
            experience_pts,
            morale_pts,
            satisfaction_step_bp: coeffs.satisfaction_step_bp,
            renewals: &round_renewals,
            building_pts: building.after_pts,
            building_share_bp: share_bp,
        })
    };

    // 2. ljudi: opterećenje odjela ne ovisi o iskustvu i moralu, pa se uzima iz računa s početnim vrijednostima
    let r_load = rooms_load(mkt.sold, st.rooms_staff, rooms_capacity, p)?;
    let mut units = vec![LoadUnit {
        open: s.rooms_state()?.open,
        staff: st.rooms_staff,
        capacity_tenths: rooms_capacity,
        load: r_load,
    }];
    for o in ops_in(&s, s.experience_pts, s.morale_pts)?.values() {
        units.push(LoadUnit {
            open: o.open,
            staff: o.staff,
            capacity_tenths: cat.dept(&o.code)?.capacity_tenths_per_staff_day,
            load: w(o.load_hundredths),
        });
    }
    let (load, penalty) = hotel_load(&units, r_load)?;
    let pe = people(
        &PeopleIn {
            state: &s,
            staff: st.staff,
            load,
            penalty,
            hires: st.hires,
            released: st.released,
            period: p,
            morale_step_bp: coeffs.morale_step_bp,
            wage_pct: plan.wage_pct,
            pend: &pend,
            sit_morale: sit.morale,
        },
        base,
    )?;

    // 3. operativni odjeli s iskustvom i moralom nakon faze ljudi
    let mut depts = ops_in(&s, c::narrow(pe.experience)?, c::narrow(pe.morale)?)?;

    // 4. oprema i gosti, račun Smještaja; osjećaj sobe uključuje odjele s udjelom (parking, stanje nakon kruga)
    let mut felt_parts: Vec<(i32, u16)> = Vec::new();
    for o in depts.values() {
        let share = cat.dept(&o.code)?.guest_share_bp;
        if share > 0 {
            felt_parts.push((o.condition_after_pts, share));
        }
    }
    let r_penalty = w(sim_hotel::load_penalty(c::narrow(r_load)?)?);
    let others: Vec<(u32, i32, i32)> = depts
        .values()
        .map(|o| (o.guest_weight, o.quality_pts, o.satisfaction_target_pts))
        .collect();
    let gu = guests(
        &GuestIn {
            state: &s,
            plan,
            equipment_pts: rooms_before.condition_pts,
            pend: &pend,
            period: p,
            satisfaction_step_bp: coeffs.satisfaction_step_bp,
            rating_step_bp: coeffs.rating_step_bp,
            price_ref_cents: avg_i64(&spec.price_ref_cents, p)?,
            rooms_load: r_load,
            rooms_penalty: r_penalty,
            rooms_satisfaction_pts: rooms_before.satisfaction_pts,
            guest_nights,
            others: &others,
            morale: pe.morale,
            experience: pe.experience,
            sit_equipment: sit.equipment,
            sit_satisfaction: sit.satisfaction,
            sit_rating: sit.rating,
            rooms_spec,
            renewal_delta_pts: rooms_renewal.delta_pts,
            building_pts: building.after_pts,
            building_share_bp: share_bp,
            felt_parts: &felt_parts,
        },
        base,
    )?;
    let rooms_plan = plan
        .departments
        .get(&DeptCode::rooms())
        .ok_or_else(|| CatalogError::Missing("plan.departments.rooms".to_string()))?;
    let rooms = rooms_outcome(&RoomsIn {
        market: &mkt,
        guests: &gu,
        open: rooms_plan.open,
        staff: st.rooms_staff,
        price_level_pct: rooms_plan.price_level_pct,
        revenue_cents: rooms_revenue,
        payroll_cents: rooms_payroll(
            st.rooms_staff,
            rooms_spec.wage_cents,
            plan.wage_pct,
            s.extra_wage_cents,
            p,
        )?,
        cpor_cents: spec.cpor_cents,
        load: r_load,
        penalty: r_penalty,
        condition_before_pts: rooms_before.condition_pts,
        satisfaction_before_pts: rooms_before.satisfaction_pts,
        maintenance_cents: rooms_maintenance_cost(plan, p, rooms_spec)?,
        guest_nights,
        renewal_cents: rooms_renewal.cents,
    })?;
    depts.insert(DeptCode::rooms(), rooms);

    // 5. neraspoređeni odjeli s troškovima odluka, 6. fiksne naknade
    let mut maintenance: i128 = 0;
    for o in depts.values() {
        maintenance = c::add(maintenance, w(o.maintenance_cents))?;
    }
    let dec = decisions(
        plan,
        (st.hires, pe.out.departures),
        c::narrow(maintenance)?,
        sit.cost,
        p,
        base,
    )?;
    let mut accounts = PeriodAccounts {
        days: p.days,
        rooms: s.rooms,
        undistributed: undistributed(&s, p, cat, mkt.sold, plan.wage_pct, &dec)?,
        departments: depts,
        fixed_charges: fixed_charges(&s, p)?,
        interest_cents: 0,
        capex_cents: 0,
        building,
        depreciation_equipment_cents: 0,
        depreciation_building_cents: 0,
    };
    let dept_departures = department_departures(&s, pe.out.departures)?;
    for (code, n) in &dept_departures {
        if let Some(o) = accounts.departments.get_mut(code) {
            o.departures = *n;
        } else if let Some(u) = accounts.undistributed.get_mut(code) {
            u.departures = *n;
        }
    }

    // 7. v1 sažetak novca iz računa; ulaganja = stara obnova + obnova v2 odjela i zgrade
    let mut renewal_cents = w(accounts.building.renewal_cents);
    for r in round_renewals.values() {
        renewal_cents = c::add(renewal_cents, w(r.cents))?;
    }
    let mo = money(
        &MoneyIn {
            state: &s,
            plan,
            accounts: &accounts,
            decisions: &dec,
            period: p,
            renewal_cents,
        },
        base,
    )?;
    accounts.interest_cents = mo.out.interest_cents;
    accounts.capex_cents = mo.out.capex_cents;

    // 8. amortizacija (od mjeseca nakon obnove) i EBIT
    let mut equipment_dep: i128 = 0;
    for (code, d) in &s.departments {
        let life = cat.dept(code)?.renewal.life_months;
        equipment_dep = c::add(equipment_dep, w(depreciation(&d.book, life, p)?))?;
    }
    accounts.depreciation_equipment_cents = c::narrow(equipment_dep)?;
    accounts.depreciation_building_cents = depreciation(
        &s.building.book,
        cat.economy.building.renewal.life_months,
        p,
    )?;
    let fin = finance(mo.out.gop_cents, &accounts)?;

    let mut after = s;
    after.building.condition_pts = accounts.building.after_pts;
    after.morale_pts = c::narrow(pe.morale)?;
    after.experience_pts = c::narrow(pe.experience)?;
    for (code, o) in &accounts.departments {
        let d = after
            .departments
            .get_mut(code)
            .ok_or_else(|| CatalogError::Missing(format!("state.departments.{}", code.as_str())))?;
        d.condition_pts = o.condition_after_pts;
        d.satisfaction_pts = o.satisfaction_after_pts;
    }
    for (code, n) in &dept_departures {
        if let Some(d) = after.departments.get_mut(code) {
            d.season_departures = d
                .season_departures
                .checked_add(*n)
                .ok_or(CalcError::Overflow)?;
        }
    }
    after.satisfaction_pts = c::narrow(gu.satisfaction)?;
    after.rating = c::narrow(gu.rating)?;
    after.cash_cents = c::narrow(mo.cash)?;
    after.load_hundredths = pe.out.load_hundredths;
    after.turnover_bp = pe.out.turnover_bp;
    after.renovated = after.renovated || plan.renovation;
    let training = training_effect(plan.training, base, p)?;
    after.pending = Pending {
        experience_pts: training.experience_pts,
        morale_pts: training.morale_pts,
        renovation: plan.renovation,
    };

    // 9. objašnjenje: „oprema" sažetka je 70/30 (Smještaj i ostali operativni); promjenu Smještaja objašnjavaju v1
    // uzroci i obnova v2 Smještaja, ostatak ide u redak održavanja, osim dijela koji donosi obnova ostalih odjela
    // (`renewal_others`: sažetak nakon kruga prema sažetku s ostalim odjelima umanjenim za njihove bodove obnove)
    let (summary_before, summary_after) = (state.summary(), after.summary());
    let equipment_others = c::sub(
        c::sub(
            w(summary_after.equipment_pts),
            w(summary_before.equipment_pts),
        )?,
        c::sub(gu.equipment, w(rooms_before.condition_pts))?,
    )?;
    let mut without = after.clone();
    for (code, o) in &accounts.departments {
        if *code != DeptCode::rooms()
            && let Some(d) = without.departments.get_mut(code)
        {
            d.condition_pts = c::narrow(c::sub(w(d.condition_pts), w(o.renewal_pts))?)?;
        }
    }
    let renewal_others = c::sub(
        w(summary_after.equipment_pts),
        w(without.summary().equipment_pts),
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
            situations: &sit.shown,
            sit_extra_wage: sit.shown_extra_wage,
            equipment_others,
            renewal_others,
        },
    )?;

    let summary = MonthReport {
        month: p.start_month(),
        before: summary_before,
        after: summary_after,
        market: mkt,
        people: pe.out,
        money: mo.out,
        guests: gu.out,
        lines: lines.0,
    };
    Ok(PeriodReport {
        period: *p,
        before: state.clone(),
        after,
        summary,
        accounts,
        finance: fin,
    })
}
