### Task 11: `server` — radnik u procesu, brana nepraćenih generiranih datoteka, upute

**Files:**
- Modify: `crates/server/src/main.rs`, `Cargo.toml` (workspace tokio značajka `time`)
- Create: `scripts/check-clean.mjs`; Modify: `package.json` (`check:api`)
- Modify: `README.md` („Lokalni razvoj": sadržaj, radnik), `.env.example` (`HOTELIXSIM_CONTENT` zakomentiran)
- Modify: `docs/records/BACKLOG.md` (briše se stavka „`check:api` ne hvata nepraćenu generiranu datoteku")

**Interfaces:**
- Consumes: Task 6 (`app::run_next_job`), Task 9 (`deps`, `content_snapshot`).
- Produces: `serve` pokreće radnika kao tokio zadatak; `node scripts/check-clean.mjs <putanje…>` → izlaz 1 ako
  `git status --porcelain -- <putanje>` nije prazan.

- [ ] **Step 1: Radnik.** Workspace `tokio` značajke + `"time"`. U `main.rs`:

```rust
/// Radnik reda poslova u istom procesu (demo; zaseban način `work` je za pilot). Pad procesa ne gubi posao:
/// zakup istekne i posao se preuzme ponovno (ARHITEKTURA §9.4).
fn spawn_worker(deps: app::Deps) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match app::run_next_job(&deps).await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
                Err(e) => {
                    tracing::warn!(error = %e, "posao reda nije uspio");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    })
}
```
  U `serve()`: `let deps = deps(store, snapshot); let worker = spawn_worker(deps.clone());` prije `axum::serve`, a
  nakon gašenja `worker.abort();`.

- [ ] **Step 2: Brana.** `scripts/check-clean.mjs`:

```js
// check:api: generirane datoteke moraju biti praćene i nepromijenjene. `git diff --exit-code` ne vidi
// NOVU nepraćenu datoteku (BACKLOG [dnevnik], Task 6 #2 kostura); `git status --porcelain` je vidi.
import { execFileSync } from "node:child_process";

const paths = process.argv.slice(2);
if (paths.length === 0) {
  console.error("upotreba: node scripts/check-clean.mjs <putanja>…");
  process.exit(2);
}
const out = execFileSync("git", ["status", "--porcelain", "--", ...paths], { encoding: "utf8" });
if (out.trim() !== "") {
  console.error(`generirane datoteke nisu commitane:\n${out}`);
  process.exit(1);
}
console.log(`čisto: ${paths.join(", ")}`);
```
  `package.json`: `"check:api": "node scripts/pnpm.mjs gen:api && node scripts/check-clean.mjs openapi.json packages/api-client/src"`.
  Proba: `touch packages/api-client/src/proba.ts; node scripts/check-clean.mjs openapi.json packages/api-client/src; echo EXIT=$?`
  → ispis `?? packages/api-client/src/proba.ts`, `EXIT=1`; `rm packages/api-client/src/proba.ts`.

- [ ] **Step 3: Upute.** README „Lokalni razvoj", korak 3 dopuni: „`serve` pri startu učitava i provjerava
  `content/hotel` (ili mapu iz `HOTELIXSIM_CONTENT`) i pokreće radnika koji obračunava zatvorene mjesece; migracije se
  ne pokreću same — uvijek `migrate` prije `serve`." `.env.example`: `# HOTELIXSIM_CONTENT=content/hotel`.

- [ ] **Step 4: Pokreni** `cargo build -p server; echo EXIT=$?` → `EXIT=0`; `corepack pnpm check:api; echo EXIT=$?` →
  `čisto: …`, `EXIT=0` (nakon commita Taska 10); provjere iz Global Constraints.

- [ ] **Step 5: Commit** — `crates/server Cargo.toml Cargo.lock scripts/check-clean.mjs package.json README.md
  .env.example docs/records/BACKLOG.md`, poruka „Dnevnik 11/12: radnik u serveru, check:api vidi nepraćene datoteke".

---

