//! L1-13 dio 1 (A1): sadržaj kvara prostora. Predložak je „kvar prostora" (`SituationSpec::space_fault`) kad je vrste
//! `kvar` ili neki odgovor ili neodgovoreno ima `EquipmentRepair`, a nema `external_cause` (Leon 18.10.: točno
//! devet). Ključevi `economy.fault_condition_pts` i `economy.fault_fixed_condition_pts` su u repou od dijela 4
//! uključeni, stari ključ hitnog popravka obrisan; bez ključa se čitaju kao 0, a pravilo isključeno ne mijenja nijedan
//! učinak (stari snimci). Leonove odluke 18.10. (četvrta anketa): **K1** — ključevi su u jedinici bodova situacije i
//! množe se faktorom kruga i razine kao ostali bodovi (mjesečno važna situacija ostaje −1 / +2, tjedno ≈ −0,34 / +0,68);
//! **L1** — bonus popravka već uz ocjenu 3.
//!
//! Tablica `fault_outcomes_table` ispisuje Leonu učinak na stanje prostora (`cargo test -p content --test
//! v2_situation_fault fault_outcomes_table -- --nocapture`).

use std::collections::BTreeMap;

use content::templates::{Kind, Template, TemplateMechanic};
use content::{Content, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};
use sim_hotel::{Mechanic, Month, SituationOutcome, SituationTemplate};
use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::situations::{DrawnSituation, RoundSituation, Scope, Significance};
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

/// Ključevi kvara u testu, u bodovima situacije (K1): 100 × 10 000 / 350 ≈ 2 857 i 5 714 — uz faktor 350 točno −1 / +2.
const FAULT_KEY: i64 = 2_857;
const FIXED_KEY: i64 = 5_714;

/// Sadržaj iz repoa s ključevima kvara iz testa i starim ključem hitnog popravka 0 (kao poslije dijela 4).
fn with_keys() -> Content {
    let mut v = repo_value();
    let e = &mut v["v2"]["economy"];
    e["fault_condition_pts"] = json!(FAULT_KEY);
    e["fault_fixed_condition_pts"] = json!(FIXED_KEY);
    e["repair_urgent_condition_pts"] = json!(0);
    build(&v).unwrap()
}

/// Najbliži cijeli broj `key × f × sig / 10^8`, polovina od nule (kao `c::div_round`): jedno zaokruživanje.
fn scaled(key: i64, f: i64, sig: i64) -> i32 {
    let n = key * f * sig;
    let d = 100_000_000_i64;
    let q = (2 * n.abs() + d) / (2 * d);
    i32::try_from(if n < 0 { -q } else { q }).unwrap()
}

/// Faktor kruga sadržaja za vrstu kruga `p`, bp.
fn factor(c: &Content, p: &Period) -> i64 {
    i64::from(
        c.v2.as_ref()
            .unwrap()
            .coeffs(p.kind)
            .unwrap()
            .situation_effect_bp,
    )
}

/// Faktor razine sadržaja, bp.
fn level_bp(c: &Content, level: Significance) -> i64 {
    let s = &c.v2.as_ref().unwrap().economy.significance_bp;
    i64::from(match level {
        Significance::Key => s.key,
        Significance::Important => s.important,
        Significance::Everyday => s.everyday,
    })
}

/// Vrijednost iz repoa bez ključeva kvara (kao stari snimak prije dijela 4).
fn without_fault_keys() -> Value {
    let mut v = repo_value();
    for k in ["fault_condition_pts", "fault_fixed_condition_pts"] {
        v["v2"]["economy"].as_object_mut().unwrap().remove(k);
    }
    v
}

/// Repo bez ključeva kvara: pravilo isključeno.
fn no_keys() -> Content {
    build(&without_fault_keys()).unwrap()
}

/// Isti sadržaj bez ključeva kvara i sa starim ključem hitnog popravka 0: osnovica za usporedbu s `with_keys`.
fn off0() -> Content {
    let mut v = without_fault_keys();
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

#[test] // 9c (L1-13 dio 4, test čuvara 6; runda 2 po Leonovoj odluci 18.10. K1 i L1): repo uključuje kvar prostora i
// briše stari ključ hitnog popravka (privremeni +1 iz ADR-050 → 0; stari snimci ga i dalje čitaju). Ključevi su u
// bodovima situacije: učinak važne situacije mjesečno je −1 / popravak uz ocjenu 3 +2 (neto +1), tjedno oko trećine
// toga (anketa: ≈ −0,34 / +0,68; prozor tjednog faktora 105–115 iz dijagnoze daje 0,28–0,40).
fn repo_fault_keys_are_on() {
    let v = repo_value();
    let e = v["v2"]["economy"].as_object().unwrap();
    for k in ["fault_condition_pts", "fault_fixed_condition_pts"] {
        assert!(
            e.get(k).and_then(Value::as_i64).is_some_and(|x| x > 0),
            "{k}: ključ uključen u economy.json"
        );
    }
    assert!(
        !e.contains_key("repair_urgent_condition_pts"),
        "stari ključ hitnog popravka se briše iz economy.json (0)"
    );
    let c = repo();
    let v2 = c.v2.as_ref().unwrap();
    assert_eq!(v2.economy.repair_urgent_condition_pts, 0);
    let category = v2.base.profile(Profile::C).unwrap().category;
    let at = |p: &Period, mechanics: Vec<Mechanic>, q: u8| {
        let s = RoundSituation {
            outcome: SituationOutcome {
                code: "P-BRAVE.a".into(),
                mechanics,
                quality: q,
            },
            space: None,
            fault: true,
            click: false,
            significance: Significance::Important,
        };
        situation_effect(&s, p, v2, category)
            .unwrap()
            .effects
            .equipment_pts
    };
    let urgent = || vec![Mechanic::EquipmentRepair { urgent: true }];
    let month = month_period(Month::Jul);
    assert_eq!(at(&month, vec![], 1), -100, "mjesečno: kvar −1");
    for q in [3, 4, 5] {
        assert_eq!(
            at(&month, urgent(), q),
            100,
            "mjesečno: hitni popravak uz ocjenu {q}: −1 + 2"
        );
    }
    assert_eq!(at(&month, urgent(), 2), -100, "ocjena 2: bez bonusa");
    let week = period_by_code(PeriodKind::Week, "t02").unwrap();
    let lose = -at(&week, vec![], 1);
    assert!(
        (28..=40).contains(&lose),
        "tjedno: kvar ≈ −0,34 (faktor kruga), a ne −1: {lose}"
    );
    let net = at(&week, urgent(), 3);
    assert!(
        (lose - 1..=lose + 1).contains(&net),
        "tjedno: popravak ≈ +2 × isto, neto ≈ +{lose}: {net}"
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

/// Učinak na stanje prostora (bp) jednog ishoda predloška u krugu `p`; `fault` kako ga postavlja igra, razina važna.
fn equipment_bp(c: &Content, p: &Period, o: SituationOutcome, fault: bool) -> i32 {
    equipment_at(c, p, o, fault, Significance::Important)
}

/// Isto uz razinu `level` (igra je uzima iz `weight` primjerka).
fn equipment_at(
    c: &Content,
    p: &Period,
    o: SituationOutcome,
    fault: bool,
    level: Significance,
) -> i32 {
    let v2 = c.v2.as_ref().unwrap();
    let category = v2.base.profile(Profile::C).unwrap().category;
    let s = RoundSituation {
        outcome: o,
        space: None,
        fault,
        click: false,
        significance: level,
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
    let c = no_keys();
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

/// Razina predloška `code` iz njegova `weight` (kao igra).
fn level_of(c: &Content, code: &str) -> Significance {
    let spec = c
        .situation_templates
        .as_ref()
        .unwrap()
        .to_core()
        .templates
        .into_iter()
        .find(|s| s.code == code)
        .unwrap_or_else(|| panic!("nema predloška {code}"));
    Significance::from_weight(spec.weight)
}

#[test] // 8 (runda 2: K1 — kvar i bonus × faktor kruga i razine; L1 — bonus već uz ocjenu 3)
fn fault_outcomes_table() {
    let c = with_keys();
    let off = off0();
    println!(
        "
L1-13-4 tablica: učinak situacije na stanje prostora (bodovi, 100 bp = 1)"
    );
    println!(
        "{:<12} {:<12} {:<6} {:>8} {:>10} {:>10} {:>10}",
        "predložak", "razina", "krug", "šutnja", "hitni q2", "hitni q3", "hitni q5"
    );
    let pts = |bp: i32| f64::from(bp) / 100.0;
    for code in ["P-BRAVE", "P-KLIMA"] {
        let level = level_of(&c, code);
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
            let (f, sig) = (factor(&c, &p), level_bp(&c, level));
            let (lose, gain) = (scaled(FAULT_KEY, f, sig), scaled(FIXED_KEY, f, sig));
            let silent = SituationOutcome::unanswered(&inst);
            let s_on = equipment_at(&c, &p, silent.clone(), true, level);
            let s_off = equipment_at(&off, &p, silent, false, level);
            assert_eq!(s_on - s_off, -lose, "{code} {:?}: šutnja −kvar", p.kind);
            for a in &urgent {
                let at = |q: u8| SituationOutcome {
                    code: inst.code.clone(),
                    mechanics: a.mechanics.clone(),
                    quality: q,
                };
                let on = |q| equipment_at(&c, &p, at(q), true, level);
                let off_at = |q| equipment_at(&off, &p, at(q), false, level);
                println!(
                    "{:<12} {:<12} {:<6} {:>8.2} {:>10.2} {:>10.2} {:>10.2}",
                    code,
                    level.code(),
                    p.kind.code(),
                    pts(s_on),
                    pts(on(2)),
                    pts(on(3)),
                    pts(on(5))
                );
                assert_eq!(
                    on(2) - off_at(2),
                    -lose,
                    "{code} {:?}: popravak uz 2 ostaje −kvar",
                    p.kind
                );
                for q in [3, 5] {
                    let d = on(q) - off_at(q);
                    assert!(
                        (gain - lose - 1..=gain - lose + 1).contains(&d),
                        "{code} {:?}: popravak uz {q} daje −kvar + bonus ({}): {d}",
                        p.kind,
                        gain - lose
                    );
                }
            }
        }
    }
}

#[test] // 8 (runda 2, K1: tjedni kvar je mjesečni puta omjer faktora kruga, ne isti −1)
fn week_fault_effect_follows_the_round_factor() {
    let c = with_keys();
    let off = off0();
    let (inst, _) = instance_of(&c, "P-BRAVE", Month::Jul);
    let level = level_of(&c, "P-BRAVE");
    assert_eq!(
        level,
        Significance::Important,
        "P-BRAVE je važna (weight 3)"
    );
    let silent = SituationOutcome::unanswered(&inst);
    let month = month_period(Month::Jul);
    let week = period_by_code(PeriodKind::Week, "t02").unwrap();
    let d = |p: &Period| {
        equipment_at(&c, p, silent.clone(), true, level)
            - equipment_at(&off, p, silent.clone(), false, level)
    };
    let sig = level_bp(&c, level);
    assert_eq!(d(&month), -scaled(FAULT_KEY, factor(&c, &month), sig));
    assert_eq!(d(&week), -scaled(FAULT_KEY, factor(&c, &week), sig));
    assert!(
        factor(&c, &week) == factor(&c, &month) || d(&week) != d(&month),
        "različit faktor kruga mora dati različit kvar"
    );
}
