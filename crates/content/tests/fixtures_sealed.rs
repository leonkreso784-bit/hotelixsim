//! Brana zamrznutih fixturea: SHA-256 `fixtures/step3-snapshot.json` (otisak koraka 3) i datoteka zamrznutog
//! sadržaja `fixtures/content-52ce161/` (testovi generatora situacija) mora biti jednak konstanti ovdje.
//! Promjena konstante je svjestan potez koji se vidi u pregledu; fixture se ne regenerira.
//! CRLF se normalizira u LF (repo ih sprema kao LF, `.gitattributes` `* text=auto eol=lf`).

use sha2::{Digest, Sha256};

const SEALED: [(&str, &str); 10] = [
    (
        "step3-snapshot.json",
        "5367a95b7862b633416bfda5852ee33a6cb0fd50a5709db01108a677fc0403ad",
    ),
    // Sadržaj s `52ce161` za testove pravila generatora situacija i otiske (`situations_common::frozen`).
    (
        "content-52ce161/catalog.json",
        "8d539e0e4ae83b1b95da6d4e112db2fea4dc53ee44ba22b6bac4ce2c471d6f66",
    ),
    (
        "content-52ce161/profiles.json",
        "8d32f9778b6eb9c01c4b411b3c49272d08f9d61dc106a8ca4064c76cdefe5343",
    ),
    (
        "content-52ce161/situations.json",
        "dd5137058485e73db65337d89aa001150ff24f7b5cadd054be873fff08a21a00",
    ),
    (
        "content-52ce161/v2/departments.json",
        "d071e744d498993e28012a21984951e8ef77e7f698952c6cd3c4cd08be9c0ed4",
    ),
    (
        "content-52ce161/v2/economy.json",
        "87ded2cdd2c68a304b21db1188c0bf5cead51d4ca3850d47012eca9e45d024b5",
    ),
    (
        "content-52ce161/v2/names.json",
        "6c6c61fe8a1f296d8f3e8ae514a01693182e9b8488d58a93993503fcf678b3d1",
    ),
    (
        "content-52ce161/v2/periods.json",
        "133e43cc4364c705715d517e498efebe49e971d25f84dfcb91ffba8b9af115ed",
    ),
    (
        "content-52ce161/v2/profiles.json",
        "efcd775551764d7d96df292f8c8e66d3ab875f5ef13c1bfed791704752d65bb9",
    ),
    (
        "content-52ce161/v2/situation_templates.json",
        "03c1dad25c2ff1cde6a95ab2dac17ec6702bf640a1537d6d052d83eaeab1f097",
    ),
];

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
    assert_eq!(
        on_disk,
        ["content-52ce161", "step3-snapshot.json"],
        "novi fixture mimo brane"
    );
    let frozen = dir.join("content-52ce161");
    let mut files: Vec<String> = ["", "v2"]
        .iter()
        .flat_map(|sub| std::fs::read_dir(frozen.join(sub)).unwrap())
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| {
            p.strip_prefix(&dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    files.sort();
    let mut sealed: Vec<&str> = SEALED.iter().map(|(n, _)| *n).skip(1).collect();
    sealed.sort_unstable();
    assert_eq!(files, sealed, "datoteka u zamrznutom sadržaju mimo brane");
}
