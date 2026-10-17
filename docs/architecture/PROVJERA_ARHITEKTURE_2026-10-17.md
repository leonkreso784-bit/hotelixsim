# Provjera arhitekture 17.10.2026. — je li Rust jezgra + TypeScript sučelje ok za produkciju

> **Status:** 🟨 ČEKA LEONOVU ODLUKU (E4). Povod: Leon 17.10. na pitanje o
> [ADR-003](../records/DECISIONS/ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md): *„Meni je ok, ti provjeri duboko
> i reci mi je li ok"* ([ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md) E4).
> **Kako je rađeno:** neovisni provjeravatelj, samo čitanje koda i dokumenata na grani `model-v2/sloj1` (e71ecc4), bez gradnje.
> Uz to jedno mjerenje nad lokalnom bazom (82 probne igre, 3 647 događaja): veličine događaja i stvarni AI tokeni.
> Mjerilo cilja je ono što je Leon rekao 17.10.: **40 studenata igra tri mjeseca bez Leona uz računalo**, razred jedni protiv
> drugih ([ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)), svaki euro mjesečno se broji.
> Brojke koje nisu izmjerene označene su **[PROC]**, a što treba provjeriti izvan repoa **[PROVJERITI]**.
> Vezano: [ARHITEKTURA](./ARHITEKTURA.md), [PLATFORMA](./PLATFORMA.md), [REVIZIJA_TEMELJA](./REVIZIJA_TEMELJA.md),
> [plan platforme](../plan/2026-11-platforma.md).

## 1. Odgovor Leonu u pet rečenica

1. **Da, arhitektura je ok i ne treba je mijenjati:** Rust za jezgru i poslužitelj, TypeScript za ekran i PostgreSQL kao
   jedina baza dobar su izbor za razred od 40 studenata kroz semestar, i bolji od prepisivanja u „sve u TypeScriptu".
2. Najteži dijelovi već su dobro napravljeni: igra je niz zapisanih događaja koji se ne mogu prepraviti, račun je u cijelim
   brojevima, pravila su zamrznuta u svakoj igri, a AI ne piše brojke i nikad nije u pregledniku.
3. **Uvjet 1:** prije prvog pravog razreda mora se riješiti što se događa sa starom igrom kad popravimo jezgru, jer danas
   cijeli razred dobije grešku 409 i ne može ni otvoriti igru.
4. **Uvjet 2:** prije razreda od 40 trebaju predmemorija složene igre, više radnika za AI poslove i jedna proba
   opterećenja, jer danas svaka naredba čita cijelu igru dok drži bravu razreda, a presude idu jednu po jednu.
5. **Uvjet 3:** Cloudflare ne može vrtjeti naš poslužitelj kakav jest; najjeftiniji put koji čuva temelj je poslužitelj i baza
   u istom podatkovnom centru (Railway ~5–10 €/mj ili mali VPS ~5–8 €/mj [PROC]), Cloudflare za domenu, zaštitu i kopije,
   uz noćnu kopiju baze i probu vraćanja prije prvog studenta.

## 2. Nalazi

Oznake: ✅ drži · ⚠️ rizik (drži, ali uz uvjet) · ❌ ne drži za cilj. Dokaz je `datoteka:redak`.

| # | tema | sud | nalaz i dokaz |
|---|---|---|---|
| 1 | izbor jezika i dijelova | ✅ | Vidi §2.1. Rust jezgra + Axum + React/Vite + Postgres. Zaključak: **zadržati**; cijena je sporija gradnja na laptopu i hosting kao „stalno upaljen program", ne „funkcije na zahtjev". |
| 2 | determinizam i dnevnik događaja | ✅ / ⚠️ | Vidi §2.2. Obnova iz dnevnika stvarna; rupe: stara igra 409 (`crates/game/src/model.rs:142-151`), povratak na staru verziju programa puca (`deny_unknown_fields` 23× u `crates/game/src/event.rs`), model AI-ja nije u snimku igre. |
| 3 | razred s N hotela | ⚠️ | Vidi §2.3. Obračun je već petlja po hotelu (`crates/game/src/resolve.rs:161-195`), ali krug je zajednički (`RoundClosed` nema hotel, `event.rs:138-140`) i igra ima točno 4 hotela (`command.rs:53,76`; `0003_platform.sql:18`). Procjena 30–45 h stoji. |
| 4 | AI sloj i trošak | ✅ / ⚠️ | Vidi §2.4. Red poslova, rokovi i rezerve dobri; web nigdje ne zove Anthropic. Izmjeren trošak ~0,4–0,8 $ po studentu za 14 tjedana [PROC iz malog uzorka]; jedan serijski radnik ne izdrži razred. |
| 5 | hosting i isporuka | ⚠️ | Vidi §2.5. Cloudflare Workers ne mogu vrtjeti ovaj Axum poslužitelj; baza mora biti uz poslužitelj; nema kopija ni Dockerfilea (planirano F0-11, F0-13). |
| 6 | prijava i sigurnost | ⚠️ | Vidi §2.6. Sesije umjesto JWT-a (E1) su dobar izbor; jedna adresa rješava kolačiće; fale uloge, `Secure`, ograničenje zahtjeva i GDPR brisanje (sve planirano). |
| 7 | veličina i zdravlje koda | ✅ / ⚠️ | Vidi §2.7. ~36 000 redaka Rusta + ~36 700 redaka Rust testova, ~18 300 redaka weba, ~1 060 Rust i ~500 web testova, CI 4–7 min zelen; tri datoteke preko 1000 redaka u dugu. |
| 8 | što promijeniti sada | — | §3: sedam stavki, ~25–35 h ukupno, sve prije prvog izlaska na internet. |

### 2.1 Rust + TypeScript ili sve u TypeScriptu?

| | **danas: Rust jezgra i poslužitelj, TS ekran** | **alternativa: sve u TypeScriptu (Node)** |
|---|---|---|
| ista igra iz istog sjemena | brane u prevoditelju: zabrana `f64`, `HashMap`, sata (`clippy.toml`, `scripts/check-core-deps.mjs`), provjera preljeva i u izdanju (`Cargo.toml` `overflow-checks = true`) | moguće, ali samo dogovorom: broj u JS-u je decimalni (`number`), cijeli `i64` traži `BigInt` (spor i nezgrapan), ništa ne ruši gradnju ako netko upotrijebi `Math.random` ili redoslijed iz mape |
| brane balansa (1000 sezona × 3 prolaza u CI-ju) | brzo, CI cijeli traje 4–7 min (zadnjih 5 prolaza, `gh run list`) | izvedivo, ali sporije [PROC 5–20×]; vjerojatno manje sjemena po prolazu |
| kod koji piše AI | prevoditelj hvata većinu grešaka prije testa | više grešaka tek u radu |
| jedan jezik | dva jezika, ali veza je generirana (`openapi.json` → `packages/api-client`, CI pada pri razilaženju) | jedan jezik, dijeljeni tipovi bez generatora |
| hosting | stalno upaljen program u kontejneru ili na VPS-u | k tome i funkcije na zahtjev (Workers, Vercel) |
| laptop | spora gradnja, veliki `target/`, najviše 2 agenta (BUGS B8, B11, B12) | brže i lakše |
| cijena promjene | 0 | prepisati ~36 000 redaka koda i ~36 700 redaka testova; [REVIZIJA §5](./REVIZIJA_TEMELJA.md) procjenjuje gradnju ispočetka na ~540–910 h |

**Zaključak u jednoj rečenici:** TypeScript bi dao lakši laptop i jeftiniji „serverless" hosting, ali bi uzeo ono što je ovoj igri
najvrjednije (determinizam koji čuva prevoditelj i brz balans) i stajao bi više od pola godine rada, pa Rust ostaje.

Sitnica: [ARHITEKTURA §5.3](./ARHITEKTURA/04-sustav-jezici-kod.md) navodi TanStack Router i i18next, a web ih nema
(`apps/web/package.json`); to je dokumentacija koja kasni, ne greška u izboru.

### 2.2 Determinizam i dnevnik događaja

**Što drži (✅):**
- Stanje igre uvijek se slaže iz dnevnika (`Game::fold`, `crates/game/src/game.rs:114`), a obračun kruga se **sprema** kao
  događaj s punim izvještajem (`RoundResolved`, `event.rs:144-147`), pa čitanje nikad ne računa ponovno ni ne zove AI.
- Dnevnik je samo za dopisivanje i to čuva baza, ne kod (`crates/store/migrations/0002_events_append_only.sql:8-18`).
- Jedna brava po igri (`pg_advisory_xact_lock`, `crates/store/src/log.rs:109-114`) i `UNIQUE (stream, stream_seq)`
  (`0001_events.sql:13`): dva istodobna upisa se ne gaze.
- Snimak pravila je u `GameCreated`, a zapečaćeni otisak v2 vezan je uz `CORE_VERSION` (`crates/game/tests/v2_sealed.rs:279-296`):
  promjena obračuna bez dizanja verzije ruši test.

**Rupe:**
- ❌ za semestar: **stara v2 igra se ne može ni otvoriti** nakon dizanja `CORE_VERSION` (`model.rs:142-151` vraća
  `Outdated` već pri čitanju; obračun još jednom provjerava u `resolve.rs:206-213`). Danas je jezgra 2.5.0, a svaka cigla
  faze B diže verziju; u semestru bi svaki popravak jezgre značio „cijeli razred bez igre". To je poznato (REVIZIJA #6, C1).
  Čitanje stvarno ne treba staru jezgru, jer su rezultati spremljeni; staru jezgru treba samo **sljedeći obračun**.
- ⚠️ **Povratak na prethodnu verziju programa** (kad deploy pođe krivo): 23 vrste događaja imaju `#[serde(deny_unknown_fields)]`
  (`event.rs`). Ako nova verzija upiše novo polje, a mi vratimo staru, stara odbije te događaje i igra je „pokvarena".
  Za sadržaj je ta stroža provjera dobra; za događaje je zamka.
- ⚠️ Verzija oblika događaja postoji (`SCHEMA_VER = 1`, `event.rs:14`), ali „podizanje" starih nije izgrađeno: druga verzija
  se odbija (`event.rs:423-430`). Dok se dodaju samo polja uz `serde(default)`, to je dovoljno.
- ⚠️ **AI model i uputa nisu u snimku igre** (odgođeno u [ADR-019](../records/DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) t. 8):
  model se bira varijablom okoline (`crates/ai/src/lib.rs:38`), pa promjena usred semestra mijenja suca za pola razreda.
  Ponovljivost to ne ruši (izlaz AI-ja je spremljen), ali pravednost unutar razreda da.
- Iskreno obećanje iz [ARHITEKTURA §17](./ARHITEKTURA/13-sucelje-isporuka-sigurnost.md) („sezona je ponovljiva svojom verzijom")
  traži da se **slika programa čuva po verziji** (F0-11). Bez toga se stara sezona ne može ponovno izračunati.

### 2.3 Razred s N hotela

Dira jezgru, ali manje nego što zvuči. Tijelo petlje u `resolve_round` već računa jedan hotel iz njegova stanja i plana
(`resolve.rs:170-192`), a `RoundResolved` već nosi popis hotela (`event.rs:146`), pa obračun jednog hotela može upisati isti
događaj s jednim hotelom, bez nove verzije oblika. Što se mijenja:

| mjesto | danas | treba |
|---|---|---|
| stanje kruga | jedno za igru (`current_round`, `status`, `game.rs:425-446`) | po hotelu, uz „vrata" koja otvara profesor |
| zatvaranje | `RoundClosed { month }` (`event.rs:138-140`), zatvara svima (`crates/app/src/commands.rs:123-138`) | zatvaranje hotela H u krugu k; posao `resolve:<igra>:<hotel>:<krug>` |
| broj hotela | `[HotelId; 4]` (`command.rs:53,76`; `crates/app/src/demo.rs:75`), `slot BETWEEN 1 AND 4` (`0003_platform.sql:18`) | 1..N; roboti samo u igri za vježbu |
| izvlačenje situacija | po krugu za sve slotove (`drawn.rs:194-237`) | po hotelu kad mu se krug otvori |
| tko je stvorio igru | `games.created_by NOT NULL REFERENCES guests` (`0003:11`) | profesor (korisnik), ne gost |
| tko piše u dnevnik | `actor` samo `guest:`, `robot:`, `system` (`0002:5-6`) | i profesor (`user:<uuid>`), npr. otvaranje kruga, ispravak presude |

Ocjena veličine: **30–45 h** u kolosijeku IGRA (kao ADR-037 i plan platforme), plus platformski dio. Najveći rizik nije
jezgra nego **opterećenje** (§2.4, §3 t. 4): jedna igra s 40 pisaca znači jednu bravu za 40 ljudi.

### 2.4 AI sloj

**Što drži (✅):** sve AI uloge idu kroz red poslova, upis poruke i posla u istoj transakciji (`log.rs:118-130`); AI se zove
izvan brave; zakup 180 s je dulji od HTTP roka 150 s (`crates/ai/src/lib.rs:17`), a obračun čeka presude najviše 120 s
(`crates/app/src/jobs.rs:25`), pa poslije ide neutralna presuda; bez ključa sve radi na rezervama (`crates/server/src/main.rs:52-66`).
Ključ je samo na poslužitelju: web zove API na istoj adresi (`apps/web/src/main.tsx:16`), u `apps/web/src` i `packages/`
nema ni jednog poziva Anthropicu ni varijable `VITE_*`, `.env` je u `.gitignore`, `.env.example` ima prazan ključ.

**Trošak, izmjereno** (lokalna baza, prosjek po pozivu iz `payload->'meta'`; cijene po milijunu tokena: Sonnet 5.5 2 $ ulaz /
10 $ izlaz, Haiku 5.5 0,10 / 0,50 $):

| uloga (model) | poziva u uzorku | ulaz / izlaz tokena | $ po pozivu |
|---|---|---|---|
| lik (Haiku) | 50 | 988 / 93 | 0,00015 |
| presuda (Sonnet, `effort: high`) | 10 | 785 / 689 | 0,0085 |
| mentor kruga (Sonnet) | 53 | 5 518 / 437 | 0,0154 |
| mentor sezone (Sonnet) | 9 | 6 136 / 534 | 0,0176 |

**Procjena za 40 studenata × 14 tjedana [PROC]:** po studentu mentor 14 × 0,0154 ≈ 0,22 $; presude 14–56 (1–4 vlastita
odgovora tjedno; ADR-047 potiče pisanje) ≈ 0,12–0,48 $; lik 84–252 odgovora ≈ 0,01–0,04 $; kraj sezone 0,02 $ →
**~0,4–0,8 $ po studentu**, **~15–30 $ po razredu**, uz rezervu za ponovne pokušaje i rezervni model ~25–45 $. Granica
2 $ po studentu ([ADR-028](../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)) tako nije tijesna. Najveća stavka je
**mentor svaki krug**, ne presuda: kad ga profesor ugasi (E6), trošak pada gotovo na pola.

**Zašto je ovo još procjena:** uzorak presuda je mali (10); `AiMeta` sprema samo `input_tokens` i `output_tokens`
(`event.rs:199-204`), a ne tokene iz predmemorije uputa ni broj pokušaja, pa račun kod Anthropica može biti malo veći;
u uzorku su i 4 poziva modela `claude-opus-5-5` (vjerojatno rezervni model pri odbijanju, skuplji: 4/20 $). Kako izmjeriti:
Leon odigra jednu tjednu v2 sezonu s pisanim odgovorima i razgovorima (~1–2 $), pa upit iz
[plana platforme §5](../plan/2026-11-platforma.md) po tipu događaja; uz to dodati tokene predmemorije u `AiMeta` (§3 t. 5).

**Rizik (⚠️ → ❌ za razred):** jedan radnik obrađuje sve poslove redom (`main.rs:125-136`). Kad 40 studenata na satu zatvori krug
s 2–3 pisana odgovora, to je ~100 presuda po ~10 s [PROC] = ~15 min, a rok je 120 s: većina bi postala neutralna 3/5.
Paralelni radnici i zasebna traka za lika (C2 = F0-8) su zato **uvjet za P3**, ne za pilot. Treba provjeriti i ograničenje
istodobnih poziva po razini Anthropic računa [PROVJERITI].

### 2.5 Hosting i isporuka

**Prijelaz s Windowsa na Linux je pripremljen (✅):** `.sqlx/` u gitu i CI provjerava `cargo sqlx prepare --check`
(`.github/workflows/ci.yml:38`), pa se gradi bez baze; `.gitattributes` drži `eol=lf` za kod, upute i SQL, pa ugrađene upute
(`include_str!`, `crates/ai/src/prompts.rs:10-18`) i otisak sadržaja su isti na oba sustava; migracije su zasebna naredba
`server migrate` (`main.rs:18-23`). **Fali:** adresa je zadano `127.0.0.1:8080` (`main.rs:144`) i nema `$PORT`; sadržaj se
bez `HOTELIXSIM_CONTENT` traži na putu iz vremena gradnje (`crates/content/src/lib.rs:91-93`), koji u kontejneru ne postoji;
nema Dockerfilea; zdravlje broji sve događaje (`crates/store/src/lib.rs:35-38`). Sve je u fazi SPREMNO ZA INTERNET (F0-1, F0-5, F0-11).

**Cloudflare (Leon E3, 🟨):**
- **Workers** izvode JavaScript ili WebAssembly u kratkim izolatima. Naš poslužitelj je stalno upaljen program s Tokiom, vezama
  na Postgres (sqlx) i radnikom u pozadini; **takav kakav jest ne može raditi na Workersima**, a prepisati ga za njih znači
  prepisati sloj baze i red poslova.
- **Cloudflare Containers** [PROVJERITI stanje i cijenu] vrte Docker sliku, ali se gase kad nema prometa (naš radnik reda mora
  stalno raditi) i ne daju Postgres; baza bi bila kod drugog dobavljača, pa dva računa i kašnjenje do baze.
- **Realno mjesto za Cloudflare:** domena (Registrar po nabavnoj cijeni), DNS, TLS i zaštita ispred poslužitelja (proxy na istoj
  adresi, pa kolačići ostaju „jedna adresa"), i **R2 za noćne kopije baze** (besplatna kvota ~10 GB [PROVJERITI]). Tako
  Cloudflare ima smisla, a temelj se ne dira.

**Latencija — gdje je stvarno važna:** student ↔ poslužitelj iz Hrvatske prema EU podatkovnom centru je ~20–40 ms [PROC] i za
igru na poteze je nebitna. Važna je udaljenost **poslužitelj ↔ baza**, jer naredba drži bravu razreda dok traju svi povratci do
baze: `BEGIN`, brava, čitanje cijelog toka, **svaki događaj zaseban `INSERT`** (`log.rs:39-56`), poslovi, `COMMIT` → 6–20
povrataka. U istom podatkovnom centru (<1 ms) to je ništa; na 8 ms (npr. Railway Amsterdam ↔ Supabase Frankfurt [PROC]) to je
50–160 ms brave po naredbi, a 40 studenata čeka redom. Leonov strah od Supabasea je dakle opravdan, ali iz drugog razloga.

| put | €/mj [PROC] | baza uz poslužitelj | kopije | tko je dežuran |
|---|---|---|---|---|
| **A. Railway Hobby: poslužitelj + Postgres u istom projektu** (preporuka) | ~5–10 | da (<1 ms) | naš `pg_dump` → R2 | Railway za stroj, mi za bazu |
| B. mali VPS (npr. Hetzner, Njemačka) + Docker + Caddy, Postgres na istom stroju | ~5–8 | da | naš `pg_dump` → R2 ili Storage Box | **mi za sve** (nadogradnje, TLS, sigurnost) |
| C. VPS u Njemačkoj + Supabase Frankfurt samo kao baza | VPS + dodatni projekt u postojećoj Pro organizaciji [PROVJERITI ~10 $] | ~3–8 ms [PROVJERITI] | Supabase dnevne (7 dana) + naš `pg_dump` | podijeljeno |
| D. Cloudflare Workers/Containers | — | ne | — | ne preporučujem (gore) |

Put C je jedini koji koristi novac koji Leon već plaća i daje upravljane kopije, ali tek **nakon** predmemorije i skupnog upisa
(§3 t. 4) i uz izravnu vezu na bazu (ne „transaction" način spajanja, zbog sqlx-a [PROVJERITI]). **Jedna baza po okolini:**
razvoj ostaje lokalno (5433), produkcija ima svoju lozinku koja nikad nije bila u gitu (lokalna je u `.env.example`), a balans
ne dira bazu. **Oporavak od pada:** noćni `pg_dump` izvan hostinga (dnevnik je izvor istine), 14–30 dana, i proba vraćanja u
praznu lokalnu bazu jednom mjesečno; pad procesa usred posla već je pokriven zakupom posla.

### 2.6 Prijava i sigurnost

- ✅ **Sesije, ne JWT (E1):** neproziran token u `httpOnly` kolačiću, u bazi samo SHA-256 (`0003_platform.sql:4`); opoziv je
  jedan `DELETE`. Za jednog autora jednostavnije i sigurnije od JWT-a. [ARHITEKTURA §12](./ARHITEKTURA/09-podaci-ai-api-prava.md)
  još piše JWT i treba ga uskladiti (već u BACKLOG-u).
- ✅ **Kolačići na dvije domene** se ne pojavljuju ako Rust servira i web i `/v1` s iste adrese
  ([PLATFORMA §6.1](./PLATFORMA/06-produkcija-i-trosak.md)); Cloudflare proxy ispred iste adrese to ne mijenja. Web već danas
  zove API na `window.location.origin`.
- ⚠️ **Fali za internet:** `Secure` (`crates/api/src/auth.rs:31-37`), ograničenje zahtjeva i pokušaja prijave, provjera `Origin`
  na POST-u, zaglavlja (F0-3); uloge profesor/student i test „tuđi razred = 404" (F1-3). Sve planirano, prije izlaska.
- ⚠️ **GDPR:** imena i e-adrese ne ulaze u dnevnik (pseudonim `guest:<uuid>`, `0002:5-6`) — dobar temelj. Ali slobodni tekst i
  AI razgovori jesu u dnevniku koji se ne briše; brisanje studenta je zato redakcija posebnom ulogom koja privremeno zaobilazi
  okidač ([PLATFORMA §7.4](./PLATFORMA/07-gdpr-i-sigurnost.md), G-3). To je izvedivo; treba ga **jednom probati na kopiji** prije
  pilota. Anthropic obrađuje u SAD-u uz DPA [PROVJERITI za fakultet, F5].
- ✅ **Javni repo:** u gitu nema ključa; jedina „tajna" u povijesti je lokalna lozinka baze (`.env.example:1`), koja se zato nikad
  ne smije koristiti izvan laptopa.

### 2.7 Veličina i zdravlje koda

| crate | redaka koda | redaka testova (`tests/`) |
|---|---|---|
| `game` | 10 337 | 11 408 |
| `sim-hotel-v2` | 7 705 | 5 663 |
| `content` | 5 278 | 8 536 |
| `api` | 3 689 | 4 020 |
| `sim-hotel` (v1, zamrznut) | 2 744 | 1 895 |
| `balance` | 2 449 | 1 582 |
| `app` | 2 411 | 1 583 |
| `ai` | 798 | 1 101 |
| `store` | 356 | 696 |
| `server` | 189 | 0 (2 testa u datoteci) |
| `engine` | 145 | 206 |
| **ukupno Rust** | **~36 100** | **~36 700** |

Web: ~18 300 redaka TypeScripta (s testovima), 37 testnih datoteka. Testova: ~1 060 Rust (`#[test]`, `#[tokio::test]`,
`#[sqlx::test]`) i ~500 web. Pokrivaju jezgru, zlatne vektore kocke, pečat v2, prava po ulozi, red poslova nad pravim
Postgresom i tok kraj-do-kraja (`crates/api/tests/e2e.rs`); ne pokrivaju opterećenje, isporuku ni vraćanje kopije.
CI (5 zadnjih prolaza): 4–7 min, zelen. Brana `check:kod`: 20 datoteka iznad 600 redaka, a tri su u dugu iznad 1000:
`crates/api/src/schema.rs` (1 770; razdvaja ga C6 prije platforme), `crates/game/tests/mentor.rs` (1 687),
`crates/game/src/mentor.rs` (1 399). Dug koji raste: **v1 i v2 žive usporedo** (`sim-hotel` + `sim-hotel-v2`, dvije petlje
sezone, REVIZIJA #8); v1 je zamrznut, ali se održava u svakoj promjeni `game`. Kad pilot krene samo na v2, v1 treba umiroviti
(staviti samo za čitanje starih igara) [PROC ~10–20 h], ne prije.

## 3. Promijeniti sada (dok je sve lokalno, jeftino)

| # | što | cijena | zašto sada, a ne poslije |
|---|---|---|---|
| 1 | **Pravilo verzije za semestar + čitanje stare igre.** Odluka: tijekom semestra igra ostaje na verziji jezgre s kojom je počela, popravci jezgre idu između semestara, a hitni preko izričite nadogradnje kao događaja (C1). Odmah: `check_version` (`model.rs:142`) samo za naredbe koje obračunavaju, a čitanje dopustiti gdje testovi pokažu da ne treba staru jezgru | 3–4 h sada (+ C1 8–15 h prije P3) | nakon prvog pravog razreda svaki popravak jezgre bez ovoga ruši cijeli razred |
| 2 | **Događaji podnose povratak na staru verziju programa:** ukloniti `deny_unknown_fields` s događaja (`event.rs`, 23×), zadržati na sadržaju; test „događaj s nepoznatim poljem se čita" | ~2 h | prvi loš deploy na internetu inače nema puta natrag bez „pokvarenih" igara |
| 3 | **Shema razreda u jednom potezu (kartica F1-2):** `events.actor` dobiva `user:<uuid>` (`0002:5-6`), `games.created_by` korisnik ili gost (`0003:11`), `slot` 1..64, `games.class_id`. Upisati u karticu prije gradnje | ~1 h (dizajn) | `actor` je u svakom zapisu dnevnika; kasnija promjena oblika aktera znači dva oblika zauvijek |
| 4 | **Predmemorija složene igre + skupni upis + proba 40 studenata lokalno** (F0-10, `INSERT … UNNEST` umjesto petlje u `log.rs:39-56`, G-8 ranije). Izmjereno: `RoundResolved` ~48 KB za 4 hotela, tj. ~12 KB po hotelu i krugu → razredna igra ~8 MB [PROC], a danas se cijela čita pod bravom pri **svakoj** naredbi i pri svakom čitanju (`crates/app/src/demo.rs:153`) | 6 + 2 + 4–6 h | odlučuje hosting (smije li baza biti 8 ms daleko) i drži li „jedan tok po igri"; bez toga P3 puca na satu |
| 5 | **AI mjerenje:** `AiMeta` dobiva tokene predmemorije i broj pokušaja (uz `serde(default)`), projekcija `ai_usage` (F0-9), jedna Leonova izmjerena sezona | ~3 h + Leonovih 1–2 h igre | granica 2 $ i obećanje profesoru/inkubatoru trebaju brojku; svaka lokalna igra je besplatno mjerenje |
| 6 | **Model i verzija upute u snimak igre** (ADR-019 t. 8): `GameCreated` zapisuje modele po ulozi; poslužitelj koristi zapisane, a ne varijablu okoline | 3–4 h | igre stvorene bez toga ostaju bez zaključanog suca; poslije se ne može dopisati u stari `GameCreated` |
| 7 | **Kopije i proba vraćanja:** `scripts/backup.sh` (F0-13), vraćanje u zasebnu lokalnu bazu, pravilo „jedna baza po okolini" u README | ~3 h | najskuplja moguća greška je izgubiti dnevnik razreda usred semestra; skripta je jeftina dok je baza mala |

Ukupno ~25–35 h; stavke 4, 5 i 7 su već cigle plana platforme, ovdje se samo traži da idu **prije** izlaska i prije RAZREDA U JEZGRI.
Što **ne** treba mijenjati: jezik, okvir, bazu, red poslova (vlastita tablica je dovoljna za 40 studenata), jedan tok po igri
(dok proba iz t. 4 ne pokaže suprotno), ID-ove (UUID v4 za igre i hotele, `bigserial` za događaje su u redu).

## 4. Pitanja za Leona

1. **ADR-003:** (a) potvrditi Rust + TypeScript **uz tri uvjeta iz §1** (stara igra, opterećenje, hosting s kopijama)
   (preporuka: temelj stoji, uvjeti su ~25–35 h i ionako planirani); (b) otvoriti raspravu o prelasku na TypeScript
   (~540–910 h, gubi se determinizam koji čuva prevoditelj).
2. **Hosting za P3:** (a) Railway Hobby, poslužitelj i Postgres u istom projektu, Cloudflare za domenu, zaštitu i kopije u R2
   (preporuka: ~5–10 €/mj, najmanje dežurstva); (b) mali VPS u Njemačkoj + Supabase Frankfurt kao baza (koristi postojeću
   pretplatu i upravljane kopije, ali tek nakon §3 t. 4 i uz provjeru cijene dodatnog projekta); (c) VPS sa svime na jednom
   stroju (~5–8 €/mj, ali Leon je dežuran za nadogradnje i sigurnost).
3. **Stara igra u semestru:** (a) igra ostaje na svojoj verziji jezgre do kraja semestra, popravci jezgre između semestara,
   hitni kroz izričitu nadogradnju (preporuka: pravedno za cijeli razred, najmanje rizika); (b) svaki popravak jezgre odmah
   nadograđuje igre u tijeku (brže popravke, ali razred usred sezone mijenja pravila).
4. **Povratak na staru verziju programa:** (a) ukloniti strogu provjeru nepoznatih polja s događaja prije prvog deploya
   (preporuka, ~2 h); (b) zadržati je i uvesti pravilo „novo polje u događaju piše tek iduće izdanje" (stroža disciplina, lako
   se zaboravi).
5. **Proba opterećenja:** (a) proba 40 simuliranih studenata u jednoj igri lokalno postaje uvjet faze RAZRED U JEZGRI
   (preporuka, ~4–6 h; odlučuje hosting); (b) ostaviti probu za PILOT kako piše danas (G-8).

**Gdje se vidi:** samo u dokumentaciji (lokalno; na GitHubu tek nakon pusha). Kod nije mijenjan.
