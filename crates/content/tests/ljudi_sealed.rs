//! Brana serija banke ljudi: svaka `serija-*.json` pod `content/hotel/v2/ljudi/radnici`, `gosti` i `vanjski` mora biti u točno
//! jednom popisu — `ZAPECACENO` (SHA-256, nepromjenjiva osim povlačenjem, BANKA_LJUDI §4.3) ili `OTVORENO`
//! (čeka Leonov pregled). Pečate se samo serije; `raspodjele/mjesta/zabranjeno` mijenja glavna sesija, a
//! `repo_banka_prolazi` svaki put ponovno provjeri cijelu banku. Obrazac: `fixtures_sealed.rs` (CRLF → LF).
//! Promjena popisa je svjestan potez koji se vidi u pregledu.

use sha2::{Digest, Sha256};

/// (put pod `ljudi/`, SHA-256 sadržaja s LF završecima).
/// Serije 000 i 001 radnika i gostiju: Leon pregledao svih 120 i odobrio pečat 17.10. („super su").
const ZAPECACENO: [(&str, &str); 4] = [
    (
        "gosti/serija-000.json",
        "70da915d34b136be92956aa70922b693913ce37a8c98b6c3342191d936ab6759",
    ),
    (
        "gosti/serija-001.json",
        "1a89ff47f834771171c4a4b2e64a682b0f744373613675cd1392b16c0096fa67",
    ),
    (
        "radnici/serija-000.json",
        "360d99eb769ff9de33a7503e7ecb87909c72cd53f700027757c01a99103dc872",
    ),
    (
        "radnici/serija-001.json",
        "f45bd629217915b678eaab28eba4cee643600dee8e17e7ae3051574645b71efa",
    ),
];

/// Serije koje postoje, ali još nisu pečaćene (Leonov pregled u tijeku; K3: serija-002 radnika i gostiju, uzorak 10 + 10;
/// K4: serija-003 radnika i gostiju; K5: serija-004 radnika i gostiju, vanjski serija-002).
const OTVORENO: &[&str] = &[
    "gosti/serija-002.json",
    "gosti/serija-003.json",
    "gosti/serija-004.json",
    "radnici/serija-002.json",
    "radnici/serija-003.json",
    "radnici/serija-004.json",
    "vanjski/serija-000.json",
    "vanjski/serija-001.json",
    "vanjski/serija-002.json",
];

fn lf_hash(bytes: &[u8]) -> String {
    let lf: Vec<u8> = bytes
        .iter()
        .enumerate()
        .filter(|&(i, &b)| !(b == b'\r' && bytes.get(i + 1) == Some(&b'\n')))
        .map(|(_, &b)| b)
        .collect();
    format!("{:x}", Sha256::digest(lf))
}

#[test]
fn ljudi_serije_su_zapecacene_ili_otvorene() {
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content/hotel/v2/ljudi");
    let mut nadjeno: Vec<String> = Vec::new();
    for vrsta in ["radnici", "gosti", "vanjski"] {
        for e in std::fs::read_dir(dir.join(vrsta)).unwrap() {
            let ime = e.unwrap().file_name().into_string().unwrap();
            if ime.starts_with("serija-") && ime.ends_with(".json") {
                nadjeno.push(format!("{vrsta}/{ime}"));
            }
        }
    }
    nadjeno.sort();

    for put in &nadjeno {
        let pecat = ZAPECACENO.iter().find(|(n, _)| n == put);
        let otvorena = OTVORENO.contains(&put.as_str());
        assert!(pecat.is_some() != otvorena, "ljudi: '{put}' mimo brane");
        if let Some((_, want)) = pecat {
            let bytes = std::fs::read(dir.join(put)).unwrap();
            assert_eq!(
                &lf_hash(&bytes),
                want,
                "ljudi: zapečaćena serija '{put}' je promijenjena"
            );
        }
    }
    // Svaki unos u popisima mora postojati na disku.
    for put in ZAPECACENO
        .iter()
        .map(|(n, _)| *n)
        .chain(OTVORENO.iter().copied())
    {
        assert!(
            nadjeno.iter().any(|n| n == put),
            "ljudi: '{put}' je u popisu brane, a nema ga na disku"
        );
    }
}
