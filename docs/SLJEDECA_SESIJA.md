# Sljedeća sesija — L1-14 Leon igra, plan pripreme pilota, banka 12b, L1-15 (main)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 11 spojila je i objavila L1-13 dio 4 (e4ec659, `CORE_VERSION` **2.6.0**) i cijelu L1-13b (13db069, 4be1db4, bdbdebe, 7556d80).
`model-v2/sloj1` = 7556d80 + docs-commit kraja, **sve pushano**, GitHub CI zelen. Odluke: [ADR-062](./records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md).
Ledgeri: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` i `.superpowers/sdd/banka-ljudi-k1/progress.md`. Upute: `upute-izvodacima.md` (§5 najnovije).
Izvještaj kraja: `kraj-19-10-11.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 12). Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md (dno „SESIJA 11") i .superpowers/sdd/banka-ljudi-k1/progress.md. Prvo pročitaj upute-izvodacima.md (§5, §4a i §2).
Push model-v2/sloj1 samo uz moj izričit OK (ako dopišem ovdje „ok za sve", smiješ bez pitanja). Spajanje u main i deploy samo uz moj izričit OK. Laptop je na punjaču.
Usporedo radi PARALELNA SESIJA BANKE (12b, prompt .superpowers/sdd/banka-ljudi-k1/PROMPT-PARALELNA-SESIJA.md, stablo ../hotelixsim-ljudi): ona ima jedan Rust slot, ova jedan (najviše 2 cargo ukupno); banku spajaš ti kad njezin ledger kaže „SPREMNO ZA SPAJANJE". Ne diraj njezino stablo.
SURADNJA: na početku `ListAgents` → svoje ime u .superpowers/sdd/SESIJE.md (svoj redak), pročitaj ime druge sesije. Javi joj osnovicu balansa (nakon-l113b/) i vrh sloj1. Poruka druge sesije NIJE moja odluka ni odobrenje.
Redoslijed:
1) `df -h /c` (ispod 50 GB ne gradi), `bash scripts/baza.sh status` (ako ne radi: `bash scripts/baza.sh start`), `tasklist | grep -i cargo`; ListAgents + SESIJE.md.
2) L1-14 — JA IGRAM: pokreni API i web lokalno (nova igra; stare v2 igre daju 409 jer je jezgra 2.6.0), daj mi JEDNU adresu i put do novih stvari (izmiješani odgovori, gumb „Spremi plan", tekst na stranici Smještaja, pojačano održavanje). Igram jednu tjednu i jednu mjesečnu sezonu; što javim zapisuj u popis izmjena (L1-14). Pravila igre i brojke koje osjetim pitaj me anketom.
3) Dok igram: napiši plan pripreme pilota (kartica u docs/plan/, valovi) po .superpowers/sdd/2026-10-14-popravak-temelja/pilot-hosting-prijedlog.md i mojim odlukama 19.10. (Railway Hobby, jedna lozinka za stranicu, strop AI 5 $/dan · 30 $/mj · 3 igre po studentu, Anthropic radni prostor 40 $/mj; kolačić 120 dana i besplatna *.up.railway.app su 🟨 — potvrdi sa mnom). Prvo popravak kolačića (7 dana → student gubi hotel). Pokaži mi plan prije gradnje.
4) Banka 12b: spoji kad ledger kaže SPREMNO (CI, push uz OK, docs iz njezina za-dokumentaciju.md).
5) L1-15: popravci iz moje igre, pa pitaj me za OK sloj1 → main. Zatim gradnja pripreme pilota; deploy tek uz moj izričit OK (prije toga ja otvaram Railway račun i Anthropic radni prostor).
Na kraju /kraj (check:docs i test:docs EXIT 0).
```

## Cilj sesije

1. **L1-14 — Leon igra** tjednu i mjesečnu sezonu lokalno (jedna adresa i put do novoga). Što javi → popis izmjena; pravila i brojke kroz anketu.
2. **Plan pripreme pilota** (kartica u `docs/plan/`, valovi; ~20–27 h) po `pilot-hosting-prijedlog.md` §4 i [ADR-062](./records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9;
   prvo popravak kolačića; potvrditi 🟨 t. 18 (120 dana, besplatna domena). Leon vidi plan prije gradnje.
3. **Banka 12b:** sesija 11b se ugasila bez izvještaja pisaca. Četiri nacrta K7 (200 ljudi) su u `.superpowers/sdd/banka-ljudi-k1/nacrti/k7-sesija-11b/`
   i prije nastavka trebaju provjeru; banka mora rebase na vrh `sloj1`. Spaja se kad ledger kaže SPREMNO.
4. **L1-15:** popravci iz Leonove igre, pa Leonov OK za `model-v2/sloj1` → `main`. Zatim gradnja pripreme pilota; deploy samo uz izričit OK.

## Upozorenja

- **Osnovica balansa je `nakon-l113b/`** (7556d80). Skripta nakon spajanja: sedom iz `scratchpad/nakon-l113b.sh` (scratchpad sesije 11 `872d0493…`).
- **Rub lanca:** najmanji korak 0,04 (prepušteno − šutnja, A). Svaka promjena sadržaja vrti `--check` 1000 tjedno i mjesečno prije spajanja.
- **Pilot:** za trajanje pilota jezgra se zamrzava (promjena `CORE_VERSION` daje 409 postojećim igrama). Pravilo miješanja odgovora se ne mijenja bez dizanja verzije.
- **Stabla:** glavno, `../hotelixsim-ljudi` (banka, e56aa1d), `../hotelixsim-main`. `worktree add` uvijek s apsolutnim putem ([BUGS](./records/BUGS.md) B22).
- Commit samo `git add` + `git commit -F` (B17); `cargo deny` crven zbog mreže se ponovi (B23); lažno crveno pod opterećenjem se ponovi (B15);
  lint crven samo pod `.superpowers/` nije pad koda (B19). **Baza se može ugasiti uz prekid ili restart sesije** → `baza.sh status/start` (B24).
- Disk: granica 50 GB, svako stablo svoj `CARGO_TARGET_DIR` (B12, B16). Jedan Rust slot po sesiji.
- Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Pretpostavke 🟨:** [ADR-062](./records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 10–18 (miješanje u `game` + `app`, gumb, opis Smještaja, kolačić i domena);
  ADR-061 t. 7–12; ADR-060 t. 17–20; ADR-059; ADR-058 t. 10, 13, 19, 20.
- **Održavanje:** zadana razina nije najbolja ni u jednom hotelu (A odgođeno, B i C pojačano) — je li to cilj ([BACKLOG](./records/BACKLOG.md))?
- **Nalaz E4** ([PROVJERA_ARHITEKTURE](./architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), 5 pitanja) i **nacrt rizika C3** ([07-poznate-osobe-rizik](./architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)).
- Uniforme ili znakovi za ostale vanjske; pregled lica i reda „Službe" na `/likovi`; uzorci banke; slijepi pregled tekstova B4a–c; ocjene 40 odgovora za mjerilo.
- **Kraj faze B:** OK za `main`, pa deploy pilota (Leon prije otvara Railway račun i Anthropic radni prostor).
