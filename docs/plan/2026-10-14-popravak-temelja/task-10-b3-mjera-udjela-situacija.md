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
