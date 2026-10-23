# Sljedeća sesija — PLAN: jedan plan za L1-15, pilot i fazu C, pa gradnja punom snagom

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

Sesija 13 (20.–23.10.): video za profesora v2 (izvan repoa), igra provjerena za prezentaciju 20.10., zapisi sesije 12
([ADR-063](./records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) pilot) i **novo pravilo za planove**
([ADR-064](./records/DECISIONS/ADR-064-plan-je-jedna-datoteka-do-1000-redaka.md)): plan je **jedna datoteka do 1000 redaka**, `docs/plan/` ima 5 datoteka.
GitHub `model-v2/sloj1` = `main` = 9b6f047; lokalni commit sesije 13 čeka Leonov OK za push. Ledger: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`.

## Prompt za početak (Leon zalijepi nakon `/clear` + `/newsession`)

```text
Nastavljamo (sesija 14) — PLAN pa gradnja. Pročitaj docs/SLJEDECA_SESIJA.md, dno ledgera .superpowers/sdd/2026-10-14-popravak-temelja/progress.md („SESIJA 13") i ADR-064 (plan = jedna datoteka do 1000 redaka).
Laptop je na punjaču. Push, spajanje u main i deploy samo uz moj izričit OK.
SURADNJA: ListAgents → svoje ime u .superpowers/sdd/SESIJE.md. Banka K8 ne kreće dok ja ne odlučim o generatoru ljudi.
1) `df -h /c`, `bash scripts/baza.sh status`, `tasklist | grep -i cargo`.
2) PLAN (prvo analiza, paralelni read-only agenti): napiši JEDAN plan u docs/plan/ (jedna datoteka, ≤ 1000 r.) koji zamjenjuje otvoreni dio plana popravka temelja: L1-15 popravci iz moje igre (l114-popis-izmjena.md; L14-1 presuda 8000 tokena + iskočni prozor; mentor „niže plaće"; stručni pojmovi), pilot dijelovi 8 (tekst privatnosti — pitaj me) i 9 (Railway), faza C (C1–C6), i redoslijed s valovima i satima. Pokaži mi plan prije gradnje.
3) Moje odluke koje čekaju: generator ljudi (docs/architecture/BANKA_LJUDI/08-generator-ljudi.md, 5 pitanja) i pitanja iz BACKLOG „Otvoreno za Leona" — anketom gdje su prave opcije.
4) Nakon mog OK: gradnja po planu (kratke grane, čuvar testova prije graditelja).
Na kraju /kraj (check:docs i test:docs EXIT 0).
```

## Cilj sesije

1. **Jedan plan** (`docs/plan/<datum>-<ime>.md`, ≤ 1000 r., taskovi kao odjeljci) za: **L1-15** (popravci iz Leonove igre, pa OK za `main`),
   **pilot** dio 8 (tekst privatnosti + kontakt, čeka Leona) i dio 9 (postava na Railwayu, obvezno ADR-063 C), **faza C** (C1–C6 iz
   [plana popravka](./plan/2026-10-14-popravak-temelja.md)). Plan popravka nakon toga ide u arhivu (ispunjen) ili ostaje pauziran.
2. **Leonove odluke:** generator ljudi (5 pitanja), L14-4 stručni pojmovi, tri pitanja iz voditeljeve igre ([BACKLOG](./records/BACKLOG.md)).
3. **Gradnja** tek nakon Leonova OK na plan.

## Upozorenja

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
