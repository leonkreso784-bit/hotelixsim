Cigla PRAVILO ima tri dijela (cijela je ~10–14 h; grana živi najviše ~1 dan). **PRAVILO-a** je pravilo u igri: postavka,
dnevnik, ocjena klika i presude, mentor i pečat (diže `CORE_VERSION`). **PRAVILO-b** daje balansu put „klik najbolji",
brane i paritet. **PRAVILO-c** pokazuje pravilo studentu (app, API, web). Redoslijed: a, pa b i c (b i c smiju usporedo:
oba prevode Rust, to su točno dva). Odluka: [ADR-047](../../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md),
nacrt [BODOVANJE_VLASTITOG_ODGOVORA](../../architecture/BODOVANJE_VLASTITOG_ODGOVORA.md) §3 (opcija A).

### Task PRAVILO: Vlastiti odgovor vrijedi više od klika (klik najviše 4, vlastite riječi do 5)

**Veličina:** a 5–6 h · b 3–4 h · c 3 h · **Ovisi o:** B2a (`GameSettings`, `GameCreated.situation_counts`), B3a/B3b
(putovi i brane), B1a (`CORE_VERSION` 2.4.0). Ne ovisi o B4c (B4c mijenja samo sadržaj; pečat igra zamrznuti snimak).
· **Paralelno s:** B4c i banka ljudi K1 samo ako najviše dva agenta prevode Rust · **Model:** čuvar i graditelj sonnet,
pregled opus (a: jezgra igre i pečat; b: balans) · **Prevodi Rust:** da (sva tri dijela).

**Cilj:**
- Nova postavka igre `AnswerRule` (`Classic` = danas, `OwnWords` = ADR-047), zapisana izričito u `GameCreated` svake
  nove v2 igre. Zadano za nove v2 igre: `OwnWords` (🟨). Stari dnevnik bez polja i v1 = `Classic`, bajt-isto kao danas.
- Uz `OwnWords`, u situaciji gdje se smije pisati (`ai_entry_for(..).free_text`, tj. ima lika), **klik** na ponuđeni
  odgovor ljudskog hotela vrijedi `min(q, 4)`. Bez lika klik ostaje do 5. Roboti se ne ograničavaju (🟨).
- **Prepisan ponuđeni odgovor = klik tog odgovora**: kvaliteta `min(q odgovora, 4)`, mehanike tog odgovora.
- **AI ne stigne** (neutralna presuda) uz `OwnWords`: kvaliteta **4** (ADR-047 t. 4), mehanike šutnje kao danas.
- Mentor i izvještaj kruga znaju da je ocjena ograničena; student vidi napomenu uz ponuđene odgovore.
- Balans: put „klik najbolji" i lanac **najbolji (vlastiti) > klik najbolji > srednji > prepušteno > šutnja**.

**Zašto (nalaz):** Leon 15.10. (ADR-047): klik na najbolji danas daje 5 kao i najbolji vlastiti odgovor
(`crates/game/src/resolve.rs:51-58`: `quality: a.quality`), pa student koji ne piše ništa ne gubi. Nacrt §2 opisuje
današnje stanje; retci u njemu su od 15.10. i ovdje su ponovno provjereni (vidi Files).

**Files** (retci provjereni na 89b1d41):
- Create (a): `crates/game/src/own_words.rs` (pravilo i provjera prepisivanja), `crates/game/tests/answer_rule.rs`.
- Modify (a):
  - `crates/game/src/lib.rs:37` uz `pub use command::MANDATORY_SITUATIONS;` dodati `pub mod own_words;` i izvoz konstanti.
  - `crates/game/src/event.rs:15-36` (`GameCreated`: novo polje iza `situation_counts` :33-35, uzor isti `serde` atributi);
    `:152-166` (`SituationResult.capped`, uzor `delegated` :162-165); `:230-247` (`VerdictRecorded.copied`, uzor `meta` :243-244).
  - `crates/game/src/command.rs:29-35` (`GameSettings.answer_rule`), `:58-62` (`new_game`: `answer_rule: None`),
    `:81` i `:105-115` (`new_game_with`: `model.answer_rule(..)?` i zapis u `GameCreated`).
  - `crates/game/src/model.rs:153-169`: uz `situation_counts` nova `answer_rule` (isti obrazac: v1 + `Some` = `InvalidSettings`).
  - `crates/game/src/game.rs:88-108` (polje `answer_rule` u `Game`), `:143-154` (čitanje u `created()`: `None` → `Classic`),
    `:195-209` (literal), `:463-466` (uz `situation_counts()` getter `answer_rule()`).
  - `crates/game/src/resolve.rs:46-58` (klik: `min(q, CLICK_MAX_QUALITY)` kad `click_capped`; `SituationOutcome` s tom
    kvalitetom umjesto `from_answer`), `:59-89` (slobodni: `capped` iz `v.copied`), ostale grane `capped: false`.
  - `crates/game/src/verdict.rs:17-18` (`NEUTRAL_QUALITY` ostaje 3 za `Classic`), `:208-234` (`record_verdict`: provjera
    prepisivanja), `:237-256` (`neutral_verdict`: 4 uz `OwnWords`; prepisano → kao klik).
  - `crates/game/src/ai_input.rs:57-80` (`DebriefSituation.capped`, `copied`, `skip_serializing_if` kao `delegated` :70-72),
    `:287-298` (punjenje).
  - `crates/game/src/mentor.rs:154-180` (`SituationFact.capped`: `rename = "ogranicena_ocjena"`, `skip_serializing_if`),
    `:1063-1080` (punjenje; `:1076-1078` neutralni tekst „srednju ocjenu 3/5" ovisi o kvaliteti). **mentor.rs je dug na
    točno 1399 redaka** (`scripts/check-kod.mjs:23`): tekstove drži u `own_words.rs`; raste li ipak, unos DUG diže se u
    istom commitu i pregled to izričito vidi.
  - `crates/game/tests/v2_sealed.rs:28-35` (`Case.rule`), `:36-113` (jedan novi slučaj `m7-odgovara-klasicno`), `:168-176`
    (`GameSettings`), `:220-236` (`digest`: `"answer_rule": c.answer_rule`), novi test uz `:404-419`; `v2-golden.json`.
  - `crates/sim-hotel-v2/Cargo.toml:3` `2.4.0` → `2.5.0` i redak u `Cargo.lock`.
  - Svih 32 literala `GameSettings {` (`grep -rn "GameSettings {" crates`, bez `NewGameSettings`) dobiva `answer_rule: None`.
- Modify (b): `crates/balance/src/v2_answers.rs` (cijela: `:22-50` put, `:60-75` `offered_choice`, `:79-121` `respond`,
  `:124-153` `play`, `:156-165` `PathRow`, `:189-240` `path_report`, `:249-254` opis `ORDER_DEBT`, `:272-276`
  `STEP_NAMES`, `:279-330` brane, `:333-366` ispis); `crates/balance/src/v2_situations.rs:31-35` (`Situations::writable`);
  `crates/balance/src/v2.rs:27-30` (izvoz), `:501-520` (`path_season_v2`); testovi `crates/balance/tests/answer_paths_v2.rs`,
  `answer_gates_v2.rs`.
- Modify (c): `crates/app/src/views.rs:46-60` (`SituationItem.click_capped`), `:98-104` (`ResponseView.copied`),
  `:264-300` (`response_view`), `:326-336`; `crates/api/src/schema_situations.rs:13-47` (`SituationBody.click_capped`),
  `:118-126` i `:128-` (`ResponseBody.copied`), `:174-200`; `openapi.json` i `packages/api-client/src` (`corepack pnpm gen:api`);
  `apps/web/src/screens/situation/OfferedAnswers.tsx:60-67` (napomena ispod naslova), `Verdict.tsx:37-45` (napomena
  „vrijedi kao klik"); `packages/i18n/src/hr.ts:238-239`, `:255-263`; `apps/web/src/test/fixtures.ts:195`, `:232` (novo polje);
  test `apps/web/src/screens/situation/OfferedAnswers.test.tsx` (novi).
- Ne dirati: jezgre `sim-hotel` i `sim-hotel-v2/src` (q ostaje 1–5, `mechanic.rs:141`), `situation_effect_bp` (L1-13),
  `prompts/` (ni `presuda.v3`, ni `mentor.v4`; vidi Rizik), `talk.rs` (razgovor i predaja ostaju), zamrznute
  `crates/game/tests/fixtures_v2/v2-snapshot.json`, tekstove poruka brana balansa (format nizova), `docs/`.

**Interfaces:**
- Consumes: `Game::ai_entry_for` (`drawn.rs:445`), `Game::situation_round_for`, `HotelRecord.setup.controller`
  (`Controller::Human`), `FreeResponse.text` (`game.rs:46-50`), `Model::situation_counts` (obrazac), B3a
  `AnswerPath`/`respond`/`PathReport`, B3b `answer_path_failures`, `plan_ruler`, `share_bp`, `common::draft_with`
  (`crates/game/tests/common/mod.rs:84`), `common::v2_game_with` (`:239`).
- Produces:
```rust
// game::event
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerRule { Classic, OwnWords }
// GameCreated:    #[serde(default, skip_serializing_if = "Option::is_none")] pub answer_rule: Option<AnswerRule>,
// SituationResult: #[serde(default, skip_serializing_if = "is_false")] pub capped: bool,  // pravilo je SNIZILO ocjenu
// VerdictRecorded: #[serde(default, skip_serializing_if = "Option::is_none")] pub copied: Option<u8>, // prepisan odgovor
// game::command::GameSettings { kind, situations, pub answer_rule: Option<AnswerRule> }   // None = zadano (v2 OwnWords)
// game::own_words
pub const CLICK_MAX_QUALITY: u8 = 4;          // klik u situaciji s likom (ADR-047 t. 1)
pub const NEUTRAL_OWN_WORDS_QUALITY: u8 = 4;  // AI ne stigne (ADR-047 t. 4)
pub const COPY_MATCH_BP: u32 = 7_000;         // 🟨 prag prepisivanja
pub fn words(text: &str) -> Vec<String>;      // mala slova, č/ć→c, đ→d, š→s, ž→z, sve osim slova i brojki = razmak
pub fn copied_answer(text: &str, labels: &[&str]) -> Option<usize>;
// zajedničke riječi (multiskup) * 10_000 >= COPY_MATCH_BP * max(riječi teksta, riječi oznake);
// najveći udio pobjeđuje, kod jednakih manji indeks; prazan tekst ili oznaka → None
// game::Game
pub fn answer_rule(&self) -> AnswerRule;
pub fn click_capped(&self, hotel: HotelId, code: &str) -> bool; // OwnWords ∧ Controller::Human ∧ free_text
// game::DebriefSituation: capped: bool, copied: bool (oba skip_serializing_if Not::not)
// game::mentor::SituationFact: #[serde(rename = "ogranicena_ocjena", skip_serializing_if = "Option::is_none")]
//   pub capped: Option<&'static str>,
// balance (b)
AnswerPath::ClickBest            // ALL = [Best, ClickBest, Middle, Delegated, Silent, NoSituations]; ime "klik najbolji"
pub const CLICK_MAX_QUALITY: u8 = 4;  // zrcalo igre, čuva ga test
pub struct PathRow { pub profile, pub end: [i32; 6], pub best_vs_silent: [i32; 3], pub click_vs_silent: i32,
                     pub steps: [i32; 4] }   // najbolji−klik, klik−srednji, srednji−prepušteno, prepušteno−šutnja
impl Situations { pub(crate) fn writable(&self, item: &RoundItem) -> bool }
// primjerak: predložak `d.template` u `content.situation_templates` ima `character`; sidro: `content.ai[code].free_text`
// app/api (c): SituationItem.click_capped / SituationBody.click_capped: bool; ResponseView.copied / ResponseBody.copied: bool
```
- Pravila ocjene uz `OwnWords` (`Classic` = točno današnji kod):

| put | kvaliteta | mehanike | `capped` |
|---|---|---|---|
| klik, ljudski hotel, situacija s likom | `min(q, 4)` | odgovora | kad `q > 4` |
| klik bez lika, ili robot | `q` | odgovora | ne |
| slobodni, AI presuda, nije prepisan | iz presude (do 5) | `nearest_answer` | ne |
| slobodni, prepisan odgovor `i` (AI ili neutralno) | `min(q_i, 4)`, `nearest_answer = i`, `copied = i` | odgovora `i` | kad `q_i > 4` |
| slobodni, neutralno, nije prepisan | **4** (`Classic` 3) | šutnje (`unanswered`), 0 € | ne |
| prepušteno / šutnja | 2 / 1 | šutnje | ne |

**Testovi čuvara:**
PRAVILO-a — `crates/game/tests/answer_rule.rs` (nova; v2 igra preko `common::v2_game_with`, sjeme 42, tjedno; pomoćne
`first_free(g, me)` (situacija s likom) i `first_closed(g, me)` (bez lika) idu kroz krugove; test pada ako ne nađe):
1. `new_v2_game_writes_own_words_rule`: zadano `Some(OwnWords)`; `Some(Classic)` se zapiše; v1 (`Snapshot::read`) `None`;
   v1 uz `Some(_)` → `GameError::InvalidSettings`.
2. `log_without_rule_reads_as_classic`: `GameCreated.answer_rule = None` u dnevniku → `g.answer_rule() == Classic`.
3. `click_with_character_is_capped_at_four`: klik najboljeg (q 5) u situaciji s likom → u `RoundResolved` `quality 4`,
   `capped: true`; napredak manji nego u istoj igri s `Classic`.
4. `click_without_character_keeps_five`: najbolji klik bez lika → 5, `capped: false`.
5. `own_words_best_keeps_five`: `submit_response(.., "Odgovaram gostu svojim riječima.")` + `draft_with(5, najbolji)` → 5.
6. `copied_best_answer_counts_as_click`: tekst = oznaka najboljeg velikim slovima, bez interpunkcije i kvačica →
   `record_verdict(draft_with(5, 0))` daje `copied: Some(najbolji)`, `nearest_answer: Some(najbolji)`, `quality 4`.
7. `neutral_verdict_is_four_under_own_words`: `neutral_verdict` → 4, `nearest_answer: None`; uz `Classic` → 3.
8. `classic_rule_keeps_old_scoring`: testovi 3, 6 i 7 uz `Some(Classic)` → 5, `copied: None`, 3 (karakterizacija).
9. `robot_clicks_are_not_capped`: robot u situaciji s likom i odgovorom q 5 → 5 (traži kroz krugove i sjemena 1..20).
10. U istoj datoteci, kroz javni `game::own_words`: `copied_answer_normalizes_text`, `copied_answer_needs_seventy_percent_of_both_sides`
    (oznaka + 50 % svojih riječi → `None`; ispremetane riječi → `Some`, 🟨), `copied_answer_prefers_highest_share_then_lowest_index`.
11. `debrief_marks_capped_and_copied` (u `crates/game/tests/situations_ai_input.rs`): `DebriefSituation.capped`/`copied`;
    mentor činjenica `ogranicena_ocjena` postoji samo uz `capped`; neutralni tekst kaže „4/5" uz `OwnWords`.
12. `crates/game/tests/v2_sealed.rs`: `digest_reacts_to_answer_rule` (slučaj `m7-odgovara` uz `Classic` i `OwnWords` →
    različit otisak); `digest_cases_cover_rounds_profiles_and_answer_paths` traži `capped` u barem jednom slučaju.
RED: novi tipovi i polja ne postoje (E0432/E0560). `v1_frozen`, `fixtures_sealed` i `mentor_v1_frozen` ostaju zeleni bez
izmjene (dokaz da `Classic` i `skip_serializing_if` čuvaju bajtove).

PRAVILO-b — `crates/balance/tests/answer_paths_v2.rs` i `answer_gates_v2.rs`:
1. `path_constants_and_names`: šest putova i imena `["najbolji", "klik najbolji", "srednji", "prepušteno", "šutnja",
   "bez situacija"]`; `mirrors_match_the_game` dodaje `CLICK_MAX_QUALITY == game::own_words::CLICK_MAX_QUALITY`.
2. `offered_choice_only_for_best_and_middle`: `ClickBest` nudi isti indeks kao `Best`.
3. `answer_paths_in_game_match_balance` (A4 obrazac): igra uz `OwnWords`; `ClickBest` = klik najboljeg; `Best` = u
   situaciji s likom `submit_response` (vlastiti tekst) + `record_verdict` (5 kriterija, `nearest` = najbolji), bez lika
   klik; trag i napredak jednaki balansu za sve profile, tjedno i mjesečno.
4. `writable_matches_game_free_text`: za sjeme 42 svaki primjerak i sidro: `Situations::writable` == `ai_entry_for(..).free_text`.
5. `click_steps_are_gates`: korak najbolji − klik 0 → „B3 redoslijed: A najbolji prema klik najbolji 0.00 na kraju sezone
   (cilj više od 0)"; isto za klik najbolji − srednji. `order_gate_fails` prelazi na četiri koraka.
6. `weight_band_stays_on_own_words_best`: težina se mjeri na `best_vs_silent` (put `Best`, ADR-047 t. 3); `click_vs_silent`
   izvan pojasa ne ruši ništa (samo ispis). `clean_report_passes_all_b3_gates` ostaje zelen.
7. `report_prints_answer_paths`: zaglavlje sa „klik najbolji" i redak težine klika po profilu.

PRAVILO-c — `crates/app/tests/answer_rule.rs` (nova): `situations_view_marks_click_capped` (s likom `true`, bez lika
`false`, uz `Classic` sve `false`) i `response_view_marks_copied`. Vitest `apps/web/src/screens/situation/OfferedAnswers.test.tsx`
(nova): napomena „Klikom dobivaš najviše 4 od 5. Za punih 5 odgovori gostu svojim riječima." vidi se samo uz
`click_capped: true`; u `Response.test.tsx` uz `copied: true` vidi se „vrijedi kao klik".

**Koraci:**
- PRAVILO-a, čuvar: (1) testovi 1–12 (i `copied_answer` kroz javni `game::own_words`, bez koda graditelja);
  (2) svih 32 literala `GameSettings` dobiva `answer_rule: None`, a `Case` u pečatu polje `rule`; (3) commit RED
  s popisom grešaka prevođenja.
- PRAVILO-a, graditelj: (1) tip, polja i čitanje (`Game::created`); (2) `click_capped` i grana klika u `resolve.rs`;
  (3) `copied_answer` i `record_verdict`/`neutral_verdict`; (4) `DebriefSituation`, `SituationFact`; (5) `cargo test
  --workspace`: postojeći v2 testovi koji klikaju q 5 u situaciji s likom mogu pasti. Graditelj smije SAMO dodati
  `answer_rule: Some(AnswerRule::Classic)` testu koji ne govori o pravilu (svaki navede u izvještaju s razlogom);
  promjenu očekivanih brojki vraća čuvaru. (6) `CORE_VERSION` 2.5.0, pa pečat (naredba niže); (7) sve provjere.
- PRAVILO-b, čuvar pa graditelj: `ClickBest` u `respond` (kvaliteta `min(q, 4)` samo kad `writable`, inače kao `Best`),
  `writable` se prosljeđuje iz `play` (`sits`); `PathRow`, `STEP_NAMES` (četiri para; opis `ORDER_DEBT` s novim indeksima,
  B3b minor „indeks ≥ 3" time nestaje), ispis; paritet. Brojke izmjeriti `--check` 1000 tjedno i mjesečno.
- PRAVILO-c, čuvar pa graditelj: `click_capped` = `g.click_capped(hotel, code)`, `copied` = `v.copied.is_some()`;
  DTO, `corepack pnpm gen:api`; napomena ispod naslova u `OfferedAnswers` (`text-sm text-muted`, `p`, samo uz
  `s.click_capped`); `verdict.copied` u `Verdict.tsx`; fiksture.

**Provjere** (svaki dio, cijeli popis iz `.github/workflows/ci.yml`; izlazi u `.superpowers/sdd/2026-10-14-popravak-temelja/pravilo-provjere/`):
`cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs`
· `node scripts/probe-mutacija.mjs` · `git diff --exit-code` (nakon commita) · `cargo sqlx migrate run --source
crates/store/migrations` + `cargo sqlx prepare --workspace --check` · `cargo test --workspace` · `cargo run -q -p balance
--release -- --seeds 100 --check` i `--seeds 1000`, oba `cmp` s osnovicom (v1 **bajt-isti**) · `--model v2 --round week
--seeds 1000 --check` i `--round month` · cargo deny samo u CI-ju · `corepack pnpm check:docs`, `test:docs`, `check:kod`,
`check:api`, `lint`, `typecheck`, `test` · `corepack pnpm --filter @hotelixsim/web build`.
- **v2 balans:** balans ne čita postavku igre (putovi su izričiti, roboti se ne ograničavaju), pa u a i c izlaz v2
  mora biti **bajt-isti** (`cmp` s osnovicom). U b se mijenja **samo blok B3**: novi stupac, novi korak, redak težine
  klika. Brojevi puta „najbolji" i težina (B3 težina) ostaju isti (`diff` pokazuje samo retke bloka B3). Očekivanje
  (🟨, mjeri se u cigli): klik najbolji − šutnja ≈ 75–85 % od najbolji − šutnja (lik ima ~40 od 55 predložaka i S1, S2),
  tjedna težina klika za B ≈ 37–42 % uz 48,9 % za vlastiti. To je broj koji troši L1-13 (ADR-047 t. 3: klik ≈ 15 % kad
  vlastiti ≈ 20 %).

**CORE_VERSION i pečat:** brojke se mijenjaju → **2.4.0 → 2.5.0** u PRAVILO-a (🟨). Za isti ulaz nova v2 igra daje
drugi obračun (zadano `OwnWords`), a pečat A3 igra slučajeve sa zadanim postavkama i mora vidjeti `answer_rule` u otisku.
Stari dnevnik bi se čitao kao `Classic` i dao isto, ali `check_version` (`model.rs:142-151`) ga ionako odbija:
**Leonove probne v2 igre opet daju 409** (vlasnik izlaza C1). Naredba (provjerena, `v2_sealed.rs:259`):
`HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest`; pečat 2.4.0 ostaje u
`v2-golden.json`, dodaje se 2.5.0. PRAVILO-b i c ne diraju igru, pa ne dižu verziju. B4c ne diže verziju; L1-13 diže sljedeću.

**Rizik i odluke 🟨:**
1. **Zadano `OwnWords` za nove v2 igre** (ADR-047 to traži; profesorov izbor dolazi s ekranom igre u PLATFORMI).
2. **Roboti se ne ograničavaju**: ne mogu pisati, samo su za demo; zato paritet A4 i mjere robota ostaju isti. Cijena:
   u demu robot koji klikne najbolje dobije 5, student 4. U razredu (ADR-037) robota nema.
3. **Prag prepisivanja 70 %** riječi s obje strane, riječi kao skup (redoslijed ne spašava). Sinonimi prolaze; to hvata
   sudac tek u C3 (`presuda.v4`).
4. **Prepisivanje se gleda pri upisu presude** (`record_verdict`, `neutral_verdict`), ne u `submit_response`: tekst ostaje
   za profesora, AI se zove kao danas, a događaj presude nosi istinu (`copied`, ocjena 4), pa ekran, izvještaj i mentor
   kažu isto. Nacrt je spominjao `talk.rs`; ovo je jednostavnije i ne mijenja API predaje.
5. **AI ne stigne → 4 s mehanikama šutnje (0 €)**: student ne plaća trošak koji nije izabrao. Pisac s ispadom AI-ja prođe
   malo bolje od klika najboljeg (isti korak, bez troška); student ispad ne može izazvati.
6. **Vrijedi tjedno i mjesečno**, i u obveznim situacijama.
7. **`capped` samo kad je ocjena stvarno snižena** (klik q 3 nije „ograničen").
8. **AI upute se ne mijenjaju**: mentor dobiva rečenicu u činjenici `ogranicena_ocjena` (kao `neutralna_ocjena`), pa
   `mentor.v4` ostaje; `presuda.v3` ostaje (prepisivanje ne gleda AI).
9. Više pisanja = više poziva suca (Sonnet) i lika (Haiku): trošak izmjeriti iz `AiMeta` prije pilota (BACKLOG).
10. Nepoznat broj postojećih v2 testova (game, app, api `student_sees.rs`) pada zbog zadanog `OwnWords` (korak 5 a).

**Leonov dio:** sve je odlučeno (ADR-047). Pretpostavke 1–6 student osjeti, ali nemaju pravu alternativu ili su tehničke;
voditelj ih zapisuje kao ADR 🟨 na `/kraj` (bez ankete). Ako Leon želi drukčije, mijenja se jedna konstanta ili postavka.

**Review Focus kandidati:**
1. Student kopira ponuđeni odgovor i doda pola rečenice svojih riječi: prolazi li kao vlastiti (prag 70 % s obje strane).
2. Prepisan odgovor + AI ispad: ide li u „klik" (4, mehanike odgovora) prije pravila „neutralno 4" (0 €).
3. Robot i čovjek u istoj igri: cap samo čovjeku; razred (ADR-037) bez robota.
4. Stari dnevnik bez `answer_rule` i v1 igra: bajt-isti obračun i ulaz mentora (`v1_frozen`, `mentor_v1_frozen`).
5. Profesor na pregledu neutralne presude vidi 4, ne 3; tekst „neutralna ocjena 4/5" je točan.
6. Napomena uz ponuđene odgovore ne smije se pokazati u situaciji bez lika (S3, S4 i ~15 predložaka).
