//! Učitavanje kataloga, profila i predložaka situacija iz `content/hotel/*.json` (ARHITEKTURA §9.5).
//! DTO-i su ovdje (serde), jezgra `sim_hotel` ostaje bez serdea.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sim_hotel::{
    ActionSpec, Catalog, Category, CategorySpec, Economy, LevelEffect, Mechanic, MechanicSpec,
    Month, MonthTable, OfferedAnswer, Profile, ProfileSpec, SituationTemplate, TrainingEffect,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    Io(PathBuf, String),
    Json(PathBuf, String),
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct Content {
    pub catalog: Catalog,
    pub situations: Vec<SituationTemplate>,
}

/// Direktorij sadržaja u repou (`content/hotel`).
pub fn repo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/hotel")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MonthsDto<T> {
    jun: T,
    jul: T,
    aug: T,
    sep: T,
}

impl<T> MonthsDto<T> {
    fn table(self) -> MonthTable<T> {
        MonthTable([self.jun, self.jul, self.aug, self.sep])
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryDto {
    price_ref_cents: MonthsDto<i64>,
    base_occupancy_bp: MonthsDto<u16>,
    seasonal_plan: MonthsDto<u16>,
    elasticity_bp: i32,
    wage_cents: i64,
    cpor_cents: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomyDto {
    other_revenue_bp: u16,
    hire_cents: i64,
    interest_bp: u16,
    marketing_half_cents: i64,
    marketing_max_bp: u16,
    goppar_floor_cents: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelDto {
    cost_cents: i64,
    equipment_pts: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainingDto {
    cost_cents: i64,
    experience_pts: i32,
    morale_pts: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionsDto {
    maintenance_deferred: LevelDto,
    maintenance_regular: LevelDto,
    maintenance_intensive: LevelDto,
    training_basic: TrainingDto,
    training_full: TrainingDto,
    renovation_cents: i64,
    renovation_equipment_pts: i32,
    renovation_satisfaction_pts: i32,
    reply_cents: i64,
    reply_rating: i32,
    price_min_pct: u16,
    price_max_pct: u16,
    marketing_max_cents: i64,
    seasonal_max: u16,
    seasonal_step: u16,
    wage_min_pct: u16,
    wage_max_pct: u16,
    release_morale_pts: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MechanicsDto {
    recovery_pts_per_q: i32,
    retention_pts_per_q: i32,
    crisis_pts_per_q: i32,
    repair_urgent_cents: i64,
    repair_temporary_cents: i64,
    repair_temporary_equipment_pts: i32,
    satisfaction_bound_pts: i32,
    morale_bound_pts: i32,
    rating_bound: i32,
    group_good_quality_cut_bp: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogDto {
    categories: BTreeMap<String, CategoryDto>,
    reference_profiles: BTreeMap<String, String>,
    economy: EconomyDto,
    actions: ActionsDto,
    mechanics: MechanicsDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDto {
    category: String,
    rooms: u16,
    permanent: u16,
    seasonal: u16,
    wage_pct: u16,
    morale_pts: i32,
    experience_pts: i32,
    equipment_pts: i32,
    satisfaction_pts: i32,
    rating: u16,
    cash_cents: i64,
    fixed_cents: i64,
    load_hundredths: u16,
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum MechanicDto {
    ServiceRecovery {
        compensation_cents: i64,
    },
    EquipmentRepair {
        urgent: bool,
    },
    StaffRetention {
        raise_one_pct: u16,
        bonus_cents: i64,
    },
    GroupBooking {
        accept: bool,
        rooms: u16,
        nights: u16,
        discount_bp: u16,
        month: String,
    },
    CrisisCommunication {
        backup_cents: i64,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerDto {
    label: String,
    quality: u8,
    mechanics: Vec<MechanicDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SituationDto {
    code: String,
    month: String,
    title: String,
    answers: Vec<AnswerDto>,
}

fn read<T: for<'de> Deserialize<'de>>(dir: &Path, name: &str) -> Result<T, ContentError> {
    let path = dir.join(name);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| ContentError::Io(path.clone(), e.to_string()))?;
    serde_json::from_str(&text).map_err(|e| ContentError::Json(path, e.to_string()))
}

fn invalid<T>(msg: String) -> Result<T, ContentError> {
    Err(ContentError::Invalid(msg))
}

fn category(code: &str, ctx: &str) -> Result<Category, ContentError> {
    match Category::from_code(code) {
        Some(c) => Ok(c),
        None => invalid(format!("{ctx}: nepoznata kategorija '{code}'")),
    }
}

fn profile(code: &str, ctx: &str) -> Result<Profile, ContentError> {
    match Profile::from_code(code) {
        Some(p) => Ok(p),
        None => invalid(format!("{ctx}: nepoznat profil '{code}'")),
    }
}

fn month(code: &str, ctx: &str) -> Result<Month, ContentError> {
    match Month::from_code(code) {
        Some(m) => Ok(m),
        None => invalid(format!("{ctx}: nepoznat mjesec '{code}'")),
    }
}

fn level(d: LevelDto) -> LevelEffect {
    LevelEffect {
        cost_cents: d.cost_cents,
        equipment_pts: d.equipment_pts,
    }
}

fn training(d: TrainingDto) -> TrainingEffect {
    TrainingEffect {
        cost_cents: d.cost_cents,
        experience_pts: d.experience_pts,
        morale_pts: d.morale_pts,
    }
}

fn build_catalog(
    c: CatalogDto,
    profiles: BTreeMap<String, ProfileDto>,
) -> Result<Catalog, ContentError> {
    let mut categories = BTreeMap::new();
    for (k, d) in c.categories {
        categories.insert(
            category(&k, "categories")?,
            CategorySpec {
                price_ref_cents: d.price_ref_cents.table(),
                base_occupancy_bp: d.base_occupancy_bp.table(),
                seasonal_plan: d.seasonal_plan.table(),
                elasticity_bp: d.elasticity_bp,
                wage_cents: d.wage_cents,
                cpor_cents: d.cpor_cents,
            },
        );
    }
    let mut reference_profiles = BTreeMap::new();
    for (k, v) in c.reference_profiles {
        reference_profiles.insert(
            category(&k, "reference_profiles")?,
            profile(&v, "reference_profiles")?,
        );
    }
    let mut out_profiles = BTreeMap::new();
    for (k, d) in profiles {
        out_profiles.insert(
            profile(&k, "profiles")?,
            ProfileSpec {
                category: category(&d.category, &format!("profiles.{k}.category"))?,
                rooms: d.rooms,
                permanent: d.permanent,
                seasonal: d.seasonal,
                wage_pct: d.wage_pct,
                morale_pts: d.morale_pts,
                experience_pts: d.experience_pts,
                equipment_pts: d.equipment_pts,
                satisfaction_pts: d.satisfaction_pts,
                rating: d.rating,
                cash_cents: d.cash_cents,
                fixed_cents: d.fixed_cents,
                load_hundredths: d.load_hundredths,
            },
        );
    }
    let (e, a, m) = (c.economy, c.actions, c.mechanics);
    Ok(Catalog {
        categories,
        economy: Economy {
            other_revenue_bp: e.other_revenue_bp,
            hire_cents: e.hire_cents,
            interest_bp: e.interest_bp,
            marketing_half_cents: e.marketing_half_cents,
            marketing_max_bp: e.marketing_max_bp,
            goppar_floor_cents: e.goppar_floor_cents,
        },
        actions: ActionSpec {
            maintenance_deferred: level(a.maintenance_deferred),
            maintenance_regular: level(a.maintenance_regular),
            maintenance_intensive: level(a.maintenance_intensive),
            training_basic: training(a.training_basic),
            training_full: training(a.training_full),
            renovation_cents: a.renovation_cents,
            renovation_equipment_pts: a.renovation_equipment_pts,
            renovation_satisfaction_pts: a.renovation_satisfaction_pts,
            reply_cents: a.reply_cents,
            reply_rating: a.reply_rating,
            price_min_pct: a.price_min_pct,
            price_max_pct: a.price_max_pct,
            marketing_max_cents: a.marketing_max_cents,
            seasonal_max: a.seasonal_max,
            seasonal_step: a.seasonal_step,
            wage_min_pct: a.wage_min_pct,
            wage_max_pct: a.wage_max_pct,
            release_morale_pts: a.release_morale_pts,
        },
        mechanics: MechanicSpec {
            recovery_pts_per_q: m.recovery_pts_per_q,
            retention_pts_per_q: m.retention_pts_per_q,
            crisis_pts_per_q: m.crisis_pts_per_q,
            repair_urgent_cents: m.repair_urgent_cents,
            repair_temporary_cents: m.repair_temporary_cents,
            repair_temporary_equipment_pts: m.repair_temporary_equipment_pts,
            satisfaction_bound_pts: m.satisfaction_bound_pts,
            morale_bound_pts: m.morale_bound_pts,
            rating_bound: m.rating_bound,
            group_good_quality_cut_bp: m.group_good_quality_cut_bp,
        },
        profiles: out_profiles,
        reference_profiles,
    })
}

fn build_mechanic(d: MechanicDto, ctx: &str) -> Result<Mechanic, ContentError> {
    Ok(match d {
        MechanicDto::ServiceRecovery { compensation_cents } => {
            Mechanic::ServiceRecovery { compensation_cents }
        }
        MechanicDto::EquipmentRepair { urgent } => Mechanic::EquipmentRepair { urgent },
        MechanicDto::StaffRetention {
            raise_one_pct,
            bonus_cents,
        } => Mechanic::StaffRetention {
            raise_one_pct,
            bonus_cents,
        },
        MechanicDto::GroupBooking {
            accept,
            rooms,
            nights,
            discount_bp,
            month: m,
        } => {
            if discount_bp > 10_000 {
                return invalid(format!("{ctx}: discount_bp {discount_bp} je veći od 10000"));
            }
            Mechanic::GroupBooking {
                accept,
                rooms,
                nights,
                discount_bp,
                month: month(&m, ctx)?,
            }
        }
        MechanicDto::CrisisCommunication { backup_cents } => {
            Mechanic::CrisisCommunication { backup_cents }
        }
    })
}

fn build_situations(list: Vec<SituationDto>) -> Result<Vec<SituationTemplate>, ContentError> {
    let mut out = Vec::new();
    for s in list {
        let ctx = format!("situacija {}", s.code);
        let mut answers = Vec::new();
        for a in s.answers {
            if !(1..=5).contains(&a.quality) {
                return invalid(format!("{ctx}: kvaliteta {} izvan 1..=5", a.quality));
            }
            if a.mechanics.is_empty() {
                return invalid(format!("{ctx}: odgovor '{}' nema mehanika", a.label));
            }
            let mechanics = a
                .mechanics
                .into_iter()
                .map(|m| build_mechanic(m, &ctx))
                .collect::<Result<Vec<_>, _>>()?;
            answers.push(OfferedAnswer {
                label: a.label,
                mechanics,
                quality: a.quality,
            });
        }
        out.push(SituationTemplate {
            month: month(&s.month, &ctx)?,
            code: s.code,
            title: s.title,
            answers,
        });
    }
    out.sort_by_key(|s| s.month); // stabilno: unutar mjeseca ostaje redoslijed iz datoteke
    Ok(out)
}

/// Učita `catalog.json`, `profiles.json` i `situations.json` iz `dir` i provjeri ih.
pub fn load(dir: &Path) -> Result<Content, ContentError> {
    let catalog_dto: CatalogDto = read(dir, "catalog.json")?;
    let profiles: BTreeMap<String, ProfileDto> = read(dir, "profiles.json")?;
    let situations: Vec<SituationDto> = read(dir, "situations.json")?;
    let catalog = build_catalog(catalog_dto, profiles)?;
    let situations = build_situations(situations)?;
    catalog
        .validate()
        .map_err(|e| ContentError::Invalid(format!("katalog: {e:?}")))?;
    Ok(Content {
        catalog,
        situations,
    })
}
