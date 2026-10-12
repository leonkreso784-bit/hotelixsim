//! Predlošci situacija v2 (`content/hotel/v2/situation_templates.json`, `v2/names.json`): format i provjera (S-1).

use content::templates::{Kind, MajorSide, Quantity, TemplateMechanic};
use content::{Content, ContentError, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};

fn v2_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Result<Content, ContentError> {
    Snapshot::from_value(v)?.build()
}

/// Indeks predloška u datoteci po šifri.
fn at(v: &Value, code: &str) -> usize {
    v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["code"] == code)
        .unwrap_or_else(|| panic!("nema predloška {code}"))
}

fn tpl<'a>(v: &'a mut Value, code: &str) -> &'a mut Value {
    let i = at(v, code);
    &mut v["v2"]["situation_templates"]["templates"][i]
}

/// Izmjena mora pasti na provjeri sadržaja s porukom koja sadrži `needle`.
fn rejected(change: impl FnOnce(&mut Value), needle: &str) {
    let mut v = v2_value();
    change(&mut v);
    match build(&v) {
        Err(ContentError::Invalid(m)) => {
            assert!(m.contains(needle), "poruka '{m}' ne sadrži '{needle}'")
        }
        other => panic!("očekivana greška sadržaja '{needle}', dobiveno {other:?}"),
    }
}

/// Izmjena mora pasti već pri čitanju JSON-a (nepoznato polje ili kriva vrsta).
fn rejected_as_json(change: impl FnOnce(&mut Value)) {
    let mut v = v2_value();
    change(&mut v);
    assert!(matches!(
        Snapshot::from_value(&v),
        Err(ContentError::Json(..))
    ));
}

#[test]
fn repo_templates_load_and_validate() {
    let c = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let t = c.situation_templates.expect("v2 snimak ima predloške");
    let codes: Vec<&str> = t.templates.iter().map(|t| t.code.as_str()).collect();
    assert_eq!(codes, ["P-KLIMA", "P-POVISICA", "P-DOTRAJALO"]);
    assert_eq!(
        (
            t.major.high_avg_min,
            t.major.low_avg_max,
            t.major.max_per_month
        ),
        (85, 35, 1)
    );
    let klima = &t.templates[0];
    assert_eq!(klima.kind, Kind::Complaint);
    assert_eq!(klima.department, "rooms");
    assert!(klima.character.is_some(), "razgovorna situacija ima lika");
    assert_eq!(
        klima
            .answers
            .iter()
            .map(|a| a.quality())
            .collect::<Vec<_>>(),
        [5, 3, 1]
    );
    assert!(matches!(
        &klima.answers[0].mechanics[0],
        TemplateMechanic::ServiceRecovery { compensation: Quantity::Param(p) } if p == "vecera"
    ));
    assert!(
        t.templates
            .iter()
            .all(|t| t.rubric.len() == 5 && !t.unanswered.is_empty())
    );
    assert!(
        !t.names.guest.f.is_empty() && !t.names.staff.m.is_empty() && !t.names.partner.f.is_empty()
    );
}

#[test]
fn v1_and_degenerate_snapshots_have_no_templates() {
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    assert!(v1.build().unwrap().situation_templates.is_none());
    let deg = v1.degenerate_v2();
    let v = deg.to_value();
    assert!(v["v2"].get("situation_templates").is_none() && v["v2"].get("names").is_none());
    assert!(deg.build().unwrap().situation_templates.is_none());
}

/// Stara v2 igra (snimak bez predložaka) čita se, gradi i ponovno zapisuje bajt-isto: isti otisak kao prije S-1.
#[test]
fn v2_snapshot_without_templates_keeps_its_fingerprint() {
    let mut old = v2_value();
    let section = old["v2"].as_object_mut().unwrap();
    section.remove("situation_templates");
    section.remove("names");
    let keys: Vec<&String> = old["v2"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["departments", "economy", "periods", "profiles"]);
    let s = Snapshot::from_value(&old).unwrap();
    assert_eq!(s.to_value(), old);
    assert_eq!(s.fingerprint(), fingerprint_value(&old));
    assert!(s.build().unwrap().situation_templates.is_none());
}

#[test]
fn v2_snapshot_with_templates_round_trips() {
    let v = v2_value();
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
    assert_ne!(
        fingerprint_value(&v),
        "sha256:7cd0c6db29b379ab8d1364a11881071cd0749b6b0cd01c5e5d057abb2e9f0cc8",
        "predlošci ulaze u otisak nove igre"
    );
}

#[test]
fn templates_and_names_come_together() {
    rejected(
        |v| {
            v["v2"].as_object_mut().unwrap().remove("names");
        },
        "imena",
    );
    rejected(
        |v| {
            v["v2"]
                .as_object_mut()
                .unwrap()
                .remove("situation_templates");
        },
        "imena",
    );
}

// --- mjesta za zamjenu u tekstu ---

#[test]
fn unknown_placeholder_is_rejected() {
    rejected(
        |v| tpl(v, "P-KLIMA")["description"] = json!("Gost u sobi {x} zove."),
        "{x}",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][0]["label"] = json!("Dati {popust}"),
        "{popust}",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["character"]["opening"] = json!("Ja sam {gost.prezime}."),
        "{gost.prezime}",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["rubric"][0] = json!("Isprika {odjela}"),
        "{odjela}",
    );
}

#[test]
fn unclosed_brace_is_rejected() {
    rejected(
        |v| tpl(v, "P-KLIMA")["title"] = json!("Klima u sobi {soba"),
        "zagrad",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["title"] = json!("Klima u sobi soba}"),
        "zagrad",
    );
}

#[test]
fn role_type_placeholder_needs_types() {
    rejected(
        |v| {
            tpl(v, "P-KLIMA")["roles"]["gost"]
                .as_object_mut()
                .unwrap()
                .remove("types");
        },
        "{gost.tip}",
    );
}

#[test]
fn department_placeholder_needs_a_department() {
    rejected(
        |v| {
            let t = tpl(v, "P-KLIMA");
            t["department"] = json!("hotel");
            t["chance"] = json!({ "base": 3 }); // pojačanje gleda stanje odjela
            t["title"] = json!("Klima ({odjel})");
        },
        "{odjel}",
    );
}

// --- odjeli ---

#[test]
fn unknown_department_is_rejected() {
    rejected(
        |v| tpl(v, "P-KLIMA")["department"] = json!("casino"),
        "casino",
    );
    rejected(
        |v| tpl(v, "P-POVISICA")["department"] = json!("*svi_odjeli"),
        "*svi_odjeli",
    );
}

#[test]
fn by_department_text_must_cover_exactly_the_possible_departments() {
    rejected(
        |v| {
            tpl(v, "P-DOTRAJALO")["params"]["stvar"]["by_department"]
                .as_object_mut()
                .unwrap()
                .remove("spa");
        },
        "spa",
    );
    rejected(
        |v| {
            tpl(v, "P-DOTRAJALO")["params"]["stvar"]["by_department"]["parking"] =
                json!("rupe u asfaltu")
        },
        "parking",
    );
}

#[test]
fn department_condition_on_hotel_template_is_rejected() {
    rejected(
        |v| {
            let t = tpl(v, "P-KLIMA");
            t["department"] = json!("hotel");
            t["when"]["dept_condition_pts"] = json!({ "below": 5000 });
        },
        "dept_condition_pts",
    );
}

// --- uvjeti pojave i izvlačenje ---

#[test]
fn invalid_conditions_are_rejected() {
    rejected(
        |v| tpl(v, "P-KLIMA")["when"]["months"] = json!(["jun", "oct"]),
        "oct",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["when"]["months"] = json!(["jun", "jun"]),
        "jun",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["when"]["occupancy_bp"] = json!({ "at_least": 12000 }),
        "occupancy_bp",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["when"]["morale_pts"] = json!({ "at_least": 6000, "below": 4000 }),
        "morale_pts",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["when"]["morale_pts"] = json!({}),
        "morale_pts",
    );
    rejected(|v| tpl(v, "P-KLIMA")["chance"]["base"] = json!(0), "chance");
    rejected(
        |v| tpl(v, "P-KLIMA")["chance"]["boost"][0]["times"] = json!(1),
        "times",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["chance"]["boost"][0]["when"] = json!({}),
        "boost",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["repeat"]["max_per_season"] = json!(0),
        "max_per_season",
    );
    rejected(|v| tpl(v, "P-KLIMA")["weight"] = json!(5), "weight");
}

#[test]
fn unknown_condition_field_is_rejected_when_read() {
    rejected_as_json(|v| tpl(v, "P-KLIMA")["when"]["temperature_above"] = json!(30));
    rejected_as_json(|v| tpl(v, "P-KLIMA")["kind"] = json!("nesreca"));
}

// --- ponuđeni odgovori, rubrika, učinak ---

#[test]
fn answer_cost_outside_effect_band_is_rejected() {
    // „Vratiti novac za cijeli boravak" = 700 % cijene noćenja; pojas do 300 % ga odbija.
    rejected(
        |v| tpl(v, "P-KLIMA")["effect_band"]["cost_night_pct"] = json!([0, 300]),
        "pojas",
    );
    // Najmanji trošak iznad nule: odgovor bez kompenzacije pada.
    rejected(
        |v| tpl(v, "P-KLIMA")["effect_band"]["cost_night_pct"] = json!([10, 800]),
        "pojas",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["effect_band"]["cost_night_pct"] = json!([200, 100]),
        "pojas",
    );
}

#[test]
fn raise_outside_band_is_rejected() {
    rejected(
        |v| tpl(v, "P-POVISICA")["effect_band"]["raise_pct"] = json!([0, 20]),
        "povišic",
    );
    rejected(
        |v| {
            tpl(v, "P-POVISICA")["effect_band"]
                .as_object_mut()
                .unwrap()
                .remove("raise_pct");
        },
        "povišic",
    );
}

#[test]
fn fixed_amount_must_be_zero() {
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][1]["mechanics"][0]["compensation"] = json!(12000),
        "iznos",
    );
}

#[test]
fn mechanic_param_must_exist_and_fit() {
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][0]["mechanics"][0]["compensation"] = json!("rucak"),
        "rucak",
    );
    // Broj sobe nije novčani iznos.
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][0]["mechanics"][0]["compensation"] = json!("soba"),
        "soba",
    );
    // Postotak cijene nije broj za povišicu.
    rejected(
        |v| tpl(v, "P-POVISICA")["answers"][1]["mechanics"][0]["raise_one_pct"] = json!("bonus"),
        "bonus",
    );
}

#[test]
fn rubric_must_have_five_levels() {
    rejected(
        |v| {
            tpl(v, "P-KLIMA")["rubric"].as_array_mut().unwrap().pop();
        },
        "rubrika",
    );
    rejected(|v| tpl(v, "P-KLIMA")["rubric"][2] = json!("  "), "rubrika");
}

#[test]
fn criteria_must_be_valid() {
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][0]["criteria"] = json!([1, 6]),
        "kriterij",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["answers"][1]["criteria"] = json!([2, 2]),
        "kriterij",
    );
}

#[test]
fn answer_count_is_two_to_four() {
    rejected(
        |v| {
            tpl(v, "P-KLIMA")["answers"]
                .as_array_mut()
                .unwrap()
                .truncate(1);
        },
        "odgovor",
    );
    rejected(
        |v| {
            let a = tpl(v, "P-KLIMA")["answers"][0].clone();
            let list = tpl(v, "P-KLIMA")["answers"].as_array_mut().unwrap();
            list.push(a.clone());
            list.push(a);
        },
        "odgovor",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["unanswered"] = json!([]),
        "unanswered",
    );
}

// --- likovi, uloge, imena ---

#[test]
fn character_role_must_exist() {
    rejected(
        |v| tpl(v, "P-KLIMA")["character"]["role"] = json!("susjed"),
        "susjed",
    );
    rejected(
        |v| tpl(v, "P-KLIMA")["character"]["fallback_reply"] = json!(""),
        "lik",
    );
}

#[test]
fn names_lists_must_be_full_and_unique() {
    rejected(|v| v["v2"]["names"]["guest"]["f"] = json!([]), "guest");
    rejected(
        |v| {
            let first = v["v2"]["names"]["staff"]["m"][0].clone();
            v["v2"]["names"]["staff"]["m"]
                .as_array_mut()
                .unwrap()
                .push(first);
        },
        "staff",
    );
}

// --- šifra, veća situacija, kolektivna ---

#[test]
fn template_code_must_be_unique_and_well_formed() {
    rejected(
        |v| tpl(v, "P-POVISICA")["code"] = json!("P-KLIMA"),
        "P-KLIMA",
    );
    rejected(|v| tpl(v, "P-KLIMA")["code"] = json!("S9"), "S9");
    rejected(
        |v| tpl(v, "P-KLIMA")["code"] = json!("P-KLIMA.t03"),
        "P-KLIMA.t03",
    );
}

#[test]
fn major_situation_is_per_hotel_and_rule_is_sane() {
    let mut v = v2_value();
    tpl(&mut v, "P-DOTRAJALO")["major"] = json!("low");
    let c = build(&v).unwrap().situation_templates.unwrap();
    assert_eq!(
        c.templates[at(&v, "P-DOTRAJALO")].major,
        Some(MajorSide::Low)
    );

    rejected(
        |v| {
            let t = tpl(v, "P-DOTRAJALO");
            t["major"] = json!("high");
            t["scope"] = json!("svi");
        },
        "svi",
    );
    rejected(
        |v| v["v2"]["situation_templates"]["major"]["low_avg_max"] = json!(90),
        "major",
    );
    rejected(
        |v| v["v2"]["situation_templates"]["major"]["high_avg_min"] = json!(101),
        "major",
    );
    rejected(
        |v| v["v2"]["situation_templates"]["major"]["max_per_month"] = json!(0),
        "major",
    );
}

#[test]
fn group_month_cannot_be_before_the_situation() {
    let group = json!({ "type": "GroupBooking", "accept": true, "rooms": 20, "nights": 5,
                        "discount_bp": 2000, "month": "jul" });
    let mut v = v2_value();
    tpl(&mut v, "P-KLIMA")["answers"][2]["mechanics"] = json!([group.clone()]);
    assert!(
        build(&v).is_err(),
        "srpanjska grupa u kolovozu (when.months) mora pasti"
    );
    tpl(&mut v, "P-KLIMA")["when"]["months"] = json!(["jun", "jul"]);
    tpl(&mut v, "P-KLIMA")["answers"][2]["criteria"] = json!([1]);
    build(&v).unwrap();
    tpl(&mut v, "P-KLIMA")["answers"][2]["mechanics"][0]["discount_bp"] = json!(10_001);
    assert!(matches!(build(&v), Err(ContentError::Invalid(m)) if m.contains("discount_bp")));
}
