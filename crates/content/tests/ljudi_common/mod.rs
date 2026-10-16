//! Zajedničko za testove banke ljudi (K1a): valjani zapisi, okvir građen u testu (neovisan o repo podacima),
//! omatanje zapisa u serije i provjera „točno jedna greška". Loš primjer mijenja jedno polje valjanog zapisa.
#![allow(dead_code)]

use content::ContentError;
use content::ljudi::{Banka, Okvir};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------------------------------------------
// Valjani zapisi
// ---------------------------------------------------------------------------------------------------------------

pub fn radnik() -> Value {
    json!({
        "id": "R-00001", "ime": "Mirela", "prezime": "Knežević", "rod": "z", "dob": 34,
        "uloga": "recepcioner", "razina": "iskusan", "godine_iskustva": 8,
        "podrijetlo": "lokalno", "zemlja": "HR", "mjesto": "Kastav",
        "jezici": [["hr", 4], ["en", 3]]
    })
}

pub fn gost() -> Value {
    json!({
        "id": "G-00001", "ime": "Gerda", "prezime": "Huber", "rod": "z", "dob": 41,
        "zemlja": "AT", "mjesto": "Graz", "jezici": [["de", 4]], "segment": "par"
    })
}

/// Kopija zapisa s jednim promijenjenim (ili dodanim) poljem.
pub fn s(mut v: Value, polje: &str, vrijednost: Value) -> Value {
    v[polje] = vrijednost;
    v
}

/// Kopija zapisa bez jednog polja.
pub fn bez(mut v: Value, polje: &str) -> Value {
    v.as_object_mut().unwrap().remove(polje);
    v
}

// ---------------------------------------------------------------------------------------------------------------
// Okvir (mjesta, zabranjeno, raspodjele) — gradi se u testu
// ---------------------------------------------------------------------------------------------------------------

pub const ULOGE: [&str; 9] = [
    "sobar",
    "kuhar",
    "konobar",
    "recepcioner",
    "referent_uprave",
    "tehnicar",
    "sanker",
    "terapeut",
    "referent_prodaje",
];

pub fn mjesta_json() -> Value {
    json!({
        "lokalno": ["Kastav", "Opatija"],
        "regija": ["Osijek"],
        "zemlje": {
            "HR": {"naziv": "Hrvatska", "materinji": ["hr"], "razumije_hr": true},
            "AT": {"naziv": "Austrija", "materinji": ["de"], "razumije_hr": false},
            "DE": {"naziv": "Njemačka", "materinji": ["de"], "razumije_hr": false},
            "BA": {"naziv": "Bosna i Hercegovina", "materinji": ["hr"], "razumije_hr": true},
            "RS": {"naziv": "Srbija", "materinji": ["sr"], "razumije_hr": true},
            "ME": {"naziv": "Crna Gora", "materinji": ["sr"], "razumije_hr": true},
            "NP": {"naziv": "Nepal", "materinji": ["ne"], "razumije_hr": false}
        },
        "jezici": ["hr", "en", "de", "it", "sr", "ne"]
    })
}

/// Popis zabranjenog je ovdje vlastiti testni primjer; stvarna imena profesora nisu u testovima.
pub fn zabranjeno_json() -> Value {
    json!({
        "osobe": ["Leon Kreso"],
        "hoteli": ["Hotel Primjer"],
        "marke": ["Marka Primjer"],
        "rijeci_opisa": [
            "pouzdan", "pouzdana", "pouzdani", "ljubazan", "ljubazna", "marljiv", "marljiva", "svadljiv",
            "nestrpljiv", "neorganiziran", "lijen", "vrijedan", "vrijedna", "mentalitet", "narod",
            "tipičan", "tipična"
        ]
    })
}

pub fn raspodjele_json() -> Value {
    let rod: serde_json::Map<String, Value> = ULOGE
        .iter()
        .map(|u| {
            (
                u.to_string(),
                json!({"z_bp": 5000, "izvor": "Testni izvor (Eurostat LFS, ISCO-08 x spol)", "potvrdeno": false}),
            )
        })
        .collect();
    json!({
        "radnici": {
            "uloga": {"sobar": 2800, "kuhar": 1400, "konobar": 1400, "recepcioner": 1100,
                      "referent_uprave": 900, "tehnicar": 800, "sanker": 700, "terapeut": 500,
                      "referent_prodaje": 400},
            "razina": {"pocetnik": 3500, "iskusan": 4500, "strucnjak": 2000},
            "dob": {"18-24": 2200, "25-34": 3000, "35-49": 3200, "50-65": 1600},
            "podrijetlo": {"lokalno": 5000, "regija": 2500, "inozemstvo": 2500},
            "zemlje_inozemstvo": {"BA": 2000, "RS": 1500, "NP": 2500, "ostalo": 4000},
            "zemlje_izvor": "Testni izvor (MUP, dozvole za turizam)",
            "jezici": {"en": [1000, 2000, 3500, 3000, 500], "de": [4500, 2000, 2000, 1200, 300],
                       "it": [7000, 1500, 1000, 400, 100]},
            "rod_po_ulozi": Value::Object(rod)
        },
        "gosti": {
            "zemlja": {"AT": 3500, "DE": 3000, "HR": 2000, "BA": 500, "RS": 500, "ME": 500},
            "segment": {"par": 3500, "obitelj": 2500, "wellness": 1500, "poslovni": 1500, "grupa": 1000},
            "dob": {"18-29": 1500, "30-49": 4000, "50-64": 2700, "65-85": 1800},
            "rod": {"z": 5000, "m": 5000}
        }
    })
}

/// Okvir iz tri JSON dijela (da ga testovi R1–R3 mogu pokvariti prije čitanja).
pub fn okvir_iz(mjesta: Value, zabranjeno: Value, raspodjele: Value) -> Okvir {
    Okvir {
        mjesta: serde_json::from_value(mjesta).unwrap(),
        zabranjeno: serde_json::from_value(zabranjeno).unwrap(),
        raspodjele: serde_json::from_value(raspodjele).unwrap(),
        partneri: vec!["Marko Perić".to_string()],
    }
}

pub fn okvir() -> Okvir {
    okvir_iz(mjesta_json(), zabranjeno_json(), raspodjele_json())
}

// ---------------------------------------------------------------------------------------------------------------
// Serije i provjera
// ---------------------------------------------------------------------------------------------------------------

/// Zaglavlje serije N (`od`/`do` = 50·N+1 … 50·N+50) s danim zapisima.
pub fn serija(vrsta: &str, n: u32, zapisi: Vec<Value>) -> Value {
    let slovo = if vrsta == "radnici" { 'R' } else { 'G' };
    json!({
        "format": 1, "vrsta": vrsta, "serija": n,
        "od": format!("{slovo}-{:05}", 50 * n + 1), "do": format!("{slovo}-{:05}", 50 * n + 50),
        "zapisi": zapisi
    })
}

pub type Datoteke = Vec<(String, Value)>;

/// Zapisi u `radnici/serija-000.json` i `gosti/serija-000.json` (prazan popis = datoteke nema).
pub fn banka(r: Vec<Value>, g: Vec<Value>) -> (Datoteke, Datoteke) {
    let radnici = if r.is_empty() {
        vec![]
    } else {
        vec![(
            "radnici/serija-000.json".to_string(),
            serija("radnici", 0, r),
        )]
    };
    let gosti = if g.is_empty() {
        vec![]
    } else {
        vec![("gosti/serija-000.json".to_string(), serija("gosti", 0, g))]
    };
    (radnici, gosti)
}

/// Čitanje (serde + oblik) pa provjera pravila, kao `Banka::read` + `provjeri`.
pub fn provjeri_s(okvir: &Okvir, radnici: Datoteke, gosti: Datoteke) -> Result<(), ContentError> {
    Banka::iz_vrijednosti(radnici, gosti)?.provjeri(okvir)
}

pub fn provjeri(radnici: Datoteke, gosti: Datoteke) -> Result<(), ContentError> {
    provjeri_s(&okvir(), radnici, gosti)
}

/// Provjera zapisa u jednoj seriji svake vrste (kratica za većinu testova).
pub fn provjeri_zapise(r: Vec<Value>, g: Vec<Value>) -> Result<(), ContentError> {
    let (r, g) = banka(r, g);
    provjeri(r, g)
}

/// Retci greške: `Invalid` je jedan redak po prekršaju, `Json` je jedna greška čitanja.
pub fn retci(res: &Result<(), ContentError>) -> Vec<String> {
    match res {
        Ok(()) => vec![],
        Err(ContentError::Invalid(s)) => s
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect(),
        Err(ContentError::Json(p, m)) => vec![format!("{}: {m}", p.display())],
        Err(ContentError::Io(p, m)) => vec![format!("IO {}: {m}", p.display())],
    }
}

/// Mora biti `Ok`; inače ispiše sve retke.
pub fn ok(res: Result<(), ContentError>) {
    assert!(res.is_ok(), "očekivan Ok, dobiveno: {:#?}", retci(&res));
}

/// Točno jedan redak greške i sadrži `igla` (loš test ne smije proći zbog druge greške).
pub fn jedina_greska(res: Result<(), ContentError>, igla: &str) {
    let r = retci(&res);
    assert!(
        r.len() == 1 && r[0].contains(igla),
        "očekivan točno jedan redak s {igla:?}, dobiveno: {r:#?}"
    );
}

/// Barem jedan redak sadrži `igla`; svaki drugi redak mora sadržavati neku od dopuštenih (posljedičnih) igala.
pub fn greska_uz(res: Result<(), ContentError>, igla: &str, dopusteno: &[&str]) {
    let r = retci(&res);
    assert!(
        r.iter().any(|l| l.contains(igla)),
        "nema retka s {igla:?}, dobiveno: {r:#?}"
    );
    for l in &r {
        assert!(
            l.contains(igla) || dopusteno.iter().any(|d| l.contains(d)),
            "neočekivan redak {l:?} uz {igla:?} (dopušteno {dopusteno:?})"
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Sintetski zapisi za veće banke (S1–S4, R4, J3, J4, O7)
// ---------------------------------------------------------------------------------------------------------------

/// Riječ iz slova: prefiks + tri slova iz broja; različiti brojevi (< 17576) daju različite riječi.
pub fn rijec(prefiks: char, mut n: usize) -> String {
    let mut s = String::from(prefiks);
    for _ in 0..3 {
        s.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
    }
    s
}

pub struct Spec {
    pub uloga: &'static str,
    pub razina: &'static str,
    pub dob: u8,
    pub rod: &'static str,
}

fn godine(razina: &str) -> u8 {
    match razina {
        "pocetnik" => 1,
        "iskusan" => 4,
        _ => 8,
    }
}

/// Valjan radnik za redni broj `i` (jedinstveno ime i prezime) i skupinu podrijetla.
pub fn radnik_i(i: usize, podrijetlo: &str, sp: &Spec) -> Value {
    let (zemlja, mjesto, jezici) = match podrijetlo {
        "lokalno" => ("HR", "Kastav", json!([["hr", 4], ["en", 3]])),
        "regija" => ("HR", "Osijek", json!([["hr", 4], ["en", 3]])),
        _ => ("NP", "Pokhara", json!([["ne", 4], ["en", 3]])),
    };
    json!({
        "id": format!("R-{:05}", i + 1), "ime": rijec('M', i), "prezime": rijec('K', i * 7 + 3),
        "rod": sp.rod, "dob": sp.dob, "uloga": sp.uloga, "razina": sp.razina,
        "godine_iskustva": godine(sp.razina), "podrijetlo": podrijetlo, "zemlja": zemlja,
        "mjesto": mjesto, "jezici": jezici
    })
}

/// Deset valjanih radnika s različitim imenima i prezimenima (R-00001 … R-00010).
pub fn deset_radnika() -> Vec<Value> {
    let sp = Spec {
        uloga: "sobar",
        razina: "iskusan",
        dob: 40,
        rod: "z",
    };
    (0..10).map(|i| radnik_i(i, "lokalno", &sp)).collect()
}

/// Skupine (podrijetlo, specifikacije) redom; ID-jevi teku kroz sve skupine, serije po 50 zapisa.
pub fn radnici_iz(skupine: Vec<(&str, Vec<Spec>)>) -> Datoteke {
    let mut svi = Vec::new();
    for (pod, specs) in &skupine {
        for sp in specs {
            svi.push(radnik_i(svi.len(), pod, sp));
        }
    }
    svi.chunks(50)
        .enumerate()
        .map(|(n, c)| {
            (
                format!("radnici/serija-{n:03}.json"),
                serija("radnici", n as u32, c.to_vec()),
            )
        })
        .collect()
}

/// Specifikacije: za svaku razinu redom `n` zapisa; rod izmjenično z/m; ista uloga i dob.
pub fn razine(uloga: &'static str, dob: u8, po_razini: &[(&'static str, usize)]) -> Vec<Spec> {
    let mut v = Vec::new();
    for (razina, n) in po_razini {
        for _ in 0..*n {
            let rod = if v.len() % 2 == 0 { "z" } else { "m" };
            v.push(Spec {
                uloga,
                razina,
                dob,
                rod,
            });
        }
    }
    v
}
