# Sljedeća sesija — L1-13 dijelovi 2–4, spajanje banke, kraj faze B i pilot

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 9 spojila je i objavila PRAVILO-c, IZGLED-c, L1-13 dijelove 1 i 1b i banku (280 ljudi). `model-v2/sloj1` = d8d991f + e700e7a + docs-commit kraja,
**sve pushano**, GitHub CI zelen. Leon je dao „ok za sve" za push sloj1, a deploy pilota odobrio je za kraj faze B
([ADR-060](./records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md)). Ledgeri: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` i
`.superpowers/sdd/banka-ljudi-k1/progress.md`. Upute: `upute-izvodacima.md` u mapi popravka (§5 je najnovije). Izvještaj kraja: `kraj-18-10-9.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 10). Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md i .superpowers/sdd/banka-ljudi-k1/progress.md. Prvo pročitaj upute-izvodacima.md (§5, §4a i §2).
Push model-v2/sloj1 smiješ bez pitanja (moj „ok za sve"); spajanje u main samo uz moj OK. Laptop je na punjaču.
Redoslijed:
1) `df -h /c` (ispod 50 GB ne gradi) i `bash scripts/baza.sh status`; `tasklist | grep -i cargo` (jedan Rust slot po sesiji).
2) Ledger banke: piše li „SPREMNO ZA SPAJANJE: 106ede1"? Ako da, spoji banku (530 ljudi, uloga lijecnik) + CI + push, pa prenesi njezin za-dokumentaciju.md (odjeljak „Paralelna sesija banke 9b") u docs.
3) Očisti ../hotelixsim-l113-2: prvo `git -C ../hotelixsim-l113-2 status` (smiju biti samo izmjene prekinutog čuvara, spremljene u l113-2-provjere/cuvar-prekinut-necommitano.diff), pa `checkout -- .` i `clean -fd`.
4) L1-13 dio 2 (značaj, task-14-2-znacaj.md): NOVI čuvar ispočetka → graditelj → pregled (opus) → spajanje + CI + push.
5) L1-13 dio 3 (gosti, task-14-3-gosti.md), isti tok.
6) L1-13 dio 4 (kalibracija, task-14-4-kalibracija.md): tablica A2 i tablica prije/poslije Leonu PRIJE spajanja; CORE_VERSION 2.6.0; ponovi mjerenje klika (ADR-060 t. 18).
7) Kraj faze B: pitaj me za OK za sloj1 → main; zatim deploy pilota — prije objave prijedlog hostinga s €/mj i pristupni kod / ograničenje AI poziva.
Na kraju /kraj (check:docs i test:docs EXIT 0).
```

## Cilj sesije

1. **Banka:** ledger banke kaže „SPREMNO ZA SPAJANJE: 106ede1" (530 ljudi: radnici 210, gosti 210, vanjski 110 s 15 `lijecnik`; CI `ci-k5/` sav EXIT 0).
   Spaja glavna sesija. Uz spajanje ide u docs: redak `lijecnik` u BANKA_LJUDI §1.5 (14 → 15 uloga), brojke §2.4, BACKLOG (Grgurević, kalupi, kvota weba).
2. **L1-13 dio 2 — značaj** ([kartica](./plan/2026-10-14-popravak-temelja/task-14-2-znacaj.md)): razine ključna / važna / svakodnevna iz `weight` (ADR-060 t. 8).
   Prekinuti čuvar se **ne nastavlja**. Njegov diff je samo trag: imao je i `src/`, vjerojatno privremenu izvedbu za dokaz ostvarivosti. `target/l113-2` ostaje.
3. **Dio 3 — gosti** ([kartica](./plan/2026-10-14-popravak-temelja/task-14-3-gosti.md)): „s gostima" = uloga gost ili odjel uz goste (27/55, t. 11).
4. **Dio 4 — kalibracija** ([kartica](./plan/2026-10-14-popravak-temelja/task-14-4-kalibracija.md)): uključuje kvar (uz ključ tjedno B 54,6 % > 50) i A8.
   Ponovno mjeri klik, a obje tablice idu Leonu prije spajanja. `CORE_VERSION` 2.6.0, stare probne v2 igre tada daju 409.
5. **Kraj faze B:** Leonov OK za `model-v2/sloj1` → `main`, pa **deploy pilota** ([ADR-060](./records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12–14).
   Prije objave: prijedlog hostinga s €/mj (najjeftinija varijanta koja čuva temelj) i zaštita AI troška, jer prijave nema ([BACKLOG](./records/BACKLOG.md) [pilot]).

## Upozorenja

- **Osnovica balansa je `nakon-l113-1b/`** (sve EXIT 0). Skripta nakon spajanja je `scratchpad/nakon-<cigla>.sh`, napravljena sedom iz prethodne; `cmp` ide prema prethodnoj osnovici.
- **Commit samo `git add` + `git commit -F <datoteka>`**, nikad `-am` uz `-F` ([BUGS](./records/BUGS.md) B17). Graditelj koji dira testove čuvara navodi svaku izmjenu, a pregled je presuđuje (B18).
- Ako balans ne drži branu, prije ankete Leonu ide **dijagnoza sastava Δ** (B20). Lokalni `lint` crven samo pod `.superpowers/` nije pad koda (B19).
- **Disk:** granica 50 GB; `target/<stablo>` se briše odmah nakon spajanja (B16). Svako stablo ima svoj `CARGO_TARGET_DIR` (B12). Jedan Rust slot po sesiji.
- Lažno crveno pod opterećenjem (vitest timeout, 0xC0000142) samo se ponovi (B15). Testovi `app`/`api` traže `DATABASE_URL` (baza `hotelixsim_agenti`).
- Railway: projekti KuhAI brišu se 20.10. 01:23 UTC (B21). Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Nalaz E4** ([PROVJERA_ARHITEKTURE](./architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), 5 pitanja) i **nacrt rizika C3** ([07-poznate-osobe-rizik](./architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)).
- **Pretpostavke 🟨:** ADR-058 t. 10, 13, 19, 20; [ADR-059](./records/DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md); [ADR-060](./records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 17–20.
- **Uzorci banke** (`.superpowers/sdd/banka-ljudi-k1/`): `uzorak-k4-k5.md`, `pregled-vanjski-10.md` (5 pitanja), uzorak K3, Excel `banka-ljudi-530.xlsx` na radnoj površini;
  rupa 011–050 u seriji-000 (preporuka: ostaviti), Grgurević u zapečaćenoj 001; talijanski kod lokalnih vanjskih i liječnici u idućim serijama.
- **Pregled lica i uniformi na `/likovi`** (10–20 lica jednog kruga, red „Službe", boje hitne i policije); slijepi pregled tekstova B4a–c; ocjene 40 odgovora za mjerilo.
- **L1-13 dio 4:** tablica A2 i prije/poslije (težina vlastitog i klika). **Kraj faze B:** OK za `main` i prijedlog hostinga pilota.
