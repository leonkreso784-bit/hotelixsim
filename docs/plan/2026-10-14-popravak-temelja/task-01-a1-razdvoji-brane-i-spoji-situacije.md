### Task A1: Razdvoji brane balansa (brane modela / mjere robota) i spoji situacije u sloj1

**Veličina:** S–M, ~3 h · **Ovisi o:** A0 (sigurnosna kopija grana) · **Paralelno s:** kartice A2 koje ne diraju
`crates/balance` · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** `balance --model v2 … --check` ruši samo na **branama modela** (ne ovise o robotima). Šest **mjera robota**
i dalje se računa i ispisuje u izvještaju, u odjeljku „Mjere robota", ali više ne ruši provjeru. Time grana situacija
(`model-v2/sloj1-l111`) ulazi u `sloj1` bez kalibracije stare mjere „udio situacija". Leon i agenti nakon toga
znaju: crveni `--check` znači da je pukao model, a ne da se robotu promijenio plan. v1 se ne mijenja ni u bajtu.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #3, #4 i #14; izvještaj 03, nalazi 1 i 11 te „Popis brana v2".
- Brane su pomiješane u jednoj funkciji, `failures_v2`: pravednost `crates/balance/src/v2.rs:619`, tj≈mj `:622-632`,
  Brižni sezonci `:633-640` i udio/Δ situacija `:651-679` stoje uz GOP/marže `:576-617`.
- Udio situacija `v2_situations.rs:218-264` pada na l111 (B Brižni tjedno 10,2 %, mjesečno 19,3 %; `l1-11-report.md`).
- Naredba u CLAUDE.md (`--seeds 1000 --check`) vrti samo v1, jer je v1 zadan (`crates/balance/src/main.rs:10`).
  ADR-033 t. 8 tvrdi „v1 s 1000", a CI vrti v1 sa 100 sjemena (`.github/workflows/ci.yml:40`).

**Podjela (izvor: 03 „Popis brana v2"):**

| brane modela, ruše `--check` (`failures_v2`) | mjere robota, samo ispis (`robot_measures_v2`) |
|---|---|
| §11.1 početak (`check_start`) | §11.3 pravednost (`check_fairness`) |
| §11.2 GOP marža 35–45 %, EBITDA < GOP | sloj 1 tjedno ≈ mjesečno (≤ 300 stotinki) |
| marže odjela, udio rada, ostali prihodi C | sloj 1 brižni robot: sezonci ≥ 80 % plana (ZP-4) |
| §11.4 dominantna proba (`check_dominant`) 🟨 | J-3 udio situacija Brižnog (`SHARE_BAND_BP`) |
| J-3 grupe, J-3 broj situacija, J-3 raznolikost | J-3 brižni Δ > 0 |
| §11.5 ponovljivost (`check_repro`) | J-3 štediša Δ < 0 i najviše 6 bodova |

Tekstovi poruka ostaju **doslovno isti** kao danas: mijenja se samo popis u koji poruka ide. Redoslijed unutar
svakog popisa ostaje kao danas.

**Files:**
- Modify: `crates/balance/src/v2.rs`
  - `:568-569`: `CARING_MIN_SEASONAL_VS_PLAN_BP` postaje `pub const`.
  - `:571-646`: `failures_v2` vraća samo brane modela.
  - `:648-699`: `situation_failures` se dijeli na dvije funkcije.
  - `:775-832`: `render_v2` na kraju ispisuje odjeljak „Mjere robota"; dodaje se nova `robot_measures_v2`.
- Modify: `crates/balance/tests/balance_v2.rs`. Test `caring_share_in_band_with_negative_delta_fails` (`:59-89`) seli
  u novu datoteku s novim očekivanjem (vidi T4); dodaje se T1.
- Modify: `crates/balance/tests/robots_v2.rs`: dodaje se T6 na kraj datoteke.
- Create: `crates/balance/tests/common/mod.rs`, `crates/balance/tests/robot_measures_v2.rs`.
- Ne dirati:
  - v1 dio `crates/balance/src/lib.rs`: `failures`, `render` i `check_*` ostaju bajt-isti;
  - `crates/balance/src/main.rs`, `v2_situations.rs` (pojas `SHARE_BAND_BP` ostaje 20–40 %, a mijenja ga tek faza B/L1-13);
  - `crates/sim-hotel*`, `content/`, `.github/workflows/ci.yml`, `docs/` i `CLAUDE.md` (dokumentaciju piše kontrolor,
    vidi korak K).

**Interfaces:**
- Consumes (postoje): `balance::v2::{run_v2, failures_v2, render_v2, OptionsV2, ReportV2, SituationReport}`,
  `crate::{check_start, check_fairness, check_dominant, check_repro}` (`lib.rs:488, 510, 548, 566`),
  `v2_situations::{SHARE_BAND_BP, SAVER_MAX_LOSS, DOMINANT_MAX_BP, count_band}`.
- Produces:
```rust
// crates/balance/src/v2.rs
/// Najmanji udio sezonskog plana hotela koji brižni robot drži kroz sezonu (ZP-4); mjera robota i test robota.
pub const CARING_MIN_SEASONAL_VS_PLAN_BP: i32 = 8_000;
/// Brane modela (ne ovise o robotima): ruše `--check`. Prazno = sve prolaze.
pub fn failures_v2(r: &ReportV2) -> Vec<String>;
/// Mjere robota (A1): pravednost, tjedno ≈ mjesečno, Brižni sezonci, udio situacija Brižnog, Brižni Δ > 0,
/// Štediša Δ. Ispisuje ih `render_v2`; ne ruše `--check`. Prazno = sve u pojasu.
pub fn robot_measures_v2(r: &ReportV2) -> Vec<String>;
// privatne: fn situation_model_failures(f: &mut Vec<String>, sr: &SituationReport)   // grupe, broj, raznolikost
//           fn situation_robot_measures(f: &mut Vec<String>, sr: &SituationReport)   // udio, Brižni Δ, Štediša Δ
//           fn push_robot_measures(s: &mut String, m: &[String])
```

**Testovi čuvara** (pišu se prije koda; sve naredbe se pokreću u stablu `C:/Users/leonk/Documents/hotelixsim-a1`):

*`crates/balance/tests/common/mod.rs`* (novo; pomoćnik bez vlastitih testova):
```rust
//! Zajednički pomoćnici testova balansa v2 (A1).
#![allow(dead_code)]
use balance::v2::ReportV2;
use content::{Snapshot, repo_dir};
use sim_hotel_v2::Strategy;

pub fn content_v2() -> content::Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

/// Svih šest mjera robota izvan pojasa; brane modela netaknute.
pub fn spoil_robot_measures(r: &mut ReportV2) {
    r.profile_medians[0].1 += 10_000; // §11.3: jedan profil daleko ispred
    r.week_vs_month[0].3 = r.week_vs_month[0].2 + 1_000; // tjedno − mjesečno = 10 bodova
    r.caring_seasonal_vs_plan_bp[0].1 = 0; // Brižni ne drži sezonce
    let sr = r.situations.as_mut().expect("repo ima predloške");
    for s in &mut sr.shares {
        match s.strategy {
            Strategy::Caring => s.share_bp = 0, // udio izvan pojasa
            Strategy::Saver => s.delta = 100,   // situacije Štediši pomažu
            _ => {}
        }
    }
}
```

**T1** `crates/balance/tests/balance_v2.rs::robot_measures_do_not_fail_check`. Na vrh datoteke ide `mod common;`.
```rust
/// A1: mjere robota ne ruše `--check`, ni kad su sve izvan pojasa.
#[test]
fn robot_measures_do_not_fail_check() {
    let mut r = run_v2(&content_v2(), &OptionsV2 { seeds: 3, kind: PeriodKind::Week }).unwrap();
    common::spoil_robot_measures(&mut r);
    let f = failures_v2(&r);
    for robot in ["pravednost", "tjedno ≈ mjesečno", "brižni robot", "udio situacija", "štediša"] {
        assert!(!f.iter().any(|x| x.contains(robot)), "{robot}: {f:?}");
    }
}
```
Danas je **crven iz pravog razloga**: `failures_v2` vraća i „§11.3 pravednost…", pa pada
`assertion failed … pravednost: [...]`. Prevodi se, jer koristi samo postojeće ime.

**T2–T5** `crates/balance/tests/robot_measures_v2.rs` (nova datoteka, `mod common;`; uvoz
`balance::v2::{OptionsV2, failures_v2, render_v2, robot_measures_v2, run_v2}`). Danas se **ne prevodi**:
`unresolved import balance::v2::robot_measures_v2`, jer funkcija je nova (Produces).
- **T2 `robot_measures_are_listed_and_printed`:** pokvareni izvještaj (seeds 3, tjedno, `spoil_robot_measures`).
  `robot_measures_v2` sadrži poruku sa svakim od pet podnizova iz T1. `render_v2` sadrži redak
  `== Mjere robota (ispis, ne ruše --check) ==` i svaku poruku iz `robot_measures_v2` doslovno.
- **T3 `model_gates_still_fail_check`:** zadaje se `r.reference_season_gop_margin_bp = 1_000`,
  `r.reproducible = false` i u `situations` `group_limit = Some(1)`, `draws.groups_max = 2`. `failures_v2` tada sadrži
  „§11.2 ekonomija", „J-3 grupe" i „§11.5 ponovljivost", a `robot_measures_v2` ne sadrži nijedan od njih.
- **T4 `caring_negative_delta_is_a_robot_measure`:** tijelo je doslovno kao današnji
  `caring_share_in_band_with_negative_delta_fails` (`balance_v2.rs:59-89`: Δ −2,00, bez situacija 6,00, udio 25 %).
  Mijenja se samo završna provjera: `robot_measures_v2(&r)` sadrži „Brižnom smanjuju napredak" i šifru profila, a
  `failures_v2(&r)` je ne sadrži. Stari test se briše iz `balance_v2.rs`. **Ovo je jedina izmjena postojećeg očekivanja.**
- **T5 `in_band_robot_measures_print_all_clear`:** izvještaj u pojasu (svi `profile_medians` = 0; `week_vs_month`
  mjesečno = tjedno; `caring_seasonal_vs_plan_bp` = 10 000; Brižni `share_bp` 3 000 i `delta` 100; Štediša `delta`
  −100) → `robot_measures_v2` je prazan, a `render_v2` sadrži `  sve mjere robota u pojasu`.

**T6** `crates/balance/tests/robots_v2.rs::caring_robot_holds_most_of_the_seasonal_plan` (test robota ZP-4,
preseljen iz `--check`):
```rust
/// ZP-4 (A1: test robota, ne brana igre): brižni robot drži najmanje 80 % sezonskog plana hotela, za svaki profil,
/// tjedno i mjesečno (sjeme ponovljivosti 7, situacije iz predložaka; isto što balans ispisuje).
#[test]
fn caring_robot_holds_most_of_the_seasonal_plan() {
    use balance::v2::{CARING_MIN_SEASONAL_VS_PLAN_BP, OptionsV2, run_v2};
    use sim_hotel_v2::PeriodKind;
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = run_v2(&content, &OptionsV2 { seeds: 1, kind }).unwrap();
        assert_eq!(r.caring_seasonal_vs_plan_bp.len(), 3);
        for (p, bp) in &r.caring_seasonal_vs_plan_bp {
            assert!(*bp >= CARING_MIN_SEASONAL_VS_PLAN_BP, "{kind:?} {}: {bp} bp", p.code());
        }
    }
}
```
Danas se ne prevodi: `constant CARING_MIN_SEASONAL_VS_PLAN_BP is private`. Nakon koda očekuje se zeleno, jer na l111 brana
prolazi (`l1-11-report.md`: pada samo udio). Vrijednost ne ovisi o `seeds`, jer se računa uz `REPRO_SEED`.

**Koraci graditelja:**
1. **Radno stablo i spajanje** (prije ikakvog koda), iz glavnog stabla:
   `git worktree add -b popravak/a1-brane ../hotelixsim-a1 model-v2/sloj1-l111`. Zatim u
   `/c/Users/leonk/Documents/hotelixsim-a1`: `git merge --no-ff --no-commit model-v2/sloj1`, pa
   `git commit -F <scratchpad>/a1-merge.txt` s porukom „Popravak A1: spoji sloj1 (docs 14.10., rustls 0.23.46) u granu
   situacija". **Sukoba ne bi trebalo biti**: `git diff 24b4e2a 3e04b08 --stat` dira samo `CLAUDE.md`, `Cargo.lock`
   (rustls) i 8 datoteka u `docs/`, a `git diff 24b4e2a b1bf446 --stat -- CLAUDE.md Cargo.lock docs` je prazan.
   Ako sukob ipak bude, stani i javi BLOCKED, bez razrješavanja.
2. **Osnovica prije izmjene** (stablo `-a1`, nakon spajanja), izlazi u scratchpad: `cargo run -q -p balance --release
   -- --seeds 100 --check` → `v1-100-prije.txt`; isto sa `--seeds 1000` → `v1-1000-prije.txt`; `… --model v2 --round
   week --seeds 1000` (bez `--check`) → `v2-week-prije.txt`, isto s `--round month` → `v2-month-prije.txt`.
   Očekivano u v2: „Provjere koje padaju (1): J-3 udio situacija: B brizni 10,2 %…" (mjesečno 19,3 %).
3. Čuvar piše T1–T6 i zapisuje RED: T1 pada na „pravednost", a T2–T6 se ne prevode.
4. `v2.rs`:
   - `CARING_MIN_SEASONAL_VS_PLAN_BP` postaje `pub const`.
   - `situation_failures` se dijeli na dvije funkcije:
     - `situation_robot_measures` dobiva cijelu petlju `for s in &sr.shares { match … }` (`:651-679`), nepromijenjenu;
     - `situation_model_failures` dobiva grupe, broj i raznolikost (`:681-699`), nepromijenjene.
   - Iz `failures_v2` se brišu pozivi `check_fairness`, petlja tj≈mj, petlja Brižnih sezonaca i dio situacija o
     udjelu; `situation_failures` zamjenjuje `situation_model_failures`.
5. Nova `robot_measures_v2`, ovim redom:
```rust
pub fn robot_measures_v2(r: &ReportV2) -> Vec<String> {
    let mut f = Vec::new();
    check_fairness(&mut f, &r.profile_medians, &r.medians);
    // … petlja tjedno ≈ mjesečno i petlja Brižnih sezonaca, premještene doslovno iz failures_v2 …
    if let Some(sr) = &r.situations {
        situation_robot_measures(&mut f, sr);
    }
    f
}
```
6. `render_v2`: nakon retka „Ponovljivost" dolazi `push_robot_measures(&mut s, &robot_measures_v2(r))`: naslov
   `== Mjere robota (ispis, ne ruše --check) ==`, pa svaka poruka kao `  - {poruka}` ili `  sve mjere robota u pojasu`.
7. Svi testovi su zeleni (`cargo test -p balance`), zatim slijede provjere. Commit „Popravak A1: brane modela ruše
   --check, mjere robota se ispisuju; test Brižnih sezonaca u testovima robota" ide s izričitim putanjama.

**Provjere (točne naredbe i očekivani izlaz):** u `/c/Users/leonk/Documents/hotelixsim-a1`, svaka naredba zasebno
pa `; echo EXIT=$?`.
- Opće provjere: `cargo fmt --all -- --check` → 0; `cargo clippy --workspace --all-targets -- -D warnings` → 0;
  `cargo test --workspace` (baza `hotelixsim_agenti`) → 0 failed; `node scripts/check-core-deps.mjs` → 0;
  `corepack pnpm check:kod` → 0 (`v2.rs` danas 832 + ~40 redaka < 1000, pa se `DUG` u `scripts/check-kod.mjs:20-24`
  ne mijenja; ako ipak raste neki unos, diže se u istom commitu i javlja).
- v1 prema osnovici iz koraka 2: `cargo run -q -p balance --release -- --seeds 100 --check` → EXIT=0 i `cmp` s
  `v1-100-prije.txt` bez razlike; isto za `--seeds 1000` i `v1-1000-prije.txt`.
- v2 `--round week --seeds 1000 --check` → **EXIT=0**; isto za `--round month`.
- `diff v2-week-prije.txt <novi izlaz bez --check>` smije pokazati **samo**: (a) novi odjeljak „Mjere robota" s retkom
  „J-3 udio situacija: B brizni 10,2 % (cilj 20,0 %–40,0 %)"; (b) završni redak „Provjere koje padaju (1)…" postaje
  „Sve provjere prolaze.". Svaki drugi redak bajt-isti. Isto za mjesečno (19,3 %).
- `git diff model-v2/sloj1 --stat -- crates/sim-hotel crates/balance/src/lib.rs crates/balance/src/main.rs` → prazno.

**Korak K (kontrolor, nakon pregleda):**
1. **Spajanje:** u glavnom stablu na `model-v2/sloj1` ide `git merge --ff-only popravak/a1-brane`. Grana sadrži
   3e04b08, pa je spajanje brzo i bez sukoba. Neprijavljene izmjene u glavnom stablu (`docs/README.md`,
   `REVIZIJA_TEMELJA.md`) grana ne dira.
   - Zatim se ponovno pokreće balans v2 week i month s 1000 sjemena i `--check` (BUGS B8: zastarjeli `balance.exe`).
   - Grane `model-v2/sloj1-l111` i `model-v2/sloj1-situacije` su time spojene. Brišu se tek nakon A0 kopije i uz Leonov OK.
2. **CLAUDE.md, redak „Balans:"** (umjesto `cargo run -p balance --release -- --seeds 1000 --check`) dobiva tekst:
   ```text
   Balans: `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` (i `--round month`;
   v1: `--seeds 100 --check`, kao CI). `--check` ruši samo brane modela; mjere robota se samo ispisuju (ADR-033 Dopuna 14.10.).
   ```
   U CLAUDE.md „ADR-033" je poveznica na `docs/records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md`.
3. **ADR-033: ispravlja se dokument, a ne CI.** Preporuka je da v1 ostane na 100 sjemena. v1 je zamrznut i čuva ga
   zlatni SHA-256 test (`crates/game/tests/v1_frozen.rs`), pa 1000 sjemena za v1 samo troši minute CI-ja i ne daje
   novu informaciju. Na kraj ADR-a ide odlomak:
   > **Dopuna 14.10. (Leon: popravak temelja, faza A1; [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §6).**
   > `balance --check` ruši samo **brane modela**: početak (§11.1), GOP/EBITDA, marže odjela, udio rada, ostali prihodi
   > C, dominantna proba (§11.4), granica grupa, broj situacija po krugu, raznolikost i ponovljivost.
   > Šest **mjera robota** računa se i ispisuje u odjeljku „Mjere robota", ali ne ruši provjeru: pravednost (§11.3),
   > tjedno ≈ mjesečno, Brižni sezonci ≥ 80 %, udio situacija Brižnog, Brižni Δ > 0 te Štediša Δ < 0 i najviše 6 bodova.
   > „Brižni sezonci ≥ 80 %" je i test robota (`crates/balance/tests/robots_v2.rs`). v1 provjere su nepromijenjene.
   > **Ispravak t. 8:** CI vrti v1 sa 100 sjemena (`ci.yml:40`), a v2 tjedno i mjesečno s 1000.
4. Ostala dokumentacija: CHANGELOG (A1, L1-11 spojen), SLJEDECA_SESIJA i ROADMAP (L1-11 gotov, a L1-13 čeka fazu B)
   pišu se u `/kraj`. `corepack pnpm check:docs` → 0.

**Rizik i odluke:**
- 🟨 **Dominantna proba (§11.4) ostaje brana modela.** Revizija je zove „polu-robotskom", ali proba je čvrst plan sa
  srednjim odgovorom, bez strategije robota. Ako Leon kaže drukčije, poziv `check_dominant` seli u
  `robot_measures_v2` (jedan redak).
- 🟨 **Broj situacija, raznolikost i grupe** računaju se nad sezonama svih strategija (`kind_medians` → `DrawStats`),
  pa ovise i o tome kako roboti mijenjaju stanje hotela. Ipak ostaju brane modela: mjere sadržaj i generator, a revizija
  ih vodi kao robot-neovisne.
- **Zelen `--check` ne znači „situacije ~20 %".** Udio B Brižnog (10,2 %) vidi se samo u ispisu. Novu mjeru donosi
  faza B (B3), a pojas mijenja L1-13. Provjeravatelj treba paziti da tekstovi poruka nisu promijenjeni, jer o njima
  ovisi usporedba s ~40 povijesnih izvještaja.
- Provjeravatelj neka gleda i: v1 izlaz bajt-isti (`cmp`); T4 je jedina izmjena postojećeg očekivanja; u
  `v2_situations.rs` nije promijenjena nijedna konstanta.

**Leonov dio:** ništa novo. Odluku o razdvajanju donio je Leon 14.10. (faza A). Na pregled mu idu samo 🟨 iz Rizika
(dominantna proba), i to u istom paketu kao ADR-033 Dopuna.

## Review Focus kandidati
1. **CI je zelen, a mjera robota je daleko izvan pojasa** (npr. udio situacija 0 % nakon nove mehanike). Nijedan test
   to ne ruši, a ispis nitko ne čita. Prijedlog za A3: nakon svakog spajanja kontrolor u izvještaj prepisuje odjeljak
   „Mjere robota".
2. **`count_band` je tvrdo u kodu** (`v2_situations.rs:267-272`). Ostaje brana modela, pa ADR-042 t. 13 (mjesečno
   4/5/6) odmah ruši `--check`. Prije t. 13 broj mora postati ulaz balansa (faza B2).
3. **Brižni sezonci (T6) mjere se jednim sjemenom (7) uz situacije.** Ako se izvlačenje promijeni, test robota može
   pasti zbog situacija, a ne zbog pravila zapošljavanja ZP-6. Poruka pada imenuje profil i vrstu kruga, ali ne i
   uzrok.
