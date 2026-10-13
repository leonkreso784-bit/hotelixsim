# PLATFORMA §6 · Produkcija i trošak

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Brojke troška su **[PROC]**: iz pretrage 12.10., uglavnom iz izvora trećih strana,
pa ih treba **provjeriti na službenim stranicama** prije odluke. Cigle su u fazi SPREMNO ZA INTERNET
([ROADMAP/platforma](../../plan/ROADMAP/platforma.md), F0-*).

## 6.1 Ključna odluka: jedna adresa

Dvije zadane domene, npr. `*.vercel.app` za web i `*.railway.app` za API, znače kolačiće među domenama. Safari i
privatni prozori takve kolačiće blokiraju, pa prijava „radi na mom računalu" i nigdje drugdje. Prijašnji nacrt (12.10.)
predlagao je web na Vercelu i API na Railwayu; odbačeno. **Rust poslužitelj servira i izgrađeni web** (`tower-http`
`ServeDir` za `apps/web/dist`, a `/v1/*` ostaje API). Rezultat:

- jedan servis i jedna isporuka;
- bez CORS-a i bez Vercela;
- SSE radi bez proxyja;
- web i API su uvijek iste verzije.

Danas Vite u razvoju već radi kao „jedna adresa" (`apps/web/vite.config.ts:10` proxy `/v1`), pa se ponašanje ne mijenja.
Ako jednom zatreba CDN: Vercel rewrite `/v1` na istoj domeni, ili poddomene iste domene uz kolačić s `Domain=` i CORS s `credentials`.

## 6.2 Preporuka o hostingu i trošku (dopune 13.10., čeka Leonovu potvrdu)

Hosting **još nije odlučen**: sve u ovom odjeljku je preporuka kontrolora, koja čeka Leonovu potvrdu (i zasebni ADR). **Leon plaća sam, bez prihoda** (već ~45–50 €/mj za Supabase Pro s tri projekta; jedan briše). Zato:

- **Do profesorova OK: sve lokalno, 0 € novog troška** ([ADR-004](../../records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md)).
- **Prvi izlazak na internet** (samo uz Leonov izričit OK, [ADR-005](../../records/DECISIONS/ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)):
  poslužitelj **i Postgres na Railwayu**, plan **Hobby** (5 $/mj, uključeno 5 $ potrošnje), u istoj regiji i na privatnoj
  mreži. Domena ~10 $/god. Ukupno **~5–10 €/mj** novog troška [PROC].
- **Supabase samo kao posrednik prijave (IdP) kasnije** (dio 03), u postojećoj organizaciji; baza igre nije na Supabaseu.
- PITR, Railway Pro, odvojen staging i Fly.io: **tek kad postoji financiranje** (fakultet, pilot, potpora). AI trošak po
  studentu ne kreće sa stvarnim razredom dok nije dogovoreno tko ga plaća (pitanje F8).

**Zašto baza uz poslužitelj:** naredba radi nekoliko povrataka do baze u jednoj transakciji (zaključavanje, čitanje toka,
upis, posao; `crates/store/src/log.rs`). Student ↔ poslužitelj ne može biti ispod 1 ms (Opatija–Amsterdam ~20–40 ms RTT;
za igru na poteze nebitno), ali poslužitelj ↔ baza može, ako su u istom podatkovnom centru. Prijašnji nacrt (12.10.)
predlagao je Supabase u Frankfurtu uz poslužitelj u Amsterdamu (5–10 ms po povratku); odbačeno. U kodu: jedna transakcija
po zahtjevu, složeno stanje u memoriji (F0-10). **Latencija se mjeri u F0-S2 prije plaćanja.**

**Kopije:** Railway Postgres je predložak, a ne upravljana baza. Kopije su zakazane snimke diska (0,15 $/GB-mj), a Railway
sam preporučuje i vlastiti `pg_dump` izvan Railwaya. Zato **noćni `pg_dump` u EU spremnik i mjesečna proba vraćanja** (F0-S5,
G-7) nisu ukras: najskuplja greška je izgubiti dnevnik događaja studenata usred semestra, jer je on izvor istine.

**Domena.** Ni Vercel ni Railway ne daju pravu domenu, nego poddomenu (`*.vercel.app`, `*.up.railway.app`). Problem kolačića
postoji samo s DVIJE adrese, ne s tuđom poddomenom. Zato su staging i pilot na **jednoj besplatnoj adresi**
`<ime>.up.railway.app` (Rust servira web + `/v1`), 0 €. Vlastita domena (`hotelixsim.com`, Cloudflare Registrar, nabavna
cijena, besplatni DNS i TLS) tek za stvarne studente (službeni izgled, adresa preživi promjenu hostinga); plan
`app.hotelixsim.com` → Railway. Kupuje Leon sam uz svoj OK (D6). Do vlastite domene Cloudflare se ne koristi.

### Mogućnosti hostinga (usporedba)

- **Railway:** Hobby 5 $/mj (uključeno 5 $ potrošnje), Pro 20 $/mj po radnom prostoru. Jedina EU regija je Amsterdam. Servis s
  diskom vezan je za regiju.
- **Supabase:** Pro 25 $/mj po organizaciji, 7 dana dnevnih kopija; PITR dodatak ~100 $/mj; SAML je na Pro i višim.
- **Vercel:** Pro 20 $ po razvojnom mjestu; nama ne treba (6.1).
- **Hetzner:** 2026. dizao cijene 30–50 %; najjeftinije, ali **mi** smo dežurni (nadogradnje, kopije, TLS, sigurnost).

| postava | trošak [PROC] | prednosti | mane |
|---|---|---|---|
| **1. Sve na Railway Hobby (servis + Postgres)** (preporuka) | ~5–10 €/mj | najjeftinije uz jednog dobavljača; baza uz poslužitelj, kašnjenje ispod 1 ms | kopije, proba vraćanja i nadogradnje Postgresa su **naš** posao; ograničenja Hobby plana |
| **2. Isto na Railway Pro** | ~20–30 $/mj | više resursa, radni prostor za staging i production | tek uz financiranje |
| **3. Railway + Supabase Pro za bazu** | ~45–55 $/mj | upravljana baza, PITR | odbačeno: dva dobavljača, kašnjenje između gradova, ~25 $/mj više; Leon već plaća Supabase bez prihoda |
| **4. Fly.io (Frankfurt)** | ~35–60 $/mj | poslužitelj i baza u istom gradu | rezervni plan ako F0-S2 pokaže problem |
| **5. Hetzner VPS + Coolify ili Caddy** | ~10–20 €/mj | najjeftinije, EU tvrtka | ruši pouzdanost uz jednog autora |
| **hrvatski smještaj** | — | GDPR **ne traži** Hrvatsku, dovoljni su EU/EEA ili zemlja s odlukom o primjerenosti | samo ako fakultet izričito traži (pitanje F5) |

### Trošak pri širenju (mjesečno, sve [PROC])

Pretpostavke: AI ~1–2 $ po studentu po sezoni; sezona ~4 mjeseca; svi studenti igraju u istom semestru. Hosting stupci
vrijede kad postoji financiranje (Pro razine); do tada vrijedi samo prvi redak 6.2.

| stavka | 30 studenata (1 razred) | 300 studenata (~10 razreda) | 3 000 studenata (~100 razreda) |
|---|---|---|---|
| Railway (poslužitelj + radnici + Postgres) | 5–20 $ | 25–60 $ | 100–250 $ |
| Cloudflare (DNS, TLS) + domena | ~1 $ | ~1 $ | 0–20 $ |
| Sentry, spremnik za `pg_dump` | 0–5 $ | 10–40 $ | 50–120 $ |
| **AI (Anthropic)** | **8–15 $** | **75–150 $** | **750–1 500 $** |
| **ukupno** | **~15–40 $** | **~110–250 $** | **~900–1 900 $** |
| udio AI-ja u ukupnom | ~40–55 % | ~60–70 % | **~80 %** |

**Zaključak:** pri širenju najveći trošak je **AI, ne hosting**. Zato su granica troška po igri (2.6), predlošci bez AI-ja
([ADR-028](../../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md) t. 8), jeftiniji model za lika ([ADR-024](../../records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md)) i mjerilo
prije promjene modela važniji od izbora hostinga. Arhitektura za 3 000 studenata ostaje ista: mijenja se broj instanci i
veličina baze, a razredi su neovisne igre koje se ne zaključavaju međusobno.

**Staging i produkcija:** dvije okoline istog oblika (Railway okoline „staging" i „production"), ali tek uz financiranje; do
tada jedna okolina. Staging dobiva svaki spoj na `main` automatski, produkcija samo označeno izdanje (`v0.x.y`) uz
obaveznog odobravatelja Leona u GitHubu (F0-S4), pa je ADR-005 ugrađen u alat, a ne u pamćenje.

## 6.3 Uvjeti za Postgres na Railwayu

1. **Aplikacijska uloga bez `UPDATE/DELETE/TRUNCATE` nad `events`** (F0-7); migracije i održavanje rade zasebnom ulogom.
2. **Migracije radi `sqlx`** (`server migrate`); jedan izvor istine u `crates/store/migrations` (od `0005` platforma).
3. **`sqlx` s TLS značajkom `tls-rustls`** (F0-6); nikad openssl (`deny.toml`).
4. Spoj preko privatne mreže Railwaya; `DATABASE_URL` iz okoline, pa je selidba na drugog dobavljača jedan `pg_dump`/`pg_restore`
   i promjena varijable (ADR-004).
5. Bez Data API-ja ni RLS logike za igru: sva logika i dnevnik ostaju u Rustu ([ARHITEKTURA §18](../ARHITEKTURA/18-odbaceno-otvoreno-redoslijed.md)).

## 6.4 Što mora biti gotovo prije izlaska na internet (s dokazom iz koda)

| stavka | danas | promjena |
|---|---|---|
| adresa slušanja | `127.0.0.1:8080` (`main.rs:144`) | `0.0.0.0:$PORT` iz okoline (F0-1) |
| postavke | sirove `env::var` na više mjesta | jedna tipizirana `Config`, provjerena pri startu (F0-1) |
| sadržaj | put iz vremena prevođenja (`content/src/lib.rs:74-76`) | `content/` kopiran u sliku + `HOTELIXSIM_CONTENT`; pri startu se provjerava kao i danas (`main.rs:35-49`) |
| gašenje | samo Ctrl-C, a `?` preskače `worker.abort()` | SIGTERM + dovršavanje poslova u tijeku (F0-2) |
| radnik reda | jedan, serijski (`main.rs:123-136`) | N paralelnih radnika + trake: razgovor s likom (brzo), presude, obračun; `SKIP LOCKED` to već podržava (F0-8, pilot) |
| kolačić | bez `Secure` (`auth.rs:31`) | `Secure` u produkciji, `SameSite=Lax`, provjera `Origin` na POST-u (F0-3) |
| ograničenja i zaglavlja | nema | ograničenje zahtjeva po IP-u i sesiji (`tower_governor`), najveća veličina tijela, CSP, HSTS, `nosniff` (F0-3) |
| zdravlje | `COUNT(*)` nad `events` | `SELECT 1` + verzija i otisak pravila (F0-5) |
| migracije | ručna naredba `migrate` ([ADR-015](../../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 12) | zasebna, kao korak prije nove verzije; samo unaprijed („proširi pa suzi"); nikad ne prepisuju `events` |
| slika | nema Dockerfilea | višefazni Dockerfile (cargo-chef, pinirani Rust 1.98.1, `SQLX_OFFLINE=true`), slika označena git SHA-om i **čuvana** (stara sezona se vrti svojom verzijom, ARHITEKTURA §17) (F0-11) |
| kopije | nema | `pg_dump` lokalno (F0-13), pa noćni u EU spremnik i proba vraćanja (F0-S5, G-7) |
| nadzor | `tracing` u konzolu | JSON zapisi, Sentry (EU), upozorenje na posao `failed`, red koji raste i AI trošak blizu granice; vanjski ping na `/v1/health` (F0-14) |
| tajne | `.env` lokalno | varijable okoline kod hostinga; AI ključ samo na poslužitelju; repo je javan, pa nikad ništa u gitu |
| AI trošak | tokeni u `AiMeta`, bez granice | projekcija `ai_usage` + granice iz 2.6 (F0-9) |
| više igara po korisniku | `/v1/me/hotel` uzima **zadnju** igru (`store/src/guests.rs` `latest_hotel_of`) | `me/classes`: odabir razreda ili vježbe (F1-13) |

## 6.5 CI/CD

**Danas (`.github/workflows/ci.yml`):** Rust fmt, clippy `-D warnings`, brana čistoće jezgre, probe mutacija, provjera sqlx
upita nad pravim Postgresom 16, svi testovi, tri prolaza balansa (v1, v2 tjedan i mjesec, 1000 sjemena), cargo-deny; web:
provjere dokumentacije, usklađenost OpenAPI ↔ klijent, lint, tipovi, Vitest, izgradnja.

**Dodati:** izgradnja Docker slike na svakom PR-u (bez objave, F0-11); e2e Playwright s pravom bazom; test migracija na
svježoj bazi **i** na kopiji sheme produkcije; akcije pinirane po SHA-u; `pnpm audit`; tok isporuke `main` → staging,
tag → produkcija uz Leonovo odobrenje (F0-S4); cache za `sqlx-cli` (F0-12).

## 6.6 Opterećenje: razred od ~30 studenata u jednoj igri

| dio | procjena [PROC] | što napraviti |
|---|---|---|
| HTTP | ~0,2–0,5 zahtjeva u sekundi po aktivnom studentu → 20–45 u sekundi na vrhu | jedan proces s 1–2 vCPU vjerojatno dovoljan; **izmjeriti** alatom (oha/k6) (F0-15) |
| fold pri svakom čitanju | tjedna v2 igra ima 17 krugova, a ovdje piše 30 studenata u isti dnevnik | predmemorija složene igre po `(game_id, zadnji stream_seq)` u `app` (F0-10) + projekcija `hotel_progress` za profesora |
| istodobno zatvaranje krugova | obračun je po hotelu, pa vrhovi su blaži nego kod „svi odjednom", ali na početku sata svi zatvaraju zajedno; svaki može čekati presude do 120 s ([ADR-019](../../records/DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) t. 5) | paralelni radnici; obračun ne smije čekati iza razgovora |
| AI | razgovori s likom moraju biti brzi; presude su sporije | traka za razgovore s prioritetom, ograničenje istodobnih poziva po modelu prema razini računa kod Anthropica [PROVJERITI] |
| baza | malo pisanja; zaključavanje po igri znači da 30 studenata piše redom | sqlx pool 10–20 veza [PROC]; **dokaz je proba 30 studenata u jednoj igri (G-8)**, ne procjena |
