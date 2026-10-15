//! G-1 popravak 1, M6 (rupe u testovima): (a) izvlačenje kruga k > 1 nad igrom složenom iz spremljenog dnevnika (fold
//! prefiksa do prije događaja) daje isti `SituationsDrawn` kao u dnevniku — izvlačenje iz spremljene igre, ne samo
//! prvi krug; (b) obračun upisan u dnevnik za krug jednak je obračunu ponovno izračunatom iz stanja, plana i ishoda
//! koje igra čita iz dnevnika (to je ulaz izvlačenja idućeg kruga, `Game::period_report`).
//!
//! NAPOMENA (b): `Game::period_report` je `pub(crate)`, pa ga integracijski test ne može zvati; (b) ga ponavlja kroz
//! javne dijelove (`state_at`, `effective_plan`, `situations_of_hotel`, odgovori hotela, `resolve_period`) — ako G-1
//! popravak promijeni poredak ili sastav popisa situacija između obračuna i ponovnog računa, test pada. Ako graditelj
//! `period_report` učini javnim, (b) se može skratiti na izravnu usporedbu (jedino mjesto: `recomputed`).

mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::dto::MonthReportDto;
use game::*;
use sim_hotel::SituationOutcome;
use sim_hotel_v2::situations::{RoundSituation, reward_only};
use sim_hotel_v2::{PeriodKind, resolve_period};
use uuid::Uuid;

const SEEDS: [u64; 3] = [3, 11, 42];

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
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect()
}

/// Odigra sve krugove (hotel igrača ne odgovara ni na što, roboti odgovaraju).
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

fn played(seed: u64, kind: PeriodKind) -> Game {
    play_all(Game::fold(&v2_events(seed, kind)).unwrap())
}

#[test]
fn redrawing_a_later_round_from_the_saved_log_gives_the_recorded_event() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in SEEDS {
            let g = played(seed, kind);
            let log = &g.history;
            let drawn_at: Vec<usize> = log
                .iter()
                .enumerate()
                .filter(|(_, r)| r.event.kind() == "SituationsDrawn")
                .map(|(i, _)| i)
                .collect();
            assert!(
                drawn_at.len() > 2,
                "{kind:?} sjeme {seed}: očekivano izvlačenje u više krugova"
            );
            // Svaki krug poslije prvog: prefiks dnevnika do prije događaja pa izvlačenje tog kruga.
            for &i in drawn_at.iter().skip(1) {
                let GameEvent::SituationsDrawn(recorded) = &log[i].event else {
                    unreachable!()
                };
                let before = Game::fold(&log[..i]).unwrap();
                let again = before
                    .draw_round(recorded.month)
                    .unwrap_or_else(|e| {
                        panic!(
                            "{kind:?} sjeme {seed} krug {:?}: izvlačenje {e:?}",
                            recorded.month
                        )
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "{kind:?} sjeme {seed} krug {:?}: igra iz dnevnika ne izvlači",
                            recorded.month
                        )
                    });
                assert_eq!(
                    again.to_payload(),
                    log[i].event.to_payload(),
                    "{kind:?} sjeme {seed} krug {:?}: izvlačenje iz spremljene igre nije kao u dnevniku",
                    recorded.month
                );
            }
        }
    }
}

/// Obračun hotela u krugu ponovno izračunat iz stanja, plana i ishoda koje igra čita iz dnevnika (jedino mjesto
/// pretpostavke o sučelju: bez `Game::period_report`, vidi napomenu).
fn recomputed(g: &Game, hotel: HotelId, r: RoundKey) -> MonthReportDto {
    let CoreState::V2(state) = g.state_at(hotel, r).unwrap() else {
        panic!("v2 stanje");
    };
    let CorePlan::V2(plan) = g.effective_plan(hotel, r).unwrap() else {
        panic!("v2 plan");
    };
    let h = g.hotel(hotel).unwrap();
    let outcomes: Vec<RoundSituation> = g
        .situations_of_hotel(hotel, r)
        .unwrap()
        .into_iter()
        .map(|t| {
            let o = match h.answers.get(&(r, t.code.clone())) {
                Some(&i) => SituationOutcome::from_answer(&t.code, &t.answers[usize::from(i)]),
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
    MonthReportDto::from(&report.summary)
}

#[test]
fn recomputed_report_matches_the_recorded_resolution() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in SEEDS {
            let g = played(seed, kind);
            assert!(!g.resolved.is_empty());
            for (&r, rr) in &g.resolved {
                for res in &rr.hotels {
                    assert_eq!(
                        recomputed(&g, res.hotel_id, r),
                        res.report,
                        "{kind:?} sjeme {seed} krug {r:?}: ponovni obračun nije kao upisani"
                    );
                }
            }
        }
    }
}
