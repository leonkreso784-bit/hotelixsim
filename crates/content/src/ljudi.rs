//! Banka ljudi ([BANKA_LJUDI](../../../docs/architecture/BANKA_LJUDI.md), ADR-042 t. 8): format zapisa RADNIK i GOST,
//! okvir (`mjesta.json`, `zabranjeno.json`, `raspodjele.json` i partneri iz `v2/names.json`) i čitanje serija iz
//! `content/hotel/v2/ljudi/{radnici,gosti}/serija-NNN.json`.
//!
//! Banka čuva **tko je osoba**; brojke (vještine, osobine, plaća) izvlači jezgra iz istih raspodjela za svakoga (§1.1),
//! zato zapis nema ta polja, a `deny_unknown_fields` ih ne pušta. Igra banku još ne čita (K5). Pravila provjere su u
//! `ljudi_check` (zapis, jedinstvenost), `ljudi_check_okvir` (okvir) i `ljudi_raspodjele` (stereotip, izvještaj, prekretnica).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::ContentError;
use crate::templates::Names;

pub use crate::ljudi_raspodjele::{izvjestaj, provjeri_prekretnicu};

// ---------------------------------------------------------------------------------------------------------------
// Kodovi
// ---------------------------------------------------------------------------------------------------------------

/// Rod lika; tekst za AI i opis pišu se u rodu lika (§3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rod {
    Z,
    M,
}

/// Struka radnika (MODEL_V2 §3.3). Voditelj nije uloga: postaje ga stručnjak struke (§1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Uloga {
    Recepcioner,
    Sobar,
    Kuhar,
    Konobar,
    Sanker,
    Terapeut,
    Tehnicar,
    ReferentUprave,
    ReferentProdaje,
}

/// Jedini ulaz banke u brojke: iz razine jezgra izvlači vještine (ADR-042 t. 8, pitanje 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Razina {
    Pocetnik,
    Iskusan,
    Strucnjak,
}

/// `lokalno` = Kvarner i Istra, `regija` = ostatak Hrvatske, `inozemstvo` = sve ostalo (§2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Podrijetlo {
    Lokalno,
    Regija,
    Inozemstvo,
}

/// Segment gosta (MODEL_V2 §3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Segment {
    Par,
    Obitelj,
    Poslovni,
    Grupa,
    Wellness,
}

/// Vrsta zapisa u seriji; mora odgovarati mapi (`radnici/`, `gosti/`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vrsta {
    Radnici,
    Gosti,
}

impl Rod {
    pub const SVE: [Rod; 2] = [Rod::Z, Rod::M];
    pub fn kod(self) -> &'static str {
        match self {
            Rod::Z => "z",
            Rod::M => "m",
        }
    }
}

impl Uloga {
    pub const SVE: [Uloga; 9] = [
        Uloga::Recepcioner,
        Uloga::Sobar,
        Uloga::Kuhar,
        Uloga::Konobar,
        Uloga::Sanker,
        Uloga::Terapeut,
        Uloga::Tehnicar,
        Uloga::ReferentUprave,
        Uloga::ReferentProdaje,
    ];
    pub fn kod(self) -> &'static str {
        match self {
            Uloga::Recepcioner => "recepcioner",
            Uloga::Sobar => "sobar",
            Uloga::Kuhar => "kuhar",
            Uloga::Konobar => "konobar",
            Uloga::Sanker => "sanker",
            Uloga::Terapeut => "terapeut",
            Uloga::Tehnicar => "tehnicar",
            Uloga::ReferentUprave => "referent_uprave",
            Uloga::ReferentProdaje => "referent_prodaje",
        }
    }
}

impl Razina {
    pub const SVE: [Razina; 3] = [Razina::Pocetnik, Razina::Iskusan, Razina::Strucnjak];
    pub fn kod(self) -> &'static str {
        match self {
            Razina::Pocetnik => "pocetnik",
            Razina::Iskusan => "iskusan",
            Razina::Strucnjak => "strucnjak",
        }
    }
}

impl Podrijetlo {
    pub const SVE: [Podrijetlo; 3] = [
        Podrijetlo::Lokalno,
        Podrijetlo::Regija,
        Podrijetlo::Inozemstvo,
    ];
    pub fn kod(self) -> &'static str {
        match self {
            Podrijetlo::Lokalno => "lokalno",
            Podrijetlo::Regija => "regija",
            Podrijetlo::Inozemstvo => "inozemstvo",
        }
    }
}

impl Segment {
    pub const SVE: [Segment; 5] = [
        Segment::Par,
        Segment::Obitelj,
        Segment::Poslovni,
        Segment::Grupa,
        Segment::Wellness,
    ];
    pub fn kod(self) -> &'static str {
        match self {
            Segment::Par => "par",
            Segment::Obitelj => "obitelj",
            Segment::Poslovni => "poslovni",
            Segment::Grupa => "grupa",
            Segment::Wellness => "wellness",
        }
    }
}

impl Vrsta {
    pub fn kod(self) -> &'static str {
        match self {
            Vrsta::Radnici => "radnici",
            Vrsta::Gosti => "gosti",
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Zapisi i serije
// ---------------------------------------------------------------------------------------------------------------

/// RADNIK (§1.2). `jezici`: (ISO 639-1, razina 1–4); jezik koji osoba ne govori se ne upisuje.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radnik {
    pub id: String,
    pub ime: String,
    pub prezime: String,
    pub rod: Rod,
    pub dob: u8,
    pub uloga: Uloga,
    pub razina: Razina,
    pub godine_iskustva: u8,
    pub podrijetlo: Podrijetlo,
    pub zemlja: String,
    pub mjesto: String,
    pub jezici: Vec<(String, u8)>,
    pub opis: Option<String>,
}

/// GOST (§1.3). `mjesto` je slobodan tekst (stvarni grad zemlje provjerava čovjek).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gost {
    pub id: String,
    pub ime: String,
    pub prezime: String,
    pub rod: Rod,
    pub dob: u8,
    pub zemlja: String,
    pub mjesto: String,
    pub jezici: Vec<(String, u8)>,
    pub segment: Segment,
    pub zanimanje: Option<String>,
    pub opis: Option<String>,
}

/// Jedna datoteka `serija-NNN.json`: najviše 50 zapisa s ID-jevima 50·N+1 … 50·N+50 (§4.1).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Serija<T> {
    pub format: u32,
    pub vrsta: Vrsta,
    pub serija: u32,
    pub od: String,
    #[serde(rename = "do")]
    pub do_: String,
    pub zapisi: Vec<T>,
}

// ---------------------------------------------------------------------------------------------------------------
// Okvir: mjesta, zabranjeno, raspodjele
// ---------------------------------------------------------------------------------------------------------------

/// `mjesta.json`: mjesta za lokalno i regiju, zemlje (materinji jezici, razumije li hrvatski) i dopušteni jezici.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mjesta {
    pub lokalno: Vec<String>,
    pub regija: Vec<String>,
    pub zemlje: BTreeMap<String, Zemlja>,
    pub jezici: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zemlja {
    pub naziv: String,
    /// Barem jedan od ovih jezika osoba iz te zemlje ima na razini 4.
    pub materinji: Vec<String>,
    /// BA, RS, ME: radnik razumije hrvatski na razini 4 (ADR-042 t. 8, odgovor 11).
    pub razumije_hr: bool,
}

/// `zabranjeno.json` (§3.5). Usporedba je po cijelim riječima, bez velikih slova i dijakritika.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zabranjeno {
    pub osobe: Vec<String>,
    pub hoteli: Vec<String>,
    pub marke: Vec<String>,
    pub rijeci_opisa: Vec<String>,
}

/// `raspodjele.json`: ciljevi cijele banke u baznim bodovima (zbroj svake dimenzije 10 000), §2.2 i §2.3.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Raspodjele {
    pub radnici: RaspodjeleRadnika,
    pub gosti: RaspodjeleGostiju,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaspodjeleRadnika {
    pub uloga: BTreeMap<Uloga, u32>,
    pub razina: BTreeMap<Razina, u32>,
    /// Dobni razredi `"18-24"` …, bez rupa od 18 do 65.
    pub dob: BTreeMap<String, u32>,
    pub podrijetlo: BTreeMap<Podrijetlo, u32>,
    /// Udio zemalja među radnicima iz inozemstva; `"ostalo"` skuplja zemlje koje nisu navedene.
    pub zemlje_inozemstvo: BTreeMap<String, u32>,
    pub zemlje_izvor: String,
    /// Po jeziku pet udjela za razine 0–4 (0 = ne govori).
    pub jezici: BTreeMap<String, [u32; 5]>,
    /// Udio žena po ulozi po stvarnom omjeru (ADR-042 t. 8, odgovor 5); izvor je obvezan.
    pub rod_po_ulozi: BTreeMap<Uloga, RodUloge>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RodUloge {
    pub z_bp: u32,
    pub izvor: String,
    pub potvrdeno: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaspodjeleGostiju {
    pub zemlja: BTreeMap<String, u32>,
    pub segment: BTreeMap<Segment, u32>,
    /// Dobni razredi bez rupa od 18 do 85.
    pub dob: BTreeMap<String, u32>,
    pub rod: BTreeMap<Rod, u32>,
}

/// Sve što provjera banke treba osim samih serija. `partneri` (puna imena iz `v2/names.json`) zauzimaju ime (§3.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Okvir {
    pub mjesta: Mjesta,
    pub zabranjeno: Zabranjeno,
    pub raspodjele: Raspodjele,
    pub partneri: Vec<String>,
}

impl Okvir {
    /// `dir` = [`crate::repo_dir`]; čita `v2/ljudi/{mjesta,zabranjeno,raspodjele}.json` i partnere iz `v2/names.json`.
    pub fn read(dir: &Path) -> Result<Okvir, ContentError> {
        let ljudi = dir.join("v2/ljudi");
        let names: Names = crate::read(&dir.join("v2"), "names.json")?;
        Ok(Okvir {
            mjesta: crate::read(&ljudi, "mjesta.json")?,
            zabranjeno: crate::read(&ljudi, "zabranjeno.json")?,
            raspodjele: crate::read(&ljudi, "raspodjele.json")?,
            partneri: names.partner.f.into_iter().chain(names.partner.m).collect(),
        })
    }

    /// Dosljednost okvira (R1–R3): zbrojevi 10 000, potpune dimenzije, izvor roda po ulozi, poznati jezici.
    pub fn provjeri(&self) -> Result<(), ContentError> {
        crate::ljudi_check_okvir::provjeri_okvir(self)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Banka
// ---------------------------------------------------------------------------------------------------------------

/// Sve serije, svaka s putem ispod `ljudi/` (`"radnici/serija-000.json"`), poredane po imenu datoteke.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banka {
    pub radnici: Vec<(String, Serija<Radnik>)>,
    pub gosti: Vec<(String, Serija<Gost>)>,
}

/// Izvještaj raspodjele: udio jedne vrijednosti dimenzije u banci prema cilju iz `raspodjele.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Odstupanje {
    pub dimenzija: String,
    pub kljuc: String,
    pub cilj_bp: u32,
    pub stvarno_bp: u32,
}

impl Banka {
    /// `dir` = [`crate::repo_dir`]; čita sve `serija-*.json` iz `v2/ljudi/radnici` i `v2/ljudi/gosti`.
    pub fn read(dir: &Path) -> Result<Banka, ContentError> {
        let ljudi = dir.join("v2/ljudi");
        Ok(Banka {
            radnici: read_serije(&ljudi, Vrsta::Radnici)?,
            gosti: read_serije(&ljudi, Vrsta::Gosti)?,
        })
    }

    /// Isto kao `read`, iz već pročitanih vrijednosti (put, JSON); za testove i alate.
    pub fn iz_vrijednosti(
        radnici: Vec<(String, Value)>,
        gosti: Vec<(String, Value)>,
    ) -> Result<Banka, ContentError> {
        Ok(Banka {
            radnici: iz_vrijednosti(radnici)?,
            gosti: iz_vrijednosti(gosti)?,
        })
    }

    /// Sva pravila zapisa (oblik iznad serdea, jedinstvenost, dosljednost, stereotip, tekst, zabranjeno). Vraća jedan
    /// `Invalid` s retkom `"{put} {id}: {polje}: {poruka}"` po prekršaju, poredano po putu i ID-u.
    pub fn provjeri(&self, okvir: &Okvir) -> Result<(), ContentError> {
        crate::ljudi_check::provjeri_banku(self, okvir)
    }
}

fn iz_vrijednosti<T: DeserializeOwned>(
    datoteke: Vec<(String, Value)>,
) -> Result<Vec<(String, Serija<T>)>, ContentError> {
    datoteke
        .into_iter()
        .map(|(put, v)| match serde_json::from_value(v) {
            Ok(s) => Ok((put, s)),
            Err(e) => Err(ContentError::Json(PathBuf::from(put), e.to_string())),
        })
        .collect()
}

fn read_serije<T: DeserializeOwned>(
    ljudi: &Path,
    vrsta: Vrsta,
) -> Result<Vec<(String, Serija<T>)>, ContentError> {
    let mapa = ljudi.join(vrsta.kod());
    let io = |e: std::io::Error| ContentError::Io(mapa.clone(), e.to_string());
    let mut imena = Vec::new();
    for e in std::fs::read_dir(&mapa).map_err(io)? {
        let ime = e.map_err(io)?.file_name().to_string_lossy().into_owned();
        if ime.starts_with('.') {
            continue;
        }
        if !(ime.starts_with("serija-") && ime.ends_with(".json")) {
            return Err(ContentError::Invalid(format!(
                "{}/{ime}: u mapi smiju biti samo serija-NNN.json",
                vrsta.kod()
            )));
        }
        imena.push(ime);
    }
    imena.sort();
    imena
        .into_iter()
        .map(|ime| {
            let s = crate::read(&mapa, &ime)?;
            Ok((format!("{}/{ime}", vrsta.kod()), s))
        })
        .collect()
}
