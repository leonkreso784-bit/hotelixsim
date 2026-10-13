# PLATFORMA §1 · Pojmovi i odgovor na pitanje „možemo li iz ovoga nastaviti"

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Odluke o razredu su u [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md).

## Pojmovi (svaki jednom)

- **Produkcija**: prava adresa na internetu na kojoj igraju stvarni studenti. **Staging**: ista postava, ali za probu
  prije produkcije, bez stvarnih studenata.
- **Prijava (autentikacija)**: tko si. **Prava (autorizacija)**: što smiješ. To su dva odvojena posla.
- **Sesija**: dokaz da si prijavljen. Kod nas je to nasumičan token u kolačiću `httpOnly`, koji JavaScript ne može
  pročitati. U bazi je samo njegov otisak (SHA-256).
- **Identitet**: način na koji je netko dokazao tko je (korisničko ime i lozinka, kasnije AAI@EduHr ili Google).
  Identitet je odvojen od korisnika, pa se jedan korisnik može kasnije prijaviti i drugim putem, uz istu povijest.
- **OIDC / SAML**: dva standarda za „prijavi se preko nekoga drugog" (Google, Microsoft, AAI@EduHr). OIDC je noviji i
  jednostavniji, SAML stariji, i njega koriste sveučilišne federacije.
- **CRUD**: stvori, čitaj, mijenjaj, briši. Ovdje znači profesorovo upravljanje kolegijima, razredima i igrama.
- **Razred**: jedna igra u kojoj igra N studenata, svaki sa svojim hotelom, jedni protiv drugih. Ulaze kodom igre
  ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) t. 1).
- **Vrata kruga**: profesor otvara krug; student igra otvoreni krug i zatvara ga kad je spreman; nitko ne ide iza
  otvorenog kruga (ADR-037 t. 2).
- **Projekcija**: tablica izvedena iz dnevnika događaja radi brzog čitanja. Može se obrisati i ponovno izgraditi.
- **SSE / WebSocket / polling**: tri načina za „uživo". Kod SSE poslužitelj šalje novosti kroz otvorenu vezu, u jednom
  smjeru. WebSocket je dvosmjerna veza. Kod pollinga preglednik svakih N sekundi pita ima li novoga.
- **Voditelj obrade / izvršitelj obrade** (GDPR): voditelj odlučuje zašto se osobni podaci obrađuju, a izvršitelj ih
  obrađuje po njegovu nalogu, uz ugovor.
- **[PROC]**: procjena, nije mjerenje. **[PROVJERITI]**: treba provjeriti izvan repoa (službene stranice, fakultet).

## Odgovor na Leonovo pitanje

**Da, iz ovoga možemo nastaviti. Ništa se ne prepisuje, ali trebaju tri nova sloja i nekoliko proširenja.**

1. **Srce ostaje kakvo jest.** Jezgra je čista i deterministička, a igra je dnevnik događaja s punim snimkom pravila
   (`crates/game/src/command.rs:93-101`). API je tanak (`crates/api/src/lib.rs:1`), a slučajevi upotrebe stoje iza
   portova (`crates/app/src/ports.rs:105-145`). To je već oblik platforme: druga baza, drugi AI ili profesorov ekran
   ne diraju jezgru.
2. **Temelji za produkciju već postoje:**
   - red poslova s zakupom i odgodom (`crates/store/migrations/0003_platform.sql:23-36`, `0004_jobs_run_after.sql`);
   - zaključavanje igre po naredbi (`crates/store/src/log.rs:109`);
   - dnevnik u koji baza ne da pisati preko starih zapisa (`0002_events_append_only.sql`);
   - AI koji se gasi bez pada igre (`crates/server/src/main.rs:52-66`);
   - kolačić `httpOnly`, a u bazi samo otisak tokena (`crates/api/src/auth.rs:32-37`, `crates/app/src/demo.rs:20-23`);
   - za svaki AI poziv već se bilježe tokeni (`crates/game/src/event.rs:140-145`).
3. **Fali identitet.** Danas postoji samo anonimni gost (`Caller { guest }`, `crates/app/src/demo.rs:10-12`).
   Nema korisnika, uloga, kolegija, razreda, koda igre ni profesorskih ekrana. Na webu „Za profesora" samo bira stari
   model (`apps/web/src/screens/StartScreen.tsx:80`).
4. **Fali produkcijska „vodovodna instalacija":**
   - poslužitelj sluša samo na `127.0.0.1` (`main.rs:144`);
   - kolačić nema `Secure` (`auth.rs:31`);
   - sqlx nema TLS (`Cargo.toml:27`);
   - sadržaj se traži na putu iz vremena prevođenja (`crates/content/src/lib.rs:74-76`);
   - radnik reda poslova je jedan i radi serijski (`main.rs:123-136`);
   - nema Dockerfilea, kopija baze, nadzora grešaka ni granice AI troška.

   Sve su to poznate stavke iz BACKLOG-a `[prije Railwaya]`; razbijene su na cigle u fazi SPREMNO ZA INTERNET
   ([ROADMAP/platforma](../../plan/ROADMAP/platforma.md)).
5. **Ciljano se mijenja jedno mjesto u jezgri, bez prepisivanja.** Igra danas ima uvijek 1 čovjeka i 3 robota
   (`[HotelId; 4]`, `command.rs:42`; `slot BETWEEN 1 AND 4`, `0003:17`), a krug se obračunava za sva četiri hotela
   odjednom (`resolve_round`). Razred traži N studentskih hotela i obračun po hotelu. To je promjena srca igre, pa ide
   u kolosijek IGRA, fazu RAZRED U JEZGRI ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md),
   [ROADMAP/igra](../../plan/ROADMAP/igra.md)), a ne u platformski kod. Hotel je uz to jedan od tri fiksna profila A/B/C
   (`profile_at`, `command.rs:28-34`; ključ kataloga v2 `BTreeMap<Profile, …>`, `crates/sim-hotel-v2/src/catalog.rs:62`);
   za generirane hotele postaje **opis hotela zapisan u snimak igre** (dio 04).
6. **„Graditi odmah na produkciju":** sve ide lokalno do profesorova pregleda, uz 0 € novog troška
   ([ADR-004](../../records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md)). Izlazak na internet je tek uz
   Leonov izričit OK ([ADR-005](../../records/DECISIONS/ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)), a stvarni
   studenti dolaze tek kad su gotovi prijava i GDPR minimum (dio 07). Troškovi: dio 06.
