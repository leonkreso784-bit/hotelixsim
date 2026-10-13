### Task 22: Kraj do kraja — cijela v2 sezona, preglednik, dokumenti

**Veličina:** S–M, 2–3 h · **Ovisi o:** sve · **Model:** sonnet (izvođač), preglednik i završni pregled radi voditelj
(opus)

**Cilj:** dokaz da tjedna v2 sezona ide od početka do kraja kroz API, da v1 demo radi kao prije, i zapis odluka.

**Files:**
- Create: `crates/api/tests/e2e_v2.rs`
- Modify: `docs/records/BACKLOG.md`, `docs/records/DECISIONS.md` (novi ADR-029 „Sloj 1: pretpostavke plana" 🟨 s
  P1–P13), korijenski `README.md` („Igra u pregledniku": kako stvoriti v2 igru)
- Ne dirati: kod (osim ako e2e otkrije grešku — tada zaseban popravak uz Ruling), `docs/README.md` (indeks vodi voditelj).

- [ ] **Step 1: E2E test** — `crates/api/tests/e2e_v2.rs`:

```rust
mod common;

use serde_json::json;

#[tokio::test]
async fn weekly_v2_season_from_start_to_end() {
    let (app, _, mem) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "model": "v2" }))).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let rounds: Vec<String> = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await.body["rounds"]
        .as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap().to_owned()).collect();
    assert_eq!(rounds.len(), 17);
    for code in &rounds {
        let s = common::call(&app, "GET", &format!("/v1/hotels/{h}/situations"), Some(&c), None).await;
        for item in s.body["items"].as_array().unwrap() {
            let sc = item["code"].as_str().unwrap();
            common::call(&app, "POST", &format!("/v1/hotels/{h}/situations/{sc}/answer"), Some(&c), Some(json!({ "answer": 0 }))).await;
        }
        let r = common::call(&app, "POST", &format!("/v1/games/{g}/rounds/{code}/close"), Some(&c), None).await;
        assert_eq!(r.status, 202, "{code}");
        common::run_jobs(&app, &mem).await;
    }
    let game = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert!(game.body["current"].is_null());
    let pl = common::call(&app, "GET", &format!("/v1/hotels/{h}/pl?scope=season"), Some(&c), None).await;
    assert_eq!(pl.status, 200);
    assert_eq!(pl.body["days"], 122);
    let kpi = common::call(&app, "GET", &format!("/v1/hotels/{h}/kpi"), Some(&c), None).await;
    assert_eq!(kpi.body["months"].as_array().unwrap().len(), 17);
    let feed = common::call(&app, "GET", &format!("/v1/hotels/{h}/feed"), Some(&c), None).await;
    assert!(feed.body["items"].as_array().unwrap().iter().any(|i| i["round"] == "t17"));
}
```

- [ ] **Step 2:** `cargo test -p api --test e2e_v2; echo EXIT=$?` → zeleno; cijeli workspace i sve provjere (uklj. tri
  retka balansa s `--seeds 1000 --check`, `corepack pnpm check:docs`).
- [ ] **Step 3: BACKLOG** (oznaka `[sloj1]`): situacija s tjednom u mjesecu (🟨 P13); zbirni mjesečni mentor u tjednoj
  igri (🟨 P3); profesorov račun dobiti hotela jedan uz drugoga (P9 v2); naknada za upravljanje i informatika (🟨 P10);
  povišice iz situacija po odjelu (🟨 P12, sloj 2); igre v2 stvorene usred sloja 1 su probne (nisu zaštićene verzijom).
- [ ] **Step 4: ADR-029** „Sloj 1 — pretpostavke plana" 🟨 ČEKA LEONOV PREGLED: tablica P1–P13 iz zaglavlja plana s
  „cijena ako krivo" (taskovi koji se mijenjaju).
- [ ] **Step 5: Commit** „Sloj 1/22: cijela tjedna v2 sezona kroz API; BACKLOG i ADR-029".
- [ ] **Step 6 (voditelj, bez commita):** preglednik bez ključa — v1 demo (390 px i 1280 px: početak, Moj hotel, S1,
  Zatvori mjesec, Lipanj završen) izgleda kao prije; v2 tjedna igra: t01 → Zatvori tjedan → Tjedan 1 završen → Odjeli
  → Restoran → Račun dobiti (krug i sezona) → Plan (zatvori bar, spremi) na 360, 390, 1024 i 1280 px; `scrollWidth ≤
  širina`, konzola bez grešaka. Zatim završni pregled grane (opus) s Review Focus popisom.

---

