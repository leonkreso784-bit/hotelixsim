# Odgođeni nalazi kostura — po temama

Svrha: nalazi iz ledgera kostura (`2026-10-04-kostur-ledger.md`), prerasporedjeni po temi i dodijeljeni planovima, kao ulaz za plan "jezgra i alat za balans".

Oznake: **[jezgra]** plan jezgre i balansa · **[dnevnik]** plan dnevnik/runde/red poslova (uključuje API dio, ARHITEKTURA §20.2) · **[AI]** AI sloj · **[ekrani]** ekrani igrača · **[prije Railwaya]** prije prvog deploya · **[možda nikad]** · **[riješeno]** već popravljeno u završnom valu.
Izvor: "L37" = redak u ledgeru. Final #1 i #2 (CRLF, `.env` i sqlx makroi) popravljeni su u završnom valu (L94) i nisu u brojenju.

## Brane determinizma

1. `#![deny]` umjesto `#![forbid]` za clippy brane u `lib.rs` (lokalni allow ih zaobilazi). Task 1 #3, L37. [jezgra]
2. Mutacijom provjeren samo `HashMap` u `engine`; fale `disallowed-methods`, HashSet, sat, float/arith i `sim-hotel` clippy.toml. Task 1 #4, L38. [jezgra]
3. Final #3: allowlist ovisnosti (indexmap), `RandomState` u oba clippy.toml, forbid + `as_conversions`, zabrana f32/f64, `env::var`, skripta mutacija u CI-u. Ledger L95, SLJEDECA_SESIJA §3.1. **Obavezan Task 1 plana, prije ijedne linije jezgre.** [jezgra]

## RNG i indeksi (engine, sim-hotel)

4. `derive_streams_are_independent` gotovo prazan (dodati assert_ne za T1≠T2, redoslijed oznaka, drugo sjeme). Task 2 #1, L42. [jezgra]
5. Grana odbacivanja u `range` se ne izvršava u testovima (mali rasponi). Task 2 #2, L43. [jezgra]
6. Doc za `derive(seed, &[])` vs `from_seed` (različiti prostori stanja). Task 2 #3, L44. [jezgra]
7. Nema testa da `position()` odgovara `Index::ALL`. Task 3 #1, L47. [jezgra]
8. Profili A i B testirani samo točkasto; pinati pune vektore (uskladiti s rupom "profil C ne kreće s 50"). Task 3 #2, L48. [jezgra]
9. Test odbijanja u `from_hundredths` provjerava samo zadnje mjesto. Task 3 #3, L49. [jezgra]
10. Imenovana i ograničavajuća izgradnja `IndexSet` (danas `[IndexValue; 5]`). Final minori, L96. [jezgra]
11. Profili u `content/hotel/` kao testno proročanstvo; kategorija hotela kao podatak. Final minori, L96. [jezgra]

## Baza i tablica events

12. `check_health` baca poruku greške (503 bez uzroka u zapisima). Task 4 #1, L53. Zapis greške dodan u završnom valu (303aeb8). [riješeno]
13. `stream_sequence_is_unique` provjerava samo `is_err`, ne SQLSTATE 23505, ni da isti seq u drugom toku prolazi. Task 4 #2, L54. [dnevnik]
14. Nema testa za `connect_lazy` na nedostupnu bazu (~3 s) ni `migrate` na svježoj bazi. Task 4 #3, L55. [prije Railwaya]
15. sqlx bez TLS značajke; za Railway dodati `tls-rustls` (nikad native-tls/openssl). Task 4 #4, L56. [prije Railwaya]
16. `COUNT(*)` u zdravlju (skup na velikoj tablici). Final minori, L96. [prije Railwaya]
17. Oblik tablice `events` (globalni redoslijed, tip `actor`, indeks na `game_id`). Final minori, L96; SLJEDECA_SESIJA §3 kaže prije Railwaya. [dnevnik]

## API i OpenAPI

18. `HealthBody.events` nije u `required` (TS tip `events?: number|null`). Task 5 #1, L61. [dnevnik]
19. OpenAPI `info` ima prazan description i license.name. Task 5 #2, L62. [dnevnik]
20. `ctrl_c` greška ignorirana (server bi se odmah ugasio). Task 5 #3, L63. [prije Railwaya]
21. Nema fallback rute: nepoznata ruta daje prazan 404, ne problem+json. Task 5 #4, L64. [dnevnik]
22. `Problem.status` slobodan u16; `to_vec(..).unwrap_or_default`. Task 5 #5, L65. [možda nikad]
23. Test OpenAPI-ja ne tvrdi 503 ni problem+json za 404; 503 testiran samo lažnim portom. Task 5 #6, L66. [dnevnik]
24. `api` zove `sim-hotel` mimo `app` (`api/src/demo.rs`). Final minori, L96; SLJEDECA_SESIJA §3.2. [dnevnik]
25. Enum kodovi u OpenAPI-ju (danas `String`). Final minori, L96. [dnevnik]
26. Base URL klijenta/API-ja. Final minori, L96. [prije Railwaya]
27. SIGTERM (graceful shutdown). Final minori, L96. [prije Railwaya]
28. Typed config (umjesto sirovih env varijabli). Final minori, L96. [prije Railwaya]

## Generirani klijent

29. `pnpm.mjs` pretpostavlja da je `npm_execpath` JS datoteka. Task 6 #1, L69. [možda nikad]
30. `check:api` ne hvata nepraćenu generiranu datoteku (dodati `git status --porcelain`). Task 6 #2, L70. [dnevnik]
31. `openapi-fetch` i `application/problem+json`: provjeriti parsiranje 404 tijela u prvom stvarnom pozivu. Task 6 #3, L71. [ekrani]
32. `gen-openapi.mjs` piše relativno na cwd. Task 6 #4, L72. [možda nikad]

## i18n i ekran

33. i18n nema lokal/parametar jezika ni zajednički tip za en rječnik; `format.ts` tvrdo "hr-HR". Task 7 #1, L77. [ekrani]
34. QueryClient `retry` 3 daje ~7 s "Učitavam…" kad API ne radi. Task 7 #2, L78. [ekrani]
35. Testovi ne pokrivaju pending, 500 na health, skriveni red događaja kad je events null; lažni fetch pretpostavlja `Request`. Task 7 #3, L79. [ekrani]
36. Health redak bez `role="status"`/`aria-live`. Task 7 #4, L80. [ekrani]
37. `t()` parametri nisu tipizirani po poruci (krivo ime = vidljiv `{placeholder}`). Task 7 #5, L82. [ekrani]

## CI, deny, alati

38. `Cargo.lock`/`Cargo.toml`/`*.mjs` nisu pinirani na LF (`.gitattributes`). Task 1 #5, L39. [jezgra]
39. `.sqlx/*.json` nije piniran na LF. Task 4 #5, L57. [jezgra]
40. CI prevodi `sqlx-cli` svaki put bez cachea (~5–10 min); bitno kad CI dobije mutacijski posao. Task 8 #1, L86. [jezgra]
41. CI na push+pull_request bez filtra, akcije po tagu ne SHA (`permissions: contents: read` već dodano u završnom valu). Task 8 #2, L87. [prije Railwaya]
42. `cargo-deny-action@v2` vlastiti Rust vs `rust-toolchain` 1.98.1: provjeriti na prvom CI-u. Task 8 #3, L88. [jezgra]
43. `deny.toml` zabranjuje samo `openssl-sys` (ne native-tls/openssl). Task 8 #4, L89. [prije Railwaya]
44. E2E "baza ne radi" dokazan krivom lozinkom, ne ugašenom bazom (3 s `acquire_timeout` nije izmjeren). Task 8 #5, L90. [prije Railwaya]
45. `vite build` u CI-ju. Final minori, L96. [ekrani]

## Ostalo

46. `Cargo.lock` ima neaktivne sqlx-sqlite/libsqlite3-sys unose; `cargo deny` s all-features ih može vidjeti (lokalno prolazi). Task 4 note, L58. [možda nikad]
47. README: uputa za `sqlx-cli`. Final minori, L96. [možda nikad]

## Zbroj po oznaci

| Oznaka | Stavki |
|---|---|
| [jezgra] | 15 |
| [dnevnik] | 9 |
| [AI] | 0 |
| [ekrani] | 7 |
| [prije Railwaya] | 10 |
| [možda nikad] | 5 |
| [riješeno] | 1 |
| **Ukupno** | **47** |

Provjera brojanja (Grep): ledger ima 34 × `minor (deferred)` + 1 × `note:` + 1 × Final #3 (L95) + 11 stavki u retku "ostali minori" (L96) = **47**; ova datoteka ima **47**. Final #1 i #2 su popravljeni i izvan broja.
