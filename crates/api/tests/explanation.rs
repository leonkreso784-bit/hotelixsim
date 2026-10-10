//! „Zašto se indeksi pomaknuli": isto što mentor dobiva (`debrief_facts`), za ekran „Mjesec završen".
mod common;
use common::*;
use game::{Actor, DriverSource, Effect, Game, GameEvent, Recorded, debrief_facts};
use serde_json::Value;

async fn drain(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

fn uri(hotel: &str, month: &str) -> String {
    format!("/v1/hotels/{hotel}/months/{month}/explanation")
}

/// Igra iz dnevnika u memoriji, kao što je vidi poslužitelj.
fn game_of(mem: &app::memory::Memory, game: &str) -> Game {
    let records: Vec<Recorded> = mem
        .events(app::GameId(game.parse().unwrap()))
        .iter()
        .map(|r| Recorded {
            seq: r.seq,
            actor: Actor::parse(&r.actor).unwrap(),
            event: GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone()).unwrap(),
        })
        .collect();
    Game::fold(&records).unwrap()
}

async fn close_june(app: &axum::Router, d: &app::Deps, tok: &str, game: &str) {
    let close = call(
        app,
        "POST",
        &format!("/v1/games/{game}/rounds/jun/close"),
        Some(tok),
        None,
    )
    .await;
    assert!(close.status.is_success(), "{:?}", close.body);
    drain(d).await;
}

fn effect_code(e: Effect) -> &'static str {
    match e {
        Effect::Up => "up",
        Effect::Down => "down",
        Effect::Neutral => "neutral",
    }
}

#[tokio::test]
async fn settled_month_explains_every_index_with_mentor_facts() {
    let (app, d, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    close_june(&app, &d, &tok, &game).await;
    let r = call(&app, "GET", &uri(&hotel, "jun"), Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200, "{:?}", r.body);

    let g = game_of(&mem, &game);
    let h = app::HotelId(hotel.parse().unwrap());
    let f = debrief_facts(
        &g.debrief_input(h, game::RoundKey::parse("jun").unwrap())
            .unwrap(),
    );

    assert_eq!(r.body["month"], "jun");
    assert_eq!(r.body["progress_text"], f.progress.as_str());
    let indices = r.body["indices"].as_array().unwrap();
    let codes: Vec<&str> = indices
        .iter()
        .map(|i| i["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["G", "R", "Z", "O", "F"]);
    let mut lines = 0;
    for (body, fact) in indices.iter().zip(&f.indices) {
        assert_eq!(body["name"], fact.name);
        assert_eq!(body["before"], fact.before);
        assert_eq!(body["after"], fact.after);
        assert_eq!(body["change"], fact.change);
        let drivers = body["drivers"].as_array().unwrap();
        assert_eq!(drivers.len(), fact.drivers.len(), "{}", fact.name);
        for (db, df) in drivers.iter().zip(&fact.drivers) {
            assert_eq!(db["name"], df.what);
            assert_eq!(db["meaning"], df.meaning);
            if df.source == DriverSource::StaffLoad {
                assert_eq!(db["var"], "staff_load");
            } else {
                assert!(db["var"].is_string() && db["var"] != "staff_load");
            }
            let ls = db["lines"].as_array().unwrap();
            assert_eq!(ls.len(), df.changes.len());
            for (lb, lf) in ls.iter().zip(&df.changes) {
                assert_eq!(lb["cause"], lf.cause.as_str());
                assert_eq!(lb["amount"], lf.amount.as_str());
                assert_eq!(lb["effect"], effect_code(lf.effect));
                lines += 1;
            }
        }
    }
    assert!(lines > 5, "obračunat mjesec ima retke: {lines}");
    // Kodovi i interni podaci ne izlaze: nema šifre situacije ni štete.
    let text = r.body.to_string();
    assert!(!text.contains("\"S1\"") && !text.contains("harm"), "{text}");
}

#[tokio::test]
async fn explanation_is_only_for_own_hotel_like_feed() {
    let (app, d, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    close_june(&app, &d, &tok, &game).await;
    let u = uri(&hotel, "jun");
    assert_problem(
        &call(&app, "GET", &u, None, None).await,
        401,
        "guest_required",
    );
    assert_problem(
        &call(&app, "GET", &u, Some(&other), None).await,
        403,
        "not_in_game",
    );
    let robot = uri(&robot_hotel(&mem, &game), "jun");
    let feed = call(
        &app,
        "GET",
        &format!("/v1/hotels/{}/feed", robot_hotel(&mem, &game)),
        Some(&tok),
        None,
    )
    .await;
    let r = call(&app, "GET", &robot, Some(&tok), None).await;
    assert_problem(&r, 403, "not_your_hotel");
    assert_eq!(r.body["code"], feed.body["code"], "isto kao dnevnik");
}

#[tokio::test]
async fn month_not_settled_or_unknown_is_problem() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    // Lipanj je otvoren, srpanj još nije ni počeo: nema obračuna.
    for m in ["jun", "jul"] {
        let r = call(&app, "GET", &uri(&hotel, m), Some(&tok), None).await;
        assert_problem(&r, 409, "round_not_open");
    }
    assert_problem(
        &call(&app, "GET", &uri(&hotel, "x"), Some(&tok), None).await,
        404,
        "unknown_month",
    );
    let nobody = "00000000-0000-0000-0000-00000000abcd";
    assert_problem(
        &call(&app, "GET", &uri(nobody, "jun"), Some(&tok), None).await,
        404,
        "unknown_hotel",
    );
}

#[tokio::test]
async fn openapi_types_driver_and_effect() {
    let doc: Value = serde_json::from_str(&api::openapi_json()).unwrap();
    assert!(
        doc["paths"]["/v1/hotels/{hotel_id}/months/{month}/explanation"]["get"].is_object(),
        "ruta je u opisu"
    );
    let s = &doc["components"]["schemas"];
    let enum_of = |name: &str| -> Vec<String> {
        s[name]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: {}", s[name]))
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(enum_of("EffectCode"), ["up", "down", "neutral"]);
    assert!(enum_of("DriverCode").contains(&"staff_load".to_owned()));
    assert!(enum_of("DriverCode").contains(&"morale".to_owned()));
}
