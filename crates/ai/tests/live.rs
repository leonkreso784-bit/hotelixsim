//! Živi test: zove pravi Claude API. Ručno: `cargo test -p ai --test live -- --ignored`
//! (treba ANTHROPIC_API_KEY). U CI-ju se ne pokreće.

use app::AiPort;
use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::*;
use uuid::Uuid;

#[tokio::test]
#[ignore = "zove pravi Claude API; treba ANTHROPIC_API_KEY"]
async fn judge_verdict_for_s1_passes_validation() {
    let key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY nije postavljen");
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let hotels = [11, 12, 13, 14].map(|n| HotelId(Uuid::from_u128(n)));
    let guest = GuestId(Uuid::from_u128(99));
    let events: Vec<Recorded> = new_game(GameId(Uuid::from_u128(1)), 42, &snap, guest, hotels)
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect();
    let mut g = Game::fold(&events).unwrap();
    let ev = g
        .submit_response(
            hotels[0],
            "S1",
            "Ispričat ću se, ponuditi zamjenu sobe i poslati tehničara.",
        )
        .unwrap();
    let seq = g.last_seq + 1;
    g.apply(Recorded {
        seq,
        actor: Actor::Guest(guest),
        event: ev,
    })
    .unwrap();
    let input = g.assess_input(hotels[0], "S1").unwrap();

    let ai = ai::Anthropic::new(key, ai::Models::default());
    let out = ai.assess(input).await.expect("presuda");
    assert_eq!(out.meta.prompt, "presuda.v2");
    assert!(g.validate_verdict("S1", out.value).is_ok());
}
