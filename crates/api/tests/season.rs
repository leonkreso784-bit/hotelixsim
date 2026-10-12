//! `GET /v1/games/{id}/season` (kraj-sezone-spec §6.1): 200 nakon zadnjeg kruga, 409 prije i za v1, pravila pristupa
//! kao `GET /v1/games/{id}` i skrivene brojke tuđeg ljudskog hotela.
mod common;

use serde_json::{Value, json};

/// v2 mjesečna igra (četiri kruga): (app, deps, memorija, kolačić, igra, moj hotel).
async fn monthly_v2() -> (
    axum::Router,
    app::Deps,
    std::sync::Arc<app::memory::Memory>,
    String,
    String,
    String,
) {
    let (app, deps, mem) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2", "round_kind": "month" })),
    )
    .await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    (app, deps, mem, c, g, h)
}

async fn play_to_end(app: &axum::Router, deps: &app::Deps, c: &str, g: &str) {
    for code in ["jun", "jul", "aug", "sep"] {
        let r = common::call(
            app,
            "POST",
            &format!("/v1/games/{g}/rounds/{code}/close"),
            Some(c),
            None,
        )
        .await;
        assert_eq!(r.status, 202, "{code}: {:?}", r.body);
        common::run_worker(deps).await;
    }
}

async fn season(app: &axum::Router, c: Option<&str>, g: &str) -> common::Resp {
    common::call(app, "GET", &format!("/v1/games/{g}/season"), c, None).await
}

fn hotel<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["hotels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["hotel_id"] == id)
        .unwrap()
}

#[tokio::test]
async fn member_sees_season_after_last_round() {
    let (app, deps, _, c, g, h) = monthly_v2().await;
    play_to_end(&app, &deps, &c, &g).await;
    let r = season(&app, Some(&c), &g).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let b = &r.body;
    assert_eq!(b["game_id"], g.as_str());
    assert_eq!(b["my_hotel_id"], h.as_str());
    assert_eq!(b["round_kind"], "month");
    assert_eq!(b["rounds"].as_array().unwrap().len(), 4);
    // Radnik je uz zadnji krug napisao i sezonskog mentora (lažni AI u testovima).
    let mentor = b["mentor"]["text"].as_str().unwrap();
    assert!(
        mentor.starts_with(
            "Kako je prošla sezona
Mentor: dobra sezona."
        ),
        "{mentor}"
    );
    let hotels = b["hotels"].as_array().unwrap();
    assert_eq!(hotels.len(), 4);
    let ranks: Vec<i64> = hotels.iter().map(|x| x["rank"].as_i64().unwrap()).collect();
    assert_eq!(ranks[0], 1);
    assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{ranks:?}");
    for x in hotels {
        assert_eq!(x["winner"], x["rank"] == 1);
        assert_eq!(x["curve"].as_array().unwrap().len(), 4);
        assert_eq!(x["start"].as_array().unwrap().len(), 5);
        assert_eq!(x["end"].as_array().unwrap().len(), 5);
        assert!(x["finances"].is_object(), "{x}");
        assert!(x["decisions"].is_object(), "{x}");
    }
    let mine = hotel(b, &h);
    assert_eq!(mine["mine"], true);
    assert_eq!(mine["seat"]["kind"], "human");
    assert_eq!(mine["curve"][3]["round"], "sep");
    let robots: Vec<&str> = hotels
        .iter()
        .filter(|x| x["seat"]["kind"] == "robot")
        .map(|x| x["seat"]["strategy"].as_str().unwrap())
        .collect();
    assert_eq!(robots.len(), 3);

    // Isti brojevi kao „Račun dobiti → Sezona".
    let pl = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?scope=season"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(pl.status, 200);
    let gop = pl.body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["lines"].as_array().unwrap())
        .find(|l| l["code"] == "gop")
        .unwrap();
    assert_eq!(mine["finances"]["gop_cents"], gop["actual_cents"]);
    assert_eq!(
        mine["finances"]["gop_margin_bp"],
        pl.body["gop_margin_bp"]["actual"]
    );
    let d = &mine["decisions"];
    assert!(d["renovation_round"].is_null());
    assert!(d["situations"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn season_in_progress_is_409_season_not_over() {
    let (app, _, _, c, g, _) = monthly_v2().await;
    let r = season(&app, Some(&c), &g).await;
    common::assert_problem(&r, 409, "season_not_over");
    assert_eq!(
        r.body["title"],
        "Sezona još traje. Rezultati se vide kad se obračuna zadnji krug."
    );
}

#[tokio::test]
async fn v1_game_is_409_model_v1() {
    let (app, _, _) = common::app();
    let (c, g, _) = common::demo(&app).await;
    let r = season(&app, Some(&c), &g).await;
    common::assert_problem(&r, 409, "model_v1");
}

#[tokio::test]
async fn access_rules_match_game_view() {
    let (app, deps, _, c, g, _) = monthly_v2().await;
    play_to_end(&app, &deps, &c, &g).await;
    let r = season(&app, None, &g).await;
    common::assert_problem(&r, 401, "guest_required");
    let (other, _, _) = common::demo(&app).await;
    let r = season(&app, Some(&other), &g).await;
    common::assert_problem(&r, 403, "not_in_game");
    let r = season(&app, Some(&c), "00000000-0000-0000-0000-000000000009").await;
    common::assert_problem(&r, 404, "unknown_game");
}

#[tokio::test]
async fn other_human_hotel_has_no_finances_or_decisions() {
    let (app, deps, mem, c, g, h) = monthly_v2().await;
    play_to_end(&app, &deps, &c, &g).await;
    let theirs = common::robot_hotel(&mem, &g);
    // Drugo sjedalo postaje čovjek (igra s više studenata još ne postoji, pa se dnevnik namješta).
    mem.patch_first_event(&g, |p| {
        p["hotels"][1]["controller"] =
            json!({ "kind": "human", "guest_id": "00000000-0000-0000-0000-000000000077" });
    });
    let r = season(&app, Some(&c), &g).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let x = hotel(&r.body, &theirs);
    assert_eq!(x["seat"]["kind"], "human");
    assert_eq!(x["mine"], false);
    assert!(x["finances"].is_null());
    assert!(x["decisions"].is_null());
    assert_eq!(x["curve"].as_array().unwrap().len(), 4);
    let mine = hotel(&r.body, &h);
    assert!(mine["finances"].is_object() && mine["decisions"].is_object());
}
