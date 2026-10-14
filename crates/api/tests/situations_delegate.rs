//! A-1: `GET /v1/hotels/{id}/situations` nosi odjel, vrstu, važnost, opis, uloge, obvezne (prve 3), prepuštene i
//! voditelja; `POST /v1/hotels/{id}/situations/{code}/delegate` prepušta situaciju voditelju (Leon P4).
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - Svaka stavka ima ključeve `department`, `kind`, `weight`, `description`, `roles`, `mandatory`, `delegated`,
//!   `leader` (uvijek prisutni, `null` kad vrijednosti nema — kao `chosen`).
//! - `department` i `leader` su KOD odjela (`restaurant`), ne ime; `leader == department` kad primjerak ima odjel.
//! - `kind` je `snake_case` enum: complaint | staff | opportunity | breakdown | crisis | request; `weight` je broj.
//! - 🟨 Sidra S1–S4: `department`, `kind`, `weight` su `null` (sadržaj sidra ih nema).
//! - `roles` = popis objekata `{ key, name, role }` (`role` = rečenica uloge ili `null`); uloga lika ima
//!   `name == character.name` i `role == character.role`.
//! - Prepuštanje vraća 200 i tijelo kao `GET …/situations`; obvezna → 409 `situation_mandatory` s hrvatskim naslovom.
//!
//! Krugovi (sjeme 42): tjedna `t01` = S1, S3, P-OLUJA.t01 (kolektivna); `t05` (nakon t01–t04) = S2 + tri primjerka;
//! mjesečna `jun` = 13 situacija.

mod common;

use axum::Router;
use common::{assert_problem, call};
use serde_json::{Value, json};

const KINDS: [&str; 6] = [
    "complaint",
    "staff",
    "opportunity",
    "breakdown",
    "crisis",
    "request",
];

struct G {
    app: Router,
    deps: app::Deps,
    mem: std::sync::Arc<app::memory::Memory>,
    c: String,
    game: String,
    hotel: String,
}

async fn v2(kind: &str) -> G {
    let (app, deps, mem) = common::app();
    let r = call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2", "round_kind": kind })),
    )
    .await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    G {
        c: common::cookie_value(r.set_cookie.as_deref().unwrap()),
        game: r.body["game_id"].as_str().unwrap().to_owned(),
        hotel: r.body["hotel_id"].as_str().unwrap().to_owned(),
        app,
        deps,
        mem,
    }
}

async fn situations(g: &G) -> Value {
    let r = call(
        &g.app,
        "GET",
        &format!("/v1/hotels/{}/situations", g.hotel),
        Some(&g.c),
        None,
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    r.body
}

async fn delegate(g: &G, code: &str) -> common::Resp {
    call(
        &g.app,
        "POST",
        &format!("/v1/hotels/{}/situations/{code}/delegate", g.hotel),
        Some(&g.c),
        None,
    )
    .await
}

/// Tjedna igra s otvorenim `t05` (t01–t04 zatvoreni i obračunati bez odgovora igrača).
async fn weekly_t05() -> G {
    let g = v2("week").await;
    for code in ["t01", "t02", "t03", "t04"] {
        let r = call(
            &g.app,
            "POST",
            &format!("/v1/games/{}/rounds/{code}/close", g.game),
            Some(&g.c),
            None,
        )
        .await;
        assert_eq!(r.status, 202, "{code}");
        common::run_worker(&g.deps).await;
    }
    g
}

fn clean(what: &str, s: &str) {
    assert!(!s.trim().is_empty(), "{what}: prazno");
    assert!(!s.contains('{') && !s.contains('}'), "{what}: {s}");
}

/// Nova polja jedne stavke na mjestu `i` popisa.
fn check_item(it: &Value, i: usize) {
    let code = it["code"].as_str().unwrap();
    for key in [
        "department",
        "kind",
        "weight",
        "description",
        "roles",
        "mandatory",
        "delegated",
        "leader",
    ] {
        assert!(it.get(key).is_some(), "{code}: nema ključa `{key}`: {it}");
    }
    assert_eq!(it["mandatory"], i < 3, "{code} (mjesto {i})");
    let leader = it["leader"]
        .as_str()
        .unwrap_or_else(|| panic!("{code}: voditelj"));
    assert!(
        !leader.is_empty() && leader.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
        "{code}: voditelj je kod odjela: {leader}"
    );
    if !code.contains('.') {
        for key in ["department", "kind", "weight"] {
            assert!(it[key].is_null(), "{code}: sidro nema `{key}`: {it}");
        }
        return;
    }
    if let Some(d) = it["department"].as_str() {
        assert_eq!(leader, d, "{code}: voditelj je odjel primjerka");
    } else {
        assert!(it["department"].is_null(), "{code}: {it}");
    }
    let kind = it["kind"]
        .as_str()
        .unwrap_or_else(|| panic!("{code}: vrsta"));
    assert!(KINDS.contains(&kind), "{code}: vrsta {kind}");
    let w = it["weight"]
        .as_u64()
        .unwrap_or_else(|| panic!("{code}: važnost"));
    assert!((1..=5).contains(&w), "{code}: važnost {w}");
    clean(
        &format!("{code} opis"),
        it["description"].as_str().unwrap_or(""),
    );
    for r in it["roles"]
        .as_array()
        .unwrap_or_else(|| panic!("{code}: uloge"))
    {
        assert!(
            r["key"].as_str().is_some_and(|k| !k.is_empty()),
            "{code}: {r}"
        );
        clean(
            &format!("{code} ime uloge"),
            r["name"].as_str().unwrap_or(""),
        );
        if !r["role"].is_null() {
            clean(
                &format!("{code} rečenica uloge"),
                r["role"].as_str().unwrap(),
            );
        }
    }
    if let Some(ch) = it["character"].as_object() {
        let roles = it["roles"].as_array().unwrap();
        assert!(
            roles
                .iter()
                .any(|r| r["name"] == ch["name"] && r["role"] == ch["role"]),
            "{code}: lik {ch:?} nije među ulogama {roles:?}"
        );
    }
}

fn check_all(body: &Value) {
    let items = body["items"].as_array().unwrap();
    for (i, it) in items.iter().enumerate() {
        check_item(it, i);
        assert_eq!(it["delegated"], false);
    }
    // primjerci po važnosti silazno (sidra su prva)
    let w: Vec<u64> = items.iter().filter_map(|i| i["weight"].as_u64()).collect();
    assert!(w.windows(2).all(|p| p[0] >= p[1]), "važnost {w:?}");
}

#[tokio::test]
async fn weekly_first_round_with_anchors_and_collective_carries_the_new_fields() {
    let g = v2("week").await;
    let s = situations(&g).await;
    assert_eq!(s["round"], "t01");
    let items = s["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "t01 sjemena 42");
    assert!(
        items[2]["code"].as_str().unwrap().contains('.'),
        "kolektivna"
    );
    check_all(&s);
}

#[tokio::test]
async fn monthly_round_items_carry_roles_of_the_character() {
    let g = v2("month").await;
    let s = situations(&g).await;
    check_all(&s);
    let items = s["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|i| !i["character"].is_null() && !i["roles"].as_array().unwrap().is_empty()),
        "jun sjemena 42 ima primjerak s likom i ulogama"
    );
}

#[tokio::test]
async fn weekly_round_of_four_delegates_the_fourth() {
    let g = weekly_t05().await;
    let s = situations(&g).await;
    assert_eq!(s["round"], "t05");
    let items = s["items"].as_array().unwrap();
    assert_eq!(items.len(), 4, "t05 sjemena 42");
    check_all(&s);
    assert_eq!(s["unanswered"], 4);
    let code = items[3]["code"].as_str().unwrap().to_owned();
    assert!(code.contains('.'), "ruta prima šifru s točkom: {code}");

    let r = delegate(&g, &code).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["round"], "t05");
    assert_eq!(r.body["unanswered"], 3);
    let it = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == code.as_str())
        .unwrap();
    assert_eq!(it["delegated"], true);
    assert!(it["chosen"].is_null());

    assert_problem(&delegate(&g, &code).await, 409, "already_answered");
    let answer = call(
        &g.app,
        "POST",
        &format!("/v1/hotels/{}/situations/{code}/answer", g.hotel),
        Some(&g.c),
        Some(json!({ "answer": 0 })),
    )
    .await;
    assert_problem(&answer, 409, "already_answered");
}

#[tokio::test]
async fn delegating_a_mandatory_situation_is_409_with_a_croatian_title() {
    let g = v2("week").await;
    let s = situations(&g).await;
    for it in s["items"].as_array().unwrap() {
        let code = it["code"].as_str().unwrap();
        let r = delegate(&g, code).await;
        assert_problem(&r, 409, "situation_mandatory");
        let title = r.body["title"].as_str().unwrap_or("");
        assert!(
            !title.is_empty() && !title.contains('_') && title != "situation_mandatory",
            "{code}: naslov za čovjeka: {title:?}"
        );
    }
}

#[tokio::test]
async fn unknown_situation_and_foreign_hotel_keep_existing_errors() {
    let g = v2("week").await;
    assert_problem(
        &delegate(&g, "P-NEPOSTOJI.t01").await,
        404,
        "unknown_situation",
    );
    let robot = common::robot_hotel(&g.mem, &g.game);
    let r = call(
        &g.app,
        "POST",
        &format!("/v1/hotels/{robot}/situations/S1/delegate"),
        Some(&g.c),
        None,
    )
    .await;
    assert_problem(&r, 403, "not_your_hotel");
}

#[test]
fn openapi_documents_the_delegate_route_and_the_new_fields() {
    let doc: Value = serde_json::from_str(&api::openapi_json()).unwrap();
    let post = &doc["paths"]["/v1/hotels/{hotel_id}/situations/{code}/delegate"]["post"];
    assert!(post.is_object(), "nema POST …/delegate");
    for status in ["200", "409", "404"] {
        assert!(post["responses"][status].is_object(), "…/delegate {status}");
    }
    let props = &doc["components"]["schemas"]["SituationBody"]["properties"];
    for key in [
        "department",
        "kind",
        "weight",
        "description",
        "roles",
        "mandatory",
        "delegated",
        "leader",
    ] {
        assert!(props[key].is_object(), "SituationBody.{key}");
    }
    let has_kind_enum = doc["components"]["schemas"]
        .as_object()
        .unwrap()
        .values()
        .any(|s| {
            s["enum"].as_array().is_some_and(|e| {
                e.len() == KINDS.len() && KINDS.iter().all(|k| e.contains(&json!(k)))
            })
        });
    assert!(has_kind_enum, "vrsta situacije kao snake_case enum u shemi");
}
