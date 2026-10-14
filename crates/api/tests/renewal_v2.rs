//! Obnova v2 kroz HTTP (obnova-v2-spec §4.3–4.4, §8 O-3): `PUT plans/{round}` bez iznosa obnove zadržava zaključani
//! iznos, drugi iznos je 422 `renewal_locked`; sve tri nove šifre grešaka; granice nude izbor iznosa; prikaz hotela
//! ima zgradu samo u v2; račun dobiti ima amortizaciju i EBIT; igra stvorena jezgrom 2.0.0 je zastarjela.
mod common;

use serde_json::{Value, json};

/// v2 igra s obnovom u sadržaju: (app, deps, kolačić, igra, hotel).
async fn renewal_game() -> (axum::Router, app::Deps, String, String, String) {
    let (app, deps) = common::renewal_app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2" })),
    )
    .await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    (app, deps, c, g, h)
}

async fn private(app: &axum::Router, c: &str) -> Value {
    let r = common::call(app, "GET", "/v1/me/hotel", Some(c), None).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    r.body["private"].clone()
}

async fn put_plan(app: &axum::Router, c: &str, h: &str, round: &str, plan: Value) -> common::Resp {
    common::call(
        app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/{round}"),
        Some(c),
        Some(plan),
    )
    .await
}

async fn close(app: &axum::Router, deps: &app::Deps, c: &str, g: &str, round: &str) {
    let r = common::call(
        app,
        "POST",
        &format!("/v1/games/{g}/rounds/{round}/close"),
        Some(c),
        None,
    )
    .await;
    assert_eq!(r.status, 202, "{round}: {:?}", r.body);
    common::run_worker(deps).await;
}

/// Plan t01 s Restoranom 40.000 € i zgradom 100.000 €, pa obračun t01: obje obnove su zaključane za lipanj.
async fn renew_in_t01() -> (axum::Router, app::Deps, String, String, String) {
    let (app, deps, c, g, h) = renewal_game().await;
    let mut plan = private(&app, &c).await["plan"]["plan"].clone();
    assert_eq!(
        plan["building_renewal_cents"], 0,
        "v2 plan uvijek pokazuje iznos"
    );
    assert_eq!(plan["departments"]["restaurant"]["renewal_cents"], 0);
    plan["departments"]["restaurant"]["renewal_cents"] = json!(4_000_000);
    plan["building_renewal_cents"] = json!(10_000_000);
    let r = put_plan(&app, &c, &h, "t01", plan).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    close(&app, &deps, &c, &g, "t01").await;
    (app, deps, c, g, h)
}

#[tokio::test]
async fn put_plan_without_renewal_keeps_the_locked_amount_and_another_is_renewal_locked() {
    let (app, _deps, c, _g, h) = renew_in_t01().await;
    let p = private(&app, &c).await;
    assert_eq!(p["plan"]["round"], "t02");
    let mut plan = p["plan"]["plan"].clone();
    assert_eq!(
        plan["departments"]["restaurant"]["renewal_cents"],
        4_000_000
    );
    assert_eq!(plan["building_renewal_cents"], 10_000_000);
    let locked = &p["plan"]["limits"]["departments"]["restaurant"]["renewal"]["locked"];
    assert_eq!(
        *locked,
        json!({ "month": "jun", "total_cents": 4_000_000, "installment_cents": 800_000, "paid_cents": 800_000,
                "rounds_left": 4, "gain_pts": 2_105, "gained_pts": 421 })
    );

    // Tijelo bez iznosa obnove (odjeli i zgrada) → zadržava zaključane iznose.
    for d in plan["departments"].as_object_mut().unwrap().values_mut() {
        d.as_object_mut().unwrap().remove("renewal_cents");
    }
    plan.as_object_mut()
        .unwrap()
        .remove("building_renewal_cents");
    let r = put_plan(&app, &c, &h, "t02", plan.clone()).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let saved = private(&app, &c).await["plan"]["plan"].clone();
    assert_eq!(
        saved["departments"]["restaurant"]["renewal_cents"],
        4_000_000
    );
    assert_eq!(saved["building_renewal_cents"], 10_000_000);

    // Drugi iznos u istom mjesecu → 422 renewal_locked.
    let mut other = plan.clone();
    other["departments"]["restaurant"]["renewal_cents"] = json!(500_000);
    let r = put_plan(&app, &c, &h, "t02", other).await;
    common::assert_problem(&r, 422, "renewal_locked");
    assert_eq!(
        r.body["title"],
        "Obnova je za ovaj mjesec već odlučena i ne mijenja se do kraja mjeseca."
    );
    let mut other = plan;
    other["building_renewal_cents"] = json!(0);
    let r = put_plan(&app, &c, &h, "t02", other).await;
    common::assert_problem(&r, 422, "renewal_locked");
}

#[tokio::test]
async fn renewal_out_of_range_and_unavailable_are_422() {
    let (app, _deps, c, _g, h) = renewal_game().await;
    let plan = private(&app, &c).await["plan"]["plan"].clone();

    // Nije u koraku od 5.000 €.
    let mut p = plan.clone();
    p["departments"]["restaurant"]["renewal_cents"] = json!(123_400);
    let r = put_plan(&app, &c, &h, "t01", p).await;
    common::assert_problem(&r, 422, "renewal_out_of_range");
    assert_eq!(
        r.body["title"],
        "Iznos obnove nije dopušten: provjeri najveći mjesečni iznos i korak."
    );
    // Zgrada iznad najvećeg mjesečnog iznosa (200.000 €).
    let mut p = plan.clone();
    p["building_renewal_cents"] = json!(21_000_000);
    common::assert_problem(
        &put_plan(&app, &c, &h, "t01", p).await,
        422,
        "renewal_out_of_range",
    );

    // Neraspoređeni (Uprava) obnovu nemaju.
    let mut p = plan.clone();
    p["departments"]["admin"]["renewal_cents"] = json!(500_000);
    let r = put_plan(&app, &c, &h, "t01", p).await;
    common::assert_problem(&r, 422, "renewal_unavailable");
    assert_eq!(
        r.body["title"],
        "Ovdje se obnova ne bira. U ovoj igri obnova se bira iznosom, po odjelu i za zgradu."
    );
    // Stara kvačica u sadržaju s obnovom po iznosu.
    let mut p = plan;
    p["renovation"] = json!(true);
    common::assert_problem(
        &put_plan(&app, &c, &h, "t01", p).await,
        422,
        "renewal_unavailable",
    );
}

#[tokio::test]
async fn error_codes_have_36_entries_and_the_three_renewal_codes() {
    let all = app::ErrorCode::ALL;
    assert_eq!(all.len(), 36);
    let mut codes: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), 36, "šifre su jedinstvene");
    for c in all {
        let p = api::problem::ProblemCode::from(c);
        assert_eq!(serde_json::to_value(p).unwrap(), json!(c.as_str()));
        assert!(!p.title().is_empty());
    }
    for code in [
        "renewal_unavailable",
        "renewal_locked",
        "renewal_out_of_range",
    ] {
        let c = all.iter().find(|c| c.as_str() == code).unwrap();
        assert_eq!(c.kind(), app::ErrorKind::Invalid, "{code}");
    }
}

#[tokio::test]
async fn limits_offer_renewal_choices_and_the_hotel_has_a_building() {
    let (app, _deps, c, _g, _h) = renewal_game().await;
    let p = private(&app, &c).await;
    let limits = &p["plan"]["limits"];
    let rest = &limits["departments"]["restaurant"]["renewal"];
    assert_eq!(rest["month"], "jun");
    assert_eq!(rest["rounds_left"], 5);
    assert_eq!(rest["locked"], Value::Null);
    let options = rest["options"].as_array().unwrap();
    assert_eq!(options.len(), 11, "0, 5.000 … 50.000 €");
    assert_eq!(
        options[2],
        json!({ "amount_cents": 1_000_000, "installment_cents": 200_000, "last_installment_cents": 200_000,
                "gain_pts": 769, "gain_capped_pts": 769, "depreciation_monthly_cents": 11_905 })
    );
    assert!(limits["departments"]["admin"].get("renewal").is_none());
    assert_eq!(
        limits["costs"]["building_renewal"]["options"]
            .as_array()
            .unwrap()
            .len(),
        21
    );
    let guide = &limits["costs"]["renewal_guide"];
    assert_eq!(guide["reserve_bp"], 400);
    assert_eq!(guide["invested_cents"], 0);
    let budget = guide["season_revenue_budget_cents"].as_i64().unwrap();
    assert_eq!(
        guide["guide_cents"].as_i64().unwrap(),
        (budget * 400 + 5_000) / 10_000
    );
    let b = &p["building"];
    assert_eq!(b["wear_pts_per_month"], 50);
    assert_eq!(b["invested_cents"], 0);
    assert_eq!(b["locked"], Value::Null);
    assert!([8_000, 4_500, 6_500].contains(&b["start_pts"].as_i64().unwrap()));
    assert_eq!(b["condition_pts"], b["start_pts"]);
}

#[tokio::test]
async fn v1_hotel_has_no_building_and_no_renewal_fields() {
    let (app, _, _) = common::app();
    let (c, _g, _h) = common::demo(&app).await;
    let p = private(&app, &c).await;
    assert!(p.get("building").is_none());
    let plan = &p["plan"]["plan"];
    assert!(plan.get("building_renewal_cents").is_none());
    assert!(plan.get("departments").is_none());
    let costs = &p["plan"]["limits"]["costs"];
    assert!(costs.get("building_renewal").is_none());
    assert!(costs.get("renewal_guide").is_none());
}

#[tokio::test]
async fn department_and_pl_show_renewal_depreciation_and_ebit() {
    let (app, deps, c, g, h) = renew_in_t01().await;
    let d = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/departments/restaurant"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(d.status, 200, "{:?}", d.body);
    let dept = &d.body["department"];
    assert_eq!(dept["invested_cents"], 800_000);
    assert_eq!(dept["renewal"]["total_cents"], 4_000_000);
    assert!(dept["start_condition_pts"].is_i64());
    assert_eq!(dept["last"]["renewal_cents"], 800_000);
    assert!(dept["last"]["renewal_pts"].as_i64().unwrap() > 0);
    assert!(dept["last"]["felt_condition_pts"].is_i64());

    for round in ["t02", "t03", "t04", "t05", "t06"] {
        close(&app, &deps, &c, &g, round).await;
    }
    let pl = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?round=t06&scope=round"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(pl.status, 200, "{:?}", pl.body);
    let sections: Vec<&str> = pl.body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["section"].as_str().unwrap())
        .collect();
    assert_eq!(
        &sections[sections.len() - 3..],
        ["ebitda", "depreciation", "ebit"]
    );
    let dep = pl.body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["section"] == "depreciation")
        .unwrap();
    let names: Vec<&str> = dep["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Amortizacija opreme", "Amortizacija zgrade"]);
    assert_eq!(dep["total"]["name"], "Ukupna amortizacija");
    // Lipanjska obnova (Restoran 40.000 € / 84, zgrada 100.000 € / 480) amortizira se od srpnja; budžet je 0.
    assert!(dep["total"]["actual_cents"].as_i64().unwrap() > 0);
    assert_eq!(dep["total"]["budget_cents"], 0);
    let ebit = pl.body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["section"] == "ebit")
        .unwrap();
    assert_eq!(
        ebit["lines"][0]["name"],
        "EBIT (dobit iz poslovanja, prije kamata i poreza)"
    );
    assert!(pl.body["ebit_margin_bp"].is_object());
    assert!(pl.body["capex_cents"].is_object());
}

#[tokio::test]
async fn v2_game_created_with_core_2_0_0_is_409_game_outdated() {
    let (app, _deps, mem) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2" })),
    )
    .await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    mem.patch_first_event(&g, |p| p["core_version"] = json!("2.0.0"));
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    common::assert_problem(&r, 409, "game_outdated");
}
