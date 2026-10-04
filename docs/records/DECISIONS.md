# Arhitektonske i proizvodne odluke (ADR)

Svaka značajna odluka: kontekst → odluka → posljedice. Najnovija na vrhu. **Ovo je POVIJEST — što vrijedi sad piše u
`CLAUDE.md` i u `docs/product`, `docs/architecture`.**

---

## ADR-011 — Dokumentacija po Sokratovom sustavu: mape po ulozi, jedan indeks, brana, komande
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, „kasnije da" na prijedlog) · **Vezano:** [ADR-010](#adr-010),
[ADR-001](#adr-001) · **Predaja:** [SLJEDECA_SESIJA.md](../SLJEDECA_SESIJA.md) §2b

**Kontekst.** Popis 04.10. (20 .md): ista činjenica živi na 3–6 mjesta (stanje CI-ja, push, baza 5433, pravila rada),
nema indeksa ni numeriranih odluka, a `SLJEDECA_SESIJA` se prepisuje pa gubi povijest. Leon: *„ne sviđa mi se kako imamo
dokumentaciju posloženu, moramo imat isti pristup kao i u Sokrat Studyju pošto će ovo biti veliki razvoj, mislim da
dokumentacije ne idu dobro kroz sesiju."*
**Odluka.** Sokratov raspored 1:1: mape po ulozi dokumenta (`product`, `architecture`, `plan`, `workflow`, `records`,
`research`, `archive`); jedini indeks `docs/README.md`; jedna činjenica jedno mjesto; najviše jedan aktivni plan
(`ROADMAP.md` se ne broji); brana `scripts/check-docs.mjs` (mrtve poveznice, .md izvan indeksa, >1 aktivni plan,
>3 ISO datuma u `product/`, budžet `CLAUDE.md`) s vlastitim testom i korakom u CI-ju; komande `/newsession` i nova
`/kraj`. Poboljšanje nad Sokratom: predaja `docs/SLJEDECA_SESIJA.md` ostaje u repou, ne u memoriji, pa ne zastarijeva.
**Posljedice.** Datoteke premještene s `git mv` (povijest ostaje), poveznice ispravljene. Povijest odluka je ovdje,
dnevnik sesija u `PROGRESS.md`, odgođeno u [BACKLOG.md](BACKLOG.md). Provedba brane i komande `/kraj` je posao na
početku sljedeće sesije, prije plana jezgre ([ROADMAP](../plan/ROADMAP.md)).

---

## ADR-010 — Rad u Claude Code dodatku za VS Code (panel), ne u terminalu
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-011](#adr-011)

**Kontekst.** Leonu je VS Code okruženje prirodnije (Explorer, preglednik uz kod, slike povlačenjem).
**Odluka.** Leon: *„nećemo mi više radit u terminalu… ostat ćemo u VS Code okruženju."* Upute za prečace i slike daju
se za panel dodatka (otvaranje: iskrica / `Ctrl + Esc`; slika: `Ctrl + V` ili povlačenje u polje za poruku), ne za terminal.
**Posljedice.** Za stvari koje treba vidjeti, zaslon se može snimiti i izravno, ali vidi se samo ono što je otvoreno.
Ne mijenja ni kod ni arhitekturu.

---

## ADR-009 — Ojačavanje brana determinizma je Task 1 plana „jezgra i alat za balans", prije ijedne linije jezgre
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude, nakon završnog pregleda kostura; Leon ne mora potvrđivati) ·
**Vezano:** [ADR-003](#adr-003), [ADR-008](#adr-008) · **Izvori:** [BRANE_DETERMINIZMA](../research/BRANE_DETERMINIZMA_2026-10-04.md),
[ledger kostura](../archive/2026-10-04-kostur-ledger.md) (Final #3)

**Kontekst.** Završni pregled kostura: brane determinizma slabije su od ARHITEKTURA §6.2/§7.2 (`indexmap`, `RandomState`,
`as` na float/u16, `#[allow]` zaobilazi `deny`, `env::var`). Jezgra je još prazna, pa ništa ne puca.
**Odluka.** Nalaz ne blokira spajanje kostura, ali je OBAVEZAN prvi task sljedećeg plana: allowlist ovisnosti,
`RandomState` u `clippy.toml`, `forbid` + `as_conversions`/cast lintovi, zabrana `f32`/`f64`, probe mutacija kao skripta
u CI-ju. Pokus izvedivosti je izmjeren (npr. `forbid` radi samo pod `cargo clippy`, pa CI mora vrtjeti clippy).
**Posljedice.** Cijena ako je krivo: kod jezgre napisan prije jačih brana mogao bi proći s nedeterminizmom. Zato prvo
brane, pa jezgra. Plan se piše na temelju istraživanja iznad.

---

## ADR-008 — Odluke iz gradnje kostura koje je Claude donio umjesto Leona
**Datum:** 2026-10-04 · **Status:** 🟨 ČEKA LEONOV PREGLED · **Vezano:** [ADR-005](#adr-005), [ADR-007](#adr-007),
[ADR-009](#adr-009) · **Izvor:** [SLJEDECA_SESIJA.md](../SLJEDECA_SESIJA.md) §4

**Kontekst.** Po pravilu „radi bez pitanja" ([ADR-005](#adr-005)) Claude je tijekom gradnje kostura sam odlučivao sitnije
stvari. Sve su u gitu i mogu se vratiti. (Točka 2 je zaseban [ADR-007](#adr-007), točka 8 je [ADR-009](#adr-009).)
**Odluka (pretpostavke, s cijenom ako su krive):**
1. Gradnja na grani `gradnja/kostur` u istom direktoriju, ne u worktreeu. Cijena: nikakva.
2. Brana ovisnosti mjeri i build-ovisnosti, sve targete i značajke. Cijena: sporija brana.
3. `cargo fmt` nad kodom iz plana (plan je pisan preširoko). Cijena: jedan commit formatiranja.
4. `scripts/pnpm.mjs` jer `pnpm` nije na PATH-u (radi lokalno i u CI-ju). Cijena: zamijeniti ako se uvede samostalni pnpm.
5. Ekran pokazuje grešku indeksa; ključevi i interpolacija u i18n su provjereni (popravak nakon pregleda).
6. README i `.gitattributes` popravljeni za svježi Windows stroj (LF svugdje, `SQLX_OFFLINE`).
7. Sitni nalazi pregleda (~40) zapisani kao ulaz za sljedeće planove, ne popravljeni odmah ([BACKLOG](BACKLOG.md)).
**Posljedice.** Leon pregledava; ono što odbije, vraća se iz gita. Do tada vrijede kao radna pretpostavka.

---

## ADR-007 — Lokalna baza: zaseban PostgreSQL 16 klaster na priključku 5433
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude kao pretpostavka; Leon pregledava s [ADR-008](#adr-008)) ·
**Vezano:** [ADR-004](#adr-004), [ADR-008](#adr-008)

**Kontekst.** Sistemski PostgreSQL na 5432 već postoji, a Leon ne zna njegovu lozinku. Treba lokalna baza za razvoj i demo.
**Odluka.** Zaseban klaster PostgreSQL 16 u `C:\Users\leonk\AppData\Local\hotelixsim\pgdata`, priključak **5433**.
Sistemski na 5432 se ne dira. Umjesto resetiranja njegove lozinke.
**Posljedice.** Ne pali se sam nakon restarta: `bash scripts/baza.sh start`. Cijena ako je krivo: obrisati `pgdata`.
Pokretanje svega opisano je u `README.md` („Lokalni razvoj").

---

## ADR-006 — Arhitektura kroz dokument i raspravu, ne kroz ankete; ankete samo za jednostavne proizvodne odluke
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](#adr-003), [ADR-005](#adr-005) ·
**Izvor:** [RASPRAVA_ARHITEKTURE](../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Na anketu o obliku sustava Leon je odgovorio ljutito (*„ovo je ozbiljno, ovo nije igraonica"*), a jedna od
ponuđenih opcija bila je lažna, ubačena radi kontrasta. Isti dan je odbio da se stack bira prema onome što on zna:
*„ne razmišljaj o tome što ja znam, razmišljaj široko… Rust, drugi jezici, budi otvoren i kreativan"*.
**Odluka.** Arhitektura se radi u [ARHITEKTURA.md](../architecture/ARHITEKTURA.md): za svaku odluku razlog, razmotrene
alternative i cijena. Leon čita, raspravlja, potvrđuje (*„sada ćemo raspravljati o arhitekturi, to još nije to"*).
Kandidati se biraju po zahtjevima sustava, ne po Leonovom iskustvu; nikad lažna opcija. Ankete (AskUserQuestion) ostaju
za jednostavne proizvodne odluke.
**Posljedice.** Dokument je polazište rasprave, nije gotov dok Leon ne kaže. Cilj je razina seniora („arhitektura 10/10").

---

## ADR-005 — Radi bez pitanja: pretpostavke u dokument; push i deploy samo uz Leonov izričit OK
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-006](#adr-006), [ADR-008](#adr-008)

**Kontekst.** Čekanje potvrde svakog koraka usporava Leona; pravilo „nijedan korak ne počinje dok Leon ne potvrdi
prethodni" nadjačano je za tok rada.
**Odluka.** Leon usred ugradnje arhitekture v2: *„automatski radi, nemoj me pitati"*. Claude ide korak po korak bez anketa
i bez „smijem li nastaviti", a odluke koje bi bile upit donosi sam i zapisuje kao **pretpostavku s razlogom**. Ne vrijedi
za push i deploy: za njih treba izričit OK, a odobrenje jedne radnje ne proteže se na sljedeću. Lokalni commit je u redu.
**Posljedice.** Stvari koje zna samo Leon (npr. koje besplatne AI ključeve ima) ostaju otvorene, a gradi se tako da ne
blokiraju (AI iza porta s lažnim pružateljem). Nagomilane pretpostavke vidi [ADR-008](#adr-008).

---

## ADR-004 — Sve se gradi i pokazuje lokalno; Railway tek za pilot
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](#adr-003), [ADR-007](#adr-007) ·
**Izvor:** [ARHITEKTURA](../architecture/ARHITEKTURA.md) §14

**Kontekst.** Demo se pokazuje profesoru uskoro; hosting nije potreban da bi se dokazala igra.
**Odluka.** Leon: *„za sada radimo sve lokalno, Railway je za kraj"*. Razvoj i demo idu lokalno; Railway (jedan servis +
Postgres) dolazi u PILOTU.
**Posljedice.** Ništa u kodu ne smije ovisiti o Railwayu: postavke dolaze iz varijabli okoline, a isporuka je zaseban
završni korak. Deploy i dalje samo uz izričit OK ([ADR-005](#adr-005)). Vidi [ROADMAP](../plan/ROADMAP.md).

---

## ADR-003 — Arhitektura: Rust (jezgra + backend) + TypeScript (sučelje), v2
**Datum:** 2026-10-04 · **Status:** 🟨 NACRT v2, čeka Leonovu potvrdu · **Vezano:** [ADR-006](#adr-006),
[ADR-004](#adr-004), [ADR-009](#adr-009) · **Dokument:** [ARHITEKTURA.md](../architecture/ARHITEKTURA.md),
[RASPRAVA_ARHITEKTURE.md](../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Nacrt v1 pisan je po zahtjevima sustava (determinizam, ponovljivost, pravednost profila), ne po Leonovu
znanju ([ADR-006](#adr-006)). Krug 1 rasprave ugrađen je u v2: ispravka presude, puni snimak pravila, pravednost profila
i AI-ja, red poslova, što tko vidi, slobodan potez, obrazloženje za Rust.
**Odluka.** Jezgra simulacije i backend u **Rustu**, sučelje u **TypeScriptu**. Domena i jezici dogovoreni su 04.10.,
ostalo je prijedlog.
**Posljedice.** **Status je izrijekom nepotvrđen:** `ARHITEKTURA.md` (zaglavlje) kaže „prijedlog dok Leon ne potvrdi", dok
`ROADMAP.md` navodi da ju je Leon pregledao; do izričite Leonove potvrde vrijedi stroža formulacija (nacrt). Kostur
(Rust workspace, API, jedan ekran) već je izgrađen na toj osnovi, pa je promjena smjera skupa. Brane determinizma: [ADR-009](#adr-009).

---

## ADR-002 — Ime „HotelixSim", nikad skraćeno na „Hotelix"
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon izabrao sam) · **Vezano:** [ADR-001](#adr-001)

**Kontekst.** Prijedlog „Xenokrat" bio je odabran 03.10. pa vraćen (revert) isti dan. Leonovi kriteriji: kratko (najviše
dvije riječi), odmah vidljivo da je hotelski simulator ili igra, i da ne postoji. Nakon više krugova odbijenih prijedloga
Leon je sam izabrao HotelixSim.
**Odluka.** Ime je **HotelixSim**. Provjereno: `hotelixsim.com` slobodan 04.10., na webu nema ničega s tim imenom.
„Hotelix" (bez Sim) postoji u istoj branši (AI softver za upravljanje hotelima i studio za hotelske stranice), pa se ime
**nikad ne skraćuje** na „Hotelix".
**Posljedice.** Prije javnog izlaska provjeriti žig (EUIPO/DZIV). Folder je `Documents\hotelixsim`.

---

## ADR-001 — HotelixSim je zaseban projekt, odvojen od Sokrat Studyja
**Datum:** 2026-10-03 · **Status:** ✅ ODLUČENO (Leon, sesija planiranja) · **Vezano:** Sokratov ADR-040 (vanjski repo) ·
**Vizija:** [VIZIJA.md](../product/VIZIJA.md) · **Plan:** [ROADMAP.md](../plan/ROADMAP.md)

**Kontekst.** Leon (03.10.): *„mijenjamo cijeli smjer razvoja"*. Profesor s FMTU-a (Opatija, Hospitality Management)
zainteresiran je za simulaciju vođenja hotela, ali prvo želi demo. Sokrat je na produkciji i dovršava se u užem opsegu.
**Odluka.** HotelixSim je zaseban proizvod u zasebnom repozitoriju (`Documents\hotelixsim`, privatni GitHub
`leonkreso784-bit/hotelixsim`). Ne nosi Sokratov kod ni njegova pravila; smije posuditi znanje i obrasce. Od 2026-10-03
je glavni projekt: Leon radi paralelno, ali više vremena daje ovome.
**Posljedice.** Ništa iz simulacije ne ide u Sokratov repo i obrnuto. Izvorna ideja je u
[archive](../archive/HOTEL_SIM_2026-08-22.md). Ocjenjuje se napredak u odnosu na početak, ne zarada.
