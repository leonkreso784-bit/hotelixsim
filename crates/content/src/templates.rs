//! Predlošci situacija modela v2 (`content/hotel/v2/situation_templates.json`) i imena likova (`v2/names.json`).
//!
//! Predložak opisuje jednu vrstu situacije s mjestima za zamjenu (`{soba}`, `{gost.ime}`, `{odjel}`); igra iz njega
//! složi primjerak za jedan hotel u jednom krugu (generator nije ovdje). Ovdje su samo format i provjera
//! (`templates_check.rs`). Tipovi su ujedno zapis u snimku pravila, pa se polja samo dodaju (`serde(default)`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Predlošci i imena iz snimka v2, provjereni (`Content::situation_templates`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationTemplates {
    pub major: MajorRule,
    pub templates: Vec<Template>,
    pub names: Names,
}

/// Sadržaj `situation_templates.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateFile {
    pub major: MajorRule,
    pub templates: Vec<Template>,
}

/// Kad dolazi „veća situacija" (Leon 13.10.): prosjek pet pokazatelja (0–100) na početku kruga je barem
/// `high_avg_min` ili najviše `low_avg_max`; takvih je najviše `max_per_month` u kalendarskom mjesecu.
/// Pet pokazatelja su indeksi jezgre `sim_hotel::Index` (`crates/sim-hotel/src/index.rs:4-10`): Gosti (G),
/// Recenzije (R), Zaposlenici (Z), Poslovanje (O) i Financije (F); u v2 ih računa `period_indices`
/// (`crates/sim-hotel-v2/src/indices.rs:112`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MajorRule {
    pub high_avg_min: u8,
    pub low_avg_max: u8,
    pub max_per_month: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    /// Stabilna šifra, `P-` pa velika slova, brojke i crtice; primjerak dobiva nastavak kruga (`P-KLIMA.t03`).
    pub code: String,
    pub kind: Kind,
    /// Šifra odjela iz `v2/departments.json`, `hotel` (cijeli hotel) ili skupina `*staffed` / `*guest_facing`
    /// iz koje igra izabere jedan odjel.
    pub department: String,
    /// Važnost 1–4, **4 = najvažnije**: na ekranu se situacije redaju od veće važnosti prema manjoj (4 prva), a veća
    /// važnost ima prednost u izvlačenju.
    pub weight: u8,
    #[serde(default, skip_serializing_if = "Scope::is_hotel")]
    pub scope: Scope,
    /// Veća situacija za izvrstan (`high`) ili vrlo loš (`low`) hotel; bez polja je obična.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub major: Option<MajorSide>,
    #[serde(default, skip_serializing_if = "Conditions::is_empty")]
    pub when: Conditions,
    pub chance: Chance,
    pub repeat: Repeat,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub roles: BTreeMap<String, Role>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, Param>,
    pub title: String,
    pub description: String,
    /// Lik za AI razgovor; s likom je situacija razgovorna i student smije odgovoriti svojim riječima (Leon P6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<TemplateCharacter>,
    /// Pet kriterija presude; kvaliteta ponuđenog odgovora = broj ispunjenih (1–5).
    pub rubric: Vec<String>,
    pub answers: Vec<TemplateAnswer>,
    /// Mehanike kad nitko ne odgovori (kvaliteta 1) i kad student situaciju prepusti voditelju odjela (kvaliteta 2).
    pub unanswered: Vec<TemplateMechanic>,
    pub effect_band: EffectBand,
}

/// Vrsta situacije (raznolikost u krugu).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    #[serde(rename = "prituzba")]
    Complaint,
    #[serde(rename = "osoblje")]
    Staff,
    #[serde(rename = "prilika")]
    Opportunity,
    #[serde(rename = "kvar")]
    Breakdown,
    #[serde(rename = "kriza")]
    Crisis,
    #[serde(rename = "zahtjev")]
    Request,
}

/// `hotel`: svaki hotel dobiva svoj primjerak; `svi`: isti primjerak za sve hotele igre (kolektivna, kao S4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    #[default]
    #[serde(rename = "hotel")]
    Hotel,
    #[serde(rename = "svi")]
    All,
}

impl Scope {
    fn is_hotel(&self) -> bool {
        *self == Scope::Hotel
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MajorSide {
    High,
    Low,
}

/// Uvjeti nad stanjem hotela na početku kruga; svi navedeni moraju vrijediti. Polja `dept_*` gledaju odjel
/// situacije, pa traže predložak s odjelom.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conditions {
    /// Mjeseci (`jun`…`sep`) u kojima krug počinje; prazno = cijela sezona.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub months: Vec<String>,
    /// Popunjenost prošlog kruga, bazni bodovi (10 000 = 100 %).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupancy_bp: Option<Bounds>,
    /// Moral hotela, bodovi 0–10 000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub morale_pts: Option<Bounds>,
    /// Blagajna, centi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cash_cents: Option<Bounds>,
    /// Hotel ima prihvaćenu grupu koja još nije došla (`true`) ili nema (`false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_accepted: Option<bool>,
    /// Stanje prostora odjela, bodovi 0–10 000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_condition_pts: Option<Bounds>,
    /// Zadovoljstvo gostiju odjela, bodovi 0–10 000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_satisfaction_pts: Option<Bounds>,
    /// Opterećenje odjela prošlog kruga u stotinkama (kao `load_hundredths` u izvještaju odjela).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_load_hundredths: Option<Bounds>,
    /// Broj zaposlenih u odjelu (stalni + sezonci).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_staff: Option<Bounds>,
    /// Odlasci zaposlenih iz odjela od početka sezone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dept_departures: Option<Bounds>,
}

impl Conditions {
    pub fn is_empty(&self) -> bool {
        *self == Conditions::default()
    }
}

/// Prag: vrijednost je barem `at_least` i/ili manja od `below`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at_least: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below: Option<i64>,
}

/// Težina u izvlačenju: `base` puta svako pojačanje čiji uvjeti vrijede.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chance {
    pub base: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boost: Vec<Boost>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boost {
    pub when: Conditions,
    pub times: u8,
}

/// Najviše pojava u sezoni i najmanji razmak u krugovima između dvije pojave istog predloška.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    pub max_per_season: u8,
    pub gap_rounds: u8,
}

/// Uloga lika koju igra popuni: ime iz `names.json` (izvor i rod), opis iz `types`. Mjesta: `{uloga.ime}` i, kad
/// ima `types`, `{uloga.tip}`. Rod je zadan jer je tekst za AI pisan u rodu lika („Umorna si i ljuta").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub from: RoleSource,
    pub gender: Gender,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleSource {
    /// Gost hotela.
    Guest,
    /// Zaposlenik odjela situacije (u sloju 2 stvarna osoba iz odjela); traži predložak s odjelom.
    Staff,
    /// Vanjski partner (agencija, dobavljač).
    Partner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    F,
    M,
}

/// Mjesto za zamjenu (`{ime}` u tekstu, ime u mehanici).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Param {
    /// Cijeli broj iz raspona `[od, do]` (uključivo).
    Range([i64; 2]),
    /// Jedan od navedenih brojeva.
    Pick(Vec<i64>),
    /// Iznos: postotak cijene noćenja mjeseca kruga za kategoriju hotela.
    PricePct(u16),
    /// Iznos: postotak mjesečne plaće jednog zaposlenog za kategoriju hotela (ista osnovica kao povišica).
    WagePct(u16),
    /// Tekst po odjelu situacije; mora pokriti točno odjele koje predložak može dobiti.
    ByDepartment(BTreeMap<String, String>),
}

impl Param {
    /// Mogući brojevi (`Range` samo krajevi); `None` za iznose i tekst.
    pub(crate) fn numbers(&self) -> Option<Vec<i64>> {
        match self {
            Param::Range(r) => Some(r.to_vec()),
            Param::Pick(v) => Some(v.clone()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateCharacter {
    /// Ključ iz `roles`; ime lika je ime te uloge.
    pub role: String,
    pub brief: String,
    pub opening: String,
    pub fallback_reply: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateAnswer {
    pub label: String,
    /// Ispunjeni kriteriji rubrike (1–5).
    pub criteria: Vec<u8>,
    pub mechanics: Vec<TemplateMechanic>,
}

impl TemplateAnswer {
    /// Kvaliteta = broj ispunjenih kriterija, najmanje 1 (isto pravilo kao `situations.json`).
    pub fn quality(&self) -> u8 {
        u8::try_from(self.criteria.len())
            .unwrap_or(u8::MAX)
            .clamp(1, 5)
    }
}

/// Mehanike jezgre (istih pet kao `situations.json`); brojevi su zadani ili mjesto iz `params`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum TemplateMechanic {
    ServiceRecovery {
        compensation: Quantity,
    },
    EquipmentRepair {
        urgent: bool,
    },
    StaffRetention {
        raise_one_pct: Quantity,
        bonus: Quantity,
    },
    GroupBooking {
        accept: bool,
        rooms: Quantity,
        nights: Quantity,
        discount_bp: Quantity,
        month: String,
    },
    CrisisCommunication {
        backup: Quantity,
    },
}

/// Broj u mehanici: zadan (`0`, `8`) ili ime mjesta iz `params` (`"vecera"`). Novčani iznos smije biti samo `0`
/// ili mjesto `price_pct`/`wage_pct`, da vrijedi za svaku kategoriju hotela.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Quantity {
    Fixed(i64),
    Param(String),
}

/// Dopušteni raspon učinka; provjera odbija predložak čiji odgovor izlazi iz njega.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectBand {
    /// Jednokratni trošak odgovora (kompenzacija, bonus, rezerva) u postotku cijene noćenja, `[od, do]`, za svaku
    /// kategoriju i svaki mjesec u kojem se predložak može pojaviti.
    pub cost_night_pct: [u16; 2],
    /// Povišica u postocima, `[od, do]`; bez polja povišica mora biti 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raise_pct: Option<[u16; 2]>,
}

/// Sadržaj `names.json`: imena (ime i prezime) po izvoru uloge i rodu. Ime ne nosi nacionalnost; ako je kasnije
/// bude, smije odrediti samo činjenice (jezik), nikad osobnost (ADR-028).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Names {
    pub guest: NameList,
    pub staff: NameList,
    pub partner: NameList,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameList {
    pub f: Vec<String>,
    pub m: Vec<String>,
}
