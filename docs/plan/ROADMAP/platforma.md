# ROADMAP — kolosijek PLATFORMA

**Što je ovo:** ZALIHA CIGLI po fazama za kolosijek platforme (server, `store`, `app/platform`, `api/platform`, web razreda i profesora,
hosting, GDPR). Faze iza prve su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle se razrađuje u planu faze
kad faza krene. Sati su [PROC]. Podjela je po **datotekama** ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
sve što dira srce igre (N hotela, vrata kruga, obračun po hotelu: F1-7, F1-9, F1-10, F1-18) je u [igra.md](./igra.md), faza RAZRED U JEZGRI.
Razred = jedna igra s N hotela, studenti igraju jedni protiv drugih ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)).
Susjedi: [kvaliteta.md](./kvaliteta.md), [ROADMAP.md](../ROADMAP.md). Arhitektura platforme (nacrt za raspravu, odluke u ADR-037): [PLATFORMA.md](../../architecture/PLATFORMA.md).

**Prekretnice (stupac P):** P1 kraj listopada · P2 kraj studenog (profesor na internetu stvori igru, studenti uđu kodom) · P3 sredina siječnja
(profesori vode razred i dodaju situacije) · P4 ožujak (studentski pilot) · poslije.
Platformski kod ide u NOVE module (`app/src/platform/`, `api/src/platform/`, nove migracije od 0005) da se smanje sukobi s igrom;
DTO-i platforme ne idu u `api/src/schema.rs`. `openapi.json` se uvijek regenerira poslije spajanja, nikad ručno.

---
## FAZA PLATFORMA — SPREMNO ZA INTERNET (P2, dio P3; 13 cigli, 48 h)

**Gotovo kad:** Leon može pokrenuti HotelixSim na jednoj adresi (lokalno **0 €**), a nakon njegovog izričitog OK i na Railwayu (~5–10 €/mj):
otvori adresu u privatnom prozoru i Safariju, odigra igru; ako padne baza ili poslužitelj, vidi se zašto. Izlazak na internet samo uz Leonov OK.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F0-1 | Tipizirana `Config` | `server/src/config.rs`: sve `env::var` u jednu strukturu; `0.0.0.0:$PORT` (`main.rs:144`); `HOTELIXSIM_CONTENT` umjesto puta iz prevođenja (`content/src/lib.rs:74-76`) | nepotpuna okolina = jasna greška; `PORT=9000` sluša na 9000 | — | 3 | P2 | — |
| F0-2 | Uredno gašenje | SIGTERM + Ctrl-C (`?` preskače `worker.abort()`, `main.rs:157`), posao u tijeku dovršen ili vraćen | `kill -TERM` usred posla: izlaz 0, posao ne ostaje `running` | F0-1 | 2 | P2 | — |
| F0-3 | HTTP zaštita | `Secure` kolačić uz https (`auth.rs:31`), `SameSite=Lax`, provjera `Origin` na POST-u, CSP/HSTS/`nosniff`, limiti tijela, `tower_governor` | tuđi `Origin` 403; tijelo > limita 413; 61. zahtjev 429 | F0-1 | 5 | P2 | — |
| F0-4 | Poslužitelj servira web | `tower-http ServeDir` za `apps/web/dist`, SPA fallback; jedna adresa | `GET /` index, `/v1/health` JSON, nepoznata ruta index | F0-1 | 3 | P2 | — |
| F0-5 | Zdravlje | `SELECT 1` + verzija + otisak pravila umjesto `COUNT(*)` (`health.rs`) | ugašena baza: 503 u 3 s | F0-1 | 2 | P2 | — |
| F0-6 | sqlx TLS i brane | `tls-rustls`; `deny.toml` zabranjuje `native-tls`/`openssl`; testovi `connect_lazy` i `migrate` na svježoj bazi | `cargo deny check` zeleno; spoj sa `sslmode=require` | — | 3 | P2 | — |
| F0-9 | `ai_usage` + granica | projekcija iz `AiMeta` (`event.rs:140-145`), cijena modela u `content/`, provjera PRIJE stavljanja AI posla u red; iznad granice rezerva (ADR-019 t. 4) | lažni AI 2 $ → idući poziv na rezervu bez mreže | — | 6 | P2 | 2 $ po studentu već odlučeno |
| F0-10 | Predmemorija složene igre | `app`: `Game` po `game_id`, nakon `decide` samo `apply` novih događaja; ključ `(game_id, last_seq)`; **preduvjet za F1-10 u igra.md** | proptest: inkrementalno = puni fold; naredba na igri s 5 000 događaja | — | 6 | P2 | — |
| F0-11 | Dockerfile + slika u CI-ju | višefazni (cargo-chef, Rust 1.98.1, `SQLX_OFFLINE=true`), `content/` i `dist` u slici; CI gradi bez objave | CI zelen; `docker run` u CI-ju odgovara na `/v1/health` | F0-1, F0-4 | 5 | P2 | — |
| F0-13 | Kopije baze (lokalno) | `scripts/backup.sh` (`pg_dump`) + `scripts/restore-test.sh` | restore ispiše jednak broj `events` i `OK` | — | 3 | P3 | — |
| F0-14 | Zapisi i Sentry | JSON zapisi, Sentry (Rust + web, EU), upozorenje na posao `failed` | bačena greška stigne u Sentry | F0-1 | 3 | P3 | račun (besplatna kvota) |
| F0-S1 | Railway Hobby: servis + Postgres | Postgres u istoj regiji, `ANTHROPIC_API_KEY` samo ondje, `migrate` prije pokretanja, `*.up.railway.app` | `curl .../v1/health` 200; kolačić `Secure`; prijava radi u Safariju | F0-1…6, F0-11 | 4 | P2 | **izričit OK + trošak ~5–10 €/mj** (D6) |
| F0-S2 | Mjerenje kašnjenja | RTT poslužitelj ↔ baza, p95 naredbe pod opterećenjem | zapisani brojevi, ne dojam | F0-S1 | 3 | P3 | — |

Što još treba za Railway (BACKLOG, sve ovdje): `Idempotency-Key` + probni apalis/pgmq ([BACKLOG.md:37](../../records/BACKLOG.md)), serijski radnik ([:61](../../records/BACKLOG.md)), `ctrl_c` ([:85](../../records/BACKLOG.md)), deadlock expiry ([:81](../../records/BACKLOG.md)); uloga baze i paralelni radnici su u PILOTU.

---
## FAZA PLATFORMA — RAZRED (P2; 21 cigla, 116 h)

**Gotovo kad:** profesor se prijavi, stvori razred, pokaže kod na projektoru, studenti se prijave svojim računom, unesu kod i uđu u čekaonicu; profesor
pritisne „Pokreni" i otvara krugove (ručno ili po rasporedu); student zatvara svoj hotel, vidi tko je gotov i hotele kolega; neaktivnom hotelu rok
sam prenese plan. **Identitet je zamjenjiv:** računi u bazi, identiteti (AAI, Google) odvojeno i vežu se kasnije na isti račun (`identities`).
D4 (kako se profesor prijavljuje za probu) je **OTVOREN**; F1-4 je preporuka (jednokratna poveznica koju izdaje admin, 0 €).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F1-2 | Migracija platforme | `0005`: `users` (korisničko ime, otisak lozinke, uloga), `identities` (odvojeno), `memberships`, `courses`, `classes` (kod UNIQUE, postavke JSONB, `join_open`, `game_id` NULL do starta), `class_players`, `audit_log`, `games.class_id`; `slot CHECK` 1..4 → 1..64 (`0003:18`) | `sqlx migrate` svježa i kopija; `sqlx prepare` bez razlike; slot 30 da, 65 ne | F0-6 | 4 | P2 | — |
| F1-3 | Uloge i prava | `Caller` proširen (`demo.rs:10-12`); `permissions`; student vidi svoj razred, profesor svoje (tuđe 404) | matrica dopušteno/zabranjeno po ulozi (gost, student, profesor, admin) | F1-2 | 6 | P2 | — |
| F1-4 | Ulaz za profesora | admin CLI `server admin add-professor` izdaje jednokratnu poveznicu (24 h); sesija (`auth.rs`) | poveznica radi jednom; druga 410 | F1-3 | 6 | P2 | **D4 OTVOREN** |
| ID-1 | Prijava korisničko ime + lozinka | argon2/bcrypt otisak, prijava, promjena lozinke, ograničenje pokušaja po IP-u i računu | test: lozinka nikad u dnevniku ni odgovoru; 11. pogrešan pokušaj 429 | F1-3 | 6 | P2 | — |
| ID-2 | Generator računa za studente | profesor unese popis studenata, sustav izda korisnička imena i početne lozinke (jednokratni prikaz/CSV), prisilna promjena pri prvoj prijavi; profesor može resetirati lozinku | test: tuđi razred 404; popis s duplikatom odbijen; reset ruši staru sesiju | ID-1, F1-5 | 6 | P2 | oblik korisničkog imena |
| ID-3 | Web: prijava i lozinka | ekrani prijave, promjene lozinke, profesorov popis i reset | Vitest; mobitel | ID-2 | 5 | P2 | — |
| F1-5 | Razred i kod igre | stvori kolegij/razred (Week/Month, model v2, veličina, sjeme), generator koda (6–7 znakova bez 0/O/1/I), otvori/zatvori upis; postavke se zaključavaju u startu | kod jedinstven; zatvoren upis = kod ne radi; profesor A ne vidi razred B | F1-3, F1-4 | 7 | P2 | D3 veličina (preporuka 40), D11 |
| F1-6 | Ulaz kodom igre | prijavljeni student upisuje kod igre → `class_players` (kod za ulazak može ostati; ključ za povratak više nije identitet) | isti račun na drugom uređaju vidi isti hotel; pridruživanje nakon starta odbijeno | F1-5, ID-1 | 4 | P2 | D2 |
| F1-8 | Pokretanje iz čekaonice | „Pokreni" jednom transakcijom stvara `GameCreated` s prijavljenima (jezgra: F1-7 u igra.md), `hotels` slot 1..N, `classes.game_id` | 7 prijavljenih → 7 hotela; drugi „Pokreni" ništa; pad usred ne ostavlja polovicu | F1-2, F1-6, F1-7 | 4 | P2 | D2 lobby |
| F1-11 | App naredbe i API | `open_round` (profesor), `close_round` po hotelu (`app/commands.rs:96-134`), rute u `api/src/platform/`, OpenAPI → klijent | svaka ruta s ispravnom i pogrešnom ulogom | F1-3, F1-10 (igra.md) | 8 | P2 | — |
| F1-12 | Vidljivost za kolege | `app`: ljestvica napretka, **radnici tuđih hotela (koliko je dobar, doprinos uspjehu; skrivene osobine skrivene)**, ocjene i cijene; troškovi i odgovori na situacije skriveni; DTO u zasebnoj datoteci | test po polju: „API ne vraća privatno" | F1-10, L2a-12 (igra.md) | 8 | P2 | D8 imena ili nadimci; plaća tuđeg radnika (anketa) |
| F1-13 | Odabir igre korisnika | `me/hotel`, `me/classes`: razred ili demo (`store/src/guests.rs` `latest_hotel_of`) | korisnik s demom i razredom dobiva razred | F1-8 | 3 | P2 | — |
| F1-14 | Web: kod i čekaonica | „upiši kod", čekaonica „čekamo profesora (N prijavljenih)" | Vitest + proba na mobitelu | F1-6, ID-3 | 6 | P2 | — |
| F1-15 | Web: student u razredu | stanje „čekaš otvaranje", zatvori svoj hotel, lista kolega (gotov / još igra, uvijek se može vidjeti hotele kolega), javni pogled | Vitest; Playwright | F1-11, F1-12 | 8 | P2 | — |
| F1-16 | Web: profesor | stvori razred, kod krupno (projektor), popis prijavljenih (polling), „Pokreni", „Otvori sljedeći krug" | Playwright tok profesora; mobitel i računalo (ADR-027) | F1-11 | 8 | P2 | — |
| F1-17 | E2E: razred od tri studenta | profesor + 3 studenta kroz 2 kruga, vrata, različit tempo | zelen u CI-ju s pravim Postgresom | F1-14…16 | 6 | P2 | — |
| RZ-1 | Raspored otvaranja i rokovi | posao `open:<igra>:<krug>` s `run_after` (`0004` postoji); raspored (npr. tjedno) ili ručno; 17 tjednih krugova ≈ semestar | posao u prošlosti otvori krug jednom (idempotentno) | F1-11 | 5 | P2 | potvrđeno 13.10. |
| RZ-2 | Pravilo za neaktivne | posao po roku zatvara hotelu krug naredbom sustava; hotel prelazi sam s istim planom, situacije neodgovorene (jezgra: F1-9, mentor: M-1 u igra.md) | test: rok istekao → sljedeći tjedan s prošlim planom, ostali netaknuti | RZ-1, F1-9 | 4 | P2 | odluka 13.10. |
| RZ-3 | Obavijesti u aplikaciji | zvonce: novi tjedan, ponuda za radnika (N-8), reset lozinke; bez e-pošte | Vitest; polling 5 s | F1-15 | 6 | P2 | odluka 13.10. |
| RZ-4 | AI granica po igri/semestru | zbroj `ai_usage` po razredu, profesor vidi i povisi, globalni osigurač (F2-9) | iznad granice svi na rezervu; povišenje djeluje odmah | F0-9 | 4 | P2 | da (13.10.); iznos |
| AAI-0 | AAI@EduHr: istraživanje (bez koda) | pitanja fakultetu/Srcu: tko je IdP, SAML/OIDC, atributi, rok; ovisi o fakultetu | pisani odgovor ili zapis da nema informacija | — | 2 | P2 | Leon/fakultet |

F1-1 (prepis arhitekture) je u [kvaliteta.md](./kvaliteta.md) kao K0-7. Kritični put razreda: F1-7 → F1-9 → F1-10 (igra.md) pa F1-11.

---
## FAZA PLATFORMA — PROFESOR (P3; 16 cigli, 117 h)

**Gotovo kad:** profesor vidi razred uživo (tko je u kojem krugu, tko čeka, AI trošak), otvara pojedinog studenta u načinu samo čitanje, vidi račun dobiti
svih hotela i presude koje treba pogledati, i **piše vlastitu situaciju** kroz formu (3–5 odgovora i ocjena kvalitete, ne JSON) koju studenti dobiju.
Linija reza: vlastite situacije (F2b) P3 → P4. F2b ovisi o spojenoj grani `../hotelixsim-s1-sit` (L1-11 u igra.md).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F2-1 | Projekcija `hotel_progress` | tablica u istoj transakciji kao obračun (`ports.rs:107-117`); ponovno izgradiva (`rebuild`); jedna igra, N redaka po krugu | projekcija = izvedeno iz dnevnika | F1-10 (igra.md), F1-2 | 7 | P3 | — |
| F2-2 | API pregleda razreda | `GET /v1/classes/{id}/overview`; ETag po `max(events.id)`, 304 | 304 kad nema promjene; tuđi razred 404; `audit_log` | F2-1 | 6 | P3 | — |
| F2-3 | Profesorov račun dobiti | ruta + ekran svih hotela jedan uz drugoga; izvor brojki je P9-1 (igra.md) | zbroj po stavkama = izvještaj hotela; student 403 | F2-2, P9-1 | 6 | P3 | — |
| F2-4 | Detalj studenta (samo čitanje) | postojeći ekrani hotela bez tipki za mijenjanje | Playwright; pristup u `audit_log` | F2-2 | 8 | P3 | — |
| F2-5 | Web: ekran razreda | tablica studenata, krug, napredak, tko čeka, AI trošak, polling 5 s s ETagom | Playwright; 304 bez tijela | F2-2, F1-16 | 10 | P3 | — |
| F2-6 | Pregled presuda | presude s `review: true` i `VerdictLate` (ADR-019), razgovor i rubrika, samo čitanje | lista = točno neutralne/zakašnjele | F2-2 | 6 | P3 | — |
| F2b-1 | Dizajn forme situacije | ADR 🟨: forma (naslov, opis, vrsta, odjel, 3–5 odgovora, ocjena 1–5, rubrika 5 kriterija, lik) → generator slaže `Template` (`templates.rs:54`) | primjer forme → JSON koji prolazi `check` | s1-sit spojen | 4 | P3 | D10 |
| F2b-2 | Provjera i prefiks | u `content`: `validate_extra_templates(...)`, šifre `P-X-<razred>-…`, greške s putanjom polja | kolizija šifre / nepoznat odjel / rubrika ≠ 5 daju imenovanu grešku | F2b-1 | 6 | P3 | — |
| F2b-3 | Pohrana i API | `class_situations`, CRUD samo za vlasnika razreda | tuđi razred 404; nevaljan 422 s poljem; izmjena nakon starta odbijena | F2b-2, F1-5 | 7 | P3 | — |
| F2b-4 | Spoj u snimak | pri „Pokreni" predlošci idu u snimak pravila (`GameCreated.snapshot`) | igra se ponovno folda; bez dodanih bajt-isto | F2b-3, F1-8 | 5 | P3 | — |
| F2b-7 | Web: uređivač situacija | forma, popis, „pogledaj kao student", samo prije starta | Playwright | F2b-3 | 14 | P3 | D10 |
| F2b-9 | E2E vlastite situacije | profesor doda → student dobije → odgovor → presuda → obračun | e2e zelen | F2b-4, F1-17 | 4 | P3 | — |
| F2b-5 | Smještaj u krug (pin) | polje „krug/mjesec" (`serde(default)`) | situacija s pin dolazi točno u tom krugu | F2b-4 | 8 | P4 | D12 |
| F2b-6 | Broj i vrste situacija | postavke razreda: najviše po krugu, uključene `Kind` vrste | isključena vrsta se ne izvuče; balans `--check` | F1-5, F2b-4 | 6 | P4 | D11, D12 |
| F2b-8 | AI pomoć pri pisanju | Sonnet predloži odgovore i rubriku, profesor odobri; trošak u `ai_usage` | izlaz prolazi `check` ili odbijen | F2b-3, F0-9 | 10 | poslije | D12 |
| F2b-10 | Uređivač lika (ADR-021) | profesor uređuje opis/uvod lika | lik prolazi provjeru | F2b-7 | 10 | poslije | — |

F2-7, F2-10 i F2-11 su u PILOTU. Sati: P3 = 83 (F2-1…6 43 + F2b 40), P4 = 14, poslije = 20.

---
## FAZA PLATFORMA — PILOT (P4; 19 cigli, 95 h)

**Gotovo kad:** razred od ~30 studenata odigra semestar na internetu pod vlastitom domenom, s propisanim čuvanjem i brisanjem podataka, bez
gubitka događaja, i profesor može ispraviti presudu. Pravni odgovori fakulteta traju tjednima: pitanja poslati odmah (G-6).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F0-7 | Uloga baze bez prava nad `events` | migracija: aplikacijska uloga bez UPDATE/DELETE/TRUNCATE, zasebna uloga održavanja | `DELETE FROM events` aplikacijskom ulogom pada na pravu | F0-6 | 4 | P4 | — |
| F0-8 | Radnici i trake | N paralelnih radnika (`main.rs:123-136`), trake razgovor/presude/obračun, fix deadlocka expiry | 30 obračuna + 30 razgovora, razgovor ne čeka iza obračuna | F0-1 | 6 | P4 | — |
| F0-12 | CI dopune | akcije po SHA, `pnpm audit`, cache `sqlx-cli`, Playwright e2e s pravom bazom | e2e hvata namjerno pokvaren ekran | F0-11 | 6 | P4 | — |
| F0-15 | Alat za opterećenje | `oha`/`k6`: prijava, 30 klijenata, p50/p95 | skripta ispiše brojke | F0-10 | 3 | P4 | — |
| F0-S3 | Vlastita domena | `hotelixsim.com` (Cloudflare Registrar), CNAME `app.`, TLS od Railwaya | `https://app.hotelixsim.com/v1/health` 200 | F0-S1 | 2 | P4 | **Leon kupuje (~10 $/god), D6** |
| F0-S4 | Tok isporuke | `main` → staging; tag `v0.x.y` → produkcija uz obaveznog odobravatelja Leona (ADR-005) | tag čeka odobrenje | F0-S1, F0-12 | 3 | P4 | — |
| F0-S5 | Kopije izvan Railwaya | noćni `pg_dump` u EU spremnik, jedna stvarna proba vraćanja | datoteka < 24 h; restore zelen | F0-S1, F0-13 | 3 | P4 | trošak ~0–5 € |
| F2-7 | Ispravak presude | `VerdictOverridden` + kompenzacija u sljedećem krugu (ne ponovni obračun); po hotelu; audit | učinak točno jednom; fold determinističan | F1-10, F2-6 | 20 | P4 | — |
| F2-10 | Izvoz CSV | napredak, pet indeksa po krugu, AI trošak | sadržaj = projekcija; student 403 | F2-1 | 5 | P4 | — |
| F2-11 | Uklanjanje studenta | hotel se smrzne ili igra na zadanim potezima (neaktivni plan); roboti samo za vježbu | ostali nastavljaju; ljestvica „ispao" | F1-10 | 5 | P4 | **D7** |
| G-1 | Obavijest o privatnosti | kratka obavijest + upozorenje „ne upisuj osobne podatke" | ekran; tekst odobren | F1-14 | 3 | P3 | tekst |
| G-2 | Rok čuvanja | posao briše razred i igru nakon roka; zapis u `audit_log` | stara igra nestane, ostale nedirnute | F0-7 | 8 | P4 | **rok (pitanje fakultetu)** |
| G-3 | Brisanje studenta u zajedničkoj igri | anonimizacija (veza račun ↔ pseudonim, imena) + redakcija slobodnog teksta/AI razgovora uz `audit_log` | iz baze se ne dođe do imena/e-pošte; ostali hoteli nepromijenjeni | F1-6, F0-7 | 8 | P4 | **pravni odgovor fakulteta** |
| G-4 | Izvoz vlastitih podataka | JSON izvoz studentova hotela | samo njegovi podaci | F1-12 | 4 | P4 | — |
| G-5 | Brisanje demo gostiju | posao: demo gosti i igre starije od 7 dana (ADR-016) | stari nestanu | F0-8 | 3 | P4 | — |
| G-6 | Pravni popis | popis obrade, podizvršitelji (Railway, Sentry, Anthropic), nacrt za službenika | dokument u `docs/`; pitanja poslana | — | 3 | P4 | **šalje Leon fakultetu** |
| G-7 | Proba vraćanja kopije | mjesečna, na stvarnoj kopiji staginga | zapis s datumom i brojem događaja | F0-S5 | 3 | P4 | — |
| G-8 | Proba 30 studenata | 30 klijenata u jednoj igri na stagingu: p95, zaključavanje, AI trošak | p95 < cilj; nema `failed` poslova; D3 granica | F0-15, F1-17, F0-S1 | 4 | P4 | D3 |
| G-9 | Čuvanje IP-a | zapisi bez trajnog IP-a, 14 dana [PROC] | nakon roka nema IP-a | F0-3 | 2 | P4 | — |

G-1 je jedina P3 cigla ove faze (3 h). Prave greške koje valja riješiti uz pilot: [kvaliteta.md](./kvaliteta.md).

---
## FAZA PLATFORMA — poslije (smjer; ~105 h)

| id | cigla | što se radi | dokaz | h | Leon |
|---|---|---|---|---|---|
| GENERATOR HOTELA | F3-1…8: opis hotela u snimku (`HotelSetup.profile` → `"gen:1"`), `generate(sjeme, postavke, slot)`, banka imena (bez stvarnih hotela s Kvarnera, ADR-002), referenca po hotelu, balans `--generate`, postavka raznolikosti | A/B/C bajt-isto; proptest determinizma | 56 | popis imena, prag; ovisi o vrstama soba i obnovi |
| AAI@EduHr | F5-1 generički OIDC (`openidconnect`, PKCE) · F5-2 Google/Microsoft za profesora · F5-3 Supabase Auth kao posrednik (SAML) · F5-4 povezivanje na isti `user_id` (povijest ostaje) · F5-5 TOTP za admina · **čim fakultet odobri** (AAI-0) | test s lažnim IdP-om; ista povijest nakon povezivanja | 29 | D4; ovisi o fakultetu i Srcu |
| SSE | `LISTEN/NOTIFY` + ruta, samo ako G-8/polling pokaže potrebu | G-8 | 8 | — |
| ENGLESKI (sučelje) | `packages/i18n/src/en.ts` uz `hr.ts` (627 redaka), tipiziran lokal, izbor jezika, `format.ts` bez tvrdog hr-HR ([BACKLOG.md:67](../../records/BACKLOG.md)), `t()` tipizirani parametri ([:69](../../records/BACKLOG.md)); sadržaj i prompti su u [igra.md](./igra.md) | ista igra na hr i en; brojevi i datumi po lokalu | ~12 | popis jezika |

---
## Odluke (Leon)

Odlučeno 13.10.: D1 obračun po hotelu · raspored i rokovi · neaktivan student (hotel sam dalje, mentor upozori) · obavijesti u aplikaciji · AI granica po igri/semestru ·
generator računa prije AAI. **Otvoreno:** D4 profesor za probu · D2 lobby · D3 veličina igre (preporuka 40, mjeri G-8) · D6 domena i izlazak (OK + trošak) ·
D7 uklonjeni student · D8 imena ili nadimci · D10 sloboda profesora (forma) · D11 dani kao razdoblje (preporuka ne) · D12 AI situacije unaprijed ili u hodu · plaća tuđeg radnika (anketa).

## Sažetak ([PROC])

| faza | cigli | h | P |
|---|---|---|---|
| SPREMNO ZA INTERNET | 13 | 48 | P2 (P3: F0-13, 14, S2) |
| RAZRED | 21 | 116 | P2 |
| PROFESOR | 16 | 117 | P3 83 · P4 14 · poslije 20 |
| PILOT | 19 | 95 | P4 (G-1 P3) |
| poslije | 4 grupe | ~105 | poslije |

## Rizici

Dvije grane na istim datotekama (`api/schema.rs` + `openapi.json`, migracije: dogovoriti raspon brojeva) — zato K0-4 u kvaliteta.md prije; jedna igra s 30 pisaca
(F0-10 preduvjet, G-8 dokaz); GDPR u zajedničkoj igri (G-3, odgovor fakulteta kasni); AI je najveća stavka troška (RZ-4); F2b blokira nespojena grana s1-sit.
Najviše 2 agenta koji prevode Rust istodobno.
