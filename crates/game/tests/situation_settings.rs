//! B2a (ADR-042 t. 13–14): broj situacija po krugu je postavka igre zapisana u `GameCreated.situation_counts`, a ne
//! konstanta u kodu. Zadano: tjedno 3/4/5, mjesečno 4/5/6 (po očekivanoj popunjenosti kruga); dopušteno 3–8 i broj ne
//! pada s popunjenošću. Stari dnevnik bez polja čita se sa zadanim brojem za vrstu kruga.
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::situations::SituationCounts;
use uuid::Uuid;

/// Događaji nove igre sa sjemenom 42 iz zadanog snimka (v2 ili v1).
fn created_in(
    snap: &Snapshot,
    kind: PeriodKind,
    situations: Option<SituationCounts>,
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings { kind, situations },
    )
}

fn created(
    kind: PeriodKind,
    situations: Option<SituationCounts>,
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    created_in(&Snapshot::read_v2(&repo_dir()).unwrap(), kind, situations)
}

fn log_of(events: Vec<(Actor, GameEvent)>) -> Vec<Recorded> {
    events
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

fn written(events: &[(Actor, GameEvent)]) -> Option<SituationCountsDto> {
    let Some((_, GameEvent::GameCreated(c))) = events.first() else {
        panic!("prvi događaj nije GameCreated")
    };
    c.situation_counts
}

fn dto(low: u8, mid: u8, high: u8) -> Option<SituationCountsDto> {
    Some(SituationCountsDto { low, mid, high })
}

fn counts(low: u8, mid: u8, high: u8) -> SituationCounts {
    SituationCounts::new(low, mid, high).expect("dopuštena postavka")
}

#[test]
fn new_v2_game_writes_default_counts_for_its_round_kind() {
    let week = created(PeriodKind::Week, None).unwrap();
    assert_eq!(written(&week), dto(3, 4, 5));
    assert_eq!(
        Game::fold(&log_of(week)).unwrap().situation_counts(),
        SituationCounts::WEEK
    );

    let month = created(PeriodKind::Month, None).unwrap();
    assert_eq!(written(&month), dto(4, 5, 6));
    assert_eq!(
        Game::fold(&log_of(month)).unwrap().situation_counts(),
        SituationCounts::MONTH
    );

    // Izričita postavka se zapisuje kakva jest, za obje vrste kruga.
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let own = created(kind, Some(counts(5, 7, 8))).unwrap();
        assert_eq!(written(&own), dto(5, 7, 8));
        assert_eq!(
            Game::fold(&log_of(own)).unwrap().situation_counts(),
            counts(5, 7, 8)
        );
    }

    // v1 igra polje nikad ne piše.
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    let old = created_in(&v1, PeriodKind::Month, None).unwrap();
    assert_eq!(written(&old), None);

    // v1 igra s izričitim brojem situacija je odbijena.
    assert_eq!(
        created_in(&v1, PeriodKind::Month, Some(SituationCounts::MONTH)).err(),
        Some(GameError::InvalidSettings)
    );
}

#[test]
fn counts_outside_three_to_eight_or_not_rising_are_refused() {
    for (low, mid, high) in [(2, 4, 5), (3, 4, 9), (3, 5, 4), (6, 5, 6)] {
        assert_eq!(
            SituationCounts::new(low, mid, high),
            None,
            "{low}/{mid}/{high}"
        );
    }
    for (low, mid, high) in [(3, 3, 3), (8, 8, 8), (3, 4, 5), (4, 5, 6)] {
        let c = SituationCounts::new(low, mid, high)
            .unwrap_or_else(|| panic!("{low}/{mid}/{high} je dopušteno"));
        assert_eq!(c.levels(), (low, mid, high));
    }
    assert_eq!(
        usize::from(SituationCounts::MIN),
        MANDATORY_SITUATIONS,
        "najmanje toliko koliko je obveznih situacija"
    );
    assert_eq!(SituationCounts::MAX, 8);
    assert_eq!(SituationCounts::WEEK.levels(), (3, 4, 5));
    assert_eq!(SituationCounts::MONTH.levels(), (4, 5, 6));
    // Pragovi 75 / 90 % ostaju u kodu.
    for (counts, want) in [
        (SituationCounts::WEEK, [3, 3, 4, 4, 5, 5]),
        (SituationCounts::MONTH, [4, 4, 5, 5, 6, 6]),
    ] {
        let got = [0, 7_499, 7_500, 8_999, 9_000, 10_000].map(|occ| counts.for_occupancy(occ));
        assert_eq!(got, want);
    }
    assert_eq!(
        SituationCounts::default_for(PeriodKind::Week),
        SituationCounts::WEEK
    );
    assert_eq!(
        SituationCounts::default_for(PeriodKind::Month),
        SituationCounts::MONTH
    );
}

#[test]
fn log_without_counts_still_reads_with_defaults() {
    for (kind, want) in [
        (PeriodKind::Week, SituationCounts::WEEK),
        (PeriodKind::Month, SituationCounts::MONTH),
    ] {
        let mut log: Vec<Recorded> = created(kind, None)
            .unwrap()
            .into_iter()
            .zip(1..)
            .map(|((actor, event), seq)| Recorded { seq, actor, event })
            .collect();
        let mut v = log[0].event.to_payload();
        v.as_object_mut()
            .unwrap()
            .remove("situation_counts")
            .expect("nova igra piše broj");
        log[0].event = GameEvent::from_record("GameCreated", SCHEMA_VER, v).unwrap();
        assert_eq!(Game::fold(&log).unwrap().situation_counts(), want);
    }
}

#[test]
fn corrupt_counts_in_log_are_refused() {
    // Ispravan zapis prolazi; neispravni (izvan 3–8 ili padajući) daju Corrupt. `new_game_with` ih ne može stvoriti,
    // jer `SituationCounts::new` vraća `None`, pa se kvare ručno u dnevniku.
    for (low, mid, high, ok) in [
        (3, 4, 5, true),
        (3, 9, 9, false),
        (2, 4, 5, false),
        (3, 5, 4, false),
        (0, 0, 0, false),
    ] {
        let mut log = log_of(created(PeriodKind::Week, None).unwrap());
        let GameEvent::GameCreated(c) = &mut log[0].event else {
            panic!("prvi događaj nije GameCreated")
        };
        c.situation_counts = dto(low, mid, high);
        let folded = Game::fold(&log);
        if ok {
            assert!(folded.is_ok(), "{low}/{mid}/{high}");
        } else {
            assert!(
                matches!(folded, Err(GameError::Corrupt(_))),
                "{low}/{mid}/{high}: {:?}",
                folded.map(|_| ())
            );
        }
    }
}

#[test]
fn monthly_game_with_three_has_three_in_every_round() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let g = common::v2_game_with(
        &snap,
        42,
        GameSettings {
            kind: PeriodKind::Month,
            situations: SituationCounts::new(3, 3, 3),
        },
    );
    assert_eq!(g.situation_counts(), counts(3, 3, 3));
    let g = common::play_through(g, "sep");
    let me = common::hotel_ids()[0];
    assert!(!g.rounds.is_empty());
    for r in g.rounds.clone() {
        assert_eq!(
            g.situations_of_hotel(me, r).unwrap().len(),
            3,
            "{}",
            r.code()
        );
    }
}
