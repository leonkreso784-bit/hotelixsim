# ARHITEKTURA §9–§12 — podaci, AI sloj, API, prijava i prava

Dio [ARHITEKTURA.md](../ARHITEKTURA.md)

## 9 · Podaci i pohrana

### 9.1 Dnevnik događaja (izvor istine)

```text
events
  id            bigserial        globalni redni broj
  game_id       uuid
  stream        text             za sada jedan tok po igri: game:<id> (hotel:<id> je moguća kasnija podjela)
  stream_seq    int              redni broj unutar toka (jedinstven par stream + stream_seq)
  type          text             npr. ResponseSubmitted, VerdictRecorded, RoundResolved
  schema_ver    int              verzija oblika događaja
  payload       jsonb
  actor         text             guest:<uuid> · robot:<strategija> · system (CHECK u bazi)
  recorded_at   timestamptz      samo informativno — jezgra ga ne čita
```

- **Samo dopisivanje:** korisnik baze kojim radi aplikacija **nema prava** `UPDATE` ni `DELETE` nad tablicom.
  To jamči baza, ne kod. Za sada nepromjenjivost čuva okidač; uloga bez prava dolazi prije Railwaya.
- **Verzije događaja:** kad se oblik promijeni, stari se događaji pri čitanju „podignu" na novu verziju; nikad se ne
  prepisuju.

### 9.2 Projekcije (izvedeno, može se ponovno izgraditi)

Stanje hotela po rundi, situacije za prikaz, ljestvica, profesorov uvid. Ako se pokvare ili promijene, brišu se i
grade ponovno iz dnevnika.

### 9.3 Platformske tablice (obične)

Izgrađene su: `guests` · `games` (postavke, sjeme, verzija pravila) · `hotels` · `jobs` (red poslova) · `events` (§9.1).
Planirane: `organizations` · `users` · `memberships` (uloga u organizaciji) · `managers` (veza upravitelj ↔ korisnici;
danas jedan) · `ai_usage` (trošak).

### 9.4 Red poslova

Red je u Postgresu jer se **odgovor i posao upisuju u istoj transakciji**, pa stanje „odgovor spremljen, posao ne"
ne može nastati. Pouzdanost je pravi posao, ne samo tablica:

- preuzimanje s `FOR UPDATE SKIP LOCKED` i **vremenskim zakupom**: posao koji radnik ne završi u roku vraća se u red;
- ključ jedinstvenosti po poslu (npr. `resolve:<game>:<round>`), ograničen broj ponovnih pokušaja s **odgodom i povećanjem razmaka** (`run_after`: 2, 4, 8, 16 s; korak 4, ADR-015 t. 8);
- **prije upisa rezultata provjerava se stanje runde** (korak 4, uz AI): zakašnjela AI presuda za već zatvorenu rundu ne upisuje se
  tiho, nego ide pravilom iz §8.3.

**Izbor izvedbe** (vlastita tablica, apalis ili pgmq) radi se **probnim testom** s ovim scenarijima: pad procesa
usred posla, zaglavljen posao, zakašnjeli AI rezultat i dvostruko preuzimanje. Ne bira se po mišljenju.

Za demo izabrana vlastita tablica `jobs` (ADR-015); probni test s apalis/pgmq ostaje u BACKLOG-u. U koraku 4 izgrađeno: odgoda posla, obračun runde čeka AI presude (do roka `VERDICT_DEADLINE_SECS` = 120 s, potom neutralna presuda s `review: true`), a presuda koja stigne nakon obračuna zapisuje se kao `VerdictLate` bez izmjene obračuna. Provjereno kraj-do-kraja na pravom Postgresu (`crates/api/tests/e2e.rs`).

### 9.5 Sadržaj

- Danas: JSON datoteke u `content/hotel/`, provjerene JSON Schemom (generirana iz Rust tipova, pa VS Code
  podcrtava greške dok se piše). Učitavaju se pri pokretanju; otisak sadržaja ulazi u `RulesetId`.
- Kasnije: profesorovi predlošci u bazi, kroz **istu** provjeru.

### 9.6 Migracije i kopije

SQLx migracije u repozitoriju. Na Railwayu (kad dođe): dnevne kopije baze + periodični izvoz dnevnika (on je izvor
istine).

---

## 10 · AI sloj

### 10.1 Četiri uloge, jedan port

```rust
trait AiPort {
    async fn generate_situation(&self, req: SituationRequest) -> Result<SituationDraft>;
    async fn assess_response(&self, req: AssessRequest) -> Result<Verdict>;
    async fn play_character(&self, req: CharacterTurn) -> Result<CharacterReply>;
    async fn debrief(&self, req: DebriefRequest) -> Result<DebriefText>;
}
```

### 10.2 Pravila

- **Strukturirani izlaz:** za situaciju i presudu AI dobiva JSON Schemu (iz `schemars`) i mora vratiti točno taj
  oblik. Izlaz se **uvijek** provjerava; AI-ju se ne vjeruje.
- **Mehanika samo iz kataloga:** shema dopušta samo mehanike koje su dopuštene u toj situaciji (popis vrijednosti).
- **Studentov tekst je podatak, ne uputa:** odvojen je u uputi kao citat; čak i kad AI „posluša" studenta, jezgra
  primijeni granice (K2).
- **Upute su datoteke** u `prompts/`, verzionirane; uz svaki AI izlaz zapisuje se verzija upute i model.
- **Pružatelji:** tanki HTTP klijenti (Anthropic, OpenAI, Google). Postavka bira pružatelja i model **po ulozi**
  (npr. jeftiniji model za situacije, jači za presude).
- **Trošak:** svaki poziv bilježi tokene i cijenu u `ai_usage`; tvrdi limit po igri i po studentu; preko limita →
  rezerve iz §8.4.
- **Nikad dvaput:** zapisani AI izlaz se ponovno koristi pri svakom ponovnom obračunu.

### 10.3 Provjera kvalitete AI-ja

Pravednost AI-ja je zaseban problem od pravednosti profila (§7.8):

- **Rubrika:** svaka situacija ima rubriku s kriterijima za ocjene 1–5. Ponuđeni odgovori ocijenjeni su po **istoj**
  rubrici, pa su slobodan tekst i opcije na istoj ljestvici.
- **Obrazložena ocjena:** uz ocjenu AI mora navesti koje kriterije rubrike odgovor ispunjava. Presuda bez toga je
  nevaljana (§8.3, korak 3).
- **Ljudsko mjerilo:** postoji skup odgovora koje su ocijenili ljudi, s očekivanom mehanikom i ocjenom. Model ide u
  igru tek kad se s njima slaže iznad praga. Skup se pokreće pri svakoj promjeni upute ili modela i uspoređuje s
  prethodnim rezultatom.
- **Zaključano za sezonu:** model i upute ulaze u snimak pravila (§7.2) i ne mijenjaju se usred sezone.
- **Prigovor:** student može uložiti prigovor profesoru. Ispravka ide pravilom iz §8.3 (korak 6).

---

## 11 · API

- **REST + JSON**, verzija u putanji (`/v1`).
- Opis: **OpenAPI** iz utoipa → `openapi.json` u repozitoriju → generirani TypeScript klijent. CI pada ako opis
  i generirani klijent nisu usklađeni.
- **Greške:** standardni oblik `application/problem+json` (RFC 9457) s kodom koji sučelje prevodi.
- **Uživo:** Server-Sent Events za „stigla je situacija" i „runda je obračunata". Na mobitelu kasnije push obavijesti.
- **Idempotentnost:** zaglavlje `Idempotency-Key` na svim naredbama koje mijenjaju stanje.

Okvirni resursi:

| | putanja |
|---|---|
| student | `GET /v1/me/hotel` · `GET /v1/hotels/{id}/situations` · `POST /v1/situations/{id}/responses` · `POST /v1/hotels/{id}/actions` · `POST /v1/situations/{id}/character` · `GET /v1/games/{id}/leaderboard` |
| profesor | `POST /v1/games` · `PATCH /v1/games/{id}/settings` · `POST /v1/games/{id}/interventions` · `GET /v1/hotels/{id}/audit` · `POST /v1/verdicts/{id}/override` |
| sustav | `GET /v1/health` · `GET /v1/events/stream` (SSE) |

---

## 12 · Prijava i prava

- **Identitet:** vanjski pružatelj prijave po standardu **OIDC**; naš backend samo provjerava token (JWT, ključevi
  preko JWKS). Prijavu ne pišemo sami.
- **Kandidati (otvoreno):** Supabase Auth (besplatan start, biblioteke za mobitel) · Clerk (najudobniji, plaća se
  iznad besplatne razine) · WorkOS (najjači za fakultetski SSO/SAML) · vlastiti Keycloak/Zitadel (najneovisnije,
  najviše održavanja).
- **Zahtjev za budućnost:** **AAI@EduHr** (SAML) — izabrani pružatelj ga mora moći podržati.
- **Prava** su u našem backendu, u sloju `app`: uloga u organizaciji (student / profesor / administrator) i
  pripadnost igri ili hotelu. Svaki slučaj upotrebe provjerava pravo prije rada; za svaku ulogu postoje testovi.
  Baza ne nosi poslovna prava (bez RLS-a) — jedno mjesto istine.
- **Što student vidi o drugima (Leon, 04.10.):** ukupnu ocjenu i pet indeksa kroz runde. Tuđe stvarne brojke,
  odluke, tekstovi i tragovi su privatni. Vrijedi dok profesori ne odgovore (pitanje 9, i imena ili anonimno).
  Pravilo se provodi u sloju `app` i ima test po ulozi: API nikad ne vraća tuđe privatne podatke.

---
