//! Banka ljudi (K1a), pravila zapisa: dosljednost D1–D8, stereotip S1–S5, tekst T1–T5, zabranjeno Z1–Z2.
//! Svaki loš primjer mijenja jedno polje valjanog zapisa i daje točno jednu grešku s doslovnom porukom
//! (osim S1–S3 i R4, gdje jedna iskrivljena skupina nužno pomiče i komplementarne kategorije).
mod ljudi_common;

use content::ContentError;
use ljudi_common::*;
use serde_json::{Value, json};

fn r1(v: Value) -> Result<(), ContentError> {
    provjeri_zapise(vec![v], vec![])
}

fn g1(v: Value) -> Result<(), ContentError> {
    provjeri_zapise(vec![], vec![v])
}

/// Opis točno `n` znakova: jedna rečenica koja završava točkom (bez zabranjenih riječi).
fn opis_duljine(n: usize) -> String {
    format!("Radila je {}.", "o".repeat(n - 11))
}

// ------------------------------------------------------------------------------------------ dosljednost D1–D8

#[test]
fn iskustvo_ne_prelazi_dob() {
    let r = s(
        s(s(radnik(), "dob", json!(30)), "razina", json!("strucnjak")),
        "godine_iskustva",
        json!(20),
    );
    jedina_greska(r1(r), "godine_iskustva: 20 veće od dob − 16 (14)");
}

#[test]
fn razina_prema_iskustvu() {
    let r = s(
        s(radnik(), "razina", json!("pocetnik")),
        "godine_iskustva",
        json!(5),
    );
    jedina_greska(r1(r), "razina pocetnik traži 0–2 godine iskustva, ima 5");
}

#[test]
fn podrijetlo_i_zemlja() {
    jedina_greska(
        r1(s(radnik(), "zemlja", json!("BA"))),
        "podrijetlo lokalno traži zemlju HR, ima BA",
    );
    jedina_greska(
        r1(s(radnik(), "podrijetlo", json!("inozemstvo"))),
        "podrijetlo inozemstvo ne može imati zemlju HR",
    );
}

#[test]
fn mjesto_iz_kataloga() {
    jedina_greska(
        r1(s(radnik(), "mjesto", json!("Osijek"))),
        "mjesto: 'Osijek' nije u mjesta.json → lokalno",
    );
}

#[test]
fn materinji_na_cetiri() {
    jedina_greska(
        g1(s(gost(), "jezici", json!([["de", 3]]))),
        "jezici: materinji jezik zemlje AT mora biti na razini 4",
    );
}

#[test]
fn jezik_jednom() {
    let r = s(radnik(), "jezici", json!([["hr", 4], ["en", 3], ["en", 2]]));
    jedina_greska(r1(r), "jezici: 'en' se ponavlja");
}

#[test]
fn susjedi_razumiju_hrvatski() {
    let r = s(
        s(
            s(radnik(), "podrijetlo", json!("inozemstvo")),
            "zemlja",
            json!("RS"),
        ),
        "jezici",
        json!([["sr", 4], ["hr", 3]]),
    );
    jedina_greska(
        r1(r),
        "jezici: zemlja RS traži hrvatski na razini 4 (ADR-042 t. 11)",
    );
}

#[test]
fn recepcioner_govori_engleski() {
    let r = s(radnik(), "jezici", json!([["hr", 4], ["en", 1]]));
    jedina_greska(r1(r), "jezici: recepcioner traži engleski barem 2");
}

// ------------------------------------------------------------------------------------------ stereotip S1–S5

/// Tri skupine podrijetla: lokalno 400, regija 50, inozemstvo 50 (ili `inoz` zapisa za S4).
fn razinska_banka(inoz: &[(&'static str, usize)]) -> Datoteke {
    radnici_iz(vec![
        (
            "lokalno",
            razine(
                "sobar",
                40,
                &[("pocetnik", 160), ("iskusan", 160), ("strucnjak", 80)],
            ),
        ),
        (
            "regija",
            razine(
                "sobar",
                40,
                &[("pocetnik", 20), ("iskusan", 20), ("strucnjak", 10)],
            ),
        ),
        ("inozemstvo", razine("sobar", 40, inoz)),
    ])
}

#[test]
fn stereotip_razina() {
    // Početnici: lokalno 40 %, regija 40 %, inozemstvo 80 %; cijela banka 220/500 = 4400 bp. Iskrivljena skupina
    // pomiče i iskusne/stručnjake (komplementarne razine), pa su drugi retci „stereotip: razina …" dopušteni.
    let b = razinska_banka(&[("pocetnik", 40), ("iskusan", 5), ("strucnjak", 5)]);
    greska_uz(
        provjeri(b, vec![]),
        "stereotip: razina pocetnik, skupina inozemstvo: 8000 bp prema 4400 bp cijele banke (dopušteno ±500 bp)",
        &["stereotip: razina "],
    );
}

#[test]
fn stereotip_dob() {
    let mlad = |n: usize| razine("sobar", 20, &[("iskusan", n)]);
    let star = |n: usize| razine("sobar", 40, &[("iskusan", n)]);
    let skupina = |m: usize, s: usize| {
        let mut v = mlad(m);
        v.extend(star(s));
        v
    };
    // Dob 18–24: lokalno 40 %, regija 40 %, inozemstvo 80 %.
    let b = radnici_iz(vec![
        ("lokalno", skupina(160, 240)),
        ("regija", skupina(20, 30)),
        ("inozemstvo", skupina(40, 10)),
    ]);
    greska_uz(
        provjeri(b, vec![]),
        "stereotip: dob 18-24, skupina inozemstvo:",
        &["stereotip: dob "],
    );
}

#[test]
fn stereotip_rod_u_ulozi() {
    // Sobari: z je 50 % u lokalnoj i regiji, 80 % u inozemstvu; cijela uloga 265/500 = 5300 bp.
    let sobari = |z: usize, m: usize| {
        let mut v = Vec::new();
        for (rod, n) in [("z", z), ("m", m)] {
            for _ in 0..n {
                v.push(Spec {
                    uloga: "sobar",
                    razina: "iskusan",
                    dob: 40,
                    rod,
                });
            }
        }
        v
    };
    let b = radnici_iz(vec![
        ("lokalno", sobari(200, 200)),
        ("regija", sobari(25, 25)),
        ("inozemstvo", sobari(40, 10)),
    ]);
    greska_uz(
        provjeri(b, vec![]),
        "stereotip: rod z u ulozi sobar, skupina inozemstvo: 8000 bp prema 5300 bp",
        &["stereotip: rod m u ulozi sobar"],
    );
}

#[test]
fn stereotip_suti_ispod_praga() {
    // Karakterizacija: ista iskrivljenost, ali skupina inozemstvo ima 39 < 40 zapisa — pravilo šuti.
    let b = razinska_banka(&[("pocetnik", 31), ("iskusan", 4), ("strucnjak", 4)]);
    ok(provjeri(b, vec![]));
}

#[test]
fn zabranjena_rijec_u_opisu() {
    let r = s(
        radnik(),
        "opis",
        json!("Svi kažu da je pouzdana. Radila je u Rovinju."),
    );
    jedina_greska(r1(r), "opis: zabranjena riječ 'pouzdana'");
}

// ------------------------------------------------------------------------------------------ tekst T1–T5

#[test]
fn opis_duljina() {
    jedina_greska(
        r1(s(radnik(), "opis", json!(opis_duljine(35)))),
        "opis: 35 znakova, dopušteno 40–220",
    );
    jedina_greska(
        r1(s(radnik(), "opis", json!(opis_duljine(221)))),
        "opis: 221 znakova",
    );
    // Granice su unutra.
    ok(r1(s(radnik(), "opis", json!(opis_duljine(40)))));
    ok(r1(s(radnik(), "opis", json!(opis_duljine(220)))));
}

#[test]
fn opis_najvise_dvije_recenice() {
    let o = "Radila je u Rijeci. Živi u Opatiji. Ima dvoje djece.";
    jedina_greska(
        r1(s(radnik(), "opis", json!(o))),
        "opis: 3 rečenice, najviše 2",
    );
}

#[test]
fn opis_zavrsava_tockom() {
    let o = "Radila je dvije sezone u Opatiji i jednu u Rijeci";
    jedina_greska(
        r1(s(radnik(), "opis", json!(o))),
        "opis: mora završiti točkom",
    );
}

#[test]
fn bez_vitica() {
    let o = "Dolazi sa suprugom i voli {gost.ime} jer je tamo bio.";
    jedina_greska(
        g1(s(gost(), "opis", json!(o))),
        "opis: vitice {} nisu dopuštene",
    );
}

#[test]
fn bez_cirilice() {
    let opis = "Radila je tri sezone u Nišu, a sada \u{436}\u{438}\u{432}\u{438} u Opatiji.";
    jedina_greska(
        r1(s(radnik(), "opis", json!(opis))),
        "opis: ćirilica nije dopuštena",
    );
    let zanimanje = "\u{423}\u{447}\u{438}\u{442}\u{435}\u{459}\u{438}\u{446}\u{430}";
    jedina_greska(
        g1(s(gost(), "zanimanje", json!(zanimanje))),
        "zanimanje: ćirilica nije dopuštena",
    );
}

// ------------------------------------------------------------------------------------------ zabranjeno Z1–Z2

#[test]
fn zabranjena_osoba() {
    let r = s(s(radnik(), "ime", json!("Leon")), "prezime", json!("Kreso"));
    jedina_greska(
        r1(r),
        "puno ime 'Leon Kreso' je na popisu zabranjenih osoba",
    );
}

#[test]
fn zabranjeni_hotel_ili_marka() {
    // Okvir iz common-a ima hotel „Hotel Primjer" i marku „Marka Primjer" (vlastiti testni primjer).
    let opis = "Dvije sezone je radila u Hotel Primjer u Rovinju.";
    jedina_greska(
        r1(s(radnik(), "opis", json!(opis))),
        "opis: zabranjeni naziv 'Hotel Primjer'",
    );
    jedina_greska(
        g1(s(gost(), "zanimanje", json!("prodavačica u Marka Primjer"))),
        "zanimanje: zabranjeni naziv",
    );
}
