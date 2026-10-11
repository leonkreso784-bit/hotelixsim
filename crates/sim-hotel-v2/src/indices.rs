//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2). Formule su v1 (`raw_indices`,
//! `centred` nad `HotelState::summary()`); v2 drži referentnu tablicu po krugovima. Rad u likvidnosti na početku je
//! `start_labor_cents` (plaće odjela v2), u krugu mjesečni ekvivalent stvarnog rada (`per_month_equiv`), da tjedni
//! krug daje isti raspon.

use engine::calc as c;
use sim_hotel::{CatalogError, Category, IndexSet, RefPoint, centred, raw_indices};

use crate::calendar::{Period, per_month_equiv};
use crate::catalog::Catalog;
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_period};
use crate::state::HotelState;

/// Referentna točka na početku i po jedna za svaki krug sezone (redom `Period::index`); `periods` su krugovi
/// za koje je tablica izračunata (prvi određuje početnu prognozu GOPPAR-a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTable {
    pub start: RefPoint,
    pub points: Vec<RefPoint>,
    pub periods: Vec<Period>,
}

/// Mjesečni rad na početku (likvidnost u početnim indeksima): Σ po odjelima zaposleni · ⌊plaća odjela / 100⌋ ·
/// postotak plaće, plus trajne povišice — isto pravilo kao plaće odjela u krugu. U degeneriranom katalogu svi su
/// ljudi u Smještaju s plaćom v1 kategorije, pa je to točno v1 `base_labor_cents`.
pub fn start_labor_cents(s: &HotelState, cat: &Catalog) -> Result<i64, ResolveError> {
    let mut wage: i128 = 0;
    for (code, d) in &s.departments {
        let staff = c::add(i128::from(d.permanent), i128::from(d.seasonal))?;
        let unit = c::div_floor(i128::from(cat.dept(code)?.wage_cents), 100)?;
        wage = c::add(wage, c::mul(c::mul(staff, unit)?, i128::from(s.wage_pct))?)?;
    }
    Ok(c::narrow(c::add(wage, i128::from(s.extra_wage_cents))?)?)
}

/// GOPPAR prognoze prvog kruga uz zadani plan, bez situacija.
fn first_goppar(s: &HotelState, first: &Period, cat: &Catalog) -> Result<i64, ResolveError> {
    let plan = PeriodPlan::default_for(s, first, cat)?;
    Ok(resolve_period(s, first, &plan, &[], cat)?
        .summary
        .money
        .goppar_cents)
}

impl RefTable {
    pub fn compute(
        cat: &Catalog,
        category: Category,
        periods: &[Period],
    ) -> Result<RefTable, ResolveError> {
        let first = periods
            .first()
            .ok_or_else(|| CatalogError::Missing("ref_table.periods".to_string()))?;
        let base = &cat.base;
        let profile = base
            .reference_profiles
            .get(&category)
            .copied()
            .ok_or_else(|| {
                CatalogError::Missing(format!("reference_profiles.{}", category.code()))
            })?;
        let mut s = HotelState::from_profile(cat, profile)?;
        let g0 = first_goppar(&s, first, cat)?;
        let start = RefPoint {
            raw: raw_indices(&s.summary(), g0, start_labor_cents(&s, cat)?, g0, base)?,
            goppar_cents: g0,
        };
        let mut points = Vec::with_capacity(periods.len());
        for p in periods {
            let plan = PeriodPlan::default_for(&s, p, cat)?;
            let r = resolve_period(&s, p, &plan, &[], cat)?;
            let money = &r.summary.money;
            let g = money.goppar_cents;
            let labor = per_month_equiv(money.labor_cents, p)?;
            points.push(RefPoint {
                raw: raw_indices(&r.summary.after, g, labor, g, base)?,
                goppar_cents: g,
            });
            s = r.after;
        }
        Ok(RefTable {
            start,
            points,
            periods: periods.to_vec(),
        })
    }
}

pub fn start_indices(
    s: &HotelState,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<IndexSet, ResolveError> {
    let base = &cat.base;
    let first = refs
        .periods
        .first()
        .ok_or_else(|| CatalogError::Missing("ref_table.periods".to_string()))?;
    let g = first_goppar(s, first, cat)?;
    let summary = s.summary();
    let raw = raw_indices(
        &summary,
        g,
        start_labor_cents(s, cat)?,
        refs.start.goppar_cents,
        base,
    )?;
    Ok(centred(&raw, &refs.start.raw)?)
}

pub fn period_indices(
    r: &PeriodReport,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<IndexSet, ResolveError> {
    // Izvješće se ne smije ocjenjivati prema tablici druge vrste kruga (tjedan prema mjesečnoj i obratno).
    if refs.periods.get(usize::from(r.period.index)) != Some(&r.period) {
        return Err(CatalogError::Missing(format!("ref_table.{}", r.period.code())).into());
    }
    let p = refs
        .points
        .get(usize::from(r.period.index))
        .ok_or_else(|| CatalogError::Missing(format!("ref_table.{}", r.period.code())))?;
    let money = &r.summary.money;
    let raw = raw_indices(
        &r.summary.after,
        money.goppar_cents,
        per_month_equiv(money.labor_cents, &r.period)?,
        p.goppar_cents,
        &cat.base,
    )?;
    Ok(centred(&raw, &p.raw)?)
}
