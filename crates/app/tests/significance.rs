//! L1-13 dio 2: pogled situacije nosi razinu značaja iz `weight` primjerka, da je web pokaže kao tekst uz naslov.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `SituationItem.significance: Option<Significance>` (jezgreni tip `sim_hotel_v2::situations::Significance`)
//!   = `Significance::from_weight(weight)` za izvučene primjerke v2; `None` za sidra S1–S4 (nemaju `weight`) i za v1.

use std::sync::Arc;

use app::memory::FakeAi;
use app::{Caller, Deps, GameId, HotelId, SituationItem};
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::situations::Significance;

async fn start(model: Model, kind: PeriodKind) -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(
        Snapshot::read(&repo_dir()).unwrap(),
        Arc::new(FakeAi::good()),
    );
    let settings = app::NewGameSettings {
        model,
        kind: (model == Model::V2).then_some(kind),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    (deps, caller, d.hotel_id, d.game_id)
}

async fn items(deps: &Deps, caller: &Caller, hotel: HotelId) -> Vec<SituationItem> {
    app::situations(deps, caller, hotel).await.unwrap().items
}

#[tokio::test]
async fn situations_view_shows_significance() {
    for kind in [PeriodKind::Month, PeriodKind::Week] {
        let (deps, caller, hotel, _) = start(Model::V2, kind).await;
        let all = items(&deps, &caller, hotel).await;
        let drawn: Vec<_> = all.iter().filter(|i| i.weight.is_some()).collect();
        let anchors: Vec<_> = all.iter().filter(|i| i.weight.is_none()).collect();
        assert!(!drawn.is_empty(), "{kind:?}: krug ima izvučene primjerke");
        assert!(!anchors.is_empty(), "{kind:?}: krug ima sidro");
        for i in drawn {
            let w = i.weight.unwrap();
            assert_eq!(
                i.significance.map(|s| s.code()),
                Some(Significance::from_weight(w).code()),
                "{kind:?} {}: weight {w}",
                i.code
            );
        }
        for i in anchors {
            assert!(
                i.significance.is_none(),
                "{kind:?} {}: sidro bez oznake",
                i.code
            );
        }
    }
}

/// Razina je izvedena iz `weight`, pa se četiri težine mapiraju na tri razine i u pogledu.
#[tokio::test]
async fn significance_matches_weight_buckets_in_the_view() {
    let (deps, caller, hotel, _) = start(Model::V2, PeriodKind::Month).await;
    for i in items(&deps, &caller, hotel).await {
        let Some(w) = i.weight else { continue };
        let want = match w {
            4 => "kljucna",
            3 => "vazna",
            _ => "svakodnevna",
        };
        assert_eq!(
            i.significance.map(|s| s.code()),
            Some(want),
            "{} weight {w}",
            i.code
        );
    }
}

#[tokio::test]
async fn classic_game_has_no_significance() {
    let (deps, caller, hotel, _) = start(Model::V1, PeriodKind::Month).await;
    let all = items(&deps, &caller, hotel).await;
    assert!(!all.is_empty());
    assert!(
        all.iter().all(|i| i.significance.is_none()),
        "v1: nema razine značaja"
    );
}
