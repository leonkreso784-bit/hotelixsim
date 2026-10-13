# ADR-015 — Dnevnik, runde i red poslova: arhitektonske pretpostavke koraka 3
**Datum:** 2026-10-05 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-012](ADR-012-ciljani-rok-demoa-14-10-prvo.md),
[ADR-013](ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md), ARHITEKTURA §6.2, §7.2, §8–§9, §11–§12 · **Plan:** [2026-10-05-dnevnik-runde.md](../../archive/2026-10-05-dnevnik-runde.md)

**Kontekst.** DEMO korak 3 pretvara jezgru u igru: gost, tri robota, runde koje zatvara gumb, obračun u redu poslova,
sve kao dnevnik događaja. Leon je 05.10. odlučio proizvodno (gost samo kao pozadina; zatvaranje s neodgovorenom
situacijom dopušteno uz ocjenu 1; zatvara svatko u igri; „kao BitLife": plan se mijenja bilo kad, feed hotela, svaka
promjena nosi uzrok). Arhitektura toga nije bila zapisana; ovo su pretpostavke s kojima plan gradi.
**Odluka (pretpostavke):**
1. **Novi čisti crate `game`** (ovisi o `engine`, `sim-hotel`, `content`, serde, uuid; nikad tokio/sqlx/axum — čuva
   `check-core-deps`): događaji, serde DTO-i, projekcija (fold), pravila runde, potezi robota, obračun runde. Jezgra
   ostaje bez serdea. **Snimak pravila** je stablo content DTO-a (`content::Snapshot`), a `ruleset_id` = SHA-256
   kanonskog JSON-a tipiziranog snimka (ne ovisi o redoslijedu ključeva koji JSONB mijenja). Cijena alternative (serde u
   jezgri): jezgra dobiva vanjsku ovisnost i gubi branu ADR-009.
2. **Neodgovorena situacija = kvaliteta 1 + mehanike iz novog polja `unanswered`** u `situations.json` (jezgra dobiva
   polje `SituationTemplate.unanswered` i `SituationOutcome::unanswered`). Odstupanje od „jezgra se ne dira": kvaliteta 1
   bez mehanike nema nikakav učinak, pa bi šutnja bila bolja od lošeg odgovora. Ishodi za S1–S4 su nacrt (S3: ponuda
   propada bez kazne) — Leonova provjera, kao i rubrike.
3. **Pet događaja** (`GameCreated` s punim snimkom, sjemenom, verzijom jezgre i hotelima · `PlanSet` · `AnswerChosen` ·
   `RoundClosed` · `RoundResolved` s punim izvještajem po hotelu), `schema_ver = 1`. Obračun uvijek iz snimka u
   `GameCreated`, nikad iz trenutnih datoteka. `actor` je tekst: `guest:<uuid>` · `robot:<strategija>` · `system`.
   **Pravilo oblika snimka:** otisak se danas računa iz TIPIZIRANOG snimka pri svakom foldu, pa promjena oblika
   content DTO-a (novo, preimenovano ili uklonjeno polje) čini sve postojeće igre nečitljivima (`Corrupt`). Do ispravka
   u koraku 4 vrijedi: nova polja DTO-a samo uz `#[serde(default)]`, a promjena oblika = nova verzija `GameCreated`
   s podizanjem. Ispravak (prije prve promjene content DTO-a): otisak računati nad kanonskim `serde_json::Value`
   spremljenog snimka, ne nad tipiziranim ponovnim zapisom (BACKLOG `[korak 4]`).
4. **Jedan tok po igri `game:<id>`, a svaka naredba je jedna transakcija**: `pg_advisory_xact_lock` po igri → učitaj tok
   → odluči (čista funkcija) → upiši. `UNIQUE(stream, stream_seq)` ostaje zadnja brana. Odstupanje od „optimistička
   konkurentnost s ponavljanjem": zaključavanje uklanja petlje ponavljanja i utrku između provjere i upisa; cijena je
   serijalizacija naredbi jedne igre, zanemariva za 4 hotela.
5. **Nepromjenjivost dnevnika jamči baza**: okidač odbija `UPDATE/DELETE/TRUNCATE` (SQLSTATE 42501), `CHECK` na oblik
   `actor`, indeks `(game_id, id)`. Uloga bez prava za Railway → BACKLOG.
6. **Platformske tablice** `guests` (u bazi samo SHA-256 tokena iz kolačića `httpOnly`), `games` (redoslijed `seq`),
   `hotels` (hotel → igra, gost → zadnja igra). Istina o igri ostaje u `events`.
7. **Roboti igraju istim naredbama**: njihovi `PlanSet`/`AnswerChosen` (actor `robot:<strategija>`) upisuju se u
   transakciji koja otvara rundu (uz `GameCreated` za lipanj, uz `RoundResolved` za srpanj); kocka
   `Rng::derive(sjeme, [svrha, mjesec, slot(, situacija)])`.
8. **Red poslova: vlastita tablica `jobs`** (jedinstveni `job_key` `resolve:<igra>:<mjesec>`, zakup 30 s,
   `FOR UPDATE SKIP LOCKED`, najviše 5 pokušaja, bez odgode). Gumb upisuje `RoundClosed` i posao u istoj transakciji
   (202); `RoundResolved`, potezi robota i „posao gotov" također u jednoj transakciji, idempotentno po stanju runde.
   Radnik je tokio zadatak u `serve`. Probni test apalis/pgmq, odgoda i zakašnjeli AI rezultat → korak 4.
9. **Projekcije se računaju foldom pri čitanju**, bez projekcijskih tablica (4 hotela × 2 mjeseca). Ponovljivost se
   provjerava tipizirano (bajt-isti `RoundResolved` nakon čitanja iz JSONB-a).
10. **Idempotencija prirodna po naredbi**: plan „zadnji vrijedi", odgovor jednom (409), zatvaranje već zatvorene runde
    vraća stanje (200 umjesto 202); **mjesec je u putanji** plana i zatvaranja, pa zastario ekran dobiva 409 umjesto
    tihog upisa u idući mjesec. Zaglavlje `Idempotency-Key` → BACKLOG.
11. **API ima vlastite DTO-e** (odvojene od DTO-a događaja): cijena je zrcaljenje tipova, dobitak je da se API mijenja
    bez nove verzije događaja. Kodovi grešaka i sve vrijednosti su enumi u OpenAPI-ju; kvaliteta i mehanike ponuđenih
    odgovora nikad ne izlaze. Rute po ARHITEKTURA §11 (`/v1/hotels/{id}/…`, `/v1/games/{id}/rounds/{mjesec}/close`).
12. **Server pri startu učitava i provjerava sadržaj** (`HOTELIXSIM_CONTENT`, zadano `content/hotel`) i ne kreće s
    pokvarenim katalogom; **migracije se pri startu ne pokreću** (`migrate` ostaje zasebna naredba, kasnije korak
    isporuke na Railwayu), jer automatska migracija s više procesa vodi u utrku.
13. **Profil: hoteli kreću s različitim profilima** (Leon 05.10., anketa): igrač dobiva profil iz sjemena, tri robota
    dobivaju A, B i C u rasporedu iz sjemena. Demo tako pokazuje bit ocjene — napredak od vlastitog početka, ne zarada.
    Imena: „Tvoj hotel", „Štediša", „Brižni", „Lovac na popunjenost".
14. **Pravo pristupa u `app`**: član igre = gost koji vodi hotel u njoj; vlasnik vidi stvarne brojke, plan, situacije i
    feed; član vidi tuđe samo kroz ocjenu i pet indeksa; ostali 403, bez kolačića 401. Test po ulozi.
**Posljedice.** Novi crate `game` i proširena brana čistoće; jezgra dobiva jedno polje i jednu funkciju; baza dobiva
dvije migracije. Ako Leon odbije t. 13, mijenja se jedna funkcija (`new_game`); ako odbije t. 2, mijenja se samo sadržaj
`unanswered`. Otvorena proizvodna pitanja (t. 2 i t. 13) su u [BACKLOG.md](../BACKLOG.md) → „Otvoreno za Leona".
