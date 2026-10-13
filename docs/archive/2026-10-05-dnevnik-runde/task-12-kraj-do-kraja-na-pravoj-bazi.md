### Task 12: Kraj do kraja na pravoj bazi, regeneracija i sve provjere CI-ja

**Files:**
- Create: `crates/api/tests/e2e.rs`; Modify: `crates/api/Cargo.toml` (dev: `sqlx.workspace = true`, `store.workspace = true`)
- Modify: `docs/architecture/ARHITEKTURA.md` (§6.1 „Postoje:", §6.2 dijagram, §9.1 `actor`, §9.4 izbor izvedbe)
- Modify: `docs/records/BACKLOG.md` (obriši preostale riješene `[dnevnik]` stavke; dodaj odgođeno — Step 6)
- Modify: `.sqlx/`, `openapi.json`, `packages/api-client/src/schema.d.ts` (ako se promijene)

**Interfaces:**
- Consumes: sve prethodno; `store::Store::from_pool`, `app::RandomIds`, `game::game::Game`.
- Produces: dokaz da cijeli put radi nad Postgresom; nijedan novi javni tip.

- [ ] **Step 1: Test kraja do kraja.** `crates/api/tests/e2e.rs`:

```rust
mod common;
use std::sync::Arc;

use api::{AppState, router};
use common::{assert_problem, call};
use content::{Snapshot, repo_dir};
use serde_json::json;
use sqlx::PgPool;

fn deps(pool: PgPool) -> app::Deps {
    let s = Arc::new(store::Store::from_pool(pool));
    app::Deps { status: s.clone(), log: s.clone(), guests: s.clone(), jobs: s,
                ids: Arc::new(app::RandomIds), snapshot: Arc::new(Snapshot::read(&repo_dir()).unwrap()) }
}

async fn run_worker(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

#[sqlx::test(migrations = "../store/migrations")]
async fn guest_plays_june_and_july_against_three_robots(pool: PgPool) {
    let d = deps(pool.clone());
    let app = router(AppState { deps: d.clone() });
    // gost → igra
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = r.set_cookie.unwrap().split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    let (game, hotel) = (r.body["game_id"].as_str().unwrap().to_owned(), r.body["hotel_id"].as_str().unwrap().to_owned());
    // lipanj: plan + jedan odgovor, S3 ostaje neodgovorena
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["marketing_cents"] = json!(500_000);
    assert_eq!(call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/jun"), Some(&tok), Some(plan)).await.status.as_u16(), 200);
    call(&app, "POST", &format!("/v1/hotels/{hotel}/situations/S1/answer"), Some(&tok), Some(json!({"answer": 0}))).await;
    assert_eq!(call(&app, "GET", &format!("/v1/games/{game}/rounds/jun"), Some(&tok), None).await.body["unanswered"], 1);
    assert_eq!(call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await.status.as_u16(), 202);
    assert_eq!(call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await.status.as_u16(), 200);
    run_worker(&d).await;
    // srpanj
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(g.body["current"], json!({"month": "jul", "status": "open", "unanswered": 1}));
    let s = call(&app, "GET", &format!("/v1/hotels/{hotel}/situations"), Some(&tok), None).await;
    assert_eq!(s.body["items"][0]["code"], "S2");
    call(&app, "POST", &format!("/v1/games/{game}/rounds/jul/close"), Some(&tok), None).await;
    run_worker(&d).await;
    // ljestvica i feed
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert!(g.body["current"].is_null(), "igra je gotova");
    for h in g.body["hotels"].as_array().unwrap() {
        assert_eq!(h["months"].as_array().unwrap().len(), 2, "{h}");
    }
    let f = call(&app, "GET", &format!("/v1/hotels/{hotel}/feed"), Some(&tok), None).await;
    let resolved = f.body["items"].as_array().unwrap().iter().filter(|i| i["kind"] == "month_resolved").count();
    assert_eq!(resolved, 2);
    assert_problem(&call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/jul"), Some(&tok),
        Some(me.body["private"]["plan"]["plan"].clone())).await, 409, "round_not_open");
    // baza: dva obračuna, nijedan posao nije ostao
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE type = 'RoundResolved'").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 2);
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status <> 'done'").fetch_one(&pool).await.unwrap();
    assert_eq!(open, 0);
}

#[sqlx::test(migrations = "../store/migrations")]
async fn replay_from_database_matches(pool: PgPool) {
    use app::EventLog;
    use game::game::Game;
    let d = deps(pool);
    let app = router(AppState { deps: d.clone() });
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = r.set_cookie.unwrap().split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    let game = app::GameId(r.body["game_id"].as_str().unwrap().parse().unwrap());
    call(&app, "POST", &format!("/v1/games/{}/rounds/jun/close", game.0), Some(&tok), None).await;
    run_worker(&d).await;
    // pročitaj iz baze (JSONB je promijenio redoslijed ključeva), obračunaj lipanj ponovno i usporedi tipizirano
    let raw = d.log.load(game).await.unwrap();
    let recs: Vec<game::Recorded> = raw
        .iter()
        .map(|r| game::Recorded {
            seq: r.seq,
            actor: game::Actor::parse(&r.actor).unwrap(),
            event: game::GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone()).unwrap(),
        })
        .collect();
    let at = recs.iter().position(|r| r.event.kind() == "RoundResolved").unwrap();
    let again = Game::fold(&recs[..at]).unwrap().resolve_round(sim_hotel::Month::Jun).unwrap();
    let game::GameEvent::RoundResolved(stored) = &recs[at].event else { panic!() };
    assert_eq!(serde_json::to_vec(&again).unwrap(), serde_json::to_vec(stored).unwrap());
}
```
  (dev-ovisnosti `api`: `game`, `sim-hotel` ako ih test ne vidi; `common/mod.rs` je iz Taska 9.)

- [ ] **Step 2: Pokreni** `bash scripts/baza.sh status`; `cargo test -p api --test e2e; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 3: Regeneracija.** `set -a; source .env; set +a; cargo sqlx migrate run --source crates/store/migrations; echo EXIT=$?`;
  `SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`; `corepack pnpm gen:api; echo EXIT=$?` — svi `EXIT=0`.

- [ ] **Step 4: Sve provjere CI-ja, lokalno, svaka zasebno:** `cargo fmt --all -- --check` · `cargo clippy --workspace
  --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` ·
  `cargo sqlx prepare --workspace --check` · `cargo test --workspace` · `cargo run -q -p balance --release -- --seeds 100 --check` ·
  `corepack pnpm check:docs` · `corepack pnpm test:docs` · `corepack pnpm check:api` (nakon commita) · `corepack pnpm lint` ·
  `corepack pnpm typecheck` · `corepack pnpm test` — svaka s `; echo EXIT=$?`, sve `EXIT=0`.

- [ ] **Step 5: Ručna proba pravog procesa** (radnik u serveru, kolačić kroz HTTP). Prvi terminal (pozadina):
  `set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`. Drugi:

```bash
J=/tmp/hx-jar; rm -f $J
curl -s -c $J -X POST http://127.0.0.1:8080/v1/demo                      # {"game_id":…,"hotel_id":…}
G=$(curl -s -b $J http://127.0.0.1:8080/v1/me/hotel | node -pe 'JSON.parse(require("fs").readFileSync(0)).game_id')
curl -s -b $J -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8080/v1/games/$G/rounds/jun/close   # 202
sleep 1; curl -s -b $J http://127.0.0.1:8080/v1/games/$G/rounds/jun       # "status":"resolved"
```
  Očekivano: 202, pa `resolved` unutar sekunde (radnik radi u procesu). Zaustavi server (Ctrl+C).

- [ ] **Step 6: Dokumenti.** ARHITEKTURA: §6.1 „Postoje:" dodaj `game`; §6.2 dijagram — `game` između `sim-hotel`/`content`
  i `app` (`app ──► game ──► sim-hotel, content`); §9.1 redak `actor` → `text: guest:<uuid> · robot:<strategija> · system
  (CHECK u bazi)` i rečenica da nepromjenjivost sad čuva okidač, a uloga bez prava dolazi prije Railwaya; §9.4 na kraj:
  „Za demo izabrana vlastita tablica `jobs` (ADR-015); probni test s apalis/pgmq i zakašnjeli AI rezultat su korak 4."
  BACKLOG: obriši preostale riješene `[dnevnik]` stavke (`stream_sequence_is_unique` preslab, oblik tablice `events`);
  dodaj odgođeno:
  - `[drugi val]` Limit 3 demo igre po IP-u na dan i brisanje gosta i igre nakon 7 dana (DEMO §1.4; Leon 05.10.).
  - `[prije Railwaya]` Uloga baze bez prava UPDATE/DELETE nad `events` (okidač ostaje); kolačić `Secure` uz HTTPS;
    `content::repo_dir()` je put iz vremena prevođenja — na Railwayu `HOTELIXSIM_CONTENT`; zaseban način `work`.
  - `[dnevnik]` → korak 4: zaglavlje `Idempotency-Key` (ARHITEKTURA §11); ponovni pokušaj s odgodom; probni test
    apalis/pgmq (ARHITEKTURA §9.4); zakašnjela AI presuda za zatvorenu rundu (§8.3).
  - `[ekrani]` Posao obračuna u statusu `failed` ostavlja rundu u „closing" zauvijek — ekran i profesor trebaju to
    vidjeti; SSE „runda je obračunata" umjesto ispitivanja (ARHITEKTURA §11).

- [ ] **Step 7: Commit** — `crates/api .sqlx openapi.json packages/api-client/src/schema.d.ts docs/architecture/ARHITEKTURA.md
  docs/records/BACKLOG.md`, poruka „Dnevnik 12/12: kraj do kraja na pravoj bazi, regeneracija i dokumenti".

---

