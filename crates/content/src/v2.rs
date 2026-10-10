//! Sadržaj modela v2 u snimku pravila: koeficijenti po vrsti razdoblja iz `content/hotel/v2/periods.json`.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sim_hotel::Catalog;
use sim_hotel_v2::{PeriodCoefficients, PeriodKind};

use crate::{ContentError, read};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PeriodCoefficientsDto {
    morale_step_bp: u16,
    satisfaction_step_bp: u16,
    rating_step_bp: u16,
    seasonal_step: u16,
}

/// Odjeljak v2 snimka; ključevi `periods` su kodovi vrsta razdoblja (`month`, `week`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct V2Dto {
    periods: BTreeMap<String, PeriodCoefficientsDto>,
}

/// Čita `dir/v2/periods.json`.
pub(crate) fn read_v2_dir(dir: &Path) -> Result<V2Dto, ContentError> {
    Ok(V2Dto {
        periods: read(&dir.join("v2"), "periods.json")?,
    })
}

fn to_dto(c: &PeriodCoefficients) -> PeriodCoefficientsDto {
    PeriodCoefficientsDto {
        morale_step_bp: c.morale_step_bp,
        satisfaction_step_bp: c.satisfaction_step_bp,
        rating_step_bp: c.rating_step_bp,
        seasonal_step: c.seasonal_step,
    }
}

/// Odjeljak v2 koji daje `sim_hotel_v2::degenerate(base)`.
pub(crate) fn degenerate_dto(base: &Catalog) -> V2Dto {
    V2Dto {
        periods: sim_hotel_v2::degenerate(base)
            .periods
            .iter()
            .map(|(k, c)| (k.code().to_string(), to_dto(c)))
            .collect(),
    }
}

/// Gradi katalog v2 iz odjeljka. Počinje od `degenerate`, pa mijenja samo ono što odjeljak nosi, da novi dijelovi
/// kataloga (kasniji taskovi) ne traže izmjenu ovdje.
pub(crate) fn build_v2(dto: &V2Dto, base: &Catalog) -> Result<sim_hotel_v2::Catalog, ContentError> {
    let mut periods = BTreeMap::new();
    for (code, d) in &dto.periods {
        let Some(kind) = PeriodKind::from_code(code) else {
            return Err(ContentError::Invalid(format!(
                "v2.periods: nepoznata vrsta razdoblja '{code}'"
            )));
        };
        periods.insert(
            kind,
            PeriodCoefficients {
                morale_step_bp: d.morale_step_bp,
                satisfaction_step_bp: d.satisfaction_step_bp,
                rating_step_bp: d.rating_step_bp,
                seasonal_step: d.seasonal_step,
            },
        );
    }
    let mut c = sim_hotel_v2::degenerate(base);
    c.periods = periods;
    Ok(c)
}
