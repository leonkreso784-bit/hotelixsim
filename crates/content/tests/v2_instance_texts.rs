//! G-2 popravak 1 (Minor, I3): svih 55 predložaka, svaki s reprezentativnim ulazima (svi odjeli skupine, najmanja i
//! najveća vrijednost brojevnih mjesta, sve vrste uloge) -- NE ovisno o sjemenu. Svaki tekst primjerka koji ide
//! studentu ili AI-ju (naslov, oznake odgovora, rubrika, lik: opis uloge, brief, uvod, rezervni odgovor) je neprazan i
//! bez `{`/`}`. Rečenica uloge partnera bez `types` koristi neobvezni naziv uloge iz sadržaja (I3).
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - Naziv uloge je neobvezno polje `title` UZ ulogu u predlošku (`roles.<ključ>.title`, tekst); ako ga uloga ima, a
//!   nema `types`, rečenica uloge (`Instance.ai.character.role`) je upravo taj naziv. Sadržaj: P-INSPEKCIJA
//!   `inspektor` -> naziv koji sadrži "inspektor" (npr. "sanitarni inspektor"), P-VJENCANJE `organizatorica` -> naziv
//!   koji sadrži "vjenčanja" (npr. "organizatorica vjenčanja"). Uloge bez `title` i bez `types` ostaju
//!   "gošća/gost hotela", "zaposlenica/zaposlenik odjela X", "vanjska suradnica/vanjski suradnik hotela".
//! - Polje `title` ne troši slučajnost (izvlačenje je isto): test ga ne dira, ali ni ne ovisi o sjemenu.
//! - Provjera sadržaja odbija prazan `title` (`ContentError::Invalid`, poruka sadrži "naziv").

use content::templates::{Gender, RoleSource, Template};
use content::{Content, ContentError, Instance, Snapshot, repo_dir};
use serde_json::{Value, json};
use sim_hotel::{Month, Profile};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{DrawnSituation, NumberParam, RoleFill, Scope};

fn content() -> Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

fn name_of(female: bool, i: usize) -> String {
    if female {
        format!("Marija Horvat{i}")
    } else {
        format!("Ivan Horvat{i}")
    }
}

/// Primjerak predloška `t` u srpnju: odjel `dept`, brojevna mjesta (`high` = najveća vrijednost), vrsta uloge `variant`.
fn drawn(
    c: &Content,
    t: &Template,
    dept: Option<&DeptCode>,
    high: bool,
    variant: usize,
) -> DrawnSituation {
    let set = c.situation_templates.as_ref().unwrap().to_core();
    let spec = set.templates.iter().find(|s| s.code == t.code).unwrap();
    DrawnSituation {
        code: format!("{}.jul", t.code),
        template: t.code.clone(),
        period: sim_hotel_v2::calendar::month_period(Month::Jul),
        day: 30,
        occurrence: 1,
        slot: 1,
        kind: spec.kind,
        weight: spec.weight,
        scope: Scope::Hotel,
        major: None,
        department: dept.cloned(),
        roles: t
            .roles
            .iter()
            .enumerate()
            .map(|(i, (k, r))| {
                let role_type =
                    (!r.types.is_empty()).then(|| r.types[variant % r.types.len()].clone());
                (
                    k.clone(),
                    RoleFill {
                        name: name_of(r.gender == Gender::F, i),
                        role_type,
                    },
                )
            })
            .collect(),
        numbers: spec
            .numbers
            .iter()
            .map(|(k, n)| {
                let v = match (n, high) {
                    (NumberParam::Range(a, _), false) => *a,
                    (NumberParam::Range(_, b), true) => *b,
                    (NumberParam::Pick(v), false) => v[0],
                    (NumberParam::Pick(v), true) => v[v.len() - 1],
                };
                (k.clone(), v)
            })
            .collect(),
    }
}

/// Svi ulazi predloška: odjeli skupine (ili jedan bez odjela) x niski/visoki brojevi x vrsta uloge 0..3.
fn instances(c: &Content, t: &Template) -> Vec<(String, Instance)> {
    let cat = c.v2.as_ref().unwrap();
    let set = c.situation_templates.as_ref().unwrap().to_core();
    let spec = set.templates.iter().find(|s| s.code == t.code).unwrap();
    let depts: Vec<Option<DeptCode>> = match spec.department.members(cat) {
        None => vec![None],
        Some(m) => m.into_iter().map(Some).collect(),
    };
    let category = cat.base.profile(Profile::C).unwrap().category;
    let name = |d: &DeptCode| {
        cat.departments
            .get(d)
            .map_or_else(|| d.0.clone(), |s| s.name.clone())
    };
    let mut out = Vec::new();
    for dept in &depts {
        for high in [false, true] {
            for variant in 0..3 {
                let d = drawn(c, t, dept.as_ref(), high, variant);
                let label = format!("{} odjel={:?} high={high} vrsta={variant}", t.code, dept);
                let inst = c
                    .situation_templates
                    .as_ref()
                    .unwrap()
                    .instance_with(&d, &cat.base, category, &name)
                    .unwrap_or_else(|e| panic!("{label}: {e:?}"));
                out.push((label, inst));
            }
        }
    }
    out
}

fn clean(label: &str, what: &str, s: &str) -> Option<String> {
    if s.trim().is_empty() {
        return Some(format!("{label}: {what} prazno"));
    }
    (s.contains('{') || s.contains('}'))
        .then(|| format!("{label}: {what} s neispunjenim mjestom: {s}"))
}

#[test]
fn all_55_templates_fill_every_text_without_placeholders() {
    let c = content();
    let all = &c.situation_templates.as_ref().unwrap().templates;
    assert_eq!(all.len(), 55);
    let mut bad = Vec::new();
    for t in all {
        for (label, i) in instances(&c, t) {
            let s = &i.situation;
            bad.extend(clean(&label, "naslov", &s.title));
            for (n, a) in s.answers.iter().enumerate() {
                bad.extend(clean(&label, &format!("oznaka odgovora {n}"), &a.label));
            }
            for (n, r) in i.ai.rubric.iter().enumerate() {
                bad.extend(clean(&label, &format!("rubrika {}", n + 1), r));
            }
            if i.ai.rubric.len() != 5 {
                bad.push(format!(
                    "{label}: rubrika ima {} kriterija",
                    i.ai.rubric.len()
                ));
            }
            if let Some(ch) = &i.ai.character {
                bad.extend(clean(&label, "lik: ime", &ch.name));
                bad.extend(clean(&label, "lik: rečenica uloge", &ch.role));
                bad.extend(clean(&label, "lik: brief", &ch.brief));
                bad.extend(clean(&label, "lik: uvod", &ch.opening));
                bad.extend(clean(&label, "lik: rezervni odgovor", &ch.fallback_reply));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} problema, prvih 10:\n{}",
        bad.len(),
        bad[..bad.len().min(10)].join("\n")
    );
}

/// Rečenica uloge lika (prvi ulaz predloška).
fn role_sentence(c: &Content, code: &str) -> String {
    let t = c
        .situation_templates
        .as_ref()
        .unwrap()
        .templates
        .iter()
        .find(|t| t.code == code)
        .unwrap();
    instances(c, t)[0].1.ai.character.clone().unwrap().role
}

/// I3: partner bez `types` nosi naziv uloge, ne "vanjski suradnik hotela".
#[test]
fn partner_without_types_uses_its_role_title() {
    let c = content();
    let ins = role_sentence(&c, "P-INSPEKCIJA");
    assert!(ins.contains("inspektor"), "P-INSPEKCIJA: '{ins}'");
    assert!(!ins.contains("vanjsk"), "P-INSPEKCIJA: '{ins}'");
    let wed = role_sentence(&c, "P-VJENCANJE");
    assert!(wed.contains("vjenčanja"), "P-VJENCANJE: '{wed}'");
    assert!(!wed.contains("vanjsk"), "P-VJENCANJE: '{wed}'");
}

/// I3: ostale uloge s likom ostaju kakve jesu (vrsta iz `types`; inače gost/zaposlenik/vanjski suradnik po rodu).
#[test]
fn other_character_roles_keep_their_sentences() {
    let c = content();
    let mut checked = 0;
    for t in &c.situation_templates.as_ref().unwrap().templates {
        let Some(ch) = &t.character else { continue };
        if t.code == "P-INSPEKCIJA" || t.code == "P-VJENCANJE" {
            continue;
        }
        let role = &t.roles[&ch.role];
        let f = role.gender == Gender::F;
        for (label, i) in instances(&c, t) {
            let got = i.ai.character.unwrap().role;
            if !role.types.is_empty() {
                assert!(role.types.contains(&got), "{label}: '{got}'");
            } else {
                match role.from {
                    RoleSource::Guest => {
                        assert_eq!(
                            got,
                            if f { "gošća hotela" } else { "gost hotela" },
                            "{label}"
                        );
                    }
                    RoleSource::Partner => assert_eq!(
                        got,
                        if f {
                            "vanjska suradnica hotela"
                        } else {
                            "vanjski suradnik hotela"
                        },
                        "{label}"
                    ),
                    RoleSource::Staff => assert!(
                        got.starts_with(if f {
                            "zaposlenica odjela "
                        } else {
                            "zaposlenik odjela "
                        }),
                        "{label}: '{got}'"
                    ),
                }
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "premalo provjerenih ulaza: {checked}");
}

fn with_title(title: Value) -> Result<Content, ContentError> {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let ts = v["v2"]["situation_templates"]["templates"]
        .as_array_mut()
        .unwrap();
    let t = ts.iter_mut().find(|t| t["code"] == "P-INSPEKCIJA").unwrap();
    t["roles"]["inspektor"]["title"] = title;
    Snapshot::from_value(&v)?.build()
}

/// I3: naziv uloge je neobvezan, a prazan se odbija provjerom sadržaja.
#[test]
fn role_title_is_optional_and_empty_title_is_rejected() {
    with_title(json!("sanitarni inspektor")).expect("naziv uloge je dopušteno polje sadržaja");
    for empty in [json!(""), json!("   ")] {
        match with_title(empty) {
            Err(ContentError::Invalid(m)) => {
                assert!(m.contains("naziv"), "poruka bez 'naziv': {m}");
            }
            other => {
                panic!("prazan naziv uloge mora pasti na provjeri sadržaja, dobiveno {other:?}")
            }
        }
    }
}
