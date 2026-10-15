# HotelixSim — dokumentacija

**Ovo je jedini ulaz.** Sve ostalo je u mapama ispod, složeno **po ulozi dokumenta**, ne po temi
(model preuzet iz Sokrat Studyja — [ADR-011](records/DECISIONS/ADR-011-dokumentacija-po-sokratovom-sustavu-mape-po.md)).

> **Brzi kontekst za sesiju:** [`../CLAUDE.md`](../CLAUDE.md) (učitava se sam: pravila, trenutno stanje, pokazivači).
> **Početak sesije:** [SLJEDECA_SESIJA.md](./SLJEDECA_SESIJA.md) (`/newsession`) · **kraj sesije:** `/kraj`.
> **Pokretanje i provjere prije commita:** korijenski [`README.md`](../README.md) → „Lokalni razvoj".

---

## Kako je ovo složeno

| mapa | uloga | mijenja se |
|---|---|---|
| **`product/`** | **ŠTO** gradimo — vizija, dizajn igre, sadržaj | rijetko, uz odluku |
| **`architecture/`** | **KAKO** je građeno — sustav, granice, ugovori | kad kod pomakne granicu |
| **`plan/`** | **ŠTO SADA** — `ROADMAP.md` + najviše **jedan** aktivni plan **po području** | stalno |
| **`workflow/`** | **KAKO RADIMO** — agenti, tok rada | povremeno |
| **`records/`** | **POVIJEST** — dnevnik, isporuke, odluke, backlog | stalno |
| **`research/`** | istraživanja — podloga za brojke i izbore | uz istraživanje |
| **`archive/`** | **ISPUNJENO** — referenca, **nikad izvor istine** | isti dan kad dokument ispuni svrhu |

### Gdje što ide — jedna činjenica, jedno mjesto

| vrsta znanja | JEDINO mjesto | ostali |
|---|---|---|
| **što sustav radi** | **kod + testovi** | dokument to samo *opisuje*, nikad ne *definira* |
| **što vrijedi sad** | `../CLAUDE.md` + `product/` + aktivni plan | nikad dnevnik, nikad `archive/` |
| **zašto je tako** | `records/DECISIONS/` (jedan ADR = jedna datoteka) | ostali linkaju ADR; `records/DECISIONS.md` je samo popis |
| **što se kad dogodilo** | `records/PROGRESS.md` (sesija) · `records/CHANGELOG.md` (isporuka) | `CLAUDE.md` ih ne ponavlja |
| **što nije riješeno** | `records/BACKLOG.md` (i pitanja za Leona) | plan uzima stavke po oznaci, ne prepisuje ih |
| **koje su se pogreške dogodile i što smo naučili** | `records/BUGS.md` | uzrok i pouka; popravak ostaje u BACKLOG-u ili ADR-u |
| **kako pokrenuti** | korijenski `README.md` | ostali linkaju |
| **što radi sljedeća sesija** | `SLJEDECA_SESIJA.md` | prepisuje se na svakom `/kraj` |

**Duplikat se briše, ne sinkronizira.**

### Pravila (čuva ih `corepack pnpm check:docs`, i u CI-ju)

1. **Svaka poveznica vodi negdje.** Poveznica na datoteku, sliku, skriptu ili mapu koje nema pada.
2. **Svaki `.md` pod `docs/` je naveden u svom indeksu.** Dokument koji nigdje nije naveden je „duh". Dokumenti u
   `docs/<uloga>/` navedeni su ovdje; dijelovi u mapi `X/` navedeni su u `X.md`.
3. **Jedan aktivni plan po području.** Svaki plan u zaglavlju ima `**Područje:** igra | platforma | kvaliteta | dokumentacija`
   (bez tog retka znači `igra`; `kod` je stari naziv za `igra`, isti brojač). U `plan/` smije biti najviše jedan aktivan
   plan za svako područje, uz `ROADMAP.md`. Plan koji čeka nosi `**Status:** ⏸️ PAUZIRAN`. Kolosijeci smiju raditi
   usporedno samo na različitim područjima.
4. **`product/` nije dnevnik** — najviše 3 ISO datuma (npr. 2026-10-10) po datoteci; kronologija ide u `records/`.
5. **`CLAUDE.md` ima budžet znakova** (vidi tablicu), jer se učitava svaku sesiju.
6. **Nema ćirilice** u tekstu izvan koda (slovo-dvojnik zna ući kopiranjem).
7. **Arhiva ne smije biti živa.** Plan u `plan/` ne smije proglasiti plan iz `archive/` aktivnim.
8. **Podindeks:** `X.md` uz mapu `X/` mora postojati i biti naveden ovdje. Više od jedne razine mapa (`X/Y/…`) je zabranjeno.
9. **Svaki dokument je unutar granice veličine** (tablica niže). Dva dokumenta s prijelaznom iznimkom (Sloj 1: plan
   `2026-10-10-v2-sloj1` zajedno s mapom dijelova i `MODEL_V2.md`) izuzeta su od tih granica dok sloj traje.
10. **ADR-ovi su jedan po datoteci.** Imena su `ADR-0NN-naziv.md`, brojevi se ne ponavljaju, a popis
    `records/DECISIONS.md` mora biti jednak onome što alat generira. Ako popis zastari, brana pada.
11. **Ne povezuj se na ADR preko popisa** (`DECISIONS.md#adr-…`): poveznica ide ravno na datoteku ADR-a.
12. **Tvrda granica: nijedan `.md` u repozitoriju nema više od 1200 redaka** — ni izvan `docs/`, ni s prijelaznom
    iznimkom. Izuzima se samo generirani ili vanjski dokument, izrijekom upisan u `IZUZECI_TVRDE_GRANICE`
    (`scripts/docs-granice.mjs`, danas prazno); gitignorirano (`node_modules`, `target`, `.superpowers`) ne ulazi u branu.

### Granice veličine

Granica se mjeri u **redcima** (osim `CLAUDE.md`: znakovi). Izvor granica za dokumente je `scripts/docs-granice.mjs`, a budžet `CLAUDE.md`
je `CLAUDE_BUDGET` u `scripts/check-docs.mjs`; brana provodi oba.
Pregled stvarnih veličina uz granice: `node scripts/check-docs.mjs --popis`.

| vrsta | gdje | granica | zašto |
|---|---|---|---|
| predaja | `docs/SLJEDECA_SESIJA.md` | 60 | već pravilo `/kraj`, sad i brana |
| ADR | `docs/records/DECISIONS/*.md` | 120 | jedna odluka; najveći danas 60 (ADR-015) |
| indeks | `docs/README.md` i svaki `X.md` uz mapu `X/` (izvan arhive) | 200 | indeks vodi, ne objašnjava |
| živi dokument, plan, task | sve ostalo pod `docs/` osim `archive/` | 300 | čita se cijeli u jednom potezu; task od 300 tjera plan da navodi datoteke i potpise umjesto da lijepi cijeli kod |
| arhiva | `docs/archive/**` | 1000 | zamrznuto; task veći od toga dijeli se po smislu na dijelove (npr. ekrani Task 4 i 4b) |
| **tvrda granica** | **svaki `.md` u repozitoriju** | **1200** | Leonovo pravilo (13.10.; s 1000 na 1200 isti dan); vrijedi i za prijelazne iznimke |
| `CLAUDE.md` | korijen | 12 000 znakova | postojeći budžet |

### Indeks + mapa istog imena

Dokument `X.md` koji preraste granicu postaje kratak **indeks** (svrha i tablica „§ → datoteka"), a dijelovi idu u
**mapu `X/`** pokraj njega. Ovaj `README.md` ostaje jedini ulaz: navodi `X.md`, a `X.md` navodi svoje dijelove. Brojevi
paragrafa (§) ostaju u naslovima dijelova, pa navodi poput „ARHITEKTURA §7.2" i stare poveznice i dalje vrijede.
Danas je tako složeno: ARHITEKTURA, DEMO, pet arhivskih planova, plan Sloja 1 i ADR-ovi (`records/DECISIONS/`).

### Planovi, područja i ADR-ovi

- **Plan** do 300 redaka smije biti jedna datoteka. Veći je **pregled** `plan/<datum>-<ime>.md` + mapa istog imena sa
  po jednim taskom u datoteci; razdvaja ga alat (`corepack pnpm docs:plan <plan.md>`, vidi [AGENTI](./workflow/AGENTI.md)).
- **Područje** je `igra`, `platforma`, `kvaliteta` ili `dokumentacija` (bez retka = `igra`; `kod` = stari naziv za `igra`);
  za svako smije biti jedan aktivan plan.
- **Odluke** žive u `records/DECISIONS/`, jedna datoteka po ADR-u. `records/DECISIONS.md` je **popis koji se generira**
  (`corepack pnpm docs:adr`): ne uređuj ga ručno.
- **Zapisnici** (`PROGRESS`, `CHANGELOG`, `BACKLOG`) imaju granicu živog dokumenta; kad je prijeđu, stariji unosi idu u
  mapu istog imena po mjesecima (npr. `PROGRESS/2026-10.md`), a indeks ostaje na vrhu (kako, vidi `/kraj`).

---

## Korijen `docs/`

| Dokument | Svrha |
|---|---|
| [SLJEDECA_SESIJA.md](./SLJEDECA_SESIJA.md) | Predaja: cilj sljedeće sesije, prvi korak, otvorena pitanja. Prepisuje se na kraju svake sesije |

## `product/` — što gradimo

| Dokument | Svrha |
|---|---|
| [VIZIJA.md](./product/VIZIJA.md) | Zašto i za koga: dobiveni hotel, fer start, pet osi situacija, AI u četiri uloge, relativna ocjena |
| [DEMO.md](./product/DEMO.md) | **Spec demoa**: stvarni sloj hotela, pet indeksa, profili A/B/C, mehanike, situacije S1–S8, roboti, provjere (§11), pravila zaokruživanja (§12, nacrt) |
| [DIZAJN.md](./product/DIZAJN.md) | Leonov logo i koncept ekrana v2 (boje, tri ekrana, likovi, kako su riješena neslaganja s igrom); slike u [dizajn/](./product/dizajn/) |
| [SITUACIJE_S1b_S5-S8.md](./product/SITUACIJE_S1b_S5-S8.md) | Nacrt ponuđenih odgovora i rubrika za S1b i S5–S8 — čeka Leonovu provjeru |
| [PITANJA_ZA_PROFESORE.md](./product/PITANJA_ZA_PROFESORE.md) | Pitanja za profesore FMTU-a — čeka Leonov pregled prije slanja |
| [PROSIRENJA.md](./product/PROSIRENJA.md) | **Put B**: katalog dodataka nakon demoa (odjeli, zaposlenici, sobe, recenzije, pokazatelji, računalo) — što student vidi, što uči, koje brojke mijenja, sloj |
| [IDEJE.md](./product/IDEJE.md) | Ideje za nakon profesorova pregleda (vrste soba, agencije, događaji, novine, platforma) — skupljaju se, ne grade se dok Leon ne odluči redoslijed |
| [UI_PRAVILA.md](./product/UI_PRAVILA.md) | Obvezni popis za svaki web task: tokeni, brojke s imenom i jedinicom, tekst bez roda, raspored, pristupačnost, obrasci za odjele, račun dobiti i birač kruga |

## `architecture/` — kako je građeno

| Dokument | Svrha |
|---|---|
| [ARHITEKTURA.md](./architecture/ARHITEKTURA.md) | Arhitektura v2: Rust (jezgra + backend) + TypeScript (sučelje), granice crateova, determinizam, dnevnik događaja, AI iza porta, redoslijed gradnje (§20) |
| [RASPRAVA_ARHITEKTURE.md](./architecture/RASPRAVA_ARHITEKTURE.md) | Zapis rasprave o nacrtu: primjedbe, odgovori, otvoreni upiti |
| [VODITELJ_I_TRZISTE.md](./architecture/VODITELJ_I_TRZISTE.md) | Voditelj odjela koji upravlja odjelom i odgovara za njega, i uvid u tržište (zainteresirani, smješteni, odbijeni gosti); nacrt za raspravu, Leonove ideje 13.10. |
| [BANKA_LJUDI.md](./architecture/BANKA_LJUDI.md) | Banka ljudi: format i pravila za tisuće radnika i gostiju (sadržaj u repou), ciljni brojevi, deterministički izbor po sjemenu, punjenje u serijama s dva agenta; nacrt za raspravu, ADR-042 t. 5 |
| [BODOVANJE_VLASTITOG_ODGOVORA.md](./architecture/BODOVANJE_VLASTITOG_ODGOVORA.md) | Vlastiti odgovor vrijedi više od klika na najbolji (Leon 15.10.): opcije s brojkama, mjesto prije L1-13, anketa; nacrt za raspravu |
| [PLATFORMA.md](./architecture/PLATFORMA.md) | **Platforma**: razred = jedna igra s N hotela, računi i uloge, profesorov pregled, generirani hoteli, produkcija (Railway) i trošak, GDPR; nacrt za raspravu, odluke u ADR-037 (dijelovi u `PLATFORMA/`) |
| [REVIZIJA_TEMELJA.md](./architecture/REVIZIJA_TEMELJA.md) | **Revizija temelja 14.10.**: što je temelj, što je nadogradnja, gdje puca (s dokazima), cijena popravka prema gradnji ispočetka, preporuka u fazama A–C; čeka Leonovu odluku |
| [MODEL_V2.md](./architecture/MODEL_V2.md) | **Put B**: kako hotel od brojeva postaje skup stvari (odjeli, zaposlenici, sobe, recenzije) — entiteti, slojevi, determinizam, stare igre |

## `plan/` — što sada

| Dokument | Svrha |
|---|---|
| [ROADMAP.md](./plan/ROADMAP.md) | 🟩 **živ** — faze projekta (TEMELJI → DEMO → PREZENTACIJA → VERZIJA ZA PROFESORE → PILOT → VIZIJA), put do pilota u dva kolosijeka (IGRA, PLATFORMA) s prekretnicama P1–P4; cigle po kolosijeku u mapi `ROADMAP/` |
| [2026-10-14-popravak-temelja.md](./plan/2026-10-14-popravak-temelja.md) | 🟩 **aktivan** — popravak temelja po [reviziji](./architecture/REVIZIJA_TEMELJA.md): faza A (brane koje mjere pravu stvar, kratke grane; 7 kartica), B (v2 mehanike po odjelu, jedna kalibracija, L1-13…L1-15), C (igra za semestar) |
| [2026-10-10-v2-sloj1.md](./plan/2026-10-10-v2-sloj1.md) | ⏸️ **pauziran** (14.10., ADR-043; ostatak u planu popravka) — put B, sloj 1: temelj v2 (v1 zamrznut, tjedni ili mjesečni krug), odjeli po USALI-ju, račun dobiti i budžet (22 taska) |
| [2026-10-10-dokumentacija.md](./plan/2026-10-10-dokumentacija.md) | ⏸️ **pauziran** (čeka spajanje Sloja 1; Taskovi 11–13) — preuređenje dokumentacije: granice veličine, planovi i ADR-ovi po datotekama, ARHITEKTURA/DEMO/MODEL_V2 po temama |

## `workflow/` — kako radimo

| Dokument | Svrha |
|---|---|
| [AGENTI.md](./workflow/AGENTI.md) | Tok rada s agentima: plan → subagenti na grani → pregledi → `/kraj`; ledger, model po poslu, paralelni agenti, CI preko `gh` |

## `records/` — povijest

| Dokument | Svrha |
|---|---|
| [PROGRESS.md](./records/PROGRESS.md) | Dnevnik sesija, najnoviji na vrhu |
| [CHANGELOG.md](./records/CHANGELOG.md) | Isporuke (ono što radi) |
| [DECISIONS.md](./records/DECISIONS.md) | **Generirani** popis odluka (ADR-001…) s datumom i statusom, najnovija na vrhu; sam ADR je u `DECISIONS/` (jedna datoteka po odluci); uključuje one koje čekaju Leonov pregled |
| [BACKLOG.md](./records/BACKLOG.md) | Sve neriješeno: odgođeni nalazi pregleda po oznakama, rupe u specu, pitanja za Leona |

## `research/` — istraživanja

| Dokument | Svrha |
|---|---|
| [HOTELSKI_POKAZATELJI.md](./research/HOTELSKI_POKAZATELJI.md) | Hotelski pokazatelji (popunjenost, ADR, RevPAR, GOPPAR…), tipične vrijednosti, uzročne veze |
| [JEZICI_GOSTIJU.md](./research/JEZICI_GOSTIJU.md) | Jezici gostiju u Opatiji po mjesecima (TZG Opatija, Novi list) — preporučena tablica za jezike osoblja (procjena 🟨) |
| [POSTOJECE_SIMULACIJE.md](./research/POSTOJECE_SIMULACIJE.md) | Postojeće hotelske simulacije i što naučiti od njih |
| [RUST_ALATI_2026-10.md](./research/RUST_ALATI_2026-10.md) | Stanje Rust alata (listopad 2026.) za izbor ovisnosti |
| [KALIBRACIJA_2026-10-04.md](./research/KALIBRACIJA_2026-10-04.md) | Kalibracija početnih indeksa i marže — opcije s brojkama za rupe u DEMO specu (prijedlog, nije odlučeno) |
| [BRANE_DETERMINIZMA_2026-10-04.md](./research/BRANE_DETERMINIZMA_2026-10-04.md) | Pokus izvedivosti brana determinizma — ulaz za Task 1 plana jezgre |

## `archive/` — ispunjeno

| Dokument | Svrha |
|---|---|
| [2026-10-04-kostur.md](./archive/2026-10-04-kostur.md) | ✅ Plan „kostur od kraja do kraja" (8 taskova) |
| [2026-10-04-kostur-ledger.md](./archive/2026-10-04-kostur-ledger.md) | ✅ Radni ledger kostura: odluke i nalazi po taskovima |
| [2026-10-04-jezgra.md](./archive/2026-10-04-jezgra.md) | ✅ Plan „jezgra i alat za balans" (14 taskova, zlatne vrijednosti iz `scripts/oracle-jezgra.mjs`) |
| [2026-10-04-jezgra-ledger.md](./archive/2026-10-04-jezgra-ledger.md) | ✅ Radni ledger jezgre: odluke, nalazi pregleda, odgođene sitnice |
| [2026-10-05-dnevnik-runde.md](./archive/2026-10-05-dnevnik-runde.md) | ✅ Plan „dnevnik, runde i red poslova" (12 taskova, DEMO korak 3) |
| [2026-10-05-dnevnik-runde-ledger.md](./archive/2026-10-05-dnevnik-runde-ledger.md) | ✅ Radni ledger koraka 3: odluke, nalazi pregleda, sažetak završnog pregleda |
| [2026-10-09-ai-sloj.md](./archive/2026-10-09-ai-sloj.md) | ✅ Plan „AI sloj" (11 taskova, DEMO korak 4) |
| [2026-10-09-ai-sloj-ledger.md](./archive/2026-10-09-ai-sloj-ledger.md) | ✅ Radni ledger koraka 4: preflight, presude voditelja, odgođeni nalazi, završni pregled |
| [2026-10-10-ekrani.md](./archive/2026-10-10-ekrani.md) | ✅ Plan „ekrani igrača" (DEMO korak 5; taskovi 1–20, spojen u `main`) |
| [2026-10-10-ekrani-ledger.md](./archive/2026-10-10-ekrani-ledger.md) | ✅ Radni ledger koraka 5: presude voditelja 1–26, odgođeni nalazi, završni pregled i val popravaka |
| [2026-10-13-temelj-rada.md](./archive/2026-10-13-temelj-rada.md) | ✅ Plan „Faza 0 TEMELJ RADA" (K0-2 pravila rada i područja, K0-3 brana `check:kod`, K0-7 arhitektura platforme; K0-1 CI dokazan 14.10.), taskovi u mapi istog imena |
| [2026-10-13-temelj-rada-ledger.md](./archive/2026-10-13-temelj-rada-ledger.md) | ✅ Radni ledger Faze 0: presude, završni pregled i val popravaka, dokaz K0-1 |
| [2026-10-15-popravak-temelja-faza-b-ledger.md](./archive/2026-10-15-popravak-temelja-faza-b-ledger.md) | Radni ledger popravka temelja: faza A i B do B1a (rulingi, ankete, nalazi pregleda, odgođeni minori) |
| [HOTEL_SIM_2026-08-22.md](./archive/HOTEL_SIM_2026-08-22.md) | Izvorna ideja iz kolovoza (iz Sokrata) — zamijenjena s `product/VIZIJA.md` |
