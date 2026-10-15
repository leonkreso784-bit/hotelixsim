//! Jezgreni oblik predložaka situacija: samo ono što treba izvlačenju (uvjeti, pojačanja, ponavljanje, uloge,
//! brojevna mjesta). Tekstovi, rubrika i odgovori ostaju u sadržaju (`content::templates`), koji ovdje pretvara
//! provjerene predloške; jezgra ne ovisi o sadržaju.

use std::collections::BTreeMap;

use sim_hotel::Month;

use super::count::SituationCounts;
use crate::catalog::Catalog;
use crate::department::{DeptCode, UsaliClass};

/// Predlošci jedne igre (iz snimka pravila), pravilo veće situacije i imena likova.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateSet {
    pub major: MajorRule,
    pub templates: Vec<SituationSpec>,
    pub names: NamePool,
    /// Najviše primjeraka koji nude grupu u sezoni hotela, sidra uključena (Leon 13.10., J-3); `None` = bez granice.
    pub group_limit: Option<u8>,
    /// Najviše ponuda grupe s dolaskom u istom mjesecu (M4, J-3); `None` = bez granice.
    pub groups_per_month: Option<u8>,
    /// Najmanji razmak između dvije pojave istog predloška u tjednima (Leon): vrijedi i kad predložak traži manje.
    pub min_gap_weeks: u8,
    /// Prilika barem jednom u ovoliko uzastopnih tjedana (kad je koja dopuštena).
    pub opportunity_every_weeks: u8,
    /// Broj situacija po krugu, postavka igre; `None` = `SituationCounts::default_for` vrste kruga. Sadržaj ga ne
    /// puni (igra ga postavlja iz `GameCreated`).
    pub counts: Option<SituationCounts>,
}

impl TemplateSet {
    /// Ima li predložak `code` oznaku „samo nagrada"; nepoznat predložak nema.
    pub fn no_penalty(&self, code: &str) -> bool {
        self.templates
            .iter()
            .any(|t| t.code == code && t.no_penalty)
    }

    /// Prostor koji predložak `code` gleda i popravlja umjesto svog odjela (`SituationSpec::condition_of`).
    pub fn condition_of(&self, code: &str) -> Option<DeptCode> {
        self.templates
            .iter()
            .find(|t| t.code == code)
            .and_then(|t| t.condition_of.clone())
    }
}

/// Veća situacija (Leon 13.10.): prosjek pet pokazatelja na početku kruga ≥ `high_avg_min` (izvrstan hotel) ili
/// ≤ `low_avg_max` (vrlo loš), najviše `max_per_month` u kalendarskom mjesecu kruga.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MajorRule {
    pub high_avg_min: u8,
    pub low_avg_max: u8,
    pub max_per_month: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationSpec {
    pub code: String,
    pub kind: Kind,
    pub department: DeptTarget,
    /// Važnost 1–4 (4 = najvažnije): poredak na ekranu i množitelj težine u izvlačenju.
    pub weight: u8,
    pub scope: Scope,
    pub major: Option<MajorSide>,
    pub when: Conditions,
    pub chance: Chance,
    pub repeat: Repeat,
    pub roles: BTreeMap<String, RoleSpec>,
    /// Brojevna mjesta (`range`, `pick`); iznosi i tekst po odjelu ne traže kocku pa ih ovdje nema.
    pub numbers: BTreeMap<String, NumberParam>,
    /// Mjesec dolaska grupe koju predložak nudi (prvi odgovor koji prihvaća grupu); `None` = ne nudi grupu.
    /// Broji se u `TemplateSet::group_limit` i `groups_per_month`.
    pub group_month: Option<Month>,
    /// „Samo nagrada" (S-4): nijedan ishod primjerka ne snižava pokazatelje (`situations::reward_only`).
    pub no_penalty: bool,
    /// Prostor čije stanje čita `when.dept_condition_pts` (i pojačanja) i kamo ide popravak; `None` = odjel predloška.
    pub condition_of: Option<DeptCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Complaint,
    Staff,
    Opportunity,
    Breakdown,
    Crisis,
    Request,
}

/// `Hotel`: svaki hotel svoj primjerak; `All`: isti primjerak za sve hotele igre (kolektivna).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Hotel,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MajorSide {
    High,
    Low,
}

/// Odjel situacije: cijeli hotel, jedan odjel ili skupina iz koje izvlačenje izabere jedan otvoren odjel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeptTarget {
    Hotel,
    One(DeptCode),
    /// Odjeli s osobljem (`wage_cents > 0`).
    Staffed,
    /// Odjeli koje gosti vide: smještaj, hrana i piće, ostali operativni.
    GuestFacing,
}

impl DeptTarget {
    /// Odjeli kataloga koje predložak može dobiti, redom po kodu; `None` za cijeli hotel.
    pub fn members(&self, cat: &Catalog) -> Option<Vec<DeptCode>> {
        let pick = |f: &dyn Fn(UsaliClass, i64) -> bool| -> Vec<DeptCode> {
            cat.departments
                .iter()
                .filter(|(_, s)| f(s.class, s.wage_cents))
                .map(|(k, _)| k.clone())
                .collect()
        };
        match self {
            DeptTarget::Hotel => None,
            DeptTarget::One(code) => Some(vec![code.clone()]),
            DeptTarget::Staffed => Some(pick(&|_, wage| wage > 0)),
            DeptTarget::GuestFacing => Some(pick(&|class, _| {
                matches!(
                    class,
                    UsaliClass::Rooms | UsaliClass::FoodBeverage | UsaliClass::OtherOperated
                )
            })),
        }
    }
}

/// Uvjeti nad stanjem na početku kruga; svi navedeni moraju vrijediti (značenje polja: `content::templates`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Conditions {
    /// Mjeseci u kojima krug počinje; prazno = cijela sezona.
    pub months: Vec<Month>,
    pub occupancy_bp: Option<Bounds>,
    pub morale_pts: Option<Bounds>,
    pub cash_cents: Option<Bounds>,
    pub group_accepted: Option<bool>,
    pub dept_condition_pts: Option<Bounds>,
    pub dept_satisfaction_pts: Option<Bounds>,
    pub dept_load_hundredths: Option<Bounds>,
    pub dept_staff: Option<Bounds>,
    pub dept_departures: Option<Bounds>,
}

/// Prag: vrijednost je barem `at_least` i/ili manja od `below`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub at_least: Option<i64>,
    pub below: Option<i64>,
}

impl Bounds {
    pub fn contains(&self, v: i64) -> bool {
        self.at_least.is_none_or(|a| v >= a) && self.below.is_none_or(|b| v < b)
    }
}

/// Težina u izvlačenju: `base` puta `times` svakog pojačanja čiji uvjeti vrijede.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chance {
    pub base: u8,
    pub boost: Vec<Boost>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boost {
    pub when: Conditions,
    pub times: u8,
}

/// Najviše pojava u sezoni i najmanji razmak između dvije pojave, u tjednima sezone (u tjednoj igri = krugovi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Repeat {
    pub max_per_season: u8,
    pub gap_rounds: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleSpec {
    pub source: RoleSource,
    pub gender: Gender,
    /// Opisi lika („majka s dvoje male djece"); prazno = bez opisa.
    pub types: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleSource {
    Guest,
    Staff,
    Partner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    F,
    M,
}

/// Brojevno mjesto: cijeli broj iz `[od, do]` (uključivo) ili jedan od navedenih.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberParam {
    Range(i64, i64),
    Pick(Vec<i64>),
}

/// Imena likova po izvoru uloge i rodu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamePool {
    pub guest: NameList,
    pub staff: NameList,
    pub partner: NameList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameList {
    pub f: Vec<String>,
    pub m: Vec<String>,
}

impl NamePool {
    pub fn list(&self, source: RoleSource, gender: Gender) -> &[String] {
        let l = match source {
            RoleSource::Guest => &self.guest,
            RoleSource::Staff => &self.staff,
            RoleSource::Partner => &self.partner,
        };
        match gender {
            Gender::F => &l.f,
            Gender::M => &l.m,
        }
    }
}
