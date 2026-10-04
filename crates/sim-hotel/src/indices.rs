//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2): sirove vrijednosti u stotinkama,
//! referentna tablica po mjesecu i centriranje na 50 = „kao referentni hotel".

use engine::calc::{self as c, Calc, CalcError};

use crate::catalog::{Catalog, Category};
use crate::index::IndexSet;
use crate::month::{Month, MonthTable};
use crate::plan::MonthPlan;
use crate::resolve::{MonthReport, ResolveError, resolve_month, w};
use crate::state::HotelState;

/// Sirovi indeksi u stotinkama; R smije biti negativan (ocjena ispod 70).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawIndices {
    pub g: i32,
    pub r: i32,
    pub z: i32,
    pub o: i32,
    pub f: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefPoint {
    pub raw: RawIndices,
    pub goppar_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTable {
    pub start: RefPoint,
    pub months: MonthTable<RefPoint>,
}

/// Plaće bez najma i obuke: osoblje · (plaća/100) · postotak + dodatak.
pub fn base_labor_cents(s: &HotelState, cat: &Catalog) -> Result<i64, ResolveError> {
    let spec = cat.category(s.category)?;
    let unit = c::div_floor(w(spec.wage_cents), 100)?;
    let wage = c::mul(c::mul(w(s.staff()), unit)?, w(s.wage_pct))?;
    Ok(c::narrow(c::add(wage, w(s.extra_wage_cents))?)?)
}

pub fn raw_indices(
    s: &HotelState,
    goppar_cents: i64,
    labor_cents: i64,
    ref_goppar_cents: i64,
    cat: &Catalog,
) -> Calc<RawIndices> {
    let retention =
        c::sub(10_000, c::div_round(c::mul(25, w(s.turnover_bp))?, 2)?)?.clamp(0, 10_000);
    let penalty = c::mul(100, c::sub(w(s.load_hundredths), 180)?)?.clamp(0, 6_000);
    let floor = w(cat.economy.goppar_floor_cents);
    let diff = c::sub(w(goppar_cents), w(ref_goppar_cents))?;
    let gi = c::add(
        5_000,
        c::div_round(c::mul(5_000, diff)?, w(ref_goppar_cents).max(floor))?,
    )?
    .clamp(0, 10_000);
    let liq = if s.cash_cents <= 0 {
        0
    } else {
        let denom = c::add(w(s.fixed_cents), w(labor_cents))?;
        // Bez gornje granice (ADR-014): zasićenje na 100 skrivalo je potrošen novac (npr. obnovu).
        // Granica 0–100 ostaje samo na konačnom centriranom indeksu.
        c::div_round(c::mul(5_000, w(s.cash_cents))?, denom)?
    };
    let g = w(s.satisfaction_pts);
    let r = c::div_round(c::mul(c::sub(w(s.rating), 700)?, 10_000)?, 280)?;
    let z = c::div_round(
        c::add(c::mul(6_000, w(s.morale_pts))?, c::mul(4_000, retention)?)?,
        10_000,
    )?;
    let o = c::div_round(
        c::sum(&[
            c::mul(4_500, w(s.equipment_pts))?,
            c::mul(3_000, w(s.experience_pts))?,
            c::mul(2_500, c::sub(10_000, penalty)?)?,
        ])?,
        10_000,
    )?;
    let f = c::div_round(c::add(c::mul(7_000, gi)?, c::mul(3_000, liq)?)?, 10_000)?;
    Ok(RawIndices {
        g: c::narrow(g)?,
        r: c::narrow(r)?,
        z: c::narrow(z)?,
        o: c::narrow(o)?,
        f: c::narrow(f)?,
    })
}

pub fn centred(raw: &RawIndices, reference: &RawIndices) -> Calc<IndexSet> {
    let one = |v: i32, r: i32| -> Calc<u16> {
        let x = c::add(5_000, c::sub(w(v), w(r))?)?.clamp(0, 10_000);
        c::narrow(x)
    };
    IndexSet::from_hundredths([
        one(raw.g, reference.g)?,
        one(raw.r, reference.r)?,
        one(raw.z, reference.z)?,
        one(raw.o, reference.o)?,
        one(raw.f, reference.f)?,
    ])
    .ok_or(CalcError::Overflow)
}

/// GOPPAR prognoze lipnja uz zadani plan, bez situacija.
fn june_goppar(s: &HotelState, cat: &Catalog) -> Result<i64, ResolveError> {
    let plan = MonthPlan::default_for(s, Month::Jun, cat)?;
    Ok(resolve_month(s, Month::Jun, &plan, &[], cat)?
        .money
        .goppar_cents)
}

impl RefTable {
    pub fn compute(cat: &Catalog, category: Category) -> Result<RefTable, ResolveError> {
        let profile = cat
            .reference_profiles
            .get(&category)
            .copied()
            .ok_or_else(|| {
                crate::catalog::CatalogError::Missing(format!(
                    "reference_profiles.{}",
                    category.code()
                ))
            })?;
        let mut s = HotelState::from_profile(cat.profile(profile)?)?;
        let g0 = june_goppar(&s, cat)?;
        let start = RefPoint {
            raw: raw_indices(&s, g0, base_labor_cents(&s, cat)?, g0, cat)?,
            goppar_cents: g0,
        };
        let mut points = Vec::with_capacity(4);
        for m in Month::SEASON {
            let plan = MonthPlan::default_for(&s, m, cat)?;
            let r = resolve_month(&s, m, &plan, &[], cat)?;
            let g = r.money.goppar_cents;
            points.push(RefPoint {
                raw: raw_indices(&r.after, g, r.money.labor_cents, g, cat)?,
                goppar_cents: g,
            });
            s = r.after;
        }
        let [a, b, c4, d]: [RefPoint; 4] = points
            .try_into()
            .map_err(|_| ResolveError::Calc(CalcError::Overflow))?;
        Ok(RefTable {
            start,
            months: MonthTable([a, b, c4, d]),
        })
    }
}

pub fn start_indices(
    s: &HotelState,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<IndexSet, ResolveError> {
    let g = june_goppar(s, cat)?;
    let raw = raw_indices(
        s,
        g,
        base_labor_cents(s, cat)?,
        refs.start.goppar_cents,
        cat,
    )?;
    Ok(centred(&raw, &refs.start.raw)?)
}

pub fn month_indices(
    r: &MonthReport,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<IndexSet, ResolveError> {
    let p = refs.months.get(r.month);
    let raw = raw_indices(
        &r.after,
        r.money.goppar_cents,
        r.money.labor_cents,
        p.goppar_cents,
        cat,
    )?;
    Ok(centred(&raw, &p.raw)?)
}
