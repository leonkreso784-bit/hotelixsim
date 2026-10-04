//! Katalog: sve brojke koje balansiranje smije mijenjati (DEMO §2.1, P5). Učitava se iz
//! `content/hotel/*.json`; `validate()` odbija katalog koji bi srušio jezgru.

use std::collections::BTreeMap;

use crate::month::{Month, MonthTable};
use crate::profile::Profile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Three,
    Four,
    Five,
}

impl Category {
    pub const ALL: [Category; 3] = [Category::Three, Category::Four, Category::Five];

    pub fn code(self) -> &'static str {
        match self {
            Category::Three => "3",
            Category::Four => "4",
            Category::Five => "5",
        }
    }

    pub fn from_code(s: &str) -> Option<Category> {
        match s {
            "3" => Some(Category::Three),
            "4" => Some(Category::Four),
            "5" => Some(Category::Five),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategorySpec {
    pub price_ref_cents: MonthTable<i64>,
    pub base_occupancy_bp: MonthTable<u16>,
    pub seasonal_plan: MonthTable<u16>,
    pub elasticity_bp: i32,
    pub wage_cents: i64,
    pub cpor_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Economy {
    pub other_revenue_bp: u16,
    pub hire_cents: i64,
    pub interest_bp: u16,
    pub marketing_half_cents: i64,
    pub marketing_max_bp: u16,
    pub goppar_floor_cents: i64,
}

/// Učinak razine održavanja.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelEffect {
    pub cost_cents: i64,
    pub equipment_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingEffect {
    pub cost_cents: i64,
    pub experience_pts: i32,
    pub morale_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionSpec {
    pub maintenance_deferred: LevelEffect,
    pub maintenance_regular: LevelEffect,
    pub maintenance_intensive: LevelEffect,
    pub training_basic: TrainingEffect,
    pub training_full: TrainingEffect,
    pub renovation_cents: i64,
    pub renovation_equipment_pts: i32,
    pub renovation_satisfaction_pts: i32,
    pub reply_cents: i64,
    pub reply_rating: i32,
    pub price_min_pct: u16,
    pub price_max_pct: u16,
    pub marketing_max_cents: i64,
    pub seasonal_max: u16,
    pub seasonal_step: u16,
    pub wage_min_pct: u16,
    pub wage_max_pct: u16,
    /// Moral pri puštanju sezonaca (−300).
    pub release_morale_pts: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MechanicSpec {
    /// Oporavak, zadržavanje, kriza: 200, 300, 300 po tromjesečju.
    pub recovery_pts_per_q: i32,
    pub retention_pts_per_q: i32,
    pub crisis_pts_per_q: i32,
    pub repair_urgent_cents: i64,
    pub repair_temporary_cents: i64,
    pub repair_temporary_equipment_pts: i32,
    /// Granice učinka situacija: 800, 800, 15.
    pub satisfaction_bound_pts: i32,
    pub morale_bound_pts: i32,
    pub rating_bound: i32,
    /// Rez popusta za grupe uz dobru kvalitetu (500).
    pub group_good_quality_cut_bp: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileSpec {
    pub category: Category,
    pub rooms: u16,
    pub permanent: u16,
    pub seasonal: u16,
    pub wage_pct: u16,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub equipment_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub fixed_cents: i64,
    pub load_hundredths: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError {
    Missing(String),
    NotPositive(String),
    OutOfRange(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub categories: BTreeMap<Category, CategorySpec>,
    pub economy: Economy,
    pub actions: ActionSpec,
    pub mechanics: MechanicSpec,
    pub profiles: BTreeMap<Profile, ProfileSpec>,
    pub reference_profiles: BTreeMap<Category, Profile>,
}

fn positive(v: i64, field: &str) -> Result<(), CatalogError> {
    if v > 0 {
        Ok(())
    } else {
        Err(CatalogError::NotPositive(field.to_string()))
    }
}

fn within(v: i64, lo: i64, hi: i64, field: &str) -> Result<(), CatalogError> {
    if (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(CatalogError::OutOfRange(field.to_string()))
    }
}

impl Catalog {
    pub fn category(&self, c: Category) -> Result<&CategorySpec, CatalogError> {
        self.categories
            .get(&c)
            .ok_or_else(|| CatalogError::Missing(format!("categories.{}", c.code())))
    }

    pub fn profile(&self, p: Profile) -> Result<&ProfileSpec, CatalogError> {
        self.profiles
            .get(&p)
            .ok_or_else(|| CatalogError::Missing(format!("profiles.{}", p.code())))
    }

    /// Provjera P5: odbija katalog koji bi dao dijeljenje nulom, nepozitivne cijene ili skale izvan 0..=10 000.
    /// Iznimka je bazna popunjenost: smije do 20 000 bp, jer potražnja u vrhu sezone smije biti veća od broja
    /// soba (ADR-014 t. 1, kolovoz rasprodan); prodaju i dalje ograničavaju raspoložive sobe i osoblje.
    pub fn validate(&self) -> Result<(), CatalogError> {
        for (cat, spec) in &self.categories {
            let c = cat.code();
            for m in Month::SEASON {
                let mc = m.code();
                positive(
                    spec.price_ref_cents.get(m),
                    &format!("categories.{c}.price_ref_cents.{mc}"),
                )?;
                within(
                    i64::from(spec.base_occupancy_bp.get(m)),
                    1,
                    20_000,
                    &format!("categories.{c}.base_occupancy_bp.{mc}"),
                )?;
            }
            positive(spec.wage_cents, &format!("categories.{c}.wage_cents"))?;
            positive(spec.cpor_cents, &format!("categories.{c}.cpor_cents"))?;
        }
        let e = &self.economy;
        positive(e.marketing_half_cents, "economy.marketing_half_cents")?;
        positive(e.goppar_floor_cents, "economy.goppar_floor_cents")?;
        positive(e.hire_cents, "economy.hire_cents")?;
        let a = &self.actions;
        if a.price_min_pct >= a.price_max_pct {
            return Err(CatalogError::OutOfRange(
                "actions.price_min_pct".to_string(),
            ));
        }
        if a.wage_min_pct > 100 || a.wage_max_pct < 100 {
            return Err(CatalogError::OutOfRange("actions.wage_min_pct".to_string()));
        }
        for (p, s) in &self.profiles {
            let k = p.code();
            if !self.categories.contains_key(&s.category) {
                return Err(CatalogError::Missing(format!(
                    "profiles.{k}.category -> categories.{}",
                    s.category.code()
                )));
            }
            positive(i64::from(s.rooms), &format!("profiles.{k}.rooms"))?;
            positive(s.fixed_cents, &format!("profiles.{k}.fixed_cents"))?;
            for (v, name) in [
                (s.morale_pts, "morale_pts"),
                (s.experience_pts, "experience_pts"),
                (s.equipment_pts, "equipment_pts"),
                (s.satisfaction_pts, "satisfaction_pts"),
            ] {
                within(i64::from(v), 0, 10_000, &format!("profiles.{k}.{name}"))?;
            }
            within(
                i64::from(s.rating),
                100,
                1_000,
                &format!("profiles.{k}.rating"),
            )?;
            within(
                i64::from(s.wage_pct),
                i64::from(a.wage_min_pct),
                i64::from(a.wage_max_pct),
                &format!("profiles.{k}.wage_pct"),
            )?;
        }
        for s in self.profiles.values() {
            let c = s.category;
            let reference = self
                .reference_profiles
                .get(&c)
                .ok_or_else(|| CatalogError::Missing(format!("reference_profiles.{}", c.code())))?;
            let r = self.profile(*reference)?;
            if r.category != c {
                return Err(CatalogError::OutOfRange(format!(
                    "reference_profiles.{}",
                    c.code()
                )));
            }
        }
        Ok(())
    }
}
