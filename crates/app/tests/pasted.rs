//! PRAVILO-a, I1 (pregled): zalijepljen tekst (≥ 2 ponuđena odgovora, kvar B13) ne zove suca. Kod odlučuje (0 tokena):
//! posao presude odmah upiše neutralnu presudu 3, a poziv `AiPort::assess` se ne dogodi.

use std::sync::Arc;

use app::memory::FakeAi;
use app::{AiPort, Caller, Deps, GameId, HotelId, ResponseStatus, SituationItem};
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

async fn start_v2(ai: Arc<dyn AiPort>) -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai);
    let settings = app::NewGameSettings {
        model: Model::V2,
        kind: Some(PeriodKind::Month),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    (deps, caller, d.hotel_id, d.game_id)
}

async fn run_all_jobs(deps: &Deps) {
    for _ in 0..60 {
        if let Ok(false) = app::run_next_job(deps).await {
            return;
        }
    }
    panic!("red se nije ispraznio u 60 koraka");
}

async fn s1(deps: &Deps, caller: &Caller, hotel: HotelId) -> SituationItem {
    let s = app::situations(deps, caller, hotel).await.unwrap();
    s.items.into_iter().find(|i| i.code == "S1").unwrap()
}

#[tokio::test]
async fn pasted_text_does_not_call_the_judge_and_gets_neutral_three() {
    let ai = Arc::new(FakeAi::good());
    let (deps, caller, hotel, _) = start_v2(ai.clone()).await;
    let item = s1(&deps, &caller, hotel).await;
    assert!(item.answers.len() >= 2);
    app::submit_response(&deps, &caller, hotel, "S1", item.answers.join(" "))
        .await
        .unwrap();
    run_all_jobs(&deps).await;
    assert_eq!(ai.calls().0, 0, "zalijepljen tekst ne smije zvati suca");
    let r = s1(&deps, &caller, hotel).await.response.unwrap();
    assert_eq!(r.status, ResponseStatus::Neutral);
    assert_eq!(
        r.quality,
        Some(3),
        "zalijepljeno je 3 (ne 4 kao AI ispad uz OwnWords)"
    );
}

#[tokio::test]
async fn own_text_still_calls_the_judge() {
    let ai = Arc::new(FakeAi::good());
    let (deps, caller, hotel, _) = start_v2(ai.clone()).await;
    app::submit_response(
        &deps,
        &caller,
        hotel,
        "S1",
        "Ispričat ću se i ponuditi rješenje.".into(),
    )
    .await
    .unwrap();
    run_all_jobs(&deps).await;
    assert_eq!(ai.calls().0, 1);
    let r = s1(&deps, &caller, hotel).await.response.unwrap();
    assert_eq!(r.status, ResponseStatus::Judged);
}
