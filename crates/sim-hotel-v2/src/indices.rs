//! Indeksi centrirani na referentni hotel (DEMO §4, ADR-013 t. 1–2). Formule su v1 (`raw_indices`,
//! `centred` nad `HotelState::summary()`); v2 drži referentnu tablicu po krugovima. Rad u likvidnosti na početku je
//! `start_labor_cents` (plaće odjela v2), u krugu mjesečni ekvivalent stvarnog rada (`per_month_equiv`), da tjedni
//! krug daje isti raspon.
//!
//! Financijski indeks F (obnova-v2-spec §3.9, O12): uz `FinanceBasis::Gop` v1 funkcija dobiva GOPPAR kruga; uz
//! `FinanceBasis::Ebit` dobiva GOPPAR referentnog hotela pomaknut za razliku EBITPAR-a prema referentnom hotelu
//! (`x = ref_goppar + (ebitpar − ref_ebitpar)`), pa je brojnik razlika EBIT-a, a nazivnik ostaje GOPPAR referentnog
//! hotela. Uz `FinanceBasis::GopAfterInvestment` (O-2 d′) `x = goppar − (amortizacija po sobi i danu − ista
//! referentnog hotela)`: bez amortizacije točno GOP. v1 funkcija se ne mijenja; referentni hotel prema sebi daje `x = g`.

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
    /// Amortizacija referentnog hotela po sobi i danu (početak: prognoza prvog kruga) za F iz GOP-a nakon ulaganja.
    pub start_deppar_cents: i64,
    pub deppar_cents: Vec<i64>,
}

/// Brojke kruga koje ulaze u indeks F: GOPPAR, EBITPAR i amortizacija po sobi i danu.
#[derive(Clone, Copy)]
struct FinancePoint {
    goppar: i64,
    ebitpar: i64,
    deppar: i64,
}

impl FinancePoint {
    /// Amortizacija kruga po sobi i danu: `div_round(amortizacija, sobe · dani)` (nazivnik kao GOPPAR).
    fn of(r: &PeriodReport) -> Result<FinancePoint, ResolveError> {
        let nights = c::mul(i128::from(r.accounts.rooms), i128::from(r.accounts.days))?;
        Ok(FinancePoint {
            goppar: r.summary.money.goppar_cents,
            ebitpar: r.finance.ebitpar_cents,
            deppar: c::narrow(c::div_round(
                i128::from(r.finance.depreciation_cents),
                nights,
            )?)?,
        })
    }
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

/// GOPPAR, EBITPAR i amortizacija po sobi i danu prognoze prvog kruga uz zadani plan, bez situacija.
fn first_forecast(
    s: &HotelState,
    first: &Period,
    cat: &Catalog,
) -> Result<FinancePoint, ResolveError> {
    let plan = PeriodPlan::default_for(s, first, cat)?;
    let r = resolve_period(s, first, &plan, &[], cat)?;
    FinancePoint::of(&r)
}

/// „GOPPAR" koji se predaje v1 `raw_indices`: uz osnovu GOP stvarni GOPPAR, uz osnovu EBIT GOPPAR referentnog
/// hotela pomaknut za razliku EBITPAR-a (F11), uz GOP nakon ulaganja GOPPAR umanjen za razliku amortizacije po
/// sobi i danu prema referentnom hotelu.
fn finance_input(
    cat: &Catalog,
    me: FinancePoint,
    reference: FinancePoint,
) -> Result<i64, ResolveError> {
    match cat.economy.finance_index {
        FinanceBasis::Gop => Ok(me.goppar),
        FinanceBasis::Ebit => Ok(c::narrow(c::add(
            i128::from(reference.goppar),
            c::sub(i128::from(me.ebitpar), i128::from(reference.ebitpar))?,
        )?)?),
        FinanceBasis::GopAfterInvestment => Ok(c::narrow(c::sub(
            i128::from(me.goppar),
            c::sub(i128::from(me.deppar), i128::from(reference.deppar))?,
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
        let f0 = first_forecast(&s, first, cat)?;
        let g0 = f0.goppar;
        let start = RefPoint {
            raw: raw_indices(&s.summary(), g0, start_labor_cents(&s, cat)?, g0, base)?,
            goppar_cents: g0,
        };
        let mut points = Vec::with_capacity(periods.len());
        let mut ebitpar_cents = Vec::with_capacity(periods.len());
        let mut deppar_cents = Vec::with_capacity(periods.len());
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
            let f = FinancePoint::of(&r)?;
            ebitpar_cents.push(f.ebitpar);
            deppar_cents.push(f.deppar);
            s = r.after;
        }
        Ok(RefTable {
            start,
            points,
            periods: periods.to_vec(),
            start_ebitpar_cents: f0.ebitpar,
            ebitpar_cents,
            start_deppar_cents: f0.deppar,
            deppar_cents,
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
        FinancePoint {
            goppar: refs.start.goppar_cents,
            ebitpar: refs.start_ebitpar_cents,
            deppar: refs.start_deppar_cents,
        },
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
    let reference = FinancePoint {
        goppar: p.goppar_cents,
        ebitpar: refs.ebitpar_cents.get(i).copied().ok_or_else(missing)?,
        deppar: refs.deppar_cents.get(i).copied().ok_or_else(missing)?,
    };
    let money = &r.summary.money;
    let x = finance_input(cat, FinancePoint::of(r)?, reference)?;
    let raw = raw_indices(
        &r.summary.after,
        x,
        per_month_equiv(money.labor_cents, &r.period)?,
        p.goppar_cents,
        &cat.base,
    )?;
    Ok(centred(&raw, &p.raw)?)
}
