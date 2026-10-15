Cigla B2 je podijeljena na dva dijela, jer ne stane u jedan dan (grana živi najviše ~1 dan). **B2a** mijenja pravilo
izvlačenja, dnevnik i pečat, uz jedno dizanje `CORE_VERSION`. **B2b** daje balansu broj situacija kao ulaz i proširuje
paritet. B2b ne mijenja brojke. Redoslijed: B2a, pa B2b.

### Task B2a: Broj situacija u postavkama igre, mjesečno 4/5/6, isti predložak najviše jednom u krugu

**Veličina:** M–L, 7–8 h · **Ovisi o:** A1, A3 · **Prvi u fazi B** (redom B2a → B2b → B3a → B3b → B1a → B1b → B4a–c, voditelj 15.10.). Ne ide paralelno s B1a, jer oba dižu
`CORE_VERSION` (B2a 2.2.0 → 2.3.0, B1a 2.3.0 → 2.4.0) i pišu `v2-golden.json`; B4 dolazi tek nakon B1b.
· **Model:** opus · **Prevodi Rust:** da

**Cilj:**
- Broj situacija po krugu više nije konstanta u kodu. Postavka igre zapisana je u `GameCreated`.
- Zadane vrijednosti: tjedno 3/4/5, mjesečno 4/5/6, po očekivanoj popunjenosti (ispod 75 % / 75–89 % / od 90 %).
  Dopušteno je 3–8 situacija, i broj ne smije padati s popunjenošću.
- Mjesečni krug se izvlači kao **jedan** krug, a ne kao niz virtualnih tjedana. Isti predložak dolazi najviše jednom u
  krugu, a iz kruga u krug smije se ponoviti.
- Najmanji razmak ponavljanja (4 tjedna) i „prilika barem svaka 3 tjedna" sele iz koda u datoteku predložaka.

**Zašto (nalaz):**
- [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §4 #13 i §6 (B2); [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 12–14.
- [BUGS.md](../../records/BUGS.md) B3 (`P-PRAONICA.jul` i `.jul.2`): uzrok je `count.rs:64-94` (`virtual_weeks`) i
  razmak u danima (`eligible.rs:136-155`).
- Izvještaji revizora: 02 #6–7, 03 #10 (`v2_situations.rs:266-272`), 01 #12.
- `faza-a-review.md` I3: pečat i paritet znaju samo vrstu kruga. Trijaža: provjera da `s.drawn` nije prazan → B2.

**Kako se danas zapisuju događaji (provjereno):**
- Postoji jedna verzija oblika za sve događaje, `SCHEMA_VER = 1` (`event.rs:13`). Kod čitanja (`event.rs:371-381`) nema
  „podizanja" starih događaja.
- Polja se dodaju samo uz `#[serde(default, skip_serializing_if = …)]` (uzor `GameCreated.rounds`, `event.rs:28-30`).
  Strukture imaju `deny_unknown_fields`.
- Zato B2a **ne** diže `SCHEMA_VER`. Novo polje je `Option`, a stari dnevnik bez njega čita se sa zadanim brojem za
  vrstu kruga.
- v1 igre polje nikad ne pišu. Da se i dalje čitaju, dokazuju `v1_frozen.rs` i `fixtures_sealed.rs`.
- Stare v2 igre ionako dobiju 409 zbog nove verzije jezgre (`model.rs:139-148`, Review Focus 1 plana).

**Files:**
- Create: `crates/game/tests/situation_settings.rs`.
- Modify (jezgra):
  - `crates/sim-hotel-v2/src/situations/count.rs:29-94`: brišu se `weekly_count`, `period_count`, `VirtualWeek` i
    `virtual_weeks`. Dolaze `SituationCounts` i `situation_count_with`. `situation_count` (`:51-54`) ostaje kao omotač
    sa zadanim brojem.
  - `situations/mod.rs:17-19` (`pub use`), `:44-46` (opis polja `day`: „početak kruga").
  - `situations/template.rs:14-22`: `TemplateSet` dobiva tri polja.
  - `situations/eligible.rs:136-155`: konstanta `MIN_GAP_WEEKS` odlazi, a `repeat_allows` dobiva `min_gap_weeks: u8`.
  - `situations/draw.rs`:
    - `:13` (uvoz), `:26-27` (konstanta `OPPORTUNITY_EVERY` odlazi);
    - `:189-218` (`candidates`: predložak koji je već u `out` ne ulazi);
    - `:286-292` (`needs_opportunity` dobiva broj tjedana);
    - `:313` i `:385` (broj iz `round_count`);
    - `:443-545`: petlja po virtualnim tjednima postaje jedan blok s `day = p.start_day`, `target = n`,
      `anchors = d.anchors` i jednom granicom odjela `dept_limit(n)`.
  - `crates/sim-hotel-v2/Cargo.toml:3` → `2.3.0` (2.2.0 → 2.3.0; sljedeće dizanje je B1a na 2.4.0), uz redak verzije u `Cargo.lock`.
- Modify (sadržaj):
  - `crates/content/src/templates.rs:12-21`: `SituationTemplates` dobiva dva polja.
  - `templates.rs:23-37`: `TemplateFile` dobiva dva `Option<u8>` polja s `serde(default, skip_serializing_if)`.
  - `crates/content/src/v2.rs:118-124`: prepisuje ta dva polja.
  - `templates_check.rs:26-29`: vrijednost 0 je greška, kao kod grupa.
  - `templates_core.rs:15-31`: zadano je 4 i 3, a `counts: None`.
  - `content/hotel/v2/situation_templates.json:3-4`: dodaju se `"min_gap_weeks": 4` i `"opportunity_every_weeks": 3`.
- Modify (igra):
  - `crates/game/src/event.rs:15-31`: `GameCreated.situation_counts` i nova struktura `SituationCountsDto`.
  - `command.rs:27-31`: `GameSettings.situations`.
  - `command.rs:41-59` i `:62-119`: provjera, pa zapis za v2. v1 polje ne piše.
  - `game.rs:88-107`: polje `situation_counts`.
  - `game.rs:126-193`: čitanje u `created()`; neispravna vrijednost daje `Corrupt`.
  - `drawn.rs:286`: `let mut set = tpl.to_core(); set.counts = Some(self.situation_counts);`.
- Modify (balans): `crates/balance/src/v2_situations.rs:266-272`: `count_band` vraća `(low, high)` iz
  `SituationCounts::default_for(kind)`, što daje tjedno 3–5 i mjesečno 4–6.
- Ne dirati:
  - API, `openapi.json`, web (profesorov ekran dolazi u kolosijeku PLATFORMA);
  - `MANDATORY_SITUATIONS`, ocjenu 2 za prepuštanje i pragove 75/90 % (vidi Rizik);
  - `situation_effect_bp` (L1-13);
  - zamrznute datoteke `game/tests/fixtures_v2/v2-snapshot.json` i `content/tests/fixtures/content-52ce161/`;
  - tekstove poruka brana balansa;
  - `docs/`.

**Interfaces:**
- Consumes: `expected_occupancy_bp` (`count.rs:17`), `Game::rounds_info`, `Model::check_version`, `GameError::InvalidSettings`
  (`error.rs:34-35`), `common::{hotel_ids, GUEST, play_through, apply_ev}`.
- Produces:
```rust
// sim_hotel_v2::situations (count.rs) — jezgra bez serde
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SituationCounts { low: u8, mid: u8, high: u8 }   // polja privatna: invarijanta samo kroz `new`
impl SituationCounts {
    pub const MIN: u8 = 3;                   // = obvezne (MANDATORY_SITUATIONS)
    pub const MAX: u8 = 8;
    pub const WEEK: SituationCounts;         // 3/4/5
    pub const MONTH: SituationCounts;        // 4/5/6
    pub fn new(low: u8, mid: u8, high: u8) -> Option<SituationCounts>; // MIN ≤ low ≤ mid ≤ high ≤ MAX
    pub fn default_for(kind: PeriodKind) -> SituationCounts;
    pub fn levels(self) -> (u8, u8, u8);
    pub fn for_occupancy(self, occupancy_bp: u16) -> u8;              // <7_500 low, <9_000 mid, inače high
}
pub fn situation_count_with(s: &HotelState, p: &Period, cat: &Catalog, c: SituationCounts) -> Result<u8, ResolveError>;
// TemplateSet: pub min_gap_weeks: u8, pub opportunity_every_weeks: u8,
//              pub counts: Option<SituationCounts>  // postavka igre; None = default_for(p.kind); sadržaj ga ne puni
// content::templates: pub const DEFAULT_MIN_GAP_WEEKS: u8 = 4; pub const DEFAULT_OPPORTUNITY_EVERY_WEEKS: u8 = 3;
// game::event
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct SituationCountsDto { pub low: u8, pub mid: u8, pub high: u8 }
// GameCreated: #[serde(default, skip_serializing_if = "Option::is_none")] pub situation_counts: Option<SituationCountsDto>,
// game::command
pub struct GameSettings { pub kind: PeriodKind, pub situations: Option<SituationCounts> } // None = zadano
// game::Game
pub fn situation_counts(&self) -> SituationCounts;
```

**Testovi čuvara** (čuvar sam uskladi i doslovne zapise `GameSettings { … }` (13 mjesta, `grep -rn "GameSettings {"
crates`) i `TemplateSet { … }` (`v2_situation_capacity.rs:226`, `v2_situation_draw_edges.rs:110`) novim poljima; sve
je to CRVENO jer se ne prevodi):
1. `crates/game/tests/situation_settings.rs` (nova; pomoćna `created(kind, situations)` zove `new_game_with` sa sjemenom
   42, `common::GUEST`, `common::hotel_ids()`):
   - `new_v2_game_writes_default_counts_for_its_round_kind`: tjedno `Some(SituationCountsDto { low: 3, mid: 4, high: 5 })`,
     mjesečno 4/5/6, `Some(new(5, 7, 8))` daje 5/7/8. Za v1 igru (`Snapshot::read`) vrijednost je `None`.
   - `counts_outside_three_to_eight_or_not_rising_are_refused`: `new` vraća `None` za (2,4,5), (3,4,9), (3,5,4) i
     (6,5,6), a `Some` za (3,3,3) i (8,8,8). Provjerava i `usize::from(SituationCounts::MIN) == MANDATORY_SITUATIONS`.
   - `log_without_counts_still_reads_with_defaults` (stari dnevnik):
```rust
let mut log: Vec<Recorded> = created(PeriodKind::Week, None).unwrap().into_iter().zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event }).collect();
let mut v = log[0].event.to_payload();
v.as_object_mut().unwrap().remove("situation_counts").expect("nova igra piše broj");
log[0].event = GameEvent::from_record("GameCreated", SCHEMA_VER, v).unwrap();
assert_eq!(Game::fold(&log).unwrap().situation_counts(), SituationCounts::WEEK);
```
   - `corrupt_counts_in_log_are_refused`: `GameCreated` s 3/9/9 → `Game::fold` daje `Err(GameError::Corrupt(_))`.
     `new_game_with` sa `situations` izvan granica nije moguć, jer `new` vraća `None`.
   - `monthly_game_with_three_has_three_in_every_round`: mjesečna igra s `new(3, 3, 3)` → `play_through(g, "sep")` →
     za svaki krug `situations_of_hotel(me, r)?.len() == 3`.
2. `crates/content/tests/v2_situation_count.rs`:
   - `monthly_count_is_proportional_to_days` (`:62-82`) postaje `monthly_count_is_four_five_six`:
     `MONTH.for_occupancy` za 7_000 / 8_000 / 9_500 daje 4/5/6;
   - po profilu preko `situation_count`: A `[4, 5, 6, 5]`, B i C `[4, 5, 6, 4]`. To je ista razina popunjenosti kao
     danas: 13→4, 17/18→5, 22→6;
   - tjedni testovi prelaze na `SituationCounts::WEEK.for_occupancy`, a očekivanja ostaju ista.
3. `crates/content/tests/v2_situation_draw_month.rs`:
   - brišu se `virtual_weeks_split_the_month_count` i test ponavljanja „od 28. dana", s očekivanjem `[5,5,5,5,2]` i
     `repeated > 0`;
   - novi `monthly_round_draws_a_template_at_most_once`: 3 profila × 300 sjemena, `month_set()`. U svakom krugu
     `drawn.len() == situation_count(...)`, nijedan predložak dvaput, svaki primjerak ima `day == p.start_day`,
     `occurrence == 1` i šifru `"{predložak}.{krug}"`;
   - isti test broji predloške koji dođu dva mjeseca zaredom i traži da ih bude `> 0` (ADR-042 t. 12: iz kruga u krug
     ponavljanje je dopušteno).
4. `crates/content/tests/v2_templates.rs`, novi `gap_and_opportunity_live_in_the_template_file`:
   - sadržaj iz repoa daje `to_core()` s `(min_gap_weeks, opportunity_every_weeks, counts) == (4, 3, None)`;
   - snimak bez ta dva ključa (`v["v2"]["situation_templates"]`, `remove`) daje 4 i 3;
   - `min_gap_weeks: 0` → `build()` je `Err`.
5. `crates/game/tests/v2_sealed.rs` (I3):
   - `Case` dobiva `situations: Option<(u8, u8, u8)>`, a `CASES: [Case; 8]`. Dodaju se `t7-broj-578` (tjedno, sjeme 7,
     odgovara, 5/7/8) i `m42-broj-333` (mjesečno, 42, šutnja, 3/3/3).
   - `play` zove novi `common::v2_game_with(snap, seed, GameSettings)`, a `v2_game_from` ostaje kao omotač.
   - `digest` iz `GameCreated` uzima i `"situation_counts": c.situation_counts`.
   - Test 3 traži slučaj sa svojim brojem za obje vrste kruga. U svakom slučaju svaki krug mora imati izvučenu situaciju
     (šifra s točkom) i najviše `high` situacija.
   - Novi test 5, `digest_reacts_to_situation_counts`: isti slučaj s 3/3/3 i sa zadanim brojem daje različit otisak.
   - Test 1 je CRVEN dok se ne zapečati nova verzija (drugi popis slučajeva).
6. `crates/api/tests/student_sees.rs:226`: makni `#[ignore]` s `monthly_round_never_repeats_a_template`. Danas je
   CRVEN s porukom „jul: predložak P-PRAONICA dvaput".
7. `crates/game/tests/situations_delegate.rs:14-17, 46-56`: lipanj ima 4 situacije, a ne 13, pa `>= 5` postaje `>= 4`
   (opis u komentaru se ispravlja). Ovo je karakterizacija starog broja.

Testovi koji moraju ostati zeleni i nepromijenjeni (karakterizacija):
- `v2_situation_draw_snapshot.rs`: tjedni otisci od 52ce161. To je dokaz da se tjedno izvlačenje nije promijenilo.
- `v1_frozen.rs`, `fixtures_sealed.rs`, test 2 i test 4 pečata;
- tjedni scenariji A6 i t05;
- `game_parity_v2.rs` (zadani broj na obje strane).

**Koraci:**
1. **Čuvar:** testovi 1–7 u jednom commitu, „Popravak B2a (čuvar): …".
   - U izvještaj ide izlaz `cargo test -p game --test situation_settings` (ne prevodi se).
   - Za test 6 ide izlaz `cargo test -p api --test student_sees -- monthly_round_never` (asercija).
2. **Graditelj, jezgra:**
   - `SituationCounts` i `round_count(d, occ) = d.templates.counts.unwrap_or(default_for(p.kind)).for_occupancy(occ)`.
   - Jedan blok u `draw_situations_with`, filtar „već u krugu" u `candidates`, i `repeat_allows` i `needs_opportunity`
     s brojem iz `TemplateSet`.
   - Za tjedni krug redoslijed izvlačenja i potrošnja kocke ostaju isti, jer je virtualni tjedan i dosad bio jedan blok.
3. **Sadržaj:** dva polja, provjera nule, `to_core` i JSON.
4. **Igra:** `SituationCountsDto` ↔ `SituationCounts`.
   - `new_game_with`: `situations.unwrap_or(default_for(kind))`, pa `Some(dto)` samo za v2.
   - `created()`: `None` → `default_for(prvi krug.period.kind)`. Neispravan zapis → `corrupt("broj situacija u postavkama")`.
   - `drawn.rs:286`: postavlja `set.counts`.
5. **Balans:** `count_band` iz `default_for` (komentar `:266` → „tjedno 3–5, mjesečno 4–6").
6. **Pečat:** digni `CORE_VERSION`, pa `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest`.
   Stari ključevi u `v2-golden.json` ostaju.
7. **Bilješke:** u `.superpowers/sdd/2026-10-14-popravak-temelja/za-dokumentaciju.md` upiši:
   - BUGS B3 → „Stanje: popravljeno B2a";
   - ADR-042 t. 12–14 provedeno;
   - MODEL_V2 i plan situacija: „mjesečno 13–22" → „4/5/6";
   - 🟨 pretpostavke iz Rizika;
   - BACKLOG: pragovi 75/90 %, ocjena 2 i „3 obvezne" još u kodu (REVIZIJA #13 ostatak; C5 za web tekst).
8. Commit „Popravak B2a: broj situacija u postavkama igre, mjesečno 4/5/6, predložak jednom u krugu".

**Provjere** (svaka zasebno, `; echo EXIT=$?`, izlaz u datoteku, cijeli `ci.yml`):
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings`
  · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` (samo kad nitko drugi ne prevodi u stablu)
  · `git diff --exit-code`.
- `cargo sqlx migrate run --source crates/store/migrations` i `SQLX_OFFLINE=false cargo sqlx prepare --workspace --check`
  (baza `hotelixsim_agenti`) · `cargo test --workspace`. Cijeli `student_sees` je zelen, bez `#[ignore]` za B3.
- Balansi: prije prve izmjene spremi osnovicu u scratchpad.
  - v1 `--seeds 100 --check` i `--seeds 1000`: `cmp` je bajt-isti.
  - v2 `--round week --seeds 1000 --check`: `diff` smije dirati samo retke odjeljka
    „== Tjedno prema mjesečnom ==" i retke „sloj 1 tjedno ≈ mjesečno" u „Mjere robota"; sve ostalo je bajt-isto.
  - v2 `--round month --seeds 1000 --check`: **brojke se mijenjaju**, pa se diže `CORE_VERSION` i obnavlja pečat A3.
    Očekuje se „najmanje 4 najviše 6" i ~19–20 po sezoni, uz EXIT=0. Ako padne brana modela (J-3 raznolikost ili
    dominantna proba), stani i vrati BLOCKED s dijagnozom. Brojke se ne štimaju, to radi L1-13.
- `corepack pnpm check:docs` · `test:docs` · `check:kod` (`draw.rs` se smanjuje) · `check:api` (bez promjene)
  · `lint` · `typecheck` · `test` · `corepack pnpm --filter @hotelixsim/web build`.
- `git diff --stat` ne smije sadržavati `openapi.json`, `apps/`, `packages/` ni zamrznute datoteke (`v2-snapshot.json`,
  `content-52ce161/`). `cargo deny` radi samo CI.

**Rizik i odluke (🟨):**
- **Mjesečni krug je jedan blok**, a ne tjedni. To je ADR-042 t. 13 doslovno („ne da se računa kao da su tjedni prošli").
  - Svi primjerci imaju dan početka kruga. Razmak od 4 tjedna zato između dva mjeseca uvijek prolazi (30 ≥ 28), pa
    ponavljanje iz kruga u krug ostaje dopušteno (t. 12).
  - Predložak s razmakom 5 ili 6 preskače jedan mjesec.
- **Prilika:** u mjesečnoj igri od srpnja vrijedi „barem jedna prilika u krugu", jer je prošli krug uvijek više od 14
  dana prije. Tako je bilo i dosad (svaki mjesec je imao 4–5 virtualnih tjedana).
- **Postavka je zapisana izričito za svaku novu v2 igru** (`Some`). Kasnija promjena zadanih brojeva u kodu ne mijenja
  staru igru. To je temelj za C1 (stara igra se čita i nakon promjene jezgre). `None` znači samo „stari dnevnik".
- **Broj ne smije padati s popunjenošću** (`low ≤ mid ≤ high`): više gostiju znači više posla (izvedeno iz t. 13, nije
  pitano). Pragovi 75/90 % ostaju u kodu, jer ih nijedna odluka ne seli.
- **Profesor mijenja broj između krugova** (t. 14): to nije u B2a. `Game::situation_counts()` vrijedi za cijelu igru.
  Događaj „postavke od kruga k" dolazi s ekranom profesora (PLATFORMA), a tada metoda dobiva krug.
- **Preračun učinka jedne situacije prema broju** (t. 14, „profesorov izbor ne mijenja pravednost") nije u B2:
  - mijenja obračun i ide uz kalibraciju (L1-13);
  - dok ga nema, svoj broj može postaviti samo test, jer API ga ne nudi.
  - Posljedica: mjesečni udio situacija u napretku (danas Brižni 19,3 %) **pada** s B2a. To je mjera robota i ne ruši
    `--check`. Scenarij A6 „mjesečno odgovori > šutnja" čuva smjer.
- **Seme u scenarijima:** lipanj sjemena 42 sada ima 4 situacije (S1, S3, kolektivna i jedna izvučena). Ako 4. nema
  voditelja ili lipanj nema lika sa slobodnim odgovorom, padaju `situations_delegate`, A6 „prepuštanje" ili A6
  „razgovor". Unaprijed dopušteno: **čuvar** u dodatnom commitu bira prvi krug (jun→sep) koji ima takvu situaciju.
  Tvrdnja testa se ne mijenja. Graditelj testove ne dira.
- **Pečat i slobodni odgovor (I3, drugi dio):** slučaj `respond` + `record_verdict` i slučaj `set_plan` čovjeka
  **ne ulaze u B2**, nego u **B1a** (voditelj 15.10.). Razlozi:
  - B1 mijenja baš taj put (mehanike najbližeg odgovora, `resolve.rs:52-79`) i ionako diže verziju, pa ga tamo
    pregledava onaj tko ga mijenja, bez dodatnog dizanja verzije (2.4.0);
  - B2 taj put ne dira, pa slučaj ovdje ne bi ništa čuvao između B2 i B1 (B3 je samo balans, a B4 mijenja sadržaj
    koji zamrznuti snimak pečata ne vidi);
  - B2a je već ~1 dan.
  - B2a priprema teren: `Case` ima postavke, a `play` zove `v2_game_with`. B1a dodaje samo nove vrste `Human`.

**Leonov dio:** — (odlučeno, ADR-042 t. 12–14). Voditelj mu **prije igranja** kaže dvije stvari:
- nakon spajanja (verzija 2.3.0) njegove probne v2 igre daju 409 i treba nova igra;
- mjesečna igra sada ima 4–6 situacija, a pojedina situacija do L1-13 teži manje.

Drugi dio kartice (B2b): [task-09b-b2b-balans-prima-broj-situacija.md](task-09b-b2b-balans-prima-broj-situacija.md).
