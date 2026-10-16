//! PRAVILO-a (ADR-047, kvar B13): vlastiti odgovor vrijedi više od klika. Postavka `AnswerRule` (`Classic` = danas,
//! `OwnWords` = klik u situaciji s likom najviše 4), prepisan ponuđeni odgovor vrijedi kao klik, neutralna presuda
//! je 4, a zalijepljeno ≥ 2 ponuđena odgovora je neutralna 3 (kod odlučuje, uz oba pravila).
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::event::AnswerRule;
use game::own_words::{
    CLICK_MAX_QUALITY, COPY_MATCH_BP, NEUTRAL_OWN_WORDS_QUALITY, copied_answer, pasted_answers,
    words,
};
use game::*;
use serde_json::Value;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

const SEED: u64 = 42;

fn me() -> HotelId {
    common::hotel_ids()[0]
}

fn guest() -> Actor {
    Actor::Guest(common::GUEST)
}

fn settings(rule: Option<AnswerRule>) -> GameSettings {
    GameSettings {
        kind: PeriodKind::Week,
        situations: None,
        answer_rule: rule,
    }
}

fn v2_snap() -> Snapshot {
    Snapshot::read_v2(&repo_dir()).unwrap()
}

fn new_game_rule(seed: u64, rule: Option<AnswerRule>) -> Game {
    common::v2_game_with(&v2_snap(), seed, settings(rule))
}

/// Situacija tekućeg kruga hotela igrača: šifra, oznake i kvalitete ponuđenih odgovora, ima li lika.
#[derive(Clone)]
struct Sit {
    code: String,
    labels: Vec<String>,
    qualities: Vec<u8>,
    free: bool,
}

impl Sit {
    fn best_q(&self) -> u8 {
        *self.qualities.iter().max().unwrap()
    }
    fn best(&self) -> u8 {
        let q = self.best_q();
        u8::try_from(self.qualities.iter().position(|&x| x == q).unwrap()).unwrap()
    }
}

fn sits(g: &Game, r: RoundKey) -> Vec<Sit> {
    g.situations_of_hotel(me(), r)
        .unwrap()
        .iter()
        .map(|t| Sit {
            code: t.code.clone(),
            labels: t.answers.iter().map(|a| a.label.clone()).collect(),
            qualities: t.answers.iter().map(|a| a.quality).collect(),
            free: g.ai_entry_for(me(), &t.code).is_some_and(|a| a.free_text),
        })
        .collect()
}

/// Zatvori tekući krug, obračunaj ga i vrati obračun hotela igrača.
fn close_and_resolve(g: &mut Game) -> HotelResolution {
    let r = g.current_round().unwrap();
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(g, guest(), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        common::apply_ev(g, a, e);
    }
    g.resolved[&r]
        .hotels
        .iter()
        .find(|h| h.hotel_id == me())
        .cloned()
        .unwrap()
}

/// Prvi krug (student šuti u prethodnima) sa situacijom koju `pick` prihvaća; vraća igru u tom krugu.
fn first_with(rule: Option<AnswerRule>, pick: &dyn Fn(&Sit) -> bool) -> (Game, Sit) {
    let mut g = new_game_rule(SEED, rule);
    while let Some(r) = g.current_round() {
        if let Some(s) = sits(&g, r).into_iter().find(|s| pick(s)) {
            return (g, s);
        }
        close_and_resolve(&mut g);
    }
    panic!("nema tražene situacije u sezoni");
}

/// Situacija s likom (smije se pisati) kojoj je najbolji odgovor q 5 (u sadržaju v2 najbolji je uvijek prvi; izbor indeksa čuva test `copied_answer_prefers_highest_share_then_lowest_index`).
fn first_free(rule: Option<AnswerRule>) -> (Game, Sit) {
    first_with(rule, &|s| s.free && s.best_q() == 5 && s.labels.len() >= 2)
}

/// Situacija bez lika kojoj je najbolji odgovor q 5.
fn first_closed(rule: Option<AnswerRule>) -> (Game, Sit) {
    first_with(rule, &|s| !s.free && s.best_q() == 5)
}

fn click(g: &mut Game, s: &Sit, answer: u8) {
    let ev = g.choose_answer(me(), &s.code, answer).unwrap();
    common::apply_ev(g, guest(), ev);
}

fn result_of(res: &HotelResolution, s: &Sit) -> SituationResult {
    res.situations
        .iter()
        .find(|x| x.code == s.code)
        .cloned()
        .unwrap()
}

fn verdict_of(ev: GameEvent) -> VerdictRecorded {
    let GameEvent::VerdictRecorded(v) = ev else {
        panic!("očekivan VerdictRecorded");
    };
    v
}

/// Slobodni odgovor `text` i presuda suda (`met` kriterija, najbliži odgovor `nearest`).
fn judge(g: &mut Game, s: &Sit, text: &str, met: usize, nearest: u8) -> VerdictRecorded {
    let ev = g.submit_response(me(), &s.code, text).unwrap();
    common::apply_ev(g, guest(), ev);
    let v = g
        .validate_verdict_for(me(), &s.code, common::draft_with(met, nearest))
        .unwrap();
    let ev = g.record_verdict(me(), &s.code, v, common::meta()).unwrap();
    common::apply_ev(g, Actor::System, ev.clone());
    verdict_of(ev)
}

/// Slobodni odgovor `text` i neutralna presuda (AI nije stigla).
fn neutral(g: &mut Game, s: &Sit, text: &str) -> VerdictRecorded {
    let ev = g.submit_response(me(), &s.code, text).unwrap();
    common::apply_ev(g, guest(), ev);
    let ev = g.neutral_verdict(me(), &s.code, NEUTRAL_FAILED).unwrap();
    common::apply_ev(g, Actor::System, ev.clone());
    verdict_of(ev)
}

/// Oznaka velikim slovima, bez interpunkcije i kvačica.
fn shout(label: &str) -> String {
    label
        .to_uppercase()
        .chars()
        .map(|c| match c {
            'Č' | 'Ć' => 'C',
            'Š' => 'S',
            'Ž' => 'Z',
            'Đ' => 'D',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect()
}

fn created_events(
    snap: &Snapshot,
    kind: PeriodKind,
    rule: Option<AnswerRule>,
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    new_game_with(
        GameId(Uuid::from_u128(1)),
        SEED,
        snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind,
            situations: None,
            answer_rule: rule,
        },
    )
}

fn written_rule(events: &[(Actor, GameEvent)]) -> Option<AnswerRule> {
    let Some((_, GameEvent::GameCreated(c))) = events.first() else {
        panic!("prvi događaj nije GameCreated");
    };
    c.answer_rule
}

#[test] // 1
fn new_v2_game_writes_own_words_rule() {
    let snap = v2_snap();
    let default = created_events(&snap, PeriodKind::Week, None).unwrap();
    assert_eq!(written_rule(&default), Some(AnswerRule::OwnWords));
    let classic = created_events(&snap, PeriodKind::Week, Some(AnswerRule::Classic)).unwrap();
    assert_eq!(written_rule(&classic), Some(AnswerRule::Classic));
    assert_eq!(
        new_game_rule(SEED, None).answer_rule(),
        AnswerRule::OwnWords
    );
    assert_eq!(
        new_game_rule(SEED, Some(AnswerRule::Classic)).answer_rule(),
        AnswerRule::Classic
    );
    // v1: nema pravila; izričito pravilo uz v1 nije dopušteno.
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    let v1_events = created_events(&v1, PeriodKind::Month, None).unwrap();
    assert_eq!(written_rule(&v1_events), None);
    for rule in [AnswerRule::Classic, AnswerRule::OwnWords] {
        assert!(matches!(
            created_events(&v1, PeriodKind::Month, Some(rule)),
            Err(GameError::InvalidSettings)
        ));
    }
}

#[test] // 2
fn log_without_rule_reads_as_classic() {
    let snap = v2_snap();
    let mut events = created_events(&snap, PeriodKind::Week, None).unwrap();
    let GameEvent::GameCreated(c) = &mut events[0].1 else {
        panic!("prvi događaj nije GameCreated");
    };
    c.answer_rule = None;
    // Bajt-isto kao danas: bez pravila polje se uopće ne zapisuje.
    assert!(events[0].1.to_payload().get("answer_rule").is_none());
    let log: Vec<Recorded> = events
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect();
    assert_eq!(Game::fold(&log).unwrap().answer_rule(), AnswerRule::Classic);
}

/// Klik na najbolji odgovor prve situacije s likom (ili bez njega) uz pravilo `rule`.
fn click_best(rule: Option<AnswerRule>, free: bool) -> (Game, Sit, HotelResolution) {
    let (mut g, s) = if free {
        first_free(rule)
    } else {
        first_closed(rule)
    };
    click(&mut g, &s, s.best());
    let res = close_and_resolve(&mut g);
    (g, s, res)
}

#[test] // 3
fn click_with_character_is_capped_at_four() {
    let (g, s, res) = click_best(None, true);
    assert_eq!(g.answer_rule(), AnswerRule::OwnWords);
    let r = result_of(&res, &s);
    assert_eq!(r.answer, Some(s.best()));
    assert_eq!(r.quality, CLICK_MAX_QUALITY);
    assert!(r.capped, "klik q 5 sa snižen na 4 mora biti capped");
    let (gc, sc, resc) = click_best(Some(AnswerRule::Classic), true);
    assert_eq!(sc.code, s.code);
    assert_eq!(result_of(&resc, &sc).quality, 5);
    assert!(
        res.progress_hundredths < resc.progress_hundredths,
        "ograničen klik daje manji napredak: {} !< {}",
        res.progress_hundredths,
        resc.progress_hundredths
    );
    // Pitanje igre prije klika: ograničenje vrijedi samo u OwnWords situaciji s likom.
    let (g2, s2) = first_free(None);
    assert!(g2.click_capped(me(), &s2.code));
    assert!(!gc.click_capped(me(), &sc.code));
}

#[test] // 4
fn click_without_character_keeps_five() {
    let (g, s, res) = click_best(None, false);
    assert!(!g.click_capped(me(), &s.code));
    let r = result_of(&res, &s);
    assert_eq!(r.quality, 5);
    assert!(!r.capped);
}

#[test] // 4a: ocjena koja nije snižena nije „ograničena" (rizik 7)
fn click_with_low_quality_is_not_capped() {
    let (mut g, s) = first_with(None, &|s| s.free && s.qualities.iter().any(|&q| q <= 4));
    let i = u8::try_from(s.qualities.iter().position(|&q| q <= 4).unwrap()).unwrap();
    click(&mut g, &s, i);
    let res = close_and_resolve(&mut g);
    let r = result_of(&res, &s);
    assert_eq!(r.quality, s.qualities[usize::from(i)]);
    assert!(!r.capped);
}

#[test] // 5
fn own_words_best_keeps_five() {
    let (mut g, s) = first_free(None);
    let v = judge(&mut g, &s, "Odgovaram gostu svojim riječima.", 5, s.best());
    assert_eq!(v.quality, 5);
    assert_eq!(v.copied, None);
    assert_eq!(v.nearest_answer, Some(s.best()));
    let res = close_and_resolve(&mut g);
    let r = result_of(&res, &s);
    assert_eq!((r.quality, r.capped, r.free_text), (5, false, true));
}

#[test] // 6
fn copied_best_answer_counts_as_click() {
    let (mut g, s) = first_free(None);
    let text = shout(&s.labels[usize::from(s.best())]);
    // Sud kaže 1, ali prepisan je najbolji odgovor: `nearest_answer` mora doći iz prepisivanja, ne iz suda.
    let v = judge(&mut g, &s, &text, 5, 1);
    assert_eq!(v.copied, Some(s.best()));
    assert_eq!(v.nearest_answer, Some(s.best()));
    assert_eq!(v.quality, CLICK_MAX_QUALITY);
    assert!(!v.pasted);
    let res = close_and_resolve(&mut g);
    let r = result_of(&res, &s);
    assert_eq!((r.quality, r.capped, r.free_text), (4, true, true));
}

#[test] // 7
fn neutral_verdict_is_four_under_own_words() {
    let (mut g, s) = first_free(None);
    let v = neutral(&mut g, &s, "Odgovaram gostu svojim riječima.");
    assert_eq!(v.quality, NEUTRAL_OWN_WORDS_QUALITY);
    assert_eq!(v.nearest_answer, None);
    assert!(v.review);
    let res = close_and_resolve(&mut g);
    let r = result_of(&res, &s);
    assert_eq!((r.quality, r.capped), (4, false));
    // Classic: kao danas, 3.
    let (mut gc, sc) = first_free(Some(AnswerRule::Classic));
    let vc = neutral(&mut gc, &sc, "Odgovaram gostu svojim riječima.");
    assert_eq!((vc.quality, vc.nearest_answer), (3, None));
}

#[test] // 8
fn classic_rule_keeps_old_scoring() {
    let classic = Some(AnswerRule::Classic);
    let (_, s, res) = click_best(classic, true);
    let r = result_of(&res, &s);
    assert_eq!((r.quality, r.capped), (5, false));
    let (mut g, s) = first_free(classic);
    let v = judge(&mut g, &s, &shout(&s.labels[usize::from(s.best())]), 5, 1);
    assert_eq!((v.quality, v.copied, v.nearest_answer), (5, None, Some(1)));
    let (mut g, s) = first_free(classic);
    let v = neutral(&mut g, &s, "Odgovaram gostu svojim riječima.");
    assert_eq!(v.quality, 3);
}

#[test] // 9
fn robot_clicks_are_not_capped() {
    for seed in 1..=20_u64 {
        let mut g = new_game_rule(seed, None);
        while let Some(r) = g.current_round() {
            close_and_resolve(&mut g);
            for h in &g.resolved[&r].hotels {
                let robot = matches!(
                    g.hotel(h.hotel_id).unwrap().setup.controller,
                    Controller::Robot { .. }
                );
                if !robot {
                    continue;
                }
                for s in &h.situations {
                    assert!(!s.capped, "robot ne smije biti ograničen: {}", s.code);
                    let free = g
                        .ai_entry_for(h.hotel_id, &s.code)
                        .is_some_and(|a| a.free_text);
                    if free && s.answer.is_some() && s.quality == 5 {
                        return;
                    }
                }
            }
        }
    }
    panic!("nijedan robot nije kliknuo q 5 u situaciji s likom (sjemena 1..20)");
}

#[test] // 10
fn copied_answer_normalizes_text() {
    assert_eq!(
        words("Dobrodošli, ČĆŠĐŽ ma-ma 12!"),
        vec!["dobrodosli", "ccsdz", "ma", "ma", "12"]
    );
    let labels = [
        "Ispričat ću se gostu i ponuditi zamjenu sobe",
        "Ignorirati pritužbu",
    ];
    assert_eq!(
        copied_answer("ISPRICAT CU SE GOSTU, I PONUDITI ZAMJENU SOBE!!", &labels),
        Some(0)
    );
    assert_eq!(copied_answer("", &labels), None);
    assert_eq!(
        copied_answer("Ignorirati pritužbu", &["", "ignorirati pritužbu"]),
        Some(1)
    );
}

#[test] // 10
fn copied_answer_needs_seventy_percent_of_both_sides() {
    assert_eq!(COPY_MATCH_BP, 7_000);
    let label = "a1 b2 c3 d4 e5 f6 g7 h8 i9 j0";
    // Oznaka + isto toliko svojih riječi: 50 % teksta, nije prepisano.
    assert_eq!(
        copied_answer(&format!("{label} k1 k2 k3 k4 k5 k6 k7 k8 k9 k0"), &[label]),
        None
    );
    // Ispremetane riječi (skup, 🟨): prepisano.
    assert_eq!(
        copied_answer("j0 i9 h8 g7 f6 e5 d4 c3 b2 a1", &[label]),
        Some(0)
    );
    // Točno 7 od 10: na granici prolazi, 6 od 10 ne.
    assert_eq!(copied_answer("a1 b2 c3 d4 e5 f6 g7", &[label]), Some(0));
    assert_eq!(copied_answer("a1 b2 c3 d4 e5 f6", &[label]), None);
    // Multiskup: ponovljena riječ ne vrijedi više od jednom.
    assert_eq!(copied_answer("a a a a", &["a b c d"]), None);
}

#[test] // 10
fn copied_answer_prefers_highest_share_then_lowest_index() {
    let (a, b) = ("a b c d e f g h i j", "a b c d e f g h i k");
    assert_eq!(copied_answer(a, &[a, b]), Some(0));
    assert_eq!(copied_answer(a, &[b, a]), Some(1));
    assert_eq!(copied_answer("x y z", &["x y z", "x y z"]), Some(0));
}

/// Cijeli dnevnik presude i obračun za zalijepljeni tekst (sve oznake jedna za drugom).
fn pasted_text(s: &Sit) -> String {
    s.labels.join(" ")
}

#[test] // 13
fn pasted_answers_score_neutral_three() {
    // Obračun neutralne presude 3 (Classic, tekst nije zalijepljen) = mjerilo za mehanike šutnje i 0 €.
    let (mut gn, sn) = first_free(Some(AnswerRule::Classic));
    neutral(&mut gn, &sn, "Odgovaram gostu svojim riječima.");
    let reference = close_and_resolve(&mut gn);
    for rule in [None, Some(AnswerRule::Classic)] {
        let (mut g, s) = first_free(rule);
        assert!(pasted_answers(
            &pasted_text(&s),
            &s.labels.iter().map(String::as_str).collect::<Vec<_>>()
        ));
        let v = judge(&mut g, &s, &pasted_text(&s), 5, 0);
        assert_eq!(v.quality, 3, "{rule:?}");
        assert!(v.pasted);
        assert_eq!(v.nearest_answer, None);
        assert_eq!(v.copied, None);
        let res = close_and_resolve(&mut g);
        let r = result_of(&res, &s);
        assert_eq!((r.quality, r.capped, r.free_text), (3, false, true));
        assert_eq!(
            res, reference,
            "mehanike šutnje (0 €) kao neutralna 3, {rule:?}"
        );
    }
    // Neutralna presuda s zalijepljenim tekstom: isto.
    let (mut g, s) = first_free(None);
    let v = neutral(&mut g, &s, &pasted_text(&s));
    assert_eq!(
        (v.quality, v.pasted, v.nearest_answer, v.copied),
        (3, true, None, None)
    );
}

#[test] // 13
fn pasted_answers_needs_two_covered_labels() {
    let labels = ["a1 b2 c3 d4", "e5 f6 g7 h8"];
    assert!(pasted_answers("a1 b2 c3 d4 e5 f6 g7 h8", &labels));
    assert!(pasted_answers("A1 B2 C3 D4, E5 F6 G7 H8 i9", &labels));
    // Jedna oznaka + svoje riječi: nije zalijepljeno.
    assert!(!pasted_answers("a1 b2 c3 d4 i9 j0 k1 l2", &labels));
    // Druga oznaka pokrivena samo 50 %.
    assert!(!pasted_answers("a1 b2 c3 d4 e5 f6", &labels));
    assert!(!pasted_answers("", &labels));
}

#[test] // 11
fn debrief_marks_capped_and_copied() {
    let ogranicena = |facts: &DebriefFacts| -> usize {
        serde_json::to_value(&facts.situations)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|s: &&Value| s.get("ogranicena_ocjena").is_some())
            .count()
    };
    // Ograničen klik.
    let (mut g, s) = first_free(None);
    let r = g.current_round().unwrap();
    click(&mut g, &s, s.best());
    close_and_resolve(&mut g);
    let input = g.debrief_input(me(), r).unwrap();
    let d = input.situations.iter().find(|x| x.code == s.code).unwrap();
    assert!(d.capped && !d.copied && !d.pasted);
    assert_eq!(ogranicena(&debrief_facts(&input)), 1);
    // Isti klik uz Classic: bez oznake.
    let (mut gc, sc) = first_free(Some(AnswerRule::Classic));
    let rc = gc.current_round().unwrap();
    click(&mut gc, &sc, sc.best());
    close_and_resolve(&mut gc);
    let inputc = gc.debrief_input(me(), rc).unwrap();
    assert!(!inputc.situations.iter().any(|x| x.capped));
    assert_eq!(ogranicena(&debrief_facts(&inputc)), 0);
    // Prepisan odgovor: copied i capped.
    let (mut g, s) = first_free(None);
    let r = g.current_round().unwrap();
    judge(&mut g, &s, &shout(&s.labels[usize::from(s.best())]), 5, 0);
    close_and_resolve(&mut g);
    let input = g.debrief_input(me(), r).unwrap();
    let d = input.situations.iter().find(|x| x.code == s.code).unwrap();
    assert!(d.copied && d.capped);
    // Zalijepljeno: pasted, mentor ima činjenicu.
    let (mut g, s) = first_free(None);
    let r = g.current_round().unwrap();
    judge(&mut g, &s, &pasted_text(&s), 5, 0);
    close_and_resolve(&mut g);
    let input = g.debrief_input(me(), r).unwrap();
    let d = input.situations.iter().find(|x| x.code == s.code).unwrap();
    assert!(d.pasted && !d.copied && !d.capped);
    let json = serde_json::to_value(&debrief_facts(&input).situations).unwrap();
    assert!(json.to_string().contains("zalijepljeni_odgovori"));
    // Neutralni tekst: 4/5 uz OwnWords, 3/5 uz Classic.
    for (rule, grade) in [(None, "4/5"), (Some(AnswerRule::Classic), "3/5")] {
        let (mut g, s) = first_free(rule);
        let r = g.current_round().unwrap();
        neutral(&mut g, &s, "Odgovaram gostu svojim riječima.");
        close_and_resolve(&mut g);
        let input = g.debrief_input(me(), r).unwrap();
        let text = serde_json::to_value(&debrief_facts(&input).situations).unwrap();
        let n = text
            .as_array()
            .unwrap()
            .iter()
            .find_map(|x| x.get("neutralna_ocjena").and_then(Value::as_str))
            .expect("neutralna ocjena u činjenicama mentora");
        assert!(n.contains(grade), "{rule:?}: {n}");
    }
}

#[test] // C1 (pregled): zalijepljeno se prepoznaje po parovima susjednih riječi, ne po skupu riječi
fn pasted_answers_uses_adjacent_word_pairs() {
    let labels = ["a1 b2 c3 d4 e5", "f6 g7 h8 i9 j0"];
    assert!(pasted_answers("a1 b2 c3 d4 e5 f6 g7 h8 i9 j0", &labels));
    assert!(
        pasted_answers("F6 G7 H8 I9 J0 A1 B2 C3 D4 E5", &labels),
        "redoslijed oznaka je slobodan"
    );
    // 🟨 Riječi obiju oznaka ispremetane bez ijednog zajedničkog para susjednih riječi: nije zalijepljeno.
    assert!(!pasted_answers("e5 d4 c3 b2 a1 j0 i9 h8 g7 f6", &labels));
    assert!(!pasted_answers("a1 f6 b2 g7 c3 h8 d4 i9 e5 j0", &labels));
}

#[test] // C1: u cijelom sadržaju jedna prepisana oznaka je prepisana, nikad zalijepljena; sve oznake spojene su zalijepljene
fn content_labels_are_copied_never_pasted() {
    let g = new_game_rule(SEED, None);
    let templates = g.content.situation_templates.as_ref().unwrap();
    assert!(templates.templates.len() >= 55);
    for t in &templates.templates {
        let labels: Vec<&str> = t.answers.iter().map(|a| a.label.as_str()).collect();
        for (i, l) in labels.iter().enumerate() {
            assert!(
                !pasted_answers(l, &labels),
                "{}[{i}]: jedna oznaka nije zalijepljeno",
                t.code
            );
            assert_eq!(
                copied_answer(l, &labels),
                Some(i),
                "{}[{i}]: prepisana oznaka",
                t.code
            );
        }
        assert!(
            pasted_answers(&labels.join(" "), &labels),
            "{}: sve oznake spojene",
            t.code
        );
        let rev: Vec<&str> = labels.iter().rev().copied().collect();
        assert!(
            pasted_answers(&rev.join(" ").to_uppercase(), &labels),
            "{}: oznake obrnutim redom, velika slova",
            t.code
        );
    }
}

#[test] // C1: dobar vlastni odgovor u obveznoj S1 nije zalijepljen, ocjena je iz presude
fn good_own_answer_in_s1_is_not_pasted() {
    for text in [
        "Ispričat ću se obitelji i odmah ih premjestiti u drugu sobu, ponuditi večeru na račun hotela i poslati tehničara da odmah popravi klimu.",
        "Ispričati se obitelji i odmah ih premjestiti u drugu sobu, ponuditi večeru na račun hotela i poslati tehničara da odmah popravi klimu.",
    ] {
        let (mut g, s) = first_with(None, &|s| s.code == "S1");
        let labels: Vec<&str> = s.labels.iter().map(String::as_str).collect();
        assert!(!pasted_answers(text, &labels), "{text}");
        let v = judge(&mut g, &s, text, 4, 1);
        assert!(!v.pasted && v.copied.is_none(), "{text}");
        assert_eq!((v.quality, v.nearest_answer), (4, Some(1)));
        assert!(!g.response_pasted(me(), &s.code));
        let res = close_and_resolve(&mut g);
        let r = result_of(&res, &s);
        assert_eq!((r.quality, r.capped), (4, false));
    }
}

#[test] // I1: javni upit koji app koristi da sud ne bude pozvan
fn response_pasted_reports_the_text_check() {
    let (mut g, s) = first_free(None);
    let ev = g
        .submit_response(me(), &s.code, &s.labels.join(" "))
        .unwrap();
    common::apply_ev(&mut g, guest(), ev);
    assert!(g.response_pasted(me(), &s.code));
    let (mut g, s) = first_free(None);
    let ev = g
        .submit_response(me(), &s.code, "Odgovaram gostu svojim riječima.")
        .unwrap();
    common::apply_ev(&mut g, guest(), ev);
    assert!(!g.response_pasted(me(), &s.code));
}
