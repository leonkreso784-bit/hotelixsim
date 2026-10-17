# HotelixSim — roadmap

**Status:** 🟩 živ (faze projekta i put do pilota; nije plan taskova) · **Otvoren:** 2026-10-03
Što gradimo: [VIZIJA.md](../product/VIZIJA.md) · kako: [ARHITEKTURA.md](../architecture/ARHITEKTURA.md) i
[MODEL_V2.md](../architecture/MODEL_V2.md) · način rada: [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)

> Faza ima ime, ne slovo. Svaka faza ima uvjet „gotovo kad…" izražen kroz ono što **čovjek može napraviti**,
> nikad kroz „test je zelen". Čvrsto se planira samo blisko (TEMELJ RADA, ZAVRŠETAK SLOJA 1, lokalni dio SPREMNO ZA
> INTERNET). Sve iza toga je **smjer**: raspored se ponovno slaže kad profesor vidi igru. Sati su [PROC].

## Faze projekta

| faza | gotovo kad… |
|---|---|
| **TEMELJI** ✅ | …arhitektura i dizajn igre su zapisani. Arhitektura v2 i domena DEMO-a još čekaju Leonovu potvrdu (ADR-003, BACKLOG) |
| **DEMO** ✅ | …igra radi u pregledniku: hotel, situacije, razgovor s likom, AI ocjena, mentor, napredak uz tri robota (koraci 1–5 i sloj 0, [CHANGELOG](../records/CHANGELOG.md), planovi u [archive/](../archive/2026-10-10-ekrani.md)) |
| **PREZENTACIJA** | …profesor je vidio igru i dobio [pitanja](../product/PITANJA_ZA_PROFESORE.md) |
| **VERZIJA ZA PROFESORE** | …profesori na internetu probaju igru s razredom (cilj sredina siječnja 2027., poželjno, ne nužno) — prekretnica P3 |
| **PILOT** | …studenti ljetnog semestra igraju jedni protiv drugih, na satu ili kroz semestar, a profesor prati razred — prekretnica P4 |
| **VIZIJA** | zajedničko tržište gostiju, više hotela i tržišta ([VIZIJA.md](../product/VIZIJA.md) §8) |

## Put do pilota: dva kolosijeka

Kolosijeci su podijeljeni **po datotekama** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
**IGRA** drži srce igre (jezgra, `crates/game`, sadržaj, balans), a **PLATFORMA** server, bazu, prijavu i ekrane
razreda. Pravila razreda: [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md).
Cigle po fazama (zaliha; kartica se razrađuje u planu faze kad faza krene):

| kolosijek | cigle |
|---|---|
| kvaliteta i TEMELJ RADA | [ROADMAP/kvaliteta.md](./ROADMAP/kvaliteta.md) |
| IGRA | [ROADMAP/igra.md](./ROADMAP/igra.md) |
| PLATFORMA | [ROADMAP/platforma.md](./ROADMAP/platforma.md) |

**Prije oba kolosijeka: TEMELJ RADA** (~26 h). Pravila rada s agentima, popravak CI-ja (Ubuntu 26 od 19.10.), brana
veličine koda, prijenos platformskog nacrta iz ledgera u `docs/` i Leonova sjednica ADR-ova 🟨. Razdvajanje
`api/src/schema.rs` i mentora ide tek kad se grane obnove i situacija spoje u `sloj1` (obje te datoteke mijenjaju). **Gotovo kad:** oba kolosijeka mogu krenuti bez ijedne zajedničke velike datoteke.

| okvirno | IGRA | PLATFORMA | prekretnica: što čovjek može |
|---|---|---|---|
| listopad | **ZAVRŠETAK SLOJA 1** (~50 h): situacije u igri, obnova, kraj sezone, spajanje tri grane, zajednički balans, profesorov račun dobiti | **SPREMNO ZA INTERNET** (~48 h, lokalno, 0 €): postavke, gašenje, zaštita, server servira web, kopije, granica AI troška, brza igra za 30 igrača | **P1** · Leon odigra cijelu sezonu Sloja 1 sa situacijama, obnovom i krajem sezone |
| studeni | **RAZRED U JEZGRI** (~45 h): N studentskih hotela, profesor otvara krug, obračun po hotelu | Platforma kreće u studenom usporedo s fazom C — Leon 16.10., [nacrt plana](./2026-11-platforma.md). **RAZRED** (~116 h): računi studenata (korisničko ime i lozinka), kod igre, čekaonica, lista kolega, raspored otvaranja i rokovi, ekrani studenta i profesora | **P2** · profesor na internetu stvori igru, studenti uđu kodom i igraju jedni protiv drugih (izlazak samo uz Leonov OK, ~5–10 €/mj) |
| prosinac – siječanj | **LJUDI** (~64 h) · **RAZGOVORI** (~35 h) · **NATJECANJE ZA LJUDE** (~45 h) | **PROFESOR** (~83 h do P3): pregled razreda, detalj studenta, pregled AI ocjena, profesor dodaje vlastite situacije | **P3** · profesori probaju verziju s ljudima i razgovorima i vode razred |
| veljača – ožujak | **JEZICI I AGENCIJA** · **VRSTE SOBA I CIJENE** · **RECENZIJE** (~85 h) | **PILOT** (~95 h + ostatak PROFESORA): GDPR, opterećenje 30 studenata, domena, ispravak AI ocjene, izvoz | **P4** · studentski pilot |
| poslije pilota | sobe (tlocrt), Hoteli u okolici, pošta s agencijama, događaji, dobavljači, novine; igra na engleskom (AI i sadržaj) | generator hotela (puni banku hotela, 17.10.), AAI@EduHr (čim fakultet odobri), sučelje na engleskom | — |

**Do P3 je ~525 h** (IGRA ~240, PLATFORMA ~250, kvaliteta ~40), više od okvirnih 425 h iz razgovora: liste su
detaljnije. Zato su linije reza dio plana, ne iznimka.

**Linije reza** ako tempo zapne, redom: vlastite situacije profesora P3 → P4 · razgovori samo tri vrste ·
NATJECANJE ZA LJUDE P3 → P4. **Nikad se ne režu** testovi, provjeravatelj i balans.

## Pilot → produkcija i banka hotela (Leon 17.10.2026., [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md))

**Leon:** *„ovo što već imamo je pilot, a ovo što gradimo je već produkcijska platforma; spremamo se za profesorov DA jer znam
da će biti da."* Standard „gotovo" zato nije „radi na Leonovu računalu" nego **40 studenata igra tri mjeseca bez Leona uz
računalo**. Igra bogata sadržajem bez prijave i hostinga nije produkcija.

**🟨 Redoslijed (prijedlog voditelja, Leon ga nije izričito potvrdio):**
1. **L1-13 → L1-13b → L1-14 → L1-15** (kraj popravka temelja, faza B).
2. **PLATFORMA do najmanje igrive razredne igre:** prijava i pristup (računi, profesorovi vlastiti podaci za prijavu), hosting
   s **izmjerenim** troškom (uz AI trošak po igri, AI-M), oporavak od pada na pravoj bazi (kopije i proba vraćanja), profesorovo
   sučelje u najmanjem obliku (otvori krug, vidi hotele, vidi razgovore), GDPR i brisanje. Cigle: [platforma.md](./ROADMAP/platforma.md)
   i [nacrt plana platforme](./2026-11-platforma.md); uz to RAZRED U JEZGRI ([igra.md](./ROADMAP/igra.md)).
3. **Tek onda** bogatstvo igre: sučelje soba, voditelji i obuka, smještaj radnika, **BANKA HOTELA**, tržište, potrošnja hotela
   ([igra.md](./ROADMAP/igra.md) „Nove cigle 17.10.").

**BANKA HOTELA** (Leon: *„profesor će moći dodijeliti hotele studentima koje želi"*; rješava A10 „da studenti ne dobiju uvijek
isti hotel sa 120 soba"): popis gotovih, pregledanih hotela raznih veličina. 🟨 Mjesto: nakon L1-13 (i L1-14/15), prije kraja
faze C; zamjenjuje GENERATOR HOTELA kao prvi korak (generator kasnije samo puni banku). Tri komada: **sadržaj** (BH-S: prvo
istraživanje pravih hotela i plaća, A10 i A12, pa 10–15 hotela 4* u serijama koje Leon pregleda i pečati, kao banka ljudi),
**jezgra** (BH-J: ključ hotela kao tekst, referenca napretka i balans po hotelu; pravednost 80 prema 160 soba kroz indekse po
sobi ili udjelu, [PLATFORMA §4.3](../architecture/PLATFORMA/04-generirani-hoteli.md)), **platforma** (BH-P: dodjela po
studentu ili nasumce, tek s razredom). Sati: 🟨 [PROC], procjenu daje kartica.

## Gdje smo

Aktivan plan: [Popravak temelja](./2026-10-14-popravak-temelja.md) (14.10., [ADR-043](../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md):
faze A → B → C, samo popravak dok nije gotov; preuzima ostatak ZAVRŠETKA SLOJA 1). [Sloj 1](./2026-10-10-v2-sloj1.md) je pauziran.
Nakon popravka: TEMELJ RADA i SPREMNO ZA INTERNET, pa RAZRED (🟨 17.10.: platforma do najmanje igrive razredne igre prije novih mehanika igre, odjeljak iznad).
Otvorena pitanja za Leona žive na jednom mjestu: [BACKLOG.md](../records/BACKLOG.md) → „Otvoreno za Leona".
