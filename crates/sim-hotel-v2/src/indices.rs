//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2). Formule su v1 (`raw_indices`,
//! `centred`, `base_labor_cents` nad `HotelState::summary()`); v2 drži samo referentnu tablicu po krugovima.

use sim_hotel::{
    CatalogError, Category, IndexSet, Month, RefPoint, base_labor_cents, centred, raw_indices,
};

use crate::catalog::Catalog;
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_month};
use crate::state::HotelState;

/// Referentna točka na početku i po jedna za svaki krug sezone (redom `Period::index`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTable {
    pub start: RefPoint,
    pub points: Vec<RefPoint>,
}

/// GOPPAR prognoze lipnja uz zadani plan, bez situacija.
fn june_goppar(s: &HotelState, cat: &Catalog) -> Result<i64, ResolveError> {
    let plan = PeriodPlan::default_for(s, Month::Jun, cat)?;
    Ok(resolve_month(s, Month::Jun, &plan, &[], cat)?
        .summary
        .money
        .goppar_cents)
}

impl RefTable {
    pub fn compute(cat: &Catalog, category: Category) -> Result<RefTable, ResolveError> {
        let base = &cat.base;
        let profile = base
            .reference_profiles
            .get(&category)
            .copied()
            .ok_or_else(|| {
                CatalogError::Missing(format!("reference_profiles.{}", category.code()))
            })?;
        let mut s = HotelState::from_profile(base.profile(profile)?)?;
        let g0 = june_goppar(&s, cat)?;
        let start = RefPoint {
            raw: raw_indices(
                &s.summary(),
                g0,
                base_labor_cents(&s.summary(), base)?,
                g0,
                base,
            )?,
            goppar_cents: g0,
        };
        let mut points = Vec::with_capacity(Month::SEASON.len());
        for m in Month::SEASON {
            let plan = PeriodPlan::default_for(&s, m, cat)?;
            let r = resolve_month(&s, m, &plan, &[], cat)?;
            let money = &r.summary.money;
            let g = money.goppar_cents;
            points.push(RefPoint {
                raw: raw_indices(&r.summary.after, g, money.labor_cents, g, base)?,
                goppar_cents: g,
            });
            s = r.after;
        }
        Ok(RefTable { start, points })
    }
}

pub fn start_indices(
    s: &HotelState,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<IndexSet, ResolveError> {
    let base = &cat.base;
    let g = june_goppar(s, cat)?;
    let summary = s.summary();
    let raw = raw_indices(
        &summary,
        g,
        base_labor_cents(&summary, base)?,
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
    let p = refs
        .points
        .get(usize::from(r.period.index))
        .ok_or_else(|| CatalogError::Missing(format!("ref_table.{}", r.period.code())))?;
    let money = &r.summary.money;
    let raw = raw_indices(
        &r.summary.after,
        money.goppar_cents,
        money.labor_cents,
        p.goppar_cents,
        &cat.base,
    )?;
    Ok(centred(&raw, &p.raw)?)
}
