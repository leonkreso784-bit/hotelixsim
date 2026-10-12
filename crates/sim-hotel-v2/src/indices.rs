//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2). Formule su v1 (`raw_indices`,
//! `centred` nad `HotelState::summary()`); v2 drži referentnu tablicu po krugovima. Rad u likvidnosti na početku je
//! `start_labor_cents` (plaće odjela v2), u krugu mjesečni ekvivalent stvarnog rada (`per_month_equiv`), da tjedni
//! krug daje isti raspon.
//!
//! Financijski indeks F (obnova-v2-spec §3.9, O12): uz `FinanceBasis::Gop` v1 funkcija dobiva GOPPAR kruga; uz
//! `FinanceBasis::Ebit` dobiva GOPPAR referentnog hotela pomaknut za razliku EBITPAR-a prema referentnom hotelu
//! (`x = ref_goppar + (ebitpar − ref_ebitpar)`), pa je brojnik razlika EBIT-a, a nazivnik ostaje GOPPAR referentnog
//! hotela. v1 funkcija se ne mijenja; referentni hotel prema sebi daje `x = g`.

use engine::calc as c;
use sim_hotel::{CatalogError, Category, IndexSet, RefPoint, centred, raw_indices};

use crate::calendar::{Period, per_month_equiv};
use crate::catalog::{Catalog, FinanceBasis};
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_period};
use crate::state::HotelState;

/// Referentna točka na početku i po jedna za svaki krug sezone (redom `Period::index`); `periods` su krugovi
/// za koje je tablica izračunata (prvi određuje početnu prognozu GOPPAR-a). Uz svaku točku EBITPAR referentnog
/// hotela (početak: prognoza prvog kruga) za indeks F iz EBIT-a.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTable {
    pub start: RefPoint,
    pub points: Vec<RefPoint>,
    pub periods: Vec<Period>,
    pub start_ebitpar_cents: i64,
    pub ebitpar_cents: Vec<i64>,
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

/// GOPPAR i EBITPAR prognoze prvog kruga uz zadani plan, bez situacija.
fn first_forecast(
    s: &HotelState,
    first: &Period,
    cat: &Catalog,
) -> Result<(i64, i64), ResolveError> {
    let plan = PeriodPlan::default_for(s, first, cat)?;
    let r = resolve_period(s, first, &plan, &[], cat)?;
    Ok((r.summary.money.goppar_cents, r.finance.ebitpar_cents))
}

/// „GOPPAR" koji se predaje v1 `raw_indices`: uz osnovu GOP stvarni GOPPAR, uz osnovu EBIT GOPPAR referentnog
/// hotela pomaknut za razliku EBITPAR-a (F11).
fn finance_input(
    cat: &Catalog,
    (goppar, ebitpar): (i64, i64),
    (ref_goppar, ref_ebitpar): (i64, i64),
) -> Result<i64, ResolveError> {
    match cat.economy.finance_index {
        FinanceBasis::Gop => Ok(goppar),
        FinanceBasis::Ebit => Ok(c::narrow(c::add(
            i128::from(ref_goppar),
            c::sub(i128::from(ebitpar), i128::from(ref_ebitpar))?,
        )?)?),
    }
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
        let (g0, e0) = first_forecast(&s, first, cat)?;
        let start = RefPoint {
            raw: raw_indices(&s.summary(), g0, start_labor_cents(&s, cat)?, g0, base)?,
            goppar_cents: g0,
        };
        let mut points = Vec::with_capacity(periods.len());
        let mut ebitpar_cents = Vec::with_capacity(periods.len());
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
            ebitpar_cents.push(r.finance.ebitpar_cents);
            s = r.after;
        }
        Ok(RefTable {
            start,
            points,
            periods: periods.to_vec(),
            start_ebitpar_cents: e0,
            ebitpar_cents,
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
    let forecast = first_forecast(s, first, cat)?;
    let x = finance_input(
        cat,
        forecast,
        (refs.start.goppar_cents, refs.start_ebitpar_cents),
    )?;
    let summary = s.summary();
    let raw = raw_indices(
        &summary,
        x,
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
    let missing = || CatalogError::Missing(format!("ref_table.{}", r.period.code()));
    let i = usize::from(r.period.index);
    let p = refs.points.get(i).ok_or_else(missing)?;
    let ref_ebitpar = refs.ebitpar_cents.get(i).copied().ok_or_else(missing)?;
    let money = &r.summary.money;
    let x = finance_input(
        cat,
        (money.goppar_cents, r.finance.ebitpar_cents),
        (p.goppar_cents, ref_ebitpar),
    )?;
    let raw = raw_indices(
        &r.summary.after,
        x,
        per_month_equiv(money.labor_cents, &r.period)?,
        p.goppar_cents,
        &cat.base,
    )?;
    Ok(centred(&raw, &p.raw)?)
}
