//! Brana zamrznutih otisaka v1: SHA-256 svake datoteke u `tests/fixtures/` mora biti jednak konstanti ovdje.
//! Ako je generator (`v1_frozen.rs`, `#[ignore]`) preradio fixturee, ovaj test pada — i to je namjera:
//! konstantu smije promijeniti samo svjestan, vidljiv potez u pregledu. Fixturei se u sloju 1 ne regeneriraju.
//! Bajtovi se normaliziraju (CRLF -> LF) jer `.gitattributes` (`* text=auto eol=lf`) na Windowsu i u Linuxu
//! daje isti sadržaj u repou; normalizacija samo osigurava isti hash i uz checkout s CRLF-om.

use sha2::{Digest, Sha256};

const SEALED: [(&str, &str); 4] = [
    (
        "v1-game-log.json",
        "499aaa9852bd37ac5643075a39361dc90f22de4fe1613304d96e27edd1c3ec2f",
    ),
    (
        "v1-golden.json",
        "46bb08091529f45a55554b18e962e5d593323c831484795d36044c4a939ad70d",
    ),
    // Obnova v2, O-5: mentor stare v1 igre (ulaz, činjenice, rezerva), napisan kodom prije obnove.
    (
        "v1-mentor.json",
        "abb4e467555a325c101d9a098938a643a8c93171a8f778146a35f5fdac2c1e69",
    ),
    (
        "v1-snapshot.json",
        "96583bde236eed5893e1c0964dbc818a167735e7e23b5a6b55f5287b8ee6a064",
    ),
];

fn sha256_lf(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let lf: Vec<u8> = bytes
        .iter()
        .enumerate()
        .filter(|&(i, &b)| !(b == b'\r' && bytes.get(i + 1) == Some(&b'\n')))
        .map(|(_, &b)| b)
        .collect();
    format!("{:x}", Sha256::digest(lf))
}

#[test]
fn frozen_v1_fixtures_match_their_sealed_hash() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (name, want) in SEALED {
        assert_eq!(
            sha256_lf(&dir.join(name)),
            want,
            "{name}: zamrznuti fixture je promijenjen — v1 se ne regenerira (greška je u novom kodu)"
        );
    }
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    on_disk.sort();
    let sealed: Vec<&str> = SEALED.iter().map(|(n, _)| *n).collect();
    assert_eq!(on_disk, sealed, "novi ili obrisani fixture mimo brane");
}
