//! Plan kruga (razdoblje je tjedan ili mjesec): što student zadaje i granice iz kataloga. Iznosi plana su za krug;
//! mjesečne granice iz kataloga razmjerno se svode na krug (`prorate`), referenca je prosjek po danima kruga.
//! Sezonci, otvorenost, razina cijene i održavanje zadaju se po odjelu (MODEL_V2 §3.8); korak sezonaca vrijedi za
//! zbroj pomaka svih odjela.

use std::collections::BTreeMap;

use engine::calc::{self, CalcError};
use sim_hotel::{CatalogError, Maintenance, Training, euro_bounds};

use crate::calendar::{Period, avg_i64, avg_u16, prorate};
use crate::catalog::Catalog;
use crate::department::{DeptCode, DeptPlan, allocate_seasonal};
use crate::state::HotelState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodPlan {
    pub price_cents: i64,
    pub marketing_cents: i64,
    pub wage_pct: u16,
    pub training: Training,
    pub renovation: bool,
    pub reply_reviews: bool,
    pub departments: BTreeMap<DeptCode, DeptPlan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanError {
    PriceNotWholeEuro,
    PriceOutOfRange,
    MarketingOutOfRange,
    SeasonalOutOfRange,
    SeasonalStepTooBig,
    WageOutOfRange,
    RenovationTwice,
    /// Račun granica nije uspio (katalog izvan granica jezgre).
    Calc,
    /// Plan ima odjel kojeg hotel nema.
    DepartmentUnknown,
    /// Plan nema odjel koji hotel ima.
    DepartmentMissing,
    /// Odjel se ne smije zatvoriti (Smještaj nikad, ostali samo uz `can_close`).
    DepartmentCannotClose,
    /// Sezonaca u odjelu više nego što odjel prima.
    DepartmentSeasonalOutOfRange,
    /// Zatvoren odjel ne smije imati sezonce.
    ClosedWithSeasonal,
    /// Razina cijene odjela izvan granica (Smještaj uvijek 100).
    PriceLevelOutOfRange,
}

impl From<calc::CalcError> for PlanError {
    fn from(_: calc::CalcError) -> Self {
        PlanError::Calc
    }
}

impl From<sim_hotel::PlanError> for PlanError {
    fn from(e: sim_hotel::PlanError) -> Self {
        match e {
            sim_hotel::PlanError::PriceNotWholeEuro => PlanError::PriceNotWholeEuro,
            sim_hotel::PlanError::PriceOutOfRange => PlanError::PriceOutOfRange,
            sim_hotel::PlanError::MarketingOutOfRange => PlanError::MarketingOutOfRange,
            sim_hotel::PlanError::SeasonalOutOfRange => PlanError::SeasonalOutOfRange,
            sim_hotel::PlanError::SeasonalStepTooBig => PlanError::SeasonalStepTooBig,
            sim_hotel::PlanError::WageOutOfRange => PlanError::WageOutOfRange,
            sim_hotel::PlanError::RenovationTwice => PlanError::RenovationTwice,
            sim_hotel::PlanError::Calc => PlanError::Calc,
        }
    }
}

/// Granice jednog odjela: sezonci `0..=seasonal_max` (zbroj pomaka svih odjela drži `seasonal_step_total`), razina
/// cijene u postocima, smije li se zatvoriti.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeptLimits {
    pub seasonal_min: u16,
    pub seasonal_max: u16,
    pub price_level_min_pct: u16,
    pub price_level_max_pct: u16,
    pub can_close: bool,
}

/// Granice plana za jedan krug (koriste ih `validate`, `carry_into`, roboti i, kasnije, sučelje).
/// `seasonal_min`/`seasonal_max` su granice ukupnog broja sezonaca hotela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanLimits {
    pub price_min_cents: i64,
    pub price_max_cents: i64,
    pub marketing_max_cents: i64,
    pub seasonal_min: u16,
    pub seasonal_max: u16,
    pub wage_min_pct: u16,
    pub wage_max_pct: u16,
    pub renovation_available: bool,
    /// Najveći zbroj |cilj − trenutno| po odjelima u ovom krugu.
    pub seasonal_step_total: u16,
    pub departments: BTreeMap<DeptCode, DeptLimits>,
}

fn catalog_calc(_: CalcError) -> CatalogError {
    CatalogError::OutOfRange("period".to_string())
}

/// Cijeli euri u centima: `div_round(x, 100)·100`.
fn whole_euro_cents(x: i64) -> Result<i64, CatalogError> {
    let euros = calc::div_round(i128::from(x), 100).map_err(catalog_calc)?;
    calc::mul(euros, 100)
        .and_then(calc::narrow)
        .map_err(catalog_calc)
}

fn sum_u16(xs: impl Iterator<Item = u16>) -> u16 {
    xs.fold(0_u16, u16::saturating_add)
}

/// Ukupno sezonaca hotela danas.
fn seasonal_now(state: &HotelState) -> u16 {
    sum_u16(state.departments.values().map(|d| d.seasonal))
}

/// Plan sezonaca svakog odjela u krugu (prosjek po danima). Plan kategorije hotela je plan cijelog hotela (kao u v1
/// i u robotima), pa Smještaj dobiva ono što od njega ostane nakon planova ostalih odjela; u degeneriranom katalogu
/// je to točno v1 plan, za bilo koju kategoriju (Task 9 pregled, t. a).
pub(crate) fn dept_seasonal_plans(
    state: &HotelState,
    p: &Period,
    cat: &Catalog,
) -> Result<BTreeMap<DeptCode, u16>, CatalogError> {
    let mut out = BTreeMap::new();
    for code in state.departments.keys() {
        if *code != DeptCode::rooms() {
            let planned = avg_u16(&cat.dept(code)?.seasonal_plan, p).map_err(catalog_calc)?;
            out.insert(code.clone(), planned);
        }
    }
    let hotel =
        avg_u16(&cat.base.category(state.category)?.seasonal_plan, p).map_err(catalog_calc)?;
    let rooms = hotel.saturating_sub(sum_u16(out.values().copied()));
    out.insert(DeptCode::rooms(), rooms);
    Ok(out)
}

/// Raspodjela `total` sezonaca po odjelima (`allocate_seasonal`) prema planovima odjela; zatvoren odjel ne prima
/// nikoga (`max` 0). `open` daje otvorenost odjela u planu.
fn allocate(
    state: &HotelState,
    plans: &BTreeMap<DeptCode, u16>,
    open: &dyn Fn(&DeptCode) -> bool,
    total: u16,
    cat: &Catalog,
) -> Result<BTreeMap<DeptCode, u16>, CatalogError> {
    let current: BTreeMap<DeptCode, u16> = state
        .departments
        .iter()
        .map(|(k, d)| (k.clone(), d.seasonal))
        .collect();
    let mut max = BTreeMap::new();
    for code in state.departments.keys() {
        if open(code) {
            max.insert(code.clone(), cat.dept(code)?.seasonal_max);
        }
    }
    Ok(allocate_seasonal(&current, plans, &max, total))
}

impl PeriodPlan {
    /// Zbroj sezonaca svih odjela (v1 `seasonal_target`).
    pub fn seasonal_total(&self) -> u16 {
        sum_u16(self.departments.values().map(|d| d.seasonal_target))
    }

    /// Razina održavanja Smještaja (v1 `maintenance`); plan bez Smještaja (neispravan) daje redovno.
    pub fn rooms_maintenance(&self) -> Maintenance {
        self.departments
            .get(&DeptCode::rooms())
            .map_or(Maintenance::Regular, |d| d.maintenance)
    }

    /// Ovaj plan s ukupnim sezoncima `total` i razinom održavanja `maintenance` iz v1 oblika tijela plana (API bez
    /// odjela, do Taska 15). Otvorenost i razine cijene odjela ostaju iz ovog plana. Kad se `total` razlikuje od
    /// `seasonal_total()`, sezonci se raspodijele ispočetka od današnjeg stanja prema planovima odjela (kao zadani
    /// plan i roboti; zatvoren odjel ne prima nikoga); kad se razina razlikuje od `rooms_maintenance()`, svi odjeli
    /// dobiju tu razinu. Granice (korak) provjerava `validate`.
    pub fn with_totals(
        &self,
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
        total: u16,
        maintenance: Maintenance,
    ) -> Result<PeriodPlan, CatalogError> {
        let mut out = self.clone();
        if total != self.seasonal_total() {
            let plans = dept_seasonal_plans(state, p, cat)?;
            let open = |k: &DeptCode| self.departments.get(k).is_some_and(|d| d.open);
            let alloc = allocate(state, &plans, &open, total, cat)?;
            for (k, d) in out.departments.iter_mut() {
                d.seasonal_target = alloc.get(k).copied().unwrap_or(0);
            }
        }
        if maintenance != self.rooms_maintenance() {
            for d in out.departments.values_mut() {
                d.maintenance = maintenance;
            }
        }
        Ok(out)
    }

    /// Postavke odjela za `total` sezonaca: otvorenost kao u stanju, raspodjela prema planovima odjela, razina
    /// cijene 100, ista razina održavanja svugdje (zadani plan i roboti).
    pub(crate) fn departments_for(
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
        total: u16,
        maintenance: Maintenance,
    ) -> Result<BTreeMap<DeptCode, DeptPlan>, CatalogError> {
        let plans = dept_seasonal_plans(state, p, cat)?;
        let open = |k: &DeptCode| state.departments.get(k).is_some_and(|d| d.open);
        let alloc = allocate(state, &plans, &open, total, cat)?;
        Ok(state
            .departments
            .iter()
            .map(|(k, d)| {
                (
                    k.clone(),
                    DeptPlan {
                        open: d.open,
                        seasonal_target: alloc.get(k).copied().unwrap_or(0),
                        price_level_pct: 100,
                        maintenance,
                    },
                )
            })
            .collect())
    }

    /// Zadano (ADR-013 t. 3): referentna cijena kruga u cijelom euru, marketing 0, ukupno sezonaca po planovima
    /// otvorenih odjela (najviše ±korak od današnjeg), raspoređeno prema planovima odjela, plaća kao sad, bez obuke,
    /// redovno održavanje svugdje, razine cijene 100, bez obnove i odgovaranja.
    pub fn default_for(
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
    ) -> Result<PeriodPlan, CatalogError> {
        let base = &cat.base;
        let spec = base.category(state.category)?;
        let step = cat.coeffs(p.kind)?.seasonal_step;
        let now = seasonal_now(state);
        let lo = now.saturating_sub(step);
        let hi = now.saturating_add(step);
        let plans = dept_seasonal_plans(state, p, cat)?;
        let planned = sum_u16(
            plans
                .iter()
                .filter(|(k, _)| state.departments.get(*k).is_some_and(|d| d.open))
                .map(|(_, v)| *v),
        );
        let target = planned.clamp(lo, hi.max(lo)).min(base.actions.seasonal_max);
        let reference = avg_i64(&spec.price_ref_cents, p).map_err(catalog_calc)?;
        Ok(PeriodPlan {
            price_cents: whole_euro_cents(reference)?,
            marketing_cents: 0,
            wage_pct: state.wage_pct,
            training: Training::None,
            renovation: false,
            reply_reviews: false,
            departments: PeriodPlan::departments_for(state, p, cat, target, Maintenance::Regular)?,
        })
    }

    /// Granice plana u ovom krugu: cijena u cijelim eurima oko reference kruga, marketing razmjerno krugu,
    /// ukupno sezonaca ±korak vrste kruga, plaća iz kataloga, obnova samo dok nije obavljena; po odjelu sezonci do
    /// `seasonal_max` odjela, razina cijene iz gospodarstva v2 (Smještaj 100), zatvaranje uz `can_close`.
    pub fn limits(
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
    ) -> Result<PlanLimits, CatalogError> {
        let a = &cat.base.actions;
        let spec = cat.base.category(state.category)?;
        let step = cat.coeffs(p.kind)?.seasonal_step;
        let reference = avg_i64(&spec.price_ref_cents, p).map_err(catalog_calc)?;
        let (lo, hi) = euro_bounds(
            i128::from(reference),
            i128::from(a.price_min_pct),
            i128::from(a.price_max_pct),
        )
        .map_err(catalog_calc)?;
        let price_min = calc::mul(lo, 100)
            .and_then(calc::narrow)
            .map_err(catalog_calc)?;
        let price_max = calc::mul(hi.max(lo), 100)
            .and_then(calc::narrow)
            .map_err(catalog_calc)?;
        let now = seasonal_now(state);
        let seasonal_min = now.saturating_sub(step);
        let seasonal_max = now
            .saturating_add(step)
            .min(a.seasonal_max)
            .max(seasonal_min);
        let e = &cat.economy;
        let mut departments = BTreeMap::new();
        for code in state.departments.keys() {
            let d = cat.dept(code)?;
            let rooms = *code == DeptCode::rooms();
            departments.insert(
                code.clone(),
                DeptLimits {
                    seasonal_min: 0,
                    seasonal_max: d.seasonal_max,
                    price_level_min_pct: if rooms { 100 } else { e.price_level_min_pct },
                    price_level_max_pct: if rooms { 100 } else { e.price_level_max_pct },
                    can_close: d.can_close && !rooms,
                },
            );
        }
        Ok(PlanLimits {
            price_min_cents: price_min,
            price_max_cents: price_max,
            marketing_max_cents: prorate(a.marketing_max_cents, p).map_err(catalog_calc)?,
            seasonal_min,
            seasonal_max,
            wage_min_pct: a.wage_min_pct,
            wage_max_pct: a.wage_max_pct,
            renovation_available: !state.renovated,
            seasonal_step_total: step,
            departments,
        })
    }

    /// Provjere redom (vraća se prva pogođena): cijena, marketing, plaća, obnova; struktura odjela; pravila odjela
    /// redom koda, pa ukupno sezonaca; na kraju korak (Σ |cilj − trenutno| po odjelima).
    pub fn validate(&self, state: &HotelState, p: &Period, cat: &Catalog) -> Result<(), PlanError> {
        let l = PeriodPlan::limits(state, p, cat).map_err(|_| PlanError::Calc)?;
        let a = &cat.base.actions;
        // (1) cijena, marketing, plaća, obnova
        let price = i128::from(self.price_cents);
        let whole = calc::div_floor(price, 100)?;
        if calc::mul(whole, 100)? != price {
            return Err(PlanError::PriceNotWholeEuro);
        }
        let spec = cat
            .base
            .category(state.category)
            .map_err(|_| PlanError::Calc)?;
        let reference = i128::from(avg_i64(&spec.price_ref_cents, p)?);
        let scaled = calc::mul(price, 100)?;
        if scaled < calc::mul(i128::from(a.price_min_pct), reference)?
            || scaled > calc::mul(i128::from(a.price_max_pct), reference)?
        {
            return Err(PlanError::PriceOutOfRange);
        }
        if self.marketing_cents < 0 || self.marketing_cents > l.marketing_max_cents {
            return Err(PlanError::MarketingOutOfRange);
        }
        if self.wage_pct < a.wage_min_pct || self.wage_pct > a.wage_max_pct {
            return Err(PlanError::WageOutOfRange);
        }
        if self.renovation && state.renovated {
            return Err(PlanError::RenovationTwice);
        }
        // (2) struktura
        if self
            .departments
            .keys()
            .any(|k| !state.departments.contains_key(k))
        {
            return Err(PlanError::DepartmentUnknown);
        }
        if state
            .departments
            .keys()
            .any(|k| !self.departments.contains_key(k))
        {
            return Err(PlanError::DepartmentMissing);
        }
        // (3) pravila odjela, redom koda
        for (code, d) in &self.departments {
            let dl = l.departments.get(code).ok_or(PlanError::Calc)?;
            if !d.open && !dl.can_close {
                return Err(PlanError::DepartmentCannotClose);
            }
            if !d.open && d.seasonal_target > 0 {
                return Err(PlanError::ClosedWithSeasonal);
            }
            if d.seasonal_target > dl.seasonal_max {
                return Err(PlanError::DepartmentSeasonalOutOfRange);
            }
            if d.price_level_pct < dl.price_level_min_pct
                || d.price_level_pct > dl.price_level_max_pct
            {
                return Err(PlanError::PriceLevelOutOfRange);
            }
        }
        if self.seasonal_total() > a.seasonal_max {
            return Err(PlanError::SeasonalOutOfRange);
        }
        // (4) korak
        let moved = self.departments.iter().fold(0_u16, |acc, (k, d)| {
            let now = state.departments.get(k).map_or(0, |s| s.seasonal);
            acc.saturating_add(d.seasonal_target.abs_diff(now))
        });
        if moved > l.seasonal_step_total {
            return Err(PlanError::SeasonalStepTooBig);
        }
        Ok(())
    }

    /// Plan prošlog kruga za novi krug: obnova se ne ponavlja; cijena (najbliži cijeli euro), marketing i plaća
    /// stisnu se u granice novog kruga. Postavke odjela se prenose: zatvoreni ostaju zatvoreni (osim ako se odjel
    /// ne smije zatvoriti), sezonci se stisnu na `seasonal_max` odjela, ukupno u granice kruga, i raspodijele od
    /// današnjeg stanja prema prenesenim ciljevima (pa korak vrijedi); razina cijene u granice odjela. Rezultat je
    /// valjan.
    pub fn carry_into(
        &self,
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
    ) -> Result<PeriodPlan, CatalogError> {
        let l = PeriodPlan::limits(state, p, cat)?;
        let mut settings: BTreeMap<DeptCode, DeptPlan> = BTreeMap::new();
        for (code, now) in &state.departments {
            let dl = l.departments.get(code).ok_or_else(|| {
                CatalogError::Missing(format!("v2.departments.{}", code.as_str()))
            })?;
            let prev = self.departments.get(code);
            let open = prev.is_none_or(|d| d.open) || !dl.can_close;
            let wanted = match prev {
                Some(d) if open => d.seasonal_target.min(dl.seasonal_max),
                Some(_) => 0,
                None => now.seasonal,
            };
            settings.insert(
                code.clone(),
                DeptPlan {
                    open,
                    seasonal_target: wanted,
                    price_level_pct: prev
                        .map_or(100, |d| d.price_level_pct)
                        .max(dl.price_level_min_pct)
                        .min(dl.price_level_max_pct),
                    maintenance: prev.map_or(Maintenance::Regular, |d| d.maintenance),
                },
            );
        }
        let wanted: BTreeMap<DeptCode, u16> = settings
            .iter()
            .map(|(k, d)| (k.clone(), d.seasonal_target))
            .collect();
        let total = sum_u16(wanted.values().copied())
            .max(l.seasonal_min)
            .min(l.seasonal_max);
        let open = |k: &DeptCode| settings.get(k).is_some_and(|d| d.open);
        let alloc = allocate(state, &wanted, &open, total, cat)?;
        for (k, d) in settings.iter_mut() {
            d.seasonal_target = alloc.get(k).copied().unwrap_or(0);
        }
        Ok(PeriodPlan {
            price_cents: whole_euro_cents(self.price_cents)?
                .max(l.price_min_cents)
                .min(l.price_max_cents),
            marketing_cents: self.marketing_cents.max(0).min(l.marketing_max_cents),
            wage_pct: self.wage_pct.max(l.wage_min_pct).min(l.wage_max_pct),
            training: self.training,
            renovation: false,
            reply_reviews: self.reply_reviews,
            departments: settings,
        })
    }
}
