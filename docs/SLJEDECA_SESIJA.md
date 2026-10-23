# Sljedeća sesija — Leonovi odgovori na plan, pa gradnja vala 0 (testovi)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 14 (24.10.): Leon izabrao **opseg pilota (a)** — profesorov tempo kao **grupa igara** (svaki student svoja igra, profesor sam
napravi grupu na stranici i dobije **profesorski kod**; [ADR-066](./records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md)).
Napisani su **jedan plan** [Pilot s profesorom](./plan/2026-10-24-pilot-s-profesorom.md) (🟨 čeka OK), dizajn [AI_TEMELJ](./architecture/AI_TEMELJ.md)
(neovisno pregledan) i dokument za fakultet [TROSKOVI_I_AI](./architecture/TROSKOVI_I_AI.md) (§8 za prezentaciju: cijena, Mistral,
putovi do besplatnog AI-ja). Plan popravka temelja je u [arhivi](./archive/2026-10-14-popravak-temelja.md). Ništa nije gradeno ni pushano.
GitHub `model-v2/sloj1` = `main` = b973b84; lokalno 1 commit sesije 14 (samo docs). Ledger: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` („SESIJA 14").

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 15) — odgovori na plan pa gradnja. Pročitaj docs/SLJEDECA_SESIJA.md, plan docs/plan/2026-10-24-pilot-s-profesorom.md (odjeljci „Pregled", „Pitanja za Leona", „Global Constraints", „Tokovi", VAL 0), ADR-066 i dno ledgera .superpowers/sdd/2026-10-14-popravak-temelja/progress.md („SESIJA 14").
Laptop je na punjaču. Push, spajanje u main, živi AI test i deploy samo uz moj izričit OK.
SURADNJA: ListAgents → svoje ime u .superpowers/sdd/SESIJE.md. Banka K8 čeka.
1) `df -h /c`, `bash scripts/baza.sh status` (ako ne radi: start), `tasklist | grep -i cargo`.
2) Postavi mi pitanja iz plana („Pitanja za Leona" 1, 2, 3, 8 kao anketa, preporuka prva; 4–7 i 9 tekstom, numerirano). Moje odgovore zapiši u ADR-066 (✅) i plan; plan dobiva status 🟩 AKTIVAN.
3) Na moje „kreni": novi ledger .superpowers/sdd/2026-10-24-pilot-s-profesorom/progress.md, pa VAL 0 — Task 0.1 RED-T i 0.2 AI-T (dva Rust toka) ∥ 0.3 WEB-T ∥ 0.4 SKRIPTE, sve u jednoj poruci, svaki tok svoje stablo ../hotelixsim-<tok> i svoj target/<tok>; 0.5 VRATA kad se oslobodi Rust slot. Čuvar testova prije graditelja, pregled prije spajanja, spajanje isti dan, lint + CI prije pusha.
4) Kraj vala 0: pokaži mi osnovicu troška (R4) i ispis opterećenja s jednim radnikom (R1) — prije vala 1.
Na kraju /kraj (check:docs, test:docs i lint EXIT 0).
```

## Cilj sesije

1. **Leonovi odgovori** na pitanja plana (situacije u grupi a/b, živi AI testovi, Playwright, migracije 0006–0008, mjerilo, lokal, mentor na Haiku,
   pretpostavke ADR-066 t. 4–13 i AI_TEMELJ §10) → ADR-066 ✅, plan 🟩.
2. **Val 0 = testovi** (~40 h rada, ~14 h zidno): opterećenje reda poslova (mora pasti s jednim radnikom), trošak po igri (osnovica za
   TROSKOVI_I_AI §3), lažni AI, web tokovi, provjera nakon deploya, vrata.
3. Ako ostane vremena: val 1 (AI temelj I + L1-15 presuda + „Nova igra" za zastarjelu igru).

## Upozorenja

- **Lint prije svakog commita** (`corepack pnpm lint`; crveno samo pod `.superpowers/` = B19).
- **Mentor kruga je najveća stavka troška** — cilj ≤ 0,25 $ po igri traži mentora na Haikuu (Leonov slijepi pregled u valu 3).
- **Anthropicov API obrađuje u SAD-u** (30 dana, bez učenja); EU = Bedrock Frankfurt (+10 %). Mistral nije besplatan put.
- Testovi koji opisuju budući temelj (R1 s trakama) stoje `#[ignore = "zelen nakon AI-2"]` — čuvar za val 1, ne „lažno zeleno".
- Osnovica balansa `nakon-pilot-4/`; rub lanca 0,04: svaka promjena sadržaja vrti `--check` 1000 tjedno i mjesečno.
- Disk 50 GB (24.10.: 82 GB slobodno), svako stablo svoj `CARGO_TARGET_DIR`, najviše 2 Rust toka; omotači `cargo run … serve` ne broje se (B26).
- `grep -r` po cijelom repou zapinje na `target/` — koristi `git grep` ili Grep alat.
- Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- Pitanja plana 1–9 (gore). **Prezentacija fakultetu:** TROSKOVI_I_AI §8 (točke za slajdove i 3 pitanja fakultetu; brojke su procjena do testa troška).
- Pilot: Railway upozorenje 7 $ / strop 15 $, tko pokreće tjedni `pg_dump`, tekst privatnosti (dio 8 čeka profesore).
- Video u Higgsfieldu (kasnije). Pretpostavke 🟨: ADR-066 t. 4–13, ADR-064 t. 2–7, ADR-063 t. 7–19, ADR-062 t. 10–18, ADR-061 t. 7–12, ADR-060 t. 17–20, ADR-059, ADR-058.
- Nalaz E4 (5 pitanja), nacrt rizika C3, uzorci banke, slijepi pregled tekstova B4a–c, ocjene 40 odgovora za mjerilo.
