# Sljedeća sesija — PLAN: jedan plan za L1-15, pilot i fazu C, pa gradnja punom snagom

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 13 (20.–23.10.): video za profesora v2 (izvan repoa), igra provjerena za prezentaciju 20.10., zapisi sesije 12
([ADR-063](./records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) pilot) i **novo pravilo za planove**
([ADR-064](./records/DECISIONS/ADR-064-plan-je-jedna-datoteka-do-1000-redaka.md)): plan je **jedna datoteka do 1000 redaka**, `docs/plan/` ima 5 datoteka.
GitHub `model-v2/sloj1` = `main` = 9b6f047; lokalni commit sesije 13 čeka Leonov OK za push. Ledger: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 14) — PLAN pa gradnja. Pročitaj docs/SLJEDECA_SESIJA.md, ADR-064 (plan = jedna datoteka ≤ 1000 r.), ADR-065 (moje odluke 24.10.), dno ledgera .superpowers/sdd/2026-10-14-popravak-temelja/progress.md („SESIJA 13") i podloge-plana-24-10.md u istoj mapi.
Laptop je na punjaču. Push, spajanje u main i deploy samo uz moj izričit OK.
SURADNJA: ListAgents → svoje ime u .superpowers/sdd/SESIJE.md. Banka K8 čeka.
1) `df -h /c`, `bash scripts/baza.sh status` (ako ne radi: start), `tasklist | grep -i cargo`.
2) Prvo me pitaj OPSEG PILOTA (ADR-065 t. 8): (a) pilot s profesorovim tempom (profesor napravi igru, krug po vremenu ili gumbom, student automatski ili sam „dalje") ili (b) kratki pilot bez profesora pa tempo odmah iza. Analiza prije prijedloga (paralelni read-only agenti; ROADMAP kolosijek PLATFORMA, plan 2026-11-platforma, ADR-037).
2b) AI TEMELJ (ADR-065 t. 12–14): brainstorming kreativnog AI sloja koji podnosi opterećenje i jeftin je (ai-temelj-ideje-24-10.md, temelj-opterecenja-24-10.md, ai-trosak-istrazivanje-24-10.md) → dizajn dokument u docs/architecture/ → neovisni pregled → cigle u plan.
3) Napiši JEDAN plan u docs/plan/ (jedna datoteka) po mom odgovoru: VAL 0 = TESTOVI (ADR-065 t. 11, revizija-testova-24-10.md: opterećenje 40 studenata, AI trošak po igri, web tokovi, dim nakon deploya, slabi testovi), pa L1-15 (presuda 8000 + iskočni prozor; mentor „niže plaće"; mentor bez obećanja povrata ulaganja; celijakija rečenicom), zaštita pilota (više AI radnika, poruka i gumb „Nova igra" za zastarjelu igru), profesorov tempo ako (a), lokal kao mehanika i generator događaja (gdje u redoslijedu), pilot dio 9, faza C poslije. Sati i valovi. Pokaži mi plan prije gradnje; plan popravka temelja tada u arhivu.
4) Nakon mog OK: gradnja (kratke grane, čuvar testova prije graditelja).
Na kraju /kraj (check:docs, test:docs i lint EXIT 0).
```

## Cilj sesije

1. **Odluka o opsegu pilota** ([ADR-065](./records/DECISIONS/ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md) t. 1 i 8): profesor vodi tempo igre
   (krug po vremenu ili gumbom; student automatski ili sam „dalje") — u pilotu ili odmah iza njega.
2. **Jedan plan** (≤ 1000 r.) s redoslijedom: **val 0 testovi** (ADR-065 t. 11; revizija `revizija-testova-24-10.md`), L1-15 (~8–14 h), zaštita pilota (~4–5 h: više AI radnika, izlaz iz zastarjele igre),
   profesorov tempo (ako je u pilotu), lokal kao mehanika i generator događaja (ADR-065 t. 2 i 6), pilot dio 9 (~2,5–4 h + Leon ~1,5 h);
   faza C poslije pilota (~50–80 h: C6 → C1 → C2 → razred → C3, C5). Podloge s datotekama i satima: `podloge-plana-24-10.md`.
3. **Gradnja** tek nakon Leonova OK na plan. Dio 8 (privatnost) čeka profesore.

## Način rada (Leon 24.10.: „strukturno i pametno, dobrom kvalitetnom brzinom, s dosta agenata")

- Plan se dijeli u **tokove po vlasništvu datoteka** (npr. AI/presuda · mentor · web · sadržaj · radnici/red), svaki tok svoje stablo
  `../hotelixsim-<tok>` i svoj `target/<tok>`. U svakom valu **odjednom** pokreni sve neovisne tokove u jednoj poruci.
- Istodobno najviše **2 agenta koji prevode Rust**; uz njih bez ograničenja web, sadržaj i docs graditelji te read-only čuvari i pregledi.
- Kvaliteta se ne reže: čuvar testova prije graditelja, pregled prije spajanja, spajanje isti dan, **lint + CI prije pusha**.
- Glavna sesija samo orkestrira (nalozi, presude, spajanje) i čuva kontekst; izlazi agenata idu u datoteke ledgera.

## Upozorenja

- **Lint prije svakog commita** (`corepack pnpm lint`; crveno samo pod `.superpowers/` = B19). Sesija 13 je pushala neformatiran test → CI crven.
- **Plan se ne razdvaja u mapu** — brana pada na podmapu u `docs/plan/` (pravilo 13). Gotove kartice sele u `docs/archive/<plan>-kartice/`.
- **Osnovica balansa `nakon-pilot-4/`** (= nakon-l113b brojke). Rub lanca 0,04: svaka promjena sadržaja vrti `--check` 1000 tjedno i mjesečno.
- **Lokalni API** nakon pusha uvijek ponovo izgraditi i `migrate` (B30); zdravlje bez `events` = novi kod.
- **Pilot:** jezgra se zamrzava za trajanje pilota; slika se ne dijeli dok proba ne pokaže 401 bez lozinke.
- Disk 50 GB, svako stablo svoj `CARGO_TARGET_DIR`, jedan Rust slot po sesiji; omotači `cargo run … serve` ne broje se (B26).
- Leonova necommitana `docs/architecture/PLATFORMA/03-prijava.md` — ne diraj i ne commitaj.

## Otvoreno za Leona

- **Video:** pjesma u MP3 (Leon dostavlja), tempo glasa A (+8 %/+2 Hz) ili B (+12 %/+4 Hz), izgovor „Hoteliks Sim" (`hotelixsim-video/proba-glas/`);
  zatim render s glazbom i „5/5" na udarcu. Tekst za Leonovo pričanje po slijedu kadrova je u razgovoru sesije 13.
- **Pretpostavke 🟨:** [ADR-064](./records/DECISIONS/ADR-064-plan-je-jedna-datoteka-do-1000-redaka.md) t. 2–7, [ADR-063](./records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) t. 7–19,
  [ADR-062](./records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 10–18, ADR-061 t. 7–12, ADR-060 t. 17–20, ADR-059, ADR-058 t. 10, 13, 19, 20.
- **Pilot:** Railway upozorenje 7 $ / strop 15 $, tko pokreće tjedni `pg_dump`, tekst privatnosti i kontakt.
- Nalaz E4 (5 pitanja), nacrt rizika C3, uzorci banke, slijepi pregled tekstova B4a–c, ocjene 40 odgovora za mjerilo.
