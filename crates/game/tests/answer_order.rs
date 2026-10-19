//! L1-13b dio 1 (ADR-042 t. 10, kartica task-15): redoslijed ponuđenih odgovora na ekranu je izmiješan iz sjemena igre,
//! slota hotela i šifre primjerka. Čista funkcija `game::answer_order(seed, slot, code, n) -> Vec<u8>`.
//!
//! SUČELJE (dogovor čuvara i graditelja; graditelj ga ne mijenja):
//! - `pub fn answer_order(seed: u64, slot: u8, code: &str, n: usize) -> Vec<u8>` u `game`, izvezena iz `game::*`;
//! - `order[prikazni_položaj] = izvorni_broj` (izvorni broj = položaj u sadržaju, onaj što ga pamte događaji);
//! - rezultat je potpuna permutacija `0..n` (n = 0 daje prazan popis, n = 1 daje `[0]`);
//! - izvedeno iz `Rng::derive(seed, [label("game:answer-order"), slot, label(code)])` i Fisher–Yatesa; ne troši
//!   tokove igre. Pravilo se ne mijenja bez dizanja verzije (inače bi klik usred kruga zapisao drugi odgovor).

use std::collections::BTreeSet;

use game::answer_order;

fn code(round: u32) -> String {
    format!("P-KLIMA.t{round:02}")
}

#[test]
fn order_is_a_complete_permutation() {
    for n in 1..=8 {
        for seed in [0, 1, 42, u64::MAX] {
            for slot in 1..=4 {
                for round in 1..=12 {
                    let o = answer_order(seed, slot, &code(round), n);
                    assert_eq!(o.len(), n, "n={n} seed={seed} slot={slot}");
                    let mut sorted = o.clone();
                    sorted.sort_unstable();
                    let expected: Vec<u8> = (0..n).map(|i| u8::try_from(i).unwrap()).collect();
                    assert_eq!(sorted, expected, "svaki broj točno jednom: {o:?}");
                }
            }
        }
    }
}

#[test]
fn degenerate_sizes_are_identity() {
    assert!(answer_order(42, 1, "P-KLIMA.t03", 0).is_empty());
    assert_eq!(answer_order(42, 1, "P-KLIMA.t03", 1), vec![0]);
}

#[test]
fn order_is_deterministic_and_independent_of_call_order() {
    let codes: Vec<String> = (1..=20).map(code).collect();
    let forward: Vec<Vec<u8>> = codes.iter().map(|c| answer_order(42, 1, c, 5)).collect();
    // Obrnuti redoslijed poziva, s drugim pozivima između: izvedeni tok ne smije imati zajedničko stanje.
    let mut backward: Vec<Vec<u8>> = Vec::new();
    for c in codes.iter().rev() {
        let _noise = answer_order(7, 3, "P-DRUGO.t09", 4);
        backward.push(answer_order(42, 1, c, 5));
    }
    backward.reverse();
    assert_eq!(forward, backward);
    assert_eq!(
        answer_order(42, 1, "P-KLIMA.t03", 5),
        answer_order(42, 1, "P-KLIMA.t03", 5)
    );
}

#[test]
fn order_differs_between_instances_of_the_same_template() {
    // Isti predložak u drugom krugu dobije novi redoslijed (student ne smije naučiti „za klimu je treći").
    let perms: BTreeSet<Vec<u8>> = (1..=30).map(|r| answer_order(42, 1, &code(r), 5)).collect();
    assert!(
        perms.len() >= 10,
        "30 primjeraka predloška dalo je samo {} različitih redoslijeda",
        perms.len()
    );
    let any_pair_differs =
        (1..30).any(|r| answer_order(42, 1, &code(r), 5) != answer_order(42, 1, &code(r + 1), 5));
    assert!(
        any_pair_differs,
        "P-KLIMA.tNN i P-KLIMA.tNN+1 nikad se ne razlikuju"
    );
}

#[test]
fn order_differs_between_slots_of_the_same_game() {
    // Različit među kolegama: isto sjeme i šifra, drugi hotel.
    let differing = (1..=30)
        .filter(|&r| answer_order(42, 1, &code(r), 5) != answer_order(42, 2, &code(r), 5))
        .count();
    assert!(
        differing >= 15,
        "slot 1 i 2 razlikuju se u samo {differing} od 30"
    );
    let perms: BTreeSet<Vec<u8>> = (1..=4)
        .map(|slot| answer_order(42, slot, "P-KLIMA.t03", 5))
        .collect();
    assert!(perms.len() >= 2, "četiri slota dala isti redoslijed");
}

#[test]
fn order_differs_between_seeds() {
    let differing = (1..=30)
        .filter(|&r| answer_order(42, 1, &code(r), 5) != answer_order(43, 1, &code(r), 5))
        .count();
    assert!(
        differing >= 15,
        "sjeme 42 i 43 razlikuju se u samo {differing} od 30"
    );
}

#[test]
fn best_original_answer_lands_on_every_position() {
    // Izvorni broj 0 je u sadržaju najbolji (ADR-042); na ekranu mora biti na svakom položaju, približno jednako.
    let n = 5;
    let mut at = [0_u32; 5];
    for seed in 1..=5 {
        for slot in 1..=3 {
            for r in 1..=10 {
                let o = answer_order(seed, slot, &code(r), n);
                let pos = o.iter().position(|&x| x == 0).unwrap();
                at[pos] += 1;
            }
        }
    }
    // 150 uzoraka, očekivano 30 po položaju.
    for (pos, count) in at.iter().enumerate() {
        assert!(
            *count >= 10,
            "izvorni 0 na položaju {pos} samo {count} puta od 150: {at:?}"
        );
    }
}

#[test]
fn order_is_not_the_identity_in_general() {
    let identity: Vec<u8> = (0..5).collect();
    let non_identity = (1..=30)
        .filter(|&r| answer_order(42, 1, &code(r), 5) != identity)
        .count();
    assert!(
        non_identity >= 25,
        "premalo izmiješanih: {non_identity} od 30"
    );
}
