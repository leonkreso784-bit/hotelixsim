//! L1-13 dio 3 (A7): dodatna mjesta u krugu pune samo situacije s gostima. Broj situacija već raste s očekivanom
//! popunjenošću kruga (`SituationCounts`: niska / srednja / visoka), ali je dodatna mjesta punio bilo koji predložak.
//! Uz ključ `TemplateSet::guest_extra_slots` (sadržaj `situation_templates.json`, u repou UKLJUČEN od dijela 4)
//! mjesta iznad `low` (`SituationCounts::levels().0`) smiju popuniti samo predlošci s gostima
//! (`SituationSpec::guest_related`: uloga `from: guest` ili odjel `*guest_facing`, 27 od 55), pa je očekivani broj
//! ostalih isti na svakoj popunjenosti. Prva mjesta (sidra, kolektivna, veća) i ukupni broj ostaju kakvi jesu.
//! Isključen ključ je TOČNO današnje izvlačenje (zlatni otisak ispod, izmjeren prije izmjene; od dijela 4 nad
//! pravilima pojave devet kvarova prostora kakva su bila na c1ef169, jer ih kalibracija A2 mijenja).

mod situations_common;

use content::{Content, Snapshot, repo_dir};
use serde_json::{Value, json};
use sim_hotel_v2::situations::{
    DeptTarget, DrawnSituation, Gender, RoleSource, RoleSpec, Scope, SituationCounts,
    SituationSpec, TemplateSet, expected_occupancy_bp,
};
use sim_hotel_v2::{Catalog, PeriodKind, Profile, season_periods};
use situations_common::{Hotel, anchors_for, general, indices, live, round, start};

const PROFILES: [Profile; 3] = [Profile::A, Profile::B, Profile::C];
const KINDS: [PeriodKind; 2] = [PeriodKind::Week, PeriodKind::Month];

/// Šifre predložaka „s gostima" (27 od 55), izvedene iz `situation_templates.json` neovisno o jezgri.
const GUEST_RELATED: [&str; 27] = [
    "P-KLIMA",
    "P-DOTRAJALO",
    "P-NOVI-LJUDI",
    "P-ROOFTOP-BUKA",
    "P-LEZALJKE",
    "P-SPA-TERMIN",
    "P-ALERGIJA",
    "P-RANI-DOLAZAK",
    "P-NESTALA-STVAR",
    "P-POSEBNA-PREHRANA",
    "P-OSPORENA-NAPLATA",
    "P-LJUBIMAC",
    "P-BUCNA-SOBA",
    "P-POHVALA",
    "P-DIZALO",
    "P-OVERBOOKING",
    "P-RED-NA-RECEPCIJI",
    "P-PIJANI-GOST",
    "P-PARKING-STETA",
    "P-UCJENA-RECENZIJOM",
    "P-DIJETE-IZGUBLJENO",
    "P-RACUN-GRESKA",
    "P-CURENJE-VODE",
    "P-PROSIDBA",
    "P-PRISTUPACNOST",
    "P-NETOCNA-RECENZIJA",
    "P-TERMINAL",
];

/// Otisak današnjeg izvlačenja (`flag_off_is_todays_draw`), izmjeren na 532b857 PRIJE ikakve izmjene izvlačenja:
/// 3 profila × tjedna i mjesečna sezona × sva 200 sjemena, sidra kao u igri, prazna povijest.
const TODAYS_DRAW: u64 = 0x331e_2f7b_2294_8233;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Level {
    Low,
    Mid,
    High,
}

/// Razina očekivane popunjenosti kruga: ispod 75 % niska, 75–89 % srednja, od 90 % visoka (ADR-042 t. 13).
fn level(occ_bp: u16) -> Level {
    match occ_bp {
        0..7_500 => Level::Low,
        7_500..9_000 => Level::Mid,
        _ => Level::High,
    }
}

fn with_flag(mut set: TemplateSet, on: bool) -> TemplateSet {
    set.guest_extra_slots = on;
    set
}

fn spec<'a>(set: &'a TemplateSet, d: &DrawnSituation) -> &'a SituationSpec {
    set.templates
        .iter()
        .find(|t| t.code == d.template)
        .expect("izvučen predložak postoji u skupu")
}

/// Jedan krug (hotel i profil iz `start`, prazna povijest, sidra kao u igri).
fn draw(
    (cat, set): (&Catalog, &TemplateSet),
    profile: Profile,
    p: &sim_hotel_v2::Period,
    seed: u64,
) -> (Vec<DrawnSituation>, usize) {
    let state = start(cat, profile);
    let hotel = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    let anchors = anchors_for(p.kind, p.index);
    let out = round(cat, set, &hotel, p, &[], seed, &anchors);
    (out, anchors.len())
}

/// Razina popunjenosti kruga `p` za profil (početno stanje profila, kao u `draw`).
fn level_of(cat: &Catalog, profile: Profile, p: &sim_hotel_v2::Period) -> Level {
    level(expected_occupancy_bp(&start(cat, profile), p, cat).unwrap())
}

/// Najmanja razina `low` zadanog broja za vrstu kruga.
fn low_of(kind: PeriodKind) -> usize {
    usize::from(SituationCounts::default_for(kind).levels().0)
}

#[test] // 1
fn extra_slots_above_low_are_guest_related() {
    let (cat, set) = live();
    let set = with_flag(set, true);
    let mut high_rounds = 0_u64;
    let mut extra_filled = 0_u64;
    for kind in KINDS {
        let low = low_of(kind);
        for profile in PROFILES {
            for p in season_periods(kind) {
                if level_of(&cat, profile, &p) != Level::High {
                    continue;
                }
                for seed in 0..1_000_u64 {
                    let (out, anchors) = draw((&cat, &set), profile, &p, seed);
                    high_rounds += 1;
                    // Prva mjesta: sidra, kolektivna i veća. Obične situacije iznad njih smiju biti negostinjske
                    // samo dok ih ima mjesta ispod `low`.
                    let pre = anchors
                        + out
                            .iter()
                            .filter(|d| d.scope == Scope::All || d.major.is_some())
                            .count();
                    let ordinary: Vec<&DrawnSituation> = out
                        .iter()
                        .filter(|d| d.scope == Scope::Hotel && d.major.is_none())
                        .collect();
                    let others = ordinary
                        .iter()
                        .filter(|d| !spec(&set, d).guest_related())
                        .count();
                    assert!(
                        others <= low.saturating_sub(pre),
                        "{} {} sjeme {seed}: {others} negostinjskih običnih uz {pre} prvih mjesta i low {low}: {:?}",
                        profile.code(),
                        p.code(),
                        ordinary.iter().map(|d| &d.template).collect::<Vec<_>>()
                    );
                    if pre + ordinary.len() > low {
                        extra_filled += 1;
                    }
                }
            }
        }
    }
    assert!(high_rounds > 0, "ima krugova s visokom popunjenošću");
    assert!(
        extra_filled * 2 > high_rounds,
        "dodatna mjesta se stvarno pune gostinjskim: {extra_filled} od {high_rounds} krugova"
    );
}

/// Prosjek po krugu (gostinjskih, ostalih) za svaku razinu, preko profila, krugova i 300 sjemena.
fn averages((cat, set): (&Catalog, &TemplateSet), kind: PeriodKind) -> [(f64, f64); 3] {
    let mut sums = [(0_u64, 0_u64, 0_u64); 3];
    for profile in PROFILES {
        for p in season_periods(kind) {
            let i = match level_of(cat, profile, &p) {
                Level::Low => 0,
                Level::Mid => 1,
                Level::High => 2,
            };
            for seed in 0..300_u64 {
                let (out, _) = draw((cat, set), profile, &p, seed);
                let guest = out.iter().filter(|d| spec(set, d).guest_related()).count() as u64;
                sums[i].0 += guest;
                sums[i].1 += out.len() as u64 - guest;
                sums[i].2 += 1;
            }
        }
    }
    sums.map(|(g, o, n)| {
        assert!(n > 0, "svaka razina popunjenosti ima krugova");
        (g as f64 / n as f64, o as f64 / n as f64)
    })
}

#[test] // 2
fn more_occupancy_never_fewer_guest_situations() {
    let (cat, set) = live();
    let set = with_flag(set, true);
    for kind in KINDS {
        let [low, mid, high] = averages((&cat, &set), kind);
        assert!(
            high.0 >= mid.0 && mid.0 >= low.0,
            "{kind:?}: gostinjskih po krugu niska {:.2} srednja {:.2} visoka {:.2}",
            low.0,
            mid.0,
            high.0
        );
        assert!(
            high.0 > low.0,
            "{kind:?}: visoka ima više gostinjskih od niske"
        );
    }
}

/// Isti krug, isti hotel, ista sjemena; mijenja se samo broj mjesta (`low` ostaje): ostale se ne smiju množiti.
/// Kontrolirano je namjerno: usporedba niske i visoke popunjenosti u STVARNIM krugovima miješa sezonu (različiti
/// uvjeti i sidra), pa bi razlika od ~0,25 bila posljedica kalendara, ne pravila (izmjereno privremenom izvedbom).
#[test] // 3
fn other_situations_same_on_average() {
    let (cat, set) = live();
    for kind in KINDS {
        let (low, small, big) = match kind {
            PeriodKind::Week => (3, 3, 5),
            PeriodKind::Month => (4, 4, 6),
        };
        let count = |n: u8| SituationCounts::new(low, n, n).unwrap();
        let mut diffs = [0.0_f64; 2];
        for (i, on) in [false, true].into_iter().enumerate() {
            let mut few = with_flag(set.clone(), on);
            few.counts = Some(count(small));
            let mut many = with_flag(set.clone(), on);
            many.counts = Some(count(big));
            let (mut few_others, mut many_others, mut rounds) = (0_u64, 0_u64, 0_u64);
            for profile in PROFILES {
                for p in season_periods(kind) {
                    // Niska popunjenost ostaje na `low` u oba skupa, pa ne govori ništa o pravilu.
                    if level_of(&cat, profile, &p) == Level::Low {
                        continue;
                    }
                    for seed in 0..300_u64 {
                        for (set, total) in [(&few, &mut few_others), (&many, &mut many_others)] {
                            let (out, _) = draw((&cat, set), profile, &p, seed);
                            *total +=
                                out.iter().filter(|d| !spec(set, d).guest_related()).count() as u64;
                        }
                        rounds += 1;
                    }
                }
            }
            diffs[i] = (many_others as f64 - few_others as f64) / rounds as f64;
        }
        // Isključeno: dodatna mjesta pune i ostale (inače test ne bi ništa dokazivao).
        assert!(
            diffs[0] > 0.5,
            "{kind:?} isključeno: razlika ostalih po krugu {:.3}",
            diffs[0]
        );
        // Uključeno: prosjek ostalih po krugu isti, razlika najviše 0,15 🟨 (u prosjeku, ne strogo po krugu).
        assert!(
            diffs[1].abs() <= 0.15,
            "{kind:?} uključeno: razlika ostalih po krugu {:.3}",
            diffs[1]
        );
    }
}

/// Otisak izlaza izvlačenja (FNV-1a): predložak, krug, odjel, dan, redni broj, utor, mjesta i imena uloga.
fn fingerprint(cat: &Catalog, set: &TemplateSet) -> (u64, usize) {
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    let mut n = 0;
    let mut eat = |s: &str| {
        for b in s.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for kind in KINDS {
        for profile in PROFILES {
            for p in season_periods(kind) {
                for seed in 0..200_u64 {
                    for s in draw((cat, set), profile, &p, seed).0 {
                        n += 1;
                        eat(&format!(
                            "{}|{}|{:?}|{}|{}|{}|{:?}|{:?};",
                            s.template,
                            p.index,
                            s.department,
                            s.day,
                            s.occurrence,
                            s.slot,
                            s.numbers,
                            s.roles.keys().collect::<Vec<_>>()
                        ));
                    }
                    eat("|");
                }
            }
        }
    }
    (h, n)
}

/// Pravila pojave (`when`, `chance`) devet kvarova prostora na c1ef169 (prije kalibracije A2, L1-13 dio 4): zlatni
/// otisak `TODAYS_DRAW` je izmjeren nad njima. Dio 4 mijenja samo čestoću tih devet (ostalo isto), pa otisak nad
/// repoom s ovim pravilima i dalje mjeri samo kod izvlačenja.
const FAULT_DRAW_RULES_C1EF169: [(&str, &str); 9] = [
    (
        "P-KLIMA",
        r#"{"when":{"months":["jun","jul","aug"],"occupancy_bp":{"at_least":6000},"dept_condition_pts":{"below":6500}},"chance":{"base":3,"boost":[{"when":{"dept_condition_pts":{"below":5000}},"times":3}]}}"#,
    ),
    (
        "P-DOTRAJALO",
        r#"{"when":{"dept_condition_pts":{"below":5500}},"chance":{"base":2,"boost":[{"when":{"dept_condition_pts":{"below":4000}},"times":3}]}}"#,
    ),
    (
        "P-BOJLER",
        r#"{"when":{"months":["jul","aug","sep"],"dept_condition_pts":{"below":6500}},"chance":{"base":2,"boost":[{"when":{"dept_condition_pts":{"below":4500}},"times":3}]}}"#,
    ),
    (
        "P-BAZEN-PUMPA",
        r#"{"when":{"dept_condition_pts":{"below":6000}},"chance":{"base":2,"boost":[{"when":{"dept_condition_pts":{"below":4500}},"times":3}]}}"#,
    ),
    (
        "P-LOBBY-KAVA",
        r#"{"when":{"dept_condition_pts":{"below":6000}},"chance":{"base":2,"boost":[{"when":{"dept_load_hundredths":{"at_least":200}},"times":2}]}}"#,
    ),
    (
        "P-DIZALO",
        r#"{"when":{"dept_condition_pts":{"below":8000}},"chance":{"base":2}}"#,
    ),
    (
        "P-PRAONICA",
        r#"{"when":{"dept_condition_pts":{"below":6500}},"chance":{"base":2}}"#,
    ),
    (
        "P-BRAVE",
        r#"{"when":{"dept_condition_pts":{"below":8000}},"chance":{"base":2}}"#,
    ),
    (
        "P-CURENJE-VODE",
        r#"{"when":{"dept_condition_pts":{"below":8000}},"chance":{"base":2}}"#,
    ),
];

/// Repo s pravilima pojave devet kvarova s c1ef169 (`FAULT_DRAW_RULES_C1EF169`).
fn live_c1ef169() -> (Catalog, TemplateSet) {
    let mut v = repo_value();
    for (code, rule) in FAULT_DRAW_RULES_C1EF169 {
        let rule: Value = serde_json::from_str(rule).unwrap();
        let list = v["v2"]["situation_templates"]["templates"]
            .as_array_mut()
            .unwrap();
        let t = list
            .iter_mut()
            .find(|t| t["code"] == code)
            .unwrap_or_else(|| panic!("nema predloška {code}"));
        t["when"] = rule["when"].clone();
        t["chance"] = rule["chance"].clone();
    }
    let c = build(&v);
    (c.v2.unwrap(), c.situation_templates.unwrap().to_core())
}

#[test] // 4
fn flag_off_is_todays_draw() {
    let (cat, set) = live_c1ef169();
    let (off, n) = fingerprint(&cat, &with_flag(set.clone(), false));
    assert!(n > 10_000, "otisak pokriva dovoljno primjeraka ({n})");
    assert_eq!(
        off, TODAYS_DRAW,
        "isključen ključ mijenja izvlačenje: {off:#018x}"
    );
    // Karakterizacija nije tautologija: uključen ključ stvarno mijenja izvlačenje.
    let (on, _) = fingerprint(&cat, &with_flag(set, true));
    assert_ne!(on, off, "uključen ključ ne mijenja ništa");
}

fn repo_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Content {
    Snapshot::from_value(v).unwrap().build().unwrap()
}

#[test] // 5
fn guest_related_templates_are_27() {
    let (_, set) = live();
    let mut got: Vec<&str> = set
        .templates
        .iter()
        .filter(|t| t.guest_related())
        .map(|t| t.code.as_str())
        .collect();
    got.sort_unstable();
    let mut want = GUEST_RELATED.to_vec();
    want.sort_unstable();
    assert_eq!(got, want, "27 od {} predložaka", set.templates.len());
    assert_eq!(set.templates.len(), 55);

    // Neovisna provjera iz sadržaja: uloga `from: guest` ili odjel `*guest_facing`.
    let c = build(&repo_value());
    let tpl = c.situation_templates.unwrap();
    for t in &tpl.templates {
        let by_content = t.department == "*guest_facing"
            || t.roles
                .values()
                .any(|r| r.from == content::templates::RoleSource::Guest);
        assert_eq!(
            by_content,
            GUEST_RELATED.contains(&t.code.as_str()),
            "{}",
            t.code
        );
    }
}

#[test] // 5 (pravilo na izmišljenim predlošcima: uloga ili odjel, ništa drugo)
fn guest_related_is_role_or_guest_facing_department() {
    let role = |source| RoleSpec {
        source,
        gender: Gender::M,
        types: vec![],
    };
    let mut t: SituationSpec = general(
        "P-X",
        sim_hotel_v2::situations::Kind::Request,
        sim_hotel_v2::situations::Repeat {
            max_per_season: 1,
            gap_rounds: 1,
        },
    );
    assert!(t.guest_related(), "uloga gost");
    t.roles.insert("gost".to_string(), role(RoleSource::Staff));
    assert!(!t.guest_related(), "samo osoblje, odjel cijeli hotel");
    t.roles
        .insert("partner".to_string(), role(RoleSource::Partner));
    assert!(!t.guest_related(), "osoblje i partner nisu gosti");
    t.department = DeptTarget::GuestFacing;
    assert!(t.guest_related(), "odjel uz goste");
    t.department = DeptTarget::Staffed;
    assert!(!t.guest_related(), "odjel s osobljem nije uz goste");
    t.roles.insert("drugi".to_string(), role(RoleSource::Guest));
    assert!(t.guest_related(), "bilo koja uloga gosta");
}

/// L1-13 dio 4 (test čuvara 6): repo uključuje ključ (`situation_templates.json`), i on dolazi do jezgre.
#[test]
fn repo_guest_extra_slots_on() {
    let v = repo_value();
    assert_eq!(
        v["v2"]["situation_templates"].get("guest_extra_slots"),
        Some(&json!(true)),
        "guest_extra_slots: true u content/hotel/v2/situation_templates.json"
    );
    let c = build(&v);
    let tpl = c.situation_templates.as_ref().unwrap();
    assert!(tpl.guest_extra_slots);
    assert!(tpl.to_core().guest_extra_slots);
}

#[test] // 5
fn missing_guest_flag_reads_false() {
    // Stari snimak (prije dijela 4) nema ključa.
    let mut v = repo_value();
    v["v2"]["situation_templates"]
        .as_object_mut()
        .unwrap()
        .remove("guest_extra_slots");
    let c = build(&v);
    let tpl = c.situation_templates.as_ref().unwrap();
    assert!(!tpl.guest_extra_slots);
    assert!(!tpl.to_core().guest_extra_slots);

    // Uključen ključ ide tamo i natrag i dolazi do jezgre.
    let mut on = v.clone();
    on["v2"]["situation_templates"]["guest_extra_slots"] = json!(true);
    let s = Snapshot::from_value(&on).unwrap();
    assert_eq!(s.to_value(), on, "ključ true ide tamo i natrag");
    let c = s.build().unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    assert!(tpl.guest_extra_slots);
    assert!(tpl.to_core().guest_extra_slots);

    // Snimak bez ključa se ne piše s ključem.
    assert_eq!(Snapshot::from_value(&v).unwrap().to_value(), v);
}
