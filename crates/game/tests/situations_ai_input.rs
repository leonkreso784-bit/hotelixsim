//! G-2: AI ulaz iz izvučenog primjerka. Lik, presuda i mentor za primjerak čitaju tekstove POPUNJENOG primjerka
//! (ime i opis uloge, brojevna mjesta), nikad predložak s neispunjenim `{…}`; razgovor, slobodan odgovor i presuda
//! rade i na primjerku (ne `NotFreeText`), presuda na primjerak ne ruši feed (M2), a `DebriefSituation` nosi odjel.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti, mijenja se samo ovaj blok):
//! - Postojeći pozivi `Game::{character_input, assess_input, debrief_input, send_message, submit_response,
//!   record_reply, validate_verdict, record_verdict, feed}` rade i za šifru primjerka (`P-X.t03`). Primjerak ima
//!   AI ulaz kad njegov predložak ima `character` (Leon P6: lik = razgovor + slobodan odgovor); predložak bez
//!   lika ostaje `NotFreeText`.
//! - Ulazi se čitaju kao JSON (`serde_json::to_value`), pa test pada ponašanjem, ne prevođenjem. `DebriefSituation`
//!   dobiva polje `department` (JSON: tekst ili `null`/odsutno). Vrijednost je ŠIFRA ili IME odjela primjerka
//!   (`department_ok` prihvaća oboje); `null`/odsutno za sidra S1–S4, v1 i predložak bez odjela.
//! - `CharacterCard.role` za primjerak sadrži opis uloge primjerka (`role_type`, npr. „majka s dvoje male djece“),
//!   `CharacterCard.name` je ime popunjene uloge iz `DrawnItem.roles[character.role]`, `brief` sadrži to ime.
//! - `AssessInput.offered[i]` = ponuđeni odgovor primjerka i (oznaka iz `situation.answers`, kriteriji iz predloška);
//!   `AssessInput.rubric` = rubrika predloška (5 kriterija); `AssessInput.character` = ime lika.
//! - Opis situacije nije u postojećim ulazima; ako ga graditelj doda kao polje, JSON provjera ga pokriva.

mod common;

use std::collections::BTreeSet;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::event::AiSource;
use game::feed::FeedItem;
use game::*;
use serde::Serialize;
use serde_json::Value;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

const STUDENT_MESSAGE: &str = "Razumijem vašu situaciju i odmah ću se pobrinuti za rješenje.";
const STUDENT_ANSWER: &str = "Ispričat ću se, ponuditi rješenje odmah i provjeriti kasnije.";

fn v2_game(seed: u64, kind: PeriodKind) -> Game {
    let log: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        &Snapshot::read_v2(&repo_dir()).unwrap(),
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind,
            situations: None,
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    Game::fold(&log).unwrap()
}

/// Što test treba znati o predlošku izvučenog primjerka.
struct Spec {
    character_role: Option<String>,
    rubric: Vec<String>,
    criteria: Vec<Vec<u8>>,
}

fn spec_of(g: &Game, template: &str) -> Spec {
    let t = g
        .content
        .situation_templates
        .as_ref()
        .unwrap()
        .templates
        .iter()
        .find(|t| t.code == template)
        .unwrap_or_else(|| panic!("predložak {template}"));
    Spec {
        character_role: t.character.as_ref().map(|c| c.role.clone()),
        rubric: t.rubric.clone(),
        criteria: t.answers.iter().map(|a| a.criteria.clone()).collect(),
    }
}

/// Sva tekstualna polja JSON-a (vrijednosti, ne ključevi).
fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(m) => m.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

/// Nijedan niz koji ide AI-ju ne smije sadržavati neispunjeno mjesto (`{` ili `}`).
fn assert_clean<T: Serialize>(what: &str, input: &T) {
    let v = serde_json::to_value(input).unwrap();
    let mut all = Vec::new();
    strings(&v, &mut all);
    for s in &all {
        assert!(
            !s.contains('{') && !s.contains('}'),
            "{what}: neispunjeno mjesto u: {s}"
        );
    }
}

fn apply_guest(g: &mut Game, ev: GameEvent) {
    common::apply_ev(g, Actor::Guest(common::GUEST), ev);
}

fn apply_system(g: &mut Game, ev: GameEvent) {
    common::apply_ev(g, Actor::System, ev);
}

fn reply() -> ReplyBody {
    ReplyBody {
        text: "Hvala, čekam vaše rješenje.".into(),
        source: AiSource::Ai,
        meta: None,
    }
}

/// Razgovor jednom porukom i odgovorom lika, pa slobodan odgovor (kao u igri).
fn talk_and_submit(g: &mut Game, h: HotelId, code: &str) {
    let ev = g
        .send_message(h, code, STUDENT_MESSAGE)
        .unwrap_or_else(|e| panic!("{code}: send_message na primjerak: {e:?}"));
    apply_guest(g, ev);
    let ev = g
        .record_reply(h, code, 1, reply())
        .unwrap_or_else(|e| panic!("{code}: record_reply: {e:?}"))
        .expect("odgovor lika se upisuje");
    apply_system(g, ev);
    let ev = g
        .submit_response(h, code, STUDENT_ANSWER)
        .unwrap_or_else(|e| panic!("{code}: submit_response na primjerak: {e:?}"));
    apply_guest(g, ev);
}

fn verdict(g: &mut Game, h: HotelId, code: &str) {
    let v = g
        .validate_verdict(code, common::draft_with(3, 0))
        .unwrap_or_else(|e| panic!("{code}: validate_verdict na primjerak: {e:?}"));
    let ev = g.record_verdict(h, code, v, common::meta()).unwrap();
    apply_system(g, ev);
}

fn department_of(s: &DebriefSituation) -> Option<String> {
    serde_json::to_value(s)
        .unwrap()
        .get("department")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// Šifra ili ime odjela primjerka (ime po profilu hotela).
fn department_ok(g: &Game, h: HotelId, got: &str, code: &str) -> bool {
    if got == code {
        return true;
    }
    let profile = g.hotel(h).unwrap().profile;
    let cat = g.content.v2.as_ref().unwrap();
    cat.dept_name(
        profile,
        &sim_hotel_v2::department::DeptCode(code.to_owned()),
    )
    .is_ok_and(|n| n == got)
}

/// Prvi primjerak prvog kruga (po sjemenima) kojem je `character` jednak traženom; vraća igru i šifru.
fn first_instance(has_character: bool) -> (Game, HotelId, String) {
    let h = common::hotel_ids()[0];
    for seed in 1..=60_u64 {
        let g = v2_game(seed, PeriodKind::Month);
        let r = g.current_round().unwrap();
        for inst in &g.hotel(h).unwrap().drawn[&r] {
            if spec_of(&g, &inst.drawn.template).character_role.is_some() == has_character {
                let code = inst.situation.code.clone();
                return (g, h, code);
            }
        }
    }
    panic!("nema primjerka (lik: {has_character}) u prvom mjesečnom krugu 60 sjemena");
}

/// Sve provjere za jedan krug hotela igrača: ulazi za lik i presudu, razgovor, presuda, feed; zatvara krug,
/// obračunava ga i provjerava `DebriefSituation`. `seen` dobiva (predložak, ima li lik).
fn exercise_round(g: &mut Game, r: RoundKey, seen: &mut BTreeSet<(String, bool)>) {
    let h = common::hotel_ids()[0];
    let insts = g
        .hotel(h)
        .unwrap()
        .drawn
        .get(&r)
        .cloned()
        .unwrap_or_default();
    for inst in &insts {
        let code = inst.situation.code.clone();
        let template = inst.drawn.template.clone();
        let spec = spec_of(g, &template);
        seen.insert((template.clone(), spec.character_role.is_some()));
        // Ponuđeni odgovori su tekst koji student vidi i koji ide sucu: i oni moraju biti popunjeni.
        let title = inst.situation.title.clone();
        assert!(
            !title.contains('{') && !title.contains('}'),
            "{code}: naslov primjerka ima neispunjeno mjesto: {title}"
        );
        for a in &inst.situation.answers {
            assert!(
                !a.label.contains('{') && !a.label.contains('}'),
                "{code}: oznaka odgovora ima neispunjeno mjesto: {}",
                a.label
            );
        }
        match &spec.character_role {
            Some(role_key) => {
                let ci = g
                    .character_input(h, &code)
                    .unwrap_or_else(|e| panic!("{code}: character_input: {e:?}"));
                assert_clean(&format!("{code}: CharacterInput"), &ci);
                assert_eq!(ci.situation, code);
                assert_eq!(ci.title, title, "{code}: naslov lika je naslov primjerka");
                let role = &inst.drawn.roles[role_key];
                assert_eq!(ci.character.name, role.name, "{code}: ime lika");
                assert!(
                    !ci.character.role.trim().is_empty(),
                    "{code}: uloga lika je prazna"
                );
                if let Some(ty) = &role.role_type {
                    assert!(
                        ci.character.role.contains(ty.as_str()),
                        "{code}: uloga '{}' ne sadrži opis uloge '{ty}'",
                        ci.character.role
                    );
                }
                assert!(
                    ci.character.brief.contains(role.name.as_str()),
                    "{code}: brief ne sadrži popunjeno ime"
                );
                assert!(!ci.character.opening.trim().is_empty());
                talk_and_submit(g, h, &code);
                let ci2 = g.character_input(h, &code).unwrap();
                assert_clean(&format!("{code}: CharacterInput s razgovorom"), &ci2);
                assert_eq!(ci2.transcript.len(), 2);
                let ai = g
                    .assess_input(h, &code)
                    .unwrap_or_else(|e| panic!("{code}: assess_input: {e:?}"));
                assert_clean(&format!("{code}: AssessInput"), &ai);
                assert_eq!(ai.title, title);
                assert_eq!(ai.rubric, spec.rubric, "{code}: rubrika predloška");
                assert_eq!(ai.character.as_deref(), Some(role.name.as_str()));
                assert_eq!(ai.response, STUDENT_ANSWER);
                assert_eq!(ai.transcript.len(), 2);
                assert_eq!(ai.offered.len(), inst.situation.answers.len());
                for (i, o) in ai.offered.iter().enumerate() {
                    assert_eq!(
                        o.label, inst.situation.answers[i].label,
                        "{code}: odgovor {i}"
                    );
                    assert_eq!(
                        o.criteria, spec.criteria[i],
                        "{code}: kriteriji odgovora {i}"
                    );
                }
                verdict(g, h, &code);
                // M2: presuda na primjerak ne ruši feed, naslov je naslov primjerka.
                let feed = g
                    .feed(h)
                    .unwrap_or_else(|e| panic!("{code}: feed nakon presude: {e:?}"));
                assert!(
                    feed.iter().any(|f| matches!(
                        f,
                        FeedItem::Verdict { situation, title: t, .. }
                            if *situation == code && *t == title
                    )),
                    "{code}: feed nema presudu s naslovom primjerka"
                );
            }
            None => {
                assert!(
                    matches!(g.character_input(h, &code), Err(GameError::NotFreeText)),
                    "{code}: predložak bez lika nema ulaz za lika"
                );
                assert_eq!(
                    g.send_message(h, &code, "Poruka."),
                    Err(GameError::NotFreeText)
                );
                assert_eq!(
                    g.submit_response(h, &code, "Odgovor."),
                    Err(GameError::NotFreeText)
                );
                assert!(matches!(
                    g.validate_verdict(&code, common::draft_with(3, 0)),
                    Err(VerdictError::NotFreeText)
                ));
                let ev = g.choose_answer(h, &code, 0).unwrap();
                apply_guest(g, ev);
                let feed = g.feed(h).unwrap();
                assert!(feed.iter().any(|f| matches!(
                    f,
                    FeedItem::AnswerChosen { situation, title: t, .. }
                        if *situation == code && *t == title
                )));
            }
        }
    }
    // Krug se zatvara i obračunava; mentor vidi situacije s odjelom.
    let ev = g.close_round(r).unwrap().unwrap();
    apply_guest(g, ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        common::apply_ev(g, a, e);
    }
    let d = g.debrief_input(h, r).unwrap();
    assert_clean("DebriefSituation", &d.situations);
    for s in &d.situations {
        match insts.iter().find(|i| i.situation.code == s.code) {
            Some(inst) => {
                assert_eq!(s.title, inst.situation.title, "{}: naslov u osvrtu", s.code);
                match (&inst.drawn.department, department_of(s)) {
                    (Some(want), Some(got)) => assert!(
                        department_ok(g, h, &got, want.as_str()),
                        "{}: odjel '{got}' nije odjel primjerka '{}'",
                        s.code,
                        want.as_str()
                    ),
                    (Some(want), None) => {
                        panic!(
                            "{}: DebriefSituation nema odjel '{}'",
                            s.code,
                            want.as_str()
                        )
                    }
                    (None, got) => assert_eq!(got, None, "{}: predložak bez odjela", s.code),
                }
            }
            None => assert_eq!(
                department_of(s),
                None,
                "{}: sidro nema odjel u osvrtu",
                s.code
            ),
        }
    }
}

fn exercise_all(
    kind: PeriodKind,
    seeds: std::ops::RangeInclusive<u64>,
) -> BTreeSet<(String, bool)> {
    let mut seen = BTreeSet::new();
    for seed in seeds {
        let mut g = v2_game(seed, kind);
        while let Some(r) = g.current_round() {
            exercise_round(&mut g, r, &mut seen);
        }
    }
    seen
}

/// Tjedna sezona: ulazi za lik, presudu i mentora iz popunjenog primjerka, razgovor, presuda i feed na svakom
/// izvučenom primjerku više sjemena.
#[test]
fn weekly_instances_give_clean_ai_inputs_and_working_talk() {
    exercise_all(PeriodKind::Week, 1..=3);
}

/// Mjesečna sezona (do 22 situacije u krugu) isto.
#[test]
fn monthly_instances_give_clean_ai_inputs_and_working_talk() {
    exercise_all(PeriodKind::Month, 1..=3);
}

/// Sjemena iz gornjih testova doista dohvaćaju i predloške s likom i bez njega (test nije prazan).
#[test]
fn the_seeds_cover_templates_with_and_without_a_character() {
    let mut all: BTreeSet<(String, bool)> = BTreeSet::new();
    for (kind, seeds) in [(PeriodKind::Week, 1..=3_u64), (PeriodKind::Month, 1..=3)] {
        for seed in seeds {
            let mut g = v2_game(seed, kind);
            while let Some(r) = g.current_round() {
                let ev = g.close_round(r).unwrap().unwrap();
                apply_guest(&mut g, ev);
                let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
                    panic!("obračun je trebao biti gotov");
                };
                for (a, e) in evs {
                    common::apply_ev(&mut g, a, e);
                }
            }
            for hr in &g.hotels {
                for i in hr.drawn.values().flatten() {
                    let has = spec_of(&g, &i.drawn.template).character_role.is_some();
                    all.insert((i.drawn.template.clone(), has));
                }
            }
        }
    }
    let with = all.iter().filter(|(_, c)| *c).count();
    let without = all.len() - with;
    assert!(with >= COVER_WITH, "predložaka s likom: {with}");
    assert!(without >= COVER_WITHOUT, "predložaka bez lika: {without}");
}

/// Izmjereno (3 sjemena tjedno + 3 mjesečno): 32 predloška s likom, 13 bez; prag s rezervom.
const COVER_WITH: usize = 28;
const COVER_WITHOUT: usize = 10;

/// Razgovor, slobodan odgovor i presuda rade na primjerku s likom (ne `NotFreeText`).
#[test]
fn talk_response_and_verdict_work_on_an_instance_with_a_character() {
    let (mut g, h, code) = first_instance(true);
    talk_and_submit(&mut g, h, &code);
    verdict(&mut g, h, &code);
}

/// Primjerak predloška bez lika ostaje bez razgovora i slobodnog odgovora (`NotFreeText`); ponuđeni odgovor radi.
#[test]
fn an_instance_without_a_character_stays_not_free_text() {
    let (g, h, code) = first_instance(false);
    assert_eq!(g.send_message(h, &code, "P."), Err(GameError::NotFreeText));
    assert_eq!(
        g.submit_response(h, &code, "O."),
        Err(GameError::NotFreeText)
    );
    assert!(matches!(
        g.validate_verdict(&code, common::draft_with(3, 0)),
        Err(VerdictError::NotFreeText)
    ));
    assert!(matches!(
        g.character_input(h, &code),
        Err(GameError::NotFreeText)
    ));
    assert!(g.choose_answer(h, &code, 0).is_ok());
}

/// M2: `VerdictRecorded` za šifru primjerka ne ruši feed (naslov se traži u izvučenim primjercima hotela),
/// a naslov u feedu je naslov primjerka; feed ostaje valjan i za druge hotele (kolektivni primjerci).
#[test]
fn verdict_on_an_instance_does_not_break_the_feed() {
    let (mut g, h, code) = first_instance(true);
    let title = g.situation_round_for(h, &code).unwrap().1.title.clone();
    talk_and_submit(&mut g, h, &code);
    verdict(&mut g, h, &code);
    for hotel in common::hotel_ids() {
        g.feed(hotel)
            .unwrap_or_else(|e| panic!("feed hotela nakon presude na primjerak: {e:?}"));
    }
    let feed = g.feed(h).unwrap();
    assert!(
        feed.iter().any(|f| matches!(
            f,
            FeedItem::Verdict { situation, title: t, .. } if *situation == code && *t == title
        )),
        "nema stavke presude s naslovom primjerka"
    );
    assert!(!title.contains('{') && !title.contains('}'), "{title}");
}

/// Uloga lika u `CharacterInput` nije prazna i nosi opis uloge primjerka (rečenica „tko je lik“).
#[test]
fn character_role_sentence_comes_from_the_filled_instance() {
    let (g, h, code) = first_instance(true);
    let ci = g.character_input(h, &code).unwrap();
    assert!(!ci.character.role.trim().is_empty());
    assert!(!ci.character.name.trim().is_empty());
    assert_clean("CharacterInput", &ci);
}

/// `DebriefSituation`: sidra S1–S4 u v2 igri i situacije v1 igre nemaju odjel.
#[test]
fn anchors_and_v1_situations_have_no_department_in_the_debrief() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let h = common::hotel_ids()[0];
    let r = RoundKey::parse("t01").unwrap();
    let d = g.debrief_input(h, r).unwrap();
    assert!(!d.situations.is_empty());
    for s in d.situations.iter().filter(|s| !s.code.contains('.')) {
        assert_eq!(department_of(s), None, "sidro {}", s.code);
    }
    let g1 = common::june_resolved_game();
    let d1 = g1
        .debrief_input(common::hotel_ids()[0], sim_hotel::Month::Jun)
        .unwrap();
    for s in &d1.situations {
        assert_eq!(department_of(s), None, "v1 {}", s.code);
    }
}
