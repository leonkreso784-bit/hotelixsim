# Stanje Rust alata — listopad 2026.

> Istraživanje 2026-10-04 (agent, crates.io / GitHub / npm / službena dokumentacija).
> **[P]** provjereno na izvoru · **[?]** pretpostavka. Lokalno: Rust 1.98, Node 24, nema Dockera.

## 1 · Axum
- [P] **0.8.9** (2026-04-14); nema 0.9. Zadnje veće promjene u 0.8.0 (putanje `/{id}`, `Handler` mora biti `Sync`).
  Repo aktivan, izdanja rijetka. → `axum = "0.8"`.
- [P] tower-http 0.7.1 (2026-08-31). [?] Usklađenost s axum 0.8 provjeriti pri prvom `cargo add`.

## 2 · OpenAPI iz Rusta i TS strana
- [P] **utoipa 6.0.0** (2026-09-22, OpenAPI 3.2, MSRV 1.88) + **utoipa-axum 0.3.0** (traži axum ^0.8.4). Najveća zajednica.
- [P] aide 0.15.1 je stabilna, ali na starom schemars 0.9; 0.16 je još alfa.
- [?] Provjeriti da utoipa 6 po zadanom daje OpenAPI 3.1, jer openapi-typescript podržava samo 3.0 i 3.1.
- [P] TS: **openapi-typescript 7.13 + openapi-fetch 0.17** su de facto standard (samo tipovi i tanak klijent).
  Orval 8.39 ako zatrebaju React Query hookovi; @hey-api je još 0.x.
- [P] TypeScript 7.0.2, Vite 8.3.2, @tanstack/react-query 5.104. [?] openapi-typescript s TS 7 nije provjeren.

## 3 · SQLx
- [P] **0.9.0** (svibanj 2026), repo preseljen na github.com/transact-rs/sqlx. MSRV 1.94; `query*()` traži
  `&'static str` ili `AssertSqlSafe(..)`; novi `sqlx.toml`.
- [P] Offline: `cargo sqlx prepare --workspace` → `.sqlx/` u git; `prepare --check` u CI-u; `SQLX_OFFLINE=true` za build.
- Lokalno nema Dockera → za `prepare` treba nativni Postgres ili udaljena razvojna baza.

## 4 · Red poslova na Postgresu
**Ključna zamka [P]:** sqlx 0.8 i 0.9 ne dijele `Transaction`. Upis „odgovor + posao" u istoj transakciji radi samo
ako biblioteka reda koristi istu verziju sqlx-a kao aplikacija.

| izvedba | stanje | sqlx | napomena |
|---|---|---|---|
| apalis | stabilno samo 0.7.4; 1.0 je RC s lomovima | apalis-postgres na 0.9 | mlad backend (9 zvjezdica); [?] transakcijski upis nije dokumentiran |
| pgmq (crate) | 0.33.7 | 0.8 ✗ | ekstenzija nije na službenom Railway Postgresu, ali postoji **SQL-only** instalacija |
| underway | 0.2.0 | 0.8 (0.9 neobjavljen) | jedan autor |
| sqlxmq | 0.6.0, napušten | 0.8 | ne uzimati |
| graphile_worker (Rust) | 0.13.6, izdanja tjedno | 0.9 | `SKIP LOCKED` + LISTEN/NOTIFY; malo korisnika; [?] transakcijski upis |
| **vlastiti** `SKIP LOCKED` + zakup | — | 0.9 ✓ | transakcijski upis siguran; sami pišemo retry, zakup, čišćenje (~150 redaka) |

Preporuka agenta: vlastiti mali red ili graphile_worker; alternativa pgmq SQL-only kroz vlastitu transakciju.
Konačan izbor ide probnim testom (ARHITEKTURA §9.4).

## 5 · Determinizam
- [P] `overflow-checks = true` u `[profile.release]` vrijedi samo u korijenskom manifestu workspacea.
- [P] clippy `disallowed-types` u `clippy.toml` zabranjuje `HashMap`/`HashSet` (s `replacement = "BTreeMap"`);
  [?] hvata samo naš kod.
- [P] **RNG:** rand 0.10.3, rand_chacha 0.10.0, rand_pcg 0.10.2. `StdRng` i `SmallRng` izričito **nisu prenosivi**.
  ChaCha i PCG stabilni su kroz patch izdanja, ali **minor izdanja smiju promijeniti izlaz**. `f32`/`f64` i `usize`
  nisu prenosivi.
  → pinirati točno (`=0.10.x`), uzimati samo `next_u64()`, vlastiti raspon/miješanje/vjerojatnost na cijelim
  brojevima, „zlatni" testovi (sjeme → poznati izlaz).
- [P] rust_decimal 1.43.0. [?] Preporuka: novac u centima (`i64`), postoci u baznim bodovima; bez `f64` u jezgri.

## 6 · Railway
- [P] Zadani builder je Railpack (Nixpacks zastario). Railpack za Rust zadano koristi 1.89; [?] workspace i
  `SQLX_OFFLINE` nisu dokumentirani.
- [P] `Dockerfile` u korijenu se prepoznaje sam; podržani BuildKit cache mountovi. Jedan servis (API + radnik) nije
  ničim ograničen.
- [P] Službeni Railway Postgres **nema dodatnih ekstenzija** i neće ih dobiti; pgmq samo kroz zajednički predložak.
- Preporuka: vlastiti višestupanjski Dockerfile (pinirani toolchain, `SQLX_OFFLINE=true`), službeni Postgres bez ekstenzija.

**Nije provjereno:** zadana OpenAPI verzija u utoipa 6 · transakcijski upis u apalis-postgres i graphile_worker ·
tower-http 0.7 uz axum 0.8 · openapi-typescript uz TS 7 · Railpack i Cargo workspace.
