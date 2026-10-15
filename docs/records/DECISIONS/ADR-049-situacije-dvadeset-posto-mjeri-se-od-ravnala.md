# ADR-049 — „Situacije ~20 %" mjeri se od ravnala; redoslijed odgovora mjesečno dobiva zapisani dug
**Datum:** 2026-10-15 · **Status:** ✅ **Leonova odluka 15.10.** (anketa B3, nakon brojki B3a; ishod B3b i B1a dopisan isti dan) ·
**Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 15, [ADR-047](ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B3a, B3b, L1-13)

**Pitanje:** „situacije nose ~20 % napretka" — 20 % od čega? Leon je odabrao prvu ponuđenu opciju.

**Odluka:**
1. **Težina situacija = |Δ| / (|Δ| + ravnalo)**, gdje je Δ = najbolji odgovor − šutnja na kraju sezone (medijan po sjemenu), a
   **ravnalo** = razlika Brižni − Štediša na zadanom planu **bez** situacija. Ravnalo se mjeri jednom i upisuje u kod s datumom
   (`plan_ruler` u `crates/balance/src/v2_answers.rs`): 15.10.2026., commit 76271d1, 1000 sjemena, stotinke boda, A/B/C:
   **tjedno 845 / 678 / 1033, mjesečno 812 / 679 / 1090**. Ne prati igru ni robote; mijenja se samo Leonovom odlukom u L1-13.
2. **Pojas brane „B3 težina" je 5–50 %** (`WEIGHT_BAND_BP` = 500–5000) do L1-13; L1-13 ga sužava na **15–25 %** (ADR-042 t. 15).
   Brana gleda predznak: težina s Δ ≤ 0 ne prolazi kao pojas, nego pada kao „B3 odgovori".
3. **Mjesečno „srednji odgovor ≤ prepušteno" popravlja se u L1-13.** Do tada B3b taj jedan korak drži kao **zapisani dug**
   (`ORDER_DEBT = [(Month, 1)]`) sa stropom −25 stotinki (`ORDER_DEBT_FLOOR`; izmjereno −14/−11/−10); dug smije samo padati.
   **Ishod:** dug je otplaćen već u B1a (ADR-050): uz bonus +1 mjesečni korak je pozitivan (+0,63/+0,92/+0,84), `ORDER_DEBT` je prazan.

**Razlog:** težina izmjerena od ravnala ne ovisi o tome koliko je robot dobar nego o tome koliko se plan i odgovori razlikuju;
ravnalo je stabilno, a Leon ga zna objasniti profesorima. Dug sa stropom (a ne „točno 0") poštuje njegovu odluku za izmjerene
negativne brojke, a ipak ne dopušta da se pogorša.

**Cijena ako krivo:** druga definicija = drugo ravnalo, jedna konstanta i ponovno mjerenje; pojas i dug su po jedan redak.
