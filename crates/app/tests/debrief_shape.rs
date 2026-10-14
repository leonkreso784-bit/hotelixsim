use std::sync::Arc;

use app::{AiError, AiOut, AiPort, BoxFut};
use content::{Snapshot, repo_dir};
use game::RoundKey;
use sim_hotel::Month;

/// Mentor kruga koji vraća tekst bez četiri dijela; ostale uloge su isključene.
struct PartlessMentor;

impl AiPort for PartlessMentor {
    fn assess(
        &self,
        _input: game::AssessInput,
    ) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, AiError>> {
        Box::pin(async { Err(AiError::Disabled) })
    }
    fn character(
        &self,
        _input: game::CharacterInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async { Err(AiError::Disabled) })
    }
    fn debrief(&self, _input: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async {
            Ok(AiOut {
                value: "Mentor: dobar mjesec.".into(),
                meta: game::AiMeta {
                    model: "bez-dijelova".into(),
                    prompt: "mentor.v4".into(),
                    input_tokens: 1,
                    output_tokens: 1,
                },
            })
        })
    }
    fn season_debrief(
        &self,
        _input: game::SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async { Err(AiError::Disabled) })
    }
}

#[tokio::test]
async fn round_mentor_text_without_four_parts_is_replaced_by_fallback() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let (deps, mem) = app::memory::deps_with_ai(snap, Arc::new(PartlessMentor));
    let d = app::start_demo(&deps, None, app::NewGameSettings::default())
        .await
        .unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    app::close_round(&deps, &caller, d.game_id, RoundKey::Month(Month::Jun))
        .await
        .unwrap();
    while app::run_next_job(&deps).await.unwrap() {}
    let e = mem
        .events(d.game_id)
        .into_iter()
        .find(|e| e.kind == "DebriefRecorded")
        .expect("DebriefRecorded");
    assert_eq!(e.payload["source"], "fallback", "{}", e.payload);
    assert!(e.payload.get("meta").is_none(), "{}", e.payload);
    let text = e.payload["text"].as_str().unwrap();
    assert!(game::debrief_has_parts(text), "{text}");
    assert!(
        text.contains("AI objašnjenje danas nije bilo dostupno"),
        "{text}"
    );
}
