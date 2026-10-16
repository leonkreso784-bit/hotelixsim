//! Banka ljudi (K1a), okvir i repo: R1–R4 i G1. Okvir se gradi u testu (`okvir_iz`), neovisno o repo podacima;
//! G1 čita pravi `content/hotel/v2/ljudi/` (K1a: okvir i prazne mape).
mod ljudi_common;

use content::ljudi::{Banka, Okvir, provjeri_prekretnicu};
use content::repo_dir;
use ljudi_common::*;
use serde_json::json;

#[test]
fn udjeli_zbroj_10000() {
    let mut r = raspodjele_json();
    r["radnici"]["uloga"]["sobar"] = json!(2700);
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "raspodjele.radnici.uloga: zbroj 9900 bp, mora biti 10000",
    );
}

#[test]
fn rod_po_ulozi_ima_izvor() {
    let mut r = raspodjele_json();
    r["radnici"]["rod_po_ulozi"]["sobar"]["izvor"] = json!("");
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "rod_po_ulozi.sobar: izvor je obvezan (ADR-042 t. 8)",
    );

    let mut r = raspodjele_json();
    r["radnici"]["rod_po_ulozi"]
        .as_object_mut()
        .unwrap()
        .remove("tehnicar");
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(okvir.provjeri(), "rod_po_ulozi: nema uloge tehnicar");
}

#[test]
fn mjesta_dosljedna() {
    let mut m = mjesta_json();
    m["zemlje"]["AT"]["materinji"] = json!(["xx"]);
    let okvir = okvir_iz(m, zabranjeno_json(), raspodjele_json());
    jedina_greska(
        okvir.provjeri(),
        "mjesta.zemlje.AT.materinji: nepoznat jezik 'xx'",
    );

    let mut m = mjesta_json();
    m["zemlje"].as_object_mut().unwrap().remove("HR");
    let okvir = okvir_iz(m, zabranjeno_json(), raspodjele_json());
    jedina_greska(okvir.provjeri(), "mjesta.zemlje: nema HR");
}

/// 100 radnika točno po ciljevima razine, dobi i podrijetla; uloge po cilju osim sobara (12 umjesto 28) i +2 svaka
/// od ostalih osam (strogo unutar ±300 bp). Rod i jezici nisu namještani (njih ovaj test ne ocjenjuje).
fn banka_sa_sobarima_ispod_cilja() -> Banka {
    let mut uloge: Vec<&'static str> = Vec::new();
    for (uloga, n) in [
        ("sobar", 12),
        ("kuhar", 16),
        ("konobar", 16),
        ("recepcioner", 13),
        ("referent_uprave", 11),
        ("tehnicar", 10),
        ("sanker", 9),
        ("terapeut", 7),
        ("referent_prodaje", 6),
    ] {
        uloge.extend(std::iter::repeat_n(uloga, n));
    }
    assert_eq!(uloge.len(), 100);
    let zapisi: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            let razina = match i % 20 {
                0..=6 => "pocetnik",
                7..=15 => "iskusan",
                _ => "strucnjak",
            };
            let dob = match (i * 7) % 100 {
                0..=21 => 24,
                22..=51 => 30,
                52..=83 => 40,
                _ => 55,
            };
            let podrijetlo = ["lokalno", "lokalno", "regija", "inozemstvo"][i % 4];
            let rod = if i % 2 == 0 { "z" } else { "m" };
            radnik_i(
                i,
                podrijetlo,
                &Spec {
                    uloga: uloge[i],
                    razina,
                    dob,
                    rod,
                },
            )
        })
        .collect();
    let datoteke = zapisi
        .chunks(50)
        .enumerate()
        .map(|(n, c)| {
            (
                format!("radnici/serija-{n:03}.json"),
                serija("radnici", n as u32, c.to_vec()),
            )
        })
        .collect();
    Banka::iz_vrijednosti(datoteke, vec![]).unwrap()
}

#[test]
fn prekretnica_odstupanje() {
    let b = banka_sa_sobarima_ispod_cilja();
    let res = provjeri_prekretnicu(&b, &okvir().raspodjele);
    let r = retci(&res);
    let igla = "prekretnica: uloga sobar 1200 bp prema cilju 2800 bp (dopušteno ±300 bp)";
    assert!(
        r.iter().any(|l| l.contains(igla)),
        "nema retka {igla:?}, dobiveno: {r:#?}"
    );
    // Nijedna druga uloga ne smije biti označena (odstupaju najviše 200 bp). Redci o drugim dimenzijama
    // (rod, jezici, zemlje) ovdje nisu predmet testa.
    for l in &r {
        assert!(
            l.starts_with("prekretnica: ") && (l.contains(igla) || !l.contains("uloga ")),
            "neočekivan redak {l:?}"
        );
    }
}

#[test]
fn repo_banka_prolazi() {
    let dir = repo_dir();
    let okvir = Okvir::read(&dir).expect("okvir se čita iz content/hotel/v2/ljudi");
    ok(okvir.provjeri());
    let banka = Banka::read(&dir).expect("banka se čita");
    ok(banka.provjeri(&okvir));
    // K1b: 10 radnika i 10 gostiju za Leonov pregled.
    let radnika: usize = banka.radnici.iter().map(|(_, s)| s.zapisi.len()).sum();
    let gostiju: usize = banka.gosti.iter().map(|(_, s)| s.zapisi.len()).sum();
    assert_eq!((radnika, gostiju), (10, 10));
}
