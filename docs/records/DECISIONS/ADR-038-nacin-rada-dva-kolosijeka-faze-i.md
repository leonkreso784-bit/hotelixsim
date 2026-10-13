# ADR-038 — Način rada: dva kolosijeka, faze i cigle, uloge agenata
**Datum:** 2026-10-13 · **Status:** ✅ **Leonova odluka 13.10.** (razgovor i anketa u sesiji) · pravila se prenose u
[AGENTI.md](../../workflow/AGENTI.md) i `check:docs` u fazi TEMELJ RADA · **Vezano:**
[ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md), [ADR-037](ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md),
[ROADMAP](../../plan/ROADMAP.md)

**Kontekst.** Nakon Sloja 1 ostaje oko 700 h posla [PROC]: slojevi igre, platforma (internet, razred, profesor),
pilot. Leon: *„ovo sada postaje ozbiljan projekt"*. Traži faze s ciglama kao u Sokrat Studyju, ali puno intenzivniji
rad s agentima, testove i lagano poliranje koda unutar svake cigle. Cilj je verzija za profesore oko sredine siječnja
(poželjno, ne nužno) i studentski pilot u ljetnom semestru.

**Odluka.**
1. **Dva kolosijeka, podijeljena po datotekama.** **IGRA** dobiva srce igre (`crates/game`, jezgra, sadržaj, balans),
   uključujući promjene za razred. **PLATFORMA** dobiva server, bazu, prijavu, nove module `platform/` i ekrane
   razreda. Razlog: tema ne sprječava sudar, a vlasništvo nad datotekama ga sprječava. Kolosijeci se sastaju na
   prekretnicama P1–P4 ([ROADMAP](../../plan/ROADMAP.md)). Prije oba ide faza **TEMELJ RADA**. Ona razdvaja velike
   zajedničke datoteke (`api/src/schema.rs`, mentor) da kolosijeci ne dijele nijednu.
2. **Faza ima ime i uvjet „gotovo kad…"** izražen kroz ono što čovjek može napraviti. Cigla je komad od ~2–8 h koji se
   zasebno testira i spaja. Faze iza ZAVRŠETKA SLOJA 1 su **smjer**: raspored se ponovno slaže kad profesor vidi igru.
3. **Anatomija cigle.** Kartica u planu faze nosi što se radi, koje datoteke smije dirati, dokaz i veličinu. Svaki
   korak radi svjež agent:
   - **voditelj** (glavna sesija) otvara granu, daje karticu, vodi ledger i spaja;
   - **čuvar testova** iz kartice piše testove **prije koda** i dokazuje da padaju iz pravog razloga;
   - **graditelj** piše kod dok testovi ne prođu i polira ono što dira. Testove ne smije mijenjati: ako misli da je
     test kriv, vraća ga voditelju;
   - **provjeravatelj** sam ponovno pokreće sve provjere i čita kod naspram kartice. Za jezgru radi i probe-mutaciju.
     Nalaz ide graditelju, a tek njegov „da" pušta ciglu dalje;
   - **čuvar dokumentacije** upisuje promjene (CHANGELOG, ADR, MODEL_V2, BACKLOG) i pokreće `check:docs`. Ne prevodi
     Rust, pa radi usporedno.
4. **Cigla je gotova** kad su testovi čuvara testova i sve stalne provjere zeleni (fmt, clippy, testovi, balans,
   `check:api`, web), provjeravatelj kaže „da", dokumentacija je upisana i nijedna dirana datoteka nije prešla granicu
   veličine.
5. **Kraj faze:** UI čuvar pregleda ekrane, završni pregled radi najjači model, a Leon igra u pregledniku (jedna adresa
   i put do novoga). Zatim slijede spajanje i pitanje za push.
6. **Ritam (anketa 13.10.).** Cigle unutar faze idu jedna za drugom bez čekanja. Voditelj staje i javlja se Leonu
   **na kraju faze**, kad treba Leonova odluka o igri ili brojkama, ili kad nešto pukne.
7. **Modeli i resursi.** Čuvar testova, graditelj i provjeravatelj rade na Sonnetu, a provjeravatelj na najjačem
   modelu za rizične cigle (jezgra, balans, sigurnost, prijava). Čuvar dokumentacije radi na Haiku ili Sonnetu.
   Najviše **dva agenta istodobno prevode Rust**, zbrojeno preko oba kolosijeka (računalo). Svaki korak dobiva novog
   agenta, a nastavak samo za mali popravak. Izlazi idu u datoteke.
8. **Područja planova** postaju `igra` i `platforma`, svako s jednim aktivnim planom faze. Danas su područja `kod` i
   `dokumentacija`. Dokumentaciju piše čuvar dokumentacije unutar cigle, pa zasebna sesija dokumentacije otpada.
9. **Linije reza** ako tempo zapne, redom: vlastite situacije profesora P3 → P4, razgovori samo tri vrste,
   NATJECANJE ZA LJUDE P3 → P4. **Nikad se ne režu** testovi, provjeravatelj i balans.

**Posljedice.**
- Pravila iz t. 3–8 prelaze u [AGENTI.md](../../workflow/AGENTI.md), a područja `igra`/`platforma` u `check:docs` (pravilo 3,
  [docs/README.md](../../README.md)). Oboje je cigla faze TEMELJ RADA. Do tada vrijedi ovaj ADR.
- Neovisnost testova od graditelja je glavna brana kvalitete. Graditelj ne može sam sebi dokazati da je gotov.
- Više agenata po cigli troši više tokena. Mehanički posao zato ne ide na najjači model, a dugi agenti se ne nastavljaju.
- Pravila rada s Leonom ostaju: push samo uz izričit OK, odluke o igri anketom, „gotovo" uz mjesto gdje se vidi.
