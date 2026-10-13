### Task 22: Kraj do kraja — cijela v2 sezona, preglednik, bilješke za dokumente

**Veličina:** S–M, 2–3 h · **Ovisi o:** sve · **Model:** sonnet (izvođač), preglednik i završni pregled radi voditelj
(opus)

**Cilj:** dokaz da tjedna v2 sezona ide od početka do kraja kroz API, da v1 demo radi kao prije, i zapis odluka —
**bez diranja `docs/` i `README.md`** (paralelna sesija preuređuje dokumentaciju; preflight Ruling 3): sve što ide u
dokumente piše se u `.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md`.

**Files:**
- Create: `crates/api/tests/e2e_v2.rs`
- Modify (dopiši na kraj, postojeće stavke ne briši): `C:/Users/leonk/Documents/hotelixsim/.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md`
  — apsolutna putanja u glavnom stablu (mapa `.superpowers/sdd/` je u `.gitignore`, pa se **ne commita** i ne postoji
  u radnom stablu taska)
- Ne dirati: **`docs/**` i korijenski `README.md`** (ni `docs/records/*`, ni `docs/README.md`), kod (osim ako e2e otkrije
  grešku — tada zaseban popravak uz Ruling).

- [ ] **Step 1: E2E test** — `crates/api/tests/e2e_v2.rs`:

```rust
mod common;

use serde_json::json;

#[tokio::test]
async fn weekly_v2_season_from_start_to_end() {
    let (app, deps, _) = common::app();
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
        common::run_worker(&deps).await; // Task 7: `crates/api/tests/common/mod.rs`
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
  retka balansa s `--seeds 1000 --check`, i `corepack pnpm check:docs` — `docs/` se ne mijenja, pa mora ostati zelen
  kao prije taska).
- [ ] **Step 3: Bilješke za dokumente** — dopiši u `za-dokumentaciju.md` odjeljak `## Sloj 1 (Task 22)` s pet
  pododjeljaka (svaki: kamo ide, tekst spreman za prijenos, izvor):
  1. **BACKLOG** (oznaka `[sloj1]`): situacija s tjednom u mjesecu (🟨 P13); zbirni mjesečni mentor u tjednoj igri
     (🟨 P3 — sada AI mentor gleda samo zadnji krug mjeseca); profesorov račun dobiti hotela jedan uz drugoga (🟨 P17);
     „Zašto" po odjelu (🟨 P18); naknada za upravljanje i informatika (🟨 P10); povišice iz situacija po odjelu (🟨 P12,
     sloj 2); igre v2 stvorene usred sloja 1 su probne (nisu zaštićene verzijom).
  2. **ADR-029** „Sloj 1 — pretpostavke plana" 🟨 ČEKA LEONOV PREGLED: kontekst (plan
     `docs/plan/2026-10-10-v2-sloj1.md`), tablica **P5–P18** iz zaglavlja plana, svaka s „cijena ako krivo" (taskovi koji
     se mijenjaju). **P1–P4 nisu u tablici kao 🟨** — to su Leonove odluke ✅ iz ADR-028 (odgovori 3, 4, 5 i 9); ADR-029
     ih samo navodi rečenicom s poveznicom na ADR-028. Zaseban pododjeljak „Svjesna odstupanja od MODEL_V2": `?round=`
     umjesto `?period=` (§6.5, P14); vrsta kruga u postavkama igre, ne u snimku (§6.2 t. 1, §11 t. 6, P15); `D₀`
     umjesto „/30" (§4.2/§4.4, P16); profesorov račun dobiti odgođen (§8 sloj 1, P17); bez „Zašto" po odjelu (§8, P18);
     tjedni mentor samo „tri uzroka" (preflight Ruling 4, P3).
  3. **CHANGELOG** (unos za sloj 1): što je gotovo — v2 jezgra (tjedan i mjesec), odjeli po USALI-ju, račun dobiti s
     budžetom, API rute (`/departments`, `/departments/{code}`, `/pl`), web ekrani Odjeli / Račun dobiti / plan po
     odjelima, balans v2 (tjedno i mjesečno), v1 zlatni test; gdje se vidi (lokalno, grana `model-v2/sloj1`).
  4. **MODEL_V2** (dopune za voditelja dokumentacije): §6.5 ime upita `round` (P14); §6.2/§11 gdje se sprema vrsta
     kruga (P15); §4.2/§4.4 `D₀` (P16); §8 sloj 1 — što je odgođeno (P17, P18) i provjera „C: ostali ≈ 30 % prihoda
     soba" u balansu (Task 16, pojas 25–35 %); §3.2 održavanje u „Održavanju" (P9) i bez naknade za upravljanje (P10).
  5. **README** („Igra u pregledniku"): kako stvoriti v2 igru — na početnom ekranu „Za profesora: postavke nove igre" →
     „Novi model: odjeli i račun dobiti" → „Po tjednima (preporučeno)" ili „Po mjesecima"; kroz API
     `POST /v1/demo` s tijelom `{ "model": "v2" }` (tjedno) ili `{ "model": "v2", "round_kind": "month" }`; bez tijela
     ostaje v1 demo lipanj–srpanj.
- [ ] **Step 4: Commit** „Sloj 1/22: cijela tjedna v2 sezona kroz API" — samo `crates/api/tests/e2e_v2.rs` (i popravci
  uz Ruling, ako ih je bilo); `za-dokumentaciju.md` se ne commita (`.gitignore`), u izvještaju taska navedi da je
  dopisan.
- [ ] **Step 5 (voditelj, bez commita):** preglednik bez ključa — v1 demo (390 px i 1280 px: početak, Moj hotel, S1,
  Zatvori mjesec, Lipanj završen) izgleda kao prije; v2 tjedna igra: t01 → Zatvori tjedan → Tjedan 1 završen → Odjeli
  → Restoran → Račun dobiti (krug i sezona) → Plan (zatvori bar, spremi) na 360, 390, 1024 i 1280 px; `scrollWidth ≤
  širina`, konzola bez grešaka. Zatim završni pregled grane (opus) s Review Focus popisom. Prijenos bilješki iz
  `za-dokumentaciju.md` u `docs/` radi voditelj tek nakon spajanja grane dokumentacije.

---

