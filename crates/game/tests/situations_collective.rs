//! G-1 popravak 1 (igra): I2 — kolektivna situacija je jednom po krugu za cijelu igru: kod svih hotela ili ni kod
//! jednog, i nijedan hotel nikad nema više situacija od broja kruga ni više ponuđenih grupa od `groups_per_season`
//! (sidra uključena); I1 — popis situacija svakog hotela je sidra pa primjerci stabilno po važnosti (`weight`)
//! silazno, bez obzira na utor hotela i na mjesto kolektivne u događaju.
//!
//! I2 koristi SINTETSKI snimak (REGATA, jedina kolektivna s grupom, dolazi u lipnju/srpnju/kolovozu, grupa za rujan
//! kao S3 i hotelske grupe, bez granice grupa po mjesecu): u stvarnom sadržaju kršenje nije dostižno (S3 nudi grupu za
//! rujan, REGATA je samo u lipnju kad je povijest prazna). I1 koristi stvaran sadržaj.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto: `situations_of`, `collective_of`):
//! * popis situacija hotela je `Game::situations_of_hotel(hotel, krug)`;
//! * kolektivni primjerak hotela prepoznaje se po `HotelRecord::drawn[&krug]` stavkama s `drawn.slot == 0`;
//! * događaj `SituationsDrawn` ostaje s `situations: Vec<DrawnItem>` (polja `code`, `slot`, `weight`) redom događaja.

mod common;

use std::collections::BTreeSet;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::event::DrawnItem;
use game::*;
use serde_json::{Value, json};
use sim_hotel_v2::situations::situation_count;
use sim_hotel_v2::{PeriodKind, group_month};
use uuid::Uuid;

const SEEDS: u64 = 25;

fn arrive_in_september(v: &mut Value) {
    match v {
        Value::Object(m) => {
            if m.get("type").is_some_and(|t| t == "GroupBooking") {
                m.insert("month".to_owned(), json!("sep"));
            }
            m.values_mut().for_each(arrive_in_september);
        }
        Value::Array(a) => a.iter_mut().for_each(arrive_in_september),
        _ => {}
    }
}

/// Stvaran snimak s jednom čestom kolektivnom (REGATA lipanj–kolovoz, grupa za rujan), bez granice po mjesecu.
fn synthetic_snapshot() -> Snapshot {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut v: Value = serde_json::to_value(&snap).unwrap();
    let tf = v["v2"]["situation_templates"].as_object_mut().unwrap();
    tf.remove("groups_per_month");
    let list = tf["templates"].as_array_mut().unwrap();
    list.retain(|t| t["scope"] != "svi" || t["code"] == "P-REGATA");
    for t in list.iter_mut().filter(|t| t["code"] == "P-REGATA") {
        let o = t.as_object_mut().unwrap();
        o.insert("when".to_owned(), json!({"months": ["jun", "jul", "aug"]}));
        o.insert(
            "repeat".to_owned(),
            json!({"max_per_season": 12, "gap_rounds": 4}),
        );
        o.insert("chance".to_owned(), json!({"base": 5}));
        arrive_in_september(t);
    }
    serde_json::from_value(v).unwrap()
}

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

/// Odigra sve krugove (hotel igrača ne odgovara ni na što).
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

fn played(snap: &Snapshot, seed: u64, kind: PeriodKind) -> Game {
    play_all(Game::fold(&events_of(snap, seed, kind)).unwrap())
}

/// Popis situacija hotela u krugu (jedino mjesto pretpostavke o sučelju).
fn situations_of(g: &Game, hotel: HotelId, r: RoundKey) -> Vec<&sim_hotel::SituationTemplate> {
    g.situations_of_hotel(hotel, r).unwrap()
}

/// Kolektivni primjerci koje hotel ima u krugu (jedino mjesto pretpostavke o prikazu).
fn collective_of(
    g: &Game,
    hotel: HotelId,
    r: RoundKey,
) -> Vec<sim_hotel_v2::situations::DrawnSituation> {
    g.hotel(hotel)
        .unwrap()
        .drawn
        .get(&r)
        .map(|l| {
            l.iter()
                .filter(|i| i.drawn.slot == 0)
                .map(|i| i.drawn.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Nijedan hotel nikad nema više situacija od broja kruga ni više ponuđenih grupa u sezoni od granice (sidra i
/// kolektivna uključene).
#[test]
fn no_hotel_goes_over_the_round_count_or_the_group_limit() {
    let snap = synthetic_snapshot();
    let mut games = 0;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in 1..=SEEDS {
            let g = played(&snap, seed, kind);
            games += 1;
            let tpl = g.content.situation_templates.as_ref().unwrap();
            let limit = usize::from(tpl.groups_per_season.expect("granica grupa"));
            let cat = g.content.v2.as_ref().unwrap();
            for h in &g.hotels {
                let id = h.setup.hotel_id;
                let mut groups = 0;
                for &r in &g.rounds {
                    let list = situations_of(&g, id, r);
                    let CoreState::V2(state) = g.state_at(id, r).unwrap() else {
                        panic!("v2 stanje");
                    };
                    let period = g.round_info(r).unwrap().period;
                    let n = usize::from(situation_count(&state, &period, cat).unwrap());
                    assert!(
                        list.len() <= n,
                        "{kind:?} sjeme {seed}: hotel utora {} u krugu {r:?} ima {} situacija, krug ima {n}",
                        h.setup.slot,
                        list.len()
                    );
                    groups += list.iter().filter(|t| group_month(t).is_some()).count();
                }
                assert!(
                    groups <= limit,
                    "{kind:?} sjeme {seed}: hotel utora {} ima {groups} ponuđenih grupa u sezoni, granica {limit}",
                    h.setup.slot
                );
            }
        }
    }
    assert_eq!(games, 2 * SEEDS);
}

/// Kolektivna (utor 0) u krugu je kod svih hotela (ista) ili ni kod jednog.
#[test]
fn collective_is_at_every_hotel_or_at_none() {
    let snap = synthetic_snapshot();
    let mut rounds_with = 0;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in 1..=SEEDS {
            let g = played(&snap, seed, kind);
            for &r in &g.rounds {
                let per_hotel: Vec<_> = g
                    .hotels
                    .iter()
                    .map(|h| collective_of(&g, h.setup.hotel_id, r))
                    .collect();
                assert!(per_hotel[0].len() <= 1, "najviše jedna kolektivna po krugu");
                for (h, c) in g.hotels.iter().zip(&per_hotel) {
                    assert_eq!(
                        c, &per_hotel[0],
                        "{kind:?} sjeme {seed} krug {r:?}: kolektivna hotela utora {} nije ista kao hotela utora 1",
                        h.setup.slot
                    );
                }
                rounds_with += usize::from(!per_hotel[0].is_empty());
            }
        }
    }
    assert!(rounds_with > 0, "kontrola: kolektivna nikad izvučena");
}

// ---------------------------------------------------------------------------------------------------------------
// I1: poredak po važnosti

fn items_of(log: &[Recorded], r: RoundKey) -> Vec<DrawnItem> {
    log.iter()
        .filter_map(|x| match &x.event {
            GameEvent::SituationsDrawn(d) if d.month == r => Some(d.situations.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

/// Popis svakog hotela (utori 1–4) je sidra, pa primjerci stabilno po `weight` silazno (jednaki redom događaja).
/// Kolektivna manje važnosti od vlastitih primjeraka hotela utora 2+ mora stajati IZA važnijih.
#[test]
fn every_hotels_list_is_anchors_then_instances_by_weight_descending() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut checked_collective_behind = 0;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in 1..=SEEDS {
            let g = played(&snap, seed, kind);
            for h in &g.hotels {
                let id = h.setup.hotel_id;
                for &r in &g.rounds {
                    let items = items_of(&g.history, r);
                    let mine: Vec<&DrawnItem> = items
                        .iter()
                        .filter(|i| i.slot == 0 || i.slot == h.setup.slot)
                        .collect();
                    let instance_codes: BTreeSet<&str> =
                        mine.iter().map(|i| i.code.as_str()).collect();
                    let list: Vec<String> = situations_of(&g, id, r)
                        .iter()
                        .map(|t| t.code.clone())
                        .collect();
                    let anchors = list
                        .iter()
                        .take_while(|c| !instance_codes.contains(c.as_str()))
                        .count();
                    // Sidra su prefiks i nijedno nije primjerak.
                    assert!(
                        list.iter()
                            .skip(anchors)
                            .all(|c| instance_codes.contains(c.as_str())),
                        "{kind:?} sjeme {seed} utor {} krug {r:?}: sidra moraju biti ispred primjeraka: {list:?}",
                        h.setup.slot
                    );
                    // Isti skup primjeraka, težine silazno (kolektivna nije povlaštena ni na jednakoj težini), a vlastiti
                    // primjerci jednake težine ostaju redom događaja (stabilno).
                    let weight_of =
                        |c: &str| mine.iter().find(|i| i.code == c).map(|i| i.weight).unwrap();
                    let got: Vec<&str> = list.iter().skip(anchors).map(String::as_str).collect();
                    let mut sorted_got = got.clone();
                    sorted_got.sort_unstable();
                    let mut want_set: Vec<&str> = mine.iter().map(|i| i.code.as_str()).collect();
                    want_set.sort_unstable();
                    assert_eq!(
                        sorted_got, want_set,
                        "{kind:?} sjeme {seed}: isti primjerci"
                    );
                    assert!(
                        got.windows(2).all(|w| weight_of(w[0]) >= weight_of(w[1])),
                        "{kind:?} sjeme {seed} utor {} krug {r:?}: primjerci nisu po važnosti silazno: {:?}",
                        h.setup.slot,
                        got.iter().map(|c| (*c, weight_of(c))).collect::<Vec<_>>()
                    );
                    let own_event: Vec<&str> = mine
                        .iter()
                        .filter(|i| i.slot != 0)
                        .map(|i| i.code.as_str())
                        .collect();
                    for w in own_event.windows(2) {
                        let (a, b) = (w[0], w[1]);
                        if weight_of(a) == weight_of(b) {
                            let pos = |c: &str| got.iter().position(|x| *x == c).unwrap();
                            assert!(
                                pos(a) < pos(b),
                                "{kind:?} sjeme {seed} utor {} krug {r:?}: jednaki po važnosti moraju ostati redom događaja ({a} prije {b})",
                                h.setup.slot
                            );
                        }
                    }
                    // Kontrola da test grize: kolektivna s manjom važnosti od nekog vlastitog primjerka.
                    if let Some(c) = mine.iter().find(|i| i.slot == 0)
                        && mine.iter().any(|i| i.slot != 0 && i.weight > c.weight)
                    {
                        checked_collective_behind += 1;
                    }
                }
            }
        }
    }
    assert!(
        checked_collective_behind > 0,
        "kontrola: nijedan slučaj kolektivne manje važnosti od vlastitih primjeraka"
    );
}
