# ARHITEKTURA §7–§8 — jezgra simulacije, tok runde

Dio [ARHITEKTURA.md](../ARHITEKTURA.md)

## 7 · Jezgra simulacije

### 7.1 Ugovor jezgre

```text
resolve(stanja svih hotela, događaji runde, katalog, kocka) → nova stanja + promjene (Delta) + novi utori
```

Čista funkcija: isti ulaz → isti izlaz, uvijek, na svakom računalu.

### 7.2 Determinizam — kako se jamči

- **Cijeli brojevi:** novac u centima (`i64`), postoci u baznim bodovima. Nema decimalnih brojeva u jezgri.
- **Bez prekoračenja:** `overflow-checks = true` i u produkcijskom profilu (Rust ga po zadanom tamo isključuje pa bi se
  broj tiho „prevrnuo"), a jezgra računa izričito provjerenim operacijama (`checked_*`, `saturating_*`).
- **Bez nasumičnog redoslijeda:** `HashMap` i `HashSet` zabranjeni su u jezgri (clippy `disallowed-types`), umjesto
  njih `BTreeMap`/`BTreeSet`. Zabrana nije dogovor nego provjera koja ruši build.
- **Vlastita kocka** (poznati algoritam, npr. PCG ili SplitMix, sa „zlatnim" testnim vektorima: sjeme → poznati niz).
  Ne ovisi o vanjskoj biblioteci: `rand` izričito dopušta da nova manja verzija promijeni niz brojeva, a `StdRng`
  nije prenosiv uopće. Bez `f32`/`f64` i `usize` u ishodima.
- **Neovisni tokovi kocke:** svako bacanje dobiva sjeme izvedeno iz `(igra, runda, hotel, svrha)`. Dodavanje novog
  bacanja na jednom mjestu **ne pomiče** sva ostala — stare sezone ostaju iste.
- **Određen redoslijed:** događaji se primjenjuju po rednom broju u dnevniku, hoteli po identifikatoru. Nikad po
  vremenu dolaska ni redoslijedu iz mape.
- **Bez sata:** vrijeme ulazi samo kao podatak (broj runde, rok), nikad kao `now()`.
- **Puni snimak pravila, ne samo oznaka:** pri stvaranju igre u dnevnik ide `GameCreated` s **cijelim** snimkom:
  katalog, koeficijenti, profili, postavke, verzija jezgre, AI model i verzija uputa. `RulesetId` je samo otisak
  tog snimka. Ponovni obračun koristi isti snimak, osim kad se namjerno isprobavaju nova pravila (alat za balans).
- **Pravila se ne mijenjaju usred sezone**, osim izričitom migracijom koja je i sama događaj u dnevniku.

### 7.3 Stanje hotela

- **Kategorija** (3*/4*/5*): podatak hotela koji bira referentne vrijednosti (cijena, elastičnost…) iz kataloga; vidi DEMO §2.0.
- **Stvarni sloj:** sobe, popunjenost, ADR, prihodi i troškovi, broj zaposlenih, moral, fluktuacija, stanje opreme,
  ocjena na portalima, sigurnosni incidenti… (točan popis: s profesorom).
- **Indeksi (5):** svaki je čista funkcija stvarnog sloja s koeficijentima iz kataloga.
- **Tragovi:** skup skrivenih zapisa s trajanjem (npr. `DeferredMaintenance { system: Boiler, since: round 2 }`).

### 7.4 Mehanike

- Rust `enum Mechanic` — svaka varijanta je jedna vrsta poteza s tipiziranim parametrima.
- Svaka mehanika je mala funkcija: `(parametri, kvaliteta, stanje, koeficijenti) → Vec<Delta> + tragovi`.
- **Granice iz kataloga** se primjenjuju nakon izračuna: nijedna promjena ne može izaći iz dopuštenog raspona,
  bez obzira na ulaz (K2).
- **Mehanike su kod, brojke su podaci:** nova vrsta poteza traži kod (i test); promjena težine poteza ne traži (K3).

### 7.5 Obračun runde — faze

| faza | što radi |
|---|---|
| 1. Prikupljanje | Svi događaji runde iz dnevnika, u određenom redoslijedu. |
| 2. Odluke | Odgovori, radnje i presude → mehanike → promjene i tragovi. |
| 3. Protok vremena | Sezonalnost, redovni troškovi, prirodno trošenje opreme i morala. |
| 4. Tržište | Potražnja → popunjenost i prihod. **Danas po hotelu; sutra zajednički bazen** (međusobni utjecaj). Za bazen vrijedi **pravilo simetrije**: ako dvama jednakim hotelima zamijeniš oznake, ishodi se samo zamijene (automatski test). Izjednačenja se dijele jednako, ostatak kockom iz sjemena runde, nikad po oznaci hotela. |
| 5. Splet okolnosti | Okidači čitaju tragove i stanje, bacaju kocku → utori za iduće runde. |
| 6. Indeksi i ocjena | Novih pet indeksa i relativni napredak. |
| 7. Objašnjenje | Popis svih promjena s uzrocima → profesorov uvid i mentorov ulaz. |

### 7.6 Raspored situacija

Ulazi: **osnovni raspored** iz sjemena igre (svi hoteli u rundi dobivaju jednako zahtjevnu mješavinu po području i
težini) · **utori iz spleta okolnosti** · **profesorove postavke** (AI uključen/isključen, količina, raspon težine) ·
**profesorove ručne situacije**. Izlaz: utori s vremenom objave, područjem, težinom, rokom i izvorom.

### 7.7 Relativna ocjena

Sučelje `ScoringRule` s jednom zadanom implementacijom (npr. ponderirani zbroj pomaka indeksa u odnosu na start,
normaliziran po početnom potencijalu). Formula je **pitanje za profesora** (§19); zamjenjiva bez promjene ostatka.

### 7.8 Pravednost profila — mjeri se, ne pretpostavlja

**Fer znači jednake mogućnosti napretka, a ne jednak početni zbroj.** Hotel s financijama 20 nije na istom mjestu
kao hotel s financijama 55, pa ocjena mjeri napredak u odnosu na **prostor za rast**.

- **Alat za balans** (`crates/balance`) vrti jezgru bez baze i ekrana: za svaki profil (A, B, C) i isti skup
  strategija mjeri raspodjelu relativne ocjene kroz tisuće sezona.
- Profili su prihvatljivi ako je **razlika medijana manja od praga** i **nijedan profil ne pobjeđuje pod svim
  strategijama**. Isti alat traži i **dominantne strategije** (jednu kombinaciju poteza koja svugdje pobjeđuje).
- Prag se utvrđuje nakon prvih mjerenja, s profesorom. Dok ga nema, profili su zapisana pretpostavka.

---

## 8 · Tok jedne runde i greške

### 8.1 Stanja runde

```text
ZAKAZANA ──(početak po kalendaru)──► OTVORENA ──(rok)──► ZATVARANJE ──(sve presude ili istek)──► OBRAČUN
   ──► OBRAČUNATA (prozor za pregled) ──(otvaranje iduće runde)──► ZAKLJUČANA
```

Prijelaze radi `worker` kao poslove; svaki prijelaz je događaj u dnevniku.

- **Prozor za pregled:** od obračuna do otvaranja iduće runde. Ispravka presude u tom prozoru znači **puni ponovni
  obračun** runde.
- **Zaključana runda:** povijest se više ne prepisuje. Ispravka postaje **kompenzacija** (§8.3, korak 6).

### 8.2 Životni ciklus situacije

```text
UTOR PLANIRAN → SADRŽAJ TRAŽEN → SPREMNA → OBJAVLJENA → ODGOVORENA / ISTEKLA → (PRESUDA) → OBRAČUNATA
                      │
                      └─ AI ne odgovori ili izlaz nije valjan ──► PREDLOŽAK (rezerva)
```

Sadržaj se stvara **unaprijed** (prije vremena objave), pa spor AI ne kasni studentu.

### 8.3 Slobodan odgovor — korak po korak

1. Student pošalje tekst → `ResponseSubmitted` u dnevnik (odmah, u transakciji) → odgovor „primljeno".
2. Posao `AssessResponse` → AI vraća presudu po shemi.
3. Provjera: oblik (shema), mehanika postoji u katalogu i dopuštena je u toj situaciji, kvaliteta 1–5.
   Nevaljano → ponovni pokušaj (ograničen broj).
4. Valjano → `VerdictRecorded` u dnevnik, s verzijom upute i modela.
5. Ako presude nema do kraja zatvaranja → presuda „neutralna" po **unaprijed zapisanom pravilu** i oznaka „na
   pregled profesoru". Profesor može upisati presudu → `VerdictOverridden`.
6. Što ispravka presude pokrene ovisi o stanju runde:
   - **runda u prozoru za pregled** → ponovni obračun te runde (moguć jer je sve u dnevniku);
   - **runda zaključana** → `CompensationScheduled`: jezgra izračuna razliku između stvarnog ishoda i ishoda s
     ispravnom presudom i primijeni je u **sljedećem** obračunu, samo na tom hotelu. Situacije koje su u
     međuvremenu nastale ostaju, jer je student na njih odgovorio s informacijama koje je tada imao.
     Kad dođe zajedničko tržište, kompenzacija **nikad ne preraspodjeljuje tuđe goste**.

### 8.4 Greške i otpornost

| što pođe po zlu | što se dogodi |
|---|---|
| AI ne radi / spor | Predložak umjesto AI situacije; presude čekaju, pa pravilo za istek (§8.3). |
| AI limit potrošen | Isto kao gore; profesor vidi upozorenje. |
| AI vrati besmislicu | Shema i katalog je odbiju; ponovni pokušaj; zatim rezerva. |
| Worker padne usred obračuna | Obračun je u jednoj transakciji i **idempotentan**: ponovno pokretanje daje isti rezultat, ništa se ne duplira. |
| Dvostruki klik / ponovljeni zahtjev | Svaka naredba nosi ključ idempotentnosti; drugi put se vraća prvi rezultat. |
| Dva obračuna iste igre istovremeno | Postgres zaključavanje po igri; drugi čeka ili odustaje. |
| Student pošalje nakon roka | Odbijeno s jasnom porukom; rok je provjeren na poslužitelju. |

### 8.5 Slobodan potez

Slobodan potez je **zasebna naredba** (`POST /v1/hotels/{id}/actions`), a ne odgovor na situaciju.

1. Student opiše potez tekstom → `FreeActionSubmitted` u dnevnik.
2. AI ga svrsta u mehaniku iz **cijelog** kataloga (s parametrima i rubrikom izvedivosti) ili ga označi kao
   **nepoznat**.
3. Nepoznat potez **nema brojčani učinak**: student dobije objašnjenje, a potez ide na popis kandidata za nove
   mehanike (profesor i autor ga vide).
4. Svrstan potez ide kroz istu provjeru kao presuda (§8.3, korak 3) i obračunava se u fazi 2 (§7.5).

**Sloboda je u izrazu, ne u posljedicama.** Kad bi AI smio izmišljati posljedice, pali bi K1 i K2. Katalog zato
raste iz stvarnih poteza studenata.

**Bez AI-ja** (zasad samo zapisano, odluka s profesorima, pitanje 15): situacije iz predložaka, likovi s napisanim
dijalozima, mentor s napisanim objašnjenjima; što sa slobodnim tekstom — otvoreno.

---
