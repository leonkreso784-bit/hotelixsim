# Sljedeća sesija — HotelixSim: jezgra igre i alat za balans

> Zapisano 2026-10-04 na kraju sesije. Radimo u **Claude Code terminalu** (Windows: Git Bash + PowerShell).
> Repo: `C:\Users\leonk\Documents\hotelixsim` · GitHub: `leonkreso784-bit/hotelixsim` (privatni).

## 1 · Prvo pročitaj — samo ovo i ciljano

1. `CLAUDE.md` (učita se sam) i projektna memorija (`MEMORY.md`, učita se sama — vidi `radi-bez-pitanja`).
2. Memoriju o Leonu (samo čitanje) iz `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\`:
   `tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md`.
3. `docs/DEMO.md` — dizajn igre (§2–§11). **Ovo je spec sljedećeg plana.**
4. `docs/ARHITEKTURA.md` v2 — samo §0, §6.2, §7 i §9.5 kad zatrebaju.
5. `docs/PLAN.md` — faze; DEMO korak 1 (kostur) je gotov.
6. `git status` i `git log --oneline -15`.

## 2 · Gdje smo (04.10. navečer)

**Gotovo i spojeno lokalno u `main`** (push na GitHub čeka Leonov OK — vidi §5):

- Arhitektura v2, `DEMO.md` (dizajn igre + kategorije hotela), tri istraživanja u `docs/istrazivanje/`.
- **Kostur od kraja do kraja** po planu `docs/superpowers/plans/2026-10-04-kostur.md`, 8 taskova, svaki s pregledom,
  plus završni pregled cijele grane (opus) i jedan krug završnih popravaka:
  - Rust workspace: `engine` (kocka SplitMix64, zlatni vektori), `sim-hotel` (5 indeksa u stotinkama, profili A/B/C
    kao konstante), `app` (port `SystemStatus`), `store` (dnevnik u Postgresu, SQLx 0.9, offline `.sqlx/`),
    `api` (Axum 0.8 + utoipa 6, OpenAPI 3.1), `server` (serve | migrate | openapi).
  - `openapi.json` → generirani TS klijent `@hotelixsim/api-client` → `packages/i18n` → jedan React ekran `apps/web`.
  - Brane: clippy (HashMap/HashSet/sat, float/arith lintovi), `scripts/check-core-deps.mjs`, cargo-deny, CI datoteka
    (`.github/workflows/ci.yml`, **još nikad nije radila** jer ništa nije pushano).
  - Testovi: Rust 28, sučelje 10. Lokalni prolaz od kraja do kraja dokazan (zdravlje 200, profil D → 404,
    kriva lozinka baze → 503 bez pada, Vite proxy).

**Lokalna baza:** zaseban PostgreSQL 16 klaster u `C:\Users\leonk\AppData\Local\hotelixsim\pgdata`, **priključak 5433**
(sistemski PostgreSQL na 5432 se ne dira — Leon ne zna njegovu lozinku). **Ne pali se sam nakon restarta:**
`bash scripts/baza.sh start`. Pokretanje svega: README → „Lokalni razvoj".

**Pixel Agents** (VS Code dodatak za prikaz agenata): ured 64×64 sa sobama i 60 radnih mjesta,
`watchAllSessions` uključen. Kopije starih rasporeda: `C:\Users\leonk\.pixel-agents\backup-2026-10-04\`.

## 3 · Cilj sljedeće sesije: plan „jezgra i alat za balans" (PLAN.md, DEMO korak 2)

1. **Napiši plan** (`superpowers:writing-plans`) iz `DEMO.md` §2–§11. **Task 1 plana je OBAVEZNO ojačavanje brana
   determinizma**, prije ijedne linije jezgre (završni pregled kostura, Final #3 — brane su slabije od ARHITEKTURA
   §6.2/§7.2):
   - `check-core-deps.mjs`: dopušteni popis (allowlist) umjesto zabranjenog — jezgra smije ovisiti samo o `engine`,
     `sim-hotel` i izričito odobrenim paketima (danas je prošao `indexmap` s nasumičnim sjemenom);
   - `clippy.toml`: dodati `std::hash::RandomState` i `std::collections::hash_map::RandomState`;
   - `lib.rs`: `#![deny]` → `#![forbid]` + `clippy::as_conversions` (ili `cast_*`), zabrana `f32`/`f64` (provjeriti
     mutacijom), razmotriti `std::env::var`;
   - probe mutacija kao skripta koja se vrti u CI-u (svaka mora pasti).
2. Ulazi za dizajn jezgre (iz završnog pregleda): imenovana i ograničavajuća izgradnja `IndexSet` (ne pozicijski
   niz), profili u `content/hotel/` s A/B/C kao testnim proročanstvom, kategorija hotela (DEMO §2.0) kao podatak,
   API pozivi kroz `app` slučajeve upotrebe (danas `api` zove `sim-hotel` izravno), enum kodovi u OpenAPI-ju.
3. Gradnja kroz `superpowers:subagent-driven-development`, kao danas: grana, ledger u `.superpowers/sdd/`,
   pregled nakon svakog taska, završni pregled na najjačem modelu.
4. Prvi zadatak alata za balans: kalibracija ekonomije (DEMO §2.1 — kolovoz daje ~57 % GOP marže, cilj 30–40 %).

Kasniji planovi (redom iz PLAN.md): dnevnik/runde/red poslova (uz to oblik tablice `events`: globalni redoslijed,
tip `actor`, indeks na `game_id` — **prije Railwaya**), AI sloj, ekrani igrača, profesorov pogled.

## 4 · Odluke koje sam donio umjesto Leona (04.10.) — za njegov pregled

Svaka ima cijenu ako je kriva; sve su u gitu i mogu se vratiti.

1. Gradnja na grani `gradnja/kostur` u istom direktoriju (ne worktree). Cijena: nikakva.
2. **Zasebna baza na 5433** umjesto resetiranja lozinke sistemskog PostgreSQL-a. Cijena: obrisati `pgdata`.
3. Brana ovisnosti mjeri i build-ovisnosti, sve targete i značajke (popravak nakon pregleda). Cijena: sporija brana.
4. `cargo fmt` nad kodom iz plana (plan je pisan preširoko). Cijena: jedan commit formatiranja.
5. `scripts/pnpm.mjs` jer `pnpm` nije na PATH-u (radi i lokalno i u CI-ju). Cijena: zamijeniti ako se uvede samostalni pnpm.
6. Ekran pokazuje grešku indeksa, provjereni ključevi i interpolacija u i18n (popravak nakon pregleda).
7. README i `.gitattributes` popravljeni da rade na svježem Windows stroju (LF svugdje, `SQLX_OFFLINE`).
8. Ojačavanje brana determinizma odgođeno na **prvi task sljedećeg plana** (ne blokira, jezgra je još prazna).
9. Sitni nalazi pregleda (~40) zapisani kao ulaz za sljedeće planove, ne popravljeni sad.

## 5 · Otvoreno za Leona (pitaj na početku, kratko)

1. **Push na GitHub** — `main` je ispred `origin/main` za više commitova. Treba izričit OK. Prvi push pokreće CI
   (prvi put ikad) — gledati rezultat.
2. **Pitanja o domeni iz DEMO.md** (kraj dokumenta), prije gradnje jezgre:
   - 3: 30 stalnih + 20 sezonaca na 120 soba (4*) i trošak zaposlenog ~2 200 €/mj — realno?
   - 4: 25 % popusta agenciji za grupu u kolovozu — realna ponuda?
   - 5: ostali prihod (hrana, piće, usluge) = 30 % prihoda soba — realno?
   - 6: priče profila A i B — prepoznaje li takve hotele?
   - (1 i 2 riješeni 04.10.: cijene i osjetljivost po kategoriji, demo svi 4*.)
3. **AI ključ za demo:** koje besplatne ključeve ima (pružatelj i model). Ne blokira jezgru.
4. Pitanja za profesore (`docs/PITANJA_ZA_PROFESORE.md`) — pregled prije slanja.

## 6 · Pravila (sažetak; puna su u CLAUDE.md i memoriji)

- Hrvatski, kratko, bez žargona; pojam koji možda ne zna objasni od početka (npr. 04.10. nije znao što je lozinka
  PostgreSQL-a — riješeno bez njega).
- **Radi bez pitanja** (Leon, 04.10.): pretpostavke zapiši u dokument; ankete samo za proizvodne odluke.
  **Push i deploy samo uz izričit OK.**
- Nikad „gotovo" ili „zeleno" prije viđenog izlaza; uz svako „gotovo" reci gdje se vidi.
- Temelj prije brzine. Kratke dionice, javi se na prirodnim granicama.
- Model po poslu: sonnet za izvođače i preglede taskova, haiku za kratke ponovne preglede, opus za završni pregled.
