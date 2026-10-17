//! PRAVILO-c (kartica task-12, dio PRAVILO-c; ADR-047, ADR-056 A5/A6): app pogled situacije nosi pravilo ocjene
//! vlastitog odgovora, da ga web može pokazati studentu bez računanja.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `SituationItem.click_capped: bool` = `Game::click_capped(hotel, code)` (v2 `OwnWords`, ljudski hotel, situacija
//!   s likom koja se smije odgovoriti svojim riječima); v1 (`Classic`) uvijek `false`;
//! - `ResponseView.copied: bool` = `verdict.copied.is_some()`; bez presude (`Pending`) `false`.

use std::sync::Arc;

use app::memory::FakeAi;
use app::{AiPort, Caller, Deps, GameId, HotelId, ResponseStatus, SituationItem};
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

async fn start(model: Model, ai: Arc<dyn AiPort>) -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai);
    let settings = app::NewGameSettings {
        model,
        kind: (model == Model::V2).then_some(PeriodKind::Month),
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

async fn items(deps: &Deps, caller: &Caller, hotel: HotelId) -> Vec<SituationItem> {
    app::situations(deps, caller, hotel).await.unwrap().items
}

async fn s1(deps: &Deps, caller: &Caller, hotel: HotelId) -> SituationItem {
    items(deps, caller, hotel)
        .await
        .into_iter()
        .find(|i| i.code == "S1")
        .unwrap()
}

/// v2: `click_capped` je `true` točno uz lik i vlastiti odgovor; bez lika `false`.
#[tokio::test]
async fn situations_view_marks_click_capped() {
    let (deps, caller, hotel, _) = start(Model::V2, Arc::new(FakeAi::good())).await;
    let all = items(&deps, &caller, hotel).await;
    let with_char: Vec<_> = all
        .iter()
        .filter(|i| i.free_text && i.character.is_some())
        .collect();
    let without: Vec<_> = all
        .iter()
        .filter(|i| !(i.free_text && i.character.is_some()))
        .collect();
    assert!(!with_char.is_empty(), "v2 krug ima situaciju s likom");
    assert!(!without.is_empty(), "v2 krug ima situaciju bez lika");
    for i in with_char {
        assert!(i.click_capped, "{}: lik i vlastiti odgovor => true", i.code);
    }
    for i in without {
        assert!(!i.click_capped, "{}: bez lika => false", i.code);
    }
}

/// v1 (`Classic`): klik nije ograničen ni u jednoj situaciji.
#[tokio::test]
async fn classic_game_never_marks_click_capped() {
    let (deps, caller, hotel, _) = start(Model::V1, Arc::new(FakeAi::good())).await;
    let all = items(&deps, &caller, hotel).await;
    assert!(!all.is_empty());
    assert!(
        all.iter().all(|i| !i.click_capped),
        "Classic: click_capped je uvijek false"
    );
}

/// Odgovor prepisan iz ponuđenog (jedan, ne dva: to je zalijepljeno) nosi `copied: true`.
#[tokio::test]
async fn response_view_marks_copied() {
    let (deps, caller, hotel, _) = start(Model::V2, Arc::new(FakeAi::good())).await;
    let item = s1(&deps, &caller, hotel).await;
    app::submit_response(&deps, &caller, hotel, "S1", item.answers[0].clone())
        .await
        .unwrap();
    let pending = s1(&deps, &caller, hotel).await.response.unwrap();
    assert!(!pending.copied, "bez presude nema `copied`");
    run_all_jobs(&deps).await;
    let r = s1(&deps, &caller, hotel).await.response.unwrap();
    assert_eq!(r.status, ResponseStatus::Judged);
    assert!(r.copied, "prepisan ponuđeni odgovor: copied");
}

/// Vlastite riječi nisu kopija.
#[tokio::test]
async fn own_words_response_is_not_copied() {
    let (deps, caller, hotel, _) = start(Model::V2, Arc::new(FakeAi::good())).await;
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
    let r = s1(&deps, &caller, hotel).await.response.unwrap();
    assert_eq!(r.status, ResponseStatus::Judged);
    assert!(!r.copied);
}
