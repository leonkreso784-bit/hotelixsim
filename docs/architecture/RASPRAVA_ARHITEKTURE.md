# HotelixSim — rasprava o arhitekturi

> Zapis rasprave o nacrtu [ARHITEKTURA.md](./ARHITEKTURA.md). Ovdje stoje primjedbe, odgovori, prijedlozi i
> otvoreni upiti. Ono što Leon potvrdi prelazi u ARHITEKTURA.md.

---

## Krug 1 · 2026-10-04 · Pregled nacrta v1

Pregled je otvorio šest točaka i dvije primjedbe o jezicima i trošku. Prošao sam svih šest i usporedio ih s
dokumentom. **Svih šest stoji.** Nijedna ne ruši smjer, ali sve su stvarne rupe. Točne su i dvije primjedbe o
jezicima: moje obrazloženje u §5 i §18 bilo je slabo.

### Šest točaka i kako bih ih zatvorio

**1. Ispravka presude nakon što igra ide dalje.** Točno, pravilo nedostaje. Prijedlog:

- Nakon obračuna runda ima **prozor za pregled**, do otvaranja iduće runde. Unutar tog prozora ispravka znači puni
  ponovni obračun runde.
- Kad se iduća runda otvori, prethodna je **zaključana** i povijest se više ne prepisuje. Ispravka tada postaje
  **kompenzacija**: jezgra izračuna razliku između stvarnog ishoda i ishoda s ispravnom presudom i primijeni je u
  sljedećem obračunu, samo na tom hotelu.
- Situacije koje su već nastale ostaju, jer je student na njih odgovorio s informacijama koje je tada imao.
- Kad dođe zajedničko tržište, kompenzacija nikad ne preraspodjeljuje tuđe goste.

**2. Oznaka verzije nije dovoljna.** Točno. Moj §17 obećava previše („stare sezone se i dalje ponovno izračunaju").
Ispravak:

- Pri stvaranju igre u dnevnik ide **puni snimak**: katalog, koeficijenti, profili, postavke, verzija jezgre,
  AI model i verzija uputa. Oznaka sama nije dovoljna.
- Pravila se **ne mijenjaju usred sezone**, osim izričitom migracijom koja je zapisana kao događaj.
- Svako izdanje backenda označava se u gitu, a njegova slika kontejnera se čuva. Stara sezona pokreće se **svojom**
  verzijom jezgre.
- Rezultati i objašnjenja završenih sezona spremaju se trajno.
- Iskreno obećanje: ponovljivo s verzijom kojom je sezona igrana. Ne obećavam iste brojke stare sezone na novoj
  jezgri.

**3. Pravednost nije isto što i ponovljivost.** Točno, i ovo je najvažnija točka.

- **Fer znači jednake mogućnosti napretka**, a ne jednak početni zbroj.
  - Alat za balans mjeri, za svaki profil, raspodjelu relativne ocjene pod istim skupom strategija.
  - Profili su prihvatljivi ako je razlika medijana manja od praga, a nijedan profil ne pobjeđuje pod svim
    strategijama. Prag utvrđujemo nakon prvih mjerenja, s profesorom.
  - Relativna ocjena mora mjeriti napredak u odnosu na **prostor za rast**: hotel s financijama 20 nije na istom
    mjestu kao hotel s financijama 55.
- **Pravednost AI-ja je zaseban problem:**
  - Svaka situacija dobiva **rubriku** s kriterijima za ocjene 1–5.
  - Ponuđeni odgovori ocijenjeni su po istoj rubrici, pa su tekst i opcije na istoj ljestvici.
  - AI uz ocjenu mora navesti koje kriterije odgovor ispunjava.
  - Postoji skup odgovora koje su ocijenili ljudi, i model ide u igru tek kad se s njima slaže iznad praga.
  - Model i upute su zaključani za cijelu sezonu, a student može uložiti prigovor profesoru.

**4. Vlastiti red poslova.** Djelomično točno. Slažem se da je pouzdanost pravi posao, a ne samo tablica. Ali slučaj
„odgovor je spremljen, a posao nije" zapravo je glavni razlog **za** red u Postgresu: odgovor i posao upisuju se u
**istoj transakciji**, pa to stanje ne može nastati.

- Pad procesa, zaglavljeni poslovi i zakašnjeli AI rezultati rješavaju se ovako: posao se preuzima samo na
  ograničeno vrijeme, neuspjeli se ponavljaju, a prije upisa rezultata provjerava se stanje runde.
- Između vlastite izvedbe i gotove biblioteke (apalis, pgmq) biram **probnim testom s tim scenarijima**, a ne
  mišljenjem.
- Za `engine` se slažem: kreće minimalno i raste prema potrebama hotela.

**5. Što tko vidi.** Točno, to nisam definirao.

- Prijedlog: javna je ljestvica, a privatno ostaje sve ostalo: odluke, tekstovi, tragovi i stvarne brojke.
- Za tržište dodajem **pravilo simetrije**: ako dvama jednakim hotelima zamijeniš oznake, ishodi se samo zamijene.
  Provjerava ga automatski test.
- Izjednačenja se dijele jednako, a ostatak kockom iz sjemena runde. Nikad po oznaci hotela.

**6. Slobodan potez i igra bez AI-ja.** Točno.

- **Slobodan potez** postaje zasebna naredba, a ne odgovor na situaciju. AI ga svrsta u mehaniku iz **cijelog**
  kataloga ili ga označi kao nepoznat. Nepoznat potez nema brojčani učinak, student dobije objašnjenje, a potez
  ide na popis kandidata za nove mehanike.
- Moram biti iskren oko ovoga: **sloboda je u izrazu, ne u posljedicama.** Kad bi AI smio izmišljati posljedice,
  pali bi K1 i K2. Katalog zato raste iz stvarnih poteza studenata.
- **Bez AI-ja:** situacije dolaze iz predložaka, likovi imaju napisane dijaloge, a mentor napisana objašnjenja.
  Za slobodan tekst treba odluka (upit 1 dolje).

### Jezici i trošak

- **Prekoračenje broja:** točno. Rust u produkcijskom načinu po zadanom ne provjerava prekoračenje, pa se broj tiho
  „prevrne". Rješenje: uključiti provjeru i u produkciji, a u jezgri računati samo izričito provjerenim operacijama.
  Postoji i zamka koju recenzija ne spominje: Rustov `HashMap` ima nasumičan redoslijed. U jezgri ga zabranjujem,
  a provjerava alat.
- **TypeScript i `bigint`:** točno, argument s decimalnim brojevima bio je slab. Determinizam dolazi iz dizajna u
  bilo kojem jeziku. Pravi razlozi za Rust su drugi:
  - stroži prevoditelj kao ograda za kod koji piše AI;
  - brzina za masovne simulacije;
  - jezgra koja radi i u pregledniku i na mobitelu.

  Obrazloženje ću prepraviti.
- **Railway:** dva servisa u dvije okoline uz Postgres prelaze minimum. Za pilot predlažem **jedan servis** u kojem
  API i pozadinski poslovi rade u istom procesu, samo produkciju, a staging lokalno. Cjenik nisam zasebno provjerio,
  pa stvarnu potrošnju mjerimo u prvom mjesecu.

### Upiti za Leona

1. **Kad je AI isključen, što sa slobodnim tekstom?** Preporučujem da ga ocjenjuje profesor kroz red za pregled.
   Tako srž igre ostaje i bez AI-ja. Druga opcija je da se slobodan tekst tada potpuno isključi.
2. **Što student vidi o drugima?** Preporučujem ukupnu ocjenu i pet indeksa kroz runde, bez stvarnih brojki i
   odluka. To je dovoljno za učenje iz usporedbe, a nema se što prepisati.
3. **Da ovo ugradim u [ARHITEKTURA.md](./ARHITEKTURA.md)?**

### Odgovori

- **Upit 1 (Leon, 04.10.):** AI ostaje sastavni dio simulacije, barem za demo. Što se događa kad se AI isključi
  definira se kasnije, prema tome što profesori žele. → Pitanje je dodano u
  [PITANJA_ZA_PROFESORE.md](../product/PITANJA_ZA_PROFESORE.md) (pitanje 22).
- **Upit 2 (Leon, 04.10.):** prihvaćena preporuka. Student o drugima vidi ukupnu ocjenu i pet indeksa kroz runde,
  bez stvarnih brojki i odluka. Vrijedi dok profesori ne odgovore na pitanje 14 (i imena ili anonimno).
- **Upit 3:** ugrađeno u nacrt v2 (krug 2).

---

## Krug 2 · 2026-10-04 · Nacrt v2

Krug 1 je ugrađen u [ARHITEKTURA.md](./ARHITEKTURA.md). Gdje je što:

| točka | gdje |
|---|---|
| 1. Ispravka presude (prozor za pregled, zaključana runda, kompenzacija) | §8.1, §8.3 (korak 6) |
| 2. Puni snimak pravila, iskreno obećanje o starim sezonama | §7.2, §17 |
| 3. Pravednost profila (alat za balans, prag) i AI-ja (rubrika, ljudsko mjerilo) | §7.8, §10.3 |
| 4. Red poslova (transakcija, zakup, izbor probnim testom) | §9.4 |
| 5. Što tko vidi, pravilo simetrije | §12, §7.5 (faza 4) |
| 6. Slobodan potez kao zasebna naredba | §8.5 |
| Rust: determinizam iz dizajna, prekoračenje, zabrana `HashMap` | §5.1, §7.2, §18 |
| Lokalno pa Railway kao jedan servis | §14, §20 |

**Novo iz istraživanja** ([research/RUST_ALATI_2026-10.md](../research/RUST_ALATI_2026-10.md)):

- Gotovi Rust redovi poslova su ili RC ili na SQLx 0.8, a SQLx 0.8 i 0.9 ne dijele transakciju. Zato je vlastiti
  mali red najvjerojatniji ishod probnog testa (§9.4).
- `rand` izričito dopušta promjenu niza brojeva u manjoj verziji, pa je vlastita kocka (§7.2) potvrđena.
- utoipa 6 zna dati OpenAPI 3.2, a TS generator čita samo 3.0 i 3.1. Izlaz se zato drži na 3.1.

**Leon, 04.10.:** sve se gradi lokalno, Railway je za kraj. Leon je rekao da radim dalje bez pitanja, pa v2 ide u
gradnju kao radna verzija. Primjedbe se upisuju ovdje kao krug 3.
