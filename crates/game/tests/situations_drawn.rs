//! G-1: situacije iz predložaka u igri. Pri otvaranju kruga igra zapiše događaj `SituationsDrawn` (primjerci
//! generatora po hotelu); `Game::situations_in` za v2 igru s predlošcima čita primjerke iz dnevnika, a roboti
//! odgovaraju na njih. Igra bez tog događaja (v2 igra nastala između S-1 i G-1, v1) igra po starom pravilu.
//!
//! Događaj se prepoznaje po `GameEvent::kind() == "SituationsDrawn"` i JSON-u `to_payload()`, a ne po imenu
//! varijante: test tako PADA PONAŠANJEM (ne prevođenjem) dok se događaj ne uvede. Pretpostavke o obliku zapisa
//! (jedino mjesto: `drawn_items`): svaki primjerak je JSON objekt s poljima `code`, `template`, `day`, `slot`,
//! `occurrence`; pretpostavka o sučelju (jedino mjesto: `situations_of`): `Game::situations_in(krug)` daje
//! situacije kruga za hotel igrača (utor 1).

mod common;

use std::collections::BTreeMap;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game, new_game_with};
use game::event::{Controller, SCHEMA_VER};
use game::*;
use serde_json::Value;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

const DRAWN: &str = "SituationsDrawn";

fn events_of(snap: &Snapshot, seed: u64, kind: PeriodKind) -> Vec<Recorded> {
    new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings { kind },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect()
}

fn v2_events(seed: u64, kind: PeriodKind) -> Vec<Recorded> {
    events_of(&Snapshot::read_v2(&repo_dir()).unwrap(), seed, kind)
}

/// Odigra sve krugove (hotel igrača ne odgovara ni na što) i vrati igru; dnevnik je `g.history`.
fn play_all(mut g: Game) -> Game {
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("čeka")
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

/// Šifre situacija kruga za hotel igrača.
fn situations_of(g: &Game, r: RoundKey) -> Vec<String> {
    g.situations_in(r).iter().map(|t| t.code.clone()).collect()
}

fn drawn_events(log: &[Recorded]) -> Vec<&Recorded> {
    log.iter().filter(|r| r.event.kind() == DRAWN).collect()
}

/// Svi primjerci u zapisu događaja (objekti s `code` i `template`), bez obzira na ugniježđenje.
fn drawn_items(v: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    match v {
        Value::Object(m) => {
            if m.get("code").is_some_and(Value::is_string)
                && m.get("template").is_some_and(Value::is_string)
            {
                out.push(v);
            }
            for x in m.values() {
                out.extend(drawn_items(x));
            }
        }
        Value::Array(a) => {
            for x in a {
                out.extend(drawn_items(x));
            }
        }
        _ => {}
    }
    out
}

fn all_items(log: &[Recorded]) -> Vec<Value> {
    drawn_events(log)
        .iter()
        .flat_map(|r| {
            let payload = r.event.to_payload();
            drawn_items(&payload)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Krug primjerka iz šifre (`P-KLIMA.t03`, `P-KLIMA.aug.2`).
fn round_of(code: &str) -> RoundKey {
    RoundKey::parse(code.split('.').nth(1).expect("šifra primjerka ima krug")).unwrap()
}

// ---------------------------------------------------------------------------------------------------------------
// Događaj pri otvaranju kruga

#[test]
fn opening_the_first_round_records_situations_drawn_before_robot_answers() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let log = v2_events(42, kind);
        let first_drawn = log
            .iter()
            .position(|r| r.event.kind() == DRAWN)
            .unwrap_or_else(|| panic!("{kind:?}: nema SituationsDrawn pri stvaranju igre"));
        let first_answer = log
            .iter()
            .position(|r| r.event.kind() == "AnswerChosen")
            .expect("roboti odgovaraju");
        assert!(
            first_drawn < first_answer,
            "{kind:?}: roboti ne mogu odgovoriti prije nego što su situacije izvučene"
        );
        assert!(first_drawn > 0, "{kind:?}: nakon GameCreated");
        assert!(
            !all_items(&log).is_empty(),
            "{kind:?}: događaj bez primjeraka"
        );
    }
}

#[test]
fn every_round_of_a_v2_game_is_opened_with_drawn_situations() {
    for (kind, lo, hi) in [(PeriodKind::Week, 3, 5), (PeriodKind::Month, 13, 22)] {
        let g = play_all(Game::fold(&v2_events(42, kind)).unwrap());
        let rounds = g.rounds.len();
        assert!(
            drawn_events(&g.history).len() >= rounds,
            "{kind:?}: manje od jednog SituationsDrawn po krugu ({rounds} krugova)"
        );
        for r in &g.rounds {
            let n = situations_of(&g, *r).len();
            assert!(
                (lo..=hi).contains(&n),
                "{kind:?} krug {}: {n} situacija, očekivano {lo}–{hi} (sidra + primjerci)",
                r.code()
            );
        }
    }
}

#[test]
fn drawn_situations_live_in_the_round_they_were_drawn_for_and_anchors_stay() {
    let g = play_all(Game::fold(&v2_events(42, PeriodKind::Week)).unwrap());
    let items = all_items(&g.history);
    assert!(!items.is_empty());
    for it in &items {
        let code = it["code"].as_str().unwrap();
        assert_eq!(g.situation_round(code).unwrap().0, round_of(code), "{code}");
    }
    // Sidra S1–S4 ostaju u svojim krugovima (staro pravilo za sidra) i broje se u 3–5.
    for info in g.rounds_info() {
        for s in sim_hotel_v2::situations_for(&info.period, &g.content.situations) {
            assert!(
                situations_of(&g, info.key).contains(&s.code),
                "sidro {} nedostaje u krugu {}",
                s.code,
                info.key.code()
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Fold je čist: isti dnevnik, ista igra; zapis se vraća bez gubitka

#[test]
fn folding_the_same_log_twice_gives_the_same_game() {
    let g = play_all(Game::fold(&v2_events(42, PeriodKind::Week)).unwrap());
    let a = Game::fold(&g.history).unwrap();
    let b = Game::fold(&g.history).unwrap();
    assert!(!drawn_events(&g.history).is_empty());
    for r in &g.rounds {
        assert_eq!(situations_of(&a, *r), situations_of(&b, *r), "{}", r.code());
        assert_eq!(situations_of(&a, *r), situations_of(&g, *r), "{}", r.code());
    }
    assert_eq!(a.last_seq, b.last_seq);
    assert_eq!(
        serde_json::to_value(&a.resolved).unwrap(),
        serde_json::to_value(&b.resolved).unwrap()
    );
}

#[test]
fn situations_drawn_survives_the_store_roundtrip() {
    let log = v2_events(42, PeriodKind::Week);
    let drawn = drawn_events(&log);
    assert!(!drawn.is_empty());
    for r in drawn {
        let back =
            GameEvent::from_record(r.event.kind(), SCHEMA_VER, r.event.to_payload()).unwrap();
        assert_eq!(back, r.event);
        assert_eq!(back.to_payload(), r.event.to_payload());
    }
}

// ---------------------------------------------------------------------------------------------------------------
// I2: povijest generatora (`day`, `slot`, `occurrence`) iz konteksta, ne zadane vrijednosti

#[test]
fn drawn_items_carry_day_slot_and_occurrence_from_the_context() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let g = play_all(Game::fold(&v2_events(42, kind)).unwrap());
        let infos = g.rounds_info();
        let slots: Vec<u8> = g.hotels.iter().map(|h| h.setup.slot).collect();
        let items = all_items(&g.history);
        assert!(!items.is_empty(), "{kind:?}");
        let mut later_day_nonzero = false;
        for it in &items {
            let code = it["code"].as_str().unwrap();
            for key in ["day", "slot", "occurrence"] {
                assert!(
                    it.get(key).is_some_and(Value::is_u64),
                    "{code}: nema `{key}`"
                );
            }
            let (day, slot, occ) = (
                it["day"].as_u64().unwrap(),
                it["slot"].as_u64().unwrap(),
                it["occurrence"].as_u64().unwrap(),
            );
            let info = infos.iter().find(|i| i.key == round_of(code)).unwrap();
            let (start, days) = (
                u64::from(info.period.start_day),
                u64::from(info.period.days),
            );
            assert!(
                (start..start + days).contains(&day),
                "{code}: dan {day} izvan kruga [{start}, {})",
                start + days
            );
            assert!(
                slot == 0 || slots.contains(&u8::try_from(slot).unwrap()),
                "{code}: utor {slot}"
            );
            assert!(occ >= 1, "{code}: redni broj pojave {occ}");
            if start > 0 && day > 0 {
                later_day_nonzero = true;
            }
        }
        assert!(
            later_day_nonzero,
            "{kind:?}: dan je svugdje 0 (zadana vrijednost)"
        );
    }
}

/// Povijest iz dnevnika je ona koju generator očekuje: razmak ponavljanja i `max_per_season` iz predloška
/// vrijede za primjerke upisane u dnevnik (da su `day`/`occurrence` zadani, razmak bi bio 0 i prekršen).
#[test]
fn history_from_the_log_respects_spacing_and_season_limit() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let g = play_all(Game::fold(&v2_events(42, kind)).unwrap());
        let set = g
            .content
            .situation_templates
            .as_ref()
            .expect("v2 igra ima predloške")
            .to_core();
        let mut by: BTreeMap<(u64, String), Vec<u64>> = BTreeMap::new();
        for it in all_items(&g.history) {
            by.entry((
                it["slot"].as_u64().unwrap(),
                it["template"].as_str().unwrap().to_owned(),
            ))
            .or_default()
            .push(it["day"].as_u64().unwrap());
        }
        assert!(!by.is_empty(), "{kind:?}");
        let mut pairs = 0;
        for ((slot, template), mut days) in by {
            let spec = set.templates.iter().find(|t| t.code == template).unwrap();
            days.sort_unstable();
            assert!(
                days.len() <= usize::from(spec.repeat.max_per_season),
                "{kind:?} utor {slot} {template}: {} pojava, najviše {}",
                days.len(),
                spec.repeat.max_per_season
            );
            for w in days.windows(2) {
                pairs += 1;
                assert!(
                    w[1] >= w[0] + 7 * u64::from(spec.repeat.gap_rounds),
                    "{kind:?} utor {slot} {template}: dani {} i {} bliže od {} tjedana",
                    w[0],
                    w[1],
                    spec.repeat.gap_rounds
                );
            }
        }
        if kind == PeriodKind::Week {
            assert!(
                pairs > 0,
                "tjedna igra: nijedno ponavljanje, test ništa ne dokazuje"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Stare igre: staro pravilo

/// Dnevnik v2 igre kakva je nastala između S-1 i G-1: snimak ima predloške, dnevnik nema `SituationsDrawn`
/// (ni odgovore robota na primjerke; ostaju samo odgovori na sidra S1–S4).
fn without_drawn(log: &[Recorded]) -> Vec<Recorded> {
    log.iter()
        .filter(|r| r.event.kind() != DRAWN)
        .filter(|r| match &r.event {
            GameEvent::AnswerChosen(a) => a.situation.len() == 2 && a.situation.starts_with('S'),
            _ => true,
        })
        .enumerate()
        .map(|(i, r)| Recorded {
            seq: i64::try_from(i).unwrap() + 1,
            actor: r.actor.clone(),
            event: r.event.clone(),
        })
        .collect()
}

#[test]
fn old_v2_game_without_the_event_keeps_the_old_rule() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let full = v2_events(42, kind);
        assert!(
            !drawn_events(&full).is_empty(),
            "{kind:?}: pretpostavka testa — nova igra ima događaj"
        );
        let g = Game::fold(&without_drawn(&full)).unwrap();
        assert!(
            g.content.situation_templates.is_some(),
            "snimak ima predloške"
        );
        for info in g.rounds_info() {
            let want: Vec<String> =
                sim_hotel_v2::situations_for(&info.period, &g.content.situations)
                    .iter()
                    .map(|t| t.code.clone())
                    .collect();
            assert_eq!(
                situations_of(&g, info.key),
                want,
                "{kind:?} {}",
                info.key.code()
            );
        }
        // I dalje se igra do kraja po starom pravilu, bez ijednog novog događaja.
        let done = play_all(g);
        assert_eq!(done.resolved.len(), done.rounds.len());
        assert!(
            drawn_events(&done.history).is_empty(),
            "{kind:?}: stara igra ne dobiva izvlačenje"
        );
    }
}

#[test]
fn v2_game_without_templates_in_the_snapshot_has_no_drawn_event() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let obj = v["v2"].as_object_mut().unwrap();
    let keys: Vec<String> = obj
        .keys()
        .filter(|k| k.contains("template") || k.as_str() == "names")
        .cloned()
        .collect();
    assert!(
        !keys.is_empty(),
        "pretpostavka: predlošci su ključ u v2 snimku"
    );
    for k in keys {
        obj.remove(&k);
    }
    let snap = Snapshot::from_value(&v).unwrap();
    assert!(snap.build().unwrap().situation_templates.is_none());
    let g = play_all(Game::fold(&events_of(&snap, 42, PeriodKind::Week)).unwrap());
    assert!(drawn_events(&g.history).is_empty());
    assert!(
        situations_of(&g, RoundKey::Week(2)).is_empty(),
        "staro pravilo: t02 nema situacija"
    );
}

#[test]
fn v1_game_never_records_situations_drawn_and_keeps_situations_by_month() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let log: Vec<Recorded> = new_game(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        common::GUEST,
        common::hotel_ids(),
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    assert!(drawn_events(&log).is_empty());
    let g = Game::fold(&log).unwrap();
    let june: Vec<String> = g
        .content
        .situations
        .iter()
        .filter(|t| t.month == sim_hotel::Month::Jun)
        .map(|t| t.code.clone())
        .collect();
    assert_eq!(
        situations_of(&g, RoundKey::Month(sim_hotel::Month::Jun)),
        june
    );
    let done = play_all(g);
    assert!(drawn_events(&done.history).is_empty());
}

// ---------------------------------------------------------------------------------------------------------------
// Roboti

#[test]
fn same_seed_gives_the_same_events_and_the_same_robot_answers() {
    let a = v2_events(42, PeriodKind::Week);
    let b = v2_events(42, PeriodKind::Week);
    let payloads = |l: &[Recorded]| -> Vec<(String, Value)> {
        l.iter()
            .map(|r| (r.event.kind().to_owned(), r.event.to_payload()))
            .collect()
    };
    assert_eq!(payloads(&a), payloads(&b));
    assert!(
        !drawn_events(&a).is_empty(),
        "pretpostavka: događaj postoji"
    );
    // Cijela igra do kraja: izbori robota u svim krugovima isti.
    let answers = |g: &Game| -> Vec<Value> {
        g.history
            .iter()
            .filter(|r| r.event.kind() == "AnswerChosen")
            .map(|r| r.event.to_payload())
            .collect()
    };
    let ga = play_all(Game::fold(&a).unwrap());
    let gb = play_all(Game::fold(&b).unwrap());
    assert!(!answers(&ga).is_empty());
    assert_eq!(answers(&ga), answers(&gb));
    // Drugo sjeme daje drugi niz (inače bi test prolazio i kad se sjeme ne koristi).
    let other = v2_events(43, PeriodKind::Week);
    assert_ne!(payloads(&other), payloads(&a));
}

#[test]
fn robots_answer_every_situation_of_their_own_round() {
    let g = play_all(Game::fold(&v2_events(42, PeriodKind::Week)).unwrap());
    let items = all_items(&g.history);
    assert!(!items.is_empty());
    let mut answered: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for r in &g.history {
        if let GameEvent::AnswerChosen(a) = &r.event {
            answered
                .entry((a.hotel_id.0.to_string(), a.month.code()))
                .or_default()
                .push(a.situation.clone());
        }
    }
    for h in g
        .hotels
        .iter()
        .filter(|h| matches!(h.setup.controller, Controller::Robot { .. }))
    {
        let id = h.setup.hotel_id.0.to_string();
        for r in &g.rounds {
            let got = answered
                .get(&(id.clone(), r.code()))
                .cloned()
                .unwrap_or_default();
            let mut uniq = got.clone();
            uniq.sort();
            uniq.dedup();
            assert_eq!(
                uniq.len(),
                got.len(),
                "robot {id} {}: dvostruki odgovor",
                r.code()
            );
            assert!(
                (3..=5).contains(&got.len()),
                "robot {id} {}: {} odgovora",
                r.code(),
                got.len()
            );
            for it in items.iter().filter(|i| {
                i["slot"].as_u64() == Some(u64::from(h.setup.slot))
                    && round_of(i["code"].as_str().unwrap()) == *r
            }) {
                assert!(
                    got.iter().any(|c| c == it["code"].as_str().unwrap()),
                    "robot {id} {}: nije odgovorio na {}",
                    r.code(),
                    it["code"]
                );
            }
        }
    }
}

#[test]
fn player_can_answer_drawn_situations_through_the_normal_commands() {
    let g = Game::fold(&v2_events(42, PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let codes = situations_of(&g, r);
    assert!(codes.len() >= 3, "{codes:?}");
    assert_eq!(usize::from(g.unanswered_count(h, r).unwrap()), codes.len());
    for c in &codes {
        assert_eq!(g.situation_round(c).unwrap().0, r);
        g.choose_answer(h, c, 0).unwrap();
    }
}

// ---------------------------------------------------------------------------------------------------------------
// m6: „grupa odbijena" iz P-POHVALA (`GroupBooking accept: false`, tehnički prazan zapis) se nikad ne prikazuje

/// Prvo sjeme mjesečne igre u kojoj hotel igrača (utor 1) u prvom krugu dobije primjerak P-POHVALA.
fn game_with_praise_for_the_player() -> (Game, String) {
    for seed in 1..=300_u64 {
        let log = v2_events(seed, PeriodKind::Month);
        let found = all_items(&log)
            .into_iter()
            .find(|i| i["template"] == "P-POHVALA" && i["slot"].as_u64() == Some(1));
        if let Some(it) = found {
            let code = it["code"].as_str().unwrap().to_owned();
            return (Game::fold(&log).unwrap(), code);
        }
    }
    panic!("P-POHVALA nije izvučena za hotel igrača ni u jednom od 300 sjemena");
}

#[test]
fn refused_group_of_the_praise_is_not_shown_to_the_student_or_the_mentor() {
    let (mut g, code) = game_with_praise_for_the_player();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    assert!(situations_of(&g, r).contains(&code));
    // Treći ponuđeni odgovor („odgovoriti gostu, njoj ništa") nosi tehničku mehaniku odbijene grupe.
    let ev = g.choose_answer(h, &code, 2).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let input = g.debrief_input(h, r).unwrap();
    let s = input.situations.iter().find(|s| s.code == code).unwrap();
    let facts = debrief_facts(&input);
    let fact = facts
        .situations
        .iter()
        .find(|f| f.title == s.title)
        .expect("situacija je u činjenicama za mentora");
    let shown = serde_json::to_string(fact).unwrap().to_lowercase();
    assert!(
        !shown.contains("odbij") && !shown.contains("grup"),
        "činjenica za mentora spominje grupu: {shown}"
    );
    // Ni obračun igrača (ono što student vidi u izvještaju) ne govori o odbijenoj grupi.
    let mine = g.resolved[&r]
        .hotels
        .iter()
        .find(|x| x.hotel_id == h)
        .unwrap();
    let report = serde_json::to_string(&mine.situations)
        .unwrap()
        .to_lowercase();
    assert!(
        !report.contains("odbij") && !report.contains("grup"),
        "{report}"
    );
}
