use app::memory::{Memory, deps};
use app::*;
use content::{Snapshot, repo_dir};
use std::sync::Arc;

fn setup() -> (Deps, Arc<Memory>) {
    deps(Snapshot::read(&repo_dir()).unwrap())
}

async fn guest(d: &Deps) -> (Caller, DemoStarted) {
    let s = start_demo(d, None).await.unwrap();
    let c = caller(d, s.new_token.as_deref()).await.unwrap();
    (c, s)
}

fn robot_hotel(mem: &Memory, game: GameId) -> HotelId {
    let created = &mem.events(game)[0];
    HotelId(
        created.payload["hotels"][1]["hotel_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
    )
}

#[tokio::test]
async fn start_demo_creates_guest_then_reuses_it() {
    let (d, _) = setup();
    let first = start_demo(&d, None).await.unwrap();
    let tok = first.new_token.clone().expect("novi gost dobiva token");
    assert_eq!(tok.len(), 32);
    let second = start_demo(&d, Some(&tok)).await.unwrap();
    assert_eq!(
        second.new_token, None,
        "postojeći gost ne dobiva novi kolačić"
    );
    assert_ne!(second.game_id, first.game_id);
    let c = caller(&d, Some(&tok)).await.unwrap();
    assert_eq!(
        my_hotel(&d, &c).await.unwrap().game_id,
        second.game_id,
        "moj hotel = zadnja igra"
    );
    let unknown = start_demo(&d, Some("0123456789abcdef0123456789abcdef"))
        .await
        .unwrap();
    assert!(unknown.new_token.is_some(), "nepoznat token = novi gost");
}

#[tokio::test]
async fn visibility_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let anon = Caller { guest: None };
    let own = hotel_view(&d, &me, s.hotel_id).await.unwrap();
    assert!(own.private.is_some(), "vlasnik vidi stvarne brojke");
    let robot = hotel_view(&d, &me, robot_hotel(&mem, s.game_id))
        .await
        .unwrap();
    assert!(
        robot.private.is_none(),
        "tuđi hotel: samo ocjena i indeksi (ARHITEKTURA §12)"
    );
    assert_eq!(
        hotel_view(&d, &other, s.hotel_id).await.unwrap_err().code,
        ErrorCode::NotInGame
    );
    assert_eq!(
        hotel_view(&d, &anon, s.hotel_id).await.unwrap_err().code,
        ErrorCode::GuestRequired
    );
    assert_eq!(
        game_view(&d, &other, s.game_id).await.unwrap_err().code,
        ErrorCode::NotInGame
    );
    assert_eq!(
        my_hotel(&d, &anon).await.unwrap_err().code,
        ErrorCode::GuestRequired
    );
}

#[tokio::test]
async fn game_view_lists_four_hotels_with_open_june() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.hotels.len(), 4);
    assert_eq!(g.my_hotel, s.hotel_id);
    let r = g.current.unwrap();
    assert_eq!(
        (r.month, r.status, r.unanswered),
        (game::dto::MonthCode::Jun, game::RoundStatus::Open, 2)
    );
}

#[tokio::test]
async fn situations_and_feed_are_owner_only() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert!(v.open);
    assert_eq!(
        v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
        ["S1", "S3"]
    );
    assert!(
        v.items
            .iter()
            .all(|i| i.answers.len() == 4 && i.chosen.is_none())
    );
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(
        situations(&d, &me, robot).await.unwrap_err().code,
        ErrorCode::NotYourHotel
    );
    assert_eq!(
        feed(&d, &me, robot).await.unwrap_err().code,
        ErrorCode::NotYourHotel
    );
    assert!(!feed(&d, &me, s.hotel_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn unknown_ids_and_profiles() {
    let (d, _) = setup();
    let (me, _) = guest(&d).await;
    let nobody = uuid::Uuid::from_u128(9999);
    assert_eq!(
        hotel_view(&d, &me, HotelId(nobody)).await.unwrap_err().code,
        ErrorCode::UnknownHotel
    );
    assert_eq!(
        game_view(&d, &me, GameId(nobody)).await.unwrap_err().code,
        ErrorCode::UnknownGame
    );
    assert_eq!(profile_indices("C").unwrap(), [5000; 5]);
    assert_eq!(
        profile_indices("D").unwrap_err().code,
        ErrorCode::UnknownProfile
    );
}

#[tokio::test]
async fn latest_hotel_is_newest_game_where_guest_has_a_hotel() {
    let (d, _) = setup();
    let (c, first) = guest(&d).await;
    let me = c.guest.unwrap();
    let hotel_for = |guest_id| {
        vec![NewHotel {
            hotel_id: HotelId(uuid::Uuid::new_v4()),
            slot: 1,
            guest_id,
        }]
    };
    let new_game = |created_by, hotels| NewGame {
        game_id: GameId(uuid::Uuid::new_v4()),
        created_by,
        hotels,
        events: vec![],
    };
    // Novija igra koju je stvorio netko drugi, ali u kojoj gost IMA hotel: ona je zadnja.
    let other = GuestId(uuid::Uuid::new_v4());
    let theirs = new_game(other, hotel_for(Some(me)));
    let (theirs_id, theirs_hotel) = (theirs.game_id, theirs.hotels[0].hotel_id);
    d.log.create_game(theirs).await.unwrap();
    assert_eq!(
        d.guests.latest_hotel_of(me).await.unwrap(),
        Some((theirs_id, theirs_hotel))
    );
    // Još novija igra koju je gost stvorio, ali bez hotela za njega: preskače se.
    d.log
        .create_game(new_game(me, hotel_for(None)))
        .await
        .unwrap();
    assert_eq!(
        d.guests.latest_hotel_of(me).await.unwrap(),
        Some((theirs_id, theirs_hotel))
    );
    assert_ne!(theirs_id, first.game_id);
}
