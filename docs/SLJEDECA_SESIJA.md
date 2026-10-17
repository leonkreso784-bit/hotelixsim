# Sljedeća sesija — CI nakon PRAVILO-b, dovršetak PRAVILO-c, banka, uniforme vanjskih, L1-13

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Spojeni su IZGLED (a+b) 872ff46 i PRAVILO-b ee6de80 u `model-v2/sloj1` (17.10.; PRAVILO-a 9a275d8 je na GitHubu). **Ništa od 17.10. nije pushano**:
20 commita ispred `origin/model-v2/sloj1` s docs-commitom kraja (provjeri `git status -sb`). PRAVILO-c je napola: čuvar 2c7af1a, graditelj zaustavljen s necommitanim kodom u `../hotelixsim-pravilo-c`.
Ledgeri (vjeruj im i git logu): `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` (popravak) i `.superpowers/sdd/banka-ljudi-k1/progress.md` (banka ljudi);
kako se šalju agenti: `upute-izvodacima.md` u mapi popravka (§5 je najnovije stanje; popis CI provjera = `.github/workflows/ci.yml`). Izvještaj ovog kraja: `kraj-17-10-8.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo. Ledgeri: .superpowers/sdd/2026-10-14-popravak-temelja/progress.md i .superpowers/sdd/banka-ljudi-k1/progress.md. Prvo pročitaj upute-izvodacima.md (§5 i §2).
Prvo `df -h /c` (ispod 50 GB ne gradi — prvo čišćenje). Laptop je na punjaču. Push samo uz moj OK.
Paralelna sesija banke ljudi radi istodobno: ona ima jedan Rust slot, ova jedan (najviše 2 cargo ukupno).
Redoslijed:
1) nakon-pravilo-b: ponovi `cargo test --workspace` (s bazom) i pogledaj diff balansa v2 tjedno i mjesečno prema nakon-izgled (smije se razlikovati samo blok klika).
2) PRAVILO-c: novi graditelj NASTAVLJA od necommitanog u ../hotelixsim-pravilo-c (ne piše ispočetka) → pregled (sonnet) → spajanje + CI → brisanje stabla i target/pravilo-c.
3) Banku ljudi spoji kad njezin ledger kaže „SPREMNO ZA SPAJANJE".
4) IZGLED-c: kartica i cigla za uniforme vanjskih (policija, vatrogasci, hitna i liječnici).
5) L1-13 po kartici task-14, s anketom (brojke A2 i nalaz I1: klik nosi 92–95 % vlastitog odgovora).
6) Moji pregledi (popis „Otvoreno za Leona" u SLJEDECA_SESIJA.md).
```

## Cilj sesije

1. **CI nakon PRAVILO-b dovršiti:** `cargo test --workspace` je pao na linkeru (EXIT 127, 0xC0000142 = okolina, [BUGS](./records/BUGS.md) B15); ponoviti s bazom
   `hotelixsim_agenti`. Diff balansa v2 prema `nakon-izgled/` pogledan je na `/kraj` (samo blok klika, stari brojevi isti); osnovica za dalje je `nakon-pravilo-b/`.
2. **PRAVILO-c** ([kartica](./plan/2026-10-14-popravak-temelja/task-12-pravilo-vlastiti-odgovor.md), dio c): necommitane izmjene u `OfferedAnswers.tsx`, `Verdict.tsx`,
   `fixtures.ts`, `schema_situations.rs`, `views.rs`, `hr.ts`; zadnji plan graditelja „dodati `click_capped` i u fiksturu s3, pa provjere". Pregled sonnet (samo prikaz).
3. **Spajanje banke ljudi** (pečat 000/001, vanjski 60, K3; K4 u tijeku) kad ledger banke kaže „SPREMNO ZA SPAJANJE"; zatim CI.
4. **IZGLED-c** (Leon 17.10.): uniforme iz uloge vanjskih (policajac, vatrogasac, djelatnik hitne), [ADR-056](./records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md) C10;
   web i API, banka ne mijenja format. Prvo kartica, čuvar prije graditelja.
5. **L1-13** ([task-14](./plan/2026-10-14-popravak-temelja/task-14-l1-13-kalibracija.md)): anketa s brojkama A2 i odluka o kliku (I1, [ADR-059](./records/DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md) t. 4).
   Prije spajanja `--check` 1000 tjedno i mjesečno.
6. **Kraj faze B:** pitati Leona za OK za spajanje `model-v2/sloj1` → `main` (deploy tek s platformom) i za push.

## Upozorenja

- **Disk:** granica 50 GB slobodnog; `target/<stablo>` briše se odmah nakon spajanja ([BUGS](./records/BUGS.md) B16). Svako stablo svoj `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>` (B12).
- **Jedan Rust slot po sesiji** (banka drži drugi); prije gradnje `tasklist | grep cargo`. Gradnja samo na punjaču.
- Lažno crveno pod opterećenjem (vitest „Timeout waiting for worker", cargo 0xC0000142) samo se ponovi (B15).
- Nalog izvođaču mora reći da testovi `app`/`api` traže `DATABASE_URL` (baza `hotelixsim_agenti`); bez nje 34 testa padaju.
- Testovi balansa ne traže iste primjerke ni monotoniju po sjemenu (B14). Tjedna težina B 48,9 % uz granicu 50 %.
- Stare probne v2 igre daju 409 (jezgra 2.5.0). U glavnom stablu je Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Nalaz E4** ([PROVJERA_ARHITEKTURE](./architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), 5 pitanja) i **nacrt rizika C3** ([07-poznate-osobe-rizik](./architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)).
- **ADR-058** tumačenja t. 10, 13, 19, 20; 🟨 [ADR-059](./records/DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md).
- **Anketa L1-13:** brojke (A2) i klik (I1) — treba li klik gubiti i mehanike ili je ~19 % nakon L1-13 prihvatljivo.
- **Pregled 10 lica IZGLED-a u igri** (C11: frizure, brada, naušnice); slijepi pregled tekstova B4a–c; ocjene 40 odgovora za mjerilo (`content/hotel/mjerilo/*.json`).
- Push `model-v2/sloj1` (20 commita) i spajanje u `main` na kraju faze B.
