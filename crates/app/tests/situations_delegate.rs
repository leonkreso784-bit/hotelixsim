//! A-1 (Leon P4): pogled situacija zna koje su obvezne (prve 3), tko je voditelj i što je prepušteno; ruta-funkcija
//! „prepusti voditelju" zapiše događaj i vrati pogled.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `SituationItem` dobiva `department: Option<String>` (kod odjela primjerka), `mandatory: bool`, `delegated: bool`,
//!   `leader: Option<String>` (kod odjela iz `Game::situation_leader`). Vrsta, važnost, opis i uloge provjerava API
//!   test (`crates/api/tests/situations_delegate.rs`) na JSON-u, da ovdje ne vežemo njihove tipove.
//! - `app::delegate_situation(&Deps, &Caller, HotelId, &str) -> Result<SituationsView, AppError>` (kao `app::answer`).
//! - `ErrorCode::SituationMandatory`.
//!
//! Krugovi (sjeme 42 iz `SeqIds`): mjesečna igra `jun` ima 13 situacija (S1, S3, P-DIJETE-IZGUBLJENO.jun |
//! P-ALERGIJA.jun, P-OLUJA.jun kolektivna bez odjela, …); tjedna `t05` (nakon t01–t04) ima 4 (S2 + tri primjerka).

use std::sync::Arc;

use app::memory::FakeAi;
use app::{Caller, Deps, ErrorCode, GameId, HotelId, SituationsView};
use content::{Model, Snapshot, repo_dir};
use game::RoundKey;
use sim_hotel_v2::PeriodKind;

async fn start(kind: PeriodKind) -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(
        Snapshot::read(&repo_dir()).unwrap(),
        Arc::new(FakeAi::good()),
    );
    let settings = app::NewGameSettings {
        model: Model::V2,
        kind: Some(kind),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    (deps, caller, d.hotel_id, d.game_id)
}

async fn run_all_jobs(deps: &Deps) {
    for _ in 0..200 {
        if let Ok(false) = app::run_next_job(deps).await {
            return;
        }
    }
    panic!("red se nije ispraznio u 200 koraka");
}

/// Tjedna igra s otvorenim `t05` (t01–t04 zatvoreni i obračunati bez odgovora igrača).
async fn weekly_t05() -> (Deps, Caller, HotelId, GameId, SituationsView) {
    let (deps, caller, hotel, game) = start(PeriodKind::Week).await;
    for code in ["t01", "t02", "t03", "t04"] {
        app::close_round(&deps, &caller, game, RoundKey::parse(code).unwrap())
            .await
            .unwrap();
        run_all_jobs(&deps).await;
    }
    let s = app::situations(&deps, &caller, hotel).await.unwrap();
    assert_eq!(s.round, Some(RoundKey::parse("t05").unwrap()));
    assert_eq!(s.items.len(), 4, "t05 sjemena 42");
    (deps, caller, hotel, game, s)
}

#[tokio::test]
async fn view_marks_exactly_the_first_three_as_mandatory_and_names_the_leader() {
    let (deps, caller, hotel, _) = start(PeriodKind::Month).await;
    let s = app::situations(&deps, &caller, hotel).await.unwrap();
    assert!(s.items.len() >= 5, "jun sjemena 42: {}", s.items.len());
    for (i, it) in s.items.iter().enumerate() {
        assert_eq!(it.mandatory, i < 3, "{} (mjesto {i})", it.code);
        assert!(!it.delegated, "{}", it.code);
        let leader = it
            .leader
            .as_deref()
            .unwrap_or_else(|| panic!("{}: v2 igra uvijek ima voditelja", it.code));
        match &it.department {
            Some(d) => assert_eq!(leader, d, "{}: voditelj je odjel primjerka", it.code),
            None => assert!(!leader.is_empty(), "{}", it.code),
        }
        if !it.code.contains('.') {
            assert_eq!(it.department, None, "{}: sidro nema odjel", it.code);
        }
    }
    assert!(
        s.items.iter().any(|i| i.department.is_some()),
        "jun sjemena 42 ima primjerke s odjelom"
    );
}

#[tokio::test]
async fn delegating_the_fourth_marks_it_and_lowers_unanswered() {
    let (deps, caller, hotel, game, before) = weekly_t05().await;
    assert_eq!(before.unanswered, 4);
    let code = before.items[3].code.clone();
    assert!(code.contains('.'), "{code}: šifra primjerka s točkom");
    let after = app::delegate_situation(&deps, &caller, hotel, &code)
        .await
        .unwrap();
    assert_eq!(after.round, before.round);
    assert_eq!(
        after.unanswered, 3,
        "prepuštena se ne broji kao neodgovorena"
    );
    let it = after.items.iter().find(|i| i.code == code).unwrap();
    assert!(it.delegated);
    assert_eq!((it.chosen, it.response.is_none()), (None, true));
    assert_eq!(
        after.items.iter().filter(|i| i.delegated).count(),
        1,
        "samo ona"
    );
    // isto pravilo u stanju kruga (upozorenje pri „Zatvori tjedan")
    let round = app::round_status(&deps, &caller, game, before.round.unwrap())
        .await
        .unwrap();
    assert_eq!(round.unanswered, 3);
}

#[tokio::test]
async fn mandatory_is_refused_and_delegated_cannot_be_answered_or_delegated_again() {
    let (deps, caller, hotel, _, s) = weekly_t05().await;
    for it in s.items.iter().take(3) {
        let e = app::delegate_situation(&deps, &caller, hotel, &it.code)
            .await
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::SituationMandatory, "{}", it.code);
    }
    let code = s.items[3].code.clone();
    app::delegate_situation(&deps, &caller, hotel, &code)
        .await
        .unwrap();
    let again = app::delegate_situation(&deps, &caller, hotel, &code)
        .await
        .unwrap_err();
    assert_eq!(again.code, ErrorCode::AlreadyAnswered);
    let answer = app::answer(&deps, &caller, hotel, &code, 0)
        .await
        .unwrap_err();
    assert_eq!(answer.code, ErrorCode::AlreadyAnswered);
    let unknown = app::delegate_situation(&deps, &caller, hotel, "P-NEPOSTOJI.t05")
        .await
        .unwrap_err();
    assert_eq!(unknown.code, ErrorCode::UnknownSituation);
}

#[tokio::test]
async fn collective_situation_beyond_the_first_three_can_be_delegated() {
    let (deps, caller, hotel, _) = start(PeriodKind::Month).await;
    let s = app::situations(&deps, &caller, hotel).await.unwrap();
    // kolektivna bez odjela iza trećeg mjesta (jun sjemena 42: P-OLUJA.jun)
    let it = s
        .items
        .iter()
        .skip(3)
        .find(|i| i.code.contains('.') && i.department.is_none())
        .expect("jun sjemena 42 ima primjerak bez odjela iza trećeg mjesta")
        .clone();
    assert!(
        it.leader.is_some(),
        "{}: voditelj = odjel s najviše zaposlenih",
        it.code
    );
    let v = app::delegate_situation(&deps, &caller, hotel, &it.code)
        .await
        .unwrap();
    assert!(
        v.items
            .iter()
            .find(|i| i.code == it.code)
            .unwrap()
            .delegated
    );
    assert_eq!(v.unanswered + 1, s.unanswered);
}
