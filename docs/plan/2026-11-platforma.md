# Platforma do P3 — studeni 2026. – siječanj 2027. — nacrt plana

**Status:** 🟨 NACRT (čeka Leona) · **Područje:** platforma · **Otvoren:** 2026-10-16 · **Grana:** cigle na kratkim granama
`platforma/<cigla>` iz integracijske `model-v2/sloj1` (nakon L1-15 iz `main`), radno stablo `../hotelixsim-<cigla>`

> **Što je ovo.** PLAN (redoslijed, sati, ovisnosti, tko što radi), ne arhitektura. Arhitektura je u
> [PLATFORMA.md](../architecture/PLATFORMA.md) (nacrt za raspravu), a sadržaj svake cigle (što se radi, dokaz) u
> [ROADMAP/platforma.md](./ROADMAP/platforma.md). Ovdje se cigle navode samo po oznaci, da činjenica živi na jednom mjestu.
> **Zašto sada:** Leon je 16.10. (sesija 7) prihvatio da **platforma kreće u studenom čak i ako faza C nije gotova**, jer P3
> ovisi o platformi više nego o predmemoriji (C4). To mijenja [ADR-043](../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)
> t. 2 („samo popravak dok nije gotov") za kolosijek PLATFORMA; ADR se piše na `/kraj`. Sati su [PROC].

## 1. Cilj P3

**Do sredine siječnja 2027. profesor na internetu sam vodi razred: studenti uđu kodom i igraju jedni protiv drugih,
a on otvara krugove i vidi tko je gdje i kako mu ide** ([ROADMAP](./ROADMAP.md): VERZIJA ZA PROFESORE; poželjno, ne nužno).

| profesor MORA moći u siječnju (minimum P3) | cigle |
|---|---|
| otvoriti adresu na internetu i prijaviti se (Safari, privatni prozor, mobitel) | SPREMNO ZA INTERNET, F0-S1, F1-4 |
| stvoriti razred, izdati studentima račune, pokazati kod na projektoru | F1-2, F1-3, ID-1…3, F1-5, F1-16 |
| pustiti studente kroz čekaonicu i pritisnuti „Pokreni" | F1-6, F1-8, F1-14 |
| otvarati krugove ručno ili po rasporedu; neaktivni hotel sam ide dalje | F1-11, RZ-1, RZ-2 |
| vidjeti razred uživo, otvoriti studenta (samo čitanje), račun dobiti svih hotela, presude za pregled | F2-1…F2-6 |
| znati da AI trošak ima granicu i da se podaci ne gube | F0-9, RZ-4, F0-13 |

**Smije čekati P4** (linije reza iz [ROADMAP-a](./ROADMAP.md) i [ROADMAP/platforma](./ROADMAP/platforma.md)): vlastite
situacije profesora (F2b, ~40 h), ispravak presude (F2-7), izvoz CSV (F2-10), uklanjanje studenta (F2-11), vlastita domena
(F0-S3), staging i tok isporuke (F0-S4), kopije izvan Railwaya (F0-S5), GDPR brisanje i rokovi (G-2…G-9), paralelni radnici
kao zasebna PILOT cigla (vidi C2 niže), proba 30 studenata (G-8), AAI@EduHr u kodu (F5).

## 2. Što platforma troši iz jezgre i igre

| ovisnost | gdje | preduvjet za P3? | što se gradi prije nje |
|---|---|---|---|
| **RAZRED U JEZGRI**: F1-7 N hotela, F1-9 vrata kruga, F1-10 obračun po hotelu, M-1, F1-18 ([ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md), [igra.md](./ROADMAP/igra.md)) | IGRA, ~45 h, nakon L1-15 | **DA**, kritični put | sve SPREMNO ZA INTERNET; računi i prijava (F1-2…4, ID-1…3), razred i kod (F1-5, F1-6), web kod i čekaonica (F1-14), AI granica (RZ-4), G-1. **Računi i prijava ne ovise o razredu u jezgri.** |
| **C1** stara igra se čita nakon promjene jezgre (REVIZIJA #6) | IGRA, faza C | **DA, 🟨** ako proba traje dulje od jednog sata: svaki deploy s novom jezgrom inače daje 409 cijelom razredu. Zakrpa: zamrznuti verziju za trajanje probe (cijena: nijedan popravak igre tijekom probe) | sve osim izlaska na internet s pravim razredom |
| **C2** više radnika poslova, zaseban red za lika (REVIZIJA #9) | IGRA, faza C; isto što i **F0-8** (PILOT) | **DA** za razred od 20–30: s jednim radnikom presude ne stignu u 120 s i postaju neutralne. Prijedlog 🟨: F0-8 se spaja u C2 i seli prije P3 | sve do F1-17 (e2e s tri studenta radi i s jednim radnikom) |
| **C4** predmemorija složene igre | IGRA, faza C; isto što i **F0-10** | **DA**, ali je ionako cigla platforme. Prijedlog 🟨: **C4 = F0-10, jedna cigla, kolosijek PLATFORMA** (dira `app`, ne jezgru); preduvjet za F1-10 | — |
| **C5** pravila iz weba u API (ocjena odjela, „Obvezna", kodovi problema) | IGRA, faza C | **NE** općenito; **DA** za F2-3/F2-5 ako profesorov ekran pokazuje ocjene odjela, inače se pravilo kopira treći put | F2-1, F2-2, F2-4, F2-6 |
| **C6** razdvojiti `api/src/schema.rs` (1770 redaka) i mentor (= K0-4, K0-5 iz [kvaliteta.md](./ROADMAP/kvaliteta.md)) | IGRA/kvaliteta | **DA**: preduvjet da dva kolosijeka ne dijele datoteku | F0-1…F0-6, F0-13 (diraju `server` i `store`, ne `api/schema.rs`) |
| **C3** `presuda.v4`, mjerilo AI-ja | IGRA, faza C | NE (F2-6 čita presude kakve jesu) | sve |
| **P9-1** račun dobiti za profesora (izvor brojki) | IGRA, ~6 h, nakon L1-15 | DA za F2-3 | F2-1, F2-2 |
| **L2a-12** vidljivi tuđi radnici | IGRA, LJUDI | NE: F1-12 se dijeli na **F1-12a** (ljestvica, ocjene, cijene; bez radnika) za P3 i **F1-12b** (radnici kolega) kad LJUDI sleti | F1-12a |

**Posljedica za kolosijek IGRA (prijedlog 🟨, Leon bira u anketi o redoslijedu):** nakon faze B prvo **C6, C1, C2**, pa
**RAZRED U JEZGRI**, a tek onda C3, C5 i LJUDI. RAZRED U JEZGRI treba stići do **~7.12.**, inače platforma u prosincu stoji
na fazi 3. P2 iz ROADMAP-a (kraj studenog) time realno klizi na **sredinu prosinca** [PROC].

## 3. Faze i cigle platforme do P3

Uloge po [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md): **Č** čuvar testova (testovi prije koda),
**G** graditelj, **P** provjeravatelj, **P★** provjeravatelj na najjačem modelu (sigurnost, prijava, jezgra), **UI** UI čuvar
na kraju faze; čuvar dokumentacije je u svakoj cigli i nije pisan. „Leon" = treba li njegova odluka prije cigle.
„Vidi se": **lok** = lokalno (nakon F0-4 jedna adresa `http://localhost:8080`, prije toga <http://localhost:5173>),
**RW** = Railway adresa `*.up.railway.app` (tek uz Leonov OK), **test** = samo u provjerama.

### Faza 1 · SPREMNO ZA INTERNET (tj. 1–2: 2.–15.11.; 48 h; lokalno 0 €)

**Gotovo kad:** Leon na jednoj lokalnoj adresi odigra igru u privatnom prozoru i Safariju; ugašena baza daje 503 u 3 s;
slika se gradi; AI iznad granice ide na rezervu. Izlazak na Railway (F0-S1) samo uz izričit OK.

| red | cigla | h | ovisi o | uloge | Leon | vidi se |
|---|---|---|---|---|---|---|
| 1 | F0-1 tipizirana `Config` (dio postoji: `HOTELIXSIM_ADDR`, `HOTELIXSIM_CONTENT` u `crates/server/src/main.rs:36,144`; nedostaje `$PORT` i jedna struktura) | 2–3 | — | Č G P | ne | test |
| 2 | F0-2 uredno gašenje | 2 | F0-1 | Č G P | ne | test |
| 3 | F0-5 zdravlje (`crates/api/src/health.rs`) | 2 | F0-1 | Č G P | ne | lok `/v1/health` |
| 4 | F0-6 sqlx TLS i brane (`deny.toml`) | 3 | — | Č G P | ne | test |
| 5 | F0-4 poslužitelj servira web | 3 | F0-1 | Č G P | ne | lok :8080 |
| 6 | F0-3 HTTP zaštita (`crates/api/src/auth.rs:31`) | 5 | F0-1 | Č G **P★** | ne | test |
| 7 | F0-10 = C4 predmemorija složene igre (`crates/app`) | 6 | — | Č G **P★** | ne | test |
| 8 | F0-9 `ai_usage` + granica (iz `AiMeta`, `crates/game/src/event.rs:183`) | 6 | — | Č G P★ | ne (2 $ po studentu odlučeno) | test |
| 9 | F0-13 kopije baze (`scripts/backup.sh`) | 3 | — | Č G P | ne | lok |
| 10 | F0-11 Dockerfile + slika u CI-ju | 5 | F0-1, F0-4 | Č G P | ne | test (CI tek nakon pusha) |
| 11 | F0-14 zapisi i Sentry | 3 | F0-1 | Č G P | **da**: besplatni Sentry račun (EU) | Sentry |
| 12 | F0-S1 Railway Hobby | 4 | 1–6, 10 | G P★ | **da**: izričit OK + ~5–10 €/mj | RW |
| 13 | F0-S2 mjerenje kašnjenja | 3 | F0-S1 | G P | ne | zapis brojki |

Redovi 12–13 rade se kad Leon da OK (prijedlog: početkom prosinca), ne nužno u tjednu 2. Redovi 1–6 i 9 diraju `server`, `store` i `api/health.rs`/`auth.rs`, pa mogu prije C6. Red 7 dira `crates/app`: ne
paralelno s C1 ni s RAZREDOM U JEZGRI. Kraj faze: UI (privatni prozor, Safari, mobitel), kopija.

### Faza 2 · RAZRED bez jezgre (tj. 3–5: 16.11.–6.12.; ~57 h)

**Gotovo kad:** profesor dobije jednokratnu poveznicu, postavi lozinku, stvori razred, unese popis studenata i dobije
korisnička imena; student se prijavi, promijeni lozinku, upiše kod i vidi „čekamo profesora (N prijavljenih)".
Igra se još ne pokreće (to traži F1-7). Kod ide u nove module (`crates/app/src/platform/`, `crates/api/src/platform/`,
migracije od `0005` u `crates/store/migrations`); DTO-i platforme ne idu u `schema.rs`.

| red | cigla | h | ovisi o | uloge | Leon | vidi se |
|---|---|---|---|---|---|---|
| 1 | F1-2 migracija `0005` (`users`, `identities`, `classes`…; `slot` 1..64 umjesto 1..4 u `0003_platform.sql`) | 4 | F0-6 | Č G **P★** | ne | test |
| 2 | F1-3 uloge i prava (`crates/api/src/demo.rs` `Caller`) | 6 | F1-2, **C6** | Č G **P★** | ne | test |
| 3 | ID-1 prijava korisničko ime + lozinka | 6 | F1-3 | Č G **P★** | ne | lok |
| 4 | F1-4 ulaz za profesora | 6 | F1-3 | Č G **P★** | **da: D4** (pitanje 2) | lok |
| 5 | F1-5 razred i kod igre | 7 | F1-3, F1-4 | Č G P | **da: D3** veličina (preporuka 40), D11 | lok |
| 6 | ID-2 generator računa | 6 | ID-1, F1-5 | Č G **P★** | **da**: oblik korisničkog imena (pitanje 1) | lok |
| 7 | F1-6 ulaz kodom igre | 4 | F1-5, ID-1 | Č G P | **da: D2** lobby | lok |
| 8 | ID-3 web: prijava i lozinka (`apps/web/src/screens/`) | 5 | ID-2 | Č G P UI | ne | lok, mobitel |
| 9 | F1-14 web: kod i čekaonica | 6 | F1-6, ID-3 | Č G P UI | ne | lok, mobitel |
| 10 | RZ-4 AI granica po igri | 4 | F0-9 | Č G P | **da**: iznos | test |
| 11 | G-1 obavijest o privatnosti | 3 | F1-14 | G P | **da**: tekst | lok |

Redovi 8, 9 i 11 su samo web i ne broje se u dva Rust agenta: idu kad igra vrti balans.

### Faza 3 · RAZRED s jezgrom (tj. 6–8: 7.–20.12. i 4.–10.1.; 57 h)

**Gotovo kad:** uvjet faze RAZRED iz [ROADMAP/platforma](./ROADMAP/platforma.md): profesor pritisne „Pokreni", otvara krugove
ručno ili po rasporedu, student zatvara svoj hotel i vidi kolege, neaktivni hotel rok sam prenese; e2e s profesorom i tri
studenta zelen. **Počinje tek kad su F1-7, F1-9 i F1-10 spojeni** (IGRA).

| red | cigla | h | ovisi o | uloge | Leon | vidi se |
|---|---|---|---|---|---|---|
| 1 | F1-8 pokretanje iz čekaonice | 4 | F1-6, **F1-7** | Č G **P★** | ne (D2 iz faze 2) | lok |
| 2 | F1-13 odabir igre korisnika (`crates/store/src/guests.rs` `latest_hotel_of`) | 3 | F1-8 | Č G P | ne | lok |
| 3 | F1-11 app naredbe i API (`crates/app/src/commands.rs`, `api/src/platform/`) | 8 | F1-3, **F1-10** | Č G **P★** | ne | test |
| 4 | F1-12a vidljivost kolega bez radnika | 5 | F1-10 | Č G **P★** | **da: D8** imena ili nadimci | test |
| 5 | RZ-1 raspored otvaranja i rokovi (`0004_jobs_run_after.sql` postoji) | 5 | F1-11 | Č G P | ne (potvrđeno 13.10.) | lok |
| 6 | RZ-2 pravilo za neaktivne | 4 | RZ-1, **F1-9** | Č G P | ne (odlučeno 13.10.) | test |
| 7 | F1-16 web: profesor | 8 | F1-11 | Č G P UI | ne | lok, računalo + mobitel |
| 8 | F1-15 web: student u razredu | 8 | F1-11, F1-12a | Č G P UI | ne | lok, mobitel |
| 9 | RZ-3 obavijesti u aplikaciji | 6 | F1-15 | Č G P | ne (odlučeno 13.10.) | lok |
| 10 | F1-17 e2e: razred od tri studenta | 6 | F1-14…16 | Č G P | ne | test |

F1-12b (radnici kolega, 3 h) ide kad LJUDI spoji L2a-12. Kraj faze: UI, **Leon igra kao profesor i tri studenta** (tri
privatna prozora), kopija.

### Faza 4 · PROFESOR (tj. 8–9: 4.–15.1.; 43 h; F2b 40 h na liniji reza)

**Gotovo kad:** profesor vidi razred uživo (krug, napredak, tko čeka, AI trošak), otvara studenta samo za čitanje, vidi
račun dobiti svih hotela i presude za pregled. Na RW, uz Leonov OK, proba s jednim stvarnim profesorom.

| red | cigla | h | ovisi o | uloge | Leon | vidi se |
|---|---|---|---|---|---|---|
| 1 | F2-1 projekcija `hotel_progress` (`crates/app/src/ports.rs`) | 7 | F1-10, F1-2 | Č G **P★** | ne | test |
| 2 | F2-2 API pregleda razreda (ETag) | 6 | F2-1 | Č G P★ | ne | test |
| 3 | F2-3 profesorov račun dobiti | 6 | F2-2, **P9-1**, C5 🟨 | Č G P | ne | lok |
| 4 | F2-4 detalj studenta (samo čitanje) | 8 | F2-2 | Č G P UI | ne | lok |
| 5 | F2-5 web: ekran razreda | 10 | F2-2, F1-16 | Č G P UI | ne | lok, projektor |
| 6 | F2-6 pregled presuda | 6 | F2-2 | Č G P | ne | lok |
| 7 | F2b-1…4, 7, 9 vlastite situacije | 40 | s1-sit spojen, F1-8 | Č G P★ UI | **da: D10** (pitanje 6) | lok |

### Zbroj do P3 [PROC]

| faza | h | tjedni (2.11.–15.1.) |
|---|---|---|
| 1 SPREMNO ZA INTERNET | 48 | 1–2 |
| 2 RAZRED bez jezgre | 57 (54 + G-1 3) | 3–5 |
| 3 RAZRED s jezgrom | 57 (+ F1-12b 3 poslije LJUDI) | 6–8 |
| 4 PROFESOR | 43 (+ F2b 40) | 8–9 |
| **ukupno** | **205** bez F2b · **245** s F2b | 9 radnih tjedana (2.11.–20.12. i 4.–15.1.; blagdani 21.12.–3.1. [PROC]) → **~23–27 h platforme tjedno** |

AAI-0 (2 h) je gotov 15.10. ([PLATFORMA §3.3](../architecture/PLATFORMA/03-prijava.md)), pa nije u zbroju. Uz to IGRA za P3
troši RAZRED U JEZGRI ~45 h, P9-1 ~6 h te C1, C2, C6 (kartice još nisu pisane; C ukupno ~50–90 h).

## 4. Usporedo s fazom C

- **Dva kolosijeka, najviše dva Rust agenta ukupno** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 7):
  prijedlog je **po jedan Rust agent na kolosijeku**; web i dokumentacija ne broje se. Kad igra vrti balans (1000 sjemena,
  tjedno i mjesečno), platforma radi web cigle.
- **Sudar u `crates/api` i `crates/app`.** Platforma prvo ide u `crates/server`, `crates/store` i nove module
  (`api/src/platform/`, `app/src/platform/`). **C6 (K0-4) je preduvjet** za svaku platformsku rutu (F1-3 nadalje): dok
  `schema.rs` nije razdvojen, ne dira ga nijedan kolosijek osim C6. `api/src/lib.rs` (ruter) mijenja se jednim retkom po
  modulu. `openapi.json` i tipovi weba regeneriraju se **samo nakon spajanja**, nikad na grani usporedo.
- **`crates/app/src/commands.rs`**: F1-11 tek kad je F1-10 spojen; F0-10 ne usporedo s C1 ni F1-7…10.
- **Migracije:** platforma `0005` nadalje; igra (C1, C2) traži broj od voditelja pri spajanju. Dvije grane nikad ne spajaju
  isti broj 🟨.
- **Radna stabla:** svako svoj `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>` i svoja baza (BUGS B9, B12); stablo se
  briše isti dan nakon spajanja; disk iznad 100 GB; gradnja samo na punjaču.
- **Grane:** cigle platforme granaju s `model-v2/sloj1` i spajaju se u nju isti dan (jedna integracijska grana, ADR-043 t. 5);
  nakon L1-15 obje strane idu iz `main`. Svaka cigla vrti cijeli popis provjera iz CI-ja.
- **Tjedni ritam (prijedlog, Leon bira; pitanje 4):** A) istodobno, svaki kolosijek po jedan Rust agent, kraj tjedna
  spajanje i Leonov pregled (preporuka); B) dani: ponedjeljak–srijeda IGRA, četvrtak–subota PLATFORMA.

## 5. Hosting i trošak za P3

P3 su **profesori na internetu, ne pilot**. Najjeftinija varijanta koja čuva temelj ([PLATFORMA §6](../architecture/PLATFORMA/06-produkcija-i-trosak.md)):

| stavka | za P3 | €/mj [PROC] |
|---|---|---|
| poslužitelj + Postgres | **Railway Hobby**, isti projekt, ista regija, Rust servira web i `/v1`, adresa `*.up.railway.app` | **~5–10** (5 $ s uključenih 5 $ potrošnje) |
| domena | ne treba (F0-S3 je P4) | 0 |
| nadzor | Sentry besplatna kvota | 0 |
| kopije | `pg_dump` s Railwaya na Leonovo računalo, tjedno (F0-13); EU spremnik je P4 | 0 |
| CI | GitHub Actions (javan repo) | 0 |
| **ukupno novo** | od izlaska (prijedlog: početak prosinca, interno) do 15.1. ≈ 1,5 mj | **~10–15 € ukupno** |

**Ostaje lokalno:** razvoj, balans, sva radna stabla, baza za razvoj; produkcijska baza nikad ne koristi lokalnu lozinku
iz povijesti repoa. Supabase se ne koristi (AAI@EduHr nudi OIDC, posrednik ne treba, §3.3).
**ADR-004** („Railway tek za pilot") treba dopunu: izlazak za P3, prije pilota, uz izričit OK 🟨.

**AI trošak po razredu [PROC]:** ~1–2 $ po studentu za cijelu sezonu (PLATFORMA §6.2). Proba P3 od 4–6 tjednih krugova
(~25–35 % sezone) daje ~0,3–0,7 $ po studentu, dakle **~10–20 $ za razred od 30**; uz 2–3 probna razreda ~30–60 $.
Granica 2 $ po studentu (F0-9) i po igri (RZ-4) mora biti gotova prije prvog stvarnog razreda.
**Kako izmjeriti umjesto procjene:** Leon odigra lokalno jednu tjednu v2 sezonu sa situacijama i razgovorima, zatim
```sql
SELECT payload->'meta'->>'model' AS model,
       SUM((payload->'meta'->>'input_tokens')::bigint)  AS ulaz,
       SUM((payload->'meta'->>'output_tokens')::bigint) AS izlaz
FROM events
WHERE game_id = $1
  AND type IN ('CharacterReplied','VerdictRecorded','DebriefRecorded','SeasonDebriefRecorded')
  AND jsonb_typeof(payload->'meta') = 'object'
GROUP BY 1;
```
i pomnoži cijenama iz [ADR-024](../records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md) (po milijunu tokena,
ulaz/izlaz: Sonnet 2/10 $, Haiku 0,10/0,50 $). To je ujedno test za F0-9 (`ai_usage` je ista projekcija).

## 6. Rizici i linije reza

| rizik | ublažavanje |
|---|---|
| RAZRED U JEZGRI kasni (igra još u B ili C) | faza 2 ima ~57 h posla bez jezgre; IGRA radi C6, C1, C2 i RAZRED U JEZGRI prije C3, C5 i LJUDI (pitanje 5) |
| sudar u `api`/`app` | C6 prvi; DTO-i u `platform/`; `openapi.json` samo nakon spajanja; F1-11 nakon F1-10 |
| stara igra 409 usred probe | C1 prije P3; ako C1 kasni, verzija se zamrzava za trajanje probe |
| presude ne stignu za 30 studenata | C2 (s F0-8) prije P3; F0-S2 mjeri |
| AI račun bez granice | F0-9 i RZ-4 prije prvog stvarnog razreda; izlazak bez njih se ne radi |
| stvarni studenti i GDPR prije G-3 | korisnička imena bez pravih imena (profesor drži popis kod sebe) 🟨, G-1 obavijest, podaci iz AAI nikad u AI ni Sentry |
| Leonov tjedni kapacitet nije poznat | zbroj ~23–27 h/tj samo za platformu; ako je manje, linije reza niže |
| računalo (dva Rust agenta, disk, baterija) | po jedan Rust agent na kolosijeku; web cigle uz balans; gradnja samo na punjaču |

**Linije reza, redom** (ako tempo zapne): 1) F2b vlastite situacije → P4 (−40 h; već u ROADMAP-u); 2) RZ-3 obavijesti →
P4, profesor najavljuje krug usmeno (−6 h); 3) F1-12b radnici kolega čekaju LJUDE (već u planu); 4) RZ-1 automatski raspored
→ P4 ako se P3 proba igra samo na satu (−5 h; ADR-037 t. 6 traži ga za semestar, ne za probu). **Nikad se ne režu:**
testovi, provjeravatelj, balans, a prije izlaska na internet ni F0-3 zaštita, F0-9 granica AI-ja ni F0-13 kopije.

## 7. Pitanja za Leona

1. **Prijava za P3:** (a) korisničko ime + lozinka koje izdaje profesor, imena bez pravog imena i prezimena (preporuka:
   0 €, ne ovisi o fakultetu; zahtjev FMTU-u za AAI@EduHr poslati odmah jer traje 1–3 mjeseca, kod ~3–5 dana kad stigne
   Lab); (b) AAI@EduHr već za P3 (najbolje iskustvo, ali P3 tada ovisi o potpisu fakulteta).
2. **D4, profesor za probu:** (a) jednokratna poveznica koju izdaje admin, vrijedi 24 h (preporuka, F1-4, 0 €); (b) Leon
   ručno postavi račun i lozinku i pošalje ih profesoru (brže za 2–3 profesora, ali lozinka putuje porukom).
3. **Railway za P3:** (a) da, Hobby, interni izlazak početkom prosinca nakon faze 1 (~5–10 €/mj, ~10–15 € do 15.1.)
   (preporuka: mjesec probe na pravom internetu prije profesora); (b) da, ali tek u siječnju (~5 €, manje vremena za
   kvarove koje lokalno ne vidimo); (c) ne, P3 lokalno na Leonovom računalu (0 €, profesor ne može sam).
4. **Ritam:** (a) istodobno, po jedan Rust agent na kolosijeku, pregled na kraju tjedna (preporuka); (b) dani u tjednu,
   npr. ponedjeljak–srijeda IGRA, četvrtak–subota PLATFORMA.
5. **Redoslijed IGRE od studenog:** (a) nakon faze B prvo C6, C1, C2 i RAZRED U JEZGRI, pa C3, C5, LJUDI; C4 prelazi u
   platformu kao F0-10 (preporuka: P3 inače stoji na fazi 3); (b) cijela faza C, pa RAZRED U JEZGRI (P3 klizi u veljaču).
6. **Vlastite situacije profesora (F2b):** (a) odmah planirati za P4 (preporuka: −40 h, P3 i bez njih pokazuje razred);
   (b) ostaviti u P3 i rezati tek ako tempo zapne.
