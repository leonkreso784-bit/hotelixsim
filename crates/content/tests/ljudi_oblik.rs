//! Banka ljudi (K1a), oblik i jedinstvenost: B0, O1–O11, J1–J4 (kartica K1, „Testovi čuvara").
//! Svaki loš primjer mijenja jedno polje valjanog zapisa i daje točno jednu grešku s doslovnom porukom.
mod ljudi_common;

use ljudi_common::*;
use serde_json::json;

const KIRILICA_ANA: &str = "\u{410}\u{43d}\u{43d}\u{430}"; // „Анна"

fn radnici_s(v: serde_json::Value) -> Result<(), content::ContentError> {
    provjeri_zapise(vec![v], vec![])
}

fn gosti_s(v: serde_json::Value) -> Result<(), content::ContentError> {
    provjeri_zapise(vec![], vec![v])
}

#[test]
fn osnovni_zapisi_prolaze() {
    ok(provjeri_zapise(vec![radnik()], vec![gost()]));
}

#[test]
fn nepoznato_polje_pada() {
    let r = s(radnik(), "vjestine", json!({"komunikacija": 3}));
    jedina_greska(radnici_s(r), "unknown field `vjestine`");
}

#[test]
fn obvezno_polje_fali() {
    jedina_greska(radnici_s(bez(radnik(), "razina")), "missing field `razina`");
}

#[test]
fn nepoznat_kod_pada() {
    let r = s(radnik(), "uloga", json!("voditelj"));
    jedina_greska(radnici_s(r), "unknown variant `voditelj`");
}

#[test]
fn oblik_id_a() {
    let r = s(radnik(), "id", json!("R-12"));
    jedina_greska(radnici_s(r), "id: 'R-12' mora biti R- i 5 znamenki");
}

#[test]
fn id_izvan_raspona_serije() {
    let r = s(radnik(), "id", json!("R-00051"));
    jedina_greska(
        radnici_s(r),
        "id: R-00051 izvan raspona serije R-00001–R-00050",
    );
}

#[test]
fn zaglavlje_serije() {
    let r = || vec![radnik()];
    let pokvari = |polje: &str, vrijednost: serde_json::Value| {
        let mut z = serija("radnici", 0, r());
        z[polje] = vrijednost;
        provjeri(vec![("radnici/serija-000.json".into(), z)], vec![])
    };
    jedina_greska(pokvari("format", json!(2)), "format 2, poznat je samo 1");
    jedina_greska(
        pokvari("vrsta", json!("gosti")),
        "vrsta 'gosti' u mapi radnici",
    );
    jedina_greska(
        pokvari("do", json!("R-00040")),
        "raspon mora biti R-00001–R-00050",
    );
    // serija: 1 u serija-000.json može povući i grešku raspona (zaglavlje 1–50 nije raspon serije 1).
    greska_uz(
        pokvari("serija", json!(1)),
        "serija 1 ne odgovara imenu serija-000",
        &["raspon mora biti"],
    );
}

#[test]
fn najvise_50_zapisa() {
    // 51 jedinstven zapis ne može stati u raspon od 50 ID-jeva: uz poruku o broju smije biti i greška raspona.
    let sp = Spec {
        uloga: "sobar",
        razina: "iskusan",
        dob: 40,
        rod: "z",
    };
    let zapisi = (0..51).map(|i| radnik_i(i, "lokalno", &sp)).collect();
    greska_uz(
        provjeri_zapise(zapisi, vec![]),
        "serija ima 51 zapis, najviše 50",
        &["izvan raspona serije"],
    );
}

#[test]
fn dob_u_rasponu() {
    // Radnik od 17 godina: iskustvo 0 (početnik) da druga pravila ne smetaju.
    let r = s(
        s(s(radnik(), "dob", json!(17)), "razina", json!("pocetnik")),
        "godine_iskustva",
        json!(0),
    );
    jedina_greska(radnici_s(r), "dob: 17 izvan 18–65");
    jedina_greska(gosti_s(s(gost(), "dob", json!(86))), "dob: 86 izvan 18–85");
}

#[test]
fn ime_i_prezime() {
    jedina_greska(
        radnici_s(s(radnik(), "ime", json!("Ana2"))),
        "ime: nedopušten znak '2'",
    );
    jedina_greska(
        radnici_s(s(radnik(), "prezime", json!("K"))),
        "prezime: 1 znak, dopušteno 2–30",
    );
    jedina_greska(
        radnici_s(s(radnik(), "ime", json!(KIRILICA_ANA))),
        "ime: nedopušten znak '\u{410}'",
    );
}

#[test]
fn jezici_kod_i_razina() {
    let uz = |jezik: serde_json::Value| {
        radnici_s(s(radnik(), "jezici", json!([["hr", 4], ["en", 3], jezik])))
    };
    jedina_greska(uz(json!(["xx", 2])), "jezici: nepoznat jezik 'xx'");
    let r = s(radnik(), "jezici", json!([["hr", 4], ["en", 5]]));
    jedina_greska(radnici_s(r), "jezici: razina 5 za 'en', dopušteno 1–4");
}

#[test]
fn zemlja_poznata() {
    jedina_greska(
        gosti_s(s(gost(), "zemlja", json!("XX"))),
        "zemlja: 'XX' nije u mjesta.json",
    );
}

#[test]
fn id_jedinstven() {
    // Isti ID dvaput u jednoj seriji (imena različita).
    let dva = vec![
        s(s(radnik(), "id", json!("R-00003")), "ime", json!("Ana")),
        s(
            s(s(radnik(), "id", json!("R-00003")), "ime", json!("Iva")),
            "prezime",
            json!("Babić"),
        ),
    ];
    jedina_greska(provjeri_zapise(dva, vec![]), "id: R-00003 se ponavlja");
    // Isti ID u dvije serije: drugi je izvan raspona svoje serije, pa je ta posljedična greška dopuštena.
    let a = serija("radnici", 0, vec![s(radnik(), "id", json!("R-00003"))]);
    let b = serija(
        "radnici",
        1,
        vec![s(
            s(s(radnik(), "id", json!("R-00003")), "ime", json!("Iva")),
            "prezime",
            json!("Babić"),
        )],
    );
    greska_uz(
        provjeri(
            vec![
                ("radnici/serija-000.json".into(), a),
                ("radnici/serija-001.json".into(), b),
            ],
            vec![],
        ),
        "id: R-00003 se ponavlja",
        &["izvan raspona serije"],
    );
}

#[test]
fn puno_ime_jedinstveno() {
    let r = s(
        s(radnik(), "ime", json!("Ivan")),
        "prezime",
        json!("Kovačević"),
    );
    let g = s(
        s(gost(), "ime", json!("Ivan")),
        "prezime",
        json!("Kovacevic"),
    );
    jedina_greska(
        provjeri_zapise(vec![r], vec![g]),
        "puno ime 'Ivan Kovacevic' već ima R-00001",
    );
    // Partner iz okvira („Marko Perić") zauzima ime i za goste.
    let g = s(s(gost(), "ime", json!("Marko")), "prezime", json!("Peric"));
    jedina_greska(
        provjeri_zapise(vec![], vec![g]),
        "puno ime 'Marko Peric' već je partner",
    );
}

#[test]
fn ucestalost_prezimena() {
    let mut r = deset_radnika();
    r[0]["prezime"] = json!("Horvat");
    r[1]["prezime"] = json!("Horvat");
    jedina_greska(
        provjeri_zapise(r, vec![]),
        "prezime 'Horvat': 2 od 10 radnika, najviše 1",
    );
}

#[test]
fn ucestalost_imena() {
    let mut r = deset_radnika();
    r[0]["ime"] = json!("Ana");
    r[1]["ime"] = json!("Ana");
    jedina_greska(
        provjeri_zapise(r, vec![]),
        "ime 'Ana': 2 od 10 radnika, najviše 1",
    );
}
