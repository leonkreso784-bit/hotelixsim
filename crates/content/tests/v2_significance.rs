//! L1-13 dio 2: razine značaja u sadržaju. (a) razina se IZVODI iz `weight` predloška (4 ključna, 3 važna, 1–2
//! svakodnevna): u repou 9 / 17 / 29 od 55 predložaka, bez novog polja u `situation_templates.json`;
//! (b) ključ `economy.significance_bp {key, important, everyday}` u repou ne postoji (sve tri 10 000 do dijela 4,
//! pečat i otisak bajt-isti), bez ključa se čita kao 10 000, a sva tri 10 000 se ne zapisuju; (c) izvan 1 000..=30 000
//! provjera kataloga odbija, a nepoznato polje u objektu odbija učitavanje.

use content::{Content, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};
use sim_hotel_v2::situations::Significance;

fn repo_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Result<Content, content::ContentError> {
    Snapshot::from_value(v)?.build()
}

fn levels(c: &Content) -> (u16, u16, u16) {
    let s = &c.v2.as_ref().unwrap().economy.significance_bp;
    (s.key, s.important, s.everyday)
}

fn with(levels: Value) -> Value {
    let mut v = repo_value();
    v["v2"]["economy"]["significance_bp"] = levels;
    v
}

#[test] // 3
fn levels_cover_all_templates_9_17_29() {
    let c = build(&repo_value()).unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    let mut n = [0_usize; 3];
    for t in &tpl.templates {
        n[match Significance::from_weight(t.weight) {
            Significance::Key => 0,
            Significance::Important => 1,
            Significance::Everyday => 2,
        }] += 1;
    }
    assert_eq!(n, [9, 17, 29], "ključnih / važnih / svakodnevnih");
    assert_eq!(n.iter().sum::<usize>(), 55);
    // Isto i na jezgrenom skupu (to čita izvlačenje): razina se izvodi iz iste `weight`.
    let set = tpl.to_core();
    let core: Vec<usize> = [4_u8, 3, 2, 1]
        .into_iter()
        .map(|w| set.templates.iter().filter(|s| s.weight == w).count())
        .collect();
    assert_eq!(core[0], 9);
    assert_eq!(core[1], 17);
    assert_eq!(core[2] + core[3], 29);
}

#[test] // 3 (razina nije novo polje predloška: situation_templates.json se ne dira)
fn templates_have_no_significance_field() {
    let path = repo_dir().join("v2/situation_templates.json");
    let text = std::fs::read_to_string(path).unwrap();
    assert!(
        !text.contains("significance"),
        "razina se izvodi iz weight, bez polja u predlošku"
    );
}

#[test] // 3
fn repo_significance_keys_are_neutral_until_part_four() {
    let v = repo_value();
    assert!(
        v["v2"]["economy"].get("significance_bp").is_none(),
        "bez ključa u economy.json do dijela 4"
    );
    assert_eq!(levels(&build(&v).unwrap()), (10_000, 10_000, 10_000));
}

#[test] // 3
fn missing_significance_reads_neutral() {
    let old = repo_value();
    assert_eq!(levels(&build(&old).unwrap()), (10_000, 10_000, 10_000));
    assert_eq!(
        fingerprint_value(&Snapshot::from_value(&old).unwrap().to_value()),
        fingerprint_value(&old),
        "snimak bez ključa se ne piše s ključem"
    );
    // Sva tri 10 000 se ne zapisuju: isti zapis i otisak kao prije.
    let full = with(json!({ "key": 10_000, "important": 10_000, "everyday": 10_000 }));
    assert_eq!(
        Snapshot::from_value(&full).unwrap().to_value(),
        old,
        "neutralni ključevi nestaju iz zapisa"
    );
    assert_eq!(levels(&build(&full).unwrap()), (10_000, 10_000, 10_000));
}

#[test] // 3
fn significance_round_trips_and_is_range_checked() {
    let v = with(json!({ "key": 15_000, "important": 10_000, "everyday": 6_000 }));
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v, "ključ ≠ neutralan ide tamo i natrag");
    assert_eq!(levels(&s.build().unwrap()), (15_000, 10_000, 6_000));
    for edge in [1_000, 30_000] {
        let w = with(json!({ "key": edge, "important": edge, "everyday": edge }));
        assert_eq!(levels(&build(&w).unwrap()), (edge, edge, edge), "{edge}");
    }
}

#[test] // 3
fn significance_out_of_range_is_rejected() {
    for bad in [0, 999, 30_001, 65_535] {
        for field in ["key", "important", "everyday"] {
            let mut o = json!({ "key": 15_000, "important": 10_000, "everyday": 6_000 });
            o[field] = json!(bad);
            assert!(build(&with(o)).is_err(), "{field} = {bad}");
        }
    }
}

#[test] // 3
fn significance_rejects_unknown_fields() {
    let o = json!({ "key": 15_000, "important": 10_000, "everyday": 6_000, "extra": 1 });
    assert!(build(&with(o)).is_err());
}
