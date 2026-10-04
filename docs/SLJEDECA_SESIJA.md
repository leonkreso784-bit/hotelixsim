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
6. Ledger kostura `docs/superpowers/plans/2026-10-04-kostur-ledger.md` — **samo grep** `minor (deferred)` i `Final #`
   (odgođeni nalazi, ulaz za plan; vidi §3.2).
7. `git status -sb` i `git log origin/main..main --oneline` (očekuje se: grana `main`, ispred `origin` samo commitovi
   predaje).

## 2 · Gdje smo (04.10. navečer)

**Gotovo, spojeno u `main` i pushano na GitHub** (04.10., Leonov OK; CI rezultat — vidi §5):

- Arhitektura v2, `DEMO.md` (dizajn igre + kategorije hotela), tri istraživanja u `docs/istrazivanje/`.
- **Kostur od kraja do kraja** po planu `docs/superpowers/plans/2026-10-04-kostur.md`, 8 taskova, svaki s pregledom,
  plus završni pregled cijele grane (opus) i jedan krug završnih popravaka:
  - Rust workspace: `engine` (kocka SplitMix64, zlatni vektori), `sim-hotel` (5 indeksa u stotinkama, profili A/B/C
    kao konstante), `app` (port `SystemStatus`), `store` (dnevnik u Postgresu, SQLx 0.9, offline `.sqlx/`),
    `api` (Axum 0.8 + utoipa 6, OpenAPI 3.1), `server` (serve | migrate | openapi).
  - `openapi.json` → generirani TS klijent `@hotelixsim/api-client` → `packages/i18n` → jedan React ekran `apps/web`.
  - Brane: clippy (HashMap/HashSet/sat, float/arith lintovi), `scripts/check-core-deps.mjs`, cargo-deny, CI datoteka
    `.github/workflows/ci.yml` (pokrenuta pushem 04.10., rezultat nije viđen — §5.1).
  - Testovi: Rust 28, sučelje 10. Lokalni prolaz od kraja do kraja dokazan (zdravlje 200, profil D → 404,
    kriva lozinka baze → 503 bez pada, Vite proxy).

**Ponovna provjera 04.10. (druga sesija, četiri agenta, samo čitanje):** lokalno sve iste naredbe kao CI izlaze s 0
— fmt, clippy `-D warnings`, brana ovisnosti, `cargo test` (28), `cargo deny`, `sqlx prepare --check`,
`pnpm check:api` (openapi svjež), lint, typecheck, test (10). Čitanjem CI-ja: vjerojatno zeleno; najveći rizik je
da `setup-rust-toolchain` sam postavlja `RUSTFLAGS="-D warnings"` (lokalno clippy s istom zastavicom prolazi, pa je
rizik mali) i sporost (`cargo install sqlx-cli` bez cachea, ~5–10 min).

**Git:** `main`, ispred `origin/main` su samo commitovi predaje (push čeka Leonov OK). Grana `gradnja/kostur` je
spojena i smije se obrisati (`git branch -d`).

**Lokalna baza:** zaseban PostgreSQL 16 klaster u `C:\Users\leonk\AppData\Local\hotelixsim\pgdata`, **priključak 5433**
(sistemski PostgreSQL na 5432 se ne dira — Leon ne zna njegovu lozinku). **Ne pali se sam nakon restarta:**
`bash scripts/baza.sh start`. Pokretanje svega: README → „Lokalni razvoj".

## 3 · Cilj sljedeće sesije: plan „jezgra i alat za balans" (PLAN.md, DEMO korak 2)

0. **Prije plana — rupe u specu DEMO.md** (revizija 04.10., račun ponovljen neovisno, oba POTVRĐENA):
   - **Profil C ne kreće s 50.** §5 (DEMO:264) i §2.1 (DEMO:165) traže 50 u svakom indeksu, ali iz §2.1 i §4 za
     početak lipnja izlazi G=70, R=50, Z≈54, O=70, F≈55 (zbroj ≈299, ne 250 iz DEMO:257). Samo je F relativan
     (prema GOPPAR_ref); G, Z, O su apsolutni, iako §2.0 kaže „prema referentnom hotelu iste kategorije".
     Odluči: centrirati formule na referentni hotel ili promijeniti početne vrijednosti — i zapiši kao pretpostavku.
   - **Kalibracija:** provjera u §2.1 izostavlja zadano redovno održavanje 15 000 €. S njim: lipanj 33,6 %,
     srpanj 50,4 %, kolovoz 55,5 %, rujan 34,3 %, sezona ≈45 % (DEMO:161 „lipanj blizu HotStats ≈45 %" ne stoji).
     Cilj 30–40 % je godišnji (DEMO:163), a demo ima četiri sezonska mjeseca — odluči s čime se uspoređuje.
   - **Cjelobrojni račun nema pravila zaokruživanja** (DEMO:9-10 kaže samo centi i bazni bodovi): kazna
     opterećenja (oblik 1,8→2,4), potražnja i f-faktori (floor/round na sobe-noći), fluktuacija (3,75 % od 50 ljudi),
     nazivnik GOPPAR-a, jedinica pohrane zadovoljstva/morala/opreme/ocjene, „p. b." vs postotak. Zlatne vrijednosti
     ovise o tome → pravila u plan prije prvog testa.
   - Nedefinirano manje: početni F (GOPPAR traži mjesec), „mješavina" iskustva, cijena automatske zamjene,
     trošak sezonca, obuka „moral +2", dijeljenje s nulom u g_i, tvrdo 120 soba u §3.1.
   - Za robote/balans (§11.3) fale ponuđeni odgovori i rubrike za S1b i S5–S8 (§7 „pišu se poslije") → task sadržaja.
1. **Napiši plan** (`superpowers:writing-plans`) iz `DEMO.md` §2–§11. **Task 1 plana je OBAVEZNO ojačavanje brana
   determinizma**, prije ijedne linije jezgre (završni pregled kostura, Final #3 — brane su slabije od ARHITEKTURA
   §6.2/§7.2):
   - `check-core-deps.mjs`: dopušteni popis (allowlist) umjesto zabranjenog `FORBIDDEN` (redci 7–18) — jezgra
     (`CORE = engine, sim-hotel`) smije ovisiti samo jedna o drugoj i o izričito odobrenim paketima (danas bi prošao
     `indexmap` s nasumičnim sjemenom). Unaprijed odluči ide li budući `content` crate (JSON) pod branu:
     `serde_json` s `preserve_order` povlači `indexmap`;
   - **dva** identična `clippy.toml` (`crates/engine/` i `crates/sim-hotel/`, u korijenu ga nema): dodati
     `std::hash::RandomState` i `std::collections::hash_map::RandomState` u oba (ili ih spojiti u jedan izvor);
   - `lib.rs` (oba): danas samo `#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]`
     (`unsafe_code = "forbid"` već je u workspace `Cargo.toml`). → `forbid` + `clippy::as_conversions` (ili `cast_*`),
     zabrana `f32`/`f64`, razmotriti `std::env::var`. Da `forbid` stvarno radi za clippy lintove — dokazati mutacijom;
   - probe mutacija kao skripta koja se vrti u CI-u (svaka mora pasti). **Danas skripte nema**; ručno je mutacijom
     provjeren samo `HashMap` u `engine` (ledger redak 38) — HashSet, sat, float/arith i `sim-hotel` nisu.
2. Ulazi za dizajn jezgre (iz završnog pregleda): imenovana i ograničavajuća izgradnja `IndexSet` (danas pozicijski
   `[IndexValue; 5]`, `index.rs:59`), profili u `content/hotel/` (mapa još ne postoji) s A/B/C kao testnim
   proročanstvom, kategorija hotela (DEMO §2.0) kao podatak. Odgođeni nalazi su u ledgeru **kronološki po taskovima**
   (34 × `minor (deferred)` + redak „Ruling: ostali minori" ~11 stavki) — pri pisanju plana grupiraj ih po temi
   (brane · rng/indeksi · baza/`events` · API · klijent · i18n/ekran · CI) i svaku grupu dodijeli tasku ili kasnijem planu.
   **API dio je izvan opsega jezgre** (ARHITEKTURA §20.2: „bez baze i bez ekrana") — ruta profila (`api/src/demo.rs`)
   zove `sim-hotel` mimo `app`, enum kodovi u OpenAPI-ju (danas `String`) → zaseban, označen task na kraju plana ili
   plan „dnevnik/runde". Zdravlje već ide kroz `app`.
3. Gradnja kroz `superpowers:subagent-driven-development`, kao danas: grana, radni ledger u `.superpowers/sdd/`
   (**gitignoriran** — na kraju ga kopiraj u `docs/superpowers/plans/<datum>-<ime>-ledger.md`, kao za kostur),
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

1. **CI na GitHubu** — `main` je pushan 04.10. (Leonov OK), pa je CI prvi put pokrenut. Rezultat nismo vidjeli
   (repo je privatan, `gh` nije instaliran, GitHub MCP nije radio). Pitaj Leona da otvori karticu **Actions** na
   GitHubu ili mu daj upute; ako je crveno, prvo popravi CI. Svaki idući push i dalje traži njegov izričit OK.
   Uz to: **push commitova predaje** (`git log origin/main..main`) — tek na njegov OK.
2. **Pitanja o domeni** — popis 1:1 iz DEMO.md „Za Leonovu provjeru" (brojevi kao ondje). **Ne blokiraju kod**:
   brojke su podaci u katalogu, gradi s pretpostavkom i zamijeni kad odgovori.
   - 2 (ostatak): ostali prihod (hrana, piće, usluge) = 30 % prihoda soba — realno?
   - 3: trošak zaposlenog 2 200 €/mj i omjer 30 stalnih + 20 sezonaca (120 soba, 4*) — realno?
   - 4: S3 — 25 % popusta agenciji za grupu u kolovozu — realna ponuda?
   - 5: kriteriji rubrika u S1–S4 — on je mjerilo (blokira sadržaj i AI presude, ne kod jezgre).
   - 6: priče profila A i B (§5) — prepoznaje li takve hotele?
   - (1 riješen 04.10.: osjetljivost cijene po kategoriji, demo svi 4*.)
3. **AI ključ za demo:** koje besplatne ključeve ima (pružatelj i model). Ne blokira jezgru.
4. Pitanja za profesore (`docs/PITANJA_ZA_PROFESORE.md`) — pregled prije slanja.
5. **Rokovi:** demo više nema datum (10.10. pomaknut) — pitaj ima li ciljani datum ili „bez roka". Prijava u
   inkubator ima rok **16.11.** (PLAN §otvoreno 4: Sokrat ili simulacija).
6. **ARHITEKTURA v2:** PLAN kaže „Leon ih je pregledao", a ARHITEKTURA:6 „prijedlog dok Leon ne potvrdi" — pitaj
   je li v2 potvrđena ili je rasprava još otvorena, pa uskladi oba dokumenta.

## 6 · Pravila (sažetak; puna su u CLAUDE.md i memoriji)

- Hrvatski, kratko, bez žargona; pojam koji možda ne zna objasni od početka (npr. 04.10. nije znao što je lozinka
  PostgreSQL-a — riješeno bez njega).
- **Radi bez pitanja** (Leon, 04.10.): pretpostavke zapiši u dokument; ankete samo za proizvodne odluke.
  **Push i deploy samo uz izričit OK.**
- Nikad „gotovo" ili „zeleno" prije viđenog izlaza; uz svako „gotovo" reci gdje se vidi.
- Temelj prije brzine. Kratke dionice, javi se na prirodnim granicama.
- Model po poslu: sonnet za izvođače i preglede taskova, haiku za kratke ponovne preglede, opus za završni pregled.

## 7 · Usput

- **Pixel Agents** (VS Code dodatak za prikaz agenata): ured 64×64 sa sobama i 60 radnih mjesta,
  `watchAllSessions` uključen. Kopije starih rasporeda: `C:\Users\leonk\.pixel-agents\backup-2026-10-04\`.
  Leon voli vidjeti puno agenata koji rade istodobno — neovisne poslove pokreći paralelno, u jednoj poruci.
- Ostala sitna proturječja dokumenata (VIZIJA:4 „tehnički izbori nisu doneseni", VIZIJA:70 „pilot" = danas DEMO,
  runda tjedan/mjesec u PITANJA i ARHITEKTURA, kategorija hotela nije u ARHITEKTURA, `kostur.md:9` „Windows servis"):
  vidi commitove nakon predaje — usklađivanje je pokrenuto 04.10.
