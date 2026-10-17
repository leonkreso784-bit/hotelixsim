//! L1-13 dio 1 (A1): sadržaj kvara prostora. Predložak je „kvar prostora" (`SituationSpec::space_fault`) kad je vrste
//! `kvar` ili neki odgovor ili neodgovoreno ima `EquipmentRepair`, a nema `external_cause` (Leon 18.10.: točno
//! devet). Ključevi `economy.fault_condition_pts` i `economy.fault_fixed_condition_pts` u repou ne postoje (isključeno
//! do dijela 4), bez ključa se čitaju kao 0, a pravilo isključeno ne mijenja nijedan učinak.
//!
//! Tablica `fault_outcomes_table` ispisuje Leonu učinak na stanje prostora (`cargo test -p content --test
//! v2_situation_fault fault_outcomes_table -- --nocapture`).

use std::collections::BTreeMap;

use content::templates::{Kind, Template, TemplateMechanic};
use content::{Content, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};
use sim_hotel::{Mechanic, Month, SituationOutcome, SituationTemplate};
use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::situations::{DrawnSituation, RoundSituation, Scope};
use sim_hotel_v2::{Period, PeriodKind, Profile, situation_effect};

const NINE: [&str; 9] = [
    "P-KLIMA",
    "P-DOTRAJALO",
    "P-BOJLER",
    "P-BAZEN-PUMPA",
    "P-LOBBY-KAVA",
    "P-DIZALO",
    "P-PRAONICA",
    "P-BRAVE",
    "P-CURENJE-VODE",
];

const EXTERNAL: [&str; 6] = [
    "P-DOBAVLJAC",
    "P-TERMINAL",
    "P-INSPEKCIJA",
    "P-TOPLINSKI-VAL",
    "P-OLUJA",
    "P-NESTANAK-STRUJE",
];

fn repo_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Result<Content, content::ContentError> {
    Snapshot::from_value(v)?.build()
}

fn repo() -> Content {
    build(&repo_value()).unwrap()
}

/// Sadržaj iz repoa s ključevima kvara (Leonovi: −1, +2) i starim ključem hitnog popravka 0 (kao poslije dijela 4).
fn with_keys() -> Content {
    let mut v = repo_value();
    let e = &mut v["v2"]["economy"];
    e["fault_condition_pts"] = json!(100);
    e["fault_fixed_condition_pts"] = json!(200);
    e["repair_urgent_condition_pts"] = json!(0);
    build(&v).unwrap()
}

/// Isti sadržaj bez ključeva kvara i sa starim ključem hitnog popravka 0: osnovica za usporedbu s `with_keys`.
fn off0() -> Content {
    let mut v = repo_value();
    v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(0);
    build(&v).unwrap()
}

fn templates(c: &Content) -> &[Template] {
    &c.situation_templates.as_ref().unwrap().templates
}

/// Nezavisno izvedeno pravilo „kvar prostora" iz sadržaja predloška.
fn is_fault_by_content(t: &Template) -> bool {
    let repairs = t
        .answers
        .iter()
        .flat_map(|a| a.mechanics.iter())
        .chain(t.unanswered.iter())
        .any(|m| matches!(m, TemplateMechanic::EquipmentRepair { .. }));
    (t.kind == Kind::Breakdown || repairs) && !t.external_cause
}

#[test] // 9a
fn space_fault_templates_are_the_nine() {
    let c = repo();
    let tpl = c.situation_templates.as_ref().unwrap();
    let set = tpl.to_core();
    let mut faults: Vec<&str> = set
        .templates
        .iter()
        .filter(|s| s.space_fault)
        .map(|s| s.code.as_str())
        .collect();
    faults.sort_unstable();
    let mut nine = NINE.to_vec();
    nine.sort_unstable();
    assert_eq!(faults, nine, "točno devet kvarova prostora");
    for code in NINE {
        assert!(set.space_fault(code), "{code}: TemplateSet::space_fault");
        assert!(
            tpl.space_fault(code),
            "{code}: SituationTemplates::space_fault"
        );
        // Šifra primjerka (predložak je dio prije prve točke), kao `no_penalty`.
        assert!(tpl.space_fault(&format!("{code}.t03")), "{code}.t03");
        assert!(set.space_fault(code));
    }
    for code in EXTERNAL {
        assert!(
            !set.space_fault(code) && !tpl.space_fault(code),
            "{code} ima vanjski uzrok, nije kvar prostora"
        );
        assert!(
            templates(&c)
                .iter()
                .any(|t| t.code == code && t.external_cause),
            "{code} je u sadržaju s external_cause"
        );
    }
    assert!(!tpl.space_fault("P-NEPOSTOJI") && !set.space_fault("P-NEPOSTOJI"));
    // Pravilo je izvedeno iz sadržaja, ne popis: isti skup daje nezavisan račun.
    for t in templates(&c) {
        assert_eq!(
            set.space_fault(&t.code),
            is_fault_by_content(t),
            "{}: vrsta/EquipmentRepair bez external_cause",
            t.code
        );
    }
    // Sidra S1–S4 nisu predlošci i nisu kvar.
    for s in ["S1", "S2", "S3", "S4"] {
        assert!(!set.space_fault(s) && !tpl.space_fault(s), "{s}");
    }
}

#[test] // 9b
fn no_template_is_fault_and_reward_only() {
    let c = repo();
    let set = c.situation_templates.as_ref().unwrap().to_core();
    for s in &set.templates {
        assert!(
            !(s.space_fault && s.no_penalty),
            "{}: kvar uz „samo nagrada“ bi kršio ‚samo nagrada‘ (−1 snizio bi pokazatelj)",
            s.code
        );
    }
}

#[test] // 9c
fn repo_fault_keys_are_off_until_part_four() {
    let v = repo_value();
    let e = v["v2"]["economy"].as_object().unwrap();
    assert!(
        !e.contains_key("fault_condition_pts"),
        "ključa nema u repou"
    );
    assert!(
        !e.contains_key("fault_fixed_condition_pts"),
        "ključa nema u repou"
    );
    let eco = repo().v2.unwrap().economy;
    assert_eq!(
        (eco.fault_condition_pts, eco.fault_fixed_condition_pts),
        (0, 0)
    );
    assert_eq!(
        eco.repair_urgent_condition_pts, 100,
        "stari ključ ostaje 100 do dijela 4 (ondje 0)"
    );
}

#[test] // 9d
fn missing_fault_keys_read_as_zero() {
    let mut v = repo_value();
    for k in ["fault_condition_pts", "fault_fixed_condition_pts"] {
        v["v2"]["economy"].as_object_mut().unwrap().remove(k);
    }
    let old = v.clone();
    let eco = build(&old).unwrap().v2.unwrap().economy;
    assert_eq!(
        (eco.fault_condition_pts, eco.fault_fixed_condition_pts),
        (0, 0),
        "stari snimak bez ključeva: 0"
    );
    assert_eq!(
        fingerprint_value(&Snapshot::from_value(&old).unwrap().to_value()),
        fingerprint_value(&old),
        "snimak bez ključeva se ne piše s ključevima"
    );
    // Ključ 0 se ne zapisuje (isti zapis i otisak kao prije).
    let mut zero = old.clone();
    zero["v2"]["economy"]["fault_condition_pts"] = json!(0);
    zero["v2"]["economy"]["fault_fixed_condition_pts"] = json!(0);
    assert_eq!(
        Snapshot::from_value(&zero).unwrap().to_value(),
        old,
        "ključevi 0 nestaju iz zapisa"
    );
    // Ključevi ≠ 0 idu tamo i natrag.
    v["v2"]["economy"]["fault_condition_pts"] = json!(100);
    v["v2"]["economy"]["fault_fixed_condition_pts"] = json!(200);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    let eco = s.build().unwrap().v2.unwrap().economy;
    assert_eq!(
        (eco.fault_condition_pts, eco.fault_fixed_condition_pts),
        (100, 200)
    );
    // Izvan 0..=10 000: provjera kataloga odbija.
    for key in ["fault_condition_pts", "fault_fixed_condition_pts"] {
        for bad in [-1, 10_001] {
            let mut w = v.clone();
            w["v2"]["economy"][key] = json!(bad);
            assert!(build(&w).is_err(), "{key} = {bad}");
        }
    }
}

fn instance_of(c: &Content, code: &str, month: Month) -> (SituationTemplate, Period) {
    let tpl = c.situation_templates.as_ref().unwrap();
    let v2 = c.v2.as_ref().unwrap();
    let spec = tpl
        .to_core()
        .templates
        .into_iter()
        .find(|s| s.code == code)
        .unwrap_or_else(|| panic!("nema predloška {code}"));
    let category = v2.base.profile(Profile::C).unwrap().category;
    let p = month_period(month);
    let d = DrawnSituation {
        code: format!("{code}.{}", month.code()),
        template: code.to_string(),
        period: p,
        day: 1,
        occurrence: 1,
        slot: 1,
        kind: spec.kind,
        weight: spec.weight,
        scope: Scope::Hotel,
        major: None,
        department: spec.department.members(v2).and_then(|d| d.first().cloned()),
        roles: BTreeMap::new(),
        numbers: spec
            .numbers
            .iter()
            .map(|(k, n)| {
                let v = match n {
                    sim_hotel_v2::situations::NumberParam::Range(a, _) => *a,
                    sim_hotel_v2::situations::NumberParam::Pick(v) => {
                        v.iter().copied().min().unwrap_or(0)
                    }
                };
                (k.clone(), v)
            })
            .collect(),
    };
    (tpl.instance(&d, &v2.base, category).unwrap(), p)
}

/// Učinak na stanje prostora (bp) jednog ishoda predloška `code` u krugu `p`; `fault` kako ga postavlja igra.
fn equipment_bp(c: &Content, p: &Period, o: SituationOutcome, fault: bool) -> i32 {
    let v2 = c.v2.as_ref().unwrap();
    let category = v2.base.profile(Profile::C).unwrap().category;
    let s = RoundSituation {
        outcome: o,
        space: None,
        fault,
    };
    situation_effect(&s, p, v2, category)
        .unwrap()
        .effects
        .equipment_pts
}

fn rounds(m: Month) -> Vec<Period> {
    let mut out = vec![month_period(m)];
    out.extend(
        sim_hotel_v2::season_periods(PeriodKind::Week)
            .into_iter()
            .find(|p| p.situation_month() == m),
    );
    out
}

#[test] // 4 (karakterizacija nad sadržajem: ključevi 0, svih devet predložaka i oba kruga)
fn fault_rule_is_off_without_keys_for_the_nine() {
    let c = repo();
    for code in NINE {
        for m in [Month::Jun, Month::Aug] {
            let (inst, _) = instance_of(&c, code, m);
            let mut outcomes = vec![SituationOutcome::unanswered(&inst)];
            outcomes.extend(
                inst.answers
                    .iter()
                    .map(|a| SituationOutcome::from_answer(&inst.code, a)),
            );
            for p in rounds(m) {
                for o in &outcomes {
                    assert_eq!(
                        equipment_bp(&c, &p, o.clone(), true),
                        equipment_bp(&c, &p, o.clone(), false),
                        "{code} {m:?} {:?}: ključevi 0 = današnji učinak",
                        p.kind
                    );
                }
            }
        }
    }
}

#[test] // 8
fn fault_outcomes_table() {
    let c = with_keys();
    let off = off0();
    println!("\nL1-13-1 tablica: učinak situacije na stanje prostora (bodovi, 100 bp = 1)");
    println!(
        "{:<12} {:<6} {:>8} {:>10} {:>10}",
        "predložak", "krug", "šutnja", "hitni q3", "hitni q5"
    );
    let pts = |bp: i32| f64::from(bp) / 100.0;
    for code in ["P-BRAVE", "P-KLIMA"] {
        let (inst, _) = instance_of(&c, code, Month::Jul);
        let urgent: Vec<_> = inst
            .answers
            .iter()
            .filter(|a| {
                a.mechanics
                    .iter()
                    .any(|m| matches!(m, Mechanic::EquipmentRepair { urgent: true }))
            })
            .collect();
        assert!(!urgent.is_empty(), "{code} ima odgovor s hitnim popravkom");
        for p in rounds(Month::Jul) {
            let silent = SituationOutcome::unanswered(&inst);
            let s_on = equipment_bp(&c, &p, silent.clone(), true);
            let s_off = equipment_bp(&off, &p, silent, false);
            assert_eq!(s_on - s_off, -100, "{code} {:?}: šutnja −1", p.kind);
            assert_eq!(pts(s_on).round(), -1.0, "{code} {:?}: šutnja", p.kind);
            for a in &urgent {
                let at = |q: u8| SituationOutcome {
                    code: inst.code.clone(),
                    mechanics: a.mechanics.clone(),
                    quality: q,
                };
                let (q3, q5) = (
                    equipment_bp(&c, &p, at(3), true),
                    equipment_bp(&c, &p, at(5), true),
                );
                let (o3, o5) = (
                    equipment_bp(&off, &p, at(3), false),
                    equipment_bp(&off, &p, at(5), false),
                );
                println!(
                    "{:<12} {:<6} {:>8.2} {:>10.2} {:>10.2}",
                    code,
                    p.kind.code(),
                    pts(s_on),
                    pts(q3),
                    pts(q5)
                );
                assert_eq!(
                    q3 - o3,
                    -100,
                    "{code} {:?}: popravak uz 3 ostaje −1",
                    p.kind
                );
                assert_eq!(
                    q5 - o5,
                    100,
                    "{code} {:?}: popravak uz 5 daje −1 + 2",
                    p.kind
                );
                assert_eq!(q5 - q3, 200, "{code} {:?}", p.kind);
                assert_eq!(
                    (pts(q3).round(), pts(q5).round()),
                    (-1.0, 1.0),
                    "{code} {:?}: −1 / +1",
                    p.kind
                );
            }
        }
    }
}

#[test] // 8 (uz Leonov „kvar bez popravka": šutnja kruga mjeseca i tjedna jednako −1)
fn week_and_month_fault_effect_is_the_same() {
    let c = with_keys();
    let off = off0();
    let (inst, _) = instance_of(&c, "P-KLIMA", Month::Jul);
    let silent = SituationOutcome::unanswered(&inst);
    let month = month_period(Month::Jul);
    let week = period_by_code(PeriodKind::Week, "t02").unwrap();
    let d = |p: &Period| {
        equipment_bp(&c, p, silent.clone(), true) - equipment_bp(&off, p, silent.clone(), false)
    };
    assert_eq!((d(&month), d(&week)), (-100, -100));
}
