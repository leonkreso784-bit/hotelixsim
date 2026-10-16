# Leonov pregled · B — tekstovi odgovora (B4) i mjerilo AI-ja

Dio indeksa [2026-10-17-leonov-pregled.md](../2026-10-17-leonov-pregled.md). B1 koči L1-15 (završnu provjeru faze B);
B2 čitaš tek **nakon** slijepog izbora.

### B1 · Slijepi pregled tekstova B4a, B4b, B4c
- **Što je:** agenti su prepisali ponuđene odgovore u 59 situacija (B4a 19, B4b 22, B4c 18) tako da najbolji ne odskače
  duljinom ni oblikom. Ti si mjera: ako najbolji pogodiš po obliku, a ne po sadržaju, tekst nije dobar.
- **Kako:** u mapi `.superpowers/sdd/2026-10-14-popravak-temelja/` otvori `b4a-pregled-tekstova.md`, pa `b4b-…` i `b4c-…`.
  Odgovori su izmiješani i označeni slovima. Za svaku situaciju upiši `Moj izbor najboljeg:` slovo, pa `Odluka:` ✔
  (prihvaćam) ili ✎ s tvojim tekstom. **Odjeljke „Ključ" i „Napomene voditelja" na kraju ne gledaj prije izbora.**
  Na kraju usporedi s Ključem i uz promašaje napiši je li te prevario oblik ili sadržaj.
- **Koliko:** ~1–1,5 min po situaciji: B4a ~25 min, B4b ~30 min, B4c ~25 min; slobodno u tri sjedenja.
- **Preporuka:** sve tri cijele, prije L1-15.
- **Inače:** samo situacije „s napomenom" (5 + 14 + 14 = 33). **Cijena:** izbor više nije slijep, a ostalih 26 nitko ne
  provjeri.
- **Odgovor:** upisuješ izravno u te tri datoteke; voditelj ih čita i šalje jedan krug ispravaka (B4-ispravci, ~1 h,
  brojke igre ostaju iste).

### B2 · Napomene koje već znamo (tek nakon B1)
- **Što je:** pregledi su našli sitnice koje ne ruše igru, ali ih ti vidiš kao student:
  a. **B4a (N1):** tri slabija odgovora nose radnju koju rubrika nagrađuje, a nemaju kriterij: P-PRISTUPACNOST („rampa u
     planu"), P-KRIZA-OSOBLJA („za dva tjedna"), P-BOJLER („provjeriti ostale"). Bodovi su isti, ali student koji napiše tu
     radnju može biti uparen sa slabijim odgovorom. Ostale napomene B4a stoje u Ključu (S3, S4, P-POHVALA,
     P-BUCNA-SOBA, P-PRISTUPACNOST).
  b. **B4a (N4):** P-BOJLER, slabiji odgovor „20 % popusta tjedan dana" ne kaže na što je popust.
  c. **B4b:** P-DIJETE-IZGUBLJENO: najbolji ima sve korake drugog i još neke („biraj onaj s najviše koraka" pogađa);
     P-SLOBODNI-DANI: slabiji doslovno sadrži kriterij najboljeg („pisani zahtjevi"); P-PREPORUKA: „njezinoj tvrtki",
     „15 % popusta".
  d. **B4c:** nakon popravka nema čistog slučaja „najbolji = slabiji + još nešto", ostala su 3 granična (P-POSEBNA-PREHRANA
     kruh, P-LEZALJKE i P-SPA-TERMIN razlikuju se samo isprikom); test dopušta najviše 2 od 18 čistih.
  e. **Svih 59 (N3):** „biraj odgovor čije riječi ima i neki drugi" pogađa najbolji u 21 od 57 (slučajno ~27 %).
- **Preporuka:** a, b, c popraviti u B4-ispravcima nakon tvog pregleda; d prihvatiti; e rješava miješanje odgovora u L1-13b.
- **Inače:** što ti u slijepom pregledu nije smetalo, ostaviti.
- **Vidi:** „Napomene voditelja" na kraju `b4b-` i `b4c-pregled-tekstova.md`,
  [ADR-055](../../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 6.

### B3 · Sitnice sadržaja situacija
- **Što je:**
  a. P-RANI-DOLAZAK: opis kaže „jedna gošća", odgovori „Stigli su";
  b. rezervni tekst lika „Gošća je previše ljuta da bi nastavila razgovor" zvuči kao ocjena studenta;
  c. S3 (ponuda agencije) nema opis (samo naslov), a S3 i S4 nemaju rubriku (kriterije za ocjenu);
  d. objašnjenje indeksa „Zaposleni" predugo je i tehničko.
- **Preporuka:** a, b, d voditelj popravi sam (samo tekst, npr. b → „Gošća je prekinula razgovor."); za c agent napiše
  nacrt opisa S3 i rubrika S3 i S4, a ti potvrdiš (~15 min).
- **Inače:** ti napišeš rubrike S3 i S4 sam.
- **Vidi:** BACKLOG „Sadržaj situacija", `content/hotel/situations.json`, `content/hotel/catalog.json`.

### B4 · Mjerilo AI suca: tvojih 40 ocjena
- **Što je:** 40 nacrta studentskih odgovora za S1 i S2; ti upišeš ocjenu 1–5 (`leon_quality`), a stroj izmjeri slaže li se
  AI sudac s tobom (cilj: barem 80 % ocjena unutar ±1 od tvoje).
- **Pretpostavili smo:** da je sudac dobar, bez te mjere.
- **Preporuka:** da, ~40 min, prije C3 (novi sudac koji ocjenjuje i razgovor) i prije svake zamjene modela suca.
- **Inače:** kasnije. **Cijena:** ne znamo koliko sudac griješi, pa ne možemo ni probati jeftiniji model.
- **Vidi:** `content/hotel/mjerilo/S1.json`, `S2.json`;
  [ADR-047](../../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md) t. 5.

### B5 · Rubrike sidara S1 i S2 i stari nacrt S1b, S5–S8
- **Što je:** rubrika = kriteriji po kojima sudac ocjenjuje odgovor; ti si mjerilo. S1b i S5–S8 su nacrt iz prve igre
  (drugi val), napisan prije nego što je v2 dobila 55 predložaka.
- **Preporuka:** pročitaj kriterije S1 i S2 (~10 min; S3 i S4 su u B3c); nacrt S1b, S5–S8 **zatvoriti** (v1 je zamrznut,
  v2 ima svoje predloške).
- **Inače:** prepisati S1b, S5–S8 u v2 predloške. **Cijena:** ~2–3 h i ponovni balans.
- **Vidi:** [SITUACIJE_S1b_S5-S8](../../product/SITUACIJE_S1b_S5-S8.md), DEMO §12 „Za Leonovu provjeru" t. 5.
