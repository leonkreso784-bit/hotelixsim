# ADR-063 — Pilot: jedna adresa, lozinka, stropovi AI-ja, tri igre po gostu
**Datum:** 2026-10-19 · **Status:** ✅ **Leonove odluke 19.10.** (t. 1–4, anketa sesije 12; t. 5–6 iz ADR-062 t. 9), 🟨 t. 7–19 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), [ADR-060](ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12–14,
[ADR-062](ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9 i 18, [kartica pilota](../../plan/2026-10-14-popravak-temelja.md#task-pilot-priprema-pilota-na-railwayu) (task-16, dio 10)

**Izvori.** Kartica task-16 (10 dijelova), ledger `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` („SESIJA 12"), izvještaji
`pilot-{1..7}-report.md`, pregledi `pilot-2-review.md` i `pilot-3-review.md`, prijedlog `pilot-hosting-prijedlog.md` (isti direktorij).
Cilj pilota: 1 profesor i 20–40 studenata igraju igru na internetu nekoliko tjedana, na jednoj adresi, bez prijave, uz zaštićen AI trošak.
Ovaj ADR je dio 10 kartice. Deploy i dalje samo uz Leonov izričit OK (ADR-060 t. 12).

## A — Leonove odluke 19.10. (anketa sesije 12, 4/4 preporučene)
1. **Kolačić gosta 120 dana s obnovom** (klizni rok): rok teče od zadnjeg otvaranja igre. Time je 🟨 ADR-062 t. 18 (dio o kolačiću) potvrđena.
2. **Besplatna adresa `*.up.railway.app`**, bez vlastite domene.
3. **Bez ograničenja po IP adresi** (cijeli razred na fakultetskom Wi-Fiju dijeli istu adresu). Pravi čuvari su lozinka i strop.
4. **Gradnja pilota odmah, usporedo s L1-14** (nijedan dio ne dira jezgru).

Iz ADR-062 t. 9 (ponovljeno da je pilot na jednom mjestu):
5. **Railway Hobby** (Amsterdam), **jedna lozinka za cijelu stranicu** (HTTP Basic Auth), **40 $/mj u Anthropic konzoli** (zaseban radni prostor).
6. **Stropovi AI troška u aplikaciji 5 $/dan i 30 $/mj** i **najviše 3 igre po gostu**.

## B — kako je izvedeno (spojeno u `model-v2/sloj1`, na GitHubu 9b6f047; 🟨 izbori voditelja i izvođača)
7. **Kolačić** (dio 1, fd09174): `COOKIE_MAX_AGE_SECS` = 10 368 000 s, jedina konstanta (`crates/api/src/auth.rs`). `GET /v1/me/hotel` poznatom
   gostu ponovo šalje isti token (samo uz 200); `POST /v1/demo` postojećem gostu ne daje novi kolačić. `Secure` samo u produkcijskom načinu.
8. **Vrata poslužitelja** (dio 2, c7ee9f8): Basic Auth provjerava samo lozinku (ime bilo koje, realm „HotelixSim"); `/v1/health` je bez lozinke i
   više ne odaje broj događaja (`SELECT 1`). Rust poslužuje `apps/web/dist` (`tower-http`, samo `fs`): `index.html` i SPA `no-cache`,
   `/assets/*` `immutable` godinu dana, nepostojeća datoteka s nastavkom 404. Tipizirane postavke (`Settings`) na jednom mjestu:
   `HOTELIXSIM_PRODUKCIJA` (1/true; 0/false/prazno = lokalno; drugo = start odbijen), `HOTELIXSIM_LOZINKA`, `HOTELIXSIM_WEB`, uz `DATABASE_URL`,
   `HOTELIXSIM_ADDR`, `HOTELIXSIM_CONTENT`. **Start se odbija** u produkciji bez lozinke, i na adresi koja nije loopback bez lozinke neovisno o
   zastavici (zaboravljena varijabla ne otvara stranicu svima). Uredno gašenje: SIGTERM uz Ctrl-C, radnik dobiva signal odmah, posao u radu ima
   8 s, pa se vraća u red. Zatvara BACKLOG „Typed config", „SIGTERM", „`COUNT(*)` u zdravlju".
9. **Strop AI-ja** (dio 3, 5c71a5a): brojač vidi **svaki** 2xx odgovor Anthropica (i odrezan, odbijen, neparsabilan, uz predmemoriju) i upisuje
   trošak u nano-$ u tablicu `ai_spend` (migracija 0005; plan platforme F1-2 prima 0006). Dan i mjesec po UTC-u. Iznad stropa `CappedAi` vraća
   `AiError::Disabled`, pa igra radi na rezervama (neutralna presuda, kratak odgovor lika, mentor iz brojki); student ne vidi posebnu poruku.
   Varijable `HOTELIXSIM_AI_STROP_DAN_USD` i `_MJESEC_USD`; **produkcija ili javna adresa uz AI ključ traži oba stropa**.
10. **Cijene** su tablica `app::spend::PRICES` sa službene stranice (19.10.2026.): Sonnet 5.5 2 / 2,50 / 0,10 / 10 $, Haiku 5.5 0,10 / 0,125 / 0,01 /
    0,50 $ po milijunu tokena (ulaz / upis u predmemoriju / čitanje / izlaz). **Nepoznat model = najskuplja cijena po komponentama (Fable)**;
    produkcija odbija start za model izvan tablice. Novi model = novi redak tablice.
11. **Istek vremena ili prekid poziva** broji se kao procjena `max_tokens` × izlazna cijena (ulaz 0). U dijelu 9 usporediti `SUM(ai_spend)` s konzolom.
12. **Greška knjige troška = fail-closed:** ako se potrošnja ne može pročitati, AI se gasi (`Disabled`) i piše `WARN`; neuspio upis samo se zapiše
    u log (odgovor je već plaćen). Upozorenje o dosegnutom stropu jednom po danu, odnosno mjesecu. Bez stropova (lokalno) ništa se ne broji.
13. **Tri igre po gostu** (dio 4, d793335): broji se `games.created_by` prije upisa; četvrta igra je 409 `game_limit` s porukom u webu.
    `HOTELIXSIM_IGARA_PO_GOSTU` (cijeli broj ≥ 1; 0 ili tekst = start odbijen); zadano 3 u produkciji **i** na javnoj adresi, lokalno bez granice.
    Brisanje kolačića daje novog gosta, a dva istodobna zahtjeva mogu prekoračiti za 1 (prihvaćeno, t. 3).
14. **Isporuka** (dijelovi 5 i 6): Dockerfile u tri faze (Rust 1.98.1, Node 24, Debian bookworm-slim), migracije ugrađene u binarnu, `CMD server
    migrate && exec server serve`. CI posao `docker` gradi sliku bez objave i uspoređuje `server openapi` s `openapi.json` (prva gradnja na GitHubu
    zelena, 39314933953). `.github/workflows/deploy-pilot.yml`: samo ručno (`workflow_dispatch`), okolina „pilot", Railway CLI 5.70.0 `railway up --ci`,
    čeka `/v1/health`; tajna `RAILWAY_TOKEN`, varijable `RAILWAY_SERVICE` i `PILOT_URL`. Railway gradi sliku iz Dockerfilea (ne ghcr).
15. **Kopije** (dio 7, 6c3acfd): `scripts/pilot-kopija.sh` (kopija / proba / usporedi), **tjedni `pg_dump`** na Leonovo računalo; proba vraćanja u
    zasebnu bazu prošla (igre 83, događaji 3661 isti). Za kopiju treba **javna (TCP proxy) adresa** baze, ne unutarnja; `pg_dump` mora biti ≥ verzija
    Railwayeva Postgresa (17+ traži klijent 17).
16. **Railway Hobby nema kopija volumena** (podrška na forumu; službena dokumentacija šuti), pa je tjedni `pg_dump` jedina kopija. Provjeriti u
    sučelju (Volume → Backups) pri postavi.
17. **`PORT` s Railwaya kod ne čita:** postavlja se `PORT=8080` uz `HOTELIXSIM_ADDR=0.0.0.0:8080`.
18. **`tls-rustls` za `sqlx` ne treba** dok baza ide unutarnjom mrežom Railwaya; provjeriti pri postavi.
19. **Leon 19.10. o banci ljudi** (tekstom): vanjski ljudi su **zajednički za sve hotele**, malo liječnika je dosta. Tumačenje voditelja javljeno
    Leonu i sesiji banke 12b.

## C — obavezno u dijelu 9 (postava i proba), prije dijeljenja adrese
- `RAILWAY_DEPLOYMENT_DRAINING_SECONDS=30` (zadano 0–3 s, a radniku treba 8 s + 5 s) i proba gašenja usred posla; deploy samo kad nema kruga u tijeku.
- `PORT=8080`; `curl -I http://…` mora preusmjeriti na https (inače Basic Auth ide otvoreno).
- **Duga nasumična lozinka** (≥ 16 znakova ili 4 riječi), jer nema ograničenja pokušaja.
- **GitHub okolina „pilot" s Leonom kao obaveznim odobravateljem** prije prvog deploya.
- **Slika se ne dijeli dok proba ne pokaže 401 bez lozinke** na pravoj adresi.
- Nova lozinka baze iz Railwaya (lokalna nikad); upozorenje potrošnje Railwaya; vanjski ping na `/v1/health`; proba na Safariju i Chromeu na
  mobitelu, privatni prozor, krug s razgovorom, presudom i mentorom, te **namjerno nizak strop → rezerva**.

## D — otvoreno za Leona
- **Dio 8:** tekst obavijesti o privatnosti (nadimci, ne prava imena; odgovori idu AI-ju, Anthropic, SAD) i **kontakt**; voditelj daje nacrt.
- **Railway:** upozorenje potrošnje 7 $ i tvrdi strop 15 $ — potvrditi.
- **Tko pokreće tjedni `pg_dump`** (Leon ručno ili podsjetnik).
- `max_tokens` presude: riješeno kroz L1-14 (L14-1: 8000 + iskočni prozor, radi se u L1-15).

## Što se mijenja
- ADR-004 („sve lokalno") prestaje vrijediti za pilot tek kad Leon da OK za deploy; kod i dalje ne ovisi o Railwayu (sve iz varijabli okoline).
- ADR-062 t. 18: kolačić potvrđen (t. 1), adresa potvrđena (t. 2).
- **Za trajanje pilota jezgra se zamrzava** (svaka promjena `CORE_VERSION` daje 409 postojećim igrama).
- Odgođeni nalazi pregleda dijelova 2 i 3 su u [BACKLOG](../BACKLOG.md) („[pilot]").
