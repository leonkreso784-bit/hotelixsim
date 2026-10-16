//! Učitavanje kataloga, profila i predložaka situacija iz `content/hotel/*.json` (ARHITEKTURA §9.5).
//! DTO-i su ovdje (serde), jezgra `sim_hotel` ostaje bez serdea.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sim_hotel::{
    ActionSpec, Catalog, Category, CategorySpec, Economy, LevelEffect, Mechanic, MechanicSpec,
    Month, MonthTable, OfferedAnswer, Profile, ProfileSpec, SituationTemplate, TrainingEffect,
};

pub mod templates;
mod templates_check;
mod templates_core;
mod templates_instance;
mod templates_principles;
mod templates_principles_debt;
pub use templates_instance::Instance;
pub use templates_principles::{Principle, Violation, check_principles_with, principle_violations};
pub use templates_principles_debt::KNOWN_VIOLATIONS;
pub mod ljudi;
mod ljudi_check;
mod ljudi_check_okvir;
mod ljudi_imena;
mod ljudi_raspodjele;
mod ljudi_tekst;
mod v2;
mod v2_dept;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    Io(PathBuf, String),
    Json(PathBuf, String),
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub catalog: Catalog,
    pub situations: Vec<SituationTemplate>,
    /// Po šifri situacije; svaka situacija ima unos (prazan ako nema AI-ja).
    pub ai: BTreeMap<String, SituationAi>,
    /// Katalog v2 (`Some` samo za snimak s `model = v2`); `base` je isti `catalog`.
    pub v2: Option<sim_hotel_v2::Catalog>,
    /// Predlošci situacija v2 (`Some` samo za v2 snimak koji ih nosi).
    pub situation_templates: Option<templates::SituationTemplates>,
}

/// Kojim se modelom igra računa. Snimak bez ključa `model` je `v1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    #[default]
    V1,
    V2,
}

fn is_v1(m: &Model) -> bool {
    *m == Model::V1
}

/// Lik s kojim student razgovara u situaciji sa slobodnim odgovorom (DEMO §7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    /// Tko je lik, jednom rečenicom za studenta (npr. „gošća u sobi 214, s dvoje djece").
    pub role: String,
    /// Upute za AI kako lik razmišlja i što skriva (studentu se ne prikazuje).
    pub brief: String,
    /// Prva rečenica lika, napisana (bez AI-ja).
    pub opening: String,
    /// Odgovor lika kad AI ne radi (rezerva, ARHITEKTURA §8.4).
    pub fallback_reply: String,
}

/// Sadržaj situacije koji treba AI sloju; jezgra `sim_hotel` ga ne poznaje.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SituationAi {
    /// Pet kriterija rubrike (DEMO §7) ili prazno.
    pub rubric: Vec<String>,
    pub character: Option<Character>,
    /// Smije li student odgovoriti svojim riječima (Leonova odluka 1: S1 i S2).
    pub free_text: bool,
    /// Za svaki ponuđeni odgovor (istim redom) brojevi ispunjenih kriterija 1..=5.
    pub criteria: Vec<Vec<u8>>,
}

/// Direktorij sadržaja u repou (`content/hotel`).
pub fn repo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/hotel")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoryDto {
    price_ref_cents: MonthsDto<i64>,
    base_occupancy_bp: MonthsDto<u16>,
    seasonal_plan: MonthsDto<u16>,
    elasticity_bp: i32,
    wage_cents: i64,
    cpor_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomyDto {
    other_revenue_bp: u16,
    hire_cents: i64,
    interest_bp: u16,
    marketing_half_cents: i64,
    marketing_max_bp: u16,
    goppar_floor_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelDto {
    cost_cents: i64,
    equipment_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainingDto {
    cost_cents: i64,
    experience_pts: i32,
    morale_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogDto {
    categories: BTreeMap<String, CategoryDto>,
    reference_profiles: BTreeMap<String, String>,
    economy: EconomyDto,
    actions: ActionsDto,
    mechanics: MechanicsDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerDto {
    label: String,
    quality: u8,
    mechanics: Vec<MechanicDto>,
    /// Ispunjeni kriteriji rubrike. `skip_serializing_if`: snimak iz koraka 3 mora dati bajt-isti stari zapis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    criteria: Vec<u8>,
}

pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterDto {
    name: String,
    role: String,
    brief: String,
    opening: String,
    fallback_reply: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SituationDto {
    code: String,
    month: String,
    title: String,
    unanswered: Vec<MechanicDto>,
    answers: Vec<AnswerDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    rubric: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    character: Option<CharacterDto>,
    #[serde(default, skip_serializing_if = "is_false")]
    free_text: bool,
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

/// Rubrika, lik i slobodan tekst jedne situacije; svaka greška imenuje situaciju.
fn build_ai(
    ctx: &str,
    rubric: Vec<String>,
    character: Option<CharacterDto>,
    free_text: bool,
    criteria: Vec<Vec<u8>>,
    qualities: &[u8],
) -> Result<SituationAi, ContentError> {
    if !rubric.is_empty() && (rubric.len() != 5 || rubric.iter().any(|r| r.trim().is_empty())) {
        return invalid(format!(
            "{ctx}: rubrika mora imati točno 5 nepraznih kriterija"
        ));
    }
    let character = match character {
        Some(c) => {
            let fields = [&c.name, &c.role, &c.brief, &c.opening, &c.fallback_reply];
            if fields.iter().any(|f| f.trim().is_empty()) {
                return invalid(format!("{ctx}: lik ima prazno polje"));
            }
            Some(Character {
                name: c.name,
                role: c.role,
                brief: c.brief,
                opening: c.opening,
                fallback_reply: c.fallback_reply,
            })
        }
        None => None,
    };
    if free_text && (rubric.is_empty() || character.is_none()) {
        return invalid(format!("{ctx}: slobodan tekst traži rubriku i lika"));
    }
    let mut sorted = Vec::new();
    for (i, mut crit) in criteria.into_iter().enumerate() {
        if let Some(bad) = crit.iter().find(|n| !(1..=5).contains(*n)) {
            return invalid(format!("{ctx}: kriterij {bad} izvan 1..=5"));
        }
        crit.sort_unstable();
        if crit.windows(2).any(|w| w[0] == w[1]) {
            return invalid(format!("{ctx}: kriterij se ponavlja u istom odgovoru"));
        }
        if rubric.is_empty() {
            if !crit.is_empty() {
                return invalid(format!("{ctx}: kriteriji bez rubrike"));
            }
        } else {
            let expected = u8::try_from(crit.len()).unwrap_or(u8::MAX).max(1);
            if qualities[i] != expected {
                return invalid(format!(
                    "{ctx}: kvaliteta {} ne odgovara broju kriterija {}",
                    qualities[i],
                    crit.len()
                ));
            }
        }
        sorted.push(crit);
    }
    Ok(SituationAi {
        rubric,
        character,
        free_text,
        criteria: sorted,
    })
}

fn build_situations(
    list: Vec<SituationDto>,
) -> Result<(Vec<SituationTemplate>, BTreeMap<String, SituationAi>), ContentError> {
    let mut out = Vec::new();
    let mut ai = BTreeMap::new();
    for s in list {
        let ctx = format!("situacija {}", s.code);
        if out.iter().any(|t: &SituationTemplate| t.code == s.code) {
            return invalid(format!("{ctx}: šifra se ponavlja"));
        }
        if s.answers.is_empty() {
            return invalid(format!("{ctx}: nema ponuđenih odgovora"));
        }
        let mut answers = Vec::new();
        let mut criteria = Vec::new();
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
            criteria.push(a.criteria);
            answers.push(OfferedAnswer {
                label: a.label,
                mechanics,
                quality: a.quality,
            });
        }
        if s.unanswered.is_empty() {
            return invalid(format!("{ctx}: nema ishoda neodgovorene situacije"));
        }
        let unanswered = s
            .unanswered
            .into_iter()
            .map(|m| build_mechanic(m, &ctx))
            .collect::<Result<Vec<_>, _>>()?;
        let qualities: Vec<u8> = answers.iter().map(|a| a.quality).collect();
        ai.insert(
            s.code.clone(),
            build_ai(
                &ctx,
                s.rubric,
                s.character,
                s.free_text,
                criteria,
                &qualities,
            )?,
        );
        out.push(SituationTemplate {
            month: month(&s.month, &ctx)?,
            code: s.code,
            title: s.title,
            answers,
            unanswered,
        });
    }
    out.sort_by_key(|s| s.month); // stabilno: unutar mjeseca ostaje redoslijed iz datoteke
    Ok((out, ai))
}

/// Puni snimak pravila (ARHITEKTURA §7.2): stablo DTO-a iz `content/hotel/*.json`. Ulazi cijeli u `GameCreated`,
/// a obračun igre gradi katalog iz njega, nikad iz trenutnih datoteka.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    catalog: CatalogDto,
    profiles: BTreeMap<String, ProfileDto>,
    situations: Vec<SituationDto>,
    /// Izostavljen za v1, pa stari snimci daju isti zapis i otisak.
    #[serde(default, skip_serializing_if = "is_v1")]
    model: Model,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    v2: Option<v2::V2Dto>,
}

impl Snapshot {
    /// Pročita `catalog.json`, `profiles.json` i `situations.json` iz `dir`.
    pub fn read(dir: &Path) -> Result<Snapshot, ContentError> {
        Ok(Snapshot {
            catalog: read(dir, "catalog.json")?,
            profiles: read(dir, "profiles.json")?,
            situations: read(dir, "situations.json")?,
            model: Model::V1,
            v2: None,
        })
    }

    /// v1 datoteke iz `dir` + `dir/v2/*.json`; `model = v2`.
    pub fn read_v2(dir: &Path) -> Result<Snapshot, ContentError> {
        let mut s = Snapshot::read(dir)?;
        s.model = Model::V2;
        s.v2 = Some(v2::read_v2_dir(dir)?);
        Ok(s)
    }

    /// `read_v2` + `build` + načela igre (`check_principles_with` uz `KNOWN_VIOLATIONS`). Samo za sadržaj s diska;
    /// snimak iz dnevnika (`from_value`) se ne provjerava — stara igra se uvijek čita.
    pub fn read_v2_checked(dir: &Path) -> Result<Snapshot, ContentError> {
        let s = Snapshot::read_v2(dir)?;
        check_principles_with(&s.build()?, KNOWN_VIOLATIONS)?;
        Ok(s)
    }

    pub fn model(&self) -> Model {
        self.model
    }

    /// v2 snimak „kao v1" za ekvivalencijske testove (MODEL_V2 §2 t. 6); odjeljak v2 = `sim_hotel_v2::degenerate`.
    /// Ako se v1 dio ne da izgraditi, odjeljak ostaje prazan, a `build()` javlja grešku v1 dijela.
    pub fn degenerate_v2(&self) -> Snapshot {
        let mut s = self.clone();
        s.model = Model::V2;
        let dto = match build_catalog(self.catalog.clone(), self.profiles.clone()) {
            Ok(base) => v2::degenerate_dto(&base),
            Err(_) => v2::V2Dto::default(),
        };
        s.v2 = Some(dto);
        s
    }

    /// Pretvori snimak u tipove jezgre i provjeri katalog.
    pub fn build(&self) -> Result<Content, ContentError> {
        let s = self.clone();
        let catalog = build_catalog(s.catalog, s.profiles)?;
        let (situations, ai) = build_situations(s.situations)?;
        catalog
            .validate()
            .map_err(|e| ContentError::Invalid(format!("katalog: {e:?}")))?;
        let (v2, situation_templates) = match (s.model, &s.v2) {
            (Model::V2, Some(dto)) => {
                let c = v2::build_v2(dto, &catalog)?;
                c.validate()
                    .map_err(|e| ContentError::Invalid(format!("katalog v2: {e:?}")))?;
                let t = v2::build_templates(dto, &c)?;
                (Some(c), t)
            }
            (Model::V1, None) => (None, None),
            _ => return invalid("model i odjeljak v2 se ne slažu".to_string()),
        };
        Ok(Content {
            catalog,
            situations,
            ai,
            v2,
            situation_templates,
        })
    }

    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self)
            .expect("snimak nema mapu s ne-tekstualnim ključem pa se uvijek serijalizira")
    }

    /// Snimak iz spremljenog `GameCreated`. Nova polja DTO-a imaju `#[serde(default)]`, pa se stari snimci čitaju.
    pub fn from_value(v: &serde_json::Value) -> Result<Snapshot, ContentError> {
        serde_json::from_value(v.clone())
            .map_err(|e| ContentError::Json(PathBuf::from("<snimak iz dnevnika>"), e.to_string()))
    }

    /// Otisak pravila (`RulesetId`) nad kanonskim `Value` snimka (ADR-015 t. 3, ispravak iz koraka 4).
    pub fn fingerprint(&self) -> String {
        fingerprint_value(&self.to_value())
    }

    /// Otisak iz koraka 3: nad tipiziranim zapisom, redoslijedom polja iz deklaracije. Vrijedi samo za igre stvorene
    /// prije koraka 4; nova polja DTO-a zato imaju `skip_serializing_if`, da stari snimak da isti stari zapis.
    pub fn legacy_fingerprint(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("snimak se uvijek serijalizira");
        format!("sha256:{:x}", Sha256::digest(&bytes))
    }
}

/// Otisak proizvoljnog JSON-a: SHA-256 kanonskog zapisa. `serde_json::Value` drži objekte u `BTreeMap` (bez značajke
/// `preserve_order`), pa su ključevi poredani i otisak ne ovisi o redoslijedu iz datoteke ni o JSONB-u.
pub fn fingerprint_value(v: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(v).expect("Value se uvijek serijalizira");
    format!("sha256:{:x}", Sha256::digest(&bytes))
}

/// Učita `catalog.json`, `profiles.json` i `situations.json` iz `dir` i provjeri ih.
pub fn load(dir: &Path) -> Result<Content, ContentError> {
    Snapshot::read(dir)?.build()
}
