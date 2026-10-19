//! L1-13b dio 1 (ADR-042 t. 10, kartica task-15, pretpostavke 1-2): izmiješani ponuđeni odgovori kako ih vidi student.
//! Ugovor API-ja ostaje "broj = položaj u `answers` na ekranu"; izvorni broj (položaj u sadržaju, u kojem je najbolji
//! odgovor prvi) nigdje ne izlazi. v1 (zamrznuti demo) ostaje nepromiješan. Sjeme je 42, igrač je slot 1 (`SeqIds`).
//!
//! Sučelje iz `answer_order.rs` u `game`: `answer_order(seed, slot, code, n)[prikazni_položaj] = izvorni_broj`.
mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use api::{AppState, router};
use app::memory::{FakeAi, deps_with_ai};
use axum::Router;
use common::{assert_problem, call, cookie_value, run_worker};
use content::{Content, Snapshot, repo_dir};
use game::answer_order;
use serde_json::{Value, json};

const SEED: u64 = 42;
const SLOT: u8 = 1;

struct Student {
    app: Router,
    deps: app::Deps,
    c: String,
    game: String,
    hotel: String,
}

impl Student {
    async fn start(body: Option<Value>) -> Student {
        let (deps, _) = deps_with_ai(
            Snapshot::read(&repo_dir()).unwrap(),
            Arc::new(FakeAi::good()),
        );
        let app = router(AppState { deps: deps.clone() });
        let r = call(&app, "POST", "/v1/demo", None, body).await;
        assert_eq!(r.status, 201, "{:?}", r.body);
        Student {
            app,
            deps,
            c: cookie_value(r.set_cookie.as_deref().unwrap()),
            game: r.body["game_id"].as_str().unwrap().into(),
            hotel: r.body["hotel_id"].as_str().unwrap().into(),
        }
    }
    async fn v2(kind: &str) -> Student {
        Self::start(Some(json!({ "model": "v2", "round_kind": kind }))).await
    }
    async fn get(&self, uri: &str) -> Value {
        let r = call(&self.app, "GET", uri, Some(&self.c), None).await;
        assert_eq!(r.status, 200, "{uri}: {:?}", r.body);
        r.body
    }
    async fn items(&self) -> Vec<Value> {
        self.get(&format!("/v1/hotels/{}/situations", self.hotel))
            .await["items"]
            .as_array()
            .unwrap()
            .clone()
    }
    async fn feed(&self) -> Vec<Value> {
        self.get(&format!("/v1/hotels/{}/feed", self.hotel)).await["items"]
            .as_array()
            .unwrap()
            .clone()
    }
    async fn answer(&self, code: &str, position: usize) -> common::Resp {
        call(
            &self.app,
            "POST",
            &format!("/v1/hotels/{}/situations/{code}/answer", self.hotel),
            Some(&self.c),
            Some(json!({ "answer": position })),
        )
        .await
    }
    async fn close(&self, round: &str) {
        let r = call(
            &self.app,
            "POST",
            &format!("/v1/games/{}/rounds/{round}/close", self.game),
            Some(&self.c),
            None,
        )
        .await;
        assert_eq!(r.status, 202, "{round}: {:?}", r.body);
        run_worker(&self.deps).await;
    }
}

fn labels_of(item: &Value) -> Vec<String> {
    item["answers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap().to_owned())
        .collect()
}

/// Oznake i kvalitete ponuđenih odgovora u SADRŽAJU (izvorni redoslijed), po šifri primjerka.
fn original(c: &Content, code: &str) -> (Vec<String>, Vec<u8>) {
    let template = code.split('.').next().unwrap();
    if let Some(t) = c
        .situation_templates
        .as_ref()
        .and_then(|s| s.templates.iter().find(|t| t.code == template))
    {
        return (
            t.answers.iter().map(|a| a.label.clone()).collect(),
            t.answers.iter().map(|a| a.quality()).collect(),
        );
    }
    let s = c
        .situations
        .iter()
        .find(|s| s.code == code)
        .unwrap_or_else(|| panic!("nema {code} u sadržaju"));
    (
        s.answers.iter().map(|a| a.label.clone()).collect(),
        s.answers.iter().map(|a| a.quality).collect(),
    )
}

fn v2_content() -> Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

/// Izvorni broj najboljeg odgovora (prvi s najvišom kvalitetom).
fn best_original(qualities: &[u8]) -> usize {
    let q = *qualities.iter().max().unwrap();
    qualities.iter().position(|&x| x == q).unwrap()
}

/// Oznaka bez rezerviranog mjesta (`{mjestima}` i sl.): takva je na ekranu doslovno ista kao u sadržaju.
fn literal(label: &str) -> bool {
    !label.contains('{')
}

/// Prikazni položaj najboljeg odgovora, nađen SAMO po oznaci (bez `answer_order`); `None` ako oznaka ima rezervirano mjesto.
fn best_position_by_label(item: &Value, c: &Content) -> Option<usize> {
    let code = item["code"].as_str().unwrap();
    let (orig_labels, quals) = original(c, code);
    let best = &orig_labels[best_original(&quals)];
    if !literal(best) {
        return None;
    }
    labels_of(item).iter().position(|l| l == best)
}

/// Sve situacije mjesečne sezone (jun-sep) kako ih student vidi u svom krugu; krugove zatvara bez odgovora.
async fn season_items(s: &Student) -> Vec<Value> {
    let mut all = Vec::new();
    for round in ["jun", "jul", "aug", "sep"] {
        all.extend(s.items().await);
        s.close(round).await;
    }
    all
}

#[tokio::test]
async fn best_answer_is_not_always_first_and_position_varies() {
    let c = v2_content();
    let s = Student::v2("month").await;
    let positions: Vec<usize> = season_items(&s)
        .await
        .iter()
        .filter_map(|it| best_position_by_label(it, &c))
        .collect();
    assert!(
        positions.len() >= 10,
        "premalo stavki za provjeru: {}",
        positions.len()
    );
    assert!(
        positions.iter().any(|&p| p != 0),
        "najbolji odgovor je u svim stavkama prvi: {positions:?}"
    );
    let distinct: BTreeSet<_> = positions.iter().collect();
    assert!(
        distinct.len() >= 3,
        "položaj najboljeg nije raznolik ({distinct:?}): {positions:?}"
    );
    let first = positions.iter().filter(|&&p| p == 0).count();
    assert!(
        first * 2 < positions.len(),
        "najbolji je prvi u {first} od {} stavki",
        positions.len()
    );
}

#[tokio::test]
async fn weekly_season_also_shuffles() {
    let c = v2_content();
    let s = Student::v2("week").await;
    let mut positions = Vec::new();
    for round in ["t01", "t02", "t03", "t04", "t05", "t06"] {
        positions.extend(
            s.items()
                .await
                .iter()
                .filter_map(|it| best_position_by_label(it, &c)),
        );
        s.close(round).await;
    }
    assert!(positions.len() >= 6, "{positions:?}");
    assert!(positions.iter().any(|&p| p != 0), "{positions:?}");
}

#[tokio::test]
async fn two_gets_and_two_games_with_the_same_seed_give_the_same_order() {
    let (a, b) = (Student::v2("month").await, Student::v2("month").await);
    let first = a.items().await;
    assert!(!first.is_empty());
    assert_eq!(first, a.items().await, "dva GET-a istog igrača");
    assert_eq!(
        first,
        b.items().await,
        "isto sjeme, isti slot: isti redoslijed"
    );
}

#[tokio::test]
async fn shown_order_is_the_seed_slot_code_permutation_of_the_content() {
    let c = v2_content();
    let s = Student::v2("month").await;
    let (mut compared, mut slot_matters) = (0, false);
    for it in season_items(&s).await {
        let code = it["code"].as_str().unwrap();
        let shown = labels_of(&it);
        let (orig, _) = original(&c, code);
        let order = answer_order(SEED, SLOT, code, shown.len());
        assert_eq!(shown.len(), orig.len(), "{code}: broj odgovora");
        for (pos, &o) in order.iter().enumerate() {
            let want = &orig[usize::from(o)];
            if literal(want) {
                assert_eq!(&shown[pos], want, "{code}: položaj {pos} je izvorni {o}");
                compared += 1;
            }
        }
        slot_matters |= order != answer_order(SEED, 2, code, shown.len());
    }
    assert!(compared >= 30, "premalo usporedbi: {compared}");
    assert!(
        slot_matters,
        "slot 2 ima isti redoslijed kao slot 1 u svim situacijama"
    );
}

#[tokio::test]
async fn click_on_shown_position_records_the_matching_original_answer() {
    let c = v2_content();
    let s = Student::v2("month").await;
    let mut moved = 0;
    let mut answered = Vec::new();
    for it in s.items().await {
        let code = it["code"].as_str().unwrap().to_owned();
        let Some(pos) = best_position_by_label(&it, &c) else {
            continue;
        };
        let r = s.answer(&code, pos).await;
        assert_eq!(r.status, 200, "{code}: {:?}", r.body);
        let shown = r.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["code"] == code.as_str())
            .unwrap()
            .clone();
        assert_eq!(shown["chosen"], pos, "{code}: chosen je prikazni položaj");
        let (orig, quals) = original(&c, &code);
        answered.push((code, pos, orig[best_original(&quals)].clone()));
        moved += usize::from(pos != 0);
    }
    assert!(answered.len() >= 3);
    assert!(
        moved >= 1,
        "nijedan najbolji nije izvan položaja 0: izvorni i prikazni broj se ne razlikuju"
    );
    let feed = s.feed().await;
    for (code, pos, best_label) in &answered {
        let f = feed
            .iter()
            .find(|f| f["kind"] == "answer_chosen" && f["situation"] == code.as_str())
            .unwrap_or_else(|| panic!("{code}: nema u dnevniku"));
        assert_eq!(
            f["label"],
            best_label.as_str(),
            "{code}: dnevnik pokazuje oznaku najboljeg (izvornog) odgovora"
        );
        // Dnevnik ne smije odati izvorni broj: polje `answer`, ako ga ima, je prikazni položaj.
        if let Some(a) = f.get("answer") {
            assert_eq!(
                a,
                &json!(pos),
                "{code}: dnevnik nosi izvorni broj umjesto prikaznog"
            );
        }
    }
}

#[tokio::test]
async fn every_shown_position_records_exactly_that_label() {
    // Zadnji položaj (ne najbolji): oznaka u dnevniku je oznaka s tog mjesta na ekranu.
    let s = Student::v2("month").await;
    for it in s.items().await.iter().take(6) {
        let code = it["code"].as_str().unwrap();
        let shown = labels_of(it);
        let pos = shown.len() - 1;
        let r = s.answer(code, pos).await;
        assert_eq!(r.status, 200, "{code}: {:?}", r.body);
        let f = s
            .feed()
            .await
            .into_iter()
            .find(|f| f["kind"] == "answer_chosen" && f["situation"] == code)
            .unwrap();
        assert_eq!(f["label"], shown[pos].as_str(), "{code}");
    }
}

#[tokio::test]
async fn displayed_number_outside_range_is_unknown_answer() {
    let s = Student::v2("month").await;
    let it = s.items().await.remove(0);
    let code = it["code"].as_str().unwrap();
    let n = labels_of(&it).len();
    for bad in [n, n + 1, 200] {
        let r = s.answer(code, bad).await;
        assert_problem(&r, 422, "unknown_answer");
    }
    // Nakon grešaka situacija je i dalje neodgovorena i valjani položaj prolazi.
    let ok = s.answer(code, n - 1).await;
    assert_eq!(ok.status, 200, "{:?}", ok.body);
}

#[tokio::test]
async fn closed_round_with_number_out_of_range_is_409_not_422() {
    let s = Student::v2("month").await;
    let code = s.items().await[0]["code"].as_str().unwrap().to_owned();
    s.close("jun").await;
    assert_problem(&s.answer(&code, 200).await, 409, "round_not_open");
}

#[tokio::test]
async fn frozen_v1_demo_keeps_content_order() {
    let c = Snapshot::read(&repo_dir()).unwrap().build().unwrap();
    let s = Student::start(None).await;
    let items = s.items().await;
    assert!(!items.is_empty());
    for it in &items {
        let code = it["code"].as_str().unwrap();
        let (orig, _) = original(&c, code);
        assert_eq!(labels_of(it), orig, "{code}: v1 ostaje nepromiješan");
    }
    let code = items[0]["code"].as_str().unwrap();
    let last = labels_of(&items[0]).len() - 1;
    let r = s.answer(code, last).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let f = s
        .feed()
        .await
        .into_iter()
        .find(|f| f["kind"] == "answer_chosen" && f["situation"] == code)
        .unwrap();
    assert_eq!(f["label"], labels_of(&items[0])[last].as_str());
    if let Some(a) = f.get("answer") {
        assert_eq!(a, &json!(last));
    }
}
