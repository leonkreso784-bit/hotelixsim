//! Brana zamrznutog otiska koraka 3: SHA-256 `fixtures/step3-snapshot.json` mora biti jednak konstanti ovdje.
//! Promjena konstante je svjestan potez koji se vidi u pregledu; fixture se ne regenerira.
//! CRLF se normalizira u LF (repo ih sprema kao LF, `.gitattributes` `* text=auto eol=lf`).

use sha2::{Digest, Sha256};

const SEALED: [(&str, &str); 1] = [(
    "step3-snapshot.json",
    "5367a95b7862b633416bfda5852ee33a6cb0fd50a5709db01108a677fc0403ad",
)];

#[test]
fn frozen_step3_fixture_matches_its_sealed_hash() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (name, want) in SEALED {
        let bytes = std::fs::read(dir.join(name)).unwrap();
        let lf: Vec<u8> = bytes
            .iter()
            .enumerate()
            .filter(|&(i, &b)| !(b == b'\r' && bytes.get(i + 1) == Some(&b'\n')))
            .map(|(_, &b)| b)
            .collect();
        assert_eq!(
            format!("{:x}", Sha256::digest(lf)),
            want,
            "{name}: zamrznuti fixture je promijenjen"
        );
    }
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    on_disk.sort();
    assert_eq!(on_disk, ["step3-snapshot.json"], "novi fixture mimo brane");
}
