//! Sadržaj modela v2 u snimku pravila: koeficijenti razdoblja, odjeli, profili i gospodarstvo iz `content/hotel/v2/`.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sim_hotel::{Catalog, Profile};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::{PeriodCoefficients, PeriodKind};

use crate::templates::{Names, SituationTemplates, TemplateFile};
use crate::v2_dept::{
    DeptDto, EconomyDto, ProfileDto, build_dept, build_economy, build_profile, dept_to_dto,
    economy_to_dto, profile_to_dto,
};
use crate::{ContentError, read};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PeriodCoefficientsDto {
    morale_step_bp: u16,
    satisfaction_step_bp: u16,
    rating_step_bp: u16,
    seasonal_step: u16,
    /// Udio učinka situacija na bodove (J-3); izostavljen = puni učinak, pa stari v2 snimak daje isti zapis,
    /// otisak i obračun kao prije.
    #[serde(default = "full_effect", skip_serializing_if = "is_full_effect")]
    situation_effect_bp: u16,
    /// Granica zbroja situacija kruga (±bodova); izostavljena = bez granice (kao prije).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    situation_round_bound_pts: Option<u16>,
}

const FULL_EFFECT_BP: u16 = 10_000;

fn full_effect() -> u16 {
    FULL_EFFECT_BP
}

fn is_full_effect(v: &u16) -> bool {
    *v == FULL_EFFECT_BP
}

/// Odjeljak v2 snimka; ključevi `periods` su kodovi vrsta razdoblja (`month`, `week`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct V2Dto {
    periods: BTreeMap<String, PeriodCoefficientsDto>,
    departments: BTreeMap<String, DeptDto>,
    /// Ključevi su kodovi profila (`A`, `B`, `C`).
    profiles: BTreeMap<String, ProfileDto>,
    economy: EconomyDto,
    /// Predlošci situacija (S-1); izostavljeni kad ih nema, pa snimak bez njih daje isti zapis i otisak kao prije.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    situation_templates: Option<TemplateFile>,
    /// Imena likova za uloge u predlošcima; dolaze zajedno s predlošcima.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    names: Option<Names>,
}

/// Čita `dir/v2/{periods,departments,profiles,economy,situation_templates,names}.json`.
pub(crate) fn read_v2_dir(dir: &Path) -> Result<V2Dto, ContentError> {
    let d = dir.join("v2");
    Ok(V2Dto {
        periods: read(&d, "periods.json")?,
        departments: read(&d, "departments.json")?,
        profiles: read(&d, "profiles.json")?,
        economy: read(&d, "economy.json")?,
        situation_templates: Some(read(&d, "situation_templates.json")?),
        names: Some(read(&d, "names.json")?),
    })
}

fn to_dto(c: &PeriodCoefficients) -> PeriodCoefficientsDto {
    PeriodCoefficientsDto {
        morale_step_bp: c.morale_step_bp,
        satisfaction_step_bp: c.satisfaction_step_bp,
        rating_step_bp: c.rating_step_bp,
        seasonal_step: c.seasonal_step,
        situation_effect_bp: c.situation_effect_bp,
        situation_round_bound_pts: c.situation_round_bound_pts,
    }
}

/// Odjeljak v2 koji daje `sim_hotel_v2::degenerate(base)`.
pub(crate) fn degenerate_dto(base: &Catalog) -> V2Dto {
    let d = sim_hotel_v2::degenerate(base);
    V2Dto {
        periods: d
            .periods
            .iter()
            .map(|(k, c)| (k.code().to_string(), to_dto(c)))
            .collect(),
        departments: d
            .departments
            .iter()
            .map(|(k, s)| (k.as_str().to_string(), dept_to_dto(s)))
            .collect(),
        profiles: d
            .profiles
            .iter()
            .map(|(k, p)| (k.code().to_string(), profile_to_dto(p)))
            .collect(),
        economy: economy_to_dto(&d.economy),
        situation_templates: None,
        names: None,
    }
}

/// Provjereni predlošci iz odjeljka; `None` kad ih snimak nema (stare v2 igre, degenerirani katalog).
pub(crate) fn build_templates(
    dto: &V2Dto,
    v2: &sim_hotel_v2::Catalog,
) -> Result<Option<SituationTemplates>, ContentError> {
    match (&dto.situation_templates, &dto.names) {
        (None, None) => Ok(None),
        (Some(file), Some(names)) => {
            crate::templates_check::check(file, names, &v2.base, v2)?;
            Ok(Some(SituationTemplates {
                major: file.major,
                templates: file.templates.clone(),
                names: names.clone(),
                groups_per_season: file.groups_per_season,
                groups_per_month: file.groups_per_month,
            }))
        }
        _ => Err(ContentError::Invalid(
            "v2: predlošci situacija i imena likova dolaze zajedno".to_string(),
        )),
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
                situation_effect_bp: d.situation_effect_bp,
                situation_round_bound_pts: d.situation_round_bound_pts,
            },
        );
    }
    let mut departments = BTreeMap::new();
    for (code, d) in &dto.departments {
        departments.insert(DeptCode(code.clone()), build_dept(code, d)?);
    }
    let mut profiles = BTreeMap::new();
    for (code, p) in &dto.profiles {
        let Some(profile) = Profile::from_code(code) else {
            return Err(ContentError::Invalid(format!(
                "v2.profiles: nepoznat profil '{code}'"
            )));
        };
        profiles.insert(profile, build_profile(p));
    }
    let mut c = sim_hotel_v2::degenerate(base);
    c.periods = periods;
    c.departments = departments;
    c.profiles = profiles;
    c.economy = build_economy(&dto.economy);
    Ok(c)
}
