//! A4 (revizija temelja #8): balans mjeri istu petlju kao igra. Igra s jednim robot-hotelom u utoru balansa (bez
//! čovjeka i AI-ja) i sezona robota iz balansa (`robot_season_v2`) za isto sjeme daju iste situacije, isto stanje na
//! početku svakog kruga, iste pokazatelje i isti napredak. Nasumični robot nije u igri i troši drugu kocku.
use balance::v2::{HOTEL_SLOT, robot_season_v2};
use content::{Snapshot, repo_dir};
use game::command::{GameSettings, ROBOTS, new_game_with};
use game::dto::indices_dto;
use game::{
    Actor, Controller, CoreState, Game, GameEvent, GameId, GuestId, HotelId, Recorded, ResolveStep,
};
use sim_hotel_v2::situations::SituationCounts;
use sim_hotel_v2::{
    PeriodKind, Profile, SeasonResult, SituationTemplate, Strategy, situations_for,
};
use uuid::Uuid;

const SEEDS: [u64; 2] = [3, 42];
/// Popis robota je jedan izvor: onaj iz igre (B2b), ne ručna kopija.
fn game_robots() -> [Strategy; 3] {
    ROBOTS.map(|(s, _)| s)
}

/// Vlastiti broj situacija po vrsti kruga uz zadani (`None`): tjedno 5/7/8, mjesečno 3/3/3.
fn own_counts(kind: PeriodKind) -> SituationCounts {
    match kind {
        PeriodKind::Week => SituationCounts::new(5, 7, 8),
        PeriodKind::Month => SituationCounts::new(3, 3, 3),
    }
    .unwrap()
}
/// Po krugu: šifre situacija (sortirane), pet pokazatelja nakon kruga, napredak od početka (stotinke).
type Trace = Vec<(Vec<String>, [u16; 5], i32)>;

fn apply(g: &mut Game, actor: Actor, event: GameEvent) {
    let seq = g.last_seq + 1;
    g.apply(Recorded { seq, actor, event }).unwrap();
}

/// v2 igra s jednim robot-hotelom (utor `HOTEL_SLOT`), odigrana do kraja sezone.
fn robot_game(
    seed: u64,
    kind: PeriodKind,
    profile: Profile,
    strategy: Strategy,
    situations: Option<SituationCounts>,
) -> Game {
    let ids = [1_u128, 2, 3, 4].map(|n| HotelId(Uuid::from_u128(n)));
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let events = new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        &snap,
        GuestId(Uuid::from_u128(99)),
        ids,
        GameSettings { kind, situations },
    )
    .unwrap();
    let Some((actor, GameEvent::GameCreated(mut created))) = events.into_iter().next() else {
        panic!("prvi događaj nije GameCreated")
    };
    let mut hotel = created.hotels.remove(0);
    hotel.slot = HOTEL_SLOT;
    hotel.profile = profile.code().into();
    hotel.controller = Controller::Robot {
        strategy: strategy.code().into(),
    };
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
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        apply(&mut g, Actor::System, ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("robot ne čeka presude")
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

#[test]
fn robot_hotel_in_game_matches_balance_season() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for situations in [None, Some(own_counts(kind))] {
            // Svoj broj se vrti samo sa sjemenom 42 (36 + 18 kombinacija).
            let seeds: &[u64] = if situations.is_some() { &[42] } else { &SEEDS };
            for profile in Profile::ALL {
                for strategy in game_robots() {
                    for &seed in seeds {
                        let at = format!(
                            "{kind:?} {situations:?} {} {} sjeme {seed}",
                            profile.code(),
                            strategy.code()
                        );
                        let g = robot_game(seed, kind, profile, strategy, situations);
                        let s =
                            robot_season_v2(&content, profile, strategy, seed, kind, situations)
                                .unwrap();
                        assert_eq!(g.rounds.len(), s.periods.len(), "{at}");
                        for (r, (rep, _)) in g.rounds.iter().zip(&s.periods) {
                            let CoreState::V2(st) =
                                g.state_at(g.hotels[0].setup.hotel_id, *r).unwrap()
                            else {
                                panic!("{at}: igra nije v2")
                            };
                            assert_eq!(*st, rep.before, "{at}: stanje na početku kruga {r:?}");
                        }
                        let gt = game_trace(&g);
                        assert_eq!(gt, season_trace(&s, &content.situations), "{at}");
                        assert_eq!(
                            gt.last().unwrap().2,
                            s.progress_pts,
                            "{at}: napredak sezone"
                        );
                        assert!(
                            s.drawn.iter().any(|d| !d.is_empty()),
                            "{at}: balans bez izvučenih situacija"
                        );
                        assert!(
                            gt.iter()
                                .any(|(codes, _, _)| codes.iter().any(|c| c.contains('.'))),
                            "{at}: igra samo sa sidrima"
                        );
                    }
                }
            }
        }
    }
}

/// Usporedba nije prazna: drugo sjeme u balansu daje drugi trag.
#[test]
fn parity_check_sees_a_different_seed() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let g = robot_game(42, PeriodKind::Week, Profile::B, Strategy::Caring, None);
    let other = robot_season_v2(
        &content,
        Profile::B,
        Strategy::Caring,
        43,
        PeriodKind::Week,
        None,
    )
    .unwrap();
    assert_ne!(game_trace(&g), season_trace(&other, &content.situations));
}
