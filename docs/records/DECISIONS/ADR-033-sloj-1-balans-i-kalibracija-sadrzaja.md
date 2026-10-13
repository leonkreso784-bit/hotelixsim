# ADR-033 — Sloj 1: balans i kalibracija sadržaja v2
**Datum:** 2026-10-11 · **Status:** 🟨 **ČEKA LEONOV PREGLED** (brojke [PROC] su procjene za hrvatski hotel 4\* na
moru) · **Vezano:** [ADR-029](ADR-029-sloj-1-pretpostavke-plana.md), [ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md),
[ADR-013](ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md), [MODEL_V2.md](../../architecture/MODEL_V2.md)
§4.2, §8 (sloj 1, „Balans")

**Kontekst.** Task 16 je dodao balans za v2, a Task 17 je kalibrirao `content/hotel/v2/` tako da sve provjere prođu na
1000 sjemena. Prvi krug kalibracije ostavio je udio rada prenizak (≈ 24 %) i „plaću 115" dominantnom, jer je zbroj
ljudi bio zaključan na v1 profil, a plaća Smještaja bila plaća v1 kategorije; presuda voditelja (R1a, R3) to je
riješila u popravku 1.

**Odluka (pretpostavke).**
1. **Balans v2** (Task 16): `balance --model v2 --round week|month` (zadano `week`); ispisuje izvještaj i sve
   provjere koje padaju (uz `--check` izlaz 1). v1 probe preneseni su na `PeriodPlan`, ali v1 „održavanje
   Intensive/Deferred" mijenja samo Smještaj (odjelne probe mijenjaju sve odjele). „Zatvori sve što se smije" otpušta
   sezonce odjela koliko korak dopušta i zatvara odjel tek kad nema sezonaca. Marže odjela, udio rada i GOP/EBITDA
   su iz referentnog profila (zadani plan, bez situacija), „ostali prihodi C" iz profila C (pojas 25–35 % prihoda
   soba). Tjedno ≈ mjesečno: medijani obje vrste krugova računaju se uvijek, pojas 300 stotinki.
2. **Kapacitet odjela** (korištenja po zaposlenom na dan, ×10): restoran 6 (bilo 5), lobby bar 20 (12), rooftop 16
   (10), wellness 7 (6), bar uz bazen ostaje 12 [PROC]. Prije su barovi stalno radili na 240 (puni kapacitet), što je
   kažnjavalo A i C (koji imaju lobby bar) i nije realno za šank.
3. **Restoran:** prosječni račun 35 € (bilo 32 €; večera s pićem u hotelu 4\* na moru), nabavna vrijednost 30 %
   prihoda (bilo 32 %; miješano hrana i piće). **Wellness:** nabavna vrijednost 15 % (12 %), materijal i rublje 5 € po
   tretmanu (3 €), plaća 2.500 € (2.300 €; kvalificirani terapeuti s provizijom) [PROC].
4. **Profil A** („ugledni, ali umorni"): veći stalni dio neraspoređenih jer ima više sadržaja (ukupni stalni troškovi
   A = 170.000 € kao v1 profil A; energija 38.000 € nakon popravka 1); stanje prostora operativnih odjela +5 bodova
   (restoran 83, lobby 81, wellness 85, rooftop 87): ugled drži uređen prostor, umorni su ljudi.
5. **Profil B** („profitabilni, ali zapušteni"): 2 stalna manje u Smještaju (11), više ljudi u Upravi i Prodaji (jake
   financije i prodaja), stalni dio tih odjela smanjen za njihove plaće (ljudi zamjenjuju vanjske usluge, GOP
   neutralno). Zbroj ljudi ostaje kao v1 profil B (28 + 28).
6. **Tjedni koeficijenti** (`periods.json`, `week`): moral 11 %, zadovoljstvo 14 %, ocjena 10 % (bilo 8 / 15 / 7),
   kalibrirani balansom tako da tjedna i mjesečna igra istih odluka završe blizu (MODEL_V2 §4.2). **Tjedni korak
   sezonaca ±3** (popravak 1; prvi krug je imao ±2, prije ±5): studentu u tjednoj igri najviše 3 sezonca više ili
   manje po tjednu.
7. **Popravak 1 (presuda R1a, R3):** plaća Smještaja v2 čita `departments.json` `rooms.wage_cents` (2.500 € [PROC]:
   bruto ≈ 1.750 € + doprinosi + smještaj i hrana sezonaca; u degeneriranom katalogu = plaća v1 kategorije, pa
   ekvivalencija ostaje); početni rad u likvidnosti = Σ plaća odjela (`indices::start_labor_cents`);
   `Catalog::validate`: sezonci = v1 profil, stalnih ≥ v1, višak samo u neraspoređenim odjelima. Sadržaj: Uprava 7 /
   Prodaja 3 / Održavanje 6 stalnih (B 7/2/4), plaće 3.500 / 3.300 / 2.900 €, stalni dio smanjen točno za te plaće.
   Rezultat: udio rada 31 %, GOP 35,6 %.
8. **CI** vrti balans s `--seeds 1000 --check` za v1, v2 tjedno i v2 mjesečno. Margine su tijesne (izvještaj Taska
   17, radne bilješke sesije).

**Otvoreno:** stalni dio Održavanja profila A nema zapisano obrazloženje ([BACKLOG](../BACKLOG.md) `[sloj1]`); norma
osoblja 4\* je u [ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 9.

**Cijena ako krivo:** brojke su podaci u `content/hotel/v2/` (`departments.json`, `profiles.json`, `periods.json`);
svaka promjena traži ponovni balans na 1000 sjemena za obje vrste kruga.
