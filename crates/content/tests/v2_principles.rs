use content::{
    Content, ContentError, KNOWN_VIOLATIONS,
    Principle::{self, *},
    Snapshot, check_principles_with, principle_violations, repo_dir,
};
use serde_json::{Value, json};
fn v2_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}
fn built(v: &Value) -> Content {
    Snapshot::from_value(v).unwrap().build().unwrap()
}
fn tpl<'a>(v: &'a mut Value, code: &str) -> &'a mut Value {
    let i = v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["code"] == code)
        .unwrap();
    &mut v["v2"]["situation_templates"]["templates"][i]
}
/// Krši li `code` načelo `p` nakon izmjene.
fn hit(change: impl FnOnce(&mut Value), p: Principle, code: &str) -> bool {
    let mut v = v2_value();
    change(&mut v);
    principle_violations(&built(&v))
        .unwrap()
        .iter()
        .any(|x| x.principle == p && x.code == code)
}
fn repo_codes(p: Principle) -> Vec<String> {
    principle_violations(&built(&v2_value()))
        .unwrap()
        .into_iter()
        .filter(|x| x.principle == p)
        .map(|x| x.code)
        .collect()
}
#[test]
fn repo_content_loads_checked_with_known_debt() {
    Snapshot::read_v2_checked(&repo_dir()).unwrap();
} // 1
#[test] // 2: dug samo za tri nova načela; stropovi = stanje 14.10.
fn known_debt_names_existing_templates_and_never_grows() {
    let v = v2_value();
    let codes: Vec<&str> = v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["code"].as_str().unwrap())
        .collect();
    let ceiling = [
        (LabelLengths, 18),
        (BreakdownCondition, 4),
        (SilenceNotBetter, 16),
    ];
    for (p, list) in KNOWN_VIOLATIONS {
        let max = ceiling.iter().find(|(q, _)| q == p).map_or(0, |(_, n)| *n);
        assert!(
            list.len() <= max,
            "{p:?}: dug {} > {max} — popis smije samo padati",
            list.len()
        );
        let mut s = list.to_vec();
        s.sort();
        s.dedup();
        assert_eq!(s.len(), list.len(), "{p:?}: šifra dvaput");
        for c in *list {
            assert!(
                codes.contains(c),
                "{p:?}: {c} nije u sadržaju — makni ga s popisa"
            );
        }
    }
    for p in [
        BestAnswer,
        MiddleAnswer,
        RepeatGap,
        NamesPerList,
        DeptPlaceholder,
        NamesUnique,
    ] {
        assert!(repo_codes(p).is_empty(), "{p:?}: {:?}", repo_codes(p));
    }
}
#[test] // 3: preseljeno iz v2_templates.rs
fn old_principles_are_load_checks() {
    assert!(!hit(|_| {}, RepeatGap, "P-KLIMA"));
    assert!(hit(
        |v| tpl(v, "P-KLIMA")["answers"][0]["criteria"] = json!([1, 2, 3, 4]),
        BestAnswer,
        "P-KLIMA"
    ));
    assert!(hit(
        |v| tpl(v, "P-KLIMA")["answers"][1]["criteria"] = json!([1, 2]),
        MiddleAnswer,
        "P-KLIMA"
    ));
    assert!(hit(
        |v| tpl(v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3),
        RepeatGap,
        "P-KLIMA"
    ));
    assert!(hit(
        |v| {
            v["v2"]["names"]["partner"]["m"]
                .as_array_mut()
                .unwrap()
                .pop();
        },
        NamesPerList,
        "names.partner.m"
    ));
}
#[test] // 4: k = 1,5, bez obzira koji je odgovor najdulji
fn label_lengths_stay_within_one_and_a_half() {
    let set = |a: usize, b: usize, c: usize| {
        move |v: &mut Value| {
            for (i, n) in [a, b, c].into_iter().enumerate() {
                tpl(v, "P-KLIMA")["answers"][i]["label"] = json!("x".repeat(n));
            }
        }
    };
    assert!(!hit(set(150, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(hit(set(151, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(
        hit(set(100, 151, 100), LabelLengths, "P-KLIMA"),
        "srednji najdulji također odaje"
    );
}
#[test] // 5
fn breakdown_needs_bad_state_or_external_cause() {
    let no_cond = |v: &mut Value| {
        tpl(v, "P-BOJLER")["when"]
            .as_object_mut()
            .unwrap()
            .remove("dept_condition_pts");
    };
    assert!(!hit(|_| {}, BreakdownCondition, "P-BOJLER"));
    assert!(
        hit(no_cond, BreakdownCondition, "P-BOJLER"),
        "pojačanje nije uvjet"
    );
    assert!(hit(
        |v| tpl(v, "P-BOJLER")["when"]["dept_condition_pts"] = json!({"at_least": 1000}),
        BreakdownCondition,
        "P-BOJLER"
    ));
    assert!(!hit(
        |v| {
            no_cond(v);
            tpl(v, "P-BOJLER")["external_cause"] = json!(true);
        },
        BreakdownCondition,
        "P-BOJLER"
    ));
    assert!(
        !hit(|_| {}, BreakdownCondition, "P-OLUJA"),
        "ADR-042 t. 7: vanjski događaj"
    );
    assert!(hit(|_| {}, BreakdownCondition, "P-BRAVE"), "dug do Faze B");
    let mut v = v2_value();
    tpl(&mut v, "P-NOVINAR")["external_cause"] = json!(true);
    let r = Snapshot::from_value(&v).unwrap().build();
    assert!(
        matches!(&r, Err(ContentError::Invalid(m)) if m.contains("external_cause")),
        "{r:?}"
    );
}
#[test]
fn dept_placeholder_only_after_odjel_or_in_brackets() {
    // 6
    let title = |s: &'static str| move |v: &mut Value| tpl(v, "P-DOTRAJALO")["title"] = json!(s);
    assert!(
        !hit(|_| {}, DeptPlaceholder, "P-DOTRAJALO"),
        "naslov s „({{odjel}})” je dopušten"
    );
    assert!(!hit(
        title("Gost fotografira {stvar} u odjelu {odjel}"),
        DeptPlaceholder,
        "P-DOTRAJALO"
    ));
    assert!(hit(
        title("Gost fotografira {stvar} u {odjel}"),
        DeptPlaceholder,
        "P-DOTRAJALO"
    ));
    assert!(hit(
        title("{odjel}: gost fotografira {stvar}"),
        DeptPlaceholder,
        "P-DOTRAJALO"
    ));
}
#[test]
fn names_are_unique_across_lists() {
    assert!(hit(
        |v| {
            let n = v["v2"]["names"]["guest"]["f"][0].clone(); // 7
            v["v2"]["names"]["staff"]["f"][0] = n;
        },
        NamesUnique,
        "names"
    ));
}
#[test] // 8: P-DOTRAJALO danas: najlošiji s hitnim popravkom je skuplji, ali šutnja gubi opremu → nema dominacije
fn silence_is_not_better_than_the_worst_answer() {
    assert!(!hit(|_| {}, SilenceNotBetter, "P-DOTRAJALO"));
    assert!(
        hit(
            |v| tpl(v, "P-DOTRAJALO")["unanswered"] =
                json!([{"type": "ServiceRecovery", "compensation": 0}]),
            SilenceNotBetter,
            "P-DOTRAJALO"
        ),
        "isti bodovi, bez troška popravka"
    );
    assert!(
        hit(|_| {}, SilenceNotBetter, "P-NOVINAR"),
        "loš odgovor plaća boravak, šutnja ništa (dug)"
    );
}
#[test] // 13 (B1b): uvjet stanja vrijedi samo na prostoru koji se troši
fn breakdown_needs_a_space_that_wears() {
    let below = |v: &mut Value, code: &str| {
        tpl(v, code)["when"] = json!({"dept_condition_pts": {"below": 6000}});
    };
    // P-BRAVE je na Održavanju (ne troši se): uvjet na vlastitom odjelu nije uvjet.
    assert!(
        hit(|v| below(v, "P-BRAVE"), BreakdownCondition, "P-BRAVE"),
        "Održavanje se ne troši"
    );
    // Isti uvjet, ali stanje Smještaja (condition_of), ispunjava načelo.
    assert!(
        !hit(
            |v| {
                below(v, "P-BRAVE");
                tpl(v, "P-BRAVE")["condition_of"] = json!("rooms");
            },
            BreakdownCondition,
            "P-BRAVE"
        ),
        "condition_of rooms: prostor se troši"
    );
    // P-DIZALO (cijeli hotel): s condition_of i uvjetom je u redu, s condition_of bez uvjeta nije.
    assert!(
        !hit(
            |v| {
                below(v, "P-DIZALO");
                tpl(v, "P-DIZALO")["condition_of"] = json!("rooms");
            },
            BreakdownCondition,
            "P-DIZALO"
        ),
        "P-DIZALO + condition_of + uvjet"
    );
    assert!(
        hit(
            |v| tpl(v, "P-DIZALO")["condition_of"] = json!("rooms"),
            BreakdownCondition,
            "P-DIZALO"
        ),
        "condition_of bez uvjeta nije uvjet"
    );
}
#[test] // 12 (B1a, I2): načelo šutnje računa v2 situacije, obje vrste kruga
fn silence_principle_uses_v2_situation_mechanics() {
    let week_off = |v: &mut Value| v["v2"]["periods"]["week"]["situation_effect_bp"] = json!(1);
    let no_bonus = |v: &mut Value| v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(0);
    assert!(!hit(|_| {}, SilenceNotBetter, "P-DOTRAJALO"));
    assert!(
        !hit(week_off, SilenceNotBetter, "P-DOTRAJALO"),
        "+10 odvaja hitni odgovor od šutnje"
    );
    assert!(
        !hit(no_bonus, SilenceNotBetter, "P-DOTRAJALO"),
        "šutnja i dalje gubi opremu (privremeni popravak)"
    );
    assert!(
        hit(
            |v| {
                week_off(v);
                no_bonus(v)
            },
            SilenceNotBetter,
            "P-DOTRAJALO"
        ),
        "tjedni krug bez bonusa: bodovi 0 = 0, šutnja jeftinija (80 € prema 250 €)"
    );
}
#[test] // 9: dug smije samo padati
fn debt_list_only_shrinks() {
    let (c, mut v) = (built(&v2_value()), v2_value());
    tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    let e = check_principles_with(&built(&v), KNOWN_VIOLATIONS).unwrap_err();
    assert!(
        matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("razmak")),
        "{e:?}"
    );
    let mut known = KNOWN_VIOLATIONS.to_vec();
    known.push((RepeatGap, &["P-KLIMA"][..]));
    let e = check_principles_with(&c, &known).unwrap_err();
    assert!(
        matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("makni")),
        "{e:?}"
    );
    let mut other_dir = KNOWN_VIOLATIONS.to_vec();
    other_dir.push((LabelLengths, &["P-NEPOSTOJI"][..]));
    check_principles_with(&c, &other_dir)
        .expect("šifra koje nema (druga mapa sadržaja) ne ruši učitavanje");
    assert!(
        check_principles_with(&c, &[]).is_err(),
        "bez popisa duga današnji sadržaj pada"
    );
}
#[test]
fn snapshot_from_the_log_is_never_checked_for_principles() {
    // 10: stara igra se uvijek čita
    let mut v = v2_value();
    tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    Snapshot::from_value(&v)
        .unwrap()
        .build()
        .expect("snimak iz dnevnika gradi se bez načela");
}
#[test] // 11: put kojim server čita HOTELIXSIM_CONTENT
fn content_dir_with_new_violation_is_refused_at_load() {
    let (src, dir) = (
        repo_dir(),
        std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("a2-sadrzaj"),
    );
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("v2")).unwrap();
    for f in [
        "catalog.json",
        "profiles.json",
        "situations.json",
        "v2/periods.json",
        "v2/departments.json",
        "v2/profiles.json",
        "v2/economy.json",
        "v2/names.json",
    ] {
        std::fs::copy(src.join(f), dir.join(f)).unwrap();
    }
    let text = std::fs::read_to_string(src.join("v2/situation_templates.json")).unwrap();
    let mut t: Value = serde_json::from_str(&text).unwrap();
    let i = t["templates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|x| x["code"] == "P-KLIMA")
        .unwrap();
    t["templates"][i]["repeat"]["gap_rounds"] = json!(3);
    std::fs::write(
        dir.join("v2/situation_templates.json"),
        serde_json::to_string(&t).unwrap(),
    )
    .unwrap();
    let e = Snapshot::read_v2_checked(&dir).unwrap_err();
    assert!(
        matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA")),
        "{e:?}"
    );
}
