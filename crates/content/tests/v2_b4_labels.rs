//! Čuvar cigle B4 (prepis oznaka odgovora i uvjeta kvarova), porcije B4a, B4b i B4c. Kartica:
//! `docs/plan/2026-10-14-popravak-temelja/task-11-b4-prepis-oznaka-i-uvjeta.md`.
//! Porcija B4c: `b4c_batch_labels_have_no_debt` (T3) i `breakdown_conditions_via_condition_of` (T9).
use content::{
    KNOWN_VIOLATIONS,
    Principle::{self, *},
    Snapshot, fingerprint_value, principle_violations, repo_dir,
};
use serde_json::{Value, json};

/// Isto što i `LABEL_RATIO_PCT` u `templates_principles.rs` (modul nije javan; graditelj ga ne mora izvoziti).
const LABEL_RATIO_PCT: usize = 150;

/// Otisak kostura (sve osim oznaka i dopuštenih uvjeta) izračunat iz neizmijenjenih datoteka stanja cc4a0f2; nepromijenjen i u B4c (8d134a4): dopuštena polja B4c u neizmijenjenom stanju ne postoje, pa ih brisanje ne mijenja.
const SKELETON_SHA: &str =
    "sha256:2ad7856323ecb07968086435e417d53291855a1ee84db31c185957de74f9c773";

/// Šifre porcije B4a (kartica, „Šifre porcija").
const B4A: [&str; 15] = [
    "P-KLIMA",
    "P-BOJLER",
    "P-KRIZA-OSOBLJA",
    "P-RANI-DOLAZAK",
    "P-NESTALA-STVAR",
    "P-LJUBIMAC",
    "P-BUCNA-SOBA",
    "P-OVERBOOKING",
    "P-RED-NA-RECEPCIJI",
    "P-PRAONICA",
    "P-UCJENA-RECENZIJOM",
    "P-DOTRAJALO",
    "P-POHVALA",
    "P-PRISTUPACNOST",
    "P-NETOCNA-RECENZIJA",
];

/// Šifre porcije B4b (kartica, „Šifre porcija").
const B4B: [&str; 22] = [
    "P-NOVINAR",
    "P-KONGRES",
    "P-VAL-RECENZIJA",
    "P-TOPLINSKI-VAL",
    "P-OLUJA",
    "P-REGATA",
    "P-GOST-POZLILO",
    "P-DIZALO",
    "P-NESTANAK-STRUJE",
    "P-DIJETE-IZGUBLJENO",
    "P-POVISICA",
    "P-BOLOVANJE",
    "P-SLOBODNI-DANI",
    "P-OSPORENA-NAPLATA",
    "P-RACUN-GRESKA",
    "P-TERMINAL",
    "P-PREPORUKA",
    "P-LAST-MINUTE",
    "P-RADNICI-TVRTKE",
    "P-BRAVE",
    "P-CURENJE-VODE",
    "P-PARKING-STETA",
];

/// Šifre porcije B4c (kartica, „Šifre porcija").
const B4C: [&str; 18] = [
    "P-NOVI-LJUDI",
    "P-INSPEKCIJA",
    "P-ALERGIJA",
    "P-POSEBNA-PREHRANA",
    "P-MANJAK-INVENTURA",
    "P-DOBAVLJAC",
    "P-PIJANI-GOST",
    "P-PROSIDBA",
    "P-BAZEN-PUMPA",
    "P-LEZALJKE",
    "P-SUSJEDI-BAZEN",
    "P-SPA-TERMIN",
    "P-SPA-TECAJ",
    "P-WELLNESS-PAKET",
    "P-VJENCANJE",
    "P-ROOFTOP-BUKA",
    "P-LOBBY-SUKOB",
    "P-LOBBY-KAVA",
];

fn v2_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}
fn built(v: &Value) -> content::Content {
    Snapshot::from_value(v).unwrap().build().unwrap()
}
fn repo_codes(p: Principle) -> Vec<String> {
    principle_violations(&built(&v2_value()))
        .unwrap()
        .into_iter()
        .filter(|x| x.principle == p)
        .map(|x| x.code)
        .collect()
}
fn templates(v: &Value) -> &Vec<Value> {
    v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
}
fn anchors(v: &Value) -> Vec<&Value> {
    v["situations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| matches!(s["code"].as_str(), Some("S1" | "S2" | "S3" | "S4")))
        .collect()
}
fn lens(answers: &Value) -> Vec<usize> {
    answers
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["label"].as_str().unwrap().chars().count())
        .collect()
}
fn within_ratio(l: &[usize]) -> bool {
    l.iter().max().unwrap() * 100 <= l.iter().min().unwrap() * LABEL_RATIO_PCT
}
/// Kvaliteta odgovora šablone: broj kriterija (1..=5), kao `TemplateAnswer::quality`.
fn tpl_quality(a: &Value) -> usize {
    a["criteria"].as_array().map_or(0, Vec::len).clamp(1, 5)
}

#[test] // T1
fn anchors_labels_within_ratio() {
    let v = v2_value();
    let a = anchors(&v);
    assert_eq!(a.len(), 4, "sidra S1–S4");
    for s in a {
        let l = lens(&s["answers"]);
        assert!(
            within_ratio(&l),
            "{}: duljine oznaka {l:?} nisu unutar {LABEL_RATIO_PCT} %",
            s["code"]
        );
    }
}

#[test] // T2, karakterizacijski (zelen od početka)
fn only_labels_and_listed_conditions_change() {
    fn skeleton(v: &Value) -> String {
        let mut v = v.clone();
        for a in v["situations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .flat_map(|s| s["answers"].as_array_mut().unwrap().iter_mut())
        {
            a.as_object_mut().unwrap().remove("label");
        }
        for t in v["v2"]["situation_templates"]["templates"]
            .as_array_mut()
            .unwrap()
        {
            for a in t["answers"].as_array_mut().unwrap() {
                a.as_object_mut().unwrap().remove("label");
            }
            let code = t["code"].as_str().unwrap().to_string();
            let o = t.as_object_mut().unwrap();
            // dopušteno po šifri: uvjet stanja (B4a: klima, praonica; B4c: brave, curenje, dizalo)
            if matches!(
                code.as_str(),
                "P-KLIMA" | "P-PRAONICA" | "P-BRAVE" | "P-CURENJE-VODE" | "P-DIZALO"
            ) {
                // `when` se ne serijalizira kad je prazan (danas ga nemaju P-PRAONICA ni ova tri): prazan objekt = izostavljen
                if let Some(w) = o.get_mut("when").and_then(Value::as_object_mut) {
                    w.remove("dept_condition_pts");
                    if w.is_empty() {
                        o.remove("when");
                    }
                }
            }
            // B4c: `condition_of` samo uz ova tri kvara, `external_cause` samo uz P-INSPEKCIJU
            if matches!(code.as_str(), "P-BRAVE" | "P-CURENJE-VODE" | "P-DIZALO") {
                o.remove("condition_of");
            }
            if code == "P-INSPEKCIJA" {
                o.remove("external_cause");
            }
        }
        fingerprint_value(&v)
    }
    assert_eq!(
        skeleton(&v2_value()),
        SKELETON_SHA,
        "B4 smije mijenjati samo oznake i uvjete kvarova: `when.dept_condition_pts` na P-KLIMA, P-PRAONICA, P-BRAVE, P-CURENJE-VODE, P-DIZALO; `condition_of` na zadnja tri; `external_cause` na P-INSPEKCIJI"
    );
}

#[test] // T3 (b4a)
fn b4a_batch_labels_have_no_debt() {
    let bad: Vec<String> = repo_codes(LabelLengths)
        .into_iter()
        .filter(|c| B4A.contains(&c.as_str()))
        .collect();
    assert!(bad.is_empty(), "oznake porcije B4a još krše 1,5×: {bad:?}");
    // popis duga ne smije više nositi ni jednu šifru porcije
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == LabelLengths)
        .map_or(&[][..], |(_, l)| *l);
    let listed: Vec<&&str> = debt.iter().filter(|c| B4A.contains(*c)).collect();
    assert!(listed.is_empty(), "makni s popisa duga: {listed:?}");
}

#[test] // T3 (b4b)
fn b4b_batch_labels_have_no_debt() {
    let bad: Vec<String> = repo_codes(LabelLengths)
        .into_iter()
        .filter(|c| B4B.contains(&c.as_str()))
        .collect();
    assert!(bad.is_empty(), "oznake porcije B4b još krše 1,5×: {bad:?}");
    // popis duga ne smije više nositi ni jednu šifru porcije
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == LabelLengths)
        .map_or(&[][..], |(_, l)| *l);
    let listed: Vec<&&str> = debt.iter().filter(|c| B4B.contains(*c)).collect();
    assert!(listed.is_empty(), "makni s popisa duga: {listed:?}");
}

#[test] // T3 (b4c)
fn b4c_batch_labels_have_no_debt() {
    let bad: Vec<String> = repo_codes(LabelLengths)
        .into_iter()
        .filter(|c| B4C.contains(&c.as_str()))
        .collect();
    assert!(bad.is_empty(), "oznake porcije B4c još krše 1,5×: {bad:?}");
    // popis duga ne smije više nositi ni jednu šifru porcije
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == LabelLengths)
        .map_or(&[][..], |(_, l)| *l);
    let listed: Vec<&&str> = debt.iter().filter(|c| B4C.contains(*c)).collect();
    assert!(listed.is_empty(), "makni s popisa duga: {listed:?}");
}

#[test] // T4
fn best_answer_is_not_a_tell() {
    let v = v2_value();
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == LabelLengths)
        .map_or(&[][..], |(_, l)| *l);
    // (najbolji je strogo najdulji, ima strogo najviše zareza) po očišćenom predlošku ili sidru
    let mut units: Vec<(bool, bool)> = Vec::new();
    let mut judge = |answers: &Value, quality: &dyn Fn(&Value) -> usize| {
        if !within_ratio(&lens(answers)) {
            return;
        }
        let a = answers.as_array().unwrap();
        let best = a.iter().position(|x| quality(x) == 5).unwrap();
        let len = |x: &Value| x["label"].as_str().unwrap().chars().count();
        let commas = |x: &Value| x["label"].as_str().unwrap().matches(',').count();
        let longest = a
            .iter()
            .enumerate()
            .all(|(i, x)| i == best || len(&a[best]) > len(x));
        let commiest = a
            .iter()
            .enumerate()
            .all(|(i, x)| i == best || commas(&a[best]) > commas(x));
        units.push((longest, commiest));
    };
    for t in templates(&v) {
        if !debt.contains(&t["code"].as_str().unwrap()) {
            judge(&t["answers"], &tpl_quality);
        }
    }
    for s in anchors(&v) {
        judge(&s["answers"], &|a: &Value| {
            a["quality"].as_u64().unwrap() as usize
        });
    }
    if units.is_empty() {
        return; // trivijalno dok nema očišćenih
    }
    let n = units.len();
    let longest = units.iter().filter(|u| u.0).count();
    let commas = units.iter().filter(|u| u.1).count();
    assert!(longest * 2 <= n, "najbolji je najdulji u {longest}/{n}");
    assert!(
        commas * 2 <= n,
        "najbolji ima najviše zareza u {commas}/{n}"
    );
}

/// 🟨 T4b: markeri „za poslije / sastojak" (korijeni, mala slova) i prag 20 % jedinica. Pregled opusa B4a (Važno 1):
/// u 9 od 19 jedinica samo najbolji odgovor imao dio „za poslije", pa se najbolji pogađao 10/10.
const MARKERS: [&str; 15] = [
    "sutra", "ubuduće", "kasnije", "idući", "sljedeć", "tjed", "potvrd", "plan", "provjer",
    "isprik", "ispri", "naknad", "popust", "besplatn", "uzrok",
];
const EXCLUSIVE_MARKER_MAX_PCT: usize = 20; // 🟨

#[test] // T4b
fn best_answer_has_no_exclusive_marker() {
    let v = v2_value();
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == LabelLengths)
        .map_or(&[][..], |(_, l)| *l);
    // (šifra, oznake, indeks najboljeg) za očišćene predloške i sidra unutar 1,5×
    let mut units: Vec<(String, Vec<String>, usize)> = Vec::new();
    let labels = |a: &Value| -> Vec<String> {
        a.as_array()
            .unwrap()
            .iter()
            .map(|x| x["label"].as_str().unwrap().to_lowercase())
            .collect()
    };
    for t in templates(&v) {
        if debt.contains(&t["code"].as_str().unwrap()) {
            continue;
        }
        let best = t["answers"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| tpl_quality(a) == 5)
            .unwrap();
        units.push((
            t["code"].as_str().unwrap().into(),
            labels(&t["answers"]),
            best,
        ));
    }
    for s in anchors(&v) {
        if !within_ratio(&lens(&s["answers"])) {
            continue;
        }
        let best = s["answers"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["quality"] == 5)
            .unwrap();
        units.push((
            s["code"].as_str().unwrap().into(),
            labels(&s["answers"]),
            best,
        ));
    }
    if units.is_empty() {
        return;
    }
    let mut hits: Vec<String> = Vec::new();
    for (code, l, best) in &units {
        let found: Vec<&str> = MARKERS
            .iter()
            .copied()
            .filter(|m| {
                l[*best].contains(m)
                    && l.iter()
                        .enumerate()
                        .all(|(i, x)| i == *best || !x.contains(m))
            })
            .collect();
        if !found.is_empty() {
            hits.push(format!("{code} (samo najbolji ima: {found:?})"));
        }
    }
    assert!(
        hits.len() * 100 <= units.len() * EXCLUSIVE_MARKER_MAX_PCT,
        "{}/{} jedinica ima marker samo u najboljem odgovoru (najviše {EXCLUSIVE_MARKER_MAX_PCT} %): {}",
        hits.len(),
        units.len(),
        hits.join("; ")
    );
}

#[test] // T5, karakterizacijski
fn exactly_one_best_answer_per_template_and_anchor() {
    let v = v2_value();
    for t in templates(&v) {
        let n = t["answers"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| tpl_quality(a) == 5)
            .count();
        assert_eq!(n, 1, "{}: odgovora kvalitete 5", t["code"]);
    }
    for s in anchors(&v) {
        let n = s["answers"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["quality"] == 5)
            .count();
        assert_eq!(n, 1, "{}: odgovora kvalitete 5", s["code"]);
    }
}

#[test] // T6 (m4): kartica očekuje RED, ali se ne da izvesti — vidi izvještaj
fn duplicate_name_in_a_list_is_never_counted_as_distinct() {
    // `Snapshot::build` odbija duplikat unutar popisa (`templates_check.rs:72`) prije nego što `principle_violations`
    // dobije `Content`, pa `v.len()` tamo uvijek jednako broju različitih. Test čuva tu garanciju (zelen).
    let mut v = v2_value();
    let list = v["v2"]["names"]["guest"]["f"].as_array_mut().unwrap();
    list.truncate(19);
    let dup = list[0].clone();
    list.push(dup);
    let r = Snapshot::from_value(&v).unwrap().build();
    assert!(
        matches!(&r, Err(content::ContentError::Invalid(m)) if m.contains("names.guest.f") && m.contains("ponavlja")),
        "{:?}",
        r.err()
    );
}

#[test] // T7, karakterizacijski
fn dept_placeholder_checked_in_character_fields() {
    for field in ["brief", "opening", "fallback_reply"] {
        let bad = |s: &'static str| {
            let mut v = v2_value();
            let i = templates(&v)
                .iter()
                .position(|t| t["code"] == "P-KLIMA")
                .unwrap();
            v["v2"]["situation_templates"]["templates"][i]["character"][field] = json!(s);
            principle_violations(&built(&v))
                .unwrap()
                .iter()
                .any(|x| x.principle == DeptPlaceholder && x.code == "P-KLIMA")
        };
        assert!(bad("Gost se žali na {odjel} cijelo jutro"), "{field}");
        assert!(
            !bad("Gost se žali na odjel {odjel} cijelo jutro"),
            "{field}"
        );
        assert!(
            !bad("Gost se žali na sobu ({odjel}) cijelo jutro"),
            "{field}"
        );
    }
}

#[test] // T8 (B4a)
fn klima_and_praonica_need_bad_rooms_state() {
    let v = v2_value();
    for code in ["P-KLIMA", "P-PRAONICA"] {
        let t = templates(&v).iter().find(|t| t["code"] == code).unwrap();
        assert!(
            t["when"]["dept_condition_pts"]["below"] == 6500,
            "{code}: when.dept_condition_pts.below mora biti točno 6500 (Leon 16.10.)"
        );
    }
    let open = repo_codes(BreakdownCondition);
    for code in ["P-KLIMA", "P-PRAONICA"] {
        assert!(
            !open.iter().any(|c| c == code),
            "{code} još krši uvjet kvara"
        );
    }
}

#[test] // T9 (B4c)
fn breakdown_conditions_via_condition_of() {
    let v = v2_value();
    let find = |code: &str| templates(&v).iter().find(|t| t["code"] == code).unwrap();
    for code in ["P-BRAVE", "P-CURENJE-VODE", "P-DIZALO"] {
        let t = find(code);
        assert_eq!(
            t["condition_of"], "rooms",
            "{code}: condition_of mora biti rooms"
        );
        // Prag 8000: Leonova odluka 16.10. (anketa s brojkama; 5000–7000 ruše tjednu branu J-3 jer hotel A
        // počinje na 7500 i dobro održan nikad ne padne ispod, a ova dva kvara su mu najčešće situacije).
        assert!(
            t["when"]["dept_condition_pts"]["below"] == 8000,
            "{code}: when.dept_condition_pts.below mora biti točno 8000 (Leon 16.10.)"
        );
    }
    assert_eq!(
        find("P-INSPEKCIJA")["external_cause"],
        true,
        "P-INSPEKCIJA je vanjski događaj"
    );
    let open = repo_codes(BreakdownCondition);
    assert!(
        open.is_empty(),
        "BreakdownCondition još ima prekršaje: {open:?}"
    );
    let debt = KNOWN_VIOLATIONS
        .iter()
        .find(|(p, _)| *p == BreakdownCondition)
        .map_or(&[][..], |(_, l)| *l);
    assert!(debt.is_empty(), "makni s popisa duga: {debt:?}");
}
