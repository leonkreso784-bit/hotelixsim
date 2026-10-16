# Leonov pregled — sve što čeka tvoju riječ, kao pitanja s preporukom

**Status:** 🟩 AKTIVAN · **Područje:** kvaliteta · **Otvoren:** 2026-10-17 · **Za:** Leon — dva dana odluka nakon PRAVILO-a i
IZGLED-a, prije faze C

> Područje je „kvaliteta" samo zato što brana `check:docs` zna četiri područja (igra, platforma, kvaliteta, dokumentacija),
> a u kvaliteti nema drugog aktivnog plana. Ovo nije plan gradnje nego popis tvojih odluka; kad su odgovorene, ide u arhivu.

## Zašto

Od 04.10. Claude radi bez pitanja i ono što bi trebao odlučiti ti zapisuje kao pretpostavku 🟨 (ADR-005). Nakupilo ih se
preko stotinu, u dvadesetak ADR-ova, karticama, nacrtima i BACKLOG-u. Ti si 16.10. prihvatio da prije faze C uzmemo dva
dana samo za tvoje odluke, da gradimo na njima, a ne na pretpostavkama. Ovdje je sve skupljeno u **43 numerirana pitanja**,
svako s preporukom.

## Kako odgovoriti

- Napiši broj i: `ok` (prihvaćaš preporuku), `ne` (biraš „Inače" ili kažeš svoje) ili `napomena: …`.
  Primjer: „A1 ok, A2 ne – 3 kvara održan, 8 zapušten, C2 vidi napomenu".
- Skupna pitanja (A5, A13, C8, F1) prihvaćaš jednim „ok"; ako ti smeta samo jedna točka: „A13 ok osim d".
- Može i kroz Claude: reci „pitaj me blok 1" i voditelj pitanja s pravim izborom postavi kao anketu (preporuka prva);
  brojke i činjenice (A2, A10, A11) pita tekstom.
- „Ne znam" je valjan odgovor: pretpostavka ostaje 🟨, a voditelj zapiše što bi pomoglo (npr. pitanje profesoru).
- Voditelj na `/kraj` odgovore pretvara u ADR (✅ Leonova odluka), briše riješeno iz BACKLOG-a i skraćuje popis 🟨 u CLAUDE.md.

## Redoslijed (prvo ono što koči gradnju)

| blok | pitanja | zašto tada | min |
|---|---|---|---|
| 1 | A1–A4 | L1-13 (jedna zajednička kalibracija brojki) ne može krenuti bez cilja | 15 |
| 2 | A5–A6 | PRAVILO-a se upravo gradi; prag je jedna brojka prije spajanja | 5 |
| 3 | C1–C2 | pečat serija banke ljudi; dok nisu zapečaćene, ne idu u igru | 70–90 |
| 4 | B1 | slijepi pregled tekstova koči L1-15 (završnu provjeru faze B) | 60–90 |
| 5 | B2 | napomene uz tekstove, tek nakon B1 | 10 |
| 6 | A7–A13 | pravila koja student osjeti i brojke iz prakse | 25 |
| 7 | C3–C11 | sitnice banke, vanjski likovi, izgled | 15 |
| 8 | B3–B5 | sadržaj situacija i mjerilo AI-ja (40 ocjena su zaseban sat) | 15 (+40) |
| 9 | D1–D5, E1–E7 | voditelj odjela, tržište, platforma | 20 |
| 10 | F1–F2 | tehničko i već riješeno, jedno „ok" | 5 |

Ukupno ~4–5 h (s mjerilom ~5–6 h): npr. prvi dan blokovi 1–4, drugi dan 5–10.

## Dijelovi

| grupa | datoteka | pitanja |
|---|---|---|
| A — pravila igre i brojke | [01-pravila-i-brojke.md](./2026-10-17-leonov-pregled/01-pravila-i-brojke.md) | A1–A13 (13) |
| B — tekstovi odgovora i mjerilo AI-ja | [02-tekstovi.md](./2026-10-17-leonov-pregled/02-tekstovi.md) | B1–B5 (5) |
| C — banka ljudi i izgled likova | [03-banka-ljudi.md](./2026-10-17-leonov-pregled/03-banka-ljudi.md) | C1–C11 (11) |
| D — voditelj odjela i tržište · E — platforma i ostalo | [04-voditelj-i-platforma.md](./2026-10-17-leonov-pregled/04-voditelj-i-platforma.md) | D1–D5, E1–E7 (12) |
| F — tehničko i već riješeno | [05-tehnicko.md](./2026-10-17-leonov-pregled/05-tehnicko.md) | F1–F2 (2 skupna, ~25 stavki) |

Svako pitanje ima isti oblik: **Što je** (za nekoga tko pojam ne zna) · **Pretpostavili smo** · **Preporuka** (prva) ·
**Inače** i cijena · **Vidi** (ADR, dokument ili datoteka).

## Pojmovi (jednom, na početku)

- **🟨 pretpostavka:** odluka koju je Claude donio umjesto tebe da gradnja ne stane; vrijedi dok je ne potvrdiš ili promijeniš.
- **L1-13:** jedna zajednička kalibracija svih brojki igre nakon faze B (popravak, kvarovi, težina situacija, šutnja).
- **Težina situacija:** koliki dio razlike između dobre i loše sezone dolazi od odgovora na situacije, a ne od plana.
- **Oznaka:** tekst ponuđenog odgovora koji student vidi. **Rubrika:** kriteriji po kojima AI sudac ocjenjuje odgovor.
- **Balans / `--check`:** računalo odigra 1000 sezona i provjeri da igra ostaje poštena; pada li, brojka se ne smije spojiti.
- **Banka ljudi, serija, pečat:** vidi uvod dijela C.

## Kamo ide odgovor

| pitanja | ide u |
|---|---|
| A1–A4 | novi ADR „Leonove odluke za L1-13" + plan popravka red L1-13 |
| A5–A6 | ADR-055 t. 1 i kartica PRAVILO (ADR sesije 7 za B13) |
| A7, A8 | ADR-051 t. 2–3; ADR-015 t. 2 |
| A9 | novi ADR „strani radnici: plaća i obuka"; MODEL_V2 §5.4, ROADMAP/igra L2a-6 i L2b-3, ADR-028 (pravilo voditelja) |
| A10, A11 | ADR-032 t. 9, ADR-033; `content/hotel/v2/departments.json` |
| A12 | BACKLOG (zatvaranje), DEMO §12 „Za Leonovu provjeru" |
| A13 | ADR-013, 019, 022, 023, 025, 026, 027, 029, 031, 035, 036, 048 → ✅ |
| B1–B2 | tri dokumenta `b4?-pregled-tekstova.md` → B4-ispravci; ADR-055 t. 6 |
| B3–B5 | BACKLOG „Sadržaj situacija"; `content/hotel/` (situations, catalog, mjerilo); SITUACIJE_S1b_S5-S8 |
| C1–C2 | ledger banke ljudi (pečat); ADR-055 t. 3; `raspodjele.json` |
| C3–C8 | `zabranjeno.json`, serije banke; BANKA_LJUDI §1–§3 |
| C9–C10 | BANKA_LJUDI §6 (pitanja 12–16) i novi ADR „vanjski u banci ljudi" |
| C11 | ADR IZGLED (piše `/kraj`) |
| D1–D5 | VODITELJ_I_TRZISTE §3 → ADR + ROADMAP/igra (L2a-5, faza C) |
| E1 | novi ADR „prijava: sesije" + ARHITEKTURA §12, PLATFORMA dio 08 (L10) |
| E2, E3 | PITANJA_ZA_PROFESORE; plan platforme §7 |
| E4, E5 | ADR-003; ROADMAP |
| E6, E7 | BACKLOG „vizija" → nacrt PLATFORME; DIZAJN §3 |
| F1, F2 | svi ADR-ovi iz tablice F1 → ✅; BACKLOG „Otvoreno za Leona" |

## Izvori i što nije uključeno

Pročitano 17.10.: svih 23 ADR-a sa statusom 🟨 ([DECISIONS.md](../records/DECISIONS.md)) i 🟨 dijelovi ADR-028,
kartice PRAVILO i IZGLED (`task-12`, `task-13`), [BACKLOG](../records/BACKLOG.md) „Otvoreno za Leona",
[SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md), [VODITELJ_I_TRZISTE](../architecture/VODITELJ_I_TRZISTE.md) §3,
[BANKA_LJUDI §6](../architecture/BANKA_LJUDI/06-pitanja.md), ledgeri i izvještaji u `.superpowers/sdd/2026-10-14-popravak-temelja/`
i `.superpowers/sdd/banka-ljudi-k1/` (`k2-*-report.md`, `k2-review.md`, `pregled-20.md`, `nacrt-vanjski-report.md`).

Nije uključeno: odluke koje si već donio (✅ ADR-ovi i odluke sesije 7: redoslijed, B13, stil lika), pitanja iz nacrta
[plana platforme](./2026-11-platforma.md) §7 (samo pokazivač E3) i stvari koje tek dolaze (L1-13 donosi vlastitu anketu s
izmjerenim brojkama nakon A1–A4).
