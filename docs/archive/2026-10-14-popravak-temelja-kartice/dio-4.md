# 2026-10-14-popravak-temelja-kartice — dio 4

Arhiva (zamrznuto). Indeks: [2026-10-14-popravak-temelja-kartice.md](../2026-10-14-popravak-temelja-kartice.md).

### Task B3: Mjera „udio situacija" na zadanom planu — B3a mjerač i ispis, B3b brane modela

**Podjela:** B3a (mjerač, ispis, paritet s igrom; bez Leona) → **anketa Leonu s brojkama iz B3a** → B3b (brane u
`failures_v2`). Zajedno ~9–12 h, pa ne stanu u jednu granu od ~1 dana. **B1a se ne spaja prije B3b** (faza-a-review I1).
Redoslijed (voditelj, 15.10.): B2a → B2b → **B3a** → anketa → **B3b** → B1a → B1b → B4a–c. B3 mjeri tek nakon B2a/B2b, jer B2a mijenja mjesečni udio situacija.

**Zašto (nalaz):**
- REVIZIJA_TEMELJA §4 #3 i izvještaj 03 nalaz 1. Stara mjera `|Δ| / (|Δ| + |bez|)` dijeli s robotovim napretkom bez
  situacija (`crates/balance/src/v2_situations.rs:217-264`). Zato pada kad se robotu promijeni plan.
- Izvještaj 03 nalaz 3 i Review Focus 2 plana: balans ne poznaje prepuštanje (q = 2) ni šutnju. Sezonu studenta koji
  prepusti sve dopušteno nitko nije vrtio.
- faza-a-review **I1**: od A1 nijedna brana modela ne mjeri koliko situacije teže, a B1 mijenja baš njih.
- Leonova odluka 15.10. (`a6-dijagnoza.md` §3 i §Opcije 5, `za-dokumentaciju.md` „## A6"): trošak odgovora pada u F
  isti tjedan, a korist od idućeg. Zato mjerač gleda **više krugova, po vrsti kruga**, a ne jedan krug.

**Je li B3 promjena brojki igre? Ne (provjereno u kodu).**
- Igra obračunava kroz `Model::resolve` (`crates/game/src/resolve.rs:150-160`), ne kroz sezonu iz jezgre.
- Jezgra se samo preuređuje: `run_season_from` (`crates/sim-hotel-v2/src/season.rs:302-361`) postaje omotač nove
  funkcije, a ponašanje ostaje isto. `run_season_from` zovu balans i budžet u igri (`crates/game/src/model.rs:638-646`,
  bez situacija). Dokaz da se ništa nije promijenilo: `game_parity_v2`, `equivalence`, `v2_sealed` i balans bajt-isti.
- Zato **nema dizanja `CORE_VERSION`** i nema novog pečata.
- Izlaz balansa: v1 je bajt-isti (`cmp`); v2 je isti osim **novih redaka**. Poruke postojećih brana ostaju doslovno iste.

---

### Task B3a: Mjerač putova odgovora na zadanom planu + paritet s igrom

**Veličina:** M, 5–7 h · **Ovisi o:** A1, A4 (spojeni), **B2a i B2b spojeni** (B2b dira isti `crates/balance/src/v2.rs`; mjeri se nakon B2a) · **Nije paralelno
s:** B2, B1 ni B4 (red je strogo slijedan) · **Model:** sonnet (opus ako paritet padne) ·
**Prevodi Rust:** da

**Cilj:** Balans v2 za svaki profil i sjeme vrti **isti zadani plan** (`PeriodPlan::default_for`, plan studenta koji
ništa ne mijenja) i **isto sjeme**. Mijenja se samo kako se rješavaju situacije, pa nastaje pet putova:
1. **najbolji** — odgovor najviše kvalitete;
2. **srednji** — `middle_index`, isti kao probe;
3. **prepušteno** — svaka dopuštena situacija ide voditelju (q 2), a na obvezne student šuti;
4. **šutnja** — q 1, mehanike `unanswered`;
5. **bez situacija**.

Ispis pokazuje napredak po putu i razliku najbolji − šutnja nakon 1 kruga, nakon N krugova i na kraju sezone. Test
dokazuje da igra s čovjekom koji igra te putove daje isto što i balans. Time je zatvoren Review Focus 2.

**Files:**
- Modify `crates/sim-hotel-v2/src/season.rs:302-361`: nova `run_season_responding`, uz `RoundItem` i `Responder`.
  `run_season_from` postaje omotač. Primjerci kruga (`:321-339`) nose i svoj `DrawnSituation` (klon).
- Modify `crates/sim-hotel-v2/src/lib.rs:38-42`: izvoz `RoundItem`, `Responder`, `run_season_responding`.
- Create `crates/balance/src/v2_answers.rs`, a u `crates/balance/src/lib.rs:5-6` dodati `mod v2_answers;`.
- Modify `crates/balance/src/v2_situations.rs:108-121`: uz `with` nova `responding`, istog oblika, koja zove
  `run_season_responding`.
- Modify `crates/balance/src/v2.rs`:
  - `SituationReport` (`:55-66`) dobiva polje `paths: PathReport`;
  - u `run_v2` (`:552-563`) dolazi `paths: path_report(...)`;
  - iza `robot_season_v2` (`:453-473`) dolazi `pub fn path_season_v2`;
  - u `render_v2` (`:862-864`) ide `push_answer_paths` odmah iza `push_situations`.
  - Ponovni izvoz: `pub use crate::v2_answers::{AnswerPath, MANDATORY_SITUATIONS, offered_choice};`.
- Create `crates/balance/tests/answer_paths_v2.rs`.
- **Ne dirati:**
  - `crates/game/**` (igra je istina, P8), `content/**`, `crates/sim-hotel/**`, `CORE_VERSION`;
  - `failures_v2` i `robot_measures_v2`, kao i stari ispis „udio situacija u napretku" i poruku „J-3 udio situacija"
    (bajt-isti izlaz);
  - `main.rs:82` „Sve provjere prolaze." Taj redak dijeli i v1, a v1 mora ostati bajt-isti (m2: ostaviti).

**Interfaces:**
- Consumes: `PeriodPlan::default_for`; `SituationOutcome::{from_answer, unanswered}`; `DrawnSituation.weight`;
  `crate::middle_index` (`lib.rs:328-333`); `Situations` (`v2_situations.rs`); `share_bp` (`:217-222`);
  `progress_pts`; u testu `game::{MANDATORY_SITUATIONS, Game, ...}` kao u `game_parity_v2.rs`.
- Produces (jezgra):
```rust
/// Situacija kruga za odgovarača sezone; `drawn` je primjerak generatora (važnost određuje poredak u igri), sidro ga nema.
#[derive(Clone, Copy)]
pub struct RoundItem<'a> { pub template: &'a SituationTemplate, pub drawn: Option<&'a DrawnSituation> }
/// Ishodi kruga, jedan po stavci i istim redom; „samo nagradu" (S-4) primjenjuje jezgra.
pub type Responder<'r> = dyn FnMut(&Period, &[RoundItem]) -> Result<Vec<SituationOutcome>, ResolveError> + 'r;
pub fn run_season_responding(profile: Profile, kind: PeriodKind, cat: &Catalog, source: SeasonSituations,
    refs: &RefTable, planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    responder: &mut Responder) -> Result<SeasonResult, ResolveError>;
```
- Produces (balans):
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerPath { Best, Middle, Delegated, Silent, NoSituations } // ALL redom; name(): najbolji/srednji/prepušteno/šutnja/bez situacija
pub const MANDATORY_SITUATIONS: usize = 3; // zrcalo game::MANDATORY_SITUATIONS (command.rs:25)
pub const DELEGATED_QUALITY: u8 = 2;       // zrcalo game resolve.rs:13 (nije izvezen; čuva ga paritet)
pub fn horizon(kind: PeriodKind) -> usize; // 🟨 tjedno 4 kruga, mjesečno 2
pub fn offered_choice(path: AnswerPath, t: &SituationTemplate) -> Option<usize>; // Best/Middle → Some
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRow { pub profile: Profile,
    pub end: [i32; 5],            // medijan napretka na kraju sezone, redom AnswerPath::ALL (stotinke)
    pub best_vs_silent: [i32; 3], // medijan razlike po sjemenu: nakon 1 kruga, nakon horizon(kind), kraj sezone
    pub steps: [i32; 3] }         // medijan razlike po sjemenu na kraju: najbolji−srednji, srednji−prepušteno, prepušteno−šutnja
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathReport { pub kind: PeriodKind, pub horizon: usize, pub rows: Vec<PathRow> }
pub fn path_season_v2(content: &content::Content, profile: Profile, path: AnswerPath, seed: u64, kind: PeriodKind)
    -> Result<SeasonResult, ResolveError>;
```

**Pravila putova** (`respond(path, items)` u `v2_answers.rs`):
- *Najbolji:* odgovor najviše kvalitete; kod jednake kvalitete onaj s manjim indeksom.
- *Srednji:* `middle_index`.
- *Šutnja* i *bez situacija:* `SituationOutcome::unanswered(t)`.
- *Prepušteno:* poredak se gradi kao u igri. Prvo idu sidra redom, pa primjerci po `weight` silazno, stabilno
  (`game/src/drawn.rs:237-241`).
  - Prvih `MANDATORY_SITUATIONS` u tom poretku su obvezne (`command.rs:227-236`) i na njih se šuti.
  - Ostale dobivaju `SituationOutcome { code, mechanics: t.unanswered.clone(), quality: DELEGATED_QUALITY }`
    (`resolve.rs:82-95`).
  - 🟨 Voditelj postoji uvijek: v2 ima otvoren odjel (`drawn.rs:495-522`). Paritet to provjerava.
- Svi putovi koriste isti planer, `|s, p| Ok(PeriodPlan::default_for(s, p, cat)?)`, i isti `Situations::responding`
  sa sjemenom `seed`.
- Put *bez situacija* koristi `SeasonSituations::Fixed(&[])`.

**Ispis** (novi redci iza bloka J-3, prije „== Ponovljivost"). Brojke su `hundredths`:
```text
== Situacije na zadanom planu (B3): isti plan i sjeme, mijenja se samo odgovor ==
  napredak na kraju sezone, medijan (najbolji / srednji / prepušteno / šutnja / bez situacija):
    A 3.10 / 1.20 / -0.40 / -1.10 / 0.20
  najbolji − šutnja, medijan razlike po sjemenu (1 krug / 4 kruga / kraj sezone):
    A -0.19 / 0.81 / 4.20
  težina za anketu: prema rasponu plana (Brižni − Štediša bez situacija) 31,2 %; prema zadanom planu bez situacija 95,0 %
```
- Prvi postotak u zadnjem retku je `share_bp(best_vs_silent[2], raspon)`. Raspon je medijan „bez" Brižnog minus medijan
  „bez" Štediše iz `sr.shares` za isti profil.
- Drugi postotak je `share_bp(end[0] − end[4], end[4])`.
- Brojke u primjeru su izmišljene, samo za oblik.

**Testovi čuvara** (`crates/balance/tests/answer_paths_v2.rs`; pomoćnici prema `game_parity_v2.rs`):
```rust
/// Zrcalo pravila igre: balans prepušta iste situacije kao igra.
#[test] fn mirrors_match_the_game() { assert_eq!(balance::v2::MANDATORY_SITUATIONS, game::MANDATORY_SITUATIONS); }

/// Review Focus 2: igra s čovjekom (utor HOTEL_SLOT, bez plana → zadani plan) koji igra put `path` daje
/// iste situacije, pokazatelje i napredak po krugu kao `path_season_v2`.
#[test] fn answer_paths_in_game_match_balance() {
    for kind in [PeriodKind::Week, PeriodKind::Month] { for profile in Profile::ALL {
        for path in [AnswerPath::Best, AnswerPath::Middle, AnswerPath::Delegated, AnswerPath::Silent] {
            let g = human_game(42, kind, profile, path);   // Controller::Human, Actor::Guest(guest)
            let s = path_season_v2(&content, profile, path, 42, kind).unwrap();
            assert_eq!(game_trace(&g), season_trace(&s, &content.situations), "{kind:?} {} {path:?}", profile.code());
        } } }
}
```
- U `human_game` se po krugu prolazi `g.situations_of_hotel(me, r)`:
  - `Best` i `Middle`: `g.choose_answer(me, code, offered_choice(path, t))`;
  - `Delegated`: `g.delegate_situation` kad `!g.situation_mandatory(me, code)`;
  - `Silent`: ništa.
  - Nakon toga `close_round` i `resolve_job` → `Done`, kao `v2_sealed.rs:86-119`.
- Uz to se provjerava da test nije prazan: put `Delegated` ima barem jedan `SituationDelegated` u **mjesečnoj** igri.
  U tjednoj igri s 3 situacije u krugu prepuštanja možda nema, i to se zapisuje.
- `paths_share_plan_and_seed`: za isto sjeme `Best` i `Silent` imaju isto stanje i iste primjerke u prvom krugu
  (`s.periods[0].0.before`, `s.drawn[0]`).
- `report_prints_answer_paths`: `run_v2` (3 sjemena, tjedno) → `paths.rows.len() == 3`, a `render_v2` sadrži zaglavlje
  „== Situacije na zadanom planu (B3)" i redak za A, B i C.
- **RED:** prevođenje pada na `unresolved import` (`AnswerPath`, `path_season_v2`, `MANDATORY_SITUATIONS`).
- **Karakterizacija:** paritet može biti zelen odmah. Ako je crven, uzrok je jedan od dva šava. Oba se popravljaju na
  strani **balansa**:
  - poredak prepuštanja;
  - stezanje razine cijene `plan_for_resolve` (`model.rs:330-353`), kad zadani plan nije unutar granica.

  Ako je uzrok u igri, **stani (BLOCKED)**.

**Koraci:**
1. Čuvar u `../hotelixsim-b3a` (grana `popravak/b3a-putovi`) napiše testove i zapiše RED.
2. Voditelj prije prve gradnje spremi osnovicu u `.superpowers/sdd/2026-10-14-popravak-temelja/osnovica-b3/`:
   `v1-100.txt`, `v1-1000.txt`, `v2-week.txt`, `v2-month.txt`.
3. Graditelj dodaje jezgru: `run_season_responding`.
   - Petlja `:340-347` gradi `items`, zove `responder(p, &items)` i provjerava
     `answers.len() == items.len()` (inače `BadChoice`).
   - Zatim, kao danas, `reward_only` za `no_penalty`.
   - `run_season_from` zove je s odgovaračem koji za svaku stavku poziva `chooser`, istim redom kao danas.
4. Graditelj dodaje balans: `v2_answers.rs` (putovi, `path_report`, ispis), `Situations::responding`, polje i ispis u
   `v2.rs`. Sva sjemena su `opts.seeds`, kao `shares`.
5. Zabilježi vrijeme izvođenja balansa prije i poslije. Ako naraste više od ~60 %, javi u izvještaju; ništa se ne skraćuje
   samo od sebe.
6. Commit „Popravak B3a: mjerač putova odgovora na zadanom planu" s izričitim putanjama.

**Provjere** (u stablu, svaka zasebno `; echo EXIT=$?`, `CARGO_TARGET_DIR` zajednički, najviše 2 agenta prevode):
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `node scripts/check-core-deps.mjs`
- `node scripts/probe-mutacija.mjs` (dira jezgru)
- `git diff --exit-code`
- `cargo sqlx migrate run --source crates/store/migrations` i `cargo sqlx prepare --workspace --check`
- `cargo test --workspace` (baza `hotelixsim_agenti`), posebno `game_parity_v2`, `equivalence`, `v2_sealed` i
  `answer_paths_v2`
- `cargo run -q -p balance --release -- --seeds 100 --check` → `cmp` s osnovicom (**bajt-isto**); isto za
  `--seeds 1000`
- `--model v2 --round week|month --seeds 1000 --check` → EXIT 0, a
  `diff osnovica-b3/v2-week.txt novo.txt | grep -c '^<'` → **0**: samo dodani redci, i to samo blok B3. Isto mjesečno.
- `cargo deny`: samo CI.
- `corepack pnpm check:docs`, `test:docs`, `check:kod` (`v2.rs` ostaje ispod 1000 redaka), `check:api`, `lint`,
  `typecheck`, `test`
- `corepack pnpm --filter @hotelixsim/web build`
- `git diff model-v2/sloj1 --stat -- crates/game content crates/sim-hotel` → prazno.

**Rizik i odluke (🟨):**
- **Isto sjeme nije isto što i iste situacije.** Generator gleda stanje hotela (`season.rs:325-331`). Zato se od 2.
  kruga putovi mogu razići u izvučenim situacijama. Tako igra i radi, pa mjera mjeri sezonu kakvu bi student odigrao.
- **Prepušteno = šutnja na obveznima.** Razlika „prepušteno − šutnja" tako je točno vrijednost prepuštanja. Student
  koji obvezne riješi sam leži između „prepušteno" i „najbolji".
- **Horizont** (tjedno 4 kruga ≈ prvi mjesec, mjesečno 2) je mjerni izbor, ne broj igre.
- **Odnos prema A2 `silence_dominates`** (`crates/content/src/templates_principles.rs:224`): A2 pri učitavanju uspoređuje
  šutnju s **najlošijim** odgovorom jednog predloška. Usporedba ide polje po polje, s trenutnim učinkom i v1 mehanikom.
  B3 uspoređuje **najbolji** odgovor sa šutnjom kroz cijelu sezonu, u napretku i po vrsti kruga. Dvije provjere se
  dopunjuju:
  - A2 hvata predložak u kojem je šutnja Pareto-bolja;
  - B3 hvata klasu C iz A6, sezonu u kojoj šutnja dugoročno pobjeđuje, što A2 ne može vidjeti.

  A2 se ovdje ne mijenja. B1 ga prebacuje na v2 mehaniku (I2).

**Leonov dio:** nema za gradnju. Ispis B3a je ulaz u anketu B3b.

---

### Task B3b: Brane modela za situacije (`failures_v2`)

**Veličina:** S–M, 3–5 h · **Ovisi o:** B3a spojen **i Leonov odgovor na anketu** · **Slijedi ga:** B1a · **Model:**
sonnet · **Prevodi Rust:** da · **Mora biti spojen prije B1a** (I1)

**Cilj:** `--check` pada kad situacije postanu nevažne, obrnute ili presudne na zadanom planu, a robot pritom ne igra
nikakvu ulogu.

**Files:**
- Modify `crates/balance/src/v2_answers.rs`: `pub(crate) fn answer_path_failures(f, r: &PathReport, ...)`, konstante
  pojasa i, ovisno o odgovoru ankete, ravnalo.
- Modify `crates/balance/src/v2.rs:645-647`: u `if let Some(sr)` dodati `answer_path_failures(&mut f, &sr.paths, ...)`.
- Prema odgovoru ankete, u ispisu B3 ostaje samo izabrani redak težine. Ostali redci B3 se ne mijenjaju.
- Create `crates/balance/tests/answer_gates_v2.rs`.
- **Ne dirati:** postojeće poruke brana, mjere robota i `CORE_VERSION`.

**Brane** (poruke točno ovako; nijedna ne sadrži „udio situacija", „štediša", „pravednost", „brižni robot" ni
„tjedno ≈ mjesečno", jer `balance_v2.rs:63-81` ih traži samo u mjerama robota):
- `"B3 odgovori: {p} najbolji odgovori prema šutnji {Δ} nakon {h}. kruga (cilj više od 0)"`, iz `best_vs_silent[1]`.
- `"B3 odgovori: {p} najbolji odgovori prema šutnji {Δ} na kraju sezone (cilj više od 0)"`, iz `best_vs_silent[2]`.
- `"B3 redoslijed: {p} {a} prema {b} {Δ} na kraju sezone (cilj više od 0)"` za svaki `steps[i] <= 0`. Lanac je najbolji
  > srednji > prepušteno > šutnja (Leon P4, `resolve.rs:12`).
- `"B3 težina: {p} situacije nose {pct} (cilj {lo}–{hi})"`. Formula je iz ankete. Pojas `WEIGHT_BAND_BP` 🟨 je
  **(500, 5000)** do L1-13: hvata „nevažne" i „presudne" — **samo ako** težine izmjerene u B3a (nakon B2a, tjedno i mjesečno) leže unutar 5–50 % s marginom (ruling voditelja 15.10.); inače B3b brani težinu samo smjerom (Δ > 0, `best_vs_silent`/redoslijed) bez pojasa, a pojas stiže u L1-13. Kalibracija L1-13 ga sužava na 15–25 % (ADR-042 t. 15).
- **Isti krug (`best_vs_silent[0]`) nije brana.** Leon je 15.10. odlučio: trošak odmah, ugled kasnije.

**Testovi čuvara** (`week_report()` s 3 sjemena kao u `robot_measures_v2.rs:16-26`; u svakom testu filtar `x.starts_with("B3")`):
- `answer_gates_fail_check`: `rows[0].best_vs_silent[2] = -100`. `failures_v2` tada sadrži „B3 odgovori" i „na kraju
  sezone", a `robot_measures_v2` ne sadrži „B3".
- `late_payoff_gate_uses_horizon`: `best_vs_silent[1] = 0`. Poruka sadrži „nakon 4. kruga".
- `same_round_cost_is_not_a_gate`: `best_vs_silent = [-500, 100, 100]`, `steps = [100; 3]`, težina u pojasu → nema „B3".
- `order_gate_fails`: `steps[1] = 0` → „B3 redoslijed" i „srednji prema prepušteno".
- `weight_gate_fails`: težina 9000 bp → „B3 težina".
- **RED:** prevođenje (`answer_path_failures` ne postoji) i tvrdnje.
- **Karakterizacija:** pravi sadržaj, 1000 sjemena, tjedno i mjesečno → `--check` EXIT 0. Ako današnji model padne na
  „B3 odgovori" ili „B3 redoslijed", to je **nalaz o igri, ne o testu**. Tada **stani (BLOCKED)**: voditelj nosi brojke
  Leonu (kao A6) i ne slabi branu.

**Provjere:** isti popis kao B3a.
- Izlaz balansa: v1 bajt-isti.
- v2 se smije razlikovati samo u retku težine (izabrano značenje). Kad brane prolaze, nema novih redaka pod
  „Sve provjere prolaze."

**Rizik i odluke:**
- 🟨 Pojas 5–50 % je privremen i uvjetovan: gradi se samo ako brojke B3a, izmjerene nakon B2a, leže unutar njega s marginom; inače samo smjer (Δ > 0). Uži pojas sada bi srušio CI prije kalibracije L1-13.
- ADR s Leonovom definicijom „~20 %" piše voditelj u `/kraj`, ne graditelj.
- Stara mjera robota „J-3 udio situacija" ostaje u ispisu, s drugim imenom nego nova („B3 težina"), pa ih se ne
  miješa. Hoće li stara otići, odlučuje L1-13 uz Leona.

**Leonov dio — anketa prije B3b (AskUserQuestion, jedno pitanje; uz nju ide ispis B3a, tjedno i mjesečno):**

> **Što znači da situacije nose „~20 % napretka"?**
> Da bismo to mogli provjeravati, mjerimo ovako. Isti hotel, isti plan (zadani, onaj koji student dobije ako ništa ne
> mijenja), isti tijek sezone. Jedina razlika: student na svaku situaciju odgovori najbolje, ili ni na jednu ne
> odgovori. Razlika u napretku na kraju sezone je **težina situacija** (B3a ispis: „najbolji − šutnja"). Pitanje je:
> 20 % **od čega**?
>
> 1. **(preporučeno) Od razlike koju čini dobar ili slab plan, izmjerene jednom i zapisane kao ravnalo.** Danas je
>    razlika između plana Brižnog i Štediše, bez situacija, oko 8,5 / 6,8 / 10,3 boda (A / B / C) tjedno i
>    8,1 / 6,8 / 10,9 mjesečno. Ti brojevi upisuju se u kod kao ravnalo, s datumom.
>    *Posljedica:* provjera ne ovisi o robotima i ne skače. Ravnalo se mijenja samo tvojom odlukom, u kalibraciji L1-13.
>    Na 20 % najbolji odgovori donose otprilike četvrtinu ravnala (~2 boda).
> 2. **Od iste razlike, ali izmjerene svaki put robotima Brižni i Štediša.**
>    *Posljedica:* sama prati promjene igre. Ali kad se robot promijeni, provjera može pasti iako situacije nitko nije
>    dirao. To je točno ono što smo 14.10. popravljali (A1), a roboti su samo za demo.
> 3. **Od napretka studenta sa zadanim planom bez situacija** (stara formula, ali bez robota).
>    *Posljedica:* zadani plan danas daje napredak blizu nule, pa postotak skače prema 100 % (vidi B3a ispis, drugi
>    postotak). Ista krhkost kao stara mjera (B Štediša 73–91 %).

## Review Focus kandidati
1. **Tjedni krug s 3 situacije nema što prepustiti.** U tjednoj igri put „prepušteno" može biti jednak šutnji, pa
   „prepušteno > šutnja" tjedno može biti 0 bez kvara. Ako B3b tjedno padne samo na tom koraku, to je mjerni rub: pitati
   voditelja, ne slabiti branu.
2. **Zrcala pravila igre u balansu:** 3 obvezne, q 2 i poredak po važnosti. Paritet ih čuva samo za sjeme 42. B2 je već
   preselio broj situacija u postavke igre (spojen prije B3a), pa paritet B3a mora uključiti i put „prepušteno" s tim brojem.
3. **Slobodni odgovor s presudom** (glavni AI put) nije među putovima. Vlasnik je C3, preko mjerila presude.
4. **Vrijeme balansa u CI-ju:** 5 sezona × 3 profila × 1000 sjemena po vrsti kruga više.

### Task B4: Prepis oznaka odgovora (55 predložaka + sidra S1–S4) i uvjeta kvarova — podijeljeno B4a / B4b / B4c

**Veličina:** B4a M, ~6 h · B4b M, ~4 h · B4c M, ~4 h (zbroj ~14 h; cijela B4 ne stane u jedan dan, vidi „Podjela")
· **Ovisi o:** A2 (načela, popis duga), **B1b** (`condition_of` za uvjete kvarova u B4c; voditelj 15.10.: B4 ide nakon B1b) · B4b i B4c ovise o B4a (alat i čuvarski testovi) · **Nije paralelno
s:** B1, B2, B3 (red faze B: B2a → B2b → B3a → B3b → B1a → B1b → **B4a → B4b → B4c** → L1-13) · **Model:** B4a sonnet (kod + tekst), B4b/B4c sonnet (tekst) · **Prevodi Rust:** B4a da (testovi), B4b/B4c samo
testovi sadržaja (`cargo test -p content`).

**Cilj:** Student ne smije prepoznati najbolji odgovor po duljini, redoslijedu rečenice ni broju koraka. Zato se prepisuju
**oznake** (`answers[].label`) svih 55 predložaka (213 oznaka) i sidara S1–S4 (16 oznaka) tako da su unutar 1,5× (P3) i da dobri
sastojci ne stoje samo u najboljem odgovoru. Kvarovi P-KLIMA i P-PRAONICA (B4a) te P-BRAVE, P-CURENJE-VODE i P-DIZALO (B4c, preko B1b `condition_of`) dobivaju uvjet lošeg stanja Smještaja (P6, ADR-042 t. 7); P-INSPEKCIJA postaje vanjski događaj (`external_cause`, Leon 15.10.).
Dug A2 za duljine pada 55 → 0, a za uvjete kvarova na 0 (`SilenceNotBetter` 16 nije u B4, ide u L1-13). Kad bude uveden prikaz s izmiješanim odgovorima (L1-13b, ADR-042 t. 10), miješanje je sigurno.

**Zašto (nalaz):** [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §6 B4 · [faza-a-review.md]
(`.superpowers/sdd/2026-10-14-popravak-temelja/faza-a-review.md`) tablica trijaže: A2 „dug 55/55 duljina", m1, m4,
`DeptPlaceholder` u `character.*`, Review Focus 5 (sidra), m5 (A6 bira `answer: 0`). Brojke danas (provjereno čitanjem JSON-a):
najbolji odgovor je **najdulji u 54 od 55** predložaka i ima **najviše zareza i veznika „i” u 55 od 55**; omjer duljina: min 2,52 (P-INSPEKCIJA), medijan
4,28, max 7,44 (P-SLOBODNI-DANI); sidra S1 3,1× · S2 5,0× · S3 10,3× („Odbiti" = 6 znakova) · S4 3,4×. U 53 od 55 predložaka
kvaliteta ide silazno (iznimke P-VJENCANJE, P-KRIZA-OSOBLJA). Kvaliteta odgovora = broj kriterija (`TemplateAnswer::quality`,
`templates.rs:295`), a svaki predložak ima točno jedan odgovor kvalitete 5.

## Što B4 mora jamčiti da je miješanje kasnije sigurno (za L1-13b)

1. **Duljina ne odaje:** svi odgovori jednog predloška i sidra unutar 1,5× (kod: `LABEL_RATIO_PCT`, `templates_principles.rs:17`),
   a graditelj cilja ≤ 1,40× (rezerva jer se broji sirovi tekst s `{sobe}`, `{ponuda_pct}`…; prikazani je kraći ili dulji za nekoliko znakova).
2. **Građa ne odaje:** najbolji nije pojedinačno najdulji ni s pojedinačno najviše zareza u više od 50 % ( 🟨 prag) očišćenih
   predložaka (test T4). Ne smije biti sastojka koji ima samo najbolji (npr. „sutra nazvati", „obrazložiti", „provjeriti kasnije").
3. **Redoslijed i kvaliteta se ne diraju:** B4 ne mijenja ni indeks odgovora, ni `criteria`, ni `mechanics`, ni `unanswered` (događaji i roboti
   pamte izvorni broj). Danas je najbolji u svim predlošcima i sidrima **na indeksu 0**; to je istina do L1-13b.
4. **Točno jedan odgovor kvalitete 5** u svakom predlošku i sidru (danas vrijedi; test T5) — jedino tako „izvorni broj najboljeg" postoji.
5. **Napomena za A6 scenarij (a):** `crates/api/tests/student_sees.rs:121` šalje `"answer": 0` jer je to najbolji danas. Kad L1-13b izmiješa
   prikaz, indeks 0 postaje nasumičan i test bi tiho mjerio drugo. L1-13b mora scenariju (a) dati **izvorni broj najboljeg odgovora**: API ne
   smije otkriti kvalitetu, pa test taj broj čita iz sadržaja (`Snapshot::read_v2(&repo_dir())`, odgovor sa 5 kriterija za šifru
   predloška bez nastavka kruga, npr. `P-KLIMA.t03` → `P-KLIMA`) i šalje ga kao `answer`. B4 to **ne** mijenja (zabranjeno), samo to ovdje piše.

## Podjela (zašto i kako)

Jedna cigla = ~230 oznaka, pristojan tekst se ne piše brže od ~1 oznake u 3–4 minute, a Leon mora pregledati tekstove. Grana ≤ 1 dan i
Leonov pregled po porciji (≤ ~85 oznaka) traže tri cigle. Dug duljina pada stepenasto (ceilings u `v2_principles.rs` test 2 ide dolje).

| cigla | sadržaj | predlošci | oznaka | dug duljina poslije |
|---|---|---|---|---|
| **B4a** | alat + testovi + sidra S1–S4 + uvjeti P-KLIMA i P-PRAONICA + prva porcija: odjeli `rooms` i `*guest_facing` | 15 | 57 + 16 sidra | 55 → 40 |
| **B4b** | `hotel`, `*staffed`, `admin`, `sales`, `maintenance`, `parking` | 22 | 85 | 40 → 18 |
| **B4c** | `restaurant`, `pool_bar`, `spa`, `rooftop_bar`, `lobby_bar` + uvjeti kvarova P-BRAVE, P-CURENJE-VODE, P-DIZALO (njihove oznake su u B4b) + P-INSPEKCIJA `external_cause` | 18 | 71 | 18 → 0 |

Šifre porcija (jedino mjesto gdje su popisane):
- **B4a:** P-KLIMA, P-BOJLER, P-KRIZA-OSOBLJA, P-RANI-DOLAZAK, P-NESTALA-STVAR, P-LJUBIMAC, P-BUCNA-SOBA, P-OVERBOOKING, P-RED-NA-RECEPCIJI,
  P-PRAONICA, P-UCJENA-RECENZIJOM, P-DOTRAJALO, P-POHVALA, P-PRISTUPACNOST, P-NETOCNA-RECENZIJA.
- **B4b:** P-NOVINAR, P-KONGRES, P-VAL-RECENZIJA, P-TOPLINSKI-VAL, P-OLUJA, P-REGATA, P-GOST-POZLILO, P-DIZALO, P-NESTANAK-STRUJE,
  P-DIJETE-IZGUBLJENO, P-POVISICA, P-BOLOVANJE, P-SLOBODNI-DANI, P-OSPORENA-NAPLATA, P-RACUN-GRESKA, P-TERMINAL, P-PREPORUKA, P-LAST-MINUTE,
  P-RADNICI-TVRTKE, P-BRAVE, P-CURENJE-VODE, P-PARKING-STETA.
- **B4c:** P-NOVI-LJUDI, P-INSPEKCIJA, P-ALERGIJA, P-POSEBNA-PREHRANA, P-MANJAK-INVENTURA, P-DOBAVLJAC, P-PIJANI-GOST, P-PROSIDBA,
  P-BAZEN-PUMPA, P-LEZALJKE, P-SUSJEDI-BAZEN, P-SPA-TERMIN, P-SPA-TECAJ, P-WELLNESS-PAKET, P-VJENCANJE, P-ROOFTOP-BUKA, P-LOBBY-SUKOB, P-LOBBY-KAVA.

## Files

**Create (B4a):**
- `crates/content/tests/v2_b4_labels.rs` — testovi T1–T7 (dolje)
- (privremeno, ne ulazi u git) skripta za pregledni dokument u scratchpadu; izlaz `.superpowers/sdd/2026-10-14-popravak-temelja/b4a-pregled-tekstova.md`
  (`b4b-…`, `b4c-…`); `.superpowers/` je u `.gitignore`, dokument se ne commitira nego Leonu preda voditelj.

**Modify:**
- B4a: `content/hotel/situations.json` (samo `answers[].label` u S1–S4); `content/hotel/v2/situation_templates.json` (oznake porcije +
  `when.dept_condition_pts` za P-KLIMA i P-PRAONICA); `crates/content/src/templates_principles_debt.rs` (makni 15 šifara iz `LabelLengths`,
  P-KLIMA i P-PRAONICA iz `BreakdownCondition`); `crates/content/src/templates_principles.rs:112-130` (m4, vidi T6) i
  `crates/content/src/lib.rs:21` (izvezi `LABEL_RATIO_PCT` ako nije izvezen — T1 ga koristi); `crates/balance/src/main.rs:60`
  (`Snapshot::read_v2(dir)` → `Snapshot::read_v2_checked(dir)`, m1; poruke brana ostaju doslovno iste);
  `crates/content/tests/v2_principles.rs:56-60` (stropovi test 2: `LabelLengths` 55 → 40, `BreakdownCondition` 6 → 4; u B4c → 0).
- B4b: `situation_templates.json` (22 predloška, samo oznake); debt `LabelLengths` −22; strop 40 → 18.
- B4c: `situation_templates.json` (18 predložaka, oznake; uz to `condition_of` + `when.dept_condition_pts` za P-BRAVE, P-CURENJE-VODE, P-DIZALO i `external_cause: true` za P-INSPEKCIJA); debt `LabelLengths` −18 (lista postaje prazna: `&[]`), `BreakdownCondition` −4 (3 uvjeta + P-INSPEKCIJA), stropovi → 0.

**Ne dirati:** `criteria`, `mechanics`, `unanswered`, `effect_band`, redoslijed `answers`, `title`, `description`, `rubric`, `character.*`,
`params`, `chance`, `repeat`, `external_cause` (pet već postavljeno u A2; jedini novi je P-INSPEKCIJA u B4c); `crates/sim-hotel*`, `crates/game`, `crates/engine`; `CORE_VERSION`;
`crates/*/tests/fixtures*/**` i `crates/game/tests/fixtures_v2/**` (zamrznuti snimci nose stare oznake — i treba tako ostati); `prompts/`; web.
Ako oznaka ne može biti poštena bez izmjene opisa/rubrike, graditelj to **prijavi** u izvještaj, ne mijenja.

## Interfaces
**Consumes:** `content::{Snapshot, Principle::*, principle_violations, check_principles_with, KNOWN_VIOLATIONS, fingerprint_value, repo_dir}`;
`TemplateAnswer::quality`. **Produces:** isti javni tipovi; novo samo `pub use templates_principles::LABEL_RATIO_PCT` (ako treba). Format JSON-a se ne mijenja.

## Testovi čuvara (`crates/content/tests/v2_b4_labels.rs`; pomoćne funkcije kopirati iz `v2_principles.rs:7-37`)
Čuvar prvo (korak 1) iz **neizmijenjenih** datoteka izračuna konstantu `SKELETON_SHA` ispod i upiše je u test.

- **T1 `anchors_labels_within_ratio`** — za S1–S4 iz `Snapshot::read_v2(&repo_dir())["situations"]`:
  `max·100 <= min·LABEL_RATIO_PCT` (znakovi: `chars().count()`). **RED** (3,1 / 5,0 / 10,3 / 3,4). Zeleno nakon B4a.
- **T2 `only_labels_and_listed_conditions_change`** (karakterizacijski, zelen od početka, ostaje zelen kroz B4a–c):
  ```rust
  fn skeleton(v: &Value) -> String { // kostur bez oznaka i bez dopuštenih uvjeta
      let mut v = v.clone();
      for a in v["situations"].as_array_mut().unwrap().iter_mut()
          .flat_map(|s| s["answers"].as_array_mut().unwrap().iter_mut()) { a.as_object_mut().unwrap().remove("label"); }
      for t in v["v2"]["situation_templates"]["templates"].as_array_mut().unwrap() {
          for a in t["answers"].as_array_mut().unwrap() { a.as_object_mut().unwrap().remove("label"); }
          if matches!(t["code"].as_str(), Some("P-KLIMA" | "P-PRAONICA")) {
              t["when"].as_object_mut().unwrap().remove("dept_condition_pts"); }
      }
      content::fingerprint_value(&v)
  }
  assert_eq!(skeleton(&v2_value()), SKELETON_SHA, "B4 smije mijenjati samo oznake, uvjete kvarova (5 predložaka) i `external_cause` P-INSPEKCIJE");
  ```
  🟨 ključ `"situations"` u `Snapshot::to_value()` provjeri čuvar. Test se briše na početku L1-13 (kalibracija mijenja brojke).
- **T3 `batch_labels_have_no_debt`** — tri testa `b4a_…`, `b4b_…`, `b4c_…` s konstantnim popisom šifara porcije (gore):
  `repo_codes(LabelLengths)` ne smije sadržavati nijednu šifru porcije. **RED** do kraja pripadne cigle. Uz to test 2 sa spuštenim stropovima.
- **T4 `best_answer_is_not_a_tell`** — za sve predloške koji **nisu** u `KNOWN_VIOLATIONS[LabelLengths]` (očišćeni) i za sidra, ako su očišćena
  (T1 zelen): udio predložaka u kojem je najbolji odgovor strogo najdulji (znakovi) ≤ 50 %, i udio u kojem ima strogo najviše zareza ≤ 50 %.
  Kad nema očišćenih, test prolazi trivijalno. **RED** tek kad čuvar u test unese sidra kao „očišćena" nakon B4a; za porcije pada kad graditelj
  preslika staru građu. 🟨 prag 50 % (slučajno bi bilo ~25–35 %).
- **T5 `exactly_one_best_answer_per_template_and_anchor`** — karakterizacijski: točno jedan odgovor s 5 kriterija (sidra: `quality == 5`).
- **T6 `duplicate_name_in_a_list_is_counted_once`** (m4) — u `v2/names.json` snimku `guest.f` s 20 unosa od kojih jedan duplikat → `NamesPerList`
  za `names.guest.f` s „19 različitih". **RED** (`principle_violations` danas broji `v.len()`). Popravak: brojati `BTreeSet`. Repo imena su
  svi različiti (provjereno: 6 popisa × 20 jedinstvenih), pa repo ostaje zelen.
- **T7 `dept_placeholder_checked_in_character_fields`** — `{odjel}` u `character.brief`/`opening`/`fallback_reply` bez „odjel*" ispred ili zagrade → `DeptPlaceholder`.
  Karakterizacijski (kod `dept_placeholder_misuse` to već radi, test nedostaje).
- **T8 (B4a) `klima_and_praonica_need_bad_rooms_state`** — `when.dept_condition_pts.below` postoji; `BreakdownCondition` ne krši ni jedan; ostala četiri
  (P-INSPEKCIJA, P-DIZALO, P-BRAVE, P-CURENJE-VODE) ostaju u dugu do B4c. **RED** do B4a.
- **T9 (B4c) `breakdown_conditions_via_condition_of`** — P-BRAVE, P-CURENJE-VODE i P-DIZALO imaju `condition_of` i `when.dept_condition_pts.below`, P-INSPEKCIJA ima `external_cause`; `BreakdownCondition` nema nijednu šifru u dugu. **RED** do B4c.

## Koraci
**B4a**
1. **Čuvar:** izračuna `SKELETON_SHA`, piše T1–T8, spušta stropove (test 2). `cargo test -p content --test v2_b4_labels --test v2_principles` → RED samo T1, T3a, T4*, T6, T8, test 2.
2. **Graditelj, kod:** T6 (m4, `BTreeSet`), m1 (`balance/main.rs:60`), po potrebi izvoz `LABEL_RATIO_PCT`.
3. **Graditelj, tekst:** prvo sidra pa porcija (pravila ispod). **Commit 1: samo oznake** (sidra + 15 predložaka). Balans mora biti **bajt-isti** (`cmp`).
4. **Graditelj, uvjeti — commit 2 zasebno:** `when.dept_condition_pts: {"below": 6500}` na P-KLIMA i P-PRAONICA (🟨 prag kao P-BOJLER, ista skupina `rooms`;
   P-KLIMA već ima pojačanje `below 5000` ×3 — ostaje). Ovdje se brojke v2 **mijenjaju** (rjeđi kvar) pa se `cmp` ne radi, nego brane `--check`.
5. **Pregledni dokument** za Leona (format niže), pa izvještaj; voditelj ga pokaže Leonu.
**B4b / B4c:** čuvar upiše porciju (T3) i spusti strop; graditelj piše oznake po pravilima, pa dokument; balans **bajt-isti** prema stanju poslije B4a (commit oznaka).
**B4c, commit 2 zasebno (uvjeti):** `condition_of` + `when.dept_condition_pts: {"below": 6500}` 🟨 za P-BRAVE, P-CURENJE-VODE, P-DIZALO i `external_cause: true` za P-INSPEKCIJA; brojke v2 se mijenjaju, pa umjesto `cmp` vrijede brane `--check` (kao B4a korak 4).

## Pravila pisanja oznaka (graditelj; ovo je i sadržaj Leonova pregleda)
- Svi odgovori istog predloška: ≤ 1,40× najkraćeg, a najkraći ≥ 55 znakova (da nije „Odbiti"). Praktično 55–100 znakova; S3: „Odbiti" → rečenica jednake težine.
- Slabiji odgovori su **vjerodostojni**: konkretna radnja s pravom manom u sadržaju (vratiti cijeli novac, čekati tehničara), ne karikatura; bez riječi koje
  samo označavaju lošu ocjenu („samo", „bez objašnjenja", „ignorirati"). Najbolji ne smije imati posebne markere dobrote.
- Sastojke „iskrena isprika / rješenje odmah / naknada / uzrok / provjera kasnije" **raspodijeli** na barem dva odgovora; broj zareza razlikuje se najviše za 1.
- Isti oblik rečenice (glagol u infinitivu, ista razina formalnosti). Ništa novo u `{…}`; postojeća mjesta (`{sobe}`, `{noci}`, `{ponuda_pct}`) ostaju.
- **Značenje se mora slagati s mehanikom** (hitan popravak ⇒ „odmah pošalje tehničara"; `compensation: 0` ⇒ ne obećava naknadu). Dokument ih stavlja jedno do drugog.
- Kvaliteta ostaje: `criteria` se ne pomiču. Kriteriji koje najbolji odgovor „pokriva" ostaju u `rubric` (presuda ih čita odande), pa kraći tekst ne gubi ocjenu.

## Leonov pregled tekstova — što graditelj predaje i što blokira spajanje
**Datoteka** `b4X-pregled-tekstova.md` (X = a/b/c): za svaki predložak `### P-KLIMA — naslov (odjel)` pa tablica **slijepog pregleda**: slova A–D redom
od stalnog nasumičnog redoslijeda (sjeme = šifra), stupci: slovo · tekst · znakova; ispod redak `Moj izbor najboljeg: __` i `Odluka: __` (✔ ili ✎ + zamjena).
Na kraju odjeljak **Ključ**: slovo · stari tekst · novi tekst · broj kriterija (5/3/1/0) · mehanika u jednoj rečenici · omjer duljine prije → poslije;
sažetak: koliko predložaka je ≤ 1,40×, koliko je najbolji još najdulji, imena predložaka bez prijavljenog problema. Skripta provjeri da su prisutne **sve** šifre porcije.
**Što blokira:**
- **Spajanje cigle u `model-v2/sloj1`** blokiraju samo automatske provjere (niže) + postojanje dokumenta sa svim šiframa porcije. Leonov pregled **ne blokira**
  (pretpostavka 🟨: grana nije `main`, ispravak je jeftin jer T2 jamči „samo oznake"). Predložak sa zapisom 🟨 TEKST ČEKA PREGLED ostaje u `docs/records/` (voditelj).
- **L1-14 (Leon igra)** čeka da je B4a–c spojena; **L1-15 (`sloj1` → `main`)** blokira svaki prazan `Odluka:` u trima dokumentima.
- Leonovi ispravci dolaze kao jedna mala cigla „B4-ispravci" (samo oznake, ista pravila, T2 i T4 zeleni).

## Provjere (svaka zasebno, `; echo EXIT=$?`)
- Prije prve izmjene spremi u scratchpad izlaz tri balansa (osnovica B4a): v1 `--seeds 100 --check`, v2 tjedni i mjesečni `--seeds 1000 --check`.
- `cargo test -p content --test v2_b4_labels --test v2_principles --test v2_templates` → `ok`, 0 failed
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs`
  · `git diff --exit-code` (nakon prevođenja) · `cargo sqlx migrate run --source crates/store/migrations` pa `cargo sqlx prepare --workspace --check`
  (kao CI) · `cargo test --workspace` (uključuje `v2_sealed`, `v1_frozen`, `fixtures_sealed`: moraju ostati zeleni jer se zamrznuti snimci ne diraju)
- `cargo run -q -p balance --release -- --seeds 100 --check`, `… --model v2 --round week --seeds 1000 --check`, `… --model v2 --round month --seeds 1000 --check`
- **`cmp` s osnovicom:** v1 balans **uvijek** bajt-isti (oznake nisu u obračunu; v1 je zamrznut). v2 (tjedni i mjesečni): **bajt-isti poslije commita 1 u B4a i u cijelim
  B4b/B4c**; **poslije commita 2 u B4a brojke se mijenjaju** — tada nema `cmp`, nego obje brane `--check` zelene i nove brojke u izvještaj.
- `cargo deny` samo CI · `corepack pnpm check:docs` · `test:docs` · `check:kod` · `check:api` · `lint` · `typecheck` · `test` · `corepack pnpm --filter @hotelixsim/web build`
  (web testovi imaju vlastite stare oznake u `apps/web/src/test/fixtures.ts`; ne mijenjaju se)
- Commit B4a: „Popravak B4a: oznake sidara i prve porcije unutar 1,5×, uvjet kvara za klimu i praonicu" (dva commita, vidi korak 3–4); B4b/B4c analogno.
- **Živi AI test i mjerilo** (`content/hotel/mjerilo/S1.json`, `S2.json`, presuda bira najbliži odgovor po oznaci) tek uz Leonov izričit OK; ne radi agent.

## CORE_VERSION i pečat (iz koda, ne iz plana)
- Pečat `crates/game/tests/v2_sealed.rs` igra nad **zamrznutim** snimkom `tests/fixtures_v2/v2-snapshot.json` (test 2 čuva njegov otisak), pa izmjena
  `content/hotel/**` **ne ruši** pečat i **ne traži** novi. `CORE_VERSION` (`crates/sim-hotel-v2/Cargo.toml`; do B4 je 2.4.0 iz B1a) se bira iz koda obračuna.
- Svaka igra nosi svoj snimak sadržaja u `GameCreated` (`event.rs:25`, otisak provjerava `fold`), pa stare igre i dalje igraju stare oznake i stare uvjete bez 409.
- Zaključak: B4 **ne diže** `CORE_VERSION` i **ne obnavlja** pečat. Plan (Global Constraints) kaže „svaka promjena koja mijenja brojke v2 igre diže verziju" — čitam to
  kao promjenu koda obračuna/izvlačenja; promjena sadržaja (uvjeti kvarova) mijenja brojke samo **novih** igara. **Ako voditelj to čita drukčije**, B4a diže na sljedeću verziju (2.5.0)
  i pečati novi otisak (`HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest`), Leonove probne igre tada daju 409. 🟨 za ADR.
- Zajednička kalibracija L1-13 ionako diže verziju; B4 je ne troši.

## Rizik i odluke (🟨)
- 🟨 **Sidra su u v1 datoteci** (`content/hotel/situations.json`) koju v2 koristi za S1–S4 (`Snapshot::read_v2` čita istu datoteku), pa se nove oznake vide i u v1. Brojke v1
  se ne mijenjaju (oznake nisu u obračunu; zamrznuti v1 fixturei imaju vlastitu kopiju), a `cmp` v1 balansa to dokazuje. Alternativa (zasebna v2 kopija sidara) traži izmjenu učitavanja
  i nije u B4. Leon 15.10.: zajednička datoteka.
- 🟨 **Prag poštene oznake** 1,40× (rezerva do 1,5×) i najkraći ≥ 55 znakova; 🟨 prag T4 50 %.
- 🟨 **Uvjet 6500** za P-KLIMA/P-PRAONICA; kalibracija ga može pomaknuti u L1-13. Dva predloška više ne dolaze kod dobro održane sobe: `DrawStats` brane J-3 (m3 iz faza-a-review)
  postaju djelomično ovisne o planu robota. Ako J-3 padne, graditelj **ne** popravlja broj, nego javlja (ovisnost se zapisuje uz branu, kao u B1).
- **P-DIZALO (`hotel`), P-BRAVE i P-CURENJE-VODE (`maintenance`) dobivaju uvjet u B4c:** `templates_check.rs:312` odbija uvjet odjela za `hotel`, a za `maintenance`
  bi gledao stanje održavanja, ne Smještaja (ADR-042 t. 7 kaže Smještaj). Zato B4 ide nakon B1b i koristi njegov „čijeg prostora" (`condition_of`). Dug `BreakdownCondition` (4 s P-INSPEKCIJOM) pada na 0 u B4c;
  P-INSPEKCIJA ide kao `external_cause` (Leon 15.10.).
- **Dug `SilenceNotBetter` (16) NIJE u B4:** nije tekst nego brojke (trošak/bodovi odgovora i šutnje); vlasnik P4 → L1-13 (voditelj 15.10.). „Dug pada na 0" u B4 znači oznake i uvjete kvarova, ne šutnju. Nakon B4c:
  `LabelLengths` 0, `BreakdownCondition` 0, `SilenceNotBetter` 16 (pada na 0 na kraju L1-13).
- Predložak s jednakim iznosima kvalitete (P-POHVALA [5,4,1]; P-VJENCANJE, P-KRIZA-OSOBLJA ne idu silazno) — redoslijed ostaje, miješanje to rješava.
- Vrijeme pisanja ovisi o sastavljanju 229 tekstova; ako graditelj prekorači porciju, B4c se dijeli po odjelu (restoran | bar/spa/lobby).

## Leonov dio (odgovoreno 15.10.)
1. **Duljina (P3):** najviše **1,5×** (Leon potvrdio; najbolji odgovor ~90 znakova umjesto ~230, detalji ostaju u rubrici).
2. **Sidra S1–S4:** prepisuju se u **zajedničkoj datoteci** (v1 i v2 vide nove tekstove, brojke iste); posebna v2 kopija otpada.
3. **P-INSPEKCIJA:** vanjski događaj, `external_cause: true` (opcija a); ide u B4c.
Preostaje samo pregled tekstova: pregled tekstova (niže) nije anketa nego dokument: A/B/C/D slijepi izbor najboljeg + `Odluka`.

## Review Focus kandidati
- Je li ijedna oznaka promijenila značenje u odnosu na mehaniku (npr. „odmah" uz `urgent: false`)? Dokument slijepog pregleda to ne hvata strojno; pregledati uzorak od 10 po porciji.
- Je li T2 stvarno ugrizao (probno promijeniti `criteria` jednog odgovora; test mora pasti) i T4 (vratiti staru oznaku na jedan predložak)?
- Vide li roboti/balans oznake igdje? (provjereno: `label` u `balance`, `sim-hotel*` ne postoji; ide samo u `ai_input`/`feed`/`mentor`.)
- Presuda (`presuda.v3`) bira najbliži odgovor po oznaci: kraći najbolji tekst može promijeniti odabir slobodnog odgovora; mjerilo S1/S2 prije L1-14 uz Leonov OK.
- A6 (a) i dalje zelen (`answer: 0` najbolji dok miješanje ne stigne); L1-13b mora promijeniti taj redak (točka 5 gore).

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
- **Zalijepljeno više ponuđenih odgovora** (kvar B13; Leon 16.10., sesija 7): tekst koji pokriva ≥ 2 ponuđene oznake (svaku ≥ 70 % njezinih
  **parova susjednih riječi**, 🟨; pregled PRAVILO-a C1: pojedinačne riječi lažno hvataju dobar vlastiti odgovor u S1, parovi 0/229 lažnih i 59/59 uhvaćenih) **nije odluka** → kvaliteta **3**, mehanike šutnje (`unanswered`, 0 €), `pasted: true`, `nearest_answer: None`; AI presuda se ne
  gleda (kod odlučuje, 0 tokena). Vrijedi uz **oba** pravila (`Classic` i `OwnWords`) jer je to kvar, ne pravilo; provjera prepisivanja ide
  **prije** provjere jednog prepisanog odgovora. Mentor i student vide „zalijepio si ponuđene odgovore, to nije tvoja odluka".
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
// VerdictRecorded: #[serde(default, skip_serializing_if = "is_false")] pub pasted: bool, // zalijepljeno ≥ 2 odgovora (B13)
// game::command::GameSettings { kind, situations, pub answer_rule: Option<AnswerRule> }   // None = zadano (v2 OwnWords)
// game::own_words
pub const CLICK_MAX_QUALITY: u8 = 4;          // klik u situaciji s likom (ADR-047 t. 1)
pub const NEUTRAL_OWN_WORDS_QUALITY: u8 = 4;  // AI ne stigne (ADR-047 t. 4)
pub const COPY_MATCH_BP: u32 = 7_000;         // 🟨 prag prepisivanja
pub fn words(text: &str) -> Vec<String>;      // mala slova, č/ć→c, đ→d, š→s, ž→z, sve osim slova i brojki = razmak
pub fn copied_answer(text: &str, labels: &[&str]) -> Option<usize>;
// zajedničke riječi (multiskup) * 10_000 >= COPY_MATCH_BP * max(riječi teksta, riječi oznake);
// najveći udio pobjeđuje, kod jednakih manji indeks; prazan tekst ili oznaka → None
pub fn pasted_answers(text: &str, labels: &[&str]) -> bool;
// B13: broj oznaka čije su riječi (multiskup) pokrivene tekstom ≥ COPY_MATCH_BP / 10_000 je ≥ 2 → true (🟨 prag i „dvije")
// game::DebriefSituation: pasted: bool; game::mentor::SituationFact: #[serde(rename = "zalijepljeni_odgovori", ...)] pub pasted: Option<&'static str>
// app/api (c): ResponseView.pasted / ResponseBody.pasted: bool; web Verdict.tsx: „Zalijepio si ponuđene odgovore — to nije tvoja odluka."
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
| slobodni, zalijepljeno ≥ 2 ponuđena odgovora (B13; AI ili neutralno; **oba pravila**) | **3**, `nearest_answer: None`, `pasted: true` | šutnje (`unanswered`), 0 € | ne (nije sniženje, nije odluka) |
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
13. `pasted_answers_score_neutral_three` (B13): tekst = sve ponuđene oznake spojene → `record_verdict(draft_with(5, 0))` daje `quality 3`,
    `pasted: true`, `nearest_answer: None`, `copied: None`, učinak šutnje (0 €); isto uz `Some(Classic)`; `neutral_verdict` s takvim tekstom isto.
    Kroz `game::own_words`: `pasted_answers_needs_two_covered_labels` (jedna oznaka + svoje riječi → `false`; dvije oznake → `true`;
    dvije oznake od kojih je jedna pokrivena 50 % → `false`, 🟨).
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

Cigla IZGLED provodi [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 11: izgled lika
situacije mora odgovarati njegovu **rodu, dobi i podrijetlu**. Leon 16.10. (sesija 7, anketa): ide **sada**, zasebno i usporedo
s PRAVILO-om; stil ostaje **crtić iz podataka** (mogućnost A nacrta [IZGLED_LIKA](../../architecture/IZGLED_LIKA.md)). Dva dijela:
**IZGLED-a** (app i API: lik dobiva tri polja) i **IZGLED-b** (web: `lookFor` ih poštuje). Jezgra, dnevnik, balans, pečat i
`CORE_VERSION` se ne diraju. Iz L1-13b time ispada „izgled lika po rodu (rod u `RoleFill`)".

### Task IZGLED: Izgled lika iz roda, dobi i podrijetla

**Veličina:** a ~2,5 h · b ~2 h · pregled ~0,5 h (ukupno ~5 h) · **Ovisi o:** ništa iz faze B (čita samo `content` snimak igre
i postojeće poglede) · **Paralelno s:** PRAVILO-a/b/c i banka ljudi K2, uz „najviše dva agenta prevode Rust" (IZGLED-a prevodi;
IZGLED-b ne). Sudar s PRAVILO-c: obje mijenjaju `schema_situations.rs`, `openapi.json`, `schema.d.ts` i `fixtures.ts` — tko
spaja drugi, ponovno pokreće `corepack pnpm gen:api` (bez ručnog spajanja generiranih datoteka) · **Model:** čuvar i graditelj
sonnet, pregled opus · **Prevodi Rust:** IZGLED-a da (`crates/app`, `crates/api`); IZGLED-b ne (samo pnpm).

**Cilj:**
- Uz svakog lika izvučenog primjerka API šalje `look`: **rod** (`f`/`m`), **dobnu skupinu** (`young`/`adult`/`senior` ili
  `null` kad nije poznata) i **podrijetlo** (`regional`/`foreign`). Isto i u razgovoru (`character_look`).
- Web: `lookFor(ime, look)` — rod određuje bradu, frizuru i naušnice; dob skupinu (i sijedu kosu starijih); podrijetlo raspon
  boje kože. Ime i dalje bira sve ostalo, pa isti lik uvijek izgleda isto.
- Gošća nikad nema bradu; gost nikad naušnice ni žensku frizuru; lik s hrvatskim imenom nikad tamnu put.
- Bez `look` (sidra S1–S4, v1, roboti-hoteli na ljestvici, stari klijent) sve ostaje **bajt-isto** kao danas.

**Zašto (nalaz):** `lookFor(name)` (`apps/web/src/avatar/look.ts:355-357`) zna samo ime: `lookFromSeed` bira kožu iz svih šest
tonova (`SEED_SKIN` :163), frizuru iz osam (`:164`, i „long"/„bun" za muškarca), bradu bez obzira na rod (`:189`). Rod se gubi na
putu: predložak ga ima (`content::Role.gender`, `crates/content/src/templates.rs:239-248`), jezgra po njemu bira ime
(`crates/sim-hotel-v2/src/situations/fill.rs:90`, `ctx.names.list(role.source, role.gender)`), ali `CharacterCardView`
(`crates/app/src/views.rs:74-77`) i `CharacterBody` (`crates/api/src/schema_situations.rs:50-54`) nose samo `name` i `role`.

**Najmanji šav (provjereno na a992973):** app već ima snimak sadržaja igre (`Game.content`, javno, `crates/game/src/game.rs:95`)
s popisom imena `content.situation_templates.names` (`content::Names`, `templates.rs:374-387`). Upit „u kojem je popisu ovo
ime" daje i **rod** i **izvor** (gost/osoblje/partner). Danas je **120 imena, nijedno u dva popisa** (provjereno čitanjem
`content/hotel/v2/names.json`); `check_names` (`crates/content/src/templates_check.rs:54-78`) jamči jedinstvenost samo unutar
jednog popisa, pa app za ime u dva popisa različitog roda vraća `None` (staro ponašanje), a test čuvara drži da takvog imena nema.
Rezerva bez promjene jezgre, ako imena prestanu biti jedinstvena (K5): predložak primjerka (`HotelRecord.drawn` → `drawn.template`
→ `Template.character.role` → `roles[key].gender/from`), sve javno; složenije (krug i primjerak), zato nije prvi izbor.

**Files** (retci provjereni na a992973):
- Create (a): `crates/app/src/look.rs` (tipovi i `character_look`, ~70 r.), `crates/app/tests/character_look.rs`.
- Modify (a):
  - `crates/app/src/lib.rs:3-10` `pub mod look;`, `:12-19` `pub use look::*;`.
  - `crates/app/src/views.rs:74-77` (`CharacterCardView.look: Option<CharacterLook>`), `:336-341` (punjenje:
    `look: character_look(&g.content, &c.name, &c.role)`).
  - `crates/app/src/talk.rs:13-24` (`ConversationView.character_look`), `:39-48` (punjenje istom funkcijom).
  - `crates/api/src/schema_situations.rs:50-54` (`CharacterBody.look`, `#[schema(required = true)]` kao `character` :22-23),
    novi `CharacterLookBody` i tri enuma uz `SituationKindBody` (:56-80, isti obrazac `From`), `:180-183` (punjenje).
  - **`crates/api/src/schema.rs` je dug na točno 1770 redaka** (`scripts/check-kod.mjs:21`) i ne smije narasti: `SpeakerBody`,
    `ChatLineBody`, `ConversationBody` i njegov `From` (`schema.rs:1154-1205`) sele u `schema_situations.rs` (ili novi
    `schema_talk.rs`), ponovno se izvoze u `schema.rs:16-19` (imena u OpenAPI-ju ista), a `DUG` u `check-kod.mjs:21` spušta se na
    novi broj redaka u istom commitu. `ConversationBody.character_look` dodaje se tek u novom domu.
  - `openapi.json:1614` (`CharacterBody`), `:1644` (`ConversationBody`) i `packages/api-client/src/schema.d.ts:420`, `:428` —
    **samo** kroz `corepack pnpm gen:api`.
- Modify (b):
  - `apps/web/src/avatar/look.ts`: novi tip `LookHint` i zamrznuti bazeni `HINT_*` iza `:179`; nova `lookFromHint`; `lookFor`
    (:355-357) prima drugi argument. Sada 357 r., nakon ~420 (granica upozorenja 600).
  - `apps/web/src/screens/SituationsScreen.tsx:64` → `lookFor(s.character.name, s.character.look)`.
  - `apps/web/src/screens/situation/Conversation.tsx:30` → `lookFor(c.character_name, c.character_look)`.
  - `apps/web/src/test/fixtures.ts:196`, `:216` (`look: null` — Petra i Marko su poznati likovi), `:265` (`character_look: null`).
  - `apps/web/src/avatar/avatar.test.tsx` (novi `describe`, postojeći testovi :32-109 netaknuti).
- Ne dirati: `crates/sim-hotel-v2`, `crates/game` (ni dnevnik ni poglede), `crates/content` (popis imena i provjere), `crates/balance`,
  `prompts/`, `content/` (nijedan JSON), `palette.ts` (nizovi samo dodavanjem; ovdje ništa), `Avatar.tsx`, `KNOWN_LOOKS`,
  `PLAYER_LOOK`, `MENTOR_LOOK`, `SEED_*` (:161-179), `lookFromSeed`, `Leaderboard.tsx:45` i `HotelCard.tsx:26` (ime hotela robota nije
  lik; ostaju na `lookFor(ime)`), `AvatarGallery.tsx`, `docs/` (osim ove kartice i tablice plana), `03-prijava.md` (Leonova izmjena).

**Interfaces:**
- Consumes: `Game.content.situation_templates: Option<content::SituationTemplates>` (`.names`), `content::Names` / `NameList`
  (`guest|staff|partner` × `f|m`), `content::Character { name, role }` (`crates/content/src/lib.rs:66-76`; `role` = opis iz
  `types`, `title` ili „gošća hotela", `templates_instance.rs:58-77`), `Game::ai_entry_for` (`drawn.rs:445`).
- Produces (konvencija API-ja: ključevi engleski `snake_case` kao `character_name`, `department`; vrijednosti enuma `snake_case`
  kao `SituationKindBody`; rod `f`/`m` kao ključevi `names.json` i `content::Gender`; dob = web `AGES` (`look.ts:25`)):
```rust
// app::look
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum LookGender { F, M }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum AgeGroup { Young, Adult, Senior }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Origin { Regional, Foreign }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterLook { pub gender: LookGender, pub age_group: Option<AgeGroup>, pub origin: Origin }
/// Ime u točno jednom rodu popisa imena snimka igre; inače None. Izvor: guest → Foreign, staff/partner → Regional.
/// Dob: ključna riječ u opisu uloge (age_hint), inače staff/partner → Adult, guest → None (web bira iz imena).
pub fn character_look(content: &content::Content, name: &str, role: &str) -> Option<CharacterLook>;
pub fn age_hint(role: &str) -> Option<AgeGroup>;
// riječi = mala slova, sve osim slova razmak; Senior: riječ počinje s „umirovljen" ili je „baka", „djed", „stariji",
// „starija", „starica", „starac"; Young: počinje s „mladić" ili „student", ili je „mlada", „mladi". Senior ima prednost.
// ostarjelom / starim (psom) NE pogađaju (cijele riječi, ne podnizovi).
// app views/talk
pub struct CharacterCardView { pub name: String, pub role: String, pub look: Option<CharacterLook> }
pub struct ConversationView { /* … */ pub character_look: Option<CharacterLook> }
// api
#[derive(Serialize, ToSchema)] pub struct CharacterBody { pub name: String, pub role: String,
    #[schema(required = true)] pub look: Option<CharacterLookBody> }
#[derive(Serialize, ToSchema)] pub struct CharacterLookBody { pub gender: GenderBody,
    #[schema(required = true)] pub age_group: Option<AgeGroupBody>, pub origin: OriginBody }
#[serde(rename_all = "snake_case")] pub enum GenderBody { F, M }            // "f" | "m"
#[serde(rename_all = "snake_case")] pub enum AgeGroupBody { Young, Adult, Senior }
#[serde(rename_all = "snake_case")] pub enum OriginBody { Regional, Foreign }
// ConversationBody: #[schema(required = true)] pub character_look: Option<CharacterLookBody>
```
```ts
// apps/web/src/avatar/look.ts
export type LookHint = { gender: "f" | "m"; age_group: Age | null; origin: "regional" | "foreign" };
export function lookFor(name: string, hint?: LookHint | null): AvatarLook;
// KNOWN_LOOKS[name] uvijek pobjeđuje; bez hinta = lookFromSeed(name) (bajt-isto); s hintom = lookFromHint(name, hint)
export function lookFromHint(name: string, hint: LookHint): AvatarLook;
// = lookFromSeed(name), pa iz NOVOG toka rng(`${name}|izgled`) fiksnim redom: skin, hair, facialHair, earrings, hairColor;
// skin: regional HINT_SKIN_REGIONAL = [0, 1, 6], foreign HINT_SKIN_FOREIGN = [0, 1, 2, 6] (palette.ts:3-12; 3, 4, 5, 7 tek iz banke)
// hair: f HINT_HAIR_F = ["long","ponytail","bun","bob","wavy","curly","side","short"]; m HINT_HAIR_M = ["short","buzz","side","slick","curly","wavy","none"]
// facialHair: f "none"; m r(3) === 0 ? pick(HINT_BEARD_M = ["stubble","mustache","beard","goatee","full"]) : "none"
// earrings: f r(2) === 0 ? pick(["studs","hoops"]) : "none"; m "none"
// age: hint.age_group ?? (iz lookFromSeed: young/adult); senior → hairColor 5 (sijeda), inače hairColor iz lookFromSeed
// Bazeni HINT_* su zamrznuti (samo dodavanje na kraj, kao SEED_*), svako izvlačenje troši rng i kad se ne koristi.
```

**Testovi čuvara:**
IZGLED-a — `crates/app/tests/character_look.rs` (nova; snimak `Snapshot::read(&repo_dir())`; v2 igra kao `start_v2` u
`crates/app/tests/instance_ai.rs:15-25`):
1. `names_are_unique_across_lists`: svako ime iz `names.json` je u točno jednom popisu (izvor × rod). Čuva pretpostavku šava.
2. `look_from_name_lists`: „Anna Müller" → `F`, `Foreign`; „Ivan Kovač" → `M`, `Foreign`; „Ana Jurić" (staff) → `F`,
   `Regional`, `Some(Adult)`; „Damir Tomić" (partner) → `M`, `Regional`, `Some(Adult)`.
3. `unknown_name_has_no_look`: „Petra Horvat" (sidro), prazno ime i `Content` bez predložaka (v1) → `None`.
4. `age_hint_reads_role_words`: „umirovljenica na godišnjem odmoru" → `Senior`; „baka koja je s unucima…" → `Senior`;
   „stariji gost koji teško hoda…" → `Senior`; „mladić koji je došao s prijateljima" → `Young`; „gost koji putuje s ostarjelom
   majkom" → `None`; „gošća koja putuje sama sa starim psom" → `None`; „gošća hotela" → `None`.
5. `situations_view_carries_look`: svaki `SituationItem` s likom iz primjerka (šifra s točkom) ima `character.look = Some`, rod
   jednak popisu imena; sidro S1/S2 `look = None`.
6. `conversation_view_carries_look`: `app::conversation` za isti primjerak daje isti `character_look` kao pogled situacija.
7. API (u postojećoj `crates/api/tests/student_sees.rs` ili novoj `character_look.rs`): JSON `character.look` ima ključeve
   `gender`, `age_group`, `origin` s vrijednostima `"f"|"m"`, `"young"|"adult"|"senior"|null`, `"regional"|"foreign"`.
RED: `app::look` i polja ne postoje (E0432/E0560). Postojeći testovi koji grade `CharacterCardView {…}` literalom dobivaju
`look: None` (čuvar ih nabroji u RED commitu).

IZGLED-b — `apps/web/src/avatar/avatar.test.tsx`, novi `describe("izgled iz podataka lika")`, 300 imena `lik-${i}`:
1. `bez hinta isto kao prije`: `lookFor(ime)` i `lookFor(ime, null)` === `lookFromSeed(ime)`; postojeći zamrznuti test „Ana Kovač"
   (:36-55) ostaje netaknut i zelen.
2. `poznati likovi pobjeđuju hint`: `lookFor("Petra Horvat", {gender: "m", …})` === `KNOWN_LOOKS["Petra Horvat"]`; `PLAYER_LOOK`,
   `MENTOR_LOOK` i `KNOWN_LOOKS` jednaki snimku (`toEqual` objekta prije izmjene).
3. `žensko ime nikad brada`: za sve `f` × dobi × podrijetla `facialHair === "none"`, `hair ∈ HINT_HAIR_F`.
4. `muško ime nikad naušnice ni ženska frizura`: `earrings === "none"`, `hair ∈ HINT_HAIR_M`; brada se pojavi barem jednom.
5. `dob iz hinta`: `age_group` zadan → `age` jednak; `senior` → `hairColor === 5`; `null` → `age ∈ ["young","adult"]`.
6. `podrijetlo bira raspon kože`: `regional` → `skin ∈ [0, 1, 6]`; `foreign` → `skin ∈ [0, 1, 2, 6]`; nikad 3, 4, 5, 7.
7. `isti ulaz uvijek isti izgled` i `izgled iz hinta je uvijek valjan` (`isValidLook`); raznolikost: 10 ženskih imena daju ≥ 4 frizure.
8. `zamrznut izgled iz hinta`: jedan `toEqual` snimak za `lookFor("Anna Müller", {gender: "f", age_group: null, origin: "foreign"})`
   (graditelj upiše vrijednost iz prvog zelenog prolaza; od tada je pravilo kao za `SEED_*`).
9. Ekran: `SituationsScreen`/`Conversation` test s fiksturom `look: {gender: "f", …}` crta avatar bez `data-part^="facial-"`.
RED: `lookFromHint` i `LookHint` ne postoje (greška tipova), testovi 3–9 padaju.

**Koraci:**
- IZGLED-a, čuvar: testovi 1–7 i `look: None` u postojećim literalima; commit RED s popisom grešaka prevođenja.
- IZGLED-a, graditelj: (1) `crates/app/src/look.rs` (`character_look`, `age_hint`); (2) `views.rs`, `talk.rs`; (3) seljenje
  razgovornih tijela iz `schema.rs` (najprije čista selidba, `check:api` mora dati **isti** `openapi.json`), pa `DUG` niže;
  (4) `CharacterLookBody`, enumi, polja; (5) `corepack pnpm gen:api`; (6) fiksture weba `look: null` / `character_look: null`
  (bez njih `typecheck` pada); (7) sve provjere.
- IZGLED-b, čuvar: testovi 1–9 (vitest), commit RED. Graditelj: `LookHint`, `HINT_*`, `lookFromHint`, `lookFor(name, hint?)`,
  dva pozivatelja; provjere; snimak testa 8.
- Pregled (opus): Review Focus niže; Leon na kraju vidi lica u pregledniku (Za profesora → situacije), jer je to proizvodna
  odluka koju student osjeti.

**Provjere** (svaki dio, cijeli popis iz `.github/workflows/ci.yml`; izlazi u `.superpowers/sdd/2026-10-14-popravak-temelja/izgled-provjere/`):
`cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
`node scripts/probe-mutacija.mjs` · `cargo sqlx migrate run --source crates/store/migrations` + `cargo sqlx prepare --workspace
--check` · `cargo test --workspace` · `corepack pnpm check:docs`, `test:docs`, `check:kod`, `check:api`, `lint`, `typecheck`,
`test` · `corepack pnpm --filter @hotelixsim/web build` · `git diff --exit-code` nakon commita · cargo deny samo u CI-ju.
- **Bajt-isto:** `cargo run -q -p balance --release -- --seeds 100 --check` i `--seeds 1000` (v1), `--model v2 --round week
  --seeds 1000 --check` i `--round month`: svaki izlaz `cmp` s osnovicom s a992973 (balans ne ovisi o `app`/`api`/webu, pa je
  razlika znak greške). Pečat: `cargo test -p game --test v2_sealed` i `v1_frozen` zeleni bez izmjene `v2-golden.json`;
  `git diff --stat a992973 -- crates/sim-hotel-v2 crates/game crates/content crates/balance content prompts` prazan.
- `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>` (B12); IZGLED-b ne prevodi Rust, pa ne broji u „dva agenta".

**Rizik i odluke 🟨** (za ADR na `/kraj`):
1. **Rod iz popisa imena**, ne iz predloška: najmanji šav, bez jezgre; ime u dva popisa → `None` (stari izgled). Ako K5 donese
   ponovljena imena, prelazi se na rezervni put (predložak primjerka), opet bez jezgre.
2. **Podrijetlo do K5 iz izvora imena**: osoblje i partneri `regional` (koža 0, 1, 6), gosti `foreign` (0, 1, 2, 6). Gosti s
   hrvatskim imenom (npr. „Ana Horvat", „Ivan Kovač") time su `foreign`, ali raspon je namjerno uzak, pa nikad ne dobiju tamnu
   put. Tamniji tonovi (3, 4, 5, 7) dolaze tek iz banke ljudi sa zemljom (K5); nema popisa „koje je ime strano" u kodu.
3. **Dob**: ključne riječi u opisu uloge (popis gore); osoblje i partneri bez ključne riječi `adult`; gosti bez nje `null` → web
   bira mlad/odrasla iz imena (kao danas). `senior` samo iz opisa (umirovljenica, baka, stariji gost), uz sijedu kosu.
4. **Nazivi u API-ju engleski** (`look`, `gender`, `age_group`, `origin`, `character_look`), kao ostatak API-ja; vrijednosti
   `f`/`m` kao `names.json`.
5. **Stari izgledi bez hinta ostaju bajt-isti** (`lookFor(ime)`): roboti-hoteli na ljestvici i u sezoni nisu likovi s rodom,
   galerija i zamrznuti test „Ana Kovač" ostaju valjani, a stari klijent (ili sidro) ne mijenja lice. Likovi primjeraka **mijenjaju
   lice** jednom (to je i cilj ADR-042 t. 11); nema spremljenih izgleda koje bi to pokvarilo.
6. **Frizure po rodu** (`HINT_HAIR_F/M`) i učestalost brade (1 od 3) i naušnica (1 od 2) su ukusne odluke; Leon ih vidi u
   pregledniku i mijenja se jedan niz.
7. Sidra S1–S4 (Petra, Marko) ostaju ručni `KNOWN_LOOKS`; API im šalje `look: null`.
8. Seljenje razgovornih tijela iz `schema.rs` je preduvjet (dug 1770 ne smije narasti), ne usputno čišćenje.

**Banka ljudi (K5, samo spomen):** kad igra počne vući likove iz banke ([BANKA_LJUDI](../../architecture/BANKA_LJUDI.md) §5.1),
`character_look` čita rod, dob i zemlju iz zapisa banke umjesto iz popisa imena i ključnih riječi; `Origin` tada dobiva
vrijednosti po zemlji, a web bazen kože po tome. Oblik `CharacterLookBody` ostaje; ova cigla to ne gradi.

**Leonov dio:** redoslijed i stil odlučeni (16.10., sesija 7). Pretpostavke 2, 3 i 6 student vidi, ali nemaju pravu alternativu
prije K5; voditelj ih zapisuje kao ADR 🟨 bez ankete. Leon na kraju pogleda 10–20 lica (situacije jednog kruga) i kaže što ne valja.

**Review Focus kandidati:**
1. Ime koje nije u popisu (sidro, budući sadržaj) ili je u dva popisa: `None` i stari izgled, bez pada ekrana.
2. `age_hint` ne pogađa podnizove („ostarjelom", „starim psom") ni opis tuđe osobe; senior ima prednost pred young.
3. Robot-hotel na ljestvici (`Leaderboard.tsx:45`) i sezoni (`HotelCard.tsx:26`) crta se kao prije.
4. `openapi.json` nakon selidbe (korak a3) bajt-isti prije dodavanja polja; `DUG` spušten, `check:kod` bez upozorenja.
5. `lookFromHint` troši rng istim redom bez obzira na granu (inače dodavanje opcije pomiče druga polja).
6. Hrvatsko ime (staff/partner i gost) nikad koža 3, 4, 5 ili 7 — test 6 prolazi kroz sve kombinacije.
