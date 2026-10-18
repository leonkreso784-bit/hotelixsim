//! L1-13 dio 1b: ključ `economy.click_gain_bp` (dio dobiti koji klik na ponuđeni odgovor zadržava, bp). Bez ključa
//! se čita kao 10 000 (stari snimak = isti obračun) i 10 000 se ne zapisuje (isti zapis i otisak kao prije, pa je
//! pečat bajt-isti); izvan 0..=10 000 provjera kataloga odbija. U repou je ključ uključen na izmjerenu vrijednost
//! (ispod 10 000, u koracima od 1 000): to je jedina promjena sadržaja ovog dijela.

use content::{Content, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};

fn repo_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Result<Content, content::ContentError> {
    Snapshot::from_value(v)?.build()
}

fn gain(c: &Content) -> u16 {
    c.v2.as_ref().unwrap().economy.click_gain_bp
}

fn without_key() -> Value {
    let mut v = repo_value();
    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("click_gain_bp");
    v
}

#[test] // 1
fn repo_click_gain_is_measured_and_below_full() {
    let v = repo_value();
    let key = v["v2"]["economy"]
        .get("click_gain_bp")
        .expect("click_gain_bp je u content/hotel/v2/economy.json");
    let bp = key.as_u64().expect("broj");
    assert!(
        (1_000..10_000).contains(&bp) && bp % 1_000 == 0,
        "izmjereno u koracima od 1 000, ispod 10 000: {bp}"
    );
    assert_eq!(u64::from(gain(&build(&v).unwrap())), bp);
}

#[test] // 2
fn missing_click_gain_reads_as_full() {
    let old = without_key();
    assert_eq!(gain(&build(&old).unwrap()), 10_000, "stari snimak: 10 000");
    assert_eq!(
        fingerprint_value(&Snapshot::from_value(&old).unwrap().to_value()),
        fingerprint_value(&old),
        "snimak bez ključa se ne piše s ključem"
    );
    // 10 000 se ne zapisuje: isti zapis i otisak kao prije.
    let mut full = old.clone();
    full["v2"]["economy"]["click_gain_bp"] = json!(10_000);
    assert_eq!(
        Snapshot::from_value(&full).unwrap().to_value(),
        old,
        "10 000 nestaje iz zapisa"
    );
    assert_eq!(gain(&build(&full).unwrap()), 10_000);
}

#[test] // 3
fn click_gain_round_trips_and_is_range_checked() {
    let mut v = without_key();
    v["v2"]["economy"]["click_gain_bp"] = json!(7_500);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v, "ključ ≠ 10 000 ide tamo i natrag");
    assert_eq!(gain(&s.build().unwrap()), 7_500);
    v["v2"]["economy"]["click_gain_bp"] = json!(0);
    assert_eq!(
        Snapshot::from_value(&v).unwrap().to_value(),
        v,
        "0 je važeća vrijednost i zapisuje se"
    );
    assert_eq!(gain(&build(&v).unwrap()), 0);
    for bad in [10_001, 65_535] {
        let mut w = v.clone();
        w["v2"]["economy"]["click_gain_bp"] = json!(bad);
        assert!(build(&w).is_err(), "click_gain_bp = {bad}");
    }
}
