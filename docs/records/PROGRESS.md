# Dnevnik rada (PROGRESS)

Najnoviji unos na vrhu. Svaka sesija: što je napravljeno, što je provjereno, što slijedi. **Povijest, nikad izvor istine** — što vrijedi sad piše u [CLAUDE.md](../../CLAUDE.md).

---

## 2026-10-10 (popodne) — demo na GitHubu, računalo, sloj 0, dizajn puta B, UI čuvar

- Završni pregled grane ekrana (opus) → val popravaka (presuda bez internih brojeva `presuda.v3` + brana u kodu
  [ADR-025](DECISIONS.md#adr-025) 🟨, utrka „Moj hotel" nakon zatvaranja, jedinice, tekstovi) → ponovni pregled → **spojeno i
  pushano**; CI je jednom pao na nestabilnom testu („▲ +6" dvaput) — reproducirano i popravljeno, CI zelen.
- **Računalo** ([ADR-027](DECISIONS.md#adr-027) 🟨): bočni izbornik, dva stupca; **sloj 0 puta B**: 15 pokazatelja (ADR,
  RevPAR, GOPPAR, točka pokrića…), ruta `/kpi`, ekran `/pokazatelji`; oba pregledana, u `main` i na GitHubu. Kalibracija:
  sloj 0 procijenjen 6–10 h, agent ga je napravio za 28 min (+14 min popravci).
- **Put B** (Leon: „kao BitLife, opsežno do bola"): katalog [PROSIRENJA.md](../product/PROSIRENJA.md), arhitektura
  [MODEL_V2.md](../architecture/MODEL_V2.md), [ADR-026](DECISIONS.md#adr-026)/[ADR-028](DECISIONS.md#adr-028) (Leonove odluke:
  ljudi kao u Simsima, burza rada, agencija, tjedni ili mjesečni krug, AI razgovori; pravilo: nacionalnost određuje samo
  činjenice 🟨), istraživanje [JEZICI_GOSTIJU.md](../research/JEZICI_GOSTIJU.md), plan sloja 1 (22 taska) s preflightom (24 nalaza,
  presude u ledgeru). Izvedba sloja 1 započeta: Task 1 i 2 paralelno.
- **UI čuvar** (Leon: „pripazi na frontend i cijeli UI"): pregled svih ekrana 360–1920 px (2 blokera, 12 važnih, 22 sitna),
  pravila `docs/product/UI_PRAVILA.md` (na grani); val popravaka na grani `gradnja/ui-popravci`.
- Leon: dokumentacija je „srce operacija", planovi od 4000+ redaka su loši → plan preuređenja za paralelnu sesiju
  (`docs/plan/2026-10-10-dokumentacija.md`, PAUZIRAN dok ta sesija ne krene).
- AI trošak ove sesije: 0 $ (sve provjere bez ključa).

## 2026-10-10 — Ekrani igrača (DEMO korak 5), mentor govori ljudski, modeli

- Grana `gradnja/ekrani` (lokalno, **nije pushana**), plan [2026-10-10-ekrani.md](../archive/2026-10-10-ekrani.md), ledger u
  `.superpowers/sdd/2026-10-10-ekrani/progress.md` (Rulings 12–23). Svaki task pregledan: 9 (Mjesec završen, + Leonov zahtjev:
  indeksi s kraja mjeseca, Moj hotel se osvježi nakon obračuna), 10 (tijek, množina), 12 (lica), 13a (dorada izgleda, cijeli
  brojevi), 13c (README, BACKLOG).
- Leon: mentor je pisao sirove brojke („4876 na 4269“, „200 bp“, „centi“) → Task 14: činjenice u ljudskim jedinicama, odluke iz
  plana, izabrani odgovor, četiri dijela, čuvar u testovima ([ADR-023](DECISIONS.md#adr-023) 🟨). Task 16: ruta objašnjenja
  mjeseca i kartica „Zašto se indeksi pomaknuli“ iz istih činjenica; Task 17: detalj svakog pokazatelja (`/pokazatelj/<kod>`);
  Task 19: Zaposlenici objašnjeni prema formuli, prvi mjesec „na početku igre“.
- Nađeno u pregledniku: lažno „Bez odgovora: 1“ (dva pravila brojanja) → Task 15; dvostruki React ključevi u tijeku → Task 16.
- Leon: presuda na Sonnetu, lik na Haikuu, mentor na Sonnetu ([ADR-024](DECISIONS.md#adr-024) ✅) → Task 18.
- Provjereno: Rust 310 / web 130 testova, balans 1000 sjemena prolazi, preglednik 390 i 360 px bez vodoravnog pomicanja i bez
  grešaka u konzoli. Jedan prolaz s pravim AI-jem (lipanj → srpanj): lik, presuda i mentor rade; mentor piše ljudski.
- Trošak AI-ja ove sesije (dnevnik događaja, od 05:51): Haiku 9 poziva 7.654/748 tokena ≈ 0,001 $; Sonnet 5 poziva
  16.886/2.451 ≈ 0,058 $; **ukupno ≈ 0,06 $** (dio poziva poslije 05:56 vjerojatno je Leonova igra). Po igri sada ≈ 0,05 $:
  presuda ≈ 0,008 $, mentor ≈ 0,015 $ po mjesecu (veći ulaz — činjenice), lik ≈ 0,0001 $ po odgovoru.
- Otvoreno: završni pregled cijele grane (opus) + val popravaka (među njima: presuda piše „ponuđenom odgovoru 0“); Leonova vizija
  (profesor kreira igru, kod kao u Kahootu, novi likovi, više situacija i mjeseci) u [BACKLOG.md](BACKLOG.md).

## 2026-10-09/10 — AI sloj (DEMO korak 4) i Leonov dizajn

- Leon kroz anketu: slobodan tekst u S1 i S2, brojke iz najbližeg ponuđenog odgovora, do 6 poruka liku, Anthropic ([ADR-018](DECISIONS.md#adr-018) ✅); pretpostavke [ADR-019](DECISIONS.md#adr-019) 🟨. Plan s 11 taskova ([archive/](../archive/2026-10-09-ai-sloj.md), ledger uz njega).
- Izgrađeno kroz subagente (sonnet izvođači i pregledi, opus završni pregled): otisak pravila nad kanonskim `Value` (stare igre i dalje rade — zamrznut hash koraka 3 provjeren na lokalnoj bazi), rubrike i likovi, razgovor, slobodan odgovor, presuda (kvalitetu računa kod), neutralna i zakašnjela presuda, mentor, odgoda u redu poslova, crate `ai` (Claude), rute, mjerilo.
- Završni pregled: „spremno uz popravke" — 4 važna (injekcija preko replika lika → `presuda.v2`, timeout klijenta, izlazni kod mjerila, e2e komentar) i 4 sitna; jedan val popravaka, ponovni pregled čist.
- Provjereno: 262 testa / 0 palih, fmt, clippy, brana jezgre, balans, check:docs, check:api, lint, typecheck — na grani i na spojenom `main`. Živi test s pravim ključem prošao (presuda ~8 s). Spojeno u `main` (`156e6df`), pushano uz Leonov OK, CI zelen (run 38003510045).
- Leon je zalijepio ključ u `.env.example` (datoteka u gitu) — premješten u `.env` prije ikakvog commita; ništa nije otišlo na GitHub.
- Leon: ekrani u sljedećoj sesiji, Tailwind, njegov logo i koncept ekrana ([ADR-020](DECISIONS.md#adr-020), [DIZAJN.md](../product/DIZAJN.md)); nađeno 6 neslaganja koncepta i igre.
- Otvoreno: Leonove ocjene 40 odgovora za mjerilo; likovi i ADR-019 na pregled; odgođeni nalazi u BACKLOG-u `[ai-sloj]`.

## 2026-10-05 — Dnevnik, runde i red poslova (DEMO korak 3)

- Plan s 12 taskova ([archive/](../archive/2026-10-05-dnevnik-runde.md), ledger uz njega) i pretpostavke [ADR-015](DECISIONS.md#adr-015) 🟨. Leon je kroz ankete odlučio gosta, zatvaranje s neodgovorenim, tko zatvara i profile ([ADR-016](DECISIONS.md#adr-016) ✅); tehničke presude voditelja su u [ADR-017](DECISIONS.md#adr-017) 🟨.
- Izgrađeno kroz subagente na grani `gradnja/dnevnik-runde` (izvođač + pregled po tasku; popravni krugovi na taskovima 6, 9, 10, 12). Završni pregled na opusu: „spremno uz popravke" → jedan val popravaka (radnik preživi paniku, pravilo otiska snimka, BACKLOG) → ponovni pregled čist.
- Što radi: cijela igra preko API-ja na pravom Postgresu — gost bez lozinke, igrač + 3 robota, lipanj i srpanj, „Zatvori mjesec" (202), radnik u serveru, ljestvica i feed hotela. Nepromjenjiv dnevnik (okidač u bazi), red poslova s `SKIP LOCKED`, novi crate `game`.
- Provjereno: sve CI provjere lokalno zelene; E2E na Postgresu provjerava stvarne ishode; ručni prolaz curlom na pravom serveru. Spojeno u `main` (`894d023`) i pushano uz Leonov OK; CI run 37250763627: rust i web uspjeh. Ekrana još nema.
- Otvoreno: prije prve promjene content DTO-a popraviti otisak snimka (BACKLOG `[korak 4]`); ishodi neodgovorenih S1–S4 su nacrt; ADR-015 i ADR-017 čekaju Leonov pregled.


## 2026-10-04 — Jezgra igre i alat za balans (DEMO korak 2), rok demoa ~14.10.

- Leon: rok demoa ~14.10., prvo profesorovih 15 minuta ([ADR-012](DECISIONS.md#adr-012)); pitanja za profesore pregledana i spremna za slanje (15 + 13).
- Plan jezgre s 12 taskova + 2 nakon završnog pregleda (sad u [archive/](../archive/2026-10-04-jezgra.md), ledger uz njega). Pretpostavke [ADR-013](DECISIONS.md#adr-013) 🟨; zlatne vrijednosti iz neovisnog modela `scripts/oracle-jezgra.mjs`.
- Izgrađeno kroz subagente (sonnet izvođači i pregledi, opus kalibracija i završni pregled), svaki task pregledan: brane determinizma s mutacijskim probama u CI-ju, `engine::calc`, obračun mjeseca, mehanike S1–S4, centrirani indeksi i ocjena, crate `content`, roboti, crate `balance`, kalibracija.
- Završni pregled našao: lekcija S3 radi naopako, obnova besplatna. Leon (anketa): kolovoz rasprodan, likvidnost bez gornje granice ([ADR-014](DECISIONS.md#adr-014)). Sad S3: kolovoz −9 953 €, lipanj +19 184 €.
- Provjereno: 106 testova, `balance --seeds 1000 --check` „Sve provjere prolaze", clippy, fmt, ovisnosti jezgre, mutacijske probe (0 proboja), check:docs, lint, check:api, typecheck, web test — sve EXIT=0, lokalno i na spojenom mainu. Spojeno u `main` i pushano (Leonov OK).
- Otvoreno za Leona: [BACKLOG](BACKLOG.md) „Otvoreno za Leona" (sezonci 50, B fiksni, A opterećenje, obnova za A/C, S3/S4 odgovori, ADR-013).

## 2026-10-04 — Preslagivanje dokumentacije po Sokratovom sustavu (ADR-011)

- Povod: Leon nije zadovoljan slaganjem dokumentacije („dokumentacije ne idu dobro kroz sesiju") pa dokumenti idu po ulozi u mape `product`, `architecture`, `plan`, `workflow`, `records`, `research`, `archive` ([ADR-011](DECISIONS.md#adr-011)). Premješteno s `git mv` (povijest ostaje), 33 relativne poveznice prepisane skriptom.
- Novo: jedini indeks [docs/README.md](../README.md); zapisi `PROGRESS.md`, [CHANGELOG.md](CHANGELOG.md), [DECISIONS.md](DECISIONS.md) (ADR-001…011), [BACKLOG.md](BACKLOG.md) (46 odgođenih nalaza + pitanja za Leona + rupe u specu); [AGENTI.md](../workflow/AGENTI.md); komanda `/kraj`; `/newsession` usklađen; `.sokratis/profile.json`.
- CLAUDE.md sveden na pravila, trenutno stanje i pokazivače (3 993 znaka, budžet 12 000); `SLJEDECA_SESIJA.md` skraćen na cilj, prvi korak i pitanja; ROADMAP više ne nosi pitanja (žive u BACKLOG-u); arhiva dobila pečate `✅ ISPUNJEN`.
- Brana `scripts/check-docs.mjs` (mrtve poveznice, indeks, jedan aktivni plan, datumi u `product/`, budžet CLAUDE.md, ćirilica, arhiva proglašena aktivnom) + 11 testova (`node:test`), svaka provjera dokazana mutacijom; korak u CI-ju (`web`). Usput nađeno: Sokratov regex `AKTIVN(O|I|A)` nikad ne hvata „AKTIVAN" — ovdje ispravljeno.
- Provjereno lokalno: `check:docs` EXIT=0 (26 .md, 158 poveznica), `test:docs` 11/11, `lint` EXIT=0, ručna mutacija mrtve poveznice → EXIT=1. CI na GitHubu nije viđen (nije pushano).
- Obrisana spojena grana `gradnja/kostur`.

## 2026-10-04 — Predaja: VS Code panel i plan preslagivanja dokumentacije

- Dogovoreno da se radi u Claude Code dodatku za VS Code (panel, ne terminal) — [ADR-010](DECISIONS.md#adr-010).
- Leon zatražio preslagivanje dokumentacije po Sokratovom sustavu; plan i dijagnoza (ista činjenica na 3–6 mjesta, nema dnevnika ni numeriranih odluka) upisani u [SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md) §2b.
- Provjereno: popis 20 `.md` datoteka, samo 2 mrtve poveznice.
- Otvoreno: sam posao preslagivanja; zatim plan jezgre i alata za balans.

## 2026-10-04 — Push i prvi CI zelen

- Uz Leonov izričit OK pushano na GitHub (`leonkreso784-bit/hotelixsim`, privatni) — [ADR-005](DECISIONS.md#adr-005).
- Provjereno: CI run 37167540215 zelen, `rust` 4m51s, `web` 1m44s; gledano preko `gh` (nije na PATH-u Bash alata, putanja u SLJEDECA_SESIJA §5).
- Upozorenje: akcije `checkout@v4` i `setup-node@v4` na Node 20 zastarjele; `ubuntu-latest` prelazi na Ubuntu 26 od 19.10.
- Revizija stanja (4 agenta) pa 7 paralelnih agenata: nađene rupe u specu [DEMO](../product/DEMO.md) (profil C ne kreće s 50, kalibracija bez redovnog održavanja, nema pravila zaokruživanja); upisane kao ulaz za plan jezgre. Ankete samo za proizvodne odluke — [ADR-006](DECISIONS.md#adr-006).
- Dodana komanda `/newsession`.

## 2026-10-04 — Gradnja kostura od kraja do kraja (8 taskova)

- Po planu [kostur](../archive/2026-10-04-kostur.md): Rust workspace (`engine`, `sim-hotel`, `app`, `store`, `api`, `server`) → `openapi.json` → generirani TS klijent → i18n → jedan React ekran.
- Svaki task s pregledom, završni pregled cijele grane i jedan krug završnih popravaka (LF za sve datoteke, `SQLX_OFFLINE`, dozvole CI-a). Odluke koje je kostur donio umjesto Leona: [ADR-008](DECISIONS.md#adr-008); lokalna baza na 5433: [ADR-007](DECISIONS.md#adr-007); brane determinizma kao Task 1 sljedećeg plana: [ADR-009](DECISIONS.md#adr-009).
- Provjereno lokalno: testovi Rust 28, sučelje 10; zdravlje 200, profil D → 404, kriva lozinka baze → 503 bez pada, Vite proxy.
- Odgođeni nalazi pregleda (47 stavki) zapisani u [BACKLOG](BACKLOG.md).
- Otvoreno: ojačati brane determinizma (allowlist ovisnosti, mutacijski probni testovi).

## 2026-10-04 — Ime, arhitektura v2 i dizajn demoa

- Ime odlučeno: HotelixSim — [ADR-002](DECISIONS.md#adr-002). Pokušaj „Xenokrat" vraćen (revert) prije toga, 03.10. navečer.
- Arhitektura nacrt v1 (Rust + TypeScript) i rasprava, zatim v2 s ugrađenim krugom 1 rasprave — [ADR-003](DECISIONS.md#adr-003); sve lokalno, Railway tek za pilot — [ADR-004](DECISIONS.md#adr-004). Vidi [ARHITEKTURA](../architecture/ARHITEKTURA.md).
- Istraživanja: hotelski pokazatelji, postojeće simulacije, Rust alati ([research](../research/HOTELSKI_POKAZATELJI.md)).
- [DEMO](../product/DEMO.md): opseg demoa i dizajn igre; kasnije kategorija hotela (3*/4*/5*), demo svi 4*. Pitanja za profesore napisana ([PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md)).
- Arhitektura se vodi kroz dokument, ne ankete — [ADR-006](DECISIONS.md#adr-006); od 04.10. radi bez pitanja — [ADR-005](DECISIONS.md#adr-005).
- Otvoreno: Leonova provjera domene (pitanja 2–6 u DEMO-u).

## 2026-10-03 — Početak: HotelixSim kao zaseban projekt

- Sesija planiranja u Sokratu: HotelixSim postaje glavni projekt, zaseban repo bez veze s kodom Sokrat Studyja; Sokrat ide u održavanje — [ADR-001](DECISIONS.md#adr-001).
- Prvi commit repoa: [VIZIJA](../product/VIZIJA.md) (dobiveni hotel, fer start, pet osi situacija, AI u četiri uloge, relativna ocjena), [ROADMAP](../plan/ROADMAP.md) (tada PLAN: prototip, prezentacija, pilot, vizija), izvorna ideja u [archive](../archive/HOTEL_SIM_2026-08-22.md), CLAUDE.md.
- Pokušano ime „Xenokrat" (xenia + -krat), isti dan vraćeno.
- Otvoreno: GitHub repo još ne postoji (nema `gh`); backend simulacije; vrijeme „1–7 dana" u simulaciji.
