# ADR-013 — Pretpostavke jezgre: centrirani indeksi, sezonci po planu, redoslijed obračuna, drugi val
**Datum:** 2026-10-04 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-012](ADR-012-ciljani-rok-demoa-14-10-prvo.md),
[ADR-009](ADR-009-ojacavanje-brana-determinizma-je-task-1.md) · **Plan:** [2026-10-04-jezgra.md](../../archive/2026-10-04-jezgra.md) · **Model:** `scripts/oracle-jezgra.mjs`

**Kontekst.** Revizija 04.10. našla je rupe u [DEMO.md](../../product/DEMO.md) (BACKLOG „Rupe u specu"): profil C po §4
ne kreće s 50 (G 70, Z 54, O 70), prosječan hotel bez odluka kroz sezonu gubi R i dobiva F (samo zato što ocjena
klizi, a blagajna raste), nije rečeno kako se računa početni F ni što je „zadano" za sezonce, a s brojkama iz §2.1
kolovoz nije pun pa grupa iz S3 nikoga ne istiskuje. Plan jezgre ne može napisati prvi test dok to nije presuđeno.
**Odluka (pretpostavke, svaka zamjenjiva podacima ili jednom funkcijom):**
1. **Svih pet indeksa mjeri se prema referentnom hotelu iste kategorije u istom mjesecu:**
   `indeks = 50 + (sirova vrijednost − sirova vrijednost referentnog hotela)`, ograničeno 0–100. Sirove vrijednosti su
   formule iz DEMO §4; referentni hotel je profil C koji igra zadani plan bez situacija. Posljedica: C bez odluka
   stoji na **točno 50 u svakom indeksu cijelu sezonu** („ništa ne gori, ništa ne blista"), a napredak mjeri samo ono
   što su odluke promijenile. Širi KALIBRACIJA opciju 1 (centriranje) s G, Z, O na svih pet i na svaki mjesec.
2. **Početni F** = F iz prognoze lipnja (zadani plan, bez situacija) i likvidnosti uz trošak rada bez zapošljavanja.
3. **Sezonci:** katalog ima plan sezonaca po mjesecu; zadani plan ga slijedi (najviše ±10 mjesečno). Radnje
   `HireSeasonal`/`ReleaseSeasonal` postaju jedna: ciljani broj sezonaca za mjesec.
4. **Redoslijed obračuna mjeseca:** odgođeni učinci prošlog mjeseca → osoblje → tržište → ljudi → novac → oprema i
   gosti. Fluktuacija se računa iz novog morala; automatska zamjena odlazaka košta kao zapošljavanje (1 500 €) i
   ulazi u „mješavinu" iskustva.
5. **Cilj marže** je sezonski (VI–IX) **35–45 %**, i nijedan mjesec iznad 50 % (kolovoz je bio 55 %).
6. **Grupna cijena** = referentna cijena ciljnog mjeseca uz popust; grupa se smije ugovoriti za tekući ili kasniji mjesec.
7. Parametri mehanika i **ponuđeni odgovori za S3 i S4** (DEMO ih nema) napisani su kao nacrt u `content/hotel/`
   i čekaju Leonovu provjeru, kao i rubrike S1b i S5–S8.
8. **Drugi val (poslije demoa 14.10.):** splet okolnosti i tragovi (§9), S1b i S5–S8, naplata otvorenih tragova na
   kraju sezone, različit učinak S4 po stanju hotela, tablice za 3* i 5*, JSON Schema kataloga.
9. Pravila cjelobrojnog računa P1–P32 iz DEMO §12 vrijede kako su napisana.
**Posljedice.** Kalibracija (Task 11 plana) mora postići i da kolovoz bude dovoljno pun da grupa iz S3 istiskuje
goste. Ako Leon odbije točku 1, mijenja se jedna funkcija (`centred`) i referentna tablica; ostale točke su podaci.
