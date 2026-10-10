//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2). Formule su v1 (`raw_indices`,
//! `centred`, `base_labor_cents` nad `HotelState::summary()`); v2 drži referentnu tablicu po krugovima.
//! Rad u likvidnosti ulazi kao mjesečni ekvivalent (`per_month_equiv`), da tjedni krug daje isti raspon.

use sim_hotel::{
    CatalogError, Category, IndexSet, RefPoint, base_labor_cents, centred, raw_indices,
};

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
        let mut s = HotelState::from_profile(base.profile(profile)?)?;
        let g0 = first_goppar(&s, first, cat)?;
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
