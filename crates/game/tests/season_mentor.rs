//! Sezonski mentor (kraj-sezone-spec §6.3, §7): činjenice cijele sezone u ljudskim jedinicama, rezerva bez AI-ja s
//! četiri dijela i događaj `SeasonDebriefRecorded` (zapis, fold, jedan po hotelu, tek kad je sezona gotova).

mod common;

use game::*;
use serde_json::{Value, json};

fn me() -> HotelId {
    common::hotel_ids()[0]
}

fn weekly_done() -> Game {
    common::v2_weekly_game_resolved_through("t17")
}

fn monthly_done() -> Game {
    common::v2_monthly_game_resolved_through("sep")
}

/// Tjedna sezona do kraja u kojoj student u t03 sam postavi plaće na 105 % (ostalo zadano).
fn weekly_with_raised_wages() -> Game {
    let mut g = common::v2_weekly_game_resolved_through("none");
    while let Some(r) = g.current_round() {
        if r == RoundKey::Week(3) {
            let mut dto = g.effective_plan(me(), r).unwrap().to_dto();
            dto.wage_pct = 105;
            let plan = g.plan_from_dto(me(), r, &dto).unwrap();
            let ev = g.set_plan(me(), r, plan).unwrap();
            common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        }
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("obračun je trebao biti gotov");
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

fn facts_json(g: &Game) -> Value {
    let input = g.season_debrief_input(me()).unwrap();
    serde_json::to_value(season_debrief_facts(&input)).unwrap()
}

// ---------------------------------------------------------------- čuvari teksta

/// Jedinice iza kojih broj smije stajati (sve ostalo je sirov broj).
const UNITS: [&str; 16] = [
    "bodova", "boda", "bod", "€", "%", "/5", "situacij", "sezonsk", "tjed", "mjesec", "mjesta",
    "mjestu", "mjesto", "hotel", "noći", "radnik",
];

const MONTHS_GENITIVE: [&str; 4] = ["lipnja", "srpnja", "kolovoza", "rujna"];

/// Svaki broj u tekstu ima jedinicu ili je datum („3. kolovoza"). Iznimka (`index_prose`): vrijednost indeksa u
/// rečenici o indeksu („Indeks Zaposlenici porastao je s 30 na 52."), jer je indeks sam svoja ljestvica.
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
        // redni broj ili datum: „2. mjesto", „3. kolovoza"
        if chars.get(k) == Some(&'.') {
            k += 1;
        }
        while chars.get(k) == Some(&' ') {
            k += 1;
        }
        let rest: String = chars[k..].iter().collect();
        if let Some(u) = UNITS
            .iter()
            .chain(&MONTHS_GENITIVE)
            .find(|u| rest.starts_with(**u))
        {
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

/// Zabranjeni ostaci internih imena: kodovi krugova, jedinice, engleski mjeseci, šifre strategija.
fn assert_no_internal_words(text: &str) {
    for bad in [
        "bp",
        "cents",
        "centi",
        "hundredths",
        "stotink",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "stedisa",
        "brizni",
        "lovac",
        "nasumicni",
        "Week",
        "Month",
    ] {
        assert!(!text.contains(bad), "„{bad}“ u tekstu:\n{text}");
    }
    let chars: Vec<char> = text.chars().collect();
    for w in chars.windows(3) {
        assert!(
            !(w[0] == 't' && w[1].is_ascii_digit() && w[2].is_ascii_digit()),
            "kod kruga u tekstu:\n{text}"
        );
        assert!(
            !(w[0] == 'S' && w[1].is_ascii_digit()),
            "šifra situacije u tekstu:\n{text}"
        );
    }
}

/// Gole brojke samo pod `na_pocetku`/`na_kraju`/`promjena` (0–100), svaki tekst s jedinicama.
fn assert_human_facts(f: &Value) {
    fn walk(v: &Value, key: &str) {
        match v {
            Value::Number(n) => {
                let n = n.as_i64().unwrap_or(i64::MAX);
                let ok = match key {
                    "na_pocetku" | "na_kraju" => (0..=100).contains(&n),
                    "promjena" => (-100..=100).contains(&n),
                    _ => false,
                };
                assert!(ok, "gola brojka {n} pod „{key}“");
            }
            Value::String(s) => {
                assert_no_internal_words(s);
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

// ---------------------------------------------------------------- činjenice

#[test]
fn facts_are_human_for_weekly_and_monthly_season() {
    for g in [weekly_done(), monthly_done(), weekly_with_raised_wages()] {
        let f = facts_json(&g);
        assert_human_facts(&f);
        assert_eq!(f["hotel"], "Tvoj hotel");
        assert_eq!(f["indeksi"].as_array().unwrap().len(), 5);
        assert_eq!(f["ostali_hoteli"].as_array().unwrap().len(), 3);
        let text = f.to_string();
        assert!(!text.contains("t05") && !text.contains("\"sep\""), "{text}");
    }
}

#[test]
fn season_length_and_dates_in_words() {
    let w = facts_json(&weekly_done());
    assert_eq!(w["sezona"], "17 tjedana, od 1. lipnja do 30. rujna");
    let m = facts_json(&monthly_done());
    assert_eq!(m["sezona"], "4 mjeseca, od 1. lipnja do 30. rujna");
}

#[test]
fn place_progress_and_winner_match_season_view() {
    for g in [weekly_done(), monthly_done()] {
        let v = g.season(me()).unwrap();
        let mine = v.hotels.iter().find(|h| h.mine).unwrap();
        let f = facts_json(&g);
        let p = (mine.progress_hundredths
            + if mine.progress_hundredths < 0 {
                -50
            } else {
                50
            })
            / 100;
        let words = f["napredak_od_pocetka"].as_str().unwrap();
        assert!(
            words.contains(&p.unsigned_abs().to_string()),
            "{words} / {p}"
        );
        let place = f["mjesto"].as_str().unwrap();
        assert!(
            place.starts_with(&format!("{}. mjesto od 4 hotela", mine.rank)),
            "{place}"
        );
        let winner = v.hotels.iter().find(|h| h.winner).unwrap();
        let w = f["pobjednik"].as_str().unwrap();
        if winner.mine {
            assert!(w.contains("Tvoj hotel"), "{w}");
        } else {
            assert!(w.contains(&winner.name) && w.contains("(robot)"), "{w}");
        }
        // Ostali hoteli redom poretka, robot s rečenicom strategije i odlukama, bez šifre strategije.
        let others = f["ostali_hoteli"].as_array().unwrap();
        let names: Vec<&str> = others.iter().map(|o| o["ime"].as_str().unwrap()).collect();
        let expected: Vec<&str> = v
            .hotels
            .iter()
            .filter(|h| !h.mine)
            .map(|h| h.name.as_str())
            .collect();
        assert_eq!(names, expected);
        for o in others {
            assert_eq!(o["vrsta"], "robot");
            assert!(o["kako_igra"].as_str().unwrap().len() > 40);
            assert!(o["odluke"].is_object(), "{o}");
            assert!(o["mjesto"].as_str().unwrap().contains(". mjesto od 4"));
        }
    }
}

/// OT-2: plaće koje student nikad nije dirao su „takve od početka" i plan je zadani — ne pripisuje mu se ništa.
#[test]
fn untouched_wages_and_plan_are_not_attributed_to_student() {
    let f = facts_json(&weekly_done());
    let wages = f["odluke"]["place"].as_str().unwrap();
    assert!(
        wages.contains("takve su od početka, student ih nije mijenjao"),
        "{wages}"
    );
    let own = f["vlastiti_plan"].as_str().unwrap();
    assert!(own.contains("nijednom nije mijenjao plan"), "{own}");
}

#[test]
fn changed_wages_show_range_and_first_own_plan() {
    let g = weekly_with_raised_wages();
    let f = facts_json(&g);
    let wages = f["odluke"]["place"].as_str().unwrap();
    assert!(!wages.contains("takve su od početka"), "{wages}");
    assert!(wages.contains("105 %"), "{wages}");
    let own = f["vlastiti_plan"].as_str().unwrap();
    assert!(
        own.contains("u tjednu od 15. lipnja do 21. lipnja"),
        "{own}"
    );
    assert!(own.contains("zadani plan"), "{own}");
}

#[test]
fn robot_decisions_follow_their_strategy() {
    let f = facts_json(&weekly_done());
    let others = f["ostali_hoteli"].as_array().unwrap();
    let brizni = others.iter().find(|o| o["ime"] == "Brižni").unwrap();
    assert_eq!(
        brizni["odluke"]["place"],
        "103 % uobičajene plaće cijelu sezonu"
    );
    assert_eq!(brizni["odluke"]["obuka"], "obuka svaki tjedan");
    let stedisa = others.iter().find(|o| o["ime"] == "Štediša").unwrap();
    assert_eq!(
        stedisa["odluke"]["marketing"],
        "bez marketinga cijelu sezonu"
    );
    assert!(
        stedisa["kako_igra"]
            .as_str()
            .unwrap()
            .starts_with("Štediša je cijelu sezonu čuvao novac")
    );
}

#[test]
fn best_and_worst_round_follow_the_curve() {
    let g = weekly_done();
    let v = g.season(me()).unwrap();
    let mine = v.hotels.iter().find(|h| h.mine).unwrap();
    let mut prev = 0;
    let deltas: Vec<i32> = mine
        .curve
        .iter()
        .map(|c| {
            let d = c.progress_hundredths - prev;
            prev = c.progress_hundredths;
            d
        })
        .collect();
    let f = facts_json(&g);
    let max = *deltas.iter().max().unwrap();
    let min = *deltas.iter().min().unwrap();
    // Krug se navodi kad je pomak bar desetinka boda (5 stotinki zaokruženo).
    assert_eq!(f["najbolji_krug"].is_object(), max >= 5, "{f}");
    assert_eq!(f["najslabiji_krug"].is_object(), min <= -5, "{f}");
    if max >= 5 {
        let pos = deltas.iter().position(|&d| d == max).unwrap();
        let first = v.rounds[pos].period.first_date();
        let krug = f["najbolji_krug"]["krug"].as_str().unwrap();
        assert!(
            krug.starts_with(&format!("tjedan od {}. ", first.0)),
            "{krug}"
        );
        assert!(
            f["najbolji_krug"]["promjena_napretka"]
                .as_str()
                .unwrap()
                .starts_with('+')
        );
    }
}

#[test]
fn situations_in_words() {
    let g = weekly_done();
    let v = g.season(me()).unwrap();
    let d = v
        .hotels
        .iter()
        .find(|h| h.mine)
        .unwrap()
        .decisions
        .clone()
        .unwrap();
    let f = facts_json(&g);
    let s = f["situacije"].as_str().unwrap();
    assert!(s.starts_with(&d.situations.to_string()), "{s}");
    assert!(s.contains("/5"), "{s}");
    if d.unanswered > 0 {
        assert!(s.contains("bez odgovora"), "{s}");
    }
}

// ---------------------------------------------------------------- rezerva

const PARTS: [&str; 4] = [
    "Kako je prošla sezona",
    "Što je išlo dobro",
    "Što nije valjalo",
    "Što ponijeti u sljedeću igru",
];

#[test]
fn fallback_has_four_parts_and_honest_last_line() {
    for g in [weekly_done(), monthly_done(), weekly_with_raised_wages()] {
        let input = g.season_debrief_input(me()).unwrap();
        let text = fallback_season_debrief(&input);
        let heads: Vec<&str> = text.lines().filter(|l| PARTS.contains(l)).collect();
        assert_eq!(heads, PARTS, "{text}");
        assert!(text.starts_with("Kako je prošla sezona\n"), "{text}");
        assert!(
            text.ends_with("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki."),
            "{text}"
        );
        assert_no_internal_words(&text);
        assert_numbers_have_units(&text, true);
        // Bez roda prema studentu.
        for gendered in ["si pobijedio", "si pobijedila", "bio si", "bila si"] {
            assert!(!text.contains(gendered), "{text}");
        }
        assert_eq!(text, fallback_season_debrief(&input), "deterministično");
    }
}

#[test]
fn fallback_names_place_and_winner() {
    let g = weekly_done();
    let v = g.season(me()).unwrap();
    let mine = v.hotels.iter().find(|h| h.mine).unwrap();
    let text = fallback_season_debrief(&g.season_debrief_input(me()).unwrap());
    assert!(
        text.contains(&format!("{}. mjesto od 4 hotela", mine.rank)),
        "{text}"
    );
    let winner = v.hotels.iter().find(|h| h.winner).unwrap();
    if winner.mine {
        assert!(text.contains("Tvoj hotel je pobijedio"), "{text}");
    } else {
        assert!(
            text.contains(&format!("Pobijedio je {} (robot)", winner.name)),
            "{text}"
        );
        // Usporedba s robotom ispred studenta navodi njegove odluke.
        assert!(text.contains("(robot) je imao plaće"), "{text}");
    }
}

/// Namješten ishod: student zadnji, a nijedan indeks mu nije pao — rezerva to kaže iskreno.
#[test]
fn fallback_says_honestly_when_no_index_fell() {
    let g = weekly_done();
    let mut input = g.season_debrief_input(me()).unwrap();
    input.me.hotel.end = input.me.hotel.start;
    let text = fallback_season_debrief(&input);
    assert!(
        text.contains("Nijedan indeks nije završio niže nego na početku."),
        "{text}"
    );
    assert!(
        text.contains("Nijedan indeks nije završio više nego na početku."),
        "{text}"
    );
}

#[test]
fn fallback_counts_unanswered_situations() {
    let g = weekly_done();
    let mut input = g.season_debrief_input(me()).unwrap();
    if let Some(d) = input.me.hotel.decisions.as_mut() {
        d.unanswered = 3;
    }
    let text = fallback_season_debrief(&input);
    assert!(
        text.contains("Na 3 situacije nije bilo odgovora."),
        "{text}"
    );
}

// ---------------------------------------------------------------- događaj

#[test]
fn input_is_only_for_finished_v2_season() {
    let g = common::v2_weekly_game_resolved_through("t16");
    assert_eq!(
        g.season_debrief_input(me()).unwrap_err(),
        GameError::SeasonNotOver
    );
    let v1 = common::june_resolved_game();
    assert_eq!(
        v1.season_debrief_input(me()).unwrap_err(),
        GameError::WrongModel
    );
    assert!(wants_season_mentor(&weekly_done(), RoundKey::Week(17)));
    assert!(!wants_season_mentor(&weekly_done(), RoundKey::Week(16)));
    assert!(wants_season_mentor(
        &monthly_done(),
        RoundKey::Month(sim_hotel::Month::Sep)
    ));
    assert!(!wants_season_mentor(
        &v1,
        RoundKey::Month(sim_hotel::Month::Jul)
    ));
}

#[test]
fn season_debrief_is_recorded_once_and_shown_in_season_view() {
    let mut g = monthly_done();
    let ev = g
        .record_season_debrief(me(), "Sezona je prošla dobro.".into(), AiSource::Ai, None)
        .unwrap()
        .expect("prvi zapis");
    assert_eq!(ev.kind(), "SeasonDebriefRecorded");
    common::apply_ev(&mut g, Actor::System, ev.clone());
    assert_eq!(
        g.season(me()).unwrap().mentor.as_deref(),
        Some("Sezona je prošla dobro.")
    );
    // Robot ga ne vidi kao svoj (svaki hotel ima svoj).
    assert_eq!(g.season(common::hotel_ids()[1]).unwrap().mentor, None);
    assert_eq!(
        g.record_season_debrief(me(), "drugi".into(), AiSource::Fallback, None)
            .unwrap(),
        None,
        "drugi zapis ne nastaje"
    );
    // Dvostruki zapis u dnevniku je pokvaren dnevnik.
    let mut log = g.history.clone();
    common::push(&mut log, Actor::System, ev);
    assert!(Game::fold(&log).is_err());
}

#[test]
fn season_debrief_before_season_end_is_rejected() {
    let mut g = common::v2_weekly_game_resolved_through("t16");
    assert_eq!(
        g.record_season_debrief(me(), "x".into(), AiSource::Fallback, None)
            .unwrap_err(),
        GameError::SeasonNotOver
    );
    // Ni dnevnik ga ne prima prije kraja sezone.
    let ev = GameEvent::SeasonDebriefRecorded(SeasonDebriefRecorded {
        hotel_id: me(),
        text: "x".into(),
        source: AiSource::Fallback,
        meta: None,
    });
    let seq = g.last_seq + 1;
    assert!(
        g.apply(Recorded {
            seq,
            actor: Actor::System,
            event: ev
        })
        .is_err()
    );
}

/// Isti JSON koji `store/tests/events.rs` provlači kroz bazu.
#[test]
fn season_debrief_event_roundtrips_through_record() {
    let payload = json!({
        "hotel_id": "00000000-0000-0000-0000-00000000000b",
        "text": "Kako je prošla sezona\nNapredak je +8 bodova.",
        "source": "ai",
        "meta": { "model": "claude-sonnet-5-5", "prompt": "mentor-sezona.v1", "input_tokens": 10, "output_tokens": 20 }
    });
    let e = GameEvent::from_record("SeasonDebriefRecorded", SCHEMA_VER, payload.clone()).unwrap();
    let GameEvent::SeasonDebriefRecorded(d) = &e else {
        panic!("{e:?}");
    };
    assert_eq!(d.hotel_id, me());
    assert_eq!(d.source, AiSource::Ai);
    assert_eq!(e.to_payload(), payload);
    let fallback = GameEvent::SeasonDebriefRecorded(SeasonDebriefRecorded {
        hotel_id: me(),
        text: "x".into(),
        source: AiSource::Fallback,
        meta: None,
    });
    assert!(fallback.to_payload().get("meta").is_none());
    assert_eq!(
        GameEvent::from_record(fallback.kind(), SCHEMA_VER, fallback.to_payload()).unwrap(),
        fallback
    );
}

// ---------------------------------------------------------------- popravak 1 (pregled KS-2)

/// Različito mjesto, a zaokruženo isti bodovi (+8,4 i +7,6 → „+8"): tada se napredak piše s desetinkama.
#[test]
fn close_progress_is_shown_with_tenths() {
    let g = weekly_done();
    let mut input = g.season_debrief_input(me()).unwrap();
    input.me.hotel.progress_hundredths = 760;
    input.me.hotel.rank = 2;
    input.me.hotel.winner = false;
    for (i, o) in input.others.iter_mut().enumerate() {
        let (rank, p) = [(1, 840), (3, -300), (4, -600)][i];
        o.hotel.rank = rank;
        o.hotel.winner = rank == 1;
        o.hotel.progress_hundredths = p;
    }
    let f = serde_json::to_value(season_debrief_facts(&input)).unwrap();
    assert_eq!(f["napredak_od_pocetka"], "+7,6 boda");
    assert!(
        f["pobjednik"].as_str().unwrap().contains("+8,4 boda"),
        "{f}"
    );
    assert_eq!(f["ostali_hoteli"][1]["napredak_od_pocetka"], "−3 boda");
    // Kad se cijeli bodovi razlikuju, ostaju cijeli (kao na ekranu).
    input.me.hotel.progress_hundredths = 520;
    let f = serde_json::to_value(season_debrief_facts(&input)).unwrap();
    assert_eq!(f["napredak_od_pocetka"], "+5 bodova");
}

/// `carry_plan` plan prenosi uz prilagodbu (obnova se ne ponavlja, iznosi u granice kruga) — tekst to kaže.
#[test]
fn carried_plan_is_described_as_carried_and_adapted() {
    let f = facts_json(&weekly_with_raised_wages());
    let own = f["vlastiti_plan"].as_str().unwrap();
    assert!(!own.contains("zadnji postavljeni plan"), "{own}");
    assert!(own.contains("prenosio"), "{own}");
    assert!(own.contains("granic"), "{own}");
}

/// Usporedba s robotom ne bira indeks u kojem je studentov hotel bolji; tada ostaju samo robotove odluke.
#[test]
fn robot_comparison_skips_index_where_student_did_better() {
    let g = weekly_done();
    let mut input = g.season_debrief_input(me()).unwrap();
    input.me.hotel.end = [10_000; 5];
    let text = fallback_season_debrief(&input);
    let tip = text.split("Što ponijeti u sljedeću igru\n").nth(1).unwrap();
    assert!(tip.contains("(robot) je imao plaće"), "{text}");
    assert!(!tip.contains("mu je"), "{text}");
}

/// Tekst AI-ja se sprema samo ako ima sva četiri dijela; inače radnik piše rezervu.
#[test]
fn season_debrief_text_needs_all_four_parts() {
    let input = weekly_done().season_debrief_input(me()).unwrap();
    assert!(season_debrief_has_parts(&fallback_season_debrief(&input)));
    assert!(!season_debrief_has_parts(""));
    assert!(!season_debrief_has_parts("   \n"));
    assert!(!season_debrief_has_parts(
        "Kako je prošla sezona\nDobro.\nŠto je išlo dobro\nSve.\nŠto nije valjalo\nNišta."
    ));
    assert!(season_debrief_has_parts(
        "Kako je prošla sezona\nDobro.\n\n  Što je išlo dobro \nSve.\nŠto nije valjalo\nNišta.\nŠto ponijeti u sljedeću igru\nObuku."
    ));
}
