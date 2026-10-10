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
    }
}

const CHEF: &str = "Šef kuhinje traži povišicu ili odlazi";
const CHEF_ANSWER: &str = "Bonus na kraju sezone ako ostane, bez razgovora";

/// Srpanj kao iz Leonova primjera, složen ručno da su brojke poznate.
fn sample() -> DebriefInput {
    DebriefInput {
        month: MonthCode::Jul,
        next_month: Some(MonthCode::Aug),
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
    // Varijabla bez ijedne promjene ne pojavljuje se uz indeks.
    let f = facts_json(&input);
    assert!(
        index(&f, "Zaposlenici")["sto_je_pokrenulo"]
            .as_array()
            .unwrap()
            .is_empty()
    );
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
    last.month = MonthCode::Sep;
    last.next_month = None;
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
        input.next_month, None,
        "u testnom sadržaju srpanj je zadnji"
    );
    let json = serde_json::to_string_pretty(&debrief_facts(&input)).unwrap();
    assert_human_facts(&facts_json(&input));
    let text = fallback_debrief(&input);
    assert_human(&text);
    // Za izvještaj: `cargo test -p game --test mentor -- --nocapture guard`.
    println!("{json}\n-----\n{text}");
    let june = g.debrief_input(h, Month::Jun).unwrap();
    assert_eq!(june.next_month, Some(MonthCode::Jul));
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
    let july_start = g.state_at(h, Month::Jul).unwrap();
    assert_eq!(
        (input.plan.wage_pct, input.seasonal_before),
        (95, july_start.seasonal)
    );
    assert!(input.price_ref_cents > 0);
    assert_eq!(
        input.load_after_hundredths,
        g.latest_state(h).unwrap().load_hundredths
    );
    let cash = g.latest_state(h).unwrap().cash_cents;
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
    assert_eq!(p["place"], "95 % uobičajene plaće (niže od uobičajenih)");
    assert_eq!(p["obuka"], "bez obuke");
    assert_eq!(
        p["odrzavanje"],
        "odgođeno (oprema se ovaj mjesec nije popravljala)"
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
    assert_eq!(p["place"], "100 % uobičajene plaće");
    assert_eq!(p["odrzavanje"], "pojačano");
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
