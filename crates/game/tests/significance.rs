//! L1-13 dio 2 (značaj): igra primjenjuje faktor razine iz `weight` izvučenog primjerka (4 ključna, 3 važna, 1–2
//! svakodnevna), a sidra S1–S4 (bez `weight`) računa kao važna. Ključevi `economy.significance_bp` stavljeni su u
//! snimak (18 000 / 12 000 / 6 000; u repou ih nema). Dvije provjere nad istom sezonom:
//! (a) obračun upisan u dnevnik jednak je obračunu ponovno izračunatom iz ishoda s razinom iz `weight` primjerka
//! (ista „replika" kao u `situations_replay`; sidro = važna), za sve krugove tjedne i mjesečne igre;
//! (b) isti snimak s neutralnim ključevima (10 000) daje drukčije stanje hotela — faktor stvarno djeluje u igri.
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::dto::MonthReportDto;
use game::*;
use serde_json::json;
use sim_hotel::SituationOutcome;
use sim_hotel_v2::situations::{RoundSituation, Significance, reward_only};
use sim_hotel_v2::{PeriodKind, resolve_period};
use uuid::Uuid;

const SEEDS: [u64; 3] = [3, 11, 42];

fn snap(levels: (u16, u16, u16)) -> Snapshot {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let e = &mut v["v2"]["economy"];
    e["significance_bp"] = json!({ "key": levels.0, "important": levels.1, "everyday": levels.2 });
    Snapshot::from_value(&v).unwrap()
}

fn events(seed: u64, kind: PeriodKind, snap: &Snapshot) -> Vec<Recorded> {
    new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        snap,
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

/// Odigra sve krugove (hotel igrača ne odgovara ni na što, roboti odgovaraju).
fn played(seed: u64, kind: PeriodKind, snap: &Snapshot) -> Game {
    let mut g = Game::fold(&events(seed, kind, snap)).unwrap();
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

/// Obračun hotela u krugu iz ishoda s razinom iz `weight` primjerka (sidro bez primjerka: važna).
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
                None if h.delegated.contains(&(r, t.code.clone())) => SituationOutcome {
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
                fault: instance.is_some_and(|i| i.space_fault),
                click: h.answers.contains_key(&(r, t.code.clone()))
                    && g.click_capped(hotel, &t.code),
                significance: instance.map_or(Significance::Important, |i| {
                    Significance::from_weight(i.drawn.weight)
                }),
            }
        })
        .collect();
    let cat = g.content.v2.as_ref().unwrap();
    let period = g.round_info(r).unwrap().period;
    let report = resolve_period(&state, &period, &plan, &outcomes, cat).unwrap();
    MonthReportDto::from(&report.summary)
}

#[test] // 1
fn game_applies_significance_from_the_drawn_weight() {
    let keyed = snap((18_000, 12_000, 6_000));
    let mut seen = [false; 3];
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in SEEDS {
            let g = played(seed, kind, &keyed);
            assert!(!g.resolved.is_empty());
            for h in &g.hotels {
                for list in h.drawn.values().flatten() {
                    let level = Significance::from_weight(list.drawn.weight);
                    seen[LEVELS.iter().position(|l| *l == level).unwrap()] = true;
                }
            }
            for (&r, rr) in &g.resolved {
                for res in &rr.hotels {
                    assert_eq!(
                        recomputed(&g, res.hotel_id, r),
                        res.report,
                        "{kind:?} sjeme {seed} krug {r:?}: razina iz weight primjerka, sidro važna"
                    );
                }
            }
        }
    }
    assert_eq!(
        seen, [true; 3],
        "sezone sadrže ključne, važne i svakodnevne primjerke"
    );
}

const LEVELS: [Significance; 3] = [
    Significance::Key,
    Significance::Important,
    Significance::Everyday,
];

#[test] // 2
fn significance_keys_change_the_games_outcome() {
    let keyed = snap((18_000, 12_000, 6_000));
    let neutral = snap((10_000, 10_000, 10_000));
    let mut differs = 0;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in SEEDS {
            let (a, b) = (played(seed, kind, &keyed), played(seed, kind, &neutral));
            assert_eq!(a.rounds, b.rounds);
            for (r, ra) in &a.resolved {
                for (x, y) in ra.hotels.iter().zip(&b.resolved[r].hotels) {
                    if x.report != y.report {
                        differs += 1;
                    }
                }
            }
        }
    }
    assert!(differs > 0, "faktori razina moraju mijenjati obračun igre");
}
