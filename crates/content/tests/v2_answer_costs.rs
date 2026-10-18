//! L1-13 dio 4 (A4, Leon „ok"; test čuvara 4): nijedan predložak ne smije imati trošak SAMO u najboljem odgovoru.
//! Kad samo najbolji troši novac (naknada, popravak, nagrada, povišica, pričuva), slabiji odgovor je besplatan i
//! „prepoznaje se" po cijeni; trošak dobiva barem jedan slabiji odgovor (kvaliteta ispod najviše).
//!
//! Novac odgovora je obračun v2 (`sim_hotel_v2::situation_effect`: `cost_cents` + `extra_wage_cents`) primjerka
//! predloška za svaku kategoriju i svaki mjesec pojave (mjesečni krug), kao provjera načela šutnje.
//!
//! Karakterizacija (izmjereno na c1ef169, prije dijela 4): trošak samo u najboljem imalo je točno 11 predložaka
//! (`ONLY_BEST_PAID_C1EF169`).
//!
//! Runda 2 (Leon 18.10., četvrta anketa): **P-POHVALA je izuzeta** — ostaje „samo najbolji nosi nagradu" (Leon 12.10.,
//! `v2_templates.rs::praise_is_reward_only_and_offered_answers_never_lower_morale`), pa A4 vrijedi za 10 predložaka.

use std::collections::BTreeMap;

use content::templates::Template;
use content::{Content, Snapshot, repo_dir};
use serde_json::{Value, json};
use sim_hotel::{Month, SituationOutcome};
use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::situation_effect;
use sim_hotel_v2::situations::{DrawnSituation, NumberParam, RoundSituation, Scope, Significance};

/// Predlošci koji su na c1ef169 imali novac samo u najboljem odgovoru (A4; kartica: grubo brojanje 18.10.).
const ONLY_BEST_PAID_C1EF169: [&str; 11] = [
    "P-BUCNA-SOBA",
    "P-DIJETE-IZGUBLJENO",
    "P-GOST-POZLILO",
    "P-KONGRES",
    "P-LEZALJKE",
    "P-LJUBIMAC",
    "P-POHVALA",
    "P-PROSIDBA",
    "P-RACUN-GRESKA",
    "P-SLOBODNI-DANI",
    "P-VJENCANJE",
];

/// Iznimke od A4 (Leon 18.10.): predlošci u kojima samo najbolji odgovor smije nositi novac.
const A4_EXEMPT: [&str; 1] = ["P-POHVALA"];

fn repo_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Content {
    Snapshot::from_value(v).unwrap().build().unwrap()
}

/// Troši li (najbolji, slabiji) odgovor novac u barem jednoj kategoriji i mjesecu pojave.
fn pays(c: &Content, t: &Template) -> (bool, bool) {
    let tpl = c.situation_templates.as_ref().unwrap();
    let v2 = c.v2.as_ref().unwrap();
    let core = tpl.to_core();
    let spec = core
        .templates
        .iter()
        .find(|s| s.code == t.code)
        .unwrap_or_else(|| panic!("{}: nema u jezgri", t.code));
    let months: Vec<Month> = if t.when.months.is_empty() {
        Month::SEASON.to_vec()
    } else {
        t.when
            .months
            .iter()
            .filter_map(|m| Month::from_code(m))
            .collect()
    };
    let numbers: BTreeMap<String, i64> = spec
        .numbers
        .iter()
        .map(|(k, n)| {
            let v = match n {
                NumberParam::Range(a, _) => *a,
                NumberParam::Pick(v) => v.iter().copied().min().unwrap_or(0),
            };
            (k.clone(), v)
        })
        .collect();
    let department = spec.department.members(v2).and_then(|d| d.first().cloned());
    let top = t.answers.iter().map(|a| a.quality()).max().unwrap();
    let (mut best, mut weaker) = (false, false);
    for &category in v2.base.categories.keys() {
        for &m in &months {
            let d = DrawnSituation {
                code: format!("{}.{}", t.code, m.code()),
                template: t.code.clone(),
                period: month_period(m),
                day: 1,
                occurrence: 1,
                slot: 1,
                kind: spec.kind,
                weight: spec.weight,
                scope: Scope::Hotel,
                major: None,
                department: department.clone(),
                roles: BTreeMap::new(),
                numbers: numbers.clone(),
            };
            let inst = tpl.instance(&d, &v2.base, category).unwrap();
            for (a, ta) in inst.answers.iter().zip(&t.answers) {
                let s = RoundSituation {
                    outcome: SituationOutcome::from_answer(&inst.code, a),
                    space: department.clone(),
                    fault: false,
                    click: false,
                    significance: Significance::Important,
                };
                let e = situation_effect(&s, &month_period(m), v2, category)
                    .unwrap()
                    .effects;
                if e.cost_cents + e.extra_wage_cents > 0 {
                    if ta.quality() == top {
                        best = true;
                    } else {
                        weaker = true;
                    }
                }
            }
        }
    }
    (best, weaker)
}

/// Šifre predložaka kojima samo najbolji odgovor troši novac, sortirano.
fn only_best_pays(c: &Content) -> Vec<String> {
    let mut out: Vec<String> = c
        .situation_templates
        .as_ref()
        .unwrap()
        .templates
        .iter()
        .filter(|t| pays(c, t) == (true, false))
        .map(|t| t.code.clone())
        .collect();
    out.sort();
    out
}

/// Repo u kojem slabiji odgovori predložaka `codes` ne troše ništa (naknada 0): stanje prije A4 za te predloške.
fn weaker_answers_free(codes: &[&str]) -> Content {
    let mut v = repo_value();
    let list = v["v2"]["situation_templates"]["templates"]
        .as_array_mut()
        .unwrap();
    for t in list.iter_mut() {
        if !codes.contains(&t["code"].as_str().unwrap()) {
            continue;
        }
        let quality = |a: &Value| a["criteria"].as_array().unwrap().len().clamp(1, 5);
        let top = t["answers"]
            .as_array()
            .unwrap()
            .iter()
            .map(quality)
            .max()
            .unwrap();
        for a in t["answers"].as_array_mut().unwrap() {
            if quality(a) < top {
                a["mechanics"] = json!([{"type": "ServiceRecovery", "compensation": 0}]);
            }
        }
    }
    build(&v)
}

#[test]
fn weaker_answer_costs_where_only_best_did() {
    let c = build(&repo_value());
    let open = only_best_pays(&c);
    assert_eq!(
        open,
        A4_EXEMPT.to_vec(),
        "A4: trošak samo u najboljem odgovoru smije ostati samo iznimkama (dodaj trošak barem jednom slabijem)"
    );
    let a4: Vec<&str> = ONLY_BEST_PAID_C1EF169
        .into_iter()
        .filter(|c| !A4_EXEMPT.contains(c))
        .collect();
    assert_eq!(a4.len(), 10, "A4 dira 10 predložaka (11 bez P-POHVALE)");
    for code in a4 {
        let t = c
            .situation_templates
            .as_ref()
            .unwrap()
            .templates
            .iter()
            .find(|t| t.code == code)
            .unwrap_or_else(|| panic!("{code} nije u sadržaju"));
        assert_eq!(
            pays(&c, t),
            (true, true),
            "{code}: najbolji i barem jedan slabiji odgovor troše novac"
        );
    }
}

/// Iznimka P-POHVALA (Leon 18.10.): i dalje samo najbolji odgovor nosi nagradu (novac), nijedan slabiji.
#[test]
fn praise_stays_only_best_paid() {
    let c = build(&repo_value());
    for code in A4_EXEMPT {
        let t = c
            .situation_templates
            .as_ref()
            .unwrap()
            .templates
            .iter()
            .find(|t| t.code == code)
            .unwrap_or_else(|| panic!("{code} nije u sadržaju"));
        assert_eq!(
            pays(&c, t),
            (true, false),
            "{code}: samo najbolji nosi nagradu"
        );
        assert!(
            ONLY_BEST_PAID_C1EF169.contains(&code),
            "{code}: iznimka je s popisa A4"
        );
    }
}

/// Karakterizacija popisa: uz besplatne slabije odgovore upravo tih 11 predložaka (i nijedan drugi) ima novac samo u
/// najboljem. Vrijedi i prije i poslije dijela 4, pa ujedno dokazuje da provjera vidi trošak.
#[test]
fn only_best_paid_list_is_exact() {
    let c = weaker_answers_free(&ONLY_BEST_PAID_C1EF169);
    assert_eq!(only_best_pays(&c), ONLY_BEST_PAID_C1EF169.to_vec());
}
