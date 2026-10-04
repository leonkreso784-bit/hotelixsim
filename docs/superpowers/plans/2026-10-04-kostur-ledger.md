# SDD ledger — plan: docs/superpowers/plans/2026-10-04-kostur.md

Spec: docs/ARHITEKTURA.md v2, docs/DEMO.md §4–§5, docs/istrazivanje/RUST_ALATI_2026-10.md (dostupno).
Grana: gradnja/kostur (od main @ 75ecef3).

Ruling: rad na grani `gradnja/kostur` u istom direktoriju, ne u zasebnom worktreeju — Leon pokreće sve lokalno iz `Documents\hotelixsim` i rekao je „radi bez pitanja"; grana je izolacija od main-a — ako je krivo: Leon mora `git switch` da vidi rad, ništa se ne gubi.
Ruling: Task 0 (baza) čeka Leonovu ruku; Taskovi 1–3 ne trebaju bazu i idu prvi — ako je krivo: nema cijene, redoslijed taskova je isti.

## Preflight scan

| par / task | proizvodi → troši | nalaz |
|---|---|---|
| T1 ↔ T4, T5 | `Cargo.toml` members + `[workspace.dependencies]` s path-ovima `app`, `store`, `api` | T1 deklarira path-ovisnosti za pakete koji još ne postoje. Rizik: cargo može odbiti nepostojeći path. → Ruling 1 |
| T1 ↔ T3 | `crates/engine/clippy.toml` → kopija u `sim-hotel` | dosljedno |
| T2 → T3 | `engine` paket → ovisnost `sim-hotel` | sim-hotel ne koristi Rng još; nekorištena ovisnost je dopuštena (cargo ne javlja) |
| T3 → T5 | `Profile::from_code`, `starting_indices`, `IndexSet::iter`, `Index::code`, `IndexValue::hundredths` | potpisi se poklapaju |
| T4 → T5 | `app::{SystemStatus, BoxFut, StatusError, check_health(&dyn), DbState}`, `Store::{connect_lazy, migrate}` | potpisi se poklapaju |
| T5 → T6 | `server openapi` → `scripts/gen-openapi.mjs` | `print!` + `\n` iz `openapi_json` → datoteka s LF; provjera `"openapi": "3.1.0"` odgovara `to_string_pretty` formatu |
| T6 → T7 | `createApiClient`, `ApiClient`, `pnpm-lock.yaml` | dosljedno |
| T6 → T8 | skripte `check:api`, `lint`, `typecheck`, `test` → CI | CI koristi `pnpm` nakon `corepack enable` (Linux, bez admin problema) |
| T1 | testovi vs kod | Step 6 očekuje 1/2 paketa — točno (cargo tree uključuje sam paket) |
| T2 | testovi vs kod | 10 testova; zlatni vektori iz nezavisne Node izvedbe |
| T3 | testovi vs kod | 7 testova; fallback za `.sum()` naveden |
| T4 | testovi vs kod | `.sqlx` mjesto nepoznato unaprijed — Step 6/8 to pokrivaju |
| T5 | testovi vs kod | 6 testova; fallback za 3.2 naveden |
| T6 | skripte vs okolina | `gen:api` zove ugniježđeni `pnpm`, a `pnpm` nije na PATH-u (samo `corepack pnpm`). → Ruling 2 |
| T7 | testovi vs kod | 4 testa; `exactOptionalPropertyTypes` može tražiti prilagodbu tipova — plan dopušta |
| T8 | ručna provjera | traži pokretanje servera u pozadini i gašenje vlastitih procesa |

Ruling 1: ako cargo u T1 odbije `[workspace.dependencies]` s path-om na nepostojeći paket, izvođač ih uklanja u T1 i dodaje u tasku koji stvara paket (T4: app, store; T5: api) — plan je argument, spec ne propisuje mjesto deklaracije — cijena ako je krivo: nikakva, samo mjesto retka.
Ruling 2: u T6 skripte ne smiju ovisiti o tome da je `pnpm` na PATH-u. Ako ugniježđeni `pnpm` ne radi kroz `corepack pnpm`, izvođač piše skripte koje zovu alate izravno (npr. `node scripts/gen-openapi.mjs && pnpm exec`-ekvivalent kroz `openapi-typescript` binar u workspaceu ili `$npm_execpath`), zadržava imena skripti i ponašanje — cijena ako je krivo: CI (Linux, `corepack enable`) i lokalno se ponašaju isto, rizik nizak.

## Napredak
Task 1: review → Spec ✅, Quality: changes requested (1 Important, 4 Minor).
Ruling: Important #1 (check-core-deps ne vidi build-ovisnosti, druge targete ni opcionalne značajke) je plan-mandated, ali spec (ARHITEKTURA §6.2 „ne smiju ovisiti", tranzitivno) je obvezujući → popravak ulazi u fix loop; plan se ispravlja istim argumentima — cijena ako je krivo: sporija brana (cargo tree s --target all), ništa drugo.
Ruling: Minor #2 (prazan izlaz cargo tree → lažno „0 zabranjenih") ulazi u isti popravak jer je isti razred greške kao #1 (brana mjeri manje nego što tvrdi), ista datoteka — cijena ako je krivo: nekoliko redaka koda.
Task 1: minor (deferred): #3 `#![deny]` umjesto `#![forbid]` za clippy brane u lib.rs — lokalni allow ih može zaobići.
Task 1: minor (deferred): #4 mutacijom provjeren samo HashMap u engine; disallowed-methods, HashSet, float/arith lintovi i sim-hotel clippy.toml nisu.
Task 1: minor (deferred): #5 Cargo.lock/Cargo.toml/*.mjs nisu pinirani na LF.
Task 1: fix round 1/5 (2 addressed, 0 open; commits 1d093fe..70f113b)
Task 1: complete (commits 75ecef3..70f113b, review clean after fix round 1)
Task 2: minor (deferred): test derive_streams_are_independent je gotovo prazan (prošao bi i da derive ignorira oznake) — dodati assert_ne za T1≠T2, redoslijed oznaka, drugo sjeme.
Task 2: minor (deferred): grana odbacivanja u range se u testovima ne izvršava (mali rasponi).
Task 2: minor (deferred): doc za derive(seed, &[]) vs from_seed — različiti prostori stanja.
Task 2: complete (commits d23a712..261158f, review clean)
Ruling: `cargo fmt --check` pada na kodu iz plana (Task 2 rng.rs, rng_golden.rs; Task 3 index.rs, profile.rs) — plan je pisan sa ~110 znakova po retku, rustfmt traži 100. fmt je vrata (Task 1 Step 6, CI, README) → Task 3 izvođač pokreće `cargo fmt --all` nad cijelim workspaceom kao zaseban commit (samo formatiranje), testovi se ponove; svaki idući dispatch mora proći `cargo fmt --all -- --check` prije commita — cijena ako je krivo: jedan commit formatiranja, nula ponašanja.
Task 3: minor (deferred): nema testa da position() odgovara Index::ALL.
Task 3: minor (deferred): profili A i B testirani samo točkasto (zamjena G/O ili Z/F bi prošla) — pinati pune vektore.
Task 3: minor (deferred): test odbijanja u from_hundredths provjerava samo zadnje mjesto.
Task 3: complete (commits 261158f..d11cc06, review clean)
Ruling: Task 0 promijenjen — Leon ne zna lozinku sistemskog PostgreSQL-a (5432). Umjesto resetiranja lozinke (admin, diranje tuđe instalacije) napravljen je zaseban klaster pod Leonovim korisnikom: C:/Users/leonk/AppData/Local/hotelixsim/pgdata, priključak 5433, superkorisnik hotelixsim/hotelixsim_lokalno (scram), baza hotelixsim. Pokretanje: `bash scripts/baza.sh start` (ne pali se sam nakon restarta). .env i .env.example → 5433. Leon je odobrio („možeš li ti to sve napraviti" + Yes na dozvolu) — cijena ako je krivo: obrisati mapu pgdata; ništa drugo nije dirano.
Task 0: complete (lokalni klaster na 5433, provjera psql: current_user=hotelixsim, PostgreSQL 16.11)
Task 4: minor (deferred): check_health baca poruku greške (503 bez uzroka u zapisima) — logirati u api/serveru.
Task 4: minor (deferred): stream_sequence_is_unique provjerava samo is_err (ne SQLSTATE 23505) i ne dokazuje da isti seq u drugom toku prolazi.
Task 4: minor (deferred): nema testa za connect_lazy na nedostupnu bazu (~3 s) ni migrate na svježoj bazi.
Task 4: minor (deferred): sqlx bez TLS značajke — za Railway dodati tls-rustls (nikad native-tls/openssl).
Task 4: minor (deferred): .sqlx/*.json nije piniran na LF.
Task 4: note: Cargo.lock sadrži sqlx-sqlite/libsqlite3-sys kao neaktivne opcionalne unose — Task 8 cargo deny s all-features to može vidjeti.
Task 4: complete (commits 8129a79..05d1177, review clean)
(d1e7ca5 = DEMO.md kategorije, izvan plana, samo dokument)
Task 5: minor (deferred): HealthBody.events nije u required → TS tip events?: number|null.
Task 5: minor (deferred): OpenAPI info ima prazan description i license.name.
Task 5: minor (deferred): ctrl_c greška ignorirana (server bi se odmah ugasio).
Task 5: minor (deferred): nema fallback rute → nepoznata ruta daje prazan 404, ne problem+json.
Task 5: minor (deferred): Problem.status slobodan u16; to_vec(..).unwrap_or_default.
Task 5: minor (deferred): test OpenAPI-ja ne tvrdi 503 ni problem+json za 404; 503 testiran samo lažnim portom.
Task 5: complete (commits d1e7ca5..347f623, review clean)
Ruling (Task 6, Ruling 2 izvedba): scripts/pnpm.mjs ponovno pokreće pnpm preko npm_execpath — radi lokalno (corepack) i u CI-ju (corepack enable); pukne samo uz samostalni pnpm binar — cijena ako je krivo: zamijeniti omotač izravnim pozivima alata.
Task 6: minor (deferred): pnpm.mjs pretpostavlja da je npm_execpath JS datoteka.
Task 6: minor (deferred): check:api ne hvata nepraćenu generiranu datoteku (dodati git status --porcelain).
Task 6: minor (deferred): openapi-fetch i application/problem+json — provjeriti parsiranje 404 tijela u prvom stvarnom pozivu.
Task 6: minor (deferred): gen-openapi.mjs piše relativno na cwd.
Task 6: complete (commits 347f623..4a7b8bb, review clean)
Task 7: review → Spec ✅, Quality: changes requested (2 Important, 6 Minor).
Ruling: Important #1 (greška/učitavanje indeksa nevidljivi) i #2 (nepoznat kod indeksa → undefined tekst) su plan-mandated, ali korisnik mora vidjeti stanje, a t() obećaje string → oba u fix loop — cijena ako je krivo: nekoliko redaka i 2–3 testa.
Ruling: Minor #3 (literal ": " i spajanje "naslov + profil" u komponenti) krši Global Constraint „tekst samo kroz packages/i18n" → ulazi u isti popravak kao jednostavna interpolacija t(key, params) — cijena ako je krivo: malo veći i18n API, aditivno.
Task 7: minor (deferred): i18n nema lokal/parametar jezika ni zajednički tip za en rječnik; format.ts tvrdo "hr-HR".
Task 7: minor (deferred): QueryClient retry 3 → ~7 s „Učitavam…" kad API ne radi.
Task 7: minor (deferred): testovi ne pokrivaju pending, 500 na health, skriveni red događaja kad je events null; lažni fetch pretpostavlja Request.
Task 7: minor (deferred): health redak bez role="status"/aria-live.
Task 7: fix round 1/5 (3 addressed, 0 open; commits f42b7cd..ced2416)
Task 7: minor (deferred): t() parametri nisu tipizirani po poruci (krivo ime parametra = vidljiv {placeholder}).
Task 7: complete (commits 4a7b8bb..ced2416, review clean after fix round 1)
Task 8: review → Spec ✅, Quality: changes requested (1 Important, 8 Minor).
Ruling: Minor #2–#4 (README: <korisnik> vs $USERNAME, migrate;serve umjesto &&, dva terminala) ulaze u isti popravak kao Important #1 — isporuka Taska 8 je README koji radi na svježem stroju, isti razred greške — cijena ako je krivo: par redaka README-a.
Task 8: minor (deferred): CI prevodi sqlx-cli svaki put bez cachea.
Task 8: minor (deferred): CI on push+pull_request bez filtra, bez permissions: contents: read, akcije po tagu ne SHA.
Task 8: minor (deferred): cargo-deny-action@v2 vlastiti Rust/verzija vs rust-toolchain 1.98.1 — provjeriti na prvom CI-u.
Task 8: minor (deferred): deny.toml zabranjuje samo openssl-sys (ne native-tls/openssl).
Task 8: minor (deferred): E2E „baza ne radi" dokazan krivom lozinkom, ne ugašenom bazom (3 s acquire_timeout put nije izmjeren).
Task 8: fix round 1/5 (4 addressed, 0 open; commits 85419f6..06c60e4)
Task 8: complete (commits ced2416..06c60e4, review clean after fix round 1)
Final review (opus): Ready to merge — with fixes. 0 Critical, 3 Important.
Ruling: Final #1 (CRLF u svježem kloniranju → lint pada) i #2 (.env tjera sqlx makroe na živu bazu → build pada kad baza ne radi) idu u jedan završni popravak prije spajanja; uz njih jeftino: tracing::warn! na grešku zdravlja (T4) i `permissions: contents: read` u CI-u — cijena ako je krivo: par redaka, sve provjereno u svježem kloniranju.
Ruling: Final #3 (brane determinizma slabije od §6.2/§7.2: indexmap, RandomState, `as` na float/u16, #[allow] zaobilazi deny, env::var) NE blokira spajanje jer jezgra danas nema ovisnosti ni takav kod — ali je OBAVEZNO prvi task sljedećeg plana (jezgra + balans): allowlist ovisnosti, RandomState u clippy.toml, forbid + as_conversions/cast lintovi, zabrana f32/f64, probe mutacija kao skripta u CI-u — cijena ako je krivo: kod jezgre napisan prije jačih brana mogao bi proći s nedeterminizmom; zato ide prije ijedne linije jezgre.
Ruling: ostali minori iz završnog pregleda (api→sim-hotel mimo app, COUNT(*) u zdravlju, oblik tablice events, enum kodovi u OpenAPI, imenovani IndexSet, profili u content/, base URL, SIGTERM, typed config, README sqlx-cli, vite build u CI) ulaze kao ulaz u sljedeći plan — cijena ako je krivo: nijedan ne utječe na lokalni demo kostura.
Final fix wave: 4 addressed, 0 open (commits 06c60e4..303aeb8); scoped re-review clean.
PLAN COMPLETE: gradnja/kostur 75ecef3..303aeb8, final review clean after one fix wave.
