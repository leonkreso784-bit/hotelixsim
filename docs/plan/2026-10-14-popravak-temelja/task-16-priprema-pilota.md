Kartica **Pilot** iz [plana popravka temelja](../2026-10-14-popravak-temelja.md) (redak „Pilot": „kartica piše sesija 12"). Cilj pilota:
1 profesor i 20–40 studenata igraju ovu igru na internetu nekoliko tjedana, na jednoj adresi, bez prijave, uz zaštićen AI trošak.
Izvor posla i brojki: prijedlog hostinga `.superpowers/sdd/2026-10-14-popravak-temelja/pilot-hosting-prijedlog.md` (19.10., §1.1, §3, §4);
kod provjeren 19.10. na `26089c3` (samo čitanje, bez prevođenja). Ništa od ovoga se ne gradi dok Leon ne vidi plan (ADR-062, „Što se mijenja").

### Task Pilot: Priprema pilota na Railwayu

**Veličina:** 🟨 [PROC] ~20–27 h rada agenata (prijedlog §4.1) + ~1 h Leonova posla · **Ovisi o:** L1-15 za spajanje u `main` i deploy;
gradnja dijelova 1–8 počinje usporedo s L1-14 (Leon 19.10.), jer nijedan dio ne dira jezgru · **Model:** čuvar i graditelj sonnet,
provjeravatelj **opus** za dijelove 2 i 3 (ulaz i novac), ostali sonnet · **Prevodi Rust:** dijelovi 1–4, **nikad dva usporedo**
(jedan Rust slot po sesiji, drugi drži sesija banke).

## Što je odlučeno, što nije

- ✅ **Odlučeno** ([ADR-062](../../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9): Railway Hobby, Amsterdam; **jedna lozinka
  za cijelu stranicu** (HTTP Basic Auth); strop AI **5 $/dan i 30 $/mj u aplikaciji**; **40 $/mj u Anthropic konzoli** (zaseban radni prostor);
  **najviše 3 igre po gostu**.
- ✅ Deploy je odobren kao pilot na kraju faze B, nakon Leonova OK-a za `model-v2/sloj1` → `main`, i to **samo taj jedan**
  ([ADR-060](../../records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12); ADR-062 t. 9: „deploy i dalje tek uz izričit OK".
  Ništa u kodu ne ovisi o Railwayu, sve iz varijabli okoline ([ADR-004](../../records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md)).
- ✅ **Leon 19.10. (anketa sesije 12):** kolačić gosta **120 dana s obnovom** pri otvaranju igre (klizni rok); besplatna adresa
  **`*.up.railway.app`**; **bez** ograničenja po IP adresi; gradnja dijelova 1–8 **smije odmah, usporedo s L1-14** (potvrđuje ADR-062 t. 18).
- **Za trajanje pilota jezgra se zamrzava:** svaka promjena `CORE_VERSION` daje 409 postojećim igrama (prijedlog §4.1, [BACKLOG](../../records/BACKLOG.md)
  „[pilot]"). Nijedan dio niže ne dira `game`, `sim-hotel*`, `content` ni balans.
- **Leon sam** otvara Railway račun (kartica, DPA) i Anthropic radni prostor s ključem i stropom; ključ upisuje izravno u varijable Railwaya.

## Što je u kodu danas (provjereno)

| stvar | gdje | posljedica |
|---|---|---|
| kolačić `HttpOnly; SameSite=Lax; Path=/; Max-Age=604800`, bez `Secure` | `crates/api/src/auth.rs:32-37` | 7 dana, pa student gubi hotel |
| kolačić se šalje samo novom gostu | `crates/api/src/demo.rs:73-76`; test `crates/api/tests/hotels.rs:5-30` (pribija `Max-Age=604800` i „postojeći gost ne dobiva novi kolačić") | nema obnove |
| gost u bazi nema roka | `crates/store/migrations/0003_platform.sql:2-6` | rok drži samo preglednik, baza ne treba promjenu |
| `POST /v1/demo` bez ikakve provjere; postojeći gost dobiva novu igru, gost bez kolačića novi račun | `crates/app/src/demo.rs:62-117`; igra pamti `created_by` (`0003_platform.sql:11`) | brojanje igara po gostu je jednostavno, ali gost bez kolačića ga zaobilazi |
| AI iz `ANTHROPIC_API_KEY`, bez stropa | `crates/server/src/main.rs:52-66` | omotač ide ovdje |
| `AiError::Disabled` se ne ponavlja i ide na rezervu | `crates/app/src/jobs.rs:121-124`, `crates/app/src/ports.rs:153-162` | strop ne ruši igru |
| tokeni se čitaju samo `input_tokens`/`output_tokens` | `crates/ai/src/response.rs:28-34`; predmemorija upita uključena `crates/ai/src/request.rs:107` | upis u predmemoriju se ne broji |
| odrezan odgovor (`max_tokens`) postaje `BadOutput` **bez** tokena, a ponavlja se do 5 puta | `crates/ai/src/response.rs:62`, `ports.rs:103`, presuda `max_tokens: 16000` `request.rs:106` | zbroj iz `events` promaši baš najskuplji slučaj |
| sav promet koji nije `/v1` dobiva 404 Problem | `crates/api/src/lib.rs:62-69` (`fallback(unknown_route)`) | posluživanje weba mijenja ovo mjesto |
| web zove istu adresu | `apps/web/src/main.tsx:16` (`window.location.origin`), Vite proxy `apps/web/vite.config.ts:10` | jedna adresa bez CORS-a |
| gašenje samo na Ctrl-C, radnik se prekida `abort()` | `crates/server/src/main.rs:155-160`; zakup posla 180 s `crates/app/src/jobs.rs:23` | SIGTERM s Railwaya: posao se ne gubi, kasni do 3 min |
| zdravlje broji `COUNT(*)` nad `events` | `crates/store/src/lib.rs:36` | skupo i odaje broj događaja |
| `tower-http` nije naša ovisnost (samo posredno) | `Cargo.toml:30` (samo `tower`), `Cargo.lock:2124` | nova ovisnost; `deny.toml:12` je provjerava |
| `tokio` već ima `signal` | `Cargo.toml:29` | SIGTERM bez nove ovisnosti |
| Dockerfile i `.dockerignore` ne postoje; **Docker nije instaliran lokalno** | korijen repoa; `docker --version` → nema naredbe | slika se može provjeriti samo u CI-ju (push) ili uz Docker Desktop (pitanje 5) |
| CI na svaki push i PR, bez isporuke | `.github/workflows/ci.yml:2-4`, web build `:62` | tok isporuke je nov |

## Dijelovi

| dio | što | dira | Rust | sati | čuvar pribija |
|---|---|---|---|---|---|
| **1 · kolačić** | trajanje 120 dana kao **jedna konstanta** (`10368000` s); **klizni rok**: `GET /v1/me/hotel` poznatom gostu ponovo šalje isti token s punim trajanjem; `Secure` kad je uključen produkcijski način | `crates/api/src/auth.rs`, `hotels.rs` (`me_hotel` 27-34), `demo.rs`, `lib.rs` (`AppState` nosi postavku), `crates/api/tests/hotels.rs`, `tests/common/mod.rs`, opis zaglavlja u `openapi.json` | da (`api`) | 1–1,5 | vidi niže |
| **2 · vrata poslužitelja** | (a) **Basic Auth**: lozinka iz varijable, usporedba u stalnom vremenu, bez lozinke u varijabli nema vrata (lokalno); (b) **Rust poslužuje `apps/web/dist`**: nepoznat `/v1/…` i dalje 404 Problem, ostali putovi `index.html`; (c) **SIGTERM** uz Ctrl-C; (d) zdravlje na `SELECT 1`; (e) **tipizirane postavke** jednim mjestom | `crates/server/src/main.rs` (+ `config.rs`), `crates/api/src/lib.rs`, `health.rs`, `crates/store/src/lib.rs:36`, `Cargo.toml` (`tower-http` s `fs`), `openapi.json` ako se mijenja `HealthBody` | da (`api`, `server`, `store`) | 4–5,5 | lozinka: bez nje 401 s `WWW-Authenticate`, s krivom 401, s dobrom 200; `/v1/health` bez lozinke 200; `/v1/nepostoji` 404 `application/problem+json`; `/hotel/x` vraća `index.html`; bez varijable lozinke ponašanje kao danas |
| **3 · strop AI** | omotač oko `AiPort` u `server`: prije poziva usporedi današnji i mjesečni trošak sa stropom, iznad vrati `AiError::Disabled` (rezerve, ne pad); trošak = tokeni × cijena modela (Sonnet 2/10 $, Haiku 0,10/0,50 $ po milijunu, prijedlog §3.1); zapis upozorenja u log | `crates/server/src/main.rs` (`ai_port`), novi modul u `server` ili `app`, `crates/ai/src/response.rs` (brojanje i odrezanih odgovora i predmemorije), 🟨 nova tablica troška (migracija) | da (`ai`, `app`, `server`, `store`) | 3–5 | ispod stropa poziv prolazi; preko dnevnog ili mjesečnog `Disabled` i posao ne ide u ponavljanje; novi dan otvara dnevni, ali ne mjesečni; odrezan odgovor se broji; strop iz varijable |
| **4 · 3 igre po gostu** | `start_demo` broji igre gosta (`games.created_by`); četvrta → nova greška (uzor `ChatLimit`, 409, `crates/app/src/error.rs:105-106`); poruka u webu | `crates/app/src/demo.rs`, `ports.rs` (`Guests`), `crates/store/src/` (+ `Memory`), `crates/app/src/error.rs`, `crates/api/src/problem.rs`, `openapi.json`, `packages/i18n/src/hr.ts`, početni ekran | da (`app`, `store`, `api`) + web | 2–3 | 3 igre prolaze, 4. odbijena i **ne** stvara igru ni gosta; drugi gost neovisan; broj iz varijable; web pokaže poruku |
| **5 · Dockerfile** | više faza: Rust `server` uz `SQLX_OFFLINE=true`, Node za `apps/web/dist`, mala završna slika s `server`, `content/`, `dist/`; `HOTELIXSIM_ADDR=0.0.0.0:8080`, `HOTELIXSIM_CONTENT` (`main.rs:35-38`, `:144`) | `Dockerfile`, `.dockerignore` (bez `.env`, `target/`, `node_modules/`) | ne lokalno (gradi CI) | 2–3 | slika se izgradi i `server openapi` u njoj radi (CI korak) |
| **6 · isporuka** | CI gradi sliku na PR-u bez objave; tok isporuke samo ručno (`workflow_dispatch`) uz GitHub okolinu „pilot" kojoj je **Leon obavezni odobravatelj**; `server migrate` prije pokretanja | `.github/workflows/ci.yml`, novi `deploy-pilot.yml` | ne | 2 | dry-run toka; okolina bez Leona ne pokreće |
| **7 · kopije** | provjeriti Railway kopije na Hobby planu (prijedlog §2a: nepotvrđeno) + skripta tjednog `pg_dump` na Leonovo računalo + **jedna proba vraćanja** u zasebnu lokalnu bazu (ne u razvojnu) | `scripts/pilot-kopija.sh`, kratko u `README.md` | ne | 2 | proba vraćanja: broj igara i događaja isti |
| **8 · privatnost** | kratka obavijest na početnom ekranu: nadimci, ne prava imena; tekst odgovora ide AI-ju (Anthropic, SAD); kontakt (prijedlog §4.1 t. 12) | `apps/web/src/screens/` početni ekran, `packages/i18n/src/hr.ts` | ne | 1–2 | vitest: tekst vidljiv na mobitelu i računalu |
| **9 · postava i proba** | usluga + Postgres u Amsterdamu, varijable (nova lozinka baze iz Railwaya, nikad lokalna), upozorenje potrošnje Railwaya i tvrdi strop, vanjski ping na `/v1/health`; proba: Safari i Chrome na mobitelu, privatni prozor, krug s razgovorom, presudom i mentorom, **namjerno nizak strop → rezerva** | Railway sučelje, varijable | ne | 2,5–4 | ručna lista u ledgeru |
| **10 · zapis** | ADR: hosting, stropovi, lozinka, kolačić; dopuna ADR-004 | `docs/records/DECISIONS/` | ne | 0,5 | `check:docs` |

**Ukupno ~20–28,5 h**, u granicama prijedloga (§4.1: 20–27 h); razlika je tablica troška u dijelu 3.

### Dio 1 pobliže (prvi, najmanji, samostalan)

- **Trajanje:** jedna konstanta u `auth.rs` (120 dana = `Max-Age=10368000`, Leon 19.10.). Ako Leon kaže drugo, mijenja se samo ona i jedan test.
- **Obnova: da, klizni rok** (Leon 19.10., ~0,5 h više): rok teče od zadnjeg otvaranja igre, ne od prvog. Mjesto je `GET /v1/me/hotel`
  jer web njime počinje (`apps/web/src/api/queries.ts:56`; graditelj potvrđuje da se zove pri svakom otvaranju). `POST /v1/demo` ostaje
  kakav je (postojeći gost ne dobiva novi kolačić — test `hotels.rs:26-29` ostaje). Bez obnove: student koji igra dulje od 120 dana opet
  gubi hotel; s obnovom samo onaj koji 120 dana ne otvori igru.
- **Produkcija:** `Secure` dolazi s varijablom produkcijskog načina (dio 2 e; do tada jedna varijabla samo za kolačić). `SameSite=Lax` ostaje:
  web i API su na istoj adresi pa kolačić nije „tuđi" (prijedlog §1.3). Lokalno (http) bez `Secure`, inače preglednik ne bi spremio kolačić.
- **Čuvar pribija:** (1) novi gost dobiva `Max-Age=10368000`, `HttpOnly`, `SameSite=Lax`, `Path=/`; (2) bez produkcijskog načina **nema** `Secure`,
  s njim **ima**; (3) `GET /v1/me/hotel` s poznatim tokenom vraća `Set-Cookie` s **istim** tokenom i punim trajanjem; (4) nepoznat ili pokvaren
  token ne dobiva kolačić; (5) postojeći gost na `POST /v1/demo` i dalje bez novog kolačića. Testovi idu u `crates/api/tests/hotels.rs`.

### Otvorene tehničke odluke (voditelj, 🟨 u ADR na `/kraj`)

1. **Basic Auth:** provjerava se samo lozinka, korisničko ime je bilo koje (student upiše bilo što); `/v1/health` je bez lozinke (ping i
   provjera Railwaya), a nakon dijela 2 d ne odaje ništa. Nalaz za probu: mobilni Safari može ponovo pitati lozinku nakon zatvaranja preglednika.
2. **Produkcijski način** (`HOTELIXSIM_PRODUKCIJA=1` ili slično): poslužitelj **odbija start** ako nedostaje lozinka, stropovi AI-ja ili mapa weba,
   i tada uključuje `Secure`. Time zaboravljena varijabla ne otvara stranicu svima. Cijena ~0,5 h; zatvara BACKLOG „Typed config".
3. **Gdje se broji trošak:** (A) zbroj `meta` iz `events` po `recorded_at` — bez migracije, ali promaši odrezane odgovore (do 5 × ~0,16 $ po
   presudi) i upis u predmemoriju; (B) **preporuka:** brojač vidi svaki odgovor Anthropica (i odrezan) i upisuje u vlastitu tablicu; točan,
   preživi restart, ~1–2 h više. Migracija bi bila `0005`, a plan platforme tim brojem zove F1-2 ([plan platforme](../2026-11-platforma.md)
   faza 2, red 1) — platforma tada prima sljedeći broj.
4. **Dan za dnevni strop:** UTC (jednostavno; u Hrvatskoj se dan otvara u 1 ili 2 h noću).
5. **Što student vidi kad je strop dosegnut:** ništa posebno, igra radi na rezervama (neutralna presuda, kratak odgovor lika, mentor iz brojki).
   Poruka „AI je danas potrošen" nije u opsegu.
6. **Gost bez kolačića zaobilazi 3 igre** (svaki put novi gost, `app/src/demo.rs:67-73`). Pravi čuvar je lozinka (dio 2) i strop (dio 3);
   ograničenje po IP-u Leon 19.10. **ne želi** za pilot (cijeli razred na fakultetskom Wi-Fiju dijeli IP).
7. **`tls-rustls` za `sqlx` ne treba** ako baza ide preko unutarnje mreže Railwaya (prijedlog §1.1: treba samo za bazu preko interneta) —
   🟨 provjeriti pri postavi (dio 9).

## Valovi

```text
val 1 (odmah po Leonovu OK-u za plan):  dio 1 kolačić (Rust slot)  ∥  dio 7 kopije (skripta)  ∥  dio 8 privatnost (web, tekst čeka Leona)
val 2 (kad je dio 1 spojen):            dio 2 vrata poslužitelja (Rust slot)  ∥  dio 5 Dockerfile (bez Rusta lokalno)
val 3 (kad je dio 2 spojen):            dio 3 strop AI (Rust slot)  ∥  dio 6 isporuka (yml)
val 4 (kad je dio 3 spojen):            dio 4 tri igre po gostu (Rust slot)
val 5 (nakon L1-15 i Leonova OK-a):     Leonovi računi → dio 9 postava i proba → deploy → dio 10 zapis
spajanje redom 1 → 2 → 3 → 4 (Rust), web i skripte kad su gotove; svaki uz cijeli CI i cmp balansa (mora biti bajt-isti)
```

- **Zašto redom:** dijelovi 1, 2 i 4 diraju `crates/api` (dio 1 i 2 oba `lib.rs`/`AppState`), a dijelovi 2 i 3 `crates/server/src/main.rs`; jedan
  Rust slot ionako ne dopušta usporedno. Dio 2 prije 3 jer strop čita postavke iz tipiziranih postavki. Kolačić prvi jer je najmanji i jedini
  koji štiti studentov hotel bez obzira na hosting.
- **Svaki dio:** svoje stablo `../hotelixsim-pilot-N` (apsolutni put), svoj `CARGO_TARGET_DIR=…/hotelixsim/target/pilot-N` (briše se odmah
  nakon spajanja), tok čuvar → graditelj → provjeravatelj ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)).
  Grana iz `model-v2/sloj1`. Gradnja samo na punjaču; ispod 50 GB slobodno ne gradi.
- **Dijelovi 5 i 6** se mogu potpuno provjeriti tek u GitHub Actions (nema Dockera lokalno), a to traži push grane — **push je objava**, uz
  Leonov OK (pitanje 5).

## Varijable okoline na Railwayu (dio 9)

`DATABASE_URL` (nova lozinka iz Railwaya) · `HOTELIXSIM_ADDR=0.0.0.0:8080` · `HOTELIXSIM_CONTENT` · `ANTHROPIC_API_KEY` (ključ pilot radnog
prostora, upisuje Leon) · lozinka stranice · stropovi (5 $/dan, 30 $/mj) · broj igara po gostu (3) · mapa weba · produkcijski način.
Imena varijabli određuje dio 2 e; nijedna vrijednost ne ide u git ni u `.env.example` (osim imena bez vrijednosti).

## Čeka Leona prije gradnje

Odlučeno 19.10.: kolačić 120 dana s obnovom, `*.up.railway.app`, bez granice po IP-u, gradnja odmah (bivša pitanja 1–3 i 8).

4. **Spustiti `max_tokens` presude 16 000 → 4 000** (najgori poziv 0,16 $ → ~0,04 $)? Dira presudu, pa samo uz mjerilo; preporuka: ne za pilot.
5. **Provjera Docker slike:** push grane pilota na GitHub da je CI izgradi (push je objava; repo je javan, tajni nema) ili Docker Desktop lokalno (disk).
6. **Railway:** upozorenje potrošnje 7 $ i tvrdi strop 15 $ (prijedlog §2a); tjedni `pg_dump` na tvoje računalo — tko ga pokreće.
7. **Tekst obavijesti o privatnosti** (dio 8) — voditelj daje nacrt, Leon odobrava.
9. **Leonov posao (~1 h, prijedlog §4.2):** Railway račun, Hobby, kartica, DPA; Anthropic radni prostor „HotelixSim pilot", novi ključ, strop 40 $/mj;
   GitHub okolina „pilot" s tobom kao odobravateljem i Railway token u tajnama; odabrati lozinku i reći je profesoru; OK za `sloj1` → `main`, pa za deploy.

## Trošak (prijedlog §2a, §3.3, §4.3; 1 $ ≈ 0,87 €, bez PDV-a)

| stavka | €/mj |
|---|---|
| Railway Hobby (poslužitelj + baza) | ~4,5–6 |
| AI (40 studenata, tjedna sezona) | ~15–30 u mjesecu igranja, inače ~0; tvrdo ograničeno na 30 $ (~26 €) u aplikaciji i 40 $ u konzoli |
| domena, nadzor, CI | 0 |
| **ukupno** | **~20–36 € u mjesecu igranja, ~5 € u mirovanju** |

**Najjeftinija varijanta koja čuva temelj** je ova (Railway Hobby + besplatna adresa): ista cijena kao OVH VPS, ali bez stroja koji netko
mora čuvati; selidba kasnije je `pg_dump`/`pg_restore` i nova `DATABASE_URL` (prijedlog §2, preporuka).

## Što se NE radi u pilotu

Prijava i računi (ID-1…3, F1-2…F1-4), razred i ulaz kodom (F1-5, F1-6, polje „kod razreda"), AAI@EduHr, više lozinki ili lozinka po razredu,
Vercel ili web i API na dvije adrese, Cloudflare kontejneri, više radnika reda (C2/F0-8; jedan radnik je za 40 studenata podnošljiv, prijedlog §1.4),
zaseban način `work`, uloga baze bez UPDATE/DELETE, i **bilo kakva promjena jezgre ili `CORE_VERSION`** dok pilot traje.
