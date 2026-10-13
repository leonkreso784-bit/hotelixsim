# ADR-029 — Sloj 1: pretpostavke plana
**Datum:** 2026-10-11 · **Status:** 🟨 **ČEKA LEONOV PREGLED** (P1–P4 su Leonove odluke ✅ iz ADR-028) ·
**Vezano:** [ADR-028](ADR-028-leonove-odluke-za-put-b-ljudi.md), [plan Sloja 1](../../plan/2026-10-10-v2-sloj1.md),
[MODEL_V2.md](../../architecture/MODEL_V2.md) §4, §6, §8, [ADR-031](ADR-031-sloj-1-tjedni-krug-u-jezgri.md),
[ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md), [ADR-033](ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md),
[ADR-034](ADR-034-zadani-model-igre-je-v2-tjedna.md)

**Kontekst.** Plan [2026-10-10-v2-sloj1](../../plan/2026-10-10-v2-sloj1.md) gradi model v2 (odjeli po USALI-ju, račun
dobiti s budžetom, tjedni i mjesečni krugovi) iznad zamrznutog v1. P1–P4 su Leonove odluke ✅ iz
[ADR-028](ADR-028-leonove-odluke-za-put-b-ljudi.md) (odgovori 3, 4, 5 i 9) i ovdje se samo navode; P5–P18 su
pretpostavke 🟨 s cijenom ako Leon odluči drukčije. Dokaz kraj-do-kraja: `crates/api/tests/e2e_v2.rs`
(`weekly_v2_season_from_start_to_end`: 17 krugova kroz API, `current` null, `/pl?scope=season` 122 dana, 17 unosa u
`/kpi`, t17 u `/feed`).

**Odluka (pretpostavke).**

| # | pretpostavka 🟨 | ako krivo, mijenja se |
|---|---|---|
| P5 | Novi model se bira izričito: `POST /v1/demo` bez tijela stvara v1; v2 traži `{ "model": "v2" }`. `main` ostaje na v1 dok Leon ne kaže. **Web je od 12.10. zadano na v2** ([ADR-034](ADR-034-zadani-model-igre-je-v2-tjedna.md)); API bez tijela i dalje stvara v1 | Task 7, Task 18 |
| P6 | v2 igra uvijek traje cijelu sezonu (4 mjeseca ili 17 tjedana); v1 demo lipanj–srpanj | Task 6 (`GameSettings`), Task 7 |
| P7 | Postavke odjela su trajne postavke plana, ne jednokratni nalozi (nalozi u sloju 2) | Task 11, 15, 21 |
| P8 | Budžet = vlastiti profil uz zadani plan i bez situacija, za isti kalendar (statički budžet), ne referentni hotel C | Task 14 |
| P9 | Održavanje se knjiži u neraspoređeni odjel „Održavanje" (POM) | Task 12, 19 |
| P10 | Naknada za upravljanje i I&T se ne modeliraju; „Fiksne naknade" = osiguranje, najam, porez na imovinu | Task 10, 13 |
| P11 | v2 igra druge verzije jezgre v2 vraća 409 „Igra je stvorena starijom verzijom igre. Započni novu igru." već pri čitanju; v1 se nikad ne odbija | Task 6, 7 |
| P12 | Trajne povišice iz situacija idu u trošak rada odjela Smještaj | Task 12 |
| P13 | Situacija mjeseca u tjednoj igri u krugu koji sadrži 1. dan mjeseca (lipanj t01, srpanj t05, kolovoz t09, rujan t14) | Task 3, 6 |
| P14 | Upit računa dobiti je `?round=`, ne `?period=` | Task 15, 20 (`usePl`), 22 — samo preimenovanje |
| P15 | Vrsta kruga i kalendar krugova u postavkama igre (`GameCreated.rounds`), ne u snimku pravila; snimak nosi koeficijente obiju vrsta | Task 4, 6, 7 |
| P16 | Likvidnost i kamata se preračunavaju na dane kalendarskog mjeseca u kojem krug počinje (`D₀`), ne „/30" | Task 3; ekvivalencijski testovi 3 i 6 |
| P17 | Profesorov račun dobiti se ne gradi u sloju 1 ([BACKLOG](../BACKLOG.md) `[sloj1]`) | novi task nakon Taska 15 (~4–6 h) |
| P18 | Odjel nema vlastiti „Zašto"; detalj pokazuje razlaganje brojkama | Task 12, 15, 19 |

Napomena uz P16 (presuda voditelja, Task 3 i Task 12): odlasci se ne računaju s `D₀` nego
`prorate(div_round(staff·turnover_bp, 10_000), p)` — mjesečni odlasci razmjerno krugu; za cijeli mjesec je to
identitet, pa ekvivalencija s v1 ostaje točna. `D₀` ostaje za kamatu i likvidnost. Pravilo i raspodjela odlazaka po
odjelu: [ADR-031](ADR-031-sloj-1-tjedni-krug-u-jezgri.md) t. 1 i [ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 8.

**Svjesna odstupanja od MODEL_V2:**
- `?round=` umjesto `?period=` (§6.5, P14).
- Vrsta kruga u postavkama igre, ne u snimku (§6.2 t. 1, §11 t. 6, P15).
- `D₀` umjesto „/30" (§4.2/§4.4, P16; odlasci razmjerno, vidi napomenu).
- Profesorov račun dobiti odgođen (§8 sloj 1, P17).
- Bez „Zašto" po odjelu (§8, P18).
- Tjedni mentor samo „tri uzroka" bez AI-ja, osim u krugu koji zatvara mjesec (presuda voditelja u preflightu, P3).

**Kalibracija sadržaja** (Task 17), također 🟨: tjedni korak sezonaca ±3, tjedni koeficijenti iz `periods.json`; plaća
Smještaja iz `departments.json` (`rooms.wage_cents` 2.500 €), pa je početni rad u likvidnosti zbroj plaća odjela;
kapaciteti odjela (restoran 6, lobby bar 20, rooftop 16, wellness 7, bar uz bazen 12). Pune brojke i obrazloženja:
[ADR-033](ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md).

**Dopuna 11.10. — robot „Brižni" (Leon, anketa; završni popravci ZP-2 i ZP-6), 🟨 formula:**
1. Potreba kruga je ⌈sobe · referentna popunjenost / 1,8⌉ zaposlenih koji služe goste, manje stalnih **izvan
   neraspoređenih odjela**. Uprava, Prodaja, Održavanje i Energija ne služe goste i nisu u opterećenju; zatvoreni
   odjel je izvan opterećenja.
2. U **tjednoj** igri Brižni planira u oba smjera, jer smije mijenjati sezonce samo ±3 po krugu: zapošljava prije
   vrhunca i otpušta prije pada. Cilj_t = min(max(potreba_t, max_k(potreba_{t+k} − 3k)), min_k(potreba_{t+k} + 3k)).
   Buduće potrebe uzima samo iz kataloga, bez slučajnosti.
3. **Mjesečna** igra i v1 ostaju bez gledanja unaprijed, jer bi to promijenilo v1 (ekvivalencija).
4. Balans 1000 sjemena, medijan Brižnog tjedno / mjesečno: A 9,08 / 10,36, B 14,05 / 11,79, C 13,59 / 14,71; sve
   provjere prolaze. Nova provjera u balansu: Brižni drži barem 80 % sezonskog plana hotela.

**Cijena ako krivo:** po retku tablice (stupac „ako krivo"), uz procjene iz tablice pretpostavki u planu.
