# PLATFORMA §7 · GDPR i sigurnost

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Pravni odgovori fakulteta traju tjednima, pa se pitanja (dio 08) šalju odmah (G-6).
Brisanje u zajedničkoj igri: [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md), posljedice.

## 7.1 Koji osobni podaci

| podatak | tko | gdje | nužno? |
|---|---|---|---|
| ime ili nadimak | student | `users` | da (profesor mora znati tko je tko); što vide kolege je odluka D8 |
| korisničko ime i otisak lozinke | student, profesor | `users` | da, za prijavu (ADR-037 t. 7); lozinka samo kao otisak |
| e-adresa | profesor; student tek uz AAI | `users`, `identities` | profesor da; student u prvom valu ne |
| AAI pseudonimni ID, ustanova, povezanost | kasnije | `identities` | da, uz AAI |
| odluke, odgovori, **slobodni tekst i AI razgovori** | student | `events` (pod pseudonimom `guest:<uuid>`) | da, za igru, ocjenu i prigovor |
| rezultat (napredak) | student | projekcije, izvoz | da, ako ulazi u ocjenu kolegija |
| IP, preglednik | svi | zapisi, ograničenje zahtjeva | kratko (npr. 14 dana [PROC], G-9) |
| kolačić sesije | svi | `sessions` (otisak) | da, tehnički nužan (bez bannera za pristanak) |

## 7.2 Pravna osnova i uloge (za fakultet i pravnika)

- **Voditelj obrade je vjerojatno fakultet** (FMTU ili Sveučilište u Rijeci), jer odlučuje da se igra koristi u
  nastavi. **Leon, odnosno njegova buduća tvrtka, je izvršitelj obrade** uz ugovor o obradi (čl. 28. GDPR-a).
  Podizvršitelji su hosting (Railway), Sentry i Anthropic, svaki s vlastitim DPA-om; uz AAI i posrednik prijave. Ovo je
  **pitanje za fakultet i njegovog službenika za zaštitu podataka** (F2).
- **Osnova:** ako igra ulazi u ocjenu, to je vjerojatno „javni interes / zadaća ustanove" (čl. 6.1.e). Pristanak je
  tada slab, jer student nije slobodan reći „ne" profesoru. Ako je pilot dobrovoljan, može biti pristanak uz
  alternativu bez igre. Odlučuje fakultet (F3).
- **Maloljetnici:** ne. Uvjeti korištenja traže studenta upisanog na visoko učilište. Ako bi itko bio mlađi od 16
  (hrvatska dob digitalnog pristanka), treba roditeljski pristanak, pa takve slučajeve isključujemo.
- **Obavijest o privatnosti** na ekranu pridruživanja, kratka i na hrvatskom: što spremamo, zašto, koliko dugo, tko je
  AI pružatelj i kome se obratiti (G-1).

## 7.3 AI razgovori

- Spremaju se jer bez njih nema prigovora ni profesorova pregleda presude
  ([ARHITEKTURA §10.3](../ARHITEKTURA/09-podaci-ai-api-prava.md)). To mora pisati u obavijesti.
- Na ekranu razgovora stoji kratko upozorenje: „ne upisuj osobne podatke, ovo je igra".
- **Prijenos izvan EU-a:** Anthropic API obrađuje podatke u SAD-u, uz DPA i standardne ugovorne klauzule
  [PROVJERITI uvjete i trajanje čuvanja]. Ako fakultet traži obradu u EU-u, Claude postoji i preko AWS Bedrocka i
  Google Vertexa u EU regijama [PROVJERITI dostupnost modela 5.5]. Kod nas je to samo nova izvedba `AiPort`
  (`ports.rs:165-177`), bez promjene igre (F5).
- Leonovi likovi sa stvarnim imenima (BACKLOG „Vizija") traže pisani pristanak tih osoba.

## 7.4 Brisanje i dnevnik u koji se ne smije pisati

Dnevnik je zaštićen okidačem koji odbija `DELETE` (`0002_events_append_only.sql`). To je dobro za ponovljivost, ali
GDPR traži brisanje. Rješenje:

- **imena i e-adrese nikad ne ulaze u dnevnik**, samo pseudonim `guest:<uuid>` (2.3);
- **brisanje studenta iz razredne igre nije brisanje igre**: u igri ostaju hoteli kolega. Zato je to **anonimizacija i
  redakcija**: briše se veza račun ↔ pseudonim (`class_players`), ime i identiteti, a slobodni tekst i AI razgovori tog
  hotela se redigiraju, uz zapis u `audit_log` (cigla G-3). Radi to zasebna **uloga održavanja** kroz jednu proceduru koja
  privremeno zaobilazi okidač samo za taj posao. Točan opseg redakcije traži **pravni odgovor fakulteta**;
- **cijela igra se briše** za vježbu i demo (jedan igrač): demo gosti nakon 7 dana (ADR-016 t. 1, G-5);
- **rok čuvanja razreda** (G-2, pitanje F4): npr. kraj akademske godine + rok za prigovor. Nakon toga posao reda briše razred i
  igru, a ostaje samo anonimna statistika (npr. raspodjela napretka) za kalibraciju;
- **izvoz vlastitih podataka** (pravo pristupa) je JSON izvoz studentova hotela, samo njegovi podaci (G-4).

## 7.5 Sigurnost (sažetak)

Prava samo u `app` + test po ulozi; profesor vidi samo svoje razrede (test „tuđi razred = 404"); student o kolegi vidi samo
dopušteno (test po polju); `audit_log` za profesorov i adminov pristup tuđim podacima; ograničenja zahtjeva i pokušaja
prijave i koda; kolačić `Secure` + `HttpOnly` + `SameSite=Lax`; AI ključ samo na poslužitelju; studentov tekst je podatak,
ne uputa ([ARHITEKTURA §10.2](../ARHITEKTURA/09-podaci-ai-api-prava.md)); cargo-deny i `pnpm audit`; drugi korak (TOTP)
za admina.
