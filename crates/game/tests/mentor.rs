//! Mentor govori ljudski (Task 14): činjenice za mentora i mentor bez AI-ja nikad ne pokazuju internu jedinicu.
mod common;
use common::*;
use game::dto::*;
use game::*;
use serde_json::Value;
use sim_hotel::Month;

fn line(var: VarCode, cause: CauseDto, amount: i64, unit: UnitCode) -> LineDto {
    LineDto {
        var,
        cause,
        amount,
        unit,
    }
}

fn sit(code: &str, title: &str, quality: u8) -> DebriefSituation {
    DebriefSituation {
        code: code.into(),
        title: title.into(),
        quality,
        free_text: false,
        rationale: None,
        neutral: false,
        unanswered: false,
        chosen_label: None,
    }
}

fn july_plan() -> PlanDto {
    PlanDto {
        price_cents: 11_500,
        marketing_cents: 0,
        seasonal_target: 20,
        wage_pct: 95,
        training: TrainingCode::None,
        maintenance: MaintenanceCode::Deferred,
        renovation: false,
        reply_reviews: false,
        departments: std::collections::BTreeMap::new(),
    }
}

const CHEF: &str = "Šef kuhinje traži povišicu ili odlazi";
const CHEF_ANSWER: &str = "Bonus na kraju sezone ako ostane, bez razgovora";

/// Srpanj kao iz Leonova primjera, složen ručno da su brojke poznate.
fn sample() -> DebriefInput {
    DebriefInput {
        round: RoundKey::parse("jul").unwrap(),
        next_round: RoundKey::parse("aug"),
        prev_round: RoundKey::parse("jun"),
        round_kind: "month".into(),
        first_date: (1, MonthCode::Jul),
        last_date: (31, MonthCode::Jul),
        hotel_name: "Hotel Lungomare".into(),
        situations: vec![DebriefSituation {
            chosen_label: Some(CHEF_ANSWER.into()),
            ..sit("S2", CHEF, 2)
        }],
        indices_before: [5_210, 4_850, 4_876, 5_020, 5_600],
        indices_after: [5_390, 4_849, 4_269, 4_990, 6_120],
        progress_hundredths: 325,
        cash_end_cents: 24_395_000,
        plan: july_plan(),
        price_ref_cents: 12_000,
        seasonal_before: 14,
        load_before_hundredths: 170,
        load_after_hundredths: 230,
        // Moral 55 → 50,47 bodova: stopa odlazaka 300 + 15 · (6000 − moral) / 100 baza.
        turnover_before_bp: 375,
        turnover_after_bp: 443,
        department_departures: Vec::new(),
        wage_before_pct: 95,
        wage_start_pct: 95,
        plan_before: Some(july_plan()),
        lines: vec![
            line(VarCode::Morale, CauseDto::Baseline, -453, UnitCode::Points),
            line(VarCode::Morale, CauseDto::Wage, -1_000, UnitCode::Points),
            line(
                VarCode::Morale,
                CauseDto::Situation("S2".into()),
                -600,
                UnitCode::Points,
            ),
            line(
                VarCode::Experience,
                CauseDto::Turnover,
                -533,
                UnitCode::Points,
            ),
            line(
                VarCode::Experience,
                CauseDto::Baseline,
                300,
                UnitCode::Points,
            ),
            line(
                VarCode::Equipment,
                CauseDto::Baseline,
                -400,
                UnitCode::Points,
            ),
            line(
                VarCode::Equipment,
                CauseDto::Maintenance,
                0,
                UnitCode::Points,
            ),
            line(VarCode::Equipment, CauseDto::Load, 30, UnitCode::Points),
            line(VarCode::Sold, CauseDto::Reputation, -200, UnitCode::Bp),
            line(VarCode::Sold, CauseDto::Price, -250, UnitCode::Bp),
            line(
                VarCode::Sold,
                CauseDto::Baseline,
                1_240,
                UnitCode::RoomNights,
            ),
            line(VarCode::Costs, CauseDto::Wage, 1_234_567, UnitCode::Cents),
            line(
                VarCode::Cash,
                CauseDto::Baseline,
                24_395_000,
                UnitCode::Cents,
            ),
            line(
                VarCode::Cash,
                CauseDto::Renovation,
                -300_000_000,
                UnitCode::Cents,
            ),
            line(VarCode::Staff, CauseDto::Turnover, 3, UnitCode::People),
            line(VarCode::Staff, CauseDto::Hires, 1, UnitCode::People),
            line(
                VarCode::Rating,
                CauseDto::Baseline,
                -12,
                UnitCode::RatingHundredths,
            ),
            line(
                VarCode::Satisfaction,
                CauseDto::Baseline,
                100,
                UnitCode::Points,
            ),
            line(
                VarCode::Satisfaction,
                CauseDto::Renovation,
                200,
                UnitCode::Points,
            ),
        ],
    }
}

fn facts_json(input: &DebriefInput) -> Value {
    serde_json::to_value(debrief_facts(input)).unwrap()
}

fn index<'a>(f: &'a Value, name: &str) -> &'a Value {
    f["indeksi"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["naziv"] == name)
        .unwrap_or_else(|| panic!("nema indeksa {name}"))
}

/// Sve promjene jedne varijable (po hrvatskom nazivu) kao (uzrok, iznos).
fn changes(f: &Value, var: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for i in f["indeksi"].as_array().unwrap() {
        for d in i["sto_je_pokrenulo"].as_array().unwrap() {
            if d["sto"] == var {
                for c in d["promjene"].as_array().unwrap() {
                    out.push((
                        c["uzrok"].as_str().unwrap().to_owned(),
                        c["iznos"].as_str().unwrap().to_owned(),
                    ));
                }
            }
        }
    }
    out
}

fn amount_for(f: &Value, var: &str, cause_part: &str) -> String {
    changes(f, var)
        .into_iter()
        .find(|(c, _)| c.contains(cause_part))
        .unwrap_or_else(|| panic!("{var}: nema uzroka „{cause_part}“"))
        .1
}

/// Jedinice iza kojih broj smije stajati u tekstu (sve ostalo je sirov broj).
const UNITS: [&str; 14] = [
    "bodova", "boda", "bod", "€", "%", "ocjene", "osoba", "osobe", "noćenja", "noćenje", "/5",
    "sezonsk", "soba", "noći",
];

/// Svaki broj u tekstu ima jedinicu. Iznimka (`index_prose`, samo tekst mentora bez AI-ja): vrijednost
/// indeksa u rečenici o indeksu („Indeks Zaposlenici pao je s 49 na 43."), jer je indeks sam svoja ljestvica.
fn assert_numbers_have_units(text: &str, index_prose: bool) {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len()
            && (chars[i].is_ascii_digit()
                || (matches!(chars[i], '.' | ',')
                    && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit())))
        {
            i += 1;
        }
        let run: String = chars[start..i].iter().collect();
        let mut k = i;
        while chars.get(k) == Some(&' ') {
            k += 1;
        }
        let rest: String = chars[k..].iter().collect();
        if let Some(u) = UNITS.iter().find(|u| rest.starts_with(**u)) {
            i = k + u.chars().count();
            continue;
        }
        let before: String = chars[..start].iter().collect();
        let sentence = before.rsplit(['.', '\n']).next().unwrap_or("");
        let index_value = index_prose
            && (before.ends_with(" s ") || before.ends_with(" na "))
            && sentence.contains("ndeks ");
        assert!(index_value, "broj „{run}“ bez jedinice:\n{text}");
    }
}

/// Jednoslovni kodovi indeksa (G R Z O F) ne smiju se pojaviti kao riječ.
fn assert_no_index_codes(text: &str) {
    let chars: Vec<char> = text.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        let alone = |j: Option<usize>| {
            j.and_then(|j| chars.get(j))
                .is_none_or(|c| !c.is_alphanumeric())
        };
        if matches!(c, 'G' | 'R' | 'Z' | 'F') && alone(i.checked_sub(1)) && alone(Some(i + 1)) {
            panic!("kod indeksa „{c}“ u tekstu:\n{text}");
        }
    }
    for code in ["G", "R", "Z", "O", "F"] {
        for prefix in ["ndeks ", "ndeksa ", "ndeksu "] {
            let bad = format!("{prefix}{code}");
            let hit = text.match_indices(&bad).any(|(at, _)| {
                text[at + bad.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric())
            });
            assert!(!hit, "„{bad}“ u tekstu:\n{text}");
        }
    }
}

/// Zabranjeni ostaci internih imena (jedinice, kodovi, engleski mjeseci, šifre situacija).
fn assert_no_internal_words(text: &str) {
    for bad in [
        "bp",
        "cents",
        "centi",
        "hundredths",
        "stotink",
        "rating_",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "june",
        "july",
        "baseline",
        "Baseline",
        "treći indeks",
        "indeks 3",
    ] {
        assert!(!text.contains(bad), "„{bad}“ u tekstu:\n{text}");
    }
    let chars: Vec<char> = text.chars().collect();
    for w in chars.windows(2) {
        assert!(
            !(w[0] == 'S' && w[1].is_ascii_digit()),
            "šifra situacije u tekstu:\n{text}"
        );
    }
}

/// Čuvar teksta mentora bez AI-ja.
fn assert_human(text: &str) {
    assert_no_internal_words(text);
    assert_no_index_codes(text);
    assert_numbers_have_units(text, true);
}

/// Čuvar činjenica: gole brojke samo pod `prije`/`poslije`/`promjena` (0–100), svaki tekst s jedinicama.
fn assert_human_facts(f: &Value) {
    fn walk(v: &Value, key: &str) {
        match v {
            Value::Number(n) => {
                let n = n.as_i64().unwrap_or(i64::MAX);
                let ok = match key {
                    "prije" | "poslije" => (0..=100).contains(&n),
                    "promjena" => (-100..=100).contains(&n),
                    _ => false,
                };
                assert!(ok, "gola brojka {n} pod „{key}“");
            }
            Value::String(s) => {
                assert_no_internal_words(s);
                assert_no_index_codes(s);
                assert!(
                    !matches!(s.as_str(), "G" | "R" | "Z" | "O" | "F"),
                    "kod indeksa pod „{key}“"
                );
                assert_numbers_have_units(s, false);
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, key)),
            Value::Object(o) => o.iter().for_each(|(k, x)| {
                assert_no_internal_words(k);
                walk(x, k);
            }),
            Value::Bool(_) | Value::Null => {}
        }
    }
    walk(f, "");
}

/// Čuvar mora uhvatiti sirove brojke od tri i manje znamenki i kodove indeksa (Leonov primjer).
#[test]
fn guard_catches_short_raw_numbers_and_index_codes() {
    let caught = |t: &'static str| std::panic::catch_unwind(|| assert_human(t)).is_err();
    for t in [
        "Moral je pao za −600 uz osnovnih −453.",
        "Napredak −245.",
        "Ugled je smanjio prodaju za 200 bp.",
        "Najviše je pao indeks G.",
        "Indeks Z pao je.",
        "Pao je treći indeks.",
    ] {
        assert!(caught(t), "čuvar nije uhvatio: {t}");
    }
    assert!(!caught(
        "Indeks Zaposlenici pao je s 49 na 43. Moral −6 bodova, blagajna 243.950 € (2/5)."
    ));
    let fact = |v: Value| std::panic::catch_unwind(|| assert_human_facts(&v)).is_err();
    assert!(fact(serde_json::json!({ "iznos": -600 })));
    assert!(fact(serde_json::json!({ "prije": 4876 })));
    assert!(fact(serde_json::json!({ "iznos": "−453" })));
    assert!(!fact(
        serde_json::json!({ "prije": 49, "iznos": "−6 bodova" })
    ));
}

#[test]
fn indices_are_whole_numbers_rounded_half_away_and_change_is_difference_of_rounded() {
    let mut input = sample();
    input.indices_before = [4_850, 4_849, 5_000, 5_049, 5_050];
    input.indices_after = [4_849, 4_850, 4_950, 5_150, 5_050];
    let f = facts_json(&input);
    let names: Vec<&str> = f["indeksi"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["naziv"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Zadovoljstvo gostiju",
            "Recenzije",
            "Zaposlenici",
            "Operativa",
            "Financije"
        ]
    );
    let g = index(&f, "Zadovoljstvo gostiju");
    assert_eq!(
        (&g["prije"], &g["poslije"], &g["promjena"]),
        (&49.into(), &48.into(), &(-1).into())
    );
    assert_eq!(g["smjer"], "pao");
    let r = index(&f, "Recenzije");
    assert_eq!(
        (&r["prije"], &r["poslije"], &r["promjena"]),
        (&48.into(), &49.into(), &1.into())
    );
    assert_eq!(r["smjer"], "porastao");
    // 5000 → 50, 4950 → 50: razlika zaokruženih je 0 iako se stotinke razlikuju.
    let z = index(&f, "Zaposlenici");
    assert_eq!(
        (&z["prije"], &z["poslije"], &z["promjena"]),
        (&50.into(), &50.into(), &0.into())
    );
    assert_eq!(z["smjer"], "isti");
    let o = index(&f, "Operativa");
    assert_eq!(
        (&o["prije"], &o["poslije"], &o["promjena"]),
        (&50.into(), &52.into(), &2.into())
    );
    assert_eq!(index(&f, "Financije")["prije"], 51);
}

#[test]
fn money_is_whole_euros_with_dot_for_thousands() {
    let f = facts_json(&sample());
    assert_eq!(f["blagajna_na_kraju_mjeseca"], "243.950 €");
    assert_eq!(amount_for(&f, "Troškovi", "plaće"), "12.346 €");
    assert_eq!(amount_for(&f, "Blagajna", "obnova"), "−3.000.000 €");
    assert_eq!(amount_for(&f, "Blagajna", "zarada"), "+243.950 €");
}

#[test]
fn demand_factors_are_percent_of_demand() {
    let f = facts_json(&sample());
    assert_eq!(amount_for(&f, "Prodane noći", "ugled"), "−2 % potražnje");
    assert_eq!(amount_for(&f, "Prodane noći", "cijena"), "−2,5 % potražnje");
    assert_eq!(amount_for(&f, "Prodane noći", "osnovna"), "1.240 noćenja");
}

#[test]
fn points_people_and_rating_in_human_units() {
    let f = facts_json(&sample());
    assert_eq!(amount_for(&f, "Moral", "plaće"), "−10 bodova");
    assert_eq!(amount_for(&f, "Moral", "ukupna"), "−5 bodova");
    assert_eq!(amount_for(&f, "Iskustvo", "otišli"), "−5 bodova");
    assert_eq!(amount_for(&f, "Iskustvo", "radom"), "+3 boda");
    assert_eq!(amount_for(&f, "Zadovoljstvo gostiju", "obnova"), "+2 boda");
    assert_eq!(amount_for(&f, "Zadovoljstvo gostiju", "usluga"), "+1 bod");
    assert_eq!(
        amount_for(&f, "Ocjena na portalima", "zadovoljstv"),
        "−0,12 ocjene"
    );
    // Ljudi riječima, bez miješanja predznaka unutar istog pokretača.
    assert_eq!(amount_for(&f, "Zaposleni", "odlasci"), "otišle 3 osobe");
    assert_eq!(
        amount_for(&f, "Zaposleni", "zapošljavanje"),
        "zaposlena 1 osoba"
    );
}

#[test]
fn situation_as_cause_carries_its_title_not_code() {
    let f = facts_json(&sample());
    let (cause, amount) = changes(&f, "Moral")
        .into_iter()
        .find(|(c, _)| c.contains(CHEF))
        .expect("situacija kao uzrok morala");
    assert_eq!(cause, format!("situacija „{CHEF}“"));
    assert_eq!(amount, "−6 bodova");
    assert!(!f.to_string().contains("S2"));
}

#[test]
fn zero_lines_and_lines_that_round_to_zero_are_left_out() {
    let f = facts_json(&sample());
    let equipment = changes(&f, "Oprema");
    assert_eq!(equipment.len(), 1, "samo trošenje: {equipment:?}");
    assert_eq!(equipment[0].1, "−4 boda");
    let mut input = sample();
    input.lines = vec![line(
        VarCode::Morale,
        CauseDto::Baseline,
        0,
        UnitCode::Points,
    )];
    assert!(changes(&facts_json(&input), "Moral").is_empty());
    // Varijabla bez ijedne promjene ne pojavljuje se uz indeks; stopa odlazaka prema prošlom mjesecu
    // je uvijek činjenica Zaposlenika (i kad je ista).
    let f = facts_json(&input);
    let drivers: Vec<&str> = index(&f, "Zaposlenici")["sto_je_pokrenulo"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["sto"].as_str().unwrap())
        .collect();
    assert_eq!(drivers, ["Zaposleni"]);
    assert_eq!(changes(&f, "Zaposleni").len(), 2);
}

#[test]
fn situations_have_grade_in_words_free_text_rationale_and_neutral_flag() {
    let mut input = sample();
    input.situations = vec![
        DebriefSituation {
            free_text: true,
            rationale: Some("Ispričao si se i odmah ponudio rješenje.".into()),
            ..sit("S1", "Klima u sobi obitelji s djecom", 4)
        },
        DebriefSituation {
            free_text: true,
            neutral: true,
            ..sit("S2", CHEF, 3)
        },
        DebriefSituation {
            unanswered: true,
            ..sit("S3", "Agencija nudi grupu", 1)
        },
    ];
    let f = facts_json(&input);
    let s = f["situacije"].as_array().unwrap();
    assert_eq!(s[0]["naslov"], "Klima u sobi obitelji s djecom");
    assert_eq!(s[0]["ocjena"], "4/5");
    assert_eq!(s[0]["ocjena_rijecima"], "dobro");
    assert_eq!(s[0]["slobodan_odgovor"], true);
    assert_eq!(
        s[0]["obrazlozenje"],
        "Ispričao si se i odmah ponudio rješenje."
    );
    assert!(s[0].get("neutralna_ocjena").is_none());
    assert_eq!(s[1]["ocjena_rijecima"], "srednje");
    assert!(
        s[1]["neutralna_ocjena"]
            .as_str()
            .unwrap()
            .contains("nije stigao")
    );
    assert_eq!(s[2]["ocjena_rijecima"], "loše");
    assert_eq!(s[2]["bez_odgovora"], true);
}

#[test]
fn month_next_month_hotel_and_progress_in_words() {
    let f = facts_json(&sample());
    assert_eq!(f["hotel"], "Hotel Lungomare");
    assert_eq!(f["mjesec"], "u srpnju");
    assert_eq!(f["iduci_mjesec"], "u kolovozu");
    assert_eq!(f["napredak_od_pocetka"], "+3 boda");
    let mut last = sample();
    last.round = RoundKey::Month(Month::Sep);
    last.next_round = None;
    last.progress_hundredths = -1_150;
    let f = facts_json(&last);
    assert_eq!(f["iduci_mjesec"], Value::Null);
    assert_eq!(f["napredak_od_pocetka"], "−12 bodova");
}

#[test]
fn facts_for_hand_made_month_are_human() {
    assert_human_facts(&facts_json(&sample()));
    assert_human(&fallback_debrief(&sample()));
}

#[test]
fn guard_facts_and_fallback_for_real_settled_month_are_human() {
    let (g, h) = july_resolved_bad_chef_game();
    let input = g.debrief_input(h, Month::Jul).unwrap();
    assert_eq!(
        input.next_round, None,
        "u testnom sadržaju srpanj je zadnji"
    );
    let json = serde_json::to_string_pretty(&debrief_facts(&input)).unwrap();
    assert_human_facts(&facts_json(&input));
    let text = fallback_debrief(&input);
    assert_human(&text);
    // Za izvještaj: `cargo test -p game --test mentor -- --nocapture guard`.
    println!("{json}\n-----\n{text}");
    let june = g.debrief_input(h, Month::Jun).unwrap();
    assert_eq!(june.next_round, Some(RoundKey::Month(Month::Jul)));
    assert_human_facts(&facts_json(&june));
    assert_human(&fallback_debrief(&june));
}

#[test]
fn real_month_input_carries_codes_cash_and_unanswered() {
    let (g, h) = july_resolved_bad_chef_game();
    let input = g.debrief_input(h, Month::Jul).unwrap();
    let s2 = input.situations.iter().find(|s| s.code == "S2").unwrap();
    assert_eq!((s2.quality, s2.unanswered), (2, false));
    assert_eq!(s2.chosen_label.as_deref(), Some(CHEF_ANSWER));
    let july_start = g.state_at(h, Month::Jul).unwrap().summary();
    assert_eq!(
        (input.plan.wage_pct, input.seasonal_before),
        (95, july_start.seasonal)
    );
    assert!(input.price_ref_cents > 0);
    assert_eq!(
        input.load_after_hundredths,
        g.latest_state(h).unwrap().summary().load_hundredths
    );
    let cash = g.latest_state(h).unwrap().summary().cash_cents;
    assert_eq!(input.cash_end_cents, cash);
}

#[test]
fn fallback_has_four_parts_and_honest_last_line() {
    let text = fallback_debrief(&sample());
    let heads: Vec<&str> = text.lines().filter(|l| !l.ends_with('.')).collect();
    assert_eq!(
        heads
            .iter()
            .filter(|l| !l.is_empty())
            .copied()
            .collect::<Vec<_>>(),
        [
            "Što je išlo dobro",
            "Što nije valjalo",
            "Zašto",
            "Što probati u kolovozu"
        ]
    );
    assert!(text.starts_with("U srpnju je napredak tvog hotela od početka igre +3 boda."));
    assert!(text.ends_with("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki."));
}

#[test]
fn fallback_good_and_bad_name_indices_and_situations() {
    let text = fallback_debrief(&sample());
    let part = |head: &str| -> String {
        let start = text.find(&format!("{head}\n")).unwrap() + head.len() + 1;
        let rest = &text[start..];
        rest[..rest.find("\n\n").unwrap_or(rest.len())].to_owned()
    };
    let good = part("Što je išlo dobro");
    // Porasli: Financije 56 → 61 (+5) prije Zadovoljstva gostiju 52 → 54 (+2).
    let fin = good
        .find("Indeks Financije porastao je s 56 na 61.")
        .unwrap();
    let sat = good
        .find("Indeks Zadovoljstvo gostiju porastao je s 52 na 54.")
        .unwrap();
    assert!(fin < sat);
    let bad = part("Što nije valjalo");
    assert!(
        bad.contains("Indeks Zaposlenici pao je s 49 na 43."),
        "{bad}"
    );
    assert!(bad.contains("Indeks Recenzije pao je s 49 na 48."), "{bad}");
    assert!(
        bad.contains(&format!(
            "Situacija „{CHEF}“ riješena je loše (2/5): izabran je odgovor „{CHEF_ANSWER}“."
        )),
        "{bad}"
    );
    let why = part("Zašto");
    // Najveći negativni uzrok Zaposlenika je plaća (−10 bodova pritiska) ispred situacije (−6).
    assert!(
        why.contains("Indeks Zaposlenici pao je ponajviše zbog nižih plaća"),
        "{why}"
    );
    assert!(
        why.contains("Indeks Recenzije pao je ponajviše zbog nižeg zadovoljstva gostiju"),
        "{why}"
    );
    let tip = part("Što probati u kolovozu");
    assert!(
        tip.contains("plaća") || tip.contains("tim"),
        "savjet za Zaposlenike: {tip}"
    );
}

#[test]
fn fallback_says_honestly_when_nothing_moved() {
    let mut input = sample();
    input.indices_after = input.indices_before;
    input.situations = vec![sit("S2", CHEF, 3)];
    let text = fallback_debrief(&input);
    assert!(
        text.contains("Ovaj mjesec nijedan indeks nije porastao"),
        "{text}"
    );
    assert!(text.contains("Nijedan indeks nije pao"), "{text}");
    assert_human(&text);
}

#[test]
fn fallback_situation_cause_uses_title() {
    let mut input = sample();
    input
        .lines
        .retain(|l| !(l.var == VarCode::Morale && l.cause == CauseDto::Wage));
    let text = fallback_debrief(&input);
    assert!(
        text.contains(&format!(
            "Indeks Zaposlenici pao je ponajviše zbog rješenja situacije „{CHEF}“"
        )),
        "{text}"
    );
}

#[test]
fn chosen_answer_is_in_facts_and_free_text_is_marked() {
    let f = facts_json(&sample());
    assert_eq!(f["situacije"][0]["odabrani_odgovor"], CHEF_ANSWER);
    let mut input = sample();
    input.situations = vec![
        DebriefSituation {
            free_text: true,
            ..sit("S1", "Klima u sobi obitelji s djecom", 4)
        },
        DebriefSituation {
            unanswered: true,
            ..sit("S3", "Agencija nudi grupu", 1)
        },
    ];
    let f = facts_json(&input);
    assert_eq!(
        f["situacije"][0]["odabrani_odgovor"],
        "vlastiti odgovor, napisan svojim riječima"
    );
    assert!(f["situacije"][1].get("odabrani_odgovor").is_none());
}

#[test]
fn plan_decisions_in_words() {
    let p = &facts_json(&sample())["odluke_u_planu"];
    assert_eq!(
        p["cijena_sobe"],
        "115 € po noći, 4 % ispod uobičajene cijene za ovaj mjesec"
    );
    assert_eq!(p["marketing"], "bez marketinga");
    assert_eq!(
        p["sezonski_radnici"],
        "20 sezonskih radnika, više nego prošli mjesec"
    );
    // Plaće su od početka 95 %: mentor ne smije reći da ih je student spustio (Sloj 1/OT-2).
    assert_eq!(
        p["place"],
        "95 % uobičajene plaće (niže od uobičajenih; takve su od početka, student ih nije mijenjao)"
    );
    assert_eq!(p["obuka"], "bez obuke, isto kao prošli mjesec");
    assert_eq!(
        p["odrzavanje"],
        "odgođeno (oprema se ovaj mjesec nije popravljala), isto kao prošli mjesec"
    );
    assert_eq!(p["obnova"], "ne");
    assert_eq!(p["odgovori_na_recenzije"], "ne");
    let mut input = sample();
    input.plan = PlanDto {
        price_cents: 12_000,
        marketing_cents: 500_000,
        seasonal_target: 1,
        wage_pct: 100,
        training: TrainingCode::Basic,
        maintenance: MaintenanceCode::Intensive,
        renovation: true,
        reply_reviews: true,
        departments: std::collections::BTreeMap::new(),
    };
    input.seasonal_before = 1;
    let p = &facts_json(&input)["odluke_u_planu"];
    assert_eq!(
        p["cijena_sobe"],
        "120 € po noći, kao uobičajena cijena za ovaj mjesec"
    );
    assert_eq!(p["marketing"], "5.000 €");
    assert_eq!(
        p["sezonski_radnici"],
        "1 sezonski radnik, isto kao prošli mjesec"
    );
    assert_eq!(
        p["place"],
        "100 % uobičajene plaće (student ih je podigao sa 95 % na 100 %)"
    );
    assert_eq!(
        p["obuka"],
        "osnovna obuka (učinak stiže idući mjesec), promijenjeno u odnosu na prošli mjesec"
    );
    assert_eq!(
        p["odrzavanje"],
        "pojačano, promijenjeno u odnosu na prošli mjesec"
    );
    assert_eq!(p["odgovori_na_recenzije"], "da");
}

#[test]
fn staff_load_is_an_operations_driver_in_words() {
    let f = facts_json(&sample());
    let load = changes(&f, "Opterećenje osoblja");
    assert_eq!(
        load,
        [
            (
                "opterećenje u ovom mjesecu".to_owned(),
                "2,3 noćenja po zaposlenom na dan, osoblje je preopterećeno".to_owned()
            ),
            (
                "opterećenje prošli mjesec".to_owned(),
                "1,7 noćenja po zaposlenom na dan".to_owned()
            ),
        ]
    );
    let drivers: Vec<&str> = index(&f, "Operativa")["sto_je_pokrenulo"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["sto"].as_str().unwrap())
        .collect();
    assert_eq!(drivers, ["Oprema", "Iskustvo", "Opterećenje osoblja"]);
    // Ispod praga i prije i poslije: opterećenje nije tema.
    let mut calm = sample();
    calm.load_before_hundredths = 150;
    calm.load_after_hundredths = 170;
    assert!(changes(&facts_json(&calm), "Opterećenje osoblja").is_empty());
}

#[test]
fn fallback_blames_overload_when_operations_falls() {
    let mut input = sample();
    input.indices_after[3] = 4_500; // Operativa 50 → 45
    let text = fallback_debrief(&input);
    assert!(
        text.contains("Indeks Operativa pao je ponajviše zbog preopterećenog osoblja"),
        "{text}"
    );
    assert_human(&text);
}

#[test]
fn same_cause_of_same_variable_is_one_line() {
    let mut input = sample();
    input.lines.push(line(
        VarCode::Costs,
        CauseDto::Situation("S2".into()),
        300_000,
        UnitCode::Cents,
    ));
    input.lines.push(line(
        VarCode::Costs,
        CauseDto::Situation("S2".into()),
        150_000,
        UnitCode::Cents,
    ));
    let costs = changes(&facts_json(&input), "Troškovi");
    let chef: Vec<_> = costs.iter().filter(|(c, _)| c.contains(CHEF)).collect();
    assert_eq!(chef.len(), 1, "{costs:?}");
    assert_eq!(chef[0].1, "4.500 €");
}

/// Smjer retka (boja na ekranu) iz `debrief_facts`: (pokretač, dio uzroka) → smjer.
fn effect_of(f: &DebriefFacts, what: &str, cause_part: &str) -> Effect {
    f.indices
        .iter()
        .flat_map(|i| &i.drivers)
        .filter(|d| d.what == what)
        .flat_map(|d| &d.changes)
        .find(|c| c.cause.contains(cause_part))
        .unwrap_or_else(|| panic!("nema retka {what} / {cause_part}"))
        .effect
}

#[test]
fn every_line_says_whether_it_pushed_its_index_up_or_down() {
    let mut input = sample();
    input.lines.push(line(
        VarCode::Costs,
        CauseDto::Situation("S2".into()),
        300_000,
        UnitCode::Cents,
    ));
    let f = debrief_facts(&input);
    for (what, cause, want) in [
        ("Moral", "plaće", Effect::Down),
        ("Moral", CHEF, Effect::Down),
        // Ukupna promjena morala je zbroj, ne uzrok.
        ("Moral", "ukupna promjena", Effect::Neutral),
        ("Iskustvo", "stečeno radom", Effect::Up),
        ("Iskustvo", "novi ljudi", Effect::Down),
        ("Oprema", "trošenje", Effect::Down),
        ("Prodane noći", "ugled", Effect::Down),
        ("Prodane noći", "cijena", Effect::Down),
        // Razine bez smjera: osnovna potražnja, troškovi, ljudi.
        ("Prodane noći", "osnovna potražnja", Effect::Neutral),
        ("Troškovi", "plaće", Effect::Neutral),
        ("Zaposleni", "odlasci", Effect::Neutral),
        ("Zaposleni", "zapošljavanje", Effect::Neutral),
        // Trošak situacije ruši Financije.
        ("Troškovi", CHEF, Effect::Down),
        ("Blagajna", "zarada iz poslovanja", Effect::Up),
        ("Blagajna", "obnova", Effect::Down),
        ("Ocjena na portalima", "ocjene gostiju", Effect::Down),
        ("Zadovoljstvo gostiju", "usluga, oprema", Effect::Up),
        ("Zadovoljstvo gostiju", "obnova", Effect::Up),
        ("Opterećenje osoblja", "u ovom mjesecu", Effect::Down),
        ("Opterećenje osoblja", "prošli mjesec", Effect::Neutral),
    ] {
        assert_eq!(effect_of(&f, what, cause), want, "{what} / {cause}");
    }
    // Jedno pravilo: redak je šteta za mentora točno kad je gurnuo indeks dolje.
    for c in f
        .indices
        .iter()
        .flat_map(|i| &i.drivers)
        .flat_map(|d| &d.changes)
    {
        assert_eq!(c.harm > 0, c.effect == Effect::Down, "{}", c.cause);
    }
    // Manje opterećenje nego prošli mjesec gura Operativu gore.
    let mut eased = sample();
    eased.load_before_hundredths = 260;
    eased.load_after_hundredths = 200;
    let f = debrief_facts(&eased);
    assert_eq!(
        effect_of(&f, "Opterećenje osoblja", "u ovom mjesecu"),
        Effect::Up
    );
}

const QUIT_NOW: &str = "udio osoblja koje je dalo otkaz ovaj mjesec";

/// Odlasci pod Zaposlenici: udio koji je dao otkaz prema prošlom mjesecu (zadržavanje je 40 % indeksa).
#[test]
fn turnover_rate_against_last_month_moves_staff_index() {
    let f = facts_json(&sample());
    assert_eq!(
        changes(&f, "Zaposleni"),
        [
            (
                QUIT_NOW.to_owned(),
                "4,43 %, više nego prošli mjesec".to_owned()
            ),
            (
                "udio osoblja koje je dalo otkaz prošli mjesec".to_owned(),
                "3,75 %".to_owned()
            ),
            // Broj ljudi ostaje razina.
            (
                "odlasci zaposlenih (zamijenjeni novima)".to_owned(),
                "otišle 3 osobe".to_owned()
            ),
            ("zapošljavanje".to_owned(), "zaposlena 1 osoba".to_owned()),
        ]
    );
    for (before, after, want, words) in [
        (375, 443, Effect::Down, "više nego prošli mjesec"),
        (443, 375, Effect::Up, "manje nego prošli mjesec"),
        (300, 300, Effect::Neutral, "isto kao prošli mjesec"),
        // Jedan od dva mjeseca ispod 8 %: zadržavanje se miče.
        (700, 900, Effect::Down, "više nego prošli mjesec"),
    ] {
        let mut input = sample();
        input.turnover_before_bp = before;
        input.turnover_after_bp = after;
        let facts = debrief_facts(&input);
        assert_eq!(
            effect_of(&facts, "Zaposleni", QUIT_NOW),
            want,
            "{before} → {after}"
        );
        assert_eq!(
            effect_of(&facts, "Zaposleni", "otkaz prošli mjesec"),
            Effect::Neutral
        );
        assert_eq!(
            effect_of(&facts, "Zaposleni", "zamijenjeni"),
            Effect::Neutral
        );
        assert!(
            amount_for(&facts_json(&input), "Zaposleni", QUIT_NOW).ends_with(words),
            "{before} → {after}"
        );
        assert_human_facts(&facts_json(&input));
    }
}

/// Riječi „prošli mjesec" / „prošlog mjeseca" bilo gdje u činjenicama ili tekstu mentora bez AI-ja.
fn mentions_last_month(input: &DebriefInput) -> Vec<String> {
    let facts = serde_json::to_string(&debrief_facts(input)).unwrap();
    let text = fallback_debrief(input);
    [facts, text]
        .iter()
        .flat_map(|t| {
            ["prošli mjesec", "prošlog mjeseca", "prošlom mjesecu"]
                .iter()
                .filter(|w| t.contains(**w))
                .map(|w| format!("{w}: {t}"))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Prvi mjesec igre (igra kaže koji je, `prev_round: None`) nema prošlog: opterećenje, sezonci i odlasci
/// uspoređuju se s početkom igre. Mjesec sam ne odlučuje — srpanj može biti prvi, lipanj ne mora.
#[test]
fn first_month_never_says_last_month_anywhere() {
    let mut first = sample();
    first.prev_round = None;
    first.plan_before = None;
    assert_eq!(mentions_last_month(&first), Vec::<String>::new());
    let f = facts_json(&first);
    assert_eq!(
        f["odluke_u_planu"]["sezonski_radnici"],
        "20 sezonskih radnika, više nego na početku igre"
    );
    assert_eq!(
        changes(&f, "Opterećenje osoblja")[1],
        (
            "opterećenje na početku igre".to_owned(),
            "1,7 noćenja po zaposlenom na dan".to_owned()
        )
    );
    assert_human_facts(&f);
    assert_human(&fallback_debrief(&first));
    // Isti mjesec kad igra ima prošli: usporedbe kažu „prošli mjesec".
    let later = sample();
    let said = mentions_last_month(&later);
    assert!(
        said.iter().any(|s| s.starts_with("prošli mjesec")),
        "srpanj s prošlim mjesecom: {said:?}"
    );
    let f = facts_json(&later);
    assert_eq!(
        f["odluke_u_planu"]["sezonski_radnici"],
        "20 sezonskih radnika, više nego prošli mjesec"
    );
    assert_eq!(
        changes(&f, "Opterećenje osoblja")[1].0,
        "opterećenje prošli mjesec"
    );
    assert!(amount_for(&f, "Zaposleni", QUIT_NOW).ends_with("više nego prošli mjesec"));
    let mut june_not_first = sample();
    june_not_first.round = RoundKey::Month(Month::Jun);
    assert!(!mentions_last_month(&june_not_first).is_empty());
}

/// Prvi mjesec nema prošlog: usporedba je s početkom igre, riječima „na početku igre".
#[test]
fn first_month_compares_turnover_with_start_of_game() {
    let mut june = sample();
    june.round = RoundKey::Month(Month::Jun);
    june.next_round = Some(RoundKey::Month(Month::Jul));
    june.prev_round = None;
    june.turnover_before_bp = 450;
    june.turnover_after_bp = 375;
    let f = facts_json(&june);
    let rows = changes(&f, "Zaposleni");
    assert_eq!(
        rows[..2],
        [
            (
                QUIT_NOW.to_owned(),
                "3,75 %, manje nego na početku igre".to_owned()
            ),
            (
                "udio osoblja koje daje otkaz na početku igre".to_owned(),
                "4,5 %".to_owned()
            ),
        ]
    );
    assert!(
        rows.iter()
            .all(|(c, a)| !c.contains("prošli mjesec") && !a.contains("prošli mjesec")),
        "{rows:?}"
    );
    assert_eq!(
        effect_of(&debrief_facts(&june), "Zaposleni", QUIT_NOW),
        Effect::Up
    );
    for (before, after, words) in [
        (375, 443, "više nego na početku igre"),
        (300, 300, "isto kao na početku igre"),
    ] {
        june.turnover_before_bp = before;
        june.turnover_after_bp = after;
        assert!(
            amount_for(&facts_json(&june), "Zaposleni", QUIT_NOW).ends_with(words),
            "{before} → {after}"
        );
    }
    // Pad Zaposlenika u lipnju okrivljuje odlaske prema početku, ne „prošlom mjesecu".
    june.turnover_before_bp = 375;
    june.turnover_after_bp = 443;
    june.lines
        .retain(|l| l.var != VarCode::Morale || l.cause == CauseDto::Baseline);
    let text = fallback_debrief(&june);
    assert!(
        text.contains(
            "Indeks Zaposlenici pao je ponajviše zbog većeg udjela osoblja koje je dalo otkaz nego na početku igre: kad moral padne ispod 60 bodova, više ljudi daje otkaz."
        ),
        "{text}"
    );
    assert_human(&text);
}

/// Oba mjeseca 8 % i više: zadržavanje je na nuli, pa se indeks zbog odlazaka ne miče — redak je siv
/// i to kaže, umjesto da „više" zvuči kao pad.
#[test]
fn turnover_at_floor_is_grey_and_says_index_cannot_move() {
    for (before, after, cmp) in [(900, 1_000, "više"), (1_000, 900, "manje")] {
        let mut input = sample();
        input.turnover_before_bp = before;
        input.turnover_after_bp = after;
        let facts = debrief_facts(&input);
        assert_eq!(
            effect_of(&facts, "Zaposleni", QUIT_NOW),
            Effect::Neutral,
            "{before} → {after}"
        );
        let pct = |bp: u16| format!("{}", bp / 100);
        assert_eq!(
            amount_for(&facts_json(&input), "Zaposleni", QUIT_NOW),
            format!(
                "{} %, {cmp} nego prošli mjesec, ali od 8 % naviše taj je dio indeksa već na najnižem, pa se ne miče",
                pct(after)
            )
        );
        assert_human_facts(&facts_json(&input));
    }
    // Isti udio na dnu: „isto" je istina i bez dodatka.
    let mut same = sample();
    same.turnover_before_bp = 900;
    same.turnover_after_bp = 900;
    assert_eq!(
        amount_for(&facts_json(&same), "Zaposleni", QUIT_NOW),
        "9 %, isto kao prošli mjesec"
    );
    // Značenje ne tvrdi da veći udio uvijek ruši indeks.
    let f = debrief_facts(&sample());
    let meaning = f
        .indices
        .iter()
        .flat_map(|i| &i.drivers)
        .find(|d| d.what == "Zaposleni")
        .unwrap()
        .meaning;
    assert!(meaning.contains("od 8 % naviše"), "{meaning}");
}

/// Ukupna promjena morala je uzrok samo kad je jedini redak morala; uz pritiske je zbroj (siva).
#[test]
fn total_morale_change_is_a_cause_only_when_alone() {
    let mut alone = sample();
    alone
        .lines
        .retain(|l| l.var != VarCode::Morale || l.cause == CauseDto::Baseline);
    // Pritisak koji zaokruženo daje nulu se ne vidi, pa je ukupna promjena i dalje sama.
    alone.lines.push(line(
        VarCode::Morale,
        CauseDto::Training,
        30,
        UnitCode::Points,
    ));
    let total = |input: &DebriefInput| {
        debrief_facts(input)
            .indices
            .into_iter()
            .flat_map(|i| i.drivers)
            .flat_map(|d| d.changes)
            .find(|c| c.cause.contains("ukupna promjena morala"))
            .unwrap()
    };
    let down = total(&alone);
    assert_eq!((down.effect, down.harm > 0), (Effect::Down, true));
    let mut rose = alone.clone();
    for l in rose.lines.iter_mut().filter(|l| l.var == VarCode::Morale) {
        l.amount = l.amount.abs();
    }
    let up = total(&rose);
    assert_eq!((up.effect, up.harm), (Effect::Up, 0));
    let with_pressures = total(&sample());
    assert_eq!(
        (with_pressures.effect, with_pressures.harm),
        (Effect::Neutral, 0)
    );
}

/// Mentor bez AI-ja okrivljuje odlaske ili pad morala prirodnom rečenicom.
#[test]
fn fallback_blames_departures_or_falling_morale_for_staff_index() {
    let mut alone = sample();
    alone
        .lines
        .retain(|l| l.var != VarCode::Morale || l.cause == CauseDto::Baseline);
    // Moral −4,53 boda ispod 60: stopa odlazaka raste za 0,68 % (zadržavanje −8,5 boda · 40 %)
    // i to gura Zaposlenike više od samog morala (−4,53 · 60 %).
    let text = fallback_debrief(&alone);
    assert!(
        text.contains(
            "Indeks Zaposlenici pao je ponajviše zbog većeg udjela osoblja koje je dalo otkaz nego prošli mjesec: kad moral padne ispod 60 bodova, više ljudi daje otkaz."
        ),
        "{text}"
    );
    assert_human(&text);
    // Isti pad morala iznad 60 bodova (70 → 65,47): stopa odlazaka ostaje 3 %, kriv je sam moral.
    let mut same_rate = alone.clone();
    same_rate.turnover_before_bp = 300;
    same_rate.turnover_after_bp = 300;
    let text = fallback_debrief(&same_rate);
    assert!(
        text.contains(
            "Indeks Zaposlenici pao je ponajviše zbog pada morala, pa je tim slabije raspoložen i lakše odlazi."
        ),
        "{text}"
    );
    assert_human(&text);
}

fn effect_word(e: Effect) -> &'static str {
    match e {
        Effect::Up => "up",
        Effect::Down => "down",
        Effect::Neutral => "neutral",
    }
}

/// Blok Zaposlenici kao na ekranu (uzrok, iznos, smjer), za izvještaj i provjeru.
fn staff_block(f: &DebriefFacts) -> Value {
    let z = f.indices.iter().find(|i| i.name == "Zaposlenici").unwrap();
    serde_json::json!({
        "name": z.name,
        "before": z.before,
        "after": z.after,
        "change": z.change,
        "drivers": z.drivers.iter().map(|d| serde_json::json!({
            "name": d.what,
            "lines": d.changes.iter().map(|c| serde_json::json!({
                "cause": c.cause,
                "amount": c.amount,
                "effect": effect_word(c.effect),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// Stvarni lipanj i srpanj: kad se Zaposlenici pomaknu, barem jedan redak pod njima ima smjer.
#[test]
fn real_months_colour_a_cause_under_staff_index() {
    let (g, h) = july_resolved_bad_chef_game();
    for m in [Month::Jun, Month::Jul] {
        let input = g.debrief_input(h, m).unwrap();
        // Igra kaže koji je mjesec prvi; u prvom nigdje nema „prošlog mjeseca".
        if m == Month::Jun {
            assert_eq!(input.prev_round, None);
            assert_eq!(mentions_last_month(&input), Vec::<String>::new());
        } else {
            assert_eq!(input.prev_round, Some(RoundKey::Month(Month::Jun)));
            assert!(!mentions_last_month(&input).is_empty());
        }
        let f = debrief_facts(&input);
        let block = staff_block(&f);
        // Za izvještaj: `cargo test -p game --test mentor -- --nocapture real_months`.
        println!("{m:?}\n{}", serde_json::to_string_pretty(&block).unwrap());
        let z = f.indices.iter().find(|i| i.name == "Zaposlenici").unwrap();
        if z.change != 0 {
            assert!(
                z.drivers
                    .iter()
                    .flat_map(|d| &d.changes)
                    .any(|c| c.effect != Effect::Neutral),
                "{m:?}: Zaposlenici {:+} bez obojanog uzroka: {block}",
                z.change
            );
        }
        for c in f
            .indices
            .iter()
            .flat_map(|i| &i.drivers)
            .flat_map(|d| &d.changes)
        {
            assert_eq!(c.harm > 0, c.effect == Effect::Down, "{}", c.cause);
        }
        assert_human_facts(&facts_json(&input));
        assert_human(&fallback_debrief(&input));
    }
}

#[test]
fn effect_stays_out_of_mentor_json() {
    let text = serde_json::to_string(&debrief_facts(&sample())).unwrap();
    for word in ["effect", "harm", "\"up\"", "\"down\"", "neutral\""] {
        assert!(!text.contains(word), "{word} u mentorovu JSON-u");
    }
}

#[test]
fn index_code_follows_index_order() {
    let f = debrief_facts(&sample());
    let codes: Vec<IndexCode> = f.indices.iter().map(|i| i.index.into()).collect();
    assert_eq!(codes, INDEX_ORDER);
}

// ------------------------------------------------------------ tjedni krug govori o tjednu

/// Isti ručni primjer kao `sample()`, ali kao tjedan 5 (29. 6. – 5. 7.) tjedne igre; uz trening, obnovu, prihod i
/// trošak situacije koju popis situacija ne poznaje, da se vide i ti tekstovi.
fn weekly_sample() -> DebriefInput {
    let mut input = sample();
    input.round = RoundKey::Week(5);
    input.next_round = Some(RoundKey::Week(6));
    input.prev_round = Some(RoundKey::Week(4));
    input.round_kind = "week".into();
    input.first_date = (29, MonthCode::Jun);
    input.last_date = (5, MonthCode::Jul);
    input.plan.training = TrainingCode::Full;
    input.plan.renovation = true;
    input.lines.extend([
        line(
            VarCode::Revenue,
            CauseDto::Baseline,
            9_000_000,
            UnitCode::Cents,
        ),
        line(
            VarCode::Costs,
            CauseDto::Situation("S9".into()),
            400_000,
            UnitCode::Cents,
        ),
        line(
            VarCode::Morale,
            CauseDto::Situation("S9".into()),
            -300,
            UnitCode::Points,
        ),
    ]);
    input
}

fn strings_in(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings_in(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings_in(x, out)),
        _ => {}
    }
}

#[test]
fn weekly_round_texts_never_say_month() {
    let input = weekly_sample();
    let mut texts = Vec::new();
    strings_in(&facts_json(&input), &mut texts);
    texts.push(fallback_debrief(&input));
    // Zadnja rečenica sažetka je o pravom kraju mjeseca („Puni pregled s mentorom stiže na kraju mjeseca").
    let summary = weekly_summary(&input);
    texts.push(
        summary
            .split("\n\nPuni pregled")
            .next()
            .unwrap_or_default()
            .to_owned(),
    );
    let said: Vec<&String> = texts
        .iter()
        .filter(|t| t.to_lowercase().contains("mjesec"))
        .collect();
    assert!(said.is_empty(), "tjedan govori o mjesecu: {said:?}");
    let f = facts_json(&input);
    assert_eq!(f["mjesec"], "u tjednu od 29. lipnja do 5. srpnja");
    assert_eq!(f["iduci_mjesec"], "sljedeći tjedan");
    // Usporedbe i značenja stoje u tjednu.
    let p = &f["odluke_u_planu"];
    assert_eq!(
        p["cijena_sobe"],
        "115 € po noći, 4 % ispod uobičajene cijene za ovaj tjedan"
    );
    assert_eq!(
        p["sezonski_radnici"],
        "20 sezonskih radnika, više nego prošli tjedan"
    );
    assert_eq!(
        p["obuka"],
        "puna obuka (učinak stiže idući tjedan), promijenjeno u odnosu na prošli tjedan"
    );
    assert_eq!(
        p["odrzavanje"],
        "odgođeno (oprema se ovaj tjedan nije popravljala), isto kao prošli tjedan"
    );
    assert_eq!(
        p["obnova"],
        "pokrenuta: plaća se sada, a sobe su bolje od idućeg tjedna"
    );
    assert!(
        f["ljestvica_indeksa"]
            .as_str()
            .unwrap()
            .contains("u istom tjednu")
    );
    assert_eq!(
        changes(&f, "Opterećenje osoblja")[0].0,
        "opterećenje u ovom tjednu"
    );
    assert_eq!(
        changes(&f, "Opterećenje osoblja")[1].0,
        "opterećenje prošli tjedan"
    );
    assert!(
        amount_for(
            &f,
            "Zaposleni",
            "udio osoblja koje je dalo otkaz ovaj tjedan"
        )
        .ends_with("više nego prošli tjedan")
    );
    assert!(
        changes(&f, "Troškovi")
            .iter()
            .any(|(cause, _)| cause == "situacija „situacija iz ovog tjedna“")
    );
}

/// Mjesečni tekstovi ostaju kakvi su bili: riječ „mjesec" stoji gdje je stajala.
#[test]
fn monthly_round_texts_keep_month_wording() {
    let f = facts_json(&sample());
    let mut texts = Vec::new();
    strings_in(&f, &mut texts);
    assert!(
        texts
            .iter()
            .any(|t| t.contains("Što je hotel platio ovaj mjesec, po vrsti troška."))
    );
    assert!(texts.iter().any(|t| t.contains("Svaki mjesec se troši")));
    assert!(texts.iter().any(|t| t.contains("Moral se svaki mjesec")));
    assert!(texts.iter().any(|t| t.contains("koje u mjesecu da otkaz")));
    assert!(
        texts
            .iter()
            .any(|t| t == "udio osoblja koje je dalo otkaz ovaj mjesec")
    );
    assert!(texts.iter().any(|t| t == "opterećenje u ovom mjesecu"));
    assert!(
        f["ljestvica_indeksa"]
            .as_str()
            .unwrap()
            .contains("u istom mjesecu")
    );
    let mut empty = sample();
    empty.situations.clear();
    empty.lines = vec![line(
        VarCode::Costs,
        CauseDto::Situation("S9".into()),
        400_000,
        UnitCode::Cents,
    )];
    assert!(
        changes(&facts_json(&empty), "Troškovi")
            .iter()
            .any(|(cause, _)| cause == "situacija „situacija iz ovog mjeseca“")
    );
}

// ------------------------------------------------------------ odlasci po odjelu (Sloj 1/OT)

fn dep(name: &str, count: u16) -> DepartmentDepartures {
    DepartmentDepartures {
        name: name.into(),
        count,
    }
}

/// `sample()` ima 3 odlaska u hotelu; v2 krug ih raspoređuje po odjelima.
fn with_departures(departures: Vec<DepartmentDepartures>) -> DebriefInput {
    DebriefInput {
        department_departures: departures,
        ..sample()
    }
}

#[test]
fn departures_name_both_departments_with_counts() {
    let input = with_departures(vec![dep("Smještaj", 2), dep("Restoran", 1)]);
    let f = facts_json(&input);
    assert_eq!(
        amount_for(&f, "Zaposleni", "odlasci"),
        "otišle 3 osobe (Smještaj 2, Restoran 1)"
    );
}

#[test]
fn departures_in_one_department() {
    let mut input = with_departures(vec![dep("Restoran", 1)]);
    for l in &mut input.lines {
        if l.var == VarCode::Staff && l.cause == CauseDto::Turnover {
            l.amount = 1;
        }
    }
    let f = facts_json(&input);
    assert_eq!(
        amount_for(&f, "Zaposleni", "odlasci"),
        "otišla 1 osoba (Restoran)"
    );
}

#[test]
fn zero_departures_have_no_parentheses() {
    let mut input = with_departures(Vec::new());
    for l in &mut input.lines {
        if l.var == VarCode::Staff && l.cause == CauseDto::Turnover {
            l.amount = 0;
        }
    }
    let f = facts_json(&input);
    assert!(
        changes(&f, "Zaposleni")
            .iter()
            .all(|(c, _)| !c.contains("odlasci")),
        "redak bez odlazaka se izostavlja"
    );
    assert!(!serde_json::to_string(&f).unwrap().contains("osobe ("));
}

#[test]
fn more_than_three_departments_are_cut_to_three_and_the_rest_counted() {
    let mut input = with_departures(vec![
        dep("Smještaj", 3),
        dep("Restoran", 2),
        dep("Uprava", 1),
        dep("Održavanje", 1),
        dep("Wellness", 1),
    ]);
    for l in &mut input.lines {
        if l.var == VarCode::Staff && l.cause == CauseDto::Turnover {
            l.amount = 8;
        }
    }
    let f = facts_json(&input);
    assert_eq!(
        amount_for(&f, "Zaposleni", "odlasci"),
        "otišlo 8 osoba (Smještaj 3, Restoran 2, Uprava 1 i još 2)"
    );
}

#[test]
fn v1_text_has_no_departments_and_is_unchanged() {
    // v1 igra nema odjela: lista je prazna i tekst je isti kao prije.
    let input = sample();
    assert!(input.department_departures.is_empty());
    let f = facts_json(&input);
    assert_eq!(amount_for(&f, "Zaposleni", "odlasci"), "otišle 3 osobe");
    // I stvarna v1 igra: obračunati mjesec nosi praznu raščlambu, pa nijedan redak o ljudima nema zagradu.
    let (g, h) = july_resolved_bad_chef_game();
    let real = g.debrief_input(h, Month::Jul).unwrap();
    assert!(real.department_departures.is_empty());
    let json = serde_json::to_string(&facts_json(&real)).unwrap();
    assert!(
        !json.contains("osob") || !json.contains("osobe ("),
        "{json}"
    );
    assert!(!json.contains("osoba ("), "{json}");
}

#[test]
fn breakdown_that_does_not_add_up_to_the_hotel_number_is_left_out() {
    // Zaštita: raščlamba koja se ne slaže s hotelskim brojem ne smije zavarati.
    let input = with_departures(vec![dep("Smještaj", 5)]);
    let f = facts_json(&input);
    assert_eq!(amount_for(&f, "Zaposleni", "odlasci"), "otišle 3 osobe");
}

// ------------------------------------------------------------ mentor ne pripisuje studentu preuzeto stanje (OT-2)

fn wages_text(input: &DebriefInput) -> String {
    facts_json(input)["odluke_u_planu"]["place"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn wages_from_profile_are_not_attributed_to_the_student() {
    // Hotel s profilom od 95 % plaća, student ih nije dirao.
    let input = sample();
    assert_eq!((input.wage_start_pct, input.wage_before_pct), (95, 95));
    let text = wages_text(&input);
    assert!(
        text.contains("takve su od početka, student ih nije mijenjao"),
        "{text}"
    );
    assert!(!text.contains("spustio"), "{text}");
}

#[test]
fn wages_the_student_lowered_say_so() {
    let mut input = sample();
    input.wage_start_pct = 100;
    input.wage_before_pct = 100;
    assert_eq!(
        wages_text(&input),
        "95 % uobičajene plaće (niže od uobičajenih; student ih je spustio sa 100 % na 95 %)"
    );
}

#[test]
fn wages_changed_earlier_and_kept_this_round_say_so() {
    let mut input = sample();
    input.wage_start_pct = 100;
    input.wage_before_pct = 95;
    assert_eq!(
        wages_text(&input),
        "95 % uobičajene plaće (niže od uobičajenih; student ih je ranije promijenio sa 100 % na 95 %, a prošli mjesec ih nije mijenjao)"
    );
}

#[test]
fn usual_wages_unchanged_have_no_note() {
    let mut input = sample();
    input.plan.wage_pct = 100;
    input.wage_start_pct = 100;
    input.wage_before_pct = 100;
    assert_eq!(wages_text(&input), "100 % uobičajene plaće");
}

#[test]
fn real_v1_game_tells_who_set_the_wages() {
    // Stvarna igra: u lipnju student ne dira plaće, u srpnju ih spušta sa 100 % na 95 %.
    let (g, h) = july_resolved_bad_chef_game();
    let june = g.debrief_input(h, Month::Jun).unwrap();
    assert_eq!(june.wage_before_pct, june.wage_start_pct);
    let text = wages_text(&june);
    assert!(
        !text.contains("spustio") && !text.contains("podigao"),
        "{text}"
    );
    let july = g.debrief_input(h, Month::Jul).unwrap();
    assert_eq!(
        wages_text(&july),
        "95 % uobičajene plaće (niže od uobičajenih; student ih je spustio sa 100 % na 95 %)"
    );
}
