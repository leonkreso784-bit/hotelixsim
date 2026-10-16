//! G-1, dodatak čuvaru (`situations_drawn.rs`): ono što čuvar ne pokriva izravno — sidra S1–S4 u izvlačenju su
//! označena kao sidra (isti poziv generatora kao sezona robota), voditelj za situaciju cijelog hotela, kolektivna
//! situacija jednom u događaju i u popisu svakog hotela, oznaka „samo nagrada“ iz dnevnika, slobodan odgovor na
//! primjerak (ne pada), pokvaren dnevnik (dvostruko izvlačenje).

mod common;

use std::collections::BTreeSet;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::event::DrawnItem;
use game::*;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::situations::DrawView;
use uuid::Uuid;

fn v2_events(seed: u64, kind: PeriodKind) -> Vec<Recorded> {
    new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        &Snapshot::read_v2(&repo_dir()).unwrap(),
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind,
            situations: None,
            answer_rule: None,
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect()
}

fn drawn_of(log: &[Recorded], round: RoundKey) -> Vec<DrawnItem> {
    log.iter()
        .filter_map(|r| match &r.event {
            GameEvent::SituationsDrawn(d) if d.month == round => Some(d.situations.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Sidra S1–S4 ulaze u izvlačenje kao `Anchor`: igra zove isti `draw_for_round` kao sezona robota s istim
/// sidrima, pa je događaj prvog kruga točno ono što taj poziv vrati; bez sidra bi generator izvukao više.
#[test]
fn anchors_are_passed_to_the_generator_as_anchors() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let log = v2_events(42, kind);
        let g = Game::fold(&log).unwrap();
        let first = g.rounds[0];
        let info = g.round_info(first).unwrap();
        let h = &g.hotels[0];
        let CoreState::V2(state) = g.state_at(h.setup.hotel_id, first).unwrap() else {
            panic!("v2 igra ima v2 stanje");
        };
        let set = g.content.situation_templates.as_ref().unwrap().to_core();
        let cat = g.content.v2.as_ref().unwrap();
        let view = DrawView {
            state: &state,
            previous: None,
            indices: &h.start_indices,
        };
        let call = |anchors| {
            sim_hotel_v2::draw_for_round(
                anchors,
                &set,
                (g.seed, h.setup.slot),
                &info.period,
                &view,
                &[],
                cat,
            )
            .unwrap()
        };
        let (anchors_here, with) = call(&g.content.situations);
        let (_, without) = call(&[]);
        assert!(
            !anchors_here.is_empty(),
            "{kind:?}: prvi krug (lipanj) nosi sidro S1"
        );
        // Događaj hotela igrača = poziv generatora sa sidrima (kolektivna je u njemu jednom, kao u pozivu).
        let want: BTreeSet<String> = with.iter().map(|d| d.code.clone()).collect();
        let got: BTreeSet<String> = drawn_of(&log, first)
            .iter()
            .filter(|i| i.slot == 0 || i.slot == h.setup.slot)
            .map(|i| i.code.clone())
            .collect();
        assert_eq!(got, want, "{kind:?}");
        // Sidra zauzimaju mjesta: bez njih generator izvlači više primjeraka.
        assert!(
            without.len() > with.len(),
            "{kind:?}: bez sidara {} primjeraka, sa sidrima {}",
            without.len(),
            with.len()
        );
        // I u popisu situacija igrača sidra stoje uz primjerke (sidra + primjerci = broj kruga).
        let codes: Vec<String> = g
            .situations_in(first)
            .iter()
            .map(|t| t.code.clone())
            .collect();
        for a in &anchors_here {
            assert!(codes.contains(&a.code), "{kind:?}: sidro {}", a.code);
        }
        assert_eq!(codes.len(), anchors_here.len() + with.len(), "{kind:?}");
    }
}

/// Kolektivna situacija (utor 0) je u događaju jednom po krugu, a u popisu svakog hotela; ostali primjerci samo
/// kod hotela s tim utorom.
#[test]
fn collective_is_recorded_once_and_shared_by_every_hotel() {
    let log = v2_events(7, PeriodKind::Month);
    let g = Game::fold(&log).unwrap();
    let first = g.rounds[0];
    let items = drawn_of(&log, first);
    let collective: Vec<&DrawnItem> = items.iter().filter(|i| i.slot == 0).collect();
    assert!(collective.len() <= 1, "najviše jedna kolektivna mjesečno");
    for h in &g.hotels {
        let codes: BTreeSet<String> = g
            .situations_of_hotel(h.setup.hotel_id, first)
            .unwrap()
            .iter()
            .map(|t| t.code.clone())
            .collect();
        // Šifra primjerka je jedinstvena samo unutar hotela (dva hotela mogu dobiti isti predložak), pa se
        // provjerava samo da hotel ima sve svoje i kolektivne primjerke.
        for i in items
            .iter()
            .filter(|i| i.slot == 0 || i.slot == h.setup.slot)
        {
            assert!(codes.contains(&i.code), "hotel {} {}", h.setup.slot, i.code);
        }
    }
}

/// Voditelj: odjel primjerka; za situaciju cijelog hotela (sidra) odjel s najviše zaposlenih, pri jednakosti prvi
/// po kodu odjela 🟨.
#[test]
fn leader_is_the_department_of_the_situation_or_the_biggest_open_department() {
    let log = v2_events(42, PeriodKind::Month);
    let g = Game::fold(&log).unwrap();
    let h = common::hotel_ids()[0];
    let first = g.rounds[0];
    let items = drawn_of(&log, first);
    let with_dept = items
        .iter()
        .find(|i| i.department.is_some() && (i.slot == 0 || i.slot == 1))
        .expect("barem jedan primjerak s odjelom");
    assert_eq!(
        g.situation_leader(h, &with_dept.code).unwrap(),
        with_dept.department
    );
    // Sidro S1 je cijeli hotel: voditelj je odjel s najviše zaposlenih (stalni + sezonski).
    let CoreState::V2(state) = g.state_at(h, first).unwrap() else {
        panic!("v2 stanje");
    };
    let mut expected: Option<(String, u32)> = None;
    for (code, d) in state.departments.iter().filter(|(_, d)| d.open) {
        let n = u32::from(d.permanent) + u32::from(d.seasonal);
        if expected.as_ref().is_none_or(|(_, b)| n > *b) {
            expected = Some((code.as_str().to_owned(), n));
        }
    }
    let expected = expected.map(|(c, _)| c);
    assert!(expected.is_some());
    assert_eq!(g.situation_leader(h, "S1").unwrap(), expected);
    assert!(matches!(
        g.situation_leader(h, "NEMA"),
        Err(GameError::UnknownSituation)
    ));
}

/// v1 igra nema odjela: nema voditelja (i nije greška).
#[test]
fn v1_game_has_no_leader() {
    let (g, h) = common::open_june_game();
    let code = g.situations_in(sim_hotel::Month::Jun)[0].code.clone();
    assert_eq!(g.situation_leader(h, &code).unwrap(), None);
}

/// G-2: primjerak čiji predložak ima lika ima razgovor, slobodan odgovor i presudu (ne `NotFreeText`); primjerak
/// predloška bez lika i sidro bez slobodnog odgovora (S3) i dalje daju „bez slobodnog odgovora“, ne grešku sustava.
/// Opsežnije provjere (ulazi bez `{…}`, feed, odjel): `situations_ai_input.rs`.
#[test]
fn free_answer_on_a_drawn_instance_follows_its_template_character() {
    let h = common::hotel_ids()[0];
    let (mut with_char, mut without) = (false, false);
    for seed in 1..=60_u64 {
        let g = Game::fold(&v2_events(seed, PeriodKind::Month)).unwrap();
        let r = g.current_round().unwrap();
        for inst in &g.hotel(h).unwrap().drawn[&r] {
            let code = inst.situation.code.clone();
            let has_char = g
                .content
                .situation_templates
                .as_ref()
                .unwrap()
                .templates
                .iter()
                .any(|t| t.code == inst.drawn.template && t.character.is_some());
            if has_char && !with_char {
                with_char = true;
                g.send_message(h, &code, "Poruka.").unwrap();
                g.submit_response(h, &code, "Odgovor.").unwrap();
                g.validate_verdict(&code, common::draft_with(3, 0)).unwrap();
                g.character_input(h, &code).unwrap();
            } else if !has_char && !without {
                without = true;
                assert_eq!(
                    g.submit_response(h, &code, "Odgovor."),
                    Err(GameError::NotFreeText)
                );
                assert_eq!(
                    g.send_message(h, &code, "Poruka."),
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
            }
            // Ponuđeni odgovor radi uvijek.
            g.choose_answer(h, &code, 0).unwrap();
        }
        if with_char && without {
            // Sidro S3 nema slobodan odgovor ni u v2 igri.
            assert_eq!(
                g.send_message(h, "S3", "Poruka."),
                Err(GameError::NotFreeText)
            );
            return;
        }
    }
    panic!("nije viđen primjerak s likom i bez njega: {with_char} {without}");
}

/// Oznaka „samo nagrada“ dolazi iz predloška primjerka u dnevniku, ne iz rastavljanja šifre.
#[test]
fn no_penalty_mark_comes_from_the_template_of_the_instance() {
    let mut seen = (false, false);
    for seed in 1..=300_u64 {
        let g = Game::fold(&v2_events(seed, PeriodKind::Month)).unwrap();
        let r = g.rounds[0];
        for inst in &g.hotel(common::hotel_ids()[0]).unwrap().drawn[&r] {
            let want = g
                .content
                .situation_templates
                .as_ref()
                .unwrap()
                .templates
                .iter()
                .any(|t| t.code == inst.drawn.template && t.no_penalty);
            assert_eq!(inst.no_penalty, want, "{}", inst.situation.code);
            if inst.no_penalty {
                seen.0 = true;
                assert_eq!(inst.drawn.template, "P-POHVALA");
            } else {
                seen.1 = true;
            }
        }
        if seen.0 && seen.1 {
            return;
        }
    }
    panic!("nije viđen primjerak s oznakom i bez nje: {seen:?}");
}

/// Dnevnik s drugim izvlačenjem istog kruga je pokvaren (ne tiho prepisan).
#[test]
fn drawing_the_same_round_twice_is_a_corrupt_log() {
    let log = v2_events(42, PeriodKind::Week);
    let mut g = Game::fold(&log).unwrap();
    let again = log
        .iter()
        .find(|r| r.event.kind() == "SituationsDrawn")
        .unwrap()
        .event
        .clone();
    let seq = g.last_seq + 1;
    let err = g
        .apply(Recorded {
            seq,
            actor: Actor::System,
            event: again,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Corrupt(_)), "{err:?}");
    // Igra koja se već izvukla ne izvlači ponovno otvoreni krug.
    assert!(g.draw_round(g.rounds[0]).unwrap().is_none());
}

/// Izvlačenje idućeg kruga ne pokreće se za igru koja nije imala izvlačenje od početka (stara v2 igra).
#[test]
fn an_old_v2_game_is_never_drawn_for_later_rounds() {
    let log: Vec<Recorded> = v2_events(42, PeriodKind::Week)
        .into_iter()
        .filter(|r| r.event.kind() != "SituationsDrawn")
        .filter(|r| match &r.event {
            GameEvent::AnswerChosen(a) => a.situation.len() == 2 && a.situation.starts_with('S'),
            _ => true,
        })
        .enumerate()
        .map(|(i, r)| Recorded {
            seq: i64::try_from(i).unwrap() + 1,
            ..r
        })
        .collect();
    let mut g = Game::fold(&log).unwrap();
    let r = g.current_round().unwrap();
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("čeka")
    };
    assert!(
        evs.iter().all(|(_, e)| e.kind() != "SituationsDrawn"),
        "stara igra ne dobiva izvlačenje u idućem krugu"
    );
    let next = g.rounds[1];
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    assert!(!g.is_drawn(next));
    assert!(g.draw_round(next).unwrap().is_none());
}
