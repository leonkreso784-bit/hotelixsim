# ADR-037 — Razred: studenti jedni protiv drugih, obračun po hotelu, natjecanje za ljude, igra kroz semestar
**Datum:** 2026-10-13 · **Status:** ✅ **Leonove odluke 13.10.** (razgovor u sesiji) · otvoreno: D4 (t. 7) i brojke
preotimanja (t. 5, anketa kad faza krene) · **Vezano:** [ADR-028](ADR-028-leonove-odluke-za-put-b-ljudi.md), [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md),
[ADR-016](ADR-016-leonove-proizvodne-odluke-za-korak-3.md) t. 3, [ROADMAP](../../plan/ROADMAP.md)

**Kontekst.** Igra danas ima jednog studenta i tri robota, a krug se obračunava za sva četiri hotela odjednom
(`crates/game/src/resolve.rs`, `resolve_round`). Svaki se hotel računa iz svog stanja i svog plana, pa hoteli danas
ne dijele goste. Platformski nacrt (12.10.) predlagao je razred kao mnogo zasebnih igara „student + 3 robota". Leon je
to 13.10. ispravio: studenti igraju **jedni protiv drugih u istoj igri**, a roboti služe samo za vježbu i usporedbu.
Ovaj ADR zapisuje što iz toga slijedi.

**Odluka.**
1. **Jedna igra po razredu, N studentskih hotela.** Profesor pri stvaranju bira koliko studenata ulazi, a studenti
   ulaze kodom. U razrednoj igri nema robota. Igra za vježbu (student + 3 robota) ostaje kakva jest.
2. **Obračun po hotelu (D1, varijanta I).** Profesor otvara krug, ručno ili po rasporedu. Student igra otvoreni krug i
   zatvara ga kad je spreman: njegov hotel se tada obračuna i on odmah vidi rezultat. Nitko ne ide iza otvorenog
   kruga, a tko zaostane može sustići. Odbačena je varijanta „svi zajedno" (krug se računa kad su svi gotovi ili kad
   profesor zatvori), jer brzi čekaju spore i jer igra mora raditi i kroz semestar (t. 5).
3. **Lista kolega.** Student uvijek vidi listu hotela u razredu: tko je zatvorio krug, a tko još igra (kao danas
   roboti), i može otvoriti hotel kolege.
4. **Vidljivost.** Tuđi **radnici su vidljivi**: koliko je radnik dobar i da je **jedan od razloga zašto hotel dobro
   radi** (doprinos uspjehu). To vrijedi za svakog radnika i za voditelje odjela. Skrivene osobine ostaju skrivene.
   Tuđi troškovi i odgovori na situacije ostaju skriveni. Ovo zamjenjuje platformski zapis od 13.10. „javno kao na
   Bookingu, ali ne ljudi".
5. **Natjecanje za ljude.** Student A može ponuditi bolju plaću radniku hotela B, a radnik sam odlučuje hoće li otići.
   - Ponuda iz A-ova kruga k stiže vlasniku B kao obavijest. **B može dati protuponudu** (povišica ili razgovor) do
     kraja svog kruga k.
   - Radnik odlučuje kad B zatvori krug k, a prelazi od kruga k+1. Odluka ovisi o zadovoljstvu (plaća, voditelj,
     opterećenje, osobine). Računa je formula s kockom iz vlastitog toka, a AI ne zapisuje brojke.
   - Ograničenje: najviše 1–2 ponude po krugu i trošak zapošljavanja, da bogat hotel ne isprazni tuđi u jednom krugu.
   - **Profesor može isključiti** preotimanje za svoj razred.
   - U igri za vježbu roboti imaju svoje pravilo (Brižni čuva ljude, Lovac preotima).
6. **Igra na satu i kroz semestar.** Ista igra mora raditi na predavanju (krug za 10–15 minuta, profesor otvara) i
   kroz više tjedana ili cijeli semestar (krug se otvara po rasporedu). 17 tjednih krugova sezone otprilike odgovara
   semestru. Raspored otvaranja i rokovi zato pripadaju fazi RAZRED, a ne tek pilotu.
   - **Neaktivan student:** kad rok prođe, njegov hotel sam prelazi u sljedeći krug s planom kakav je imao, a
     situacije ostaju neodgovorene. Mentor ga na to upozorava.
   - **Obavijesti** (novi krug, ponuda za radnika) su lagane, u aplikaciji.
   - **AI granica** vrijedi po igri, odnosno po semestru, ne po satu.
7. **Prijava.** Cilj je **AAI@EduHr** za profesore i studente. To ovisi o fakultetu i Srcu, a o tome još nema
   informacija. **Prije toga** gradimo vlastite račune: profesor ili sustav studentima izda **korisničko ime i
   lozinku**, na primjer iz popisa studenata. Lozinka se čuva samo kao otisak, profesor je može resetirati, a kod igre
   i dalje služi za ulazak u igru. Identitet je odvojen od korisnika, pa se AAI kasnije veže na isti račun. Kako se
   profesor prijavljuje za prvu probu (D4) ostaje otvoreno.
8. **Engleski.** Igra kasnije mora biti i na engleskom: sučelje, AI (likovi, presuda, mentor) i sadržaj situacija.
   To je smjer za poslije pilota.

**Posljedice.**
- Obračun po hotelu je najveća promjena srca igre na putu do pilota (~30 h [PROC]). Ide u kolosijek IGRA, faza
  RAZRED U JEZGRI ([ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)). Stare igre se čitaju kao i prije.
- Utjecaj među hotelima djeluje samo **na prijelazu kruga**, nikad usred tuđeg kruga. To čuva obračun po hotelu i
  determinizam. Isti obrazac kasnije nosi i druge veze među hotelima (npr. zajedničku burzu rada). Zajedničko
  tržište gostiju (VIZIJA) ostaje izvan pilota. Ako ikad dođe, mora poštovati isto pravilo.
- Brisanje jednog studenta iz zajedničke igre nije brisanje cijele partije, nego anonimizacija i redakcija (GDPR,
  faza PILOT). To traži pravni odgovor fakulteta.
- Platformski nacrt (`platforma-arhitektura.md`, još u ledgeru) prepisuje se po ovom ADR-u u fazi TEMELJ RADA.
