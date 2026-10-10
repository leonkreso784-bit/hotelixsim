//! Katalog v2: v1 katalog (`base`) i koeficijenti po vrsti razdoblja (MODEL_V2 §4.3). Pomaci prema cilju
//! (moral, zadovoljstvo, ocjena) ovise o duljini kruga; mjesečni koeficijenti su v1 konstante.

use std::collections::BTreeMap;

use sim_hotel::CatalogError;

use crate::calendar::PeriodKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodCoefficients {
    /// Udio razlike do ciljnog morala koji se zatvori u jednom krugu (bp).
    pub morale_step_bp: u16,
    /// Udio razlike do ciljnog zadovoljstva gostiju u jednom krugu (bp).
    pub satisfaction_step_bp: u16,
    /// Udio razlike do ciljne ocjene u jednom krugu (bp).
    pub rating_step_bp: u16,
    /// Najveća promjena broja sezonaca u jednom krugu (ljudi).
    pub seasonal_step: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub base: sim_hotel::Catalog,
    pub periods: BTreeMap<PeriodKind, PeriodCoefficients>,
}

fn step_within(v: u16, field: &str) -> Result<(), CatalogError> {
    if (1..=10_000).contains(&v) {
        Ok(())
    } else {
        Err(CatalogError::OutOfRange(field.to_string()))
    }
}

impl Catalog {
    pub fn coeffs(&self, k: PeriodKind) -> Result<&PeriodCoefficients, CatalogError> {
        self.periods
            .get(&k)
            .ok_or_else(|| CatalogError::Missing(format!("v2.periods.{}", k.code())))
    }

    /// v1 provjera pa koeficijenti obje vrste razdoblja: pomaci u `1..=10_000` bp, korak sezonaca barem 1.
    pub fn validate(&self) -> Result<(), CatalogError> {
        self.base.validate()?;
        for k in PeriodKind::ALL {
            let p = self.coeffs(k)?;
            let field = |name: &str| format!("v2.periods.{}.{name}", k.code());
            step_within(p.morale_step_bp, &field("morale_step_bp"))?;
            step_within(p.satisfaction_step_bp, &field("satisfaction_step_bp"))?;
            step_within(p.rating_step_bp, &field("rating_step_bp"))?;
            if p.seasonal_step < 1 {
                return Err(CatalogError::OutOfRange(field("seasonal_step")));
            }
        }
        Ok(())
    }
}

/// Katalog v2 koji u mjesečnoj igri daje točno v1 (konstante iz `sim_hotel::resolve`); tjedni koeficijenti
/// su procjena [PROC] do sadržaja iz `content/hotel/v2/periods.json` (Task 4).
pub fn degenerate(base: &sim_hotel::Catalog) -> Catalog {
    Catalog {
        base: base.clone(),
        periods: BTreeMap::from([
            (
                PeriodKind::Month,
                PeriodCoefficients {
                    morale_step_bp: 3_000,
                    satisfaction_step_bp: 5_000,
                    rating_step_bp: 2_500,
                    seasonal_step: base.actions.seasonal_step,
                },
            ),
            (
                PeriodKind::Week,
                PeriodCoefficients {
                    morale_step_bp: 800,
                    satisfaction_step_bp: 1_500,
                    rating_step_bp: 700,
                    seasonal_step: 5,
                },
            ),
        ]),
    }
}
