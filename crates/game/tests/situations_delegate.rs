//! A-1 (Leon P4, 13.10.): „prve 3 situacije kruga su obvezne, ostale smije prepustiti voditelju odjela (kvaliteta 2,
//! bez AI-ja)". Naredba, događaj, obračun, mentor i kraj sezone za prepuštenu situaciju.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `Game::delegate_situation(&self, hotel: HotelId, code: &str) -> Result<GameEvent, GameError>` (kao `choose_answer`).
//! - Događaj se prepoznaje po `GameEvent::kind() == "SituationDelegated"` i JSON-u `to_payload()` s poljima
//!   `hotel_id`, `month`, `situation` (ne po imenu varijante); `GameEvent::from_record` ga čita natrag.
//! - Nova greška `GameError::SituationMandatory` (jedinična varijanta).
//! - `SituationResult.delegated: bool` (`#[serde(default, skip_serializing_if = "is_false")]`).
//! - `DebriefSituation.delegated: bool`; za prepuštenu `unanswered == false`.
//! - 🟨 „Prve 3" = prve tri u poretku `Game::situations_of_hotel` (sidra S1–S4 prva, pa primjerci po važnosti silazno,
//!   stabilno po redu događaja) — isti poredak koji vidi pogled.
//!
//! Krugovi (sjeme 42, hotel igrača, nađeno sondom 14.10.):
//! - mjesečna igra, `jun` (otvoren odmah): od B2a 4 situacije (mjesečno 4/5/6 umjesto 13–22): S1, S3, P-OLUJA.jun
//!   (kolektivna) i P-PRAONICA.jun. Testovi koji trebaju 5. ili 6. mjesto gledaju kolovoz (6: S4, P-TOPLINSKI-VAL,
//!   P-NESTALA-STVAR, P-POVISICA, P-DOBAVLJAC, P-POHVALA; 4. i 6. imaju lika), v. `monthly_round_where`.
//! - tjedna igra, `t05` (nakon obračuna t01–t04): 4 situacije — S2 (sidro), P-OSPORENA-NAPLATA.t05,
//!   P-LJUBIMAC.t05 | P-DOBAVLJAC.t05 (sva tri primjerka važnosti 2: poredak među njima je red događaja).
//! - tjedna igra, `t10`: 5 situacija (prva je kolektivna P-TOPLINSKI-VAL.t10); `t01`: 3 (S1, S3, P-OLUJA.t01).

mod common;

use std::collections::BTreeSet;

use game::feed::FeedItem;
use game::*;
use serde_json::Value;
use sim_hotel::SituationOutcome;
use sim_hotel_v2::resolve_period;
use sim_hotel_v2::situations::{RoundSituation, reward_only};

const DELEGATED: &str = "SituationDelegated";

fn me() -> HotelId {
    common::hotel_ids()[0]
}

fn codes(g: &Game, r: RoundKey) -> Vec<String> {
    g.situations_of_hotel(me(), r)
        .unwrap()
        .iter()
        .map(|t| t.code.clone())
        .collect()
}

/// Mjesečna igra, sjeme 42, lipanj otvoren (od B2a 4 situacije, prije 13).
fn monthly_june() -> (Game, RoundKey) {
    let g = common::v2_monthly_game_resolved_through("none");
    let r = g.current_round().unwrap();
    assert_eq!(r.code(), "jun");
    assert!(
        codes(&g, r).len() >= 4,
        "jun sjemena 42: {:?}",
        codes(&g, r)
    );
    (g, r)
}

/// Mjesečna igra, sjeme 42 (bez odgovora igrača): prvi krug jun→sep u kojem `ok` vrijedi. Od B2a mjesečni krug ima
/// 4/5/6 situacija, pa testovi koji trebaju 5. ili 6. mjesto ne mogu u lipnju (4); tvrdnje testova ostaju iste, mijenja
/// se samo krug koji se gleda.
fn monthly_round_where(ok: impl Fn(&Game, RoundKey) -> bool) -> (Game, RoundKey) {
    for prev in ["none", "jun", "jul", "aug"] {
        let g = common::v2_monthly_game_resolved_through(prev);
        if let Some(r) = g.current_round()
            && ok(&g, r)
        {
            return (g, r);
        }
    }
    panic!("nijedan mjesečni krug jun-sep sjemena 42 nema tražene situacije");
}

/// Mjesečni krug sjemena 42 sa barem šest situacija (6. mjesto postoji).
fn monthly_with_six() -> (Game, RoundKey) {
    monthly_round_where(|g, r| codes(g, r).len() >= 6)
}

/// Tjedna igra, sjeme 42: obračunati krugovi do uključivo `prev` (bez ijednog odgovora igrača; `"none"` = nijedan),
/// pa je otvoren krug `code`.
fn weekly_open(prev: &str, code: &str) -> (Game, RoundKey) {
    let g = common::v2_weekly_game_resolved_through(prev);
    let r = RoundKey::parse(code).unwrap();
    assert_eq!(g.current_round(), Some(r));
    (g, r)
}

fn delegate(g: &mut Game, code: &str) {
    let ev = g
        .delegate_situation(me(), code)
        .unwrap_or_else(|e| panic!("{code}: {e:?}"));
    common::apply_ev(g, Actor::Guest(common::GUEST), ev);
}

fn close_and_resolve(g: &mut Game, r: RoundKey) {
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun čeka")
    };
    for (a, e) in evs {
        common::apply_ev(g, a, e);
    }
}

fn play_rest(g: &mut Game) {
    while let Some(r) = g.current_round() {
        close_and_resolve(g, r);
    }
}

fn my_result(g: &Game, r: RoundKey, code: &str) -> SituationResult {
    g.resolved[&r]
        .hotels
        .iter()
        .find(|h| h.hotel_id == me())
        .unwrap()
        .situations
        .iter()
        .find(|s| s.code == code)
        .unwrap_or_else(|| panic!("{code} nije u obračunu"))
        .clone()
}

// ---------------------------------------------------------------------------------------------------------------
// 1. Naredba i događaj

#[test]
fn delegating_the_fourth_situation_records_the_event() {
    let (mut g, r) = weekly_open("t04", "t05");
    let list = codes(&g, r);
    assert_eq!(list.len(), 4, "t05 sjemena 42: {list:?}");
    let ev = g.delegate_situation(me(), &list[3]).unwrap();
    assert_eq!(ev.kind(), DELEGATED);
    let p = ev.to_payload();
    assert_eq!(p["situation"], list[3].as_str());
    assert_eq!(p["month"], serde_json::to_value(r).unwrap());
    assert_eq!(p["hotel_id"], serde_json::to_value(me()).unwrap());
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    // prepuštena se više ne broji kao neodgovorena
    assert_eq!(g.unanswered_count(me(), r).unwrap(), 3);
}

#[test]
fn in_a_round_of_five_both_the_fourth_and_the_fifth_can_be_delegated() {
    let (mut g, r) = weekly_open("t09", "t10");
    let list = codes(&g, r);
    assert_eq!(list.len(), 5, "t10 sjemena 42: {list:?}");
    delegate(&mut g, &list[3]);
    delegate(&mut g, &list[4]);
    assert_eq!(g.unanswered_count(me(), r).unwrap(), 3);
}

#[test]
fn delegated_event_survives_the_store_roundtrip() {
    let (g, r) = monthly_june();
    let code = &codes(&g, r)[3];
    let ev = g.delegate_situation(me(), code).unwrap();
    let back = GameEvent::from_record(DELEGATED, SCHEMA_VER, ev.to_payload()).unwrap();
    assert_eq!(back, ev);
}

// ---------------------------------------------------------------------------------------------------------------
// 2. Pravila naredbe

#[test]
fn the_first_three_are_mandatory() {
    let (g, r) = monthly_june();
    for code in codes(&g, r).iter().take(3) {
        assert_eq!(
            g.delegate_situation(me(), code),
            Err(GameError::SituationMandatory),
            "{code}"
        );
    }
    // Krug s točno 3 situacije (t01: dva sidra i kolektivna): ništa se ne smije prepustiti.
    let (g, r) = weekly_open("none", "t01");
    let list = codes(&g, r);
    assert_eq!(list.len(), 3, "t01 sjemena 42: {list:?}");
    for code in &list {
        assert_eq!(
            g.delegate_situation(me(), code),
            Err(GameError::SituationMandatory),
            "{code}"
        );
    }
}

#[test]
fn delegating_twice_or_after_an_answer_is_already_answered() {
    // treba krug u kojem 4. mjesto ima lika sa slobodnim odgovorom (razgovor se ne smije tiho preskočiti) i 6. mjesto
    // (ili dalje) isto tako
    let free = |g: &Game, c: &str| g.ai_entry_for(me(), c).is_some_and(|a| a.free_text);
    let (mut g, r) = monthly_round_where(|g, r| {
        let list = codes(g, r);
        list.get(3).is_some_and(|c| free(g, c)) && list.iter().skip(5).any(|c| free(g, c))
    });
    let list = codes(&g, r);
    delegate(&mut g, &list[3]);
    assert_eq!(
        g.delegate_situation(me(), &list[3]),
        Err(GameError::AlreadyAnswered)
    );
    // prepuštena se ne može ni odgovoriti ni razgovarati (4. mjesto odabranog kruga ima lika, vidi predikat gore)
    assert_eq!(
        g.choose_answer(me(), &list[3], 0),
        Err(GameError::AlreadyAnswered)
    );
    assert!(free(&g, &list[3]), "{}: 4. mjesto ima lika", list[3]);
    assert_eq!(
        g.submit_response(me(), &list[3], "Ja ću to riješiti."),
        Err(GameError::AlreadyAnswered)
    );
    assert_eq!(
        g.send_message(me(), &list[3], "Dobar dan."),
        Err(GameError::AlreadyAnswered)
    );
    // ponuđeni odgovor pa prepuštanje
    let ev = g.choose_answer(me(), &list[4], 0).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    assert_eq!(
        g.delegate_situation(me(), &list[4]),
        Err(GameError::AlreadyAnswered)
    );
    // slobodni odgovor pa prepuštanje
    let free = list
        .iter()
        .skip(5)
        .find(|c| g.ai_entry_for(me(), c).is_some_and(|a| a.free_text))
        .expect("krug ima primjerak s likom iza petog mjesta")
        .clone();
    let ev = g
        .submit_response(me(), &free, "Ispričat ću se i ponuditi rješenje.")
        .unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    assert_eq!(
        g.delegate_situation(me(), &free),
        Err(GameError::AlreadyAnswered)
    );
}

#[test]
fn closed_or_resolved_round_is_round_not_open() {
    let (mut g, r) = monthly_june();
    let code = codes(&g, r)[3].clone();
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    assert_eq!(
        g.delegate_situation(me(), &code),
        Err(GameError::RoundNotOpen)
    );
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun čeka")
    };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    assert_eq!(
        g.delegate_situation(me(), &code),
        Err(GameError::RoundNotOpen)
    );
}

#[test]
fn unknown_situation_is_unknown_situation() {
    let (g, _) = monthly_june();
    assert_eq!(
        g.delegate_situation(me(), "P-NEPOSTOJI.jun"),
        Err(GameError::UnknownSituation)
    );
}

/// v1 igra nema voditelja (`situation_leader` = `None`) i ima najviše 2 situacije po krugu: prepuštanje se odbija.
#[test]
fn v1_game_never_delegates() {
    let (g, h) = common::open_june_game();
    for t in g.situations_in(sim_hotel::Month::Jun) {
        assert!(g.delegate_situation(h, &t.code).is_err(), "{}", t.code);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// 1. Obračun: kvaliteta 2, `delegated: true`, mehanike `unanswered`

/// Obračun hotela igrača ponovno izračunat iz javnih dijelova (obrazac `situations_replay.rs`), s prepuštenima kao
/// mehanike `unanswered` uz kvalitetu 2.
fn recomputed(g: &Game, r: RoundKey, delegated: &BTreeSet<String>) -> game::dto::MonthReportDto {
    let CoreState::V2(state) = g.state_at(me(), r).unwrap() else {
        panic!("v2 stanje");
    };
    let CorePlan::V2(plan) = g.effective_plan(me(), r).unwrap() else {
        panic!("v2 plan");
    };
    let h = g.hotel(me()).unwrap();
    let outcomes: Vec<RoundSituation> = g
        .situations_of_hotel(me(), r)
        .unwrap()
        .into_iter()
        .map(|t| {
            let o = match h.answers.get(&(r, t.code.clone())) {
                Some(&i) => SituationOutcome::from_answer(&t.code, &t.answers[usize::from(i)]),
                None if delegated.contains(&t.code) => SituationOutcome {
                    code: t.code.clone(),
                    mechanics: t.unanswered.clone(),
                    quality: 2,
                },
                None => SituationOutcome::unanswered(t),
            };
            let instance = h
                .drawn
                .get(&r)
                .and_then(|l| l.iter().find(|i| i.situation.code == t.code));
            let no_penalty = instance.is_some_and(|i| i.no_penalty);
            RoundSituation {
                outcome: if no_penalty { reward_only(o) } else { o },
                space: instance.and_then(|i| i.drawn.department.clone()),
            }
        })
        .collect();
    let cat = g.content.v2.as_ref().unwrap();
    let period = g.round_info(r).unwrap().period;
    let report = resolve_period(&state, &period, &plan, &outcomes, cat).unwrap();
    game::dto::MonthReportDto::from(&report.summary)
}

#[test]
fn delegated_situation_resolves_with_quality_two_and_unanswered_mechanics() {
    let (mut g, r) = monthly_with_six();
    let list = codes(&g, r);
    // četvrta i peta situacija kruga (kolovoz) prepuštene; šesta ostaje bez odgovora
    delegate(&mut g, &list[3]);
    delegate(&mut g, &list[4]);
    close_and_resolve(&mut g, r);
    for code in &list[3..5] {
        let s = my_result(&g, r, code);
        assert_eq!(
            (s.answer, s.quality, s.free_text, s.delegated),
            (None, 2, false, true),
            "{code}"
        );
    }
    let ignored = my_result(&g, r, &list[5]);
    assert_eq!(
        (ignored.answer, ignored.quality, ignored.delegated),
        (None, 1, false),
        "{}: neodgovorena ostaje kvaliteta 1",
        list[5]
    );
    let delegated: BTreeSet<String> = list[3..5].iter().cloned().collect();
    let recorded = &g.resolved[&r]
        .hotels
        .iter()
        .find(|h| h.hotel_id == me())
        .unwrap()
        .report;
    assert_eq!(
        &recomputed(&g, r, &delegated),
        recorded,
        "obračun prepuštene nije `unanswered` s kvalitetom 2"
    );
}

#[test]
fn delegated_result_serializes_the_flag_and_old_records_read_as_not_delegated() {
    let (mut g, r) = monthly_june();
    let code = codes(&g, r)[3].clone();
    delegate(&mut g, &code);
    close_and_resolve(&mut g, r);
    let s = my_result(&g, r, &code);
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["delegated"], true, "{v}");
    let old = r#"{"code":"S1","answer":null,"quality":1}"#;
    let back: SituationResult = serde_json::from_str(old).unwrap();
    assert!(!back.delegated);
    assert_eq!(serde_json::to_string(&back).unwrap(), old, "bajt-isto");
}

// ---------------------------------------------------------------------------------------------------------------
// 3. Fold je čist

#[test]
fn folding_a_game_with_delegations_twice_gives_the_same_game() {
    let (mut g, r) = weekly_open("t04", "t05");
    let code = codes(&g, r)[3].clone();
    delegate(&mut g, &code);
    play_rest(&mut g);
    assert!(g.history.iter().any(|x| x.event.kind() == DELEGATED));
    let a = Game::fold(&g.history).unwrap();
    let b = Game::fold(&g.history).unwrap();
    assert_eq!(a.last_seq, b.last_seq);
    let ja = serde_json::to_value(&a.resolved).unwrap();
    assert_eq!(ja, serde_json::to_value(&b.resolved).unwrap());
    assert_eq!(ja, serde_json::to_value(&g.resolved).unwrap());
    assert!(my_result(&a, r, &code).delegated);
}

// ---------------------------------------------------------------------------------------------------------------
// 4. Mentor: prepuštena nije „bez odgovora"

#[test]
fn debrief_input_tells_delegated_from_unanswered() {
    let (mut g, r) = monthly_with_six();
    let list = codes(&g, r);
    delegate(&mut g, &list[3]);
    close_and_resolve(&mut g, r);
    let input = g.debrief_input(me(), r).unwrap();
    let find = |c: &str| input.situations.iter().find(|s| s.code == c).unwrap();
    let d = find(&list[3]);
    assert_eq!((d.delegated, d.unanswered, d.quality), (true, false, 2));
    let u = find(&list[5]);
    assert_eq!((u.delegated, u.unanswered, u.quality), (false, true, 1));

    // Činjenice za AI mentora: prepuštena nema „bez_odgovora".
    let facts = serde_json::to_value(debrief_facts(&input)).unwrap();
    let title = &d.title;
    let fact = facts["situacije"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["naslov"] == title.as_str())
        .unwrap_or_else(|| panic!("činjenica za „{title}“"));
    assert_ne!(fact["bez_odgovora"], Value::Bool(true), "{fact}");

    // Mentor bez AI-ja ne kaže da na prepuštenu „nije bilo odgovora".
    let text = fallback_debrief(&input);
    assert!(
        !text.contains(&format!("Na situaciju „{title}“ nije bilo odgovora")),
        "{text}"
    );
}

// ---------------------------------------------------------------------------------------------------------------
// 🟨 Isto pravilo na drugim mjestima koja čitaju `answer == None && !free_text` kao „bez odgovora"
// (`season.rs::decisions`, `feed.rs` `SituationIgnored`) — izvan popisa datoteka graditelja, vidi izvještaj.

#[test]
fn season_decisions_do_not_count_delegated_as_unanswered() {
    let (mut g, r) = monthly_june();
    let code = codes(&g, r)[3].clone();
    delegate(&mut g, &code);
    play_rest(&mut g);
    let s = g.season(me()).unwrap();
    let d = s
        .hotels
        .iter()
        .find(|h| h.mine)
        .unwrap()
        .decisions
        .clone()
        .unwrap();
    // igrač ni na što nije odgovorio, jednu je prepustio
    assert_eq!(d.unanswered + 1, d.situations, "{d:?}");
}

#[test]
fn feed_does_not_show_delegated_as_ignored() {
    let (mut g, r) = monthly_with_six();
    let list = codes(&g, r);
    delegate(&mut g, &list[3]);
    close_and_resolve(&mut g, r);
    let ignored: Vec<String> = g
        .feed(me())
        .unwrap()
        .into_iter()
        .filter_map(|i| match i {
            FeedItem::SituationIgnored { situation, .. } => Some(situation),
            _ => None,
        })
        .collect();
    assert!(ignored.contains(&list[5]), "neodgovorena je u feedu");
    assert!(!ignored.contains(&list[3]), "prepuštena nije „ignorirana“");
}
