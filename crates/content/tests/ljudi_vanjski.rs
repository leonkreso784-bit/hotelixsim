//! Banka ljudi (K2v-a), treća vrsta VANJSKI: oblik, dosljednost, ustanova, jedinstvenost i okvir (kartica K2v,
//! „Testovi čuvara": V0, VO1–VO6, VD1–VD6, VT1–VT4, VJ1–VJ2, VR1–VR5). Svaki loš primjer mijenja jedno polje
//! valjanog `vanjski()` i traži točno jedan redak s doslovnom porukom. Okvir se gradi u testu.
mod ljudi_common;

use content::ContentError;
use content::ljudi::{Banka, izvjestaj, provjeri_prekretnicu};
use ljudi_common::*;
use serde_json::{Value, json};

fn v(zapis: Value) -> Result<(), ContentError> {
    provjeri_v(vec![zapis])
}

/// Valjan vanjski u tržišnoj ulozi (bez dobnih pravila službi i inspektora).
fn novinar() -> Value {
    s(vanjski(), "uloga", json!("novinar"))
}

// ------------------------------------------------------------------------------------------------ V0, VO: oblik

#[test]
fn vanjski_osnovni_zapis_prolazi() {
    ok(v(vanjski()));
}

#[test]
fn vanjski_nema_razine() {
    let z = s(vanjski(), "razina", json!("iskusan"));
    jedina_greska(v(z), "unknown field `razina`");
}

#[test]
fn vanjski_bez_uloge() {
    jedina_greska(v(bez(vanjski(), "uloga")), "missing field `uloga`");
}

#[test]
fn vanjski_nepoznata_uloga() {
    let z = s(vanjski(), "uloga", json!("sobar"));
    jedina_greska(v(z), "unknown variant `sobar`");
}

#[test]
fn vanjski_oblik_id_a() {
    let z = s(vanjski(), "id", json!("R-00001"));
    jedina_greska(v(z), "id: 'R-00001' mora biti V- i 5 znamenki");
}

#[test]
fn vanjski_zaglavlje() {
    let pokvari = |polje: &str, vrijednost: Value| {
        let mut z = serija("vanjski", 0, vec![vanjski()]);
        z[polje] = vrijednost;
        provjeri_svi(
            &okvir(),
            vec![],
            vec![],
            vec![("vanjski/serija-000.json".into(), z)],
        )
    };
    jedina_greska(
        pokvari("vrsta", json!("gosti")),
        "vrsta 'gosti' u mapi vanjski",
    );
    jedina_greska(
        pokvari("do", json!("V-00040")),
        "raspon mora biti V-00001–V-00050",
    );
}

#[test]
fn vanjski_dob_u_rasponu() {
    // Tržišna uloga: ni dobna pravila službi ni inspektora ne smetaju.
    jedina_greska(v(s(novinar(), "dob", json!(19))), "dob: 19 izvan 20–80");
    jedina_greska(v(s(novinar(), "dob", json!(81))), "dob: 81 izvan 20–80");
}

// ------------------------------------------------------------------------------------------------ VD: dosljednost

#[test]
fn lokalne_uloge_iz_hrvatske() {
    let z = s(
        s(
            s(vanjski(), "uloga", json!("policajac")),
            "zemlja",
            json!("AT"),
        ),
        "jezici",
        json!([["de", 4]]),
    );
    jedina_greska(v(z), "zemlja: uloga policajac traži HR, ima AT");
}

#[test]
fn lokalne_uloge_iz_kataloga() {
    let z = s(
        s(vanjski(), "uloga", json!("susjed")),
        "mjesto",
        json!("Osijek"),
    );
    jedina_greska(v(z), "mjesto: 'Osijek' nije u mjesta.json → lokalno");
}

#[test]
fn trzisna_uloga_smije_biti_strana() {
    let z = s(
        s(s(novinar(), "zemlja", json!("AT")), "mjesto", json!("Graz")),
        "jezici",
        json!([["de", 4]]),
    );
    ok(v(z));
}

#[test]
fn sluzbe_najvise_65() {
    let z = s(s(vanjski(), "uloga", json!("vatrogasac")), "dob", json!(70));
    jedina_greska(v(z), "dob: 70, uloga vatrogasac najviše 65");
}

#[test]
fn inspektor_najmanje_25() {
    let z = s(
        s(vanjski(), "uloga", json!("inspektor_porezni")),
        "dob",
        json!(22),
    );
    jedina_greska(v(z), "dob: 22, uloga inspektor_porezni najmanje 25");
}

#[test]
fn vanjski_materinji_na_cetiri() {
    let z = s(vanjski(), "jezici", json!([["hr", 3]]));
    jedina_greska(
        v(z),
        "jezici: materinji jezik zemlje HR mora biti na razini 4",
    );
}

// ------------------------------------------------------------------------------------------------ VT: ustanova i opis

#[test]
fn ustanova_duljina() {
    jedina_greska(
        v(s(vanjski(), "ustanova", json!("a"))),
        "ustanova: 1 znak, dopušteno 2–60",
    );
    jedina_greska(
        v(s(vanjski(), "ustanova", json!("a".repeat(61)))),
        "ustanova: 61 znakova",
    );
}

#[test]
fn ustanova_bez_vlastitog_imena() {
    let z = s(vanjski(), "ustanova", json!("sanitarna inspekcija Rijeka"));
    jedina_greska(
        v(z),
        "ustanova: veliko slovo u 'Rijeka' (samo generički naziv)",
    );
}

#[test]
fn ustanova_zabranjeni_naziv() {
    // Marka iz testnog okvira („Marka Primjer", vidi `zabranjeno_json`), malim slovima da pravilo velikog slova šuti.
    let z = s(vanjski(), "ustanova", json!("agencija marka primjer"));
    jedina_greska(v(z), "ustanova: zabranjeni naziv 'Marka Primjer'");
}

#[test]
fn vanjski_opis_po_pravilima() {
    let z = s(
        vanjski(),
        "opis",
        json!("Svi kažu da je pouzdan. Radi u inspekciji."),
    );
    jedina_greska(v(z), "opis: zabranjena riječ 'pouzdan'");
}

// ------------------------------------------------------------------------------------------------ VJ: jedinstvenost

#[test]
fn vanjski_puno_ime_jedinstveno() {
    // Isto puno ime kao radnik (R-00001 je prvi u poretku, pa vanjski dobiva grešku).
    let r = radnik();
    let isti = s(
        s(vanjski(), "ime", r["ime"].clone()),
        "prezime",
        r["prezime"].clone(),
    );
    let (rd, gd) = banka(vec![r], vec![]);
    jedina_greska(
        provjeri_svi(&okvir(), rd, gd, vanjski_datoteke(vec![isti])),
        "već ima R-00001",
    );
    // Partner iz okvira („Marko Perić") zauzima ime i za vanjske.
    let z = s(
        s(vanjski(), "ime", json!("Marko")),
        "prezime",
        json!("Peric"),
    );
    jedina_greska(v(z), "puno ime 'Marko Peric' već je partner");
}

#[test]
fn ucestalost_vanjskih() {
    let mut z = deset_vanjskih();
    z[0]["prezime"] = json!("Horvat");
    z[1]["prezime"] = json!("Horvat");
    jedina_greska(
        provjeri_v(z),
        "prezime 'Horvat': 2 od 10 vanjskih, najviše 1",
    );
}

// ------------------------------------------------------------------------------------------------ VR: okvir, izvještaj

#[test]
fn raspodjele_vanjskih_zbroj() {
    let mut r = raspodjele_json();
    r["vanjski"]["uloga"]["policajac"] = json!(500);
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "raspodjele.vanjski.uloga: zbroj 9900 bp, mora biti 10000",
    );
}

#[test]
fn rod_vanjskih_ima_izvor() {
    let mut r = raspodjele_json();
    r["vanjski"]["rod_po_ulozi"]["policajac"]["izvor"] = json!("");
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(okvir.provjeri(), "rod_po_ulozi.policajac: izvor je obvezan");

    let mut r = raspodjele_json();
    r["vanjski"]["rod_po_ulozi"]
        .as_object_mut()
        .unwrap()
        .remove("vatrogasac");
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(okvir.provjeri(), "rod_po_ulozi: nema uloge vatrogasac");
}

#[test]
fn dob_vanjskih_bez_rupa() {
    let mut r = raspodjele_json();
    r["vanjski"]["dob"] = json!({"20-29": 1000, "31-80": 9000});
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "raspodjele.vanjski.dob: razredi moraju pokriti 20–80 bez rupa i preklapanja",
    );
}

fn banka_samo_vanjski() -> Banka {
    Banka::iz_vrijednosti(vec![], vec![])
        .unwrap()
        .s_vanjskima(vanjski_datoteke(vec![vanjski()]))
        .unwrap()
}

#[test]
fn izvjestaj_ukljucuje_vanjske() {
    let izv = izvjestaj(&banka_samo_vanjski(), &okvir().raspodjele);
    for pocetak in ["vanjski: uloga", "vanjski: dob", "vanjski: rod"] {
        assert!(
            izv.iter().any(|o| o.dimenzija.starts_with(pocetak)),
            "izvještaj nema dimenziju koja počinje s {pocetak:?}: {:#?}",
            izv.iter().map(|o| &o.dimenzija).collect::<Vec<_>>()
        );
    }
}

#[test]
fn prekretnica_samo_vanjski_nije_prazna() {
    let res = provjeri_prekretnicu(&banka_samo_vanjski(), &okvir().raspodjele);
    let r = retci(&res);
    assert!(
        !r.iter().any(|l| l.contains("banka je prazna")),
        "banka s vanjskim ne smije biti prazna: {r:#?}"
    );
}

// ------------------------------------------------------------------------------------------------ K5-L: uloga lijecnik

/// Valjan liječnik izvan hitne: služba (HR, lokalno), stručna uloga.
fn lijecnik() -> Value {
    s(vanjski(), "uloga", json!("lijecnik"))
}

#[test]
fn lijecnik_lokalno_dob_40_prolazi() {
    ok(v(s(lijecnik(), "dob", json!(40))));
}

#[test]
fn lijecnik_je_lokalna_sluzba_iz_hrvatske() {
    // Mjesto ostaje iz `lokalno` (kao `lokalne_uloge_iz_hrvatske`); mjesto izvan kataloga je `lijecnik_mjesto_iz_kataloga`.
    let z = s(
        s(lijecnik(), "zemlja", json!("AT")),
        "jezici",
        json!([["de", 4]]),
    );
    jedina_greska(v(z), "zemlja: uloga lijecnik traži HR, ima AT");
}

#[test]
fn lijecnik_mjesto_iz_kataloga() {
    let z = s(lijecnik(), "mjesto", json!("Osijek"));
    jedina_greska(v(z), "mjesto: 'Osijek' nije u mjesta.json → lokalno");
}

#[test]
fn lijecnik_najvise_65() {
    jedina_greska(
        v(s(lijecnik(), "dob", json!(70))),
        "dob: 70, uloga lijecnik najviše 65",
    );
}

#[test]
fn lijecnik_najmanje_25() {
    jedina_greska(
        v(s(lijecnik(), "dob", json!(24))),
        "dob: 24, uloga lijecnik najmanje 25",
    );
}

#[test]
fn raspodjele_traze_lijecnika() {
    let mut r = raspodjele_json();
    r["vanjski"]["uloga"]
        .as_object_mut()
        .unwrap()
        .remove("lijecnik");
    r["vanjski"]["uloga"]["dobavljac"] = json!(1700);
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "raspodjele.vanjski.uloga: nema uloge lijecnik",
    );

    let mut r = raspodjele_json();
    r["vanjski"]["rod_po_ulozi"]
        .as_object_mut()
        .unwrap()
        .remove("lijecnik");
    let okvir = okvir_iz(mjesta_json(), zabranjeno_json(), r);
    jedina_greska(
        okvir.provjeri(),
        "raspodjele.vanjski.rod_po_ulozi: nema uloge lijecnik",
    );
}

#[test]
fn repo_raspodjele_imaju_lijecnika() {
    let tekst = std::fs::read_to_string(content::repo_dir().join("v2/ljudi/raspodjele.json"))
        .expect("raspodjele.json se čita");
    let r: Value = serde_json::from_str(&tekst).unwrap();
    let uloga = &r["vanjski"]["uloga"];
    assert_eq!(uloga["lijecnik"], 400);
    assert_eq!(uloga["djelatnik_hitne"], 400);
    assert_eq!(uloga["dobavljac"], 1300);
    assert_eq!(uloga["poslovni_klijent"], 1300);
    assert_eq!(uloga["zastupnik_agencije"], 1000);
    let zbroj: u64 = uloga
        .as_object()
        .unwrap()
        .values()
        .map(|x| x.as_u64().unwrap())
        .sum();
    assert_eq!(zbroj, 10_000);
    let rod = &r["vanjski"]["rod_po_ulozi"]["lijecnik"];
    assert_eq!(rod["z_bp"], 6500);
    assert!(
        !rod["izvor"].as_str().unwrap_or("").trim().is_empty(),
        "izvor je obvezan"
    );
    assert_eq!(rod["potvrdeno"], false);
}
