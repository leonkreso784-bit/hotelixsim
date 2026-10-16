//! Brana serija banke ljudi: svaka `serija-*.json` pod `content/hotel/v2/ljudi/radnici` i `gosti` mora biti u točno
//! jednom popisu — `ZAPECACENO` (SHA-256, nepromjenjiva osim povlačenjem, BANKA_LJUDI §4.3) ili `OTVORENO`
//! (čeka Leonov pregled). Pečate se samo serije; `raspodjele/mjesta/zabranjeno` mijenja glavna sesija, a
//! `repo_banka_prolazi` svaki put ponovno provjeri cijelu banku. Obrazac: `fixtures_sealed.rs` (CRLF → LF).
//! Promjena popisa je svjestan potez koji se vidi u pregledu.

use sha2::{Digest, Sha256};

/// (put pod `ljudi/`, SHA-256 sadržaja s LF završecima).
const ZAPECACENO: [(&str, &str); 0] = [];

/// Serije koje postoje, ali još nisu pečaćene (Leonov pregled u tijeku).
const OTVORENO: &[&str] = &[];

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
    for vrsta in ["radnici", "gosti"] {
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
