//! Pilot-2 r2 (I3): posao prekinut gašenjem vraća se u red BEZ potrošenog pokušaja, pa ga nova instanca
//! preuzme odmah (ne tek nakon isteka zakupa od 180 s).
use std::sync::Arc;

use app::memory::JobStatus;
use app::memory::MemJob;
use app::{AiOut, AiPort, BoxFut, NewGameSettings};
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

/// AI koji nikad ne odgovori (poziv u tijeku u trenutku gašenja).
struct HangingAi;

impl AiPort for HangingAi {
    fn assess(
        &self,
        _: game::AssessInput,
    ) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, app::AiError>> {
        Box::pin(std::future::pending())
    }
    fn character(
        &self,
        _: game::CharacterInput,
    ) -> BoxFut<'_, Result<AiOut<String>, app::AiError>> {
        Box::pin(std::future::pending())
    }
    fn debrief(&self, _: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, app::AiError>> {
        Box::pin(std::future::pending())
    }
    fn season_debrief(
        &self,
        _: game::SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, app::AiError>> {
        Box::pin(std::future::pending())
    }
}

fn open(mem: &app::memory::Memory) -> Vec<MemJob> {
    mem.jobs()
}

#[tokio::test]
async fn cancelled_job_goes_back_to_queue_without_spending_an_attempt() {
    let (deps, mem) =
        app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), Arc::new(HangingAi));
    let d = app::start_demo(
        &deps,
        None,
        NewGameSettings {
            model: Model::V2,
            kind: Some(PeriodKind::Month),
        },
    )
    .await
    .unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    let s = app::situations(&deps, &caller, d.hotel_id).await.unwrap();
    let item = s
        .items
        .into_iter()
        .find(|i| i.free_text && i.character.is_some())
        .expect("situacija s likom");
    app::send_message(&deps, &caller, d.hotel_id, &item.code, "Dobar dan.".into())
        .await
        .unwrap();
    let before = open(&mem);
    assert!(!before.is_empty());

    // posao se preuzme, AI visi, gašenje stiže: posao mora natrag u red
    let did = app::run_next_job_until(&deps, async {}).await.unwrap();
    assert!(did);
    let after = open(&mem);
    assert!(
        after.iter().all(|j| j.status != JobStatus::Running),
        "prekinut posao ne smije ostati 'running': {after:?}"
    );
    let attempts_before: i32 = before.iter().map(|j| j.attempts).sum();
    let attempts_after: i32 = after.iter().map(|j| j.attempts).sum();
    assert_eq!(attempts_before, attempts_after, "pokušaj se ne troši");
}
