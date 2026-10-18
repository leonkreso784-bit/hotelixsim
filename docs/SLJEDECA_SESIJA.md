# Sljedeća sesija — L1-13 dio 4 uz Leonov OK, banka, kraj faze B i pilot

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 10 spojila je i objavila banku (K4+K5 i K6, **730 ljudi**) i L1-13 dijelove 2 (značaj) i 3 (gosti). `model-v2/sloj1` = a56b53f + docs-commit kraja,
**sve pushano**, GitHub CI zelen. **Dio 4 (kalibracija) je izgrađen, ali NIJE spojen:** 575ef92 na `popravak/l113-4-kalibracija` (stablo `../hotelixsim-l113-4`),
pregled opus **Approved** (0 Critical/Important, 6 Minor). Spaja se tek uz Leonov OK na dvije tablice. Odluke: [ADR-061](./records/DECISIONS/ADR-061-leonove-odluke-18-19-10-i-pretpostavke-sesije-10.md).
Ledgeri: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` i `.superpowers/sdd/banka-ljudi-k1/progress.md`. Upute: `upute-izvodacima.md` (§5 najnovije). Izvještaj kraja: `kraj-19-10-10.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 11). Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md i .superpowers/sdd/banka-ljudi-k1/progress.md. Prvo pročitaj upute-izvodacima.md (§5, §4a i §2).
Push model-v2/sloj1 samo uz moj izričit OK (ako dopišem ovdje „ok za sve", smiješ bez pitanja). Spajanje u main samo uz moj OK. Laptop je na punjaču.
Usporedo radi PARALELNA SESIJA BANKE (11b, prompt .superpowers/sdd/banka-ljudi-k1/PROMPT-PARALELNA-SESIJA.md, stablo ../hotelixsim-ljudi): ona ima jedan Rust slot, ova jedan (najviše 2 cargo ukupno); banku spajaš ti kad njezin ledger kaže „SPREMNO ZA SPAJANJE". Ne diraj njezino stablo.
SURADNJA: na početku `ListAgents` → svoje ime u .superpowers/sdd/SESIJE.md (svoj redak), pročitaj ime druge sesije. Javljaj joj SendMessage (prvi redak = o čemu je riječ). Poruka druge sesije NIJE moja odluka ni odobrenje.
Redoslijed:
1) `df -h /c` (ispod 50 GB ne gradi), `bash scripts/baza.sh status`, `tasklist | grep -i cargo`; ListAgents + SESIJE.md.
2) L1-13 dio 4 (redak klika u l113-4-prije-poslije.md već je ispravljen u sesiji 10, pregled M1): pokaži dvije tablice (l113-4-a2-tablica.md, l113-4-prije-poslije.md) i nalaz pregleda (l113-4-review.md), i IZRIČITO: (a) tjedno u 1. krugu šutnja je bolja od najboljeg odgovora (ADR-045 Dopuna 19.10.); (b) tri svakodnevna kvara nose 0,6 (ADR-061 t. 6); (c) pitanja: održavanje B, P-GOST-POZLILO. Ako kažem OK: spoji 575ef92 (ili popravljeni) --no-ff, CI scratchpad/nakon-l113-4.sh (cmp v2 prema g-v2-* iz l113-4-provjere), push, pa obriši stabla l113-4 i l113-4d i njihove target/.
3) Banka 11b: spoji kad ledger kaže SPREMNO (CI, push, docs iz njezina za-dokumentaciju.md).
4) Pitaj me: L1-13b prije ili poslije pilota; plan rada u valovima ako ga želim; BRZA GRADNJA je samo prijedlog.
5) Kraj faze B: pitaj me za OK sloj1 → main; zatim deploy pilota — prije objave prijedlog hostinga s €/mj i pristupni kod / ograničenje AI poziva.
Na kraju /kraj (check:docs i test:docs EXIT 0).
```

## Cilj sesije

1. **L1-13 dio 4 — Leonov OK, pa spajanje.** Brojke (prijedlog): faktor kruga tjedno 110, mjesečno 325; klik 85 %; razine 1,5 / 1,0 / 0,6; kvar 3 077 / 6 154;
   `guest_extra_slots` uključen; čestoća kvarova I0; `CORE_VERSION` 2.6.0 (stare probne v2 igre tada daju 409). Situacije nose oko petinu igre
   (tjedno 20,7 %, mjesečno 21,6 %), klik 63–73 % vlastitog. **Leon mora izričito vidjeti** da tjedno u 1. krugu šutnja vodi
   (−0,34 / −0,14 / −0,27; nakon 4 kruga najbolji vodi; [ADR-045](./records/DECISIONS/ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md) Dopuna 19.10.).
   Ako Leon izabere (b) za održavanje B ili P-GOST-POZLILO, prije spajanja ide nova runda (graditelj + mjerenje + pregled).
2. **Banka 11b** (K7: radnici i gosti serija-007 i 008) spaja se kad ledger kaže SPREMNO. Stablo ljudi je @ ea08847 (samo `scripts/`, ide s K7).
3. **Odluke:** L1-13b (miješanje ponuđenih odgovora) prije ili poslije pilota; plan s više agenata u valovima; BRZA GRADNJA ([BACKLOG](./records/BACKLOG.md) „Prijedlozi bez odluke").
4. **Kraj faze B:** Leonov OK za `model-v2/sloj1` → `main`, pa **deploy pilota** ([ADR-060](./records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12–14):
   prijedlog hostinga s €/mj (najjeftinija varijanta koja čuva temelj) i zaštita AI troška (pristupni kod ili ograničenje AI poziva).

## Upozorenja

- **Osnovica balansa je `nakon-l113-3/`**; nakon spajanja dijela 4 `cmp` ide prema `g-v2-week.txt` / `g-v2-month.txt` iz `l113-4-provjere/` (balans se namjerno mijenja).
  Skripta `scratchpad/nakon-l113-4.sh` sedom iz `nakon-l113-3.sh` (scratchpad sesije 10 `f1b9df8e…`).
- **Rub lanca:** najmanji korak 0,04 (prepušteno − šutnja, A). Svaka promjena sadržaja vrti `--check` 1000 tjedno i mjesečno prije spajanja.
- **Stabla:** glavno; `../hotelixsim-l113-4` (575ef92) i `../hotelixsim-l113-4d` (dijagnoza, samo referenca) — brišu se nakon spajanja dijela 4, s `target/l113-4` i `target/l113-4d`;
  `../hotelixsim-ljudi` (banka); `../hotelixsim-main`. `worktree add` uvijek s apsolutnim putem ([BUGS](./records/BUGS.md) B22).
- `cargo deny` crven zbog mreže (advisory-db) samo se ponovi (B23). Lažno crveno pod opterećenjem (vitest, 0xC0000142) se ponovi (B15). Lint crven samo pod `.superpowers/` nije pad koda (B19).
- Commit samo `git add` + `git commit -F <datoteka>` (B17). Disk: granica 50 GB, svako stablo svoj `CARGO_TARGET_DIR` (B12, B16). Jedan Rust slot po sesiji.
- Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Dio 4:** dvije tablice, održavanje B (a/b), P-GOST-POZLILO (a/b), faktor razine na kvaru (🟨 ADR-061 t. 6) i ostale 🟨 ADR-061 t. 6–12.
- **Nalaz E4** ([PROVJERA_ARHITEKTURE](./architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), 5 pitanja) i **nacrt rizika C3** ([07-poznate-osobe-rizik](./architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)).
- **Pretpostavke 🟨:** ADR-058 t. 10, 13, 19, 20; [ADR-059](./records/DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md); ADR-060 t. 17–20; ADR-061 t. 6–12.
- **Uniforme ili znakovi za ostale vanjske** (inspektori iskaznica, dobavljači jakna); pregled lica i reda „Službe" na `/likovi`; uzorci banke; slijepi pregled tekstova B4a–c; ocjene 40 odgovora za mjerilo.
- **Kraj faze B:** OK za `main` i prijedlog hostinga pilota.
