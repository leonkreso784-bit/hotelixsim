//! Plan kruga (razdoblje je tjedan ili mjesec): što student zadaje i granice iz kataloga. Iznosi plana su za krug;
//! mjesečne granice iz kataloga razmjerno se svode na krug (`prorate`), referenca je prosjek po danima kruga.

use engine::calc::{self, CalcError};
use sim_hotel::{CatalogError, Maintenance, Training, euro_bounds};

use crate::calendar::{Period, avg_i64, avg_u16, prorate};
use crate::catalog::Catalog;
use crate::state::HotelState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodPlan {
    pub price_cents: i64,
    pub marketing_cents: i64,
    pub seasonal_target: u16,
    pub wage_pct: u16,
    pub training: Training,
    pub maintenance: Maintenance,
    pub renovation: bool,
    pub reply_reviews: bool,
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

/// Granice plana za jedan krug (koriste ih `validate`, `carry_into`, roboti i, kasnije, sučelje).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanLimits {
    pub price_min_cents: i64,
    pub price_max_cents: i64,
    pub marketing_max_cents: i64,
    pub seasonal_min: u16,
    pub seasonal_max: u16,
    pub wage_min_pct: u16,
    pub wage_max_pct: u16,
    pub renovation_available: bool,
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

impl PeriodPlan {
    /// Zadano (ADR-013 t. 3): referentna cijena kruga u cijelom euru, marketing 0, sezonci po planu kataloga
    /// (najviše ±korak od današnjeg), plaća kao sad, bez obuke, redovno održavanje, bez obnove i odgovaranja.
    pub fn default_for(
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
    ) -> Result<PeriodPlan, CatalogError> {
        let base = &cat.base;
        let spec = base.category(state.category)?;
        let step = cat.coeffs(p.kind)?.seasonal_step;
        let lo = state.seasonal.saturating_sub(step);
        let hi = state.seasonal.saturating_add(step);
        let planned = avg_u16(&spec.seasonal_plan, p).map_err(catalog_calc)?;
        let target = planned.clamp(lo, hi.max(lo)).min(base.actions.seasonal_max);
        let reference = avg_i64(&spec.price_ref_cents, p).map_err(catalog_calc)?;
        Ok(PeriodPlan {
            price_cents: whole_euro_cents(reference)?,
            marketing_cents: 0,
            seasonal_target: target,
            wage_pct: state.wage_pct,
            training: Training::None,
            maintenance: Maintenance::Regular,
            renovation: false,
            reply_reviews: false,
        })
    }

    /// Granice plana u ovom krugu: cijena u cijelim eurima oko reference kruga, marketing razmjerno krugu,
    /// sezonci ±korak vrste kruga, plaća iz kataloga, obnova samo dok nije obavljena.
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
        let seasonal_min = state.seasonal.saturating_sub(step);
        let seasonal_max = state
            .seasonal
            .saturating_add(step)
            .min(a.seasonal_max)
            .max(seasonal_min);
        Ok(PlanLimits {
            price_min_cents: price_min,
            price_max_cents: price_max,
            marketing_max_cents: prorate(a.marketing_max_cents, p).map_err(catalog_calc)?,
            seasonal_min,
            seasonal_max,
            wage_min_pct: a.wage_min_pct,
            wage_max_pct: a.wage_max_pct,
            renovation_available: !state.renovated,
        })
    }

    pub fn validate(&self, state: &HotelState, p: &Period, cat: &Catalog) -> Result<(), PlanError> {
        let base = &cat.base;
        let spec = base.category(state.category).map_err(|_| PlanError::Calc)?;
        let a = &base.actions;
        let step = cat
            .coeffs(p.kind)
            .map_err(|_| PlanError::Calc)?
            .seasonal_step;
        let price = i128::from(self.price_cents);
        let whole = calc::div_floor(price, 100)?;
        if calc::mul(whole, 100)? != price {
            return Err(PlanError::PriceNotWholeEuro);
        }
        let reference = i128::from(avg_i64(&spec.price_ref_cents, p)?);
        let scaled = calc::mul(price, 100)?;
        if scaled < calc::mul(i128::from(a.price_min_pct), reference)?
            || scaled > calc::mul(i128::from(a.price_max_pct), reference)?
        {
            return Err(PlanError::PriceOutOfRange);
        }
        if self.marketing_cents < 0 || self.marketing_cents > prorate(a.marketing_max_cents, p)? {
            return Err(PlanError::MarketingOutOfRange);
        }
        if self.seasonal_target > a.seasonal_max {
            return Err(PlanError::SeasonalOutOfRange);
        }
        if self.seasonal_target.abs_diff(state.seasonal) > step {
            return Err(PlanError::SeasonalStepTooBig);
        }
        if self.wage_pct < a.wage_min_pct || self.wage_pct > a.wage_max_pct {
            return Err(PlanError::WageOutOfRange);
        }
        if self.renovation && state.renovated {
            return Err(PlanError::RenovationTwice);
        }
        Ok(())
    }

    /// Plan prošlog kruga za novi krug: obnova se ne ponavlja; cijena (najbliži cijeli euro), marketing,
    /// sezonci i plaća stisnu se u granice novog kruga, pa je rezultat valjan.
    pub fn carry_into(
        &self,
        state: &HotelState,
        p: &Period,
        cat: &Catalog,
    ) -> Result<PeriodPlan, CatalogError> {
        let l = PeriodPlan::limits(state, p, cat)?;
        Ok(PeriodPlan {
            price_cents: whole_euro_cents(self.price_cents)?
                .max(l.price_min_cents)
                .min(l.price_max_cents),
            marketing_cents: self.marketing_cents.max(0).min(l.marketing_max_cents),
            seasonal_target: self.seasonal_target.max(l.seasonal_min).min(l.seasonal_max),
            wage_pct: self.wage_pct.max(l.wage_min_pct).min(l.wage_max_pct),
            training: self.training,
            maintenance: self.maintenance,
            renovation: false,
            reply_reviews: self.reply_reviews,
        })
    }
}
