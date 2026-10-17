//! B3a: balans za svaki profil i sjeme vrti isti zadani plan, a mijenja se samo kako se rješavaju situacije (pet
//! putova). Review Focus 2: igra s čovjekom koji igra put daje isto što i `path_season_v2`.
//! PRAVILO-b (ADR-047): šesti put „klik najbolji“ (klik na najbolji ponuđeni odgovor, najviše 4 gdje student može pisati).
use balance::v2::{
    AnswerPath, CLICK_MAX_QUALITY, DELEGATED_QUALITY, HOTEL_SLOT, MANDATORY_SITUATIONS, OptionsV2,
    horizon, offered_choice, path_season_v2, render_v2, run_v2, writable_v2,
};
use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::dto::indices_dto;
use game::event::{AiMeta, AnswerRule, CriterionVerdict};
use game::{
    Actor, Controller, Game, GameEvent, GameId, GuestId, HotelId, Recorded, ResolveStep,
    VerdictDraft,
};
use sim_hotel_v2::{
    PeriodKind, Profile, RoundItem, SeasonResult, SituationTemplate, situations_for,
};
use uuid::Uuid;

/// Po krugu: šifre situacija (sortirane), pet pokazatelja nakon kruga, napredak od početka (stotinke).
type Trace = Vec<(Vec<String>, [u16; 5], i32)>;

fn apply(g: &mut Game, actor: Actor, event: GameEvent) {
    let seq = g.last_seq + 1;
    g.apply(Recorded { seq, actor, event }).unwrap();
}

/// Student piše vlastitim riječima: ni jedan ponuđeni odgovor nije prepisan ni zalijepljen.
const OWN_TEXT: &str = "Poštovani gostu, hvala vam na strpljenju. Preuzimam odgovornost, a do kraja dana javljam se s konkretnim rješenjem koje vam odgovara.";

/// Nacrt presude: svih pet kriterija ispunjeno, najbliži ponuđeni odgovor `nearest` (ocjena 5).
fn full_marks(nearest: u8) -> VerdictDraft {
    VerdictDraft {
        criteria: (1u8..=5)
            .map(|n| CriterionVerdict {
                n,
                met: true,
                reason: format!("razlog {n}"),
            })
            .collect(),
        nearest_answer: nearest,
        rationale: "obrazloženje".into(),
    }
}

fn test_meta() -> AiMeta {
    AiMeta {
        model: "test-model".into(),
        prompt: "test-prompt".into(),
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Igra s jednim ljudskim hotelom u utoru balansa, zadani broj situacija, bez plana (→ zadani plan). Igrač igra
/// put `path` kroz naredbe igre i odigra sezonu do kraja.
fn human_game(seed: u64, kind: PeriodKind, profile: Profile, path: AnswerPath) -> Game {
    let guest = GuestId(Uuid::from_u128(99));
    let ids = [1_u128, 2, 3, 4].map(|n| HotelId(Uuid::from_u128(n)));
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let events = new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        &snap,
        guest,
        ids,
        GameSettings {
            kind,
            situations: None,
            // PRAVILO-b: paritet uz novo pravilo (zadano za nove v2 igre): klik u situaciji s likom vrijedi najviše 4.
            answer_rule: Some(AnswerRule::OwnWords),
        },
    )
    .unwrap();
    let Some((actor, GameEvent::GameCreated(mut created))) = events.into_iter().next() else {
        panic!("prvi događaj nije GameCreated")
    };
    let mut hotel = created.hotels.remove(0);
    hotel.slot = HOTEL_SLOT;
    hotel.profile = profile.code().into();
    hotel.controller = Controller::Human { guest_id: guest };
    let me = hotel.hotel_id;
    created.hotels = vec![hotel];
    let first = Recorded {
        seq: 1,
        actor,
        event: GameEvent::GameCreated(created),
    };
    let mut g = Game::fold(std::slice::from_ref(&first)).unwrap();
    let r = g.rounds[0];
    for (a, e) in g.open_round(r).unwrap() {
        apply(&mut g, a, e);
    }
    let me_actor = Actor::Guest(guest);
    while let Some(r) = g.current_round() {
        let list: Vec<(String, Option<usize>)> = g
            .situations_of_hotel(me, r)
            .unwrap()
            .iter()
            .map(|t| (t.code.clone(), offered_choice(path, t)))
            .collect();
        for (code, choice) in list {
            match path {
                AnswerPath::Best if g.click_capped(me, &code) => {
                    // Vlastiti odgovor (ADR-047 t. 3): u situaciji s likom student piše, presuda daje 5 kriterija.
                    let a = u8::try_from(choice.expect("Best nudi odgovor")).unwrap();
                    let ev = g.submit_response(me, &code, OWN_TEXT).unwrap();
                    apply(&mut g, me_actor.clone(), ev);
                    let v = g.validate_verdict_for(me, &code, full_marks(a)).unwrap();
                    let ev = g.record_verdict(me, &code, v, test_meta()).unwrap();
                    apply(&mut g, Actor::System, ev);
                }
                AnswerPath::Best | AnswerPath::ClickBest | AnswerPath::Middle => {
                    let a =
                        u8::try_from(choice.expect("Best/ClickBest/Middle nude odgovor")).unwrap();
                    let ev = g.choose_answer(me, &code, a).unwrap();
                    apply(&mut g, me_actor.clone(), ev);
                }
                AnswerPath::Delegated => {
                    if !g.situation_mandatory(me, &code).unwrap() {
                        let ev = g.delegate_situation(me, &code).unwrap();
                        apply(&mut g, me_actor.clone(), ev);
                    }
                }
                AnswerPath::Silent | AnswerPath::NoSituations => {}
            }
        }
        let ev = g.close_round(r).unwrap().unwrap();
        apply(&mut g, me_actor.clone(), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("obračun čeka presude")
        };
        for (a, e) in evs {
            apply(&mut g, a, e);
        }
    }
    g
}

fn game_trace(g: &Game) -> Trace {
    let id = g.hotels[0].setup.hotel_id;
    g.rounds
        .iter()
        .map(|r| {
            let h = g.resolved[r]
                .hotels
                .iter()
                .find(|h| h.hotel_id == id)
                .unwrap();
            let mut codes: Vec<String> = h.situations.iter().map(|s| s.code.clone()).collect();
            codes.sort();
            (codes, h.indices, h.progress_hundredths)
        })
        .collect()
}

fn season_trace(s: &SeasonResult, anchors: &[SituationTemplate]) -> Trace {
    s.periods
        .iter()
        .enumerate()
        .map(|(k, (rep, idx))| {
            let mut codes: Vec<String> = situations_for(&rep.period, anchors)
                .iter()
                .map(|t| t.code.clone())
                .chain(s.drawn.get(k).into_iter().flatten().map(|d| d.code.clone()))
                .collect();
            codes.sort();
            (
                codes,
                indices_dto(idx),
                sim_hotel::progress_pts(&s.start, idx).unwrap(),
            )
        })
        .collect()
}

/// Zrcalo pravila igre: balans prepušta iste situacije kao igra.
#[test]
fn mirrors_match_the_game() {
    assert_eq!(MANDATORY_SITUATIONS, game::MANDATORY_SITUATIONS);
    assert_eq!(CLICK_MAX_QUALITY, game::own_words::CLICK_MAX_QUALITY);
    assert_eq!(CLICK_MAX_QUALITY, 4);
}

#[test]
fn path_constants_and_names() {
    assert_eq!(DELEGATED_QUALITY, 2);
    assert_eq!(
        AnswerPath::ALL,
        [
            AnswerPath::Best,
            AnswerPath::ClickBest,
            AnswerPath::Middle,
            AnswerPath::Delegated,
            AnswerPath::Silent,
            AnswerPath::NoSituations
        ]
    );
    let names: Vec<&str> = AnswerPath::ALL.iter().map(|p| p.name()).collect();
    assert_eq!(
        names,
        [
            "najbolji",
            "klik najbolji",
            "srednji",
            "prepušteno",
            "šutnja",
            "bez situacija"
        ]
    );
    assert_eq!(horizon(PeriodKind::Week), 4);
    assert_eq!(horizon(PeriodKind::Month), 2);
}

/// Ponuđeni odgovor: najbolji (najviša kvaliteta, kod jednakih manji indeks) i srednji (stabilno po kvaliteti,
/// indeks len/2) nude indeks, ostali putovi ne nude ništa.
#[test]
fn offered_choice_only_for_best_and_middle() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let mut seen = 0;
    for t in content.situations.iter().filter(|t| t.answers.len() >= 2) {
        let best = offered_choice(AnswerPath::Best, t).unwrap();
        assert_eq!(
            offered_choice(AnswerPath::ClickBest, t),
            Some(best),
            "{}: klik najbolji nudi isti odgovor kao najbolji",
            t.code
        );
        let top = t.answers.iter().map(|a| a.quality).max().unwrap();
        assert_eq!(t.answers[best].quality, top, "{}", t.code);
        assert!(
            t.answers[..best].iter().all(|a| a.quality < top),
            "{}: kod jednake kvalitete manji indeks",
            t.code
        );
        let mut order: Vec<usize> = (0..t.answers.len()).collect();
        order.sort_by_key(|&i| t.answers[i].quality);
        assert_eq!(
            offered_choice(AnswerPath::Middle, t),
            Some(order[order.len() / 2]),
            "{}",
            t.code
        );
        for p in [
            AnswerPath::Delegated,
            AnswerPath::Silent,
            AnswerPath::NoSituations,
        ] {
            assert_eq!(offered_choice(p, t), None, "{} {p:?}", t.code);
        }
        seen += 1;
    }
    assert!(seen > 0, "nema sidara s barem dva odgovora");
}

/// Review Focus 2: igra s čovjekom (utor HOTEL_SLOT, bez plana → zadani plan) koji igra put `path` daje iste
/// situacije, pokazatelje i napredak po krugu kao `path_season_v2`.
#[test]
fn answer_paths_in_game_match_balance() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let mut delegated_month = 0_usize;
    let mut delegated_week = 0_usize;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in Profile::ALL {
            for path in [
                AnswerPath::Best,
                AnswerPath::ClickBest,
                AnswerPath::Middle,
                AnswerPath::Delegated,
                AnswerPath::Silent,
            ] {
                let at = format!("{kind:?} {} {path:?}", profile.code());
                let g = human_game(42, kind, profile, path);
                let s = path_season_v2(&content, profile, path, 42, kind).unwrap();
                let gt = game_trace(&g);
                assert_eq!(gt, season_trace(&s, &content.situations), "{at}");
                assert_eq!(
                    gt.last().unwrap().2,
                    s.progress_pts,
                    "{at}: napredak sezone"
                );
                assert!(
                    gt.iter()
                        .any(|(codes, _, _)| codes.iter().any(|c| c.contains('.'))),
                    "{at}: igra samo sa sidrima"
                );
                if path == AnswerPath::Delegated {
                    let n = g
                        .history
                        .iter()
                        .filter(|r| matches!(r.event, GameEvent::SituationDelegated(_)))
                        .count();
                    match kind {
                        PeriodKind::Week => delegated_week += n,
                        PeriodKind::Month => delegated_month += n,
                    }
                }
            }
        }
    }
    // Usporedba nije prazna: u mjesečnoj igri (4/5/6 situacija) prepuštanja ima. U tjednoj (3/4/5) ih možda
    // nema (sve tri situacije su obvezne), pa se samo ispisuje.
    assert!(delegated_month > 0, "prepuštanje u mjesečnoj igri");
    println!("prepuštanja u tjednoj igri: {delegated_week}");
}

/// Usporedba nije prazna: drugo sjeme daje drugi trag.
#[test]
fn path_comparison_sees_a_different_seed() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let g = human_game(42, PeriodKind::Month, Profile::B, AnswerPath::Best);
    let other = path_season_v2(
        &content,
        Profile::B,
        AnswerPath::Best,
        43,
        PeriodKind::Month,
    )
    .unwrap();
    assert_ne!(game_trace(&g), season_trace(&other, &content.situations));
}

/// Isto sjeme, isti zadani plan: `Best` i `Silent` počinju od istog stanja i istih primjeraka u prvom krugu;
/// put bez situacija nema primjeraka.
#[test]
fn paths_share_plan_and_seed() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in Profile::ALL {
            let best = path_season_v2(&content, profile, AnswerPath::Best, 7, kind).unwrap();
            let silent = path_season_v2(&content, profile, AnswerPath::Silent, 7, kind).unwrap();
            let none =
                path_season_v2(&content, profile, AnswerPath::NoSituations, 7, kind).unwrap();
            let at = format!("{kind:?} {}", profile.code());
            assert_eq!(best.start, silent.start, "{at}");
            assert_eq!(best.periods[0].0.before, silent.periods[0].0.before, "{at}");
            assert_eq!(best.drawn[0], silent.drawn[0], "{at}");
            assert!(
                !best.drawn[0].is_empty(),
                "{at}: nema primjeraka u 1. krugu"
            );
            assert_eq!(best.periods.len(), none.periods.len(), "{at}");
            assert!(none.drawn.iter().all(|d| d.is_empty()), "{at}");
        }
    }
}

#[test]
fn report_prints_answer_paths() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let r = run_v2(
        &content,
        &OptionsV2 {
            seeds: 3,
            kind: PeriodKind::Week,
            situations: None,
        },
    )
    .unwrap();
    let paths = &r.situations.as_ref().expect("sadržaj ima predloške").paths;
    assert_eq!(paths.kind, PeriodKind::Week);
    assert_eq!(paths.horizon, horizon(PeriodKind::Week));
    assert_eq!(paths.rows.len(), 3);
    let text = render_v2(&r);
    assert!(
        text.contains("== Situacije na zadanom planu (B3)"),
        "zaglavlje bloka B3"
    );
    for p in ["A", "B", "C"] {
        assert!(
            text.lines()
                .any(|l| l.trim_start().starts_with(&format!("{p} "))
                    && l.matches(" / ").count() == 5),
            "redak za profil {p} sa šest putova"
        );
        // Redak težine klika po profilu (PRAVILO-b): počinje profilom, spominje „klik“ i postotak (pct: „25,0 %“).
        assert!(
            text.lines()
                .any(|l| l.trim_start().starts_with(&format!("{p} "))
                    && l.contains("klik")
                    && l.contains(" %")),
            "redak težine klika za profil {p}"
        );
    }
    assert!(
        text.lines().any(|l| l
            .contains("najbolji / klik najbolji / srednji / prepušteno / šutnja / bez situacija")),
        "zaglavlje s klik najbolji"
    );
    assert!(text.contains("težina za anketu"));
    assert!(text.contains("prema rasponu plana"));
    for row in &paths.rows {
        assert_eq!(row.end.len(), 6);
        assert_eq!(row.steps.len(), 4);
        // Ograničenje samo snižava: klik najbolji nikad nije bolji od vlastitog odgovora.
        assert!(
            row.end[1] <= row.end[0],
            "{:?}: klik <= najbolji",
            row.profile
        );
        assert!(row.steps[0] >= 0, "{:?}: najbolji − klik", row.profile);
        assert!(
            row.click_vs_silent <= row.best_vs_silent[2],
            "{:?}: klik − šutnja <= najbolji − šutnja",
            row.profile
        );
    }
}

/// Sidro i primjerak: balans zna smije li se na situaciji pisati kao igra (`ai_entry_for(..).free_text`).
#[test]
fn writable_matches_game_free_text() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let (mut with_character, mut without_character) = (0_usize, 0_usize);
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let g = human_game(42, kind, Profile::B, AnswerPath::Silent);
        let me = g.hotels[0].setup.hotel_id;
        let mut check = |item: &RoundItem, at: &str| {
            let code = &item.template.code;
            let game_says = g.ai_entry_for(me, code).is_some_and(|a| a.free_text);
            assert_eq!(
                writable_v2(&content, item),
                game_says,
                "{kind:?} {at} {code}"
            );
            if game_says {
                with_character += 1;
            } else {
                without_character += 1;
            }
        };
        for t in &content.situations {
            let item = RoundItem {
                template: t,
                drawn: None,
            };
            check(&item, "sidro");
        }
        for i in g.hotels[0].drawn.values().flatten() {
            let item = RoundItem {
                template: &i.situation,
                drawn: Some(&i.drawn),
            };
            check(&item, "primjerak");
        }
    }
    assert!(
        with_character > 0,
        "usporedba nije prazna: ima situacija s likom"
    );
    assert!(
        without_character > 0,
        "usporedba nije prazna: ima situacija bez lika"
    );
}

/// Klik najbolji ne smije biti bolji od vlastitog odgovora (isti odgovor, kvaliteta `min(q, 4)` samo gdje se piše),
/// a negdje mora biti strogo lošiji: ograničenje se vidi u brojkama.
#[test]
fn click_best_is_never_above_best_and_sometimes_below() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let mut strictly_below = 0_usize;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in Profile::ALL {
            for seed in 0..5 {
                let best = path_season_v2(&content, profile, AnswerPath::Best, seed, kind).unwrap();
                let click =
                    path_season_v2(&content, profile, AnswerPath::ClickBest, seed, kind).unwrap();
                let at = format!("{kind:?} {} sjeme {seed}", profile.code());
                assert!(click.progress_pts <= best.progress_pts, "{at}");
                // Isti primjerci: put mijenja samo ocjenu, ne izvlačenje.
                assert_eq!(click.drawn, best.drawn, "{at}: isti primjerci");
                if click.progress_pts < best.progress_pts {
                    strictly_below += 1;
                }
            }
        }
    }
    assert!(
        strictly_below > 0,
        "ograničenje klika nigdje ne mijenja napredak"
    );
}
