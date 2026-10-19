//! Pilot, dio 4 (ADR-062 t. 9): najviše 3 igre po gostu. Broje se igre koje je gost STVORIO (`games.created_by`),
//! svih vrsta; gost bez kolačića dobiva novog gosta pa granica na njega ne djeluje (Leon 19.10.: bez granice po IP-u).
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `Deps.max_games_per_guest: Option<u32>` (`None` = bez granice; `memory::deps` ga ostavlja na `None`);
//! - `ErrorCode::GameLimit` (HTTP 409, kod `game_limit`) kad gost koji ima `max` igara traži novu;
//! - `Guests::count_games_created_by(guest) -> Result<u32, StoreError>` (broj redaka `games` s tim `created_by`).

use app::memory::{Memory, deps};
use app::*;
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;
use std::sync::Arc;

fn setup(max: Option<u32>) -> (Deps, Arc<Memory>) {
    let (mut d, m) = deps(Snapshot::read(&repo_dir()).unwrap());
    d.max_games_per_guest = max;
    (d, m)
}

fn v1() -> NewGameSettings {
    NewGameSettings::default()
}

fn v2(kind: PeriodKind) -> NewGameSettings {
    NewGameSettings {
        model: Model::V2,
        kind: Some(kind),
    }
}

async fn guest_of(d: &Deps, token: &str) -> GuestId {
    caller(d, Some(token)).await.unwrap().guest.unwrap()
}

#[tokio::test]
async fn fourth_game_of_a_guest_is_refused_and_creates_nothing() {
    let (d, _) = setup(Some(3));
    let first = start_demo(&d, None, v1()).await.unwrap();
    let tok = first.new_token.clone().unwrap();
    let guest = guest_of(&d, &tok).await;
    for n in 2..=3 {
        let s = start_demo(&d, Some(&tok), v1())
            .await
            .unwrap_or_else(|e| panic!("igra {n} mora uspjeti: {e:?}"));
        assert_eq!(s.new_token, None);
    }
    assert_eq!(d.guests.count_games_created_by(guest).await.unwrap(), 3);
    let err = start_demo(&d, Some(&tok), v1()).await.unwrap_err();
    assert_eq!(err.code, ErrorCode::GameLimit);
    assert_eq!(err.code.kind(), ErrorKind::Conflict);
    assert_eq!(err.code.as_str(), "game_limit");
    assert_eq!(
        d.guests.count_games_created_by(guest).await.unwrap(),
        3,
        "odbijena igra se ne upisuje"
    );
    // i dalje isti gost
    assert_eq!(guest_of(&d, &tok).await, guest);
}

#[tokio::test]
async fn all_kinds_of_games_count_towards_the_limit() {
    let (d, _) = setup(Some(3));
    let first = start_demo(&d, None, v2(PeriodKind::Week)).await.unwrap();
    let tok = first.new_token.clone().unwrap();
    start_demo(&d, Some(&tok), v2(PeriodKind::Month))
        .await
        .unwrap();
    start_demo(&d, Some(&tok), v1()).await.unwrap();
    for s in [v1(), v2(PeriodKind::Week), v2(PeriodKind::Month)] {
        let err = start_demo(&d, Some(&tok), s).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::GameLimit, "{s:?}");
    }
}

#[tokio::test]
async fn limit_is_per_guest() {
    let (d, _) = setup(Some(3));
    let a = start_demo(&d, None, v1()).await.unwrap();
    let ta = a.new_token.clone().unwrap();
    start_demo(&d, Some(&ta), v1()).await.unwrap();
    start_demo(&d, Some(&ta), v1()).await.unwrap();
    assert_eq!(
        start_demo(&d, Some(&ta), v1()).await.unwrap_err().code,
        ErrorCode::GameLimit
    );
    let b = start_demo(&d, None, v1()).await.unwrap();
    let tb = b.new_token.clone().unwrap();
    assert_ne!(guest_of(&d, &ta).await, guest_of(&d, &tb).await);
    assert_eq!(
        d.guests
            .count_games_created_by(guest_of(&d, &tb).await)
            .await
            .unwrap(),
        1,
        "drugi gost ima svoj brojač"
    );
    start_demo(&d, Some(&tb), v1()).await.unwrap();
    start_demo(&d, Some(&tb), v1()).await.unwrap();
    assert_eq!(
        start_demo(&d, Some(&tb), v1()).await.unwrap_err().code,
        ErrorCode::GameLimit
    );
}

#[tokio::test]
async fn without_a_limit_a_guest_may_start_any_number_of_games() {
    let (d, _) = setup(None);
    let first = start_demo(&d, None, v1()).await.unwrap();
    let tok = first.new_token.clone().unwrap();
    for _ in 0..5 {
        start_demo(&d, Some(&tok), v1()).await.unwrap();
    }
}

#[tokio::test]
async fn guest_without_cookie_is_a_new_guest_and_is_not_limited() {
    // 🟨 poznato zaobilaženje (Leon 19.10.: bez granice po IP-u): bez kolačića = novi gost s novim brojačem.
    let (d, _) = setup(Some(3));
    for _ in 0..5 {
        let s = start_demo(&d, None, v1()).await.unwrap();
        assert!(s.new_token.is_some());
    }
}

#[tokio::test]
async fn unknown_token_counts_as_a_new_guest() {
    let (d, _) = setup(Some(1));
    let s = start_demo(&d, Some("0123456789abcdef0123456789abcdef"), v1())
        .await
        .unwrap();
    assert!(s.new_token.is_some());
}

#[tokio::test]
async fn limit_of_one_refuses_the_second_game() {
    let (d, _) = setup(Some(1));
    let first = start_demo(&d, None, v1()).await.unwrap();
    let tok = first.new_token.unwrap();
    assert_eq!(
        start_demo(&d, Some(&tok), v1()).await.unwrap_err().code,
        ErrorCode::GameLimit
    );
}
