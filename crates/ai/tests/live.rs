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
    assert_eq!(out.meta.prompt, "presuda.v3");
    assert!(g.validate_verdict("S1", out.value).is_ok());
}

/// Lik govori samo svoje riječi: bez opisa radnji (`*…*`), bez svog imena na početku i bez izlaska iz uloge
/// (Leon 12.10.: „Marko, gledam ga pravo…" — lik.v1 je miješao pripovijedanje i ulogu upravitelja).
#[tokio::test]
#[ignore = "zove pravi Claude API; treba ANTHROPIC_API_KEY"]
async fn character_speaks_only_its_own_words() {
    let key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY nije postavljen");
    let ai = ai::Anthropic::new(key, ai::Models::default());
    let input = CharacterInput {
        situation: "S1".into(),
        title: "Šef kuhinje traži povišicu".into(),
        character: CharacterCard {
            name: "Marko Babić".into(),
            role: "šef kuhinje, u hotelu sedam sezona".into(),
            brief: "Konkurentski hotel nudi ti 20 % veću plaću. Pravi razlog je da radiš 12 sati dnevno bez \
                    slobodnog dana i da te nitko nije pitao za mišljenje o jelovniku."
                .into(),
            opening: "Šefe, imam ponudu iz hotela preko puta, 20 % više. Ili mi dignete plaću ili odlazim na kraju \
                      mjeseca."
                .into(),
        },
        transcript: vec![ChatLine {
            from: Speaker::Student,
            text: "Žao mi je to čuti, koje su vaše želje i preporuke.".into(),
        }],
    };
    for _ in 0..3 {
        let out = ai.character(input.clone()).await.expect("odgovor lika");
        assert_eq!(out.meta.prompt, "lik.v2");
        let t = out.value.trim().to_lowercase();
        assert!(!t.contains('*'), "opis radnje: {t}");
        assert!(!t.starts_with("marko"), "ime na početku: {t}");
        assert!(
            !t.contains("glumim") && !t.contains("kao lik"),
            "izlazak iz uloge: {t}"
        );
    }
}
