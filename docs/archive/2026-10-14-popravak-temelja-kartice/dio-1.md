# 2026-10-14-popravak-temelja-kartice — dio 1

Arhiva (zamrznuto). Indeks: [2026-10-14-popravak-temelja-kartice.md](../2026-10-14-popravak-temelja-kartice.md).

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

### Task A2: Načela igre kao provjere sadržaja pri učitavanju (s popisom duga koji smije samo padati)

**Veličina:** M, 6–8 h · **Ovisi o:** A1 (grana `sloj1-l111` spojena u `sloj1`; kod čitan na b1bf446) · **Paralelno s:** A3
(druge datoteke; najviše 2 agenta koji prevode Rust) · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Sadržaj s diska (repo ili `HOTELIXSIM_CONTENT`, npr. profesorova mapa) koji krši načelo igre ne prolazi: server
ne krene i kaže koji predložak krši koje načelo. Današnja kršenja (dug za Fazu B) su popis u kodu: server kreće, dug je vidljiv
i ne raste (novi prekršaj pada; popravljeni predložak ostao na popisu pada). Snimak stare igre se načelima nikad ne provjerava.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #5 (+ 02 #1, 2, 8, 9, 10, 11). Načela su samo testovi (`crates/content/tests/
v2_templates.rs:130-148, 154-167`), učitavanje gleda oblik (`templates_check.rs:18-189`), server prima sadržaj mimo `cargo test`
(`crates/server/src/main.rs:35-48`). Najbolji odgovor najdulji u 54/55, kvar bez uvjeta stanja u 11/15 (brojanje nad JSON-om).

**Files:**
- Create: `crates/content/src/{templates_principles,templates_principles_debt}.rs`, `crates/content/tests/v2_principles.rs`
- Modify: `crates/content/src/lib.rs` (moduli i `pub use` uz r. 14-21; `Snapshot::read_v2_checked` uz r. 606-612),
  `crates/content/src/templates.rs` (polje `external_cause` u `Template`, iza `no_penalty`, r. 88-91),
  `crates/content/src/templates_check.rs` (jedan uvjet u `check_template`, iza r. 134),
  `crates/server/src/main.rs:47` (`read_v2` → `read_v2_checked`),
  `crates/content/tests/v2_templates.rs` (iz `repo_templates_cover_kinds_and_departments` makni petlju r. 130-148;
  obriši `repo_names_have_twenty_per_list` r. 152-167 — oboje seli u načela),
  `content/hotel/v2/situation_templates.json` (samo `"external_cause": true` u P-TOPLINSKI-VAL, P-OLUJA, P-NESTANAK-STRUJE,
  P-DOBAVLJAC, P-TERMINAL — Leonova odluka ✅ ADR-042 t. 7; nijedan drugi bajt).
- Ne dirati: `build()` i `Snapshot::from_value` (načela NE ulaze u put snimka iz dnevnika), `crates/sim-hotel*`, `crates/game`,
  `crates/balance`, `content/hotel/{catalog,profiles,situations}.json`, `crates/content/tests/fixtures/**`, `docs/`.
  Zajednička pravila: `.superpowers/sdd/2026-10-10-v2-sloj1/global-constraints.md` i `implementer-common.md`.

**Interfaces:**
- Consumes: `Content { v2, situation_templates, .. }`, `SituationTemplates::{to_core, instance}` (`templates_instance.rs:252`),
  `sim_hotel::{effects, Effects, SituationOutcome, Month, Category}`, `sim_hotel_v2::situations::{reward_only, DrawnSituation,
  NumberParam, Scope}`, `sim_hotel_v2::calendar::month_period`, `TemplateAnswer::quality`.
- Produces:
```rust
// crates/content/src/templates_principles.rs (NOVO); lib.rs: pub use templates_principles::{Principle, Violation,
//   principle_violations, check_principles_with}; pub use templates_principles_debt::KNOWN_VIOLATIONS;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Principle { // prva četiri iz testa (ocjena 5; ocjena 3 ili 4; gap_rounds >= 4; >= 20 imena), ostalo NOVO (02 #1, 2, 9, 10, 11)
    BestAnswer, MiddleAnswer, RepeatGap, NamesPerList,
    LabelLengths, BreakdownCondition, DeptPlaceholder, NamesUnique, SilenceNotBetter,
}
impl Principle { pub fn name(self) -> &'static str } // hrvatski: npr. RepeatGap → "razmak ponavljanja barem 4 kruga"
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation { pub principle: Principle, pub code: String, pub detail: String }
// code = šifra predloška; za imena "names.<izvor>.<rod>" (NamesPerList) ili "names" (NamesUnique)
pub const LABEL_RATIO_PCT: usize = 150;   // 🟨 k = 1,5
pub const MIN_GAP_ROUNDS: u8 = 4;
pub const MIN_NAMES_PER_LIST: usize = 20;
pub(crate) fn is_breakdown(t: &Template) -> bool; // kind == Kind::Breakdown ili EquipmentRepair u odgovoru/unanswered
pub fn principle_violations(c: &Content) -> Result<Vec<Violation>, ContentError>; // bez predložaka → Ok(vec![])
pub fn check_principles_with(c: &Content, known: &[(Principle, &[&str])]) -> Result<(), ContentError>;
// crates/content/src/templates_principles_debt.rs (NOVO)
pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])];
// crates/content/src/lib.rs
impl Snapshot { pub fn read_v2_checked(dir: &Path) -> Result<Snapshot, ContentError> } // read_v2 + build + načela
// crates/content/src/templates.rs, struct Template (JSON ključ "external_cause"):
#[serde(default, skip_serializing_if = "crate::is_false")] pub external_cause: bool,
```

**Pravila novih načela (stara četiri točno kao danas u testu):** **LabelLengths:** duljina = `label.chars().count()` sirove
oznake; krši kad `max * 100 > min * LABEL_RATIO_PCT` među svim odgovorima, bez obzira koji je najdulji (Rizik).
**BreakdownCondition:** za `is_breakdown(t)` traži `t.when.dept_condition_pts` s `below: Some(_)` ili `t.external_cause`;
`chance.boost` nije uvjet (kvar bi i dalje dolazio u dobro održan odjel). **DeptPlaceholder:** u tekstovima iz `check_texts`
(naslov, opis, rubrika, oznake, tri polja lika) svaki `{odjel}` je ili točno „({odjel})", ili mu prethodna riječ (`trim_end`,
zadnja po razmaku) malim slovima počinje s `odjel` („u {odjel}" ispisalo bi „u Restoran"). **NamesUnique:** isto ime u dva
od šest popisa; `detail` imenuje oba. **SilenceNotBetter:** za svaku kategoriju (`base.categories`) i mjesec (`when.months`,
prazno = `Month::SEASON`) složi primjerak (korak 2), `effects` šutnje (`SituationOutcome::unanswered`) i svakog odgovora
najmanje kvalitete (`from_answer`), uz `no_penalty` oboje kroz `reward_only`. Šutnja dominira kad su `groups` jednaki,
`satisfaction_pts, morale_pts, rating, equipment_pts` ≥, `cost_cents, extra_wage_cents` ≤ i `silent != worst`.

**Testovi čuvara** — `crates/content/tests/v2_principles.rs`. Danas crveni svi zajedno iz istog razloga: datoteka se ne
prevodi (`error[E0432]: unresolved imports content::Principle, content::KNOWN_VIOLATIONS…` i `E0599 no function
read_v2_checked`). Test 10 bi s današnjim `build()` prošao — čuva da načela ne uđu u put starih igara.
```rust
use content::{Content, ContentError, KNOWN_VIOLATIONS, Principle::{self, *}, Snapshot, check_principles_with,
              principle_violations, repo_dir};
use serde_json::{Value, json};
fn v2_value() -> Value { Snapshot::read_v2(&repo_dir()).unwrap().to_value() }
fn built(v: &Value) -> Content { Snapshot::from_value(v).unwrap().build().unwrap() }
fn tpl<'a>(v: &'a mut Value, code: &str) -> &'a mut Value {
    let i = v["v2"]["situation_templates"]["templates"].as_array().unwrap().iter().position(|t| t["code"] == code).unwrap();
    &mut v["v2"]["situation_templates"]["templates"][i]
}
/// Krši li `code` načelo `p` nakon izmjene.
fn hit(change: impl FnOnce(&mut Value), p: Principle, code: &str) -> bool {
    let mut v = v2_value();
    change(&mut v);
    principle_violations(&built(&v)).unwrap().iter().any(|x| x.principle == p && x.code == code)
}
fn repo_codes(p: Principle) -> Vec<String> { principle_violations(&built(&v2_value())).unwrap().into_iter()
    .filter(|x| x.principle == p).map(|x| x.code).collect() }
#[test] fn repo_content_loads_checked_with_known_debt() { Snapshot::read_v2_checked(&repo_dir()).unwrap(); } // 1
#[test] // 2: dug samo za tri nova načela; stropovi = stanje 14.10.
fn known_debt_names_existing_templates_and_never_grows() {
    let v = v2_value();
    let codes: Vec<&str> = v["v2"]["situation_templates"]["templates"].as_array().unwrap().iter()
        .map(|t| t["code"].as_str().unwrap()).collect();
    let ceiling = [(LabelLengths, 55), (BreakdownCondition, 6), (SilenceNotBetter, 16)];
    for (p, list) in KNOWN_VIOLATIONS {
        let max = ceiling.iter().find(|(q, _)| q == p).map_or(0, |(_, n)| *n);
        assert!(list.len() <= max, "{p:?}: dug {} > {max} — popis smije samo padati", list.len());
        let mut s = list.to_vec(); s.sort(); s.dedup();
        assert_eq!(s.len(), list.len(), "{p:?}: šifra dvaput");
        for c in *list { assert!(codes.contains(c), "{p:?}: {c} nije u sadržaju — makni ga s popisa"); }
    }
    for p in [BestAnswer, MiddleAnswer, RepeatGap, NamesPerList, DeptPlaceholder, NamesUnique] {
        assert!(repo_codes(p).is_empty(), "{p:?}: {:?}", repo_codes(p));
    }
}
#[test] // 3: preseljeno iz v2_templates.rs
fn old_principles_are_load_checks() {
    assert!(!hit(|_| {}, RepeatGap, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["answers"][0]["criteria"] = json!([1, 2, 3, 4]), BestAnswer, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["answers"][1]["criteria"] = json!([1, 2]), MiddleAnswer, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3), RepeatGap, "P-KLIMA"));
    assert!(hit(|v| { v["v2"]["names"]["partner"]["m"].as_array_mut().unwrap().pop(); }, NamesPerList, "names.partner.m"));
}
#[test] // 4: k = 1,5, bez obzira koji je odgovor najdulji
fn label_lengths_stay_within_one_and_a_half() {
    let set = |a: usize, b: usize, c: usize| move |v: &mut Value| {
        for (i, n) in [a, b, c].into_iter().enumerate() { tpl(v, "P-KLIMA")["answers"][i]["label"] = json!("x".repeat(n)); }
    };
    assert!(!hit(set(150, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(hit(set(151, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(hit(set(100, 151, 100), LabelLengths, "P-KLIMA"), "srednji najdulji također odaje");
}
#[test] // 5
fn breakdown_needs_bad_state_or_external_cause() {
    let no_cond = |v: &mut Value| { tpl(v, "P-BOJLER")["when"].as_object_mut().unwrap().remove("dept_condition_pts"); };
    assert!(!hit(|_| {}, BreakdownCondition, "P-BOJLER"));
    assert!(hit(no_cond, BreakdownCondition, "P-BOJLER"), "pojačanje nije uvjet");
    assert!(hit(|v| tpl(v, "P-BOJLER")["when"]["dept_condition_pts"] = json!({"at_least": 1000}), BreakdownCondition, "P-BOJLER"));
    assert!(!hit(|v| { no_cond(v); tpl(v, "P-BOJLER")["external_cause"] = json!(true); }, BreakdownCondition, "P-BOJLER"));
    assert!(!hit(|_| {}, BreakdownCondition, "P-OLUJA"), "ADR-042 t. 7: vanjski događaj");
    assert!(hit(|_| {}, BreakdownCondition, "P-BRAVE"), "dug do Faze B");
    let mut v = v2_value(); tpl(&mut v, "P-NOVINAR")["external_cause"] = json!(true);
    let r = Snapshot::from_value(&v).unwrap().build();
    assert!(matches!(&r, Err(ContentError::Invalid(m)) if m.contains("external_cause")), "{r:?}");
}
#[test] fn dept_placeholder_only_after_odjel_or_in_brackets() { // 6
    let title = |s: &'static str| move |v: &mut Value| tpl(v, "P-DOTRAJALO")["title"] = json!(s);
    assert!(!hit(|_| {}, DeptPlaceholder, "P-DOTRAJALO"), "naslov s „({{odjel}})” je dopušten");
    assert!(!hit(title("Gost fotografira {stvar} u odjelu {odjel}"), DeptPlaceholder, "P-DOTRAJALO"));
    assert!(hit(title("Gost fotografira {stvar} u {odjel}"), DeptPlaceholder, "P-DOTRAJALO"));
    assert!(hit(title("{odjel}: gost fotografira {stvar}"), DeptPlaceholder, "P-DOTRAJALO"));
}
#[test] fn names_are_unique_across_lists() { assert!(hit(|v| { let n = v["v2"]["names"]["guest"]["f"][0].clone(); // 7
    v["v2"]["names"]["staff"]["f"][0] = n; }, NamesUnique, "names")); }
#[test] // 8: P-DOTRAJALO danas: najlošiji s hitnim popravkom je skuplji, ali šutnja gubi opremu → nema dominacije
fn silence_is_not_better_than_the_worst_answer() {
    assert!(!hit(|_| {}, SilenceNotBetter, "P-DOTRAJALO"));
    assert!(hit(|v| tpl(v, "P-DOTRAJALO")["unanswered"] = json!([{"type": "ServiceRecovery", "compensation": 0}]),
                SilenceNotBetter, "P-DOTRAJALO"), "isti bodovi, bez troška popravka");
    assert!(hit(|_| {}, SilenceNotBetter, "P-NOVINAR"), "loš odgovor plaća boravak, šutnja ništa (dug)");
}
#[test] // 9: dug smije samo padati
fn debt_list_only_shrinks() {
    let (c, mut v) = (built(&v2_value()), v2_value());
    tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    let e = check_principles_with(&built(&v), KNOWN_VIOLATIONS).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("razmak")), "{e:?}");
    let mut known = KNOWN_VIOLATIONS.to_vec();
    known.push((RepeatGap, &["P-KLIMA"][..]));
    let e = check_principles_with(&c, &known).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("makni")), "{e:?}");
    let mut other_dir = KNOWN_VIOLATIONS.to_vec();
    other_dir.push((LabelLengths, &["P-NEPOSTOJI"][..]));
    check_principles_with(&c, &other_dir).expect("šifra koje nema (druga mapa sadržaja) ne ruši učitavanje");
    assert!(check_principles_with(&c, &[]).is_err(), "bez popisa duga današnji sadržaj pada");
}
#[test] fn snapshot_from_the_log_is_never_checked_for_principles() { // 10: stara igra se uvijek čita
    let mut v = v2_value(); tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    Snapshot::from_value(&v).unwrap().build().expect("snimak iz dnevnika gradi se bez načela");
}
#[test] // 11: put kojim server čita HOTELIXSIM_CONTENT
fn content_dir_with_new_violation_is_refused_at_load() {
    let (src, dir) = (repo_dir(), std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("a2-sadrzaj"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("v2")).unwrap();
    for f in ["catalog.json", "profiles.json", "situations.json", "v2/periods.json", "v2/departments.json",
              "v2/profiles.json", "v2/economy.json", "v2/names.json"] { std::fs::copy(src.join(f), dir.join(f)).unwrap(); }
    let text = std::fs::read_to_string(src.join("v2/situation_templates.json")).unwrap();
    let mut t: Value = serde_json::from_str(&text).unwrap();
    let i = t["templates"].as_array().unwrap().iter().position(|x| x["code"] == "P-KLIMA").unwrap();
    t["templates"][i]["repeat"]["gap_rounds"] = json!(3);
    std::fs::write(dir.join("v2/situation_templates.json"), serde_json::to_string(&t).unwrap()).unwrap();
    let e = Snapshot::read_v2_checked(&dir).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA")), "{e:?}");
}
```

**Koraci graditelja:**
1. `templates.rs`: polje `external_cause` (doc: „Kvar ili popravak čiji uzrok nije stanje prostora hotela — vanjski
   događaj, dobavljač, oprema partnera (ADR-042 t. 7); izuzet od načela ‚kvar samo kad je stanje loše'").
2. `templates_principles.rs`: tipovi, konstante, `is_breakdown`, `principle_violations` (imena, pa predlošci redom datoteke;
   rezultat sortiran). Primjerak za šutnju (privatna `fn probe`) kao `july_instance` (`v2_templates.rs:709-748`): jednom
   `to_core()`, pa `DrawnSituation { code: format!("{code}.{}", m.code()), template: code.into(), period: month_period(m),
   day: 1, occurrence: 1, slot: 1, kind: spec.kind, weight: spec.weight, scope: Scope::Hotel, major: None, roles:
   BTreeMap::new(), department: spec.department.members(v2).and_then(|d| d.first().cloned()), numbers: najmanja vrijednost
   (Range(a, _) → a, Pick(v) → min) }` i `instance(&d, &v2.base, category)`; greška `effects` → `Invalid("predložak {code}: …")`.
3. `check_principles_with`: (a) prekršaj kojeg nema u `known` → redak `"načelo '{name}': {code}: {detail} — novo kršenje;
   popravi sadržaj (popis duga smije samo padati)"`; (b) šifra iz `known` koja **postoji** u sadržaju bez prekršaja tog načela →
   `"načelo '{name}': {code} više ne krši — makni ga s popisa KNOWN_VIOLATIONS (crates/content/src/templates_principles_debt.rs)"`;
   šifre koje nema preskoči. Retci sortirani: `ContentError::Invalid(format!("načela igre:\n{}", retci.join("\n")))`.
4. `templates_check.rs` iza r. 134: `if t.external_cause && !crate::templates_principles::is_breakdown(t) { return
   invalid(format!("{ctx}: external_cause ide samo uz kvar ili popravak")); }`
5. JSON: `"external_cause": true` u pet predložaka iz Files (iza `"no_penalty"` ako ga ima, inače kao zadnji ključ).
6. `templates_principles_debt.rs` (doc: što je dug, da smije samo padati, da ga briše Faza B uz Leonov pregled):
   - `LabelLengths` — svih 55 šifri iz `situation_templates.json`, redom datoteke (P-KLIMA … P-TERMINAL); najmanji omjer
     danas je 2,52 (P-INSPEKCIJA), pa nijedan predložak ne prolazi.
   - `BreakdownCondition` — P-KLIMA P-INSPEKCIJA P-DIZALO P-PRAONICA P-BRAVE P-CURENJE-VODE.
   - `SilenceNotBetter` — P-KLIMA P-POVISICA P-BOJLER P-ROOFTOP-BUKA P-SPA-TECAJ P-LOBBY-SUKOB P-LOBBY-KAVA P-NOVINAR
     P-VAL-RECENZIJA P-KRIZA-OSOBLJA P-ALERGIJA P-OVERBOOKING P-BOLOVANJE P-PRAONICA P-PARKING-STETA P-UCJENA-RECENZIJOM.
   Prva dva popisa su strukturna: razlika od `principle_violations` = greška u kodu, popravi kod. Treći je iz Python
   zrcala `effects`: ako kod daje podskup, upiši stvarni popis i navedi razliku u izvještaju; ako daje **više** → stani, NEEDS_CONTEXT.
7. `lib.rs`: `read_v2_checked` = `let s = Snapshot::read_v2(dir)?; check_principles_with(&s.build()?, KNOWN_VIOLATIONS)?;
   Ok(s)`; doc: „Samo za sadržaj s diska; snimak iz dnevnika (`from_value`) se ne provjerava — stara igra se uvijek čita."
8. `server/src/main.rs:47` → `read(content::Snapshot::read_v2_checked)?`. 9. `v2_templates.rs`: makni preseljeno (Files).

**Provjere (iz korijena stabla, svaka zasebno, `; echo EXIT=$?`):**
- Prije koraka 1 u scratchpad: `cargo run -q -p balance --release -- --seeds 100 --check > b1-prije.txt`, isto s
  `--model v2 --round week --seeds 1000 --check` → `b2w-prije.txt` i `--round month` → `b2m-prije.txt` (zapiši EXIT).
- `cargo test -p content --test v2_principles` → `test result: ok. 11 passed`; zatim `cargo fmt --all -- --check` ·
  `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `node scripts/check-core-deps.mjs` ·
  `corepack pnpm check:kod` → EXIT=0. Ista tri balansa poslije: `diff` prazan, isti EXIT (oznaka se u obračunu ne čita).
- `HOTELIXSIM_CONTENT=target/tmp/a2-sadrzaj timeout 60 cargo run -q -p server -- serve; echo EXIT=$?` (mapu ostavlja test 11)
  → EXIT=1, poruka sadrži „načela igre" i „P-KLIMA"; pada prije baze i porta. Server s repo sadržajem ne pokreći (slušao bi).

**Rizik i odluke:**
- 🟨 **k = 1,5 i „svi odgovori unutar k"**, ne „ili najbolji nije najdulji" (nalog dopušta oba): Leon je rekao „duljine se
  ujednačuju" (ADR-042 t. 10), a „ili" propušta P-POVISICA (75 i 78 prema 25 znakova). Cijena: dug 55/55, ne 54/55.
- 🟨 **Šutnja s troškom:** usporedba uključuje trošak, pa je u dugu 16 predložaka gdje loš odgovor plaća, a šutnja ne (npr.
  P-NOVINAR; test `v2_templates.rs:750` ga drži jednakim samo u bodovima). „Samo bodovi" = makni `cost_cents`/`extra_wage_cents`, dug 0.
- 🟨 **P-INSPEKCIJA** (kriza s popravkom u restoranu) nije u Leonovoj podjeli iz ADR-042 t. 7: ostaje u dugu.
- **Uvjet „čijeg prostora" (Faza B, 02 #3):** P-BRAVE i P-CURENJE-VODE su `maintenance` (ne troši se), P-DIZALO `hotel`;
  kad Faza B doda stanje drugog odjela ili zgrade, načelo ga mora priznati kao uvjet, inače te stavke ne izlaze iz duga.
- **Redoslijed odgovora (najbolji uvijek prvi, B1) NIJE načelo sadržaja** nego prikaza (ADR-042 t. 10, miješanje po hotelu
  u igri): ne ide u provjeru, inače bi autor „ručno miješao", a stari zapisi odgovora značili bi drugo.
- **Provjeravatelj:** `build()`/`from_value` ne zovu načela (test 10); poruka imenuje predložak i načelo; `external_cause`
  u snimku samo kad je `true` (stari snimci isti JSON i otisak); balans bajt-isti. 🟨 u izvještaj taska („Za ADR").

**Leonov dio** (kroz anketu prije Faze B; A2 ne čeka): (1) koliko smiju razlikovati duljine odgovora: 1,5× (preporuka) /
1,3× / 2×; (2) smije li neodgovorena situacija biti jeftinija od lošeg odgovora uz iste bodove: ne, šutnja nikad nije bolja
(preporuka) / smije, gledaju se samo bodovi; (3) inspekcija u restoranu: kvar koji dolazi kad je stanje loše / vanjski događaj.

## Review Focus kandidati
1. **Sidra S1–S4** (v1 `situations.json`, zamrznut) idu u svaku v2 igru, najbolji odgovor prvi i najdulji; načela ih ne gledaju.
2. **Prepuštanje voditelju** (kvaliteta 2, mehanike `unanswered`) često je bolje od najlošijeg odgovora; načelo gleda samo
   šutnju (q 1), pa student može naučiti „prepusti umjesto da biraš".
3. **Put sadržaja mimo starta servera** (budući profesorov uređivač ili API, `balance` nad tuđom mapom) ne zove `read_v2_checked`.

### Task A3: Zapečaćeni otisak sezone v2 vezan uz `CORE_VERSION`

**Veličina:** S, 2–3 h · **Ovisi o:** A1 (pečat se piše nad kodom spojenim u `sloj1`; kod čitan na b1bf446) ·
**Paralelno s:** A2 (druge datoteke; najviše 2 agenta koji prevode Rust) · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Svaka promjena koda koja bi igri u tijeku promijenila brojke (obračun, izvlačenje situacija, primjerak predloška,
potezi robota, početno stanje) a ne digne verziju jezgre v2 ruši test u CI-ju. Dizanje verzije traži novi pečat — svjestan,
vidljiv potez u pregledu. Tako se pravila igre koja traje više ne mogu tiho promijeniti (danas se mogu, i već jesu).

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #7; 01 nalaz 4. Verzija se diže ručno: `crates/sim-hotel-v2/src/lib.rs:50-52`
(`CORE_VERSION = env!("CARGO_PKG_VERSION")`), `crates/sim-hotel-v2/Cargo.toml:3` = `2.1.0` od f9c635c (O-1), a J-3, G-1 i L1-11
su poslije mijenjali obračun. v1 ima pečat (`crates/game/tests/v1_frozen.rs:193`, `fixtures_sealed.rs:9`), v2 nema.
Jedina čuvarica je `Model::check_version` (`crates/game/src/model.rs:140-149`), a ona vjeruje broju.

**Odluka: kanonski zapis i mjesto otisaka.**
- **Zapis = JSON događaja svježe v2 igre** (`GameEvent::to_payload`, `event.rs:352`), redom dnevnika, s vrstom i autorom.
  To je točno ono što ide u bazu i što stara igra čita: `RoundResolved` nosi plan, ishode situacija, izvješće, indekse,
  napredak i cijelo v2 stanje nakon kruga (`detail`, `event.rs:103-122`); `SituationsDrawn` nosi izvlačenje. Novo polje
  događaja ima `skip_serializing_if`, pa dodavanje polja ne mijenja otisak — mijenja ga samo promjena brojki. `sim-hotel-v2`
  nema serde, a `Debug` ispis se mijenja sa svakim novim poljem (lažni padovi), pa otisak nad jezgrom izravno nije bolji.
  Iz `GameCreated` ulaze samo `months`, `hotels`, `rounds` (snimak ima svoj otisak, a `core_version` je ključ pečata).
- **Igra, ne `run_season`:** stare igre računa petlja igre (`open_round` → `robot_moves` → `resolve_job`), ne petlja
  balansa (`sim-hotel-v2/src/season.rs:142`; razlika je #8, posebna kartica). Petlja igre pokriva i `content::instance`,
  `game::drawn` i prepuštanje voditelju — sve što određuje brojke igre u tijeku.
- **Zamrznuti snimak pravila**, ne trenutni sadržaj: izmjena sadržaja ne smije tražiti novu verziju jezgre (igra nosi svoj
  snimak). Snimak se piše jednom (generator), u **novu** mapu `crates/game/tests/fixtures_v2/` — mapu `tests/fixtures/`
  čuva `frozen_v1_fixtures_match_their_sealed_hash` (`fixtures_sealed.rs:47-56`), koji pada na svaku novu datoteku.
- **Otisci po verziji** u `fixtures_v2/v2-golden.json`: `{"snapshot": "<otisak snimka>", "digests": {"2.2.0": {"<slučaj>":
  "<sha256>", …}}}`. Provjerava se samo ključ jednak `CORE_VERSION`; stari ključevi ostaju kao povijest; generator nikad ne
  prepisuje postojeći ključ.
- **Balans v2 zlatni izlaz (03 nalaz 12) je DRUGI test**, ne ovaj: on mjeri trenutni sadržaj i robote na 1000 sjemena i
  legitimno se mijenja sa svakom kalibracijom sadržaja (bez nove verzije jezgre); da je u ovom pečatu, svaka izmjena sadržaja
  tražila bi lažno dizanje verzije ili bi se pečat navikao prepisivati. Preporuka: staviti ga u karticu „igra i balans daju
  isto" (#8), jer obje čuvaju petlju balansa.

**Files:**
- Create: `crates/game/tests/v2_sealed.rs`; generatorom `crates/game/tests/fixtures_v2/v2-snapshot.json` i `v2-golden.json`.
- Modify: `crates/sim-hotel-v2/Cargo.toml:3` (`2.1.0` → `2.2.0`, 🟨 dolje); `Cargo.lock` (samo redak verzije paketa
  `sim-hotel-v2`, koji cargo sam osvježi); `crates/sim-hotel-v2/src/lib.rs:50` (komentar: „raste sa svakom promjenom koja
  igri u tijeku mijenja brojke; čuva je pečat `crates/game/tests/v2_sealed.rs`"); `crates/game/tests/renewal_v2.rs:667` i
  `:693` (literal `"2.1.0"` → `sim_hotel_v2::CORE_VERSION`; r. 667 `assert_eq!` na literal se briše).
- Ne dirati: `crates/game/tests/fixtures/**`, `fixtures_sealed.rs`, `v1_frozen.rs`, `crates/sim-hotel/**`, sav `src/` osim
  komentara iz Modify, sadržaj, `docs/`. Zajednička pravila: `.superpowers/sdd/2026-10-10-v2-sloj1/global-constraints.md`
  i `implementer-common.md`.

**Interfaces:**
- Consumes: `common::{v2_game_from, hotel_ids, apply_ev, GUEST}` (`crates/game/tests/common/mod.rs:10-70, 227`),
  `Game::{current_round, close_round, resolve_job, situations_of_hotel, situation_mandatory, delegate_situation,
  choose_answer, history, hotels}`, `ResolveStep::Done`, `GameEvent::{kind, to_payload}`, `Actor::as_db`,
  `content::{Snapshot, fingerprint_value, repo_dir}`, `sim_hotel_v2::{CORE_VERSION, PeriodKind}`, `sha2` (dev-dep već postoji).
- Produces: samo test i dvije datoteke pečata (oblik gore). Nijedan javni potpis se ne mijenja.

**Testovi čuvara** — `crates/game/tests/v2_sealed.rs`. Danas crveni 1, 2 i 3 s porukom `nema tests/fixtures_v2/…: v2 još
nije zapečaćen` (datoteke ne postoje). Test 4 je zelen odmah: dokazuje da otisak reagira na pravila (bez njega bi pečat
mogao biti „prazan").
```rust
//! Zapečaćeni otisak v2 (A3): igre nad zamrznutim snimkom pravila daju otisak vezan uz `sim_hotel_v2::CORE_VERSION`.
//! Promjena obračuna bez dizanja verzije ruši test 1; nova verzija traži novi pečat (`seal_v2_digest`), stari ostaju.
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use content::{Snapshot, fingerprint_value, repo_dir};
use game::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sim_hotel_v2::{CORE_VERSION, PeriodKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Human { Silent, Answers }
struct Case { name: &'static str, seed: u64, kind: PeriodKind, human: Human }
const CASES: [Case; 6] = [
    Case { name: "t7-odgovara", seed: 7, kind: PeriodKind::Week, human: Human::Answers },
    Case { name: "m7-odgovara", seed: 7, kind: PeriodKind::Month, human: Human::Answers },
    Case { name: "t42-sutnja", seed: 42, kind: PeriodKind::Week, human: Human::Silent },
    Case { name: "m42-sutnja", seed: 42, kind: PeriodKind::Month, human: Human::Silent },
    Case { name: "t2026-odgovara", seed: 2026, kind: PeriodKind::Week, human: Human::Answers },
    Case { name: "m2026-sutnja", seed: 2026, kind: PeriodKind::Month, human: Human::Silent },
];
#[derive(Serialize, Deserialize)]
struct Golden { snapshot: String, digests: BTreeMap<String, BTreeMap<String, String>> }

fn dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_v2") }
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap_or_else(|_| panic!("nema tests/fixtures_v2/{name}: v2 još nije zapečaćen"))
}
fn golden() -> Golden { serde_json::from_str(&read("v2-golden.json")).unwrap() }
fn frozen_value() -> Value { serde_json::from_str(&read("v2-snapshot.json")).unwrap() }

/// Cijela sezona: roboti igraju sami; igrač šuti ili odgovara redom (n-ta situacija sezone dobiva odgovor n mod broj
/// odgovora), a svaku drugu situaciju koju smije prepušta voditelju.
fn play(snap: &Snapshot, c: &Case) -> Game {
    let (mut g, me, guest) = (common::v2_game_from(snap, c.seed, c.kind), common::hotel_ids()[0], Actor::Guest(common::GUEST));
    let mut n: usize = 0;
    while let Some(r) = g.current_round() {
        if c.human == Human::Answers {
            let list: Vec<(String, usize)> =
                g.situations_of_hotel(me, r).unwrap().iter().map(|t| (t.code.clone(), t.answers.len())).collect();
            for (code, len) in list {
                let ev = if n % 2 == 1 && !g.situation_mandatory(me, &code).unwrap() {
                    g.delegate_situation(me, &code).unwrap()
                } else {
                    g.choose_answer(me, &code, u8::try_from(n % len).unwrap()).unwrap()
                };
                common::apply_ev(&mut g, guest.clone(), ev);
                n += 1;
            }
        }
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, guest.clone(), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else { panic!("obračun čeka") };
        for (a, e) in evs { common::apply_ev(&mut g, a, e); }
    }
    g
}

/// Kanonski zapis: vrsta, autor i JSON svakog događaja (kako ide u bazu); iz `GameCreated` samo krugovi i hoteli.
fn digest(g: &Game) -> String {
    let mut h = Sha256::new();
    for r in &g.history {
        let payload = match &r.event {
            GameEvent::GameCreated(c) => json!({ "months": c.months, "hotels": c.hotels, "rounds": c.rounds }),
            e => e.to_payload(),
        };
        let (kind, actor, body) = (r.event.kind(), r.actor.as_db(), serde_json::to_vec(&payload).unwrap());
        for part in [kind.as_bytes(), actor.as_bytes(), &body[..]] { h.update(part); h.update(b"\n"); }
    }
    format!("{:x}", h.finalize())
}
fn digests(snap: &Snapshot) -> BTreeMap<String, String> {
    CASES.iter().map(|c| (c.name.to_string(), digest(&play(snap, c)))).collect()
}

#[test] // 1
fn v2_digest_is_sealed_for_core_version() {
    let g = golden();
    let want = g.digests.get(CORE_VERSION).unwrap_or_else(|| panic!(
        "jezgra v2 {CORE_VERSION} nema pečat: nova verzija traži novi otisak — \
         HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest"));
    let got = digests(&Snapshot::from_value(&frozen_value()).unwrap());
    assert_eq!(got.keys().collect::<Vec<_>>(), want.keys().collect::<Vec<_>>(), "slučajevi pečata");
    for (name, d) in &got {
        assert_eq!(d, &want[name], "{name}: obračun v2 se promijenio, a CORE_VERSION {CORE_VERSION} nije dignut — \
            digni verziju u crates/sim-hotel-v2/Cargo.toml i zapečati novi otisak");
    }
}

#[test] // 2
fn frozen_snapshot_matches_its_seal() {
    assert_eq!(fingerprint_value(&frozen_value()), golden().snapshot,
        "zamrznuti snimak se ne mijenja: nova pravila = nova verzija jezgre, ne novi snimak");
}

#[test] // 3: pečat pokriva oba kruga, sva tri profila u svakoj igri i sve putove odgovora igrača
fn digest_cases_cover_rounds_profiles_and_answer_paths() {
    let snap = Snapshot::from_value(&frozen_value()).unwrap();
    let me = common::hotel_ids()[0];
    let (mut answered, mut delegated, mut silent) = (0, 0, 0);
    for c in &CASES {
        let g = play(&snap, c);
        let profiles: BTreeSet<&str> = g.hotels.iter().map(|h| h.setup.profile.as_str()).collect();
        assert_eq!(profiles, BTreeSet::from(["A", "B", "C"]), "{}", c.name);
        for r in &g.history {
            match &r.event {
                GameEvent::AnswerChosen(a) if a.hotel_id == me => answered += 1,
                GameEvent::SituationDelegated(_) => delegated += 1,
                GameEvent::RoundResolved(rr) => silent += rr.hotels.iter().filter(|h| h.hotel_id == me)
                    .flat_map(|h| &h.situations).filter(|s| s.answer.is_none()).count(),
                _ => {}
            }
        }
    }
    assert!(answered > 0 && delegated > 0 && silent > 0, "odgovori {answered}, prepušteno {delegated}, šutnja {silent}");
    assert!(CASES.iter().any(|c| c.kind == PeriodKind::Week) && CASES.iter().any(|c| c.kind == PeriodKind::Month));
}

#[test] // 4: otisak reagira na pravila (bez pečata; trenutni sadržaj)
fn digest_reacts_to_a_rule_change() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let mut changed = v.clone();
    changed["v2"]["periods"]["week"]["situation_effect_bp"] = json!(326);
    let (a, b) = (Snapshot::from_value(&v).unwrap(), Snapshot::from_value(&changed).unwrap());
    assert_ne!(digest(&play(&a, &CASES[0])), digest(&play(&b, &CASES[0])));
}

#[test]
#[ignore = "piše pečat v2; pokreće se samo uz dizanje CORE_VERSION"]
#[allow(clippy::disallowed_methods)] // samo generator pečata čita okolinu
fn seal_v2_digest() {
    assert_eq!(std::env::var("HOTELIXSIM_SEAL_V2").as_deref(), Ok("1"));
    std::fs::create_dir_all(dir()).unwrap();
    let pretty = |v: &Value| format!("{}\n", serde_json::to_string_pretty(v).unwrap());
    let path = dir().join("v2-snapshot.json");
    if !path.exists() {
        std::fs::write(&path, pretty(&Snapshot::read_v2(&repo_dir()).unwrap().to_value())).unwrap();
    }
    let gpath = dir().join("v2-golden.json");
    let mut g = if gpath.exists() { golden() } else {
        Golden { snapshot: fingerprint_value(&frozen_value()), digests: BTreeMap::new() }
    };
    assert!(!g.digests.contains_key(CORE_VERSION), "pečat za {CORE_VERSION} postoji i ne prepisuje se — digni verziju");
    g.digests.insert(CORE_VERSION.to_string(), digests(&Snapshot::from_value(&frozen_value()).unwrap()));
    std::fs::write(&gpath, pretty(&serde_json::to_value(&g).unwrap())).unwrap();
}
```

**Koraci graditelja:**
1. `cargo test -p game --test v2_sealed` nad čuvarovim testom → 1–3 padaju s „nema tests/fixtures_v2/…", 4 prolazi (RED).
   Test se ne mijenja osim da se prevede. **Ne pečatiti 2.1.0** (Rizik).
2. `crates/sim-hotel-v2/Cargo.toml:3` → `version = "2.2.0"`; komentar `lib.rs:50`; `renewal_v2.rs:667` (makni
   `assert_eq!(sim_hotel_v2::CORE_VERSION, "2.1.0");`) i `:693` (`current: sim_hotel_v2::CORE_VERSION.into()`).
3. `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest; echo EXIT=$?` → EXIT=0; nastaju
   `fixtures_v2/v2-snapshot.json` (~250 KB) i `v2-golden.json` s ključem `"2.2.0"` i 6 otisaka.
4. Dokaz da pečat čuva: (a) drugo pokretanje generatora pada („postoji i ne prepisuje se"); (b) privremeno u
   `crates/sim-hotel-v2/src/market.rs:23` promijeni `840` u `841` → test 1 pada s „obračun v2 se promijenio" → vrati
   (`git diff crates/sim-hotel-v2/src` prazan). Izlaze (a) i (b) u izvještaj. Commit: `Cargo.lock` samo redak verzije.

**Provjere (iz korijena stabla, svaka zasebno, `; echo EXIT=$?`):**
- `cargo test -p game --test v2_sealed` → `ok. 4 passed; 0 failed; 1 ignored`; `--test renewal_v2`, `--test v1_frozen`, `--test fixtures_sealed` → ok.
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` ·
  `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` (dira paket jezgre) · `corepack pnpm check:kod`.
- Balans bajt-isti prije/poslije (verzija ne ulazi u obračun): `cargo run -q -p balance --release -- --seeds 100 --check`,
  `-- --model v2 --round week --seeds 1000 --check` i `--round month` → izlaz u scratchpad prije koraka 2 i poslije; `diff`
  prazan, isti EXIT. `git diff --stat` dira samo datoteke iz Files.

**Rizik i odluke:**
- 🟨 **Dizanje na 2.2.0 u ovoj kartici.** 2.1.0 je postavljen kod O-1; J-3, G-1 i L1-11 su poslije promijenili brojke, pa
  „2.1.0" danas znači dvije različite igre; pečat na 2.1.0 naknadno bi ozakonio tihu promjenu. Posljedica: Leonove lokalne
  v2 igre dobiju 409 `game_outdated` (web još nema poruku — #6, Faza C); v2 nije na `main`, pa tuđih igara nema. Ako Leon
  kaže drukčije: preskoči korak 2 i pečatiraj 2.1.0.
- **Svaka buduća promjena brojki = nova verzija = stare v2 igre 409** dok Faza C (#6) ne postoji. Namjerno: bolje 409 nego
  tiho drukčija igra; Leon nakon takve promjene počinje novu igru.
- **Pečat je širi od jezgre** (pokriva i `content::instance`, `game::drawn`, robote u `game`): i promjena u tim paketima
  koja mijenja brojke traži dizanje `sim_hotel_v2::CORE_VERSION`, jer je to jedini broj koji `check_version` gleda. Graditelji
  budućih kartica to moraju znati — kontrolor neka doda redak u `global-constraints.md` popravka.
- **Provjeravatelj neka gleda:** da generator ne prepisuje postojeći ključ; da se `fixtures_v2/` ne preklapa s
  `fixtures/`; da test 3 stvarno broji prepuštanje (nije 0); da je `Cargo.lock` promijenjen samo u verziji.

**Leonov dio:** samo obavijest (ne odluka o igri): nakon ove kartice njegove postojeće lokalne v2 igre više se ne otvaraju
(409); otvara novu igru. Ako želi zadržati staru igru za pregled, javlja prije koraka 2 (tada 🟨 alternativa gore).

## Review Focus kandidati
1. **Konverzacijske situacije sa slobodnim odgovorom i presudom AI-ja** nisu u pečatu (igrač bira samo ponuđene odgovore,
   bez `record_verdict`); promjena puta presude → kvaliteta ne bi pala ovdje.
2. **Igra iz stvarnog starog dnevnika v2** (fold zapisanog loga, ne svježa igra) nije pokrivena; fold početno stanje
   računa trenutnim kodom (01 nalaz 3), pa bi pomak koji ne mijenja svježu igru, a mijenja čitanje stare, prošao.
3. **Promjena plana igrača** (cijene, sezonci, obnova) nije u slučajevima: igrač vozi prenesen zadani plan; putove plana
   pokrivaju samo roboti (tri strategije).
