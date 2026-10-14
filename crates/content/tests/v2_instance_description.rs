//! G-2 popravak 1 (I4): `Instance` nosi popunjen opis situacije, na isti način kao naslov (mjesta `{…}` ispunjena).
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `content::Instance.description: String` -- opis predloška s popunjenim mjestima (uloge, brojke, odjel, iznosi);
//!   neprazan i bez `{`/`}`. Test je u zasebnoj datoteci da prevođenje ostalih čuvara ne ovisi o imenu polja.
//! - Ulaz u UI/AI ovdje se ne uvodi (A-1/W-1).

use content::{Snapshot, repo_dir};
use sim_hotel::{Month, Profile};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{DrawnSituation, NumberParam, RoleFill, Scope};

#[test]
fn all_55_templates_fill_the_description() {
    let c = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = c.v2.as_ref().unwrap();
    let tpls = c.situation_templates.as_ref().unwrap();
    let set = tpls.to_core();
    let category = cat.base.profile(Profile::C).unwrap().category;
    let name = |d: &DeptCode| {
        cat.departments
            .get(d)
            .map_or_else(|| d.0.clone(), |s| s.name.clone())
    };
    let mut bad = Vec::new();
    let mut with_places = 0;
    for t in &tpls.templates {
        let spec = set.templates.iter().find(|s| s.code == t.code).unwrap();
        let depts: Vec<Option<DeptCode>> = match spec.department.members(cat) {
            None => vec![None],
            Some(m) => m.into_iter().map(Some).collect(),
        };
        for dept in &depts {
            for high in [false, true] {
                let d = DrawnSituation {
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
                    department: dept.clone(),
                    roles: t
                        .roles
                        .iter()
                        .map(|(k, r)| {
                            (
                                k.clone(),
                                RoleFill {
                                    name: "Ana Kovač".into(),
                                    role_type: r.types.first().cloned(),
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
                };
                let i = tpls.instance_with(&d, &cat.base, category, &name).unwrap();
                if t.description.contains('{') {
                    with_places += 1;
                }
                let s = &i.description;
                if s.trim().is_empty() || s.contains('{') || s.contains('}') {
                    bad.push(format!("{} odjel={dept:?}: opis '{s}'", t.code));
                }
            }
        }
    }
    assert!(
        with_places > 50,
        "očekivano mnogo opisa s mjestima, ima {with_places} ulaza"
    );
    assert!(
        bad.is_empty(),
        "{} problema, prvih 5:\n{}",
        bad.len(),
        bad[..bad.len().min(5)].join("\n")
    );
}
