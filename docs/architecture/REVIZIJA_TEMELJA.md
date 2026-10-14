# Revizija temelja (14.10.2026.)

> **Status:** prijedlog za Leonovu odluku. Ništa nije građeno ni mijenjano u kodu.
> **Povod:** Leon, 14.10.: *„ne gradimo od baze nego od sagrađenog bungalova radimo ciglenu nadogradnju koja se može samo
> srušit u jednom trenutku"*.
> **Kako je rađeno:** šest revizora (agenti koji samo čitaju), svaki na svom sloju, nad stanjem grane `model-v2/sloj1-l111`
> (b1bf446: sloj 1 + obnova + situacije). Njihovi puni izvještaji (s `datoteka:redak` za svaku tvrdnju) su u
> `.superpowers/sdd/2026-10-14-revizija/01…06-*.md`. Nalaze označene ✔ provjerio sam i sam, ponovnim čitanjem koda ili brojanjem.

## 1. Odgovor u tri rečenice

1. **Temelj nije bungalov.** Ono što je u ovakvom sustavu najteže i najskuplje dobiti točno već je točno: dnevnik događaja,
   ista igra iz istog sjemena, račun u cijelim brojevima, zamrznuta pravila u svakoj igri, baza koja ne dopušta prepravljanje
   povijesti, čvrst red poslova, AI iza jednog sučelja i ugovor između weba i poslužitelja. Gradnja ispočetka bi završila na
   **istom nacrtu**, ali uz 4–5 puta veću cijenu.
2. **Leonov osjećaj je ipak djelomično točan, i to na tri mjesta.** (a) v2 stoji na **v1 kalupu**: situacije v2 računaju
   se mehanikom iz v1, koja ne zna za odjele. To je ona „cigla na bungalovu" i odatle dolazi B2. (b) Brane balansa koje stalno
   padaju **mjere robota, a ne studenta**, pa svaka kalibracija lovi robotov broj. (c) **Duge odvojene grane** skrivaju
   međusobne učinke do spajanja (B6, B8, B9).
3. **Preporuka: popravak u tri faze (A → B → C), ne gradnja ispočetka.** Prvo brane koje mjere pravu stvar, zatim zamjena
   v1 kalupa ondje gdje već puca, i tek onda **jedna** kalibracija. Redoslijed iz predaje (odmah kalibrirati L1-13) mijenjam,
   jer bi kalibracija stare mjere značila kalibrirati dvaput (§5).

## 2. Kuća u slikama

| dio kuće | što je to u kodu | stanje |
|---|---|---|
| **Temelj** | dnevnik događaja, determinizam, cjelobrojni račun, snimak pravila u igri, baza samo za dopisivanje, red poslova, AI iza sučelja, ugovor openapi → web | **stoji**, dobro izveden |
| **Nosivi zid od starog materijala** | v2 posuđuje iz v1 mehanike situacija, oblik izvješća, indekse i robota | **puca** na mjestima gdje v2 treba odjele (B2, ADR-042 t. 1 i 7) |
| **Katovi kojih još nema** | krug po hotelu, prijava i uloge, nadogradnja pravila za igru koja traje, osobe (radnici kao ljudi) | **planirani**, a ne pokvareni; isto bi nedostajalo i u novoj gradnji |
| **Instalacije koje ne mjere** | brane balansa preko robota, načela igre koja nisu provjere, mjerilo AI-ja koje nije nikad pokrenuto | **krive mjere**, a alat je dobar |
| **Gradilište** | duge grane, rad koji postoji samo na ovom disku | **rizik**, jeftino se popravlja |

## 3. Što stoji (temelj)

- **Dnevnik događaja je stvaran.** Stanje igre se uvijek slaže iz dnevnika (`crates/game/src/game.rs:110-125`), a obračun
  kruga se sprema u dnevnik, ne računa se ponovno pri čitanju (`event.rs:103-122`). ✔ Naredba vraća događaj, ne mijenja stanje.
- **Ista igra iz istog sjemena.** Vlastita kocka sa zlatnim testom (`crates/engine/src/rng.rs`) i račun u cijelim brojevima s
  provjerom preljeva. Zabranjeni su `f64`, `HashMap` i sat, a to ne čuva dogovor nego prevoditelj (`clippy.toml`, `#![forbid]`).
- **Pravila su zamrznuta u igri.** Cijeli sadržaj s otiskom upisuje se pri stvaranju igre, pa izmjena kataloga ne mijenja
  igru u tijeku (`crates/content/src/lib.rs:579-592`). Server s neispravnim sadržajem ne krene.
- **Baza čuva povijest.** Okidač odbija izmjenu i brisanje dnevnika (`crates/store/migrations/0002_events_append_only.sql`).
  Jedna brava po igri, pa se dva istodobna upisa ne gaze (`crates/store/src/log.rs:188-244`). To je točno ono što razred treba.
- **Red poslova je čvrst:** zakup, ponovni pokušaji, jedinstveni ključ posla, pa drugi klik ne radi drugi obračun (`store/src/jobs.rs`).
- **AI ne piše brojke.** AI kaže samo koji je od pet kriterija ispunjen, a ocjenu broji igra (`crates/game/src/verdict.rs:53-56`).
  Izlaz AI-ja postaje događaj, pa ponovno čitanje igre nikad ne zove AI. Svaka uloga ima rezervu kad AI padne. Ključ je samo na poslužitelju.
- **Ugovor web ↔ poslužitelj:** `openapi.json` se generira iz koda, web tipovi iz njega, a CI pada kad se razilaze.
- **Proces hvata greške u kodu:** pregledi su vratili cigle s pravim kvarovima koje bi student vidio (G-1, G-2, W-1).
  Brane determinizma (`check-core-deps`, probe) mjere ono što tvrde.

## 4. Gdje puca

Težina: **K** = kritično, **V** = važno. Stupac „sloj" pokazuje u kojem je izvještaju puni dokaz.

| # | nalaz | dokaz | kad pukne | tež. | sloj |
|---|---|---|---|---|---|
| 1 | Rad postoji **samo na ovom disku**: `sloj1-l111` (24 commita, cijele situacije), `sloj1-situacije` (23), `sit-sadrzaj` (5); cijeli `.superpowers/` (533 datoteke, sve odluke i izvještaji) je gitignoriran | ✔ `git rev-list` prema `origin`; `.superpowers/sdd/.gitignore:1` | kvar diska ili slučajno brisanje mape | **K** | 06 |
| 2 | Situacije v2 računaju se **v1 mehanikom**: hitan popravak je samo trošak, a „oprema" iz svih situacija ide u jedno opće stanje (popravak pumpe na bazenu diže sobe) | ✔ `sim-hotel/src/mechanic.rs:160-167`; ✔ `sim-hotel-v2/src/resolve/guests.rs:105-111` | već puklo (B2); ADR-042 t. 1 i 7 ne mogu se napraviti bez nove v2 mehanike | **K** | 01, 02 |
| 3 | **Brana „udio situacija" mjeri robota.** Dijeli se s napretkom koji robot postigne bez situacija, pa pada čim se robotu poboljša plan, iako se situacije nisu promijenile | `balance/src/v2_situations.rs:218-264` | B6; ponovno pri ADR-042 t. 1–2, banci ljudi i voditeljima | **K** | 03 |
| 4 | **6 od 17 v2 brana mjeri robote.** U 10.–14.10. mjerilo je ~10 puta „popravljeno" promjenom sadržaja, robota ili granice | izvještaj 03, popis s izvorima | svaka nova mehanika | V | 03 |
| 5 | **Načela igre nisu provjere.** Najbolji odgovor je prvi u 55/55 predložaka i najdulji u 54/55; samo 4 od 15 kvarova imaju uvjet stanja | ✔ vlastito brojanje nad `situation_templates.json`; `content/src/templates_check.rs` | već puklo (B1, B2); svaki novi predložak | **K** | 02 |
| 6 | **Nakon promjene jezgre stara v2 igra se ne može ni otvoriti** (409 `game_outdated` već pri čitanju), a web nema poruku ni izlaz | ✔ `game/src/model.rs:140-150`; `apps/web/src/App.tsx:108-118` | prvi popravak jezgre usred semestra: cijeli razred bez igre | **K** za pilot | 01, 05 |
| 7 | Verzija v2 se diže ručno i nema zapečaćeni otisak (v1 ga ima). J-3, G-1 i L1-11 promijenili su obračun bez dizanja verzije | `sim-hotel-v2/src/lib.rs:50-52`; `game/tests/fixtures_sealed.rs` (samo v1) | tiha promjena pravila u igri koja traje | V | 01 |
| 8 | **Dvije petlje sezone** (igra i balans) bez testa da daju isto | `game/src/command.rs:142` prema `sim-hotel-v2/src/season.rs:142` | balans mjeri petlju koja nije igra | V | 01, 03 |
| 9 | **Jedan radnik** obrađuje sve poslove redom, pa odgovor lika čeka iza presude | ✔ `crates/server/src/main.rs:125-135` | razred od 30: presude ne stignu u 120 s i postaju neutralne 3/5 | **K** za razred | 04, 05 |
| 10 | Krug se zatvara **za cijelu igru**: bilo koji član zatvara ga svima | ✔ `crates/app/src/commands.rs:123-138`; `RoundClosed` nema hotel (`game/src/event.rs:97`) | ADR-037 t. 2; poznato i planirano (~30 h) | **K** za razred | 01, 05 |
| 11 | AI ne zna granice koje igra provodi (6 poruka, redni broj poruke), a sudac ne vidi opis situacije ni cijene odgovora | ✔ granica samo u `game/src/talk.rs:16`, nema je u `prompts/`; `ai/src/request.rs:66-99` | B4; presuda bira „najbliži odgovor", a on određuje trošak | V | 04 |
| 12 | **Kvaliteta presude je nepoznata:** mjerilo nije nikad pokrenuto (0 od 40 ocjena), pokriva samo S1 i S2 i gradi ulaz vlastitom kopijom koda | `content/hotel/mjerilo/*.json`; `ai/src/benchmark.rs:94-125` | studenti uspoređuju ocjene, a nitko ne zna jesu li dobre | V | 04 |
| 13 | Brojke koje student osjeti su u kodu (broj situacija 3/4/5, razmak, ocjena 2 za prepuštanje), i dvaput u web tekstovima | ✔ `sim-hotel-v2/src/situations/count.rs:30-36`; `hr.ts:50` | ADR-042 t. 12–14 (profesor bira broj) | V | 02, 05 |
| 14 | Duge grane: `sloj1` 157 commita i 4 dana odvojen od `main`; obnova i situacije balansirane odvojeno | `git log main..model-v2/sloj1` | svaka faza koja dira napredak (razred, banka ljudi) | **K** za proces | 06 |
| 15 | Dio pravila živi u webu (ocjena odjela s prepisanim brojkama jezgre, tumačenje „Obvezna") | `apps/web/src/screens/departments/score.ts`; `situation/rules.ts:17-23` | profesorov pregled, engleski, promjena jezgre | V | 05 |

**Što revizori NISU našli:** nedeterminizam, prepravljanje povijesti, AI koji piše brojke, ključ u pregledniku, gaženje
istodobnih upisa ni sadržaj koji tiho prođe s pogrešnim oblikom. Toga se kod ovakvih projekata najviše treba bojati, i to stoji.

## 5. Cijena: popravak prema gradnji ispočetka

Sati su procjena rada s agentima [PROC], bez Leonova vremena. Zbroj je umanjen za poslove koje su dva revizora
brojala dvaput (v2 mehanike, radnici, broj situacija, granice AI-ja).

| sloj | popravak | ispočetka | što bi se izgubilo ispočetka |
|---|---|---|---|
| jezgra i model | 30–45 h | 150–300 h | zlatni testovi, v1 kompatibilnost, kalibracija |
| sadržaj i provjere | 35–54 h | 60–90 h | snimak + otisak, ~25 testova odbijanja |
| balans | 21–25 h | 30–40 h (ne bi riješilo ništa: krive su mjere, ne alat) | usporedivost s ~40 mjerenja |
| AI sloj | 35–55 h | 50–80 h | zaštita od ubacivanja uputa, rezerve, iskustvo u uputama |
| web, API, baza | 20–30 h čistih popravaka | 250–400 h | ~150 Rust i ~500 web testova, kaljeni red poslova |
| način rada | 10–16 h | — | — |
| **ukupno** | **~125–185 h** | **~540–910 h** | i isti nedostajući katovi kao danas |

Gradnja ispočetka **ne bi uklonila** najveće rupe (#6, #10, osobe, nadogradnja pravila), jer to nisu greške temelja, nego
pojmovi koji još nisu građeni. Morali bi se graditi i u novoj kući. Planirana gradnja za razred (prijava, uloge, obračun po
hotelu, ~75–120 h) je jednaka u oba slučaja i nije u ovoj tablici.

## 6. Preporuka: popravak u tri faze

**Faza A: učvrsti (prije ikakve nove značajke), ~25–35 h**
- **A0. Sigurnosna kopija, odmah (0,5 h):** `git bundle` svih lokalnih grana i arhiva `.superpowers/` izvan ove mape (#1).
  Gdje točno, odlučuje Leon (pitanje 2).
- **A1. Razdvojiti brane (3 h):** „brane modela" i dalje ruše provjeru, a „mjere robota" se samo ispisuju. Time `sloj1-l111`
  može ući u `sloj1` bez kalibracije stare mjere, i grana situacija se zatvara (#3, #14).
- **A2. Brane koje mjere pravu stvar (~15–20 h):** načela sadržaja kao provjere pri učitavanju (redoslijed i duljina
  odgovora, uvjet stanja za kvar, `{odjel}` padež; #5); zapečaćeni otisak v2 vezan uz verziju jezgre (#7); test da igra i
  balans daju isto (#8); granice igre u ulazu AI-ja, `lik.v3` (#11).
- **A3. Način rada (2 h):** jedna integracijska grana, nijedna grana dulja od ~1 dana, balans nakon svakog spajanja; Leon
  igra svaka 2–3 dana na jednoj adresi; 3–5 testova „što student vidi" na razini API-ja.

**Faza B: zamijeni v1 kalup ondje gdje već puca, ~35–50 h**
- **B1.** Vlastite v2 mehanike situacija s ciljem odjela: popravak diže stanje tog odjela (ADR-042 t. 1, 7; #2).
- **B2.** Broj situacija i razmak u postavke igre (`GameCreated`) i u sadržaj, ne u kod (ADR-042 t. 12–14; #13).
- **B3.** Nova mjera „udio situacija" koja ne ovisi o robotovu planu. Leon odlučuje što „~20 %" znači; to ide kroz anketu
  kad dođe na red.
- **B4.** Prepisati oznake 55 predložaka (duljine, uvjeti kvarova), uz Leonov pregled.
- **Tek tada jedna kalibracija (L1-13)** i ostatak ADR-042 (L1-13b). Stari plan je bio kalibrirati sad pa opet nakon ADR-042
  t. 1–2, odnosno dvaput.

**Faza C: prije razreda i pilota, ~50–90 h**
- Igra koja traje semestar: stara igra se čita i nakon promjene jezgre, a nadogradnja pravila postaje događaj (#6).
  Web dobiva poruku i izlaz za staru igru.
- Više radnika za poslove i neuspjeli poslovi vidljivi u zdravlju sustava (#9).
- Mjerilo AI-ja na v2 predlošcima uz Leonove ocjene; profesorov pregled i ispravka presude (#12).
- Predmemorija složene igre; pravila iz weba seliti u API (#15).
- Zatim planirani katovi: obračun po hotelu, prijava i uloge, osobe (banka ljudi).

**Što se ne radi:** ne prepisuje se jezgra, baza, red poslova, AI sloj ni web. v1 kalup se ne ruši odjednom. Izvješće
i indeksi odvajaju se od v1 postupno, uz sloj 2, kad osobe ionako mijenjaju izvješće (~30–50 h, procijenjeno u izvještaju 01).

## 7. Što je pouka (za BUGS i AGENTI nakon odluke)

- Proces je dobro čuvao da kod radi ono što kartica kaže. **Nije mogao uhvatiti B1–B5**, jer načela igre nisu bila zapisana
  ni kao provjera ni kao scenarij. Lijek je zapisati načelo kao provjeru prije gradnje, a ne pisati više papira.
- Brana koja mjeri robota ne smije rušiti provjeru, jer nema vanjskog sidra koje kaže tko je u pravu. Prava brana koja mjeri
  **studenta** dolazi kad postoje snimljene igre (Leonove, pilot): dnevnici događaja vrte se u balansu.
- Grane koje diraju isti napredak spajaju se isti dan.
