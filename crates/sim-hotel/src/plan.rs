//! Plan mjeseca (DEMO §3.1): što student zadaje i granice iz kataloga (P6).

use engine::calc;

use crate::catalog::{Catalog, CatalogError};
use crate::month::Month;
use crate::state::HotelState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maintenance {
    Deferred,
    Regular,
    Intensive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Training {
    None,
    Basic,
    Full,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthPlan {
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

impl MonthPlan {
    /// Zadano (ADR-013 t. 3): referentna cijena, marketing 0, sezonci po planu kataloga (najviše ±korak od
    /// današnjeg), plaća kao sad, bez obuke, redovno održavanje, bez obnove i odgovaranja.
    pub fn default_for(
        state: &HotelState,
        month: Month,
        cat: &Catalog,
    ) -> Result<MonthPlan, CatalogError> {
        let spec = cat.category(state.category)?;
        let step = cat.actions.seasonal_step;
        let lo = state.seasonal.saturating_sub(step);
        let hi = state.seasonal.saturating_add(step);
        let target = spec
            .seasonal_plan
            .get(month)
            .clamp(lo, hi.max(lo))
            .min(cat.actions.seasonal_max);
        Ok(MonthPlan {
            price_cents: spec.price_ref_cents.get(month),
            marketing_cents: 0,
            seasonal_target: target,
            wage_pct: state.wage_pct,
            training: Training::None,
            maintenance: Maintenance::Regular,
            renovation: false,
            reply_reviews: false,
        })
    }

    pub fn validate(
        &self,
        state: &HotelState,
        month: Month,
        cat: &Catalog,
    ) -> Result<(), PlanError> {
        let spec = cat.category(state.category).map_err(|_| PlanError::Calc)?;
        let a = &cat.actions;
        let price = i128::from(self.price_cents);
        let whole = calc::div_floor(price, 100)?;
        if calc::mul(whole, 100)? != price {
            return Err(PlanError::PriceNotWholeEuro);
        }
        let reference = i128::from(spec.price_ref_cents.get(month));
        let scaled = calc::mul(price, 100)?;
        if scaled < calc::mul(i128::from(a.price_min_pct), reference)?
            || scaled > calc::mul(i128::from(a.price_max_pct), reference)?
        {
            return Err(PlanError::PriceOutOfRange);
        }
        if self.marketing_cents < 0 || self.marketing_cents > a.marketing_max_cents {
            return Err(PlanError::MarketingOutOfRange);
        }
        if self.seasonal_target > a.seasonal_max {
            return Err(PlanError::SeasonalOutOfRange);
        }
        if self.seasonal_target.abs_diff(state.seasonal) > a.seasonal_step {
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
}
