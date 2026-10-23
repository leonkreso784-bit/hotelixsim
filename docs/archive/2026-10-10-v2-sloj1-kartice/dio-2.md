# 2026-10-10-v2-sloj1-kartice — dio 2

Arhiva (zamrznuto). Indeks: [2026-10-10-v2-sloj1-kartice.md](../2026-10-10-v2-sloj1-kartice.md).

### Task 3: Obračun razdoblja (tjedan ili mjesec) i ekvivalencija s v1

**Veličina:** L, 5–6 h · **Ovisi o:** Task 2 · **Paralelno s:** Task 4, Task 5 · **Model:** sonnet (opus ako se
ekvivalencija ne poklapa nakon prvog pokušaja)

**Cilj:** `resolve_period` računa bilo koje razdoblje po **Pravilima razdoblja** (gore); mjesečna razdoblja daju
**točno** v1 brojke, tjedna sezona od 17 krugova radi bez greške i ponovljivo.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{plan,market,resolve,explain,indices,robots,season}.rs`,
  `crates/sim-hotel-v2/src/lib.rs` (samo ponovni izvozi: `pub use resolve::{PeriodReport, ResolveError,
  resolve_period};`, `pub use season::{SeasonResult, run_season, run_season_with, situations_for};`,
  `pub use plan::{PeriodPlan, PlanError, PlanLimits};`, `pub use robots::robot_plan;` — testovi ispod ih uvoze kroz
  `use sim_hotel_v2::*`)
- Create: `crates/sim-hotel-v2/tests/period.rs`; Modify: `tests/equivalence.rs`
- Ne dirati: `calendar.rs`, `streams.rs`, `catalog.rs` (osim ako fali getter — tada samo dodaj), sve izvan paketa.

**Interfaces:**
- Consumes: Task 2 (`calendar::*`, `Catalog::coeffs`, `PeriodCoefficients`).
- Produces (potpisi koje troše Task 6, 11, 16):

```rust
pub fn resolve_period(state: &HotelState, p: &Period, plan: &PeriodPlan, situations: &[SituationOutcome], cat: &Catalog) -> Result<PeriodReport, ResolveError>;
impl PeriodPlan {
    pub fn default_for(state: &HotelState, p: &Period, cat: &Catalog) -> Result<PeriodPlan, CatalogError>;
    pub fn validate(&self, state: &HotelState, p: &Period, cat: &Catalog) -> Result<(), PlanError>;
    /// Plan prošlog kruga za novi krug: obnova se ne ponavlja; cijena, marketing, sezonci i plaća stisnu se u granice.
    pub fn carry_into(&self, state: &HotelState, p: &Period, cat: &Catalog) -> Result<PeriodPlan, CatalogError>;
    pub fn limits(state: &HotelState, p: &Period, cat: &Catalog) -> Result<PlanLimits, CatalogError>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanLimits { pub price_min_cents: i64, pub price_max_cents: i64, pub marketing_max_cents: i64, pub seasonal_min: u16, pub seasonal_max: u16, pub wage_min_pct: u16, pub wage_max_pct: u16, pub renovation_available: bool }
impl RefTable { pub fn compute(cat: &Catalog, category: Category, periods: &[Period]) -> Result<RefTable, ResolveError>; }
pub fn start_indices(s: &HotelState, cat: &Catalog, refs: &RefTable) -> Result<IndexSet, ResolveError>;
pub fn period_indices(r: &PeriodReport, cat: &Catalog, refs: &RefTable) -> Result<IndexSet, ResolveError>;
pub fn robot_plan(strategy: Strategy, s: &HotelState, p: &Period, cat: &Catalog, rng: &mut Rng) -> Result<PeriodPlan, ResolveError>;
/// Situacije čiji mjesec počinje u ovom razdoblju (razdoblje koje sadrži 1. dan mjeseca; 🟨 P13).
pub fn situations_for<'a>(p: &Period, all: &'a [SituationTemplate]) -> Vec<&'a SituationTemplate>;
pub fn run_season(profile: Profile, strategy: Strategy, seed: u64, kind: PeriodKind, cat: &Catalog, situations: &[SituationTemplate], refs: &RefTable) -> Result<SeasonResult, ResolveError>;
pub fn run_season_with(profile: Profile, kind: PeriodKind, cat: &Catalog, situations: &[SituationTemplate], refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>) -> Result<SeasonResult, ResolveError>;
```

`resolve_month` se briše (zamjena je `resolve_period(.., &month_period(m), ..)`).

- [ ] **Step 1: Testovi (padaju)** — `tests/period.rs`:

```rust
mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code, season_periods};
use sim_hotel_v2::*;

fn cat() -> Catalog {
    degenerate(&common::test_catalog())
}

fn refs(c: &Catalog, kind: PeriodKind) -> RefTable {
    RefTable::compute(c, Category::Four, &season_periods(kind)).unwrap()
}

#[test]
fn weekly_season_runs_for_every_profile_and_strategy() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    for p in Profile::ALL {
        for s in Strategy::ALL {
            let out = run_season(p, s, 7, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
            assert_eq!(out.periods.len(), 17, "{p:?} {s:?}");
        }
    }
}

#[test]
fn weekly_season_is_reproducible() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    let a = run_season(Profile::C, Strategy::Random, 11, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    let b = run_season(Profile::C, Strategy::Random, 11, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    assert_eq!(a, b);
}

fn sum_lines(out: &SeasonResult, var: Var, cause: Cause) -> i64 {
    out.periods
        .iter()
        .flat_map(|(r, _)| r.summary.lines.iter())
        .filter(|l| l.var == var && l.cause == cause)
        .map(|l| l.amount)
        .sum()
}

fn default_season(kind: PeriodKind) -> SeasonResult {
    let c = cat();
    let r = refs(&c, kind);
    run_season_with(Profile::C, kind, &c, &[], &r, &mut |s, p| Ok(PeriodPlan::default_for(s, p, &c)?), &mut |_| Ok(0)).unwrap()
}

#[test]
fn weekly_fixed_costs_over_season_equal_monthly() {
    let w = default_season(PeriodKind::Week);
    let m = default_season(PeriodKind::Month);
    let fixed = common::profile_c().fixed_cents;
    assert_eq!(sum_lines(&m, Var::Costs, Cause::FixedCosts), 4 * fixed);
    assert_eq!(sum_lines(&w, Var::Costs, Cause::FixedCosts), 4 * fixed);
}

#[test]
fn wages_of_week_two_are_the_prorated_june_wage() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    let plan = PeriodPlan::default_for(&s, &t02, &c).unwrap();
    let r = resolve_period(&s, &t02, &plan, &[], &c).unwrap();
    let staff = i64::from(plan.seasonal_target) + i64::from(s.permanent);
    let monthly = staff * (c.base.category(Category::Four).unwrap().wage_cents / 100) * i64::from(plan.wage_pct);
    let wage = r.summary.lines.iter().find(|l| l.var == Var::Costs && l.cause == Cause::Wage).unwrap().amount;
    assert_eq!(wage, monthly * 14 / 30 - monthly * 7 / 30);
}

#[test]
fn group_nights_split_across_weeks_sum_to_the_block() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    let out = run_season(Profile::C, Strategy::Caring, 7, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    let total: u32 = out.periods.iter().map(|(p, _)| p.summary.market.group_room_nights).sum();
    assert_eq!(total, 100); // T2: 20 soba × 5 noći u kolovozu
    assert!(out.periods.last().unwrap().0.after.groups.is_empty());
}

#[test]
fn carried_price_is_clamped_into_new_period_bounds() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let aug = month_period(Month::Aug);
    let mut plan = PeriodPlan::default_for(&s, &aug, &c).unwrap();
    let aug_max = PeriodPlan::limits(&s, &aug, &c).unwrap().price_max_cents;
    plan.price_cents = aug_max;
    plan.renovation = true;
    let sep = month_period(Month::Sep);
    let carried = plan.carry_into(&s, &sep, &c).unwrap();
    assert_eq!(carried.price_cents, PeriodPlan::limits(&s, &sep, &c).unwrap().price_max_cents);
    assert!(!carried.renovation);
    assert_eq!(carried.validate(&s, &sep, &c), Ok(()));
}

#[test]
fn default_weekly_price_is_day_weighted_reference_in_whole_euros() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let june = c.base.category(Category::Four).unwrap().price_ref_cents.get(Month::Jun);
    let july = c.base.category(Category::Four).unwrap().price_ref_cents.get(Month::Jul);
    let expected_cents = (2 * june + 5 * july + 3) / 7; // div_round
    let expected = (expected_cents + 50) / 100 * 100;
    assert_eq!(PeriodPlan::default_for(&s, &t05, &c).unwrap().price_cents, expected);
}

#[test]
fn situations_attach_to_period_containing_first_of_month() {
    let mut sits = common::test_situations();
    sits[1].month = Month::Jul;
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let t06 = period_by_code(PeriodKind::Week, "t06").unwrap();
    assert_eq!(situations_for(&t05, &sits).iter().map(|t| t.code.as_str()).collect::<Vec<_>>(), ["T2"]);
    assert!(situations_for(&t06, &sits).is_empty());
}

#[test]
fn weekly_and_monthly_default_seasons_end_close() {
    let w = default_season(PeriodKind::Week);
    let m = default_season(PeriodKind::Month);
    let (wi, mi) = (&w.periods.last().unwrap().1, &m.periods.last().unwrap().1);
    for (i, v) in wi.iter() {
        let diff = i32::from(v.hundredths()) - i32::from(mi.get(i).hundredths());
        assert!(diff.abs() <= 1_000, "{i:?}: tjedno {} mjesečno {}", v.hundredths(), mi.get(i).hundredths());
    }
}
```

(U testovima je obična aritmetika dopuštena — lintovi `forbid` vrijede za `src`, ne za `tests`; ako clippy ipak
prigovori, koristi `checked_*` u testu.)

U `tests/equivalence.rs` zamijeni poziv s `run_season(p, s, seed, PeriodKind::Month, &v2, sits, &r2)` i
`RefTable::compute(&v2, category, &season_periods(PeriodKind::Month))`; test ostaje isti i mora ostati zelen.

- [ ] **Step 2:** `cargo test -p sim-hotel-v2` → FAIL (potpisi ne postoje).
- [ ] **Step 3: Pretvori obračun** redak po redak po tablici **Pravila razdoblja**. Redoslijed rada (svaki korak
  `cargo test -p sim-hotel-v2 --test equivalence` mora ostati zelen): (1) `days` → `p.days`, reference i plan sezonaca
  kroz `avg_*`; (2) koeficijenti pomaka iz `cat.coeffs(p.kind)`; (3) `prorate` za plaće, fiksno, održavanje, obuku,
  rast iskustva, trošenje opreme, odgovaranje; (4) marketing (polovica i najviše razmjerno); (5) odlasci i kamata s
  `D₀`; (6) grupe po dijelovima mjeseca; (7) `RefTable` po popisu razdoblja (`start` iz prvog razdoblja); (8)
  `robot_plan` s referencama razdoblja i razmjernim marketingom (`500_000` i `2_500_000` iz v1 `Strategy::plan` su
  mjesečni iznosi → `prorate`); (9) `explain`: `−400` i `300` postaju razmjerni iznosi iz koraka (ne konstante).
  Generator kocke u `run_season`: `Rng::derive(seed, &[u64::from(p.index), label("robot:plan")])` za plan i
  `Rng::derive(seed, &[pozicija mjeseca situacije, label(&t.code)])` za izbor — u mjesečnoj igri isto kao v1. To je
  **jedina imenovana iznimka** od pravila „slučajnost samo kroz `streams::stream`" (Global Constraints); nigdje drugdje
  u `src` ovog paketa se `Rng::derive` ne zove.
  `carry_into`: cijena na najbliži cijeli euro unutar `limits` (ispod najmanje → najmanja, iznad najveće → najveća),
  marketing `min(max)`, sezonci u `[seasonal_min, seasonal_max]`, plaća u granice, `renovation = false`.
- [ ] **Step 4:** `cargo test -p sim-hotel-v2; echo EXIT=$?` → svi zeleni (uklj. ekvivalenciju).
- [ ] **Step 5: Sve provjere** (i `node scripts/probe-mutacija.mjs`) i commit „Sloj 1/3: obračun razdoblja (tjedan ili
  mjesec) uz ekvivalenciju s v1".

**Prihvaćanje:** ekvivalencija zelena bez ijedne iznimke; 17 tjednih krugova za sve profile i strategije; nijedna
v1 konstanta iz tablice **Pravila razdoblja** ne ostaje tvrdo zapisana u `resolve.rs`/`explain.rs`
(`grep -n "3_000\|5_000\|2_500\|-400\|300," crates/sim-hotel-v2/src/resolve.rs` — svaki pogodak je obrazložen
komentarom ili nije pravilo razdoblja, npr. formula kvalitete).

---

### Task 4: Snimak pravila — polje `model` i sadržaj v2 (koeficijenti razdoblja)

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 1, Task 2 (test `v1_fixture()` čita
`crates/game/tests/fixtures/v1-snapshot.json` iz Taska 1) · **Paralelno s:** Task 3, Task 5 · **Model:** sonnet

**Cilj:** snimak zna kojim se modelom igra računa (`"v1"` kad ga nema) i nosi odjeljak v2; stari snimci daju isti
otisak bajt po bajt.

**Files:**
- Create: `crates/content/src/v2.rs`, `content/hotel/v2/periods.json`, `crates/content/tests/v2.rs`
- Modify: `crates/content/src/lib.rs`, `crates/content/Cargo.toml` (`sim-hotel-v2.workspace = true`)
- Ne dirati: `content/hotel/{catalog,profiles,situations}.json`, `crates/content/tests/fixtures/*`, `crates/game/**`
  (uključujući fixture Taska 1 — samo se čita).

**Interfaces (Produces):**

```rust
// crates/content/src/lib.rs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model { #[default] V1, V2 }

pub struct Content {
    pub catalog: Catalog,
    pub situations: Vec<SituationTemplate>,
    pub ai: BTreeMap<String, SituationAi>,
    /// Katalog v2 (`Some` samo za snimak s `model = v2`); `base` je isti `catalog`.
    pub v2: Option<sim_hotel_v2::Catalog>,
}

pub struct Snapshot {
    catalog: CatalogDto,
    profiles: BTreeMap<String, ProfileDto>,
    situations: Vec<SituationDto>,
    #[serde(default, skip_serializing_if = "is_v1")]
    model: Model,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    v2: Option<v2::V2Dto>,
}
impl Snapshot {
    pub fn model(&self) -> Model;
    /// v1 datoteke iz `dir` + `dir/v2/*.json`; `model = v2`.
    pub fn read_v2(dir: &Path) -> Result<Snapshot, ContentError>;
    /// v2 snimak „kao v1" za ekvivalencijske testove (MODEL_V2 §2 t. 6); odjeljak v2 = `sim_hotel_v2::degenerate`.
    pub fn degenerate_v2(&self) -> Snapshot;
}
```

`content/hotel/v2/periods.json`:

```json
{
  "month": { "morale_step_bp": 3000, "satisfaction_step_bp": 5000, "rating_step_bp": 2500, "seasonal_step": 10 },
  "week": { "morale_step_bp": 800, "satisfaction_step_bp": 1500, "rating_step_bp": 700, "seasonal_step": 5 }
}
```

- [ ] **Step 1: Testovi (padaju)** — `crates/content/tests/v2.rs`:

```rust
use content::{Model, Snapshot, fingerprint_value, repo_dir};
use sim_hotel_v2::PeriodKind;

fn v1_fixture() -> serde_json::Value {
    let p = repo_dir().join("../../crates/game/tests/fixtures/v1-snapshot.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn old_v1_snapshot_round_trips_without_new_keys() {
    let v = v1_fixture();
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.model(), Model::V1);
    let again = s.to_value();
    assert!(again.get("model").is_none() && again.get("v2").is_none());
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn current_v1_snapshot_has_no_model_key() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V1);
    assert!(s.to_value().get("model").is_none());
    assert!(s.build().unwrap().v2.is_none());
}

#[test]
fn v2_snapshot_builds_with_period_coefficients() {
    let s = Snapshot::read_v2(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V2);
    assert_eq!(s.to_value()["model"], "v2");
    let c = s.build().unwrap();
    let v2 = c.v2.unwrap();
    assert_eq!(v2.base, c.catalog);
    let m = v2.coeffs(PeriodKind::Month).unwrap();
    assert_eq!((m.morale_step_bp, m.satisfaction_step_bp, m.rating_step_bp), (3_000, 5_000, 2_500));
    let w = v2.coeffs(PeriodKind::Week).unwrap();
    assert!(w.morale_step_bp < m.morale_step_bp && w.satisfaction_step_bp < m.satisfaction_step_bp);
}

#[test]
fn v2_snapshot_round_trip_keeps_fingerprint() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn model_and_section_must_agree() {
    let mut only_model = Snapshot::read(&repo_dir()).unwrap().to_value();
    only_model["model"] = "v2".into();
    assert!(Snapshot::from_value(&only_model).unwrap().build().is_err());
    let mut only_section = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    only_section.as_object_mut().unwrap().remove("model");
    assert!(Snapshot::from_value(&only_section).unwrap().build().is_err());
}

#[test]
fn degenerate_v2_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    assert_eq!(c.v2.unwrap(), sim_hotel_v2::degenerate(&c.catalog));
}
```

- [ ] **Step 2:** `cargo test -p content --test v2` → FAIL.
- [ ] **Step 3: Implementiraj.** `v2.rs`: `pub(crate) struct V2Dto { periods: BTreeMap<String, PeriodCoefficientsDto> }`
  (`deny_unknown_fields`), `read_v2_dir(dir)` čita `dir.join("v2/periods.json")` u `periods`, `build_v2(&V2Dto,
  &Catalog) -> Result<sim_hotel_v2::Catalog, ContentError>` (nepoznata vrsta razdoblja → `Invalid`), `degenerate_dto(&Catalog)`.
  `build_v2` **ne piše literal** `sim_hotel_v2::Catalog { base, periods }`, nego `let mut c =
  sim_hotel_v2::degenerate(&base); c.periods = <iz DTO-a>;` — tako Task 9 smije dodati polja u `Catalog` (odjeli,
  profili v2, ekonomija; do Taska 10 ostaju degenerirana) bez diranja `crates/content`.
  `build()` gradi v1 katalog kao danas, pa: `model == V2 && v2.is_some()` → `Some(build_v2)` i `validate()`;
  `model == V1 && v2.is_none()` → `None`; inače `ContentError::Invalid("model i odjeljak v2 se ne slažu")`.
- [ ] **Step 4:** `cargo test -p content; echo EXIT=$?` → zeleno, uključujući **postojeće** testove otiska
  (`crates/content/tests/snapshot.rs`, korak 3).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/4: snimak pravila zna model; sadržaj v2 s koeficijentima razdoblja".

**Prihvaćanje:** postojeći testovi otiska nepromijenjeni i zeleni; Task 1 `v1_frozen` zelen (Task 1 je spojen prije
ovog taska); `build_v2` počinje od `sim_hotel_v2::degenerate(&base)` (u `crates/content/src/v2.rs` nema literala
`sim_hotel_v2::Catalog { … }` — recenzent provjerava).

---

### Task 5: Ključ kruga u igri (v1, ponašanje isto)

**Veličina:** L, 5–7 h (širok, mehanički) · **Ovisi o:** Task 1 · **Paralelno s:** Task 2, 3, 4 · **Model:** sonnet

**Cilj:** `game`, `app` i `api` više ne koriste `Month` kao ključ kruga nego `RoundKey` (`Month(m)` ili `Week(n)`),
a JSON svih v1 događaja i poslova ostaje bajt-isti. Tjedni još ne postoje u igri (to je Task 6) — ovo je čisto
preimenovanje tipa ključa.

**Files:**
- Create: `crates/game/src/round.rs`, `crates/game/tests/round_key.rs`
- Modify: `crates/game/src/{lib,game,event,command,resolve,view,feed,ai_input,talk,verdict,mentor,kpi}.rs`,
  `crates/game/tests/**` (mehanički), `crates/app/src/{views,commands,jobs,talk}.rs`, `crates/app/tests/**`,
  `crates/api/src/{schema,games,hotels,explanation,kpi}.rs`, `crates/api/tests/**`
- Mehaničke izmjene izvan paketa i u testu Taska 1 (Global Constraints, „Mehaničke izmjene literala"; očekivanja se
  ne mijenjaju):
  - `crates/game/tests/v1_frozen.rs` — **samo** maknuti `.into()` u
    `old_v1_game_rounds_resolve_to_the_stored_numbers` (`before.resolve_round(stored.month.into())` →
    `before.resolve_round(stored.month)`) i u `old_v1_game_resolve_job_replays_the_stored_system_events`
    (`g.resolve_job(c.month.into(), false)` → `g.resolve_job(c.month, false)`): nakon ovog taska `stored.month` i
    `c.month` su već `RoundKey`, a `.into()` u parametar `impl Into<RoundKey>` je greška E0283 (provjereno).
    Fixturei (`crates/game/tests/fixtures/**`) ostaju netaknuti.
  - `crates/ai/tests/request.rs:133-136` (`sample_debrief`): `month: MonthCode::Jul, next_month: Some(MonthCode::Aug),
    prev_month: Some(MonthCode::Jun)` → `round: game::RoundKey::parse("jul").unwrap(), next_round:
    game::RoundKey::parse("aug"), prev_round: game::RoundKey::parse("jun")` (isto u `crates/game/tests/mentor.rs:49-52`,
    koji je već pod `crates/game/tests/**`).
- Ne dirati: `crates/sim-hotel/**`, `crates/content/**`, `crates/game/tests/fixtures/**`, `crates/game/src/dto.rs`
  (`MonthCode` ostaje kao **kalendarski** mjesec u `MonthReportDto.month` i `GroupBlockDto.month`), `openapi.json`
  (ne smije se promijeniti — provjera `corepack pnpm check:api`), `crates/ai/src/**` (čita samo `debrief_facts`).

**Interfaces (Produces):**

```rust
// crates/game/src/round.rs
/// Ključ kruga (MODEL_V2 §4.4): `Month(m)` → "jun"…; `Week(n)`, n = 1..=17 → "t01"…"t17".
/// Serde je kod (niz), isti kao stari `MonthCode` za mjesece — v1 događaji su bajt-isti.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoundKey { Month(Month), Week(u8) }
pub const MAX_WEEK: u8 = 17;
impl RoundKey {
    pub fn code(self) -> String;
    pub fn parse(s: &str) -> Option<RoundKey>;  // "jun".."sep", "t01".."t17"; ništa drugo
    pub fn month(self) -> Option<Month>;
}
impl From<Month> for RoundKey;
impl Serialize for RoundKey; impl<'de> Deserialize<'de> for RoundKey;
```

Promjene tipova (sve s `RoundKey`, imena JSON polja ostaju `month`/`months`):
`GameCreated.months: Vec<RoundKey>`; `PlanSet/AnswerChosen/RoundClosed/RoundResolved/ChatMessageSent/CharacterReplied/
ResponseSubmitted/VerdictRecorded/DebriefRecorded.month: RoundKey`; `Game.months` → `Game.rounds: Vec<RoundKey>`;
`closed/resolved: BTreeMap<RoundKey, …>`; `HotelRecord.{plans, debriefs}` po `RoundKey`, `answers/chats/responses` po
`(RoundKey, String)`; `current_month()` → `current_round()`; metode koje su primale `Month` (`status`, `state_at`,
`effective_plan`, `set_plan`, `close_round`, `resolve_round`, `resolve_job`, `robot_moves`, `situations_in`,
`unanswered_count`, `record_debrief`, `debrief_input`, `pending_verdicts`) primaju `impl Into<RoundKey>` (testovi s
`Month::Jun` se tako ne mijenjaju). `situations_in(r)`: za `RoundKey::Month(m)` situacije mjeseca `m`, za tjedan
prazno (Task 6). Pogledi: `MonthScore.round`, `LastMonth.round`, `PlanState.round`, `RoundView.round`,
`SituationsView.round`, `FeedItem::*.round`, `KpiMonth.round`, `DebriefInput.{round, next_round, prev_round}`.
Mentor (`crates/game/src/mentor.rs:180`): `month_locative(m: MonthCode)` postaje `round_locative(r: RoundKey) ->
&'static str` (`RoundKey::Month` → „u lipnju" … „u rujnu" kao danas; `RoundKey::Week(_)` → „u ovom tjednu", pravi
tekst tjedna dolazi u Tasku 8), a pozivi na `mentor.rs:861-862` čitaju `input.round` / `input.next_round` — tako
`MonthCode` ostaje samo u `dto.rs` (Prihvaćanje) i v1 tekst mentora je isti.
Poslovi (`crates/app/src/jobs.rs`) `ResolvePayload.month`, `DebriefPayload.month`: `RoundKey` (JSON isti); ključevi
poslova `resolve:<game>:<code>` i `debrief:<game>:<code>:<hotel>` isti tekst.
API: `parse_month` → `parse_round(s) -> Result<RoundKey, Problem>` (404 `unknown_month` za neispravan kod);
`month` polja u tijelima ostaju `MonthCode` i pune se iz `round.month()` (za `Week` → 500 `internal`, dok Task 7
ne doda kalendar; u v1 igrama se nikad ne događa).

- [ ] **Step 1: Testovi ključa (padaju)** — `crates/game/tests/round_key.rs`:

```rust
use game::RoundKey;
use sim_hotel::Month;

#[test]
fn month_keys_print_and_parse_like_old_month_codes() {
    for m in Month::SEASON {
        let k = RoundKey::from(m);
        assert_eq!(k.code(), m.code());
        assert_eq!(RoundKey::parse(m.code()), Some(k));
        assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{}\"", m.code()));
    }
}

#[test]
fn week_keys_print_and_parse() {
    assert_eq!(RoundKey::Week(7).code(), "t07");
    assert_eq!(RoundKey::parse("t01"), Some(RoundKey::Week(1)));
    assert_eq!(RoundKey::parse("t17"), Some(RoundKey::Week(17)));
    assert_eq!(serde_json::from_str::<RoundKey>("\"t03\"").unwrap(), RoundKey::Week(3));
}

#[test]
fn bad_codes_are_rejected() {
    for bad in ["t00", "t18", "t7", "T07", "Jun", "jun ", "", "t1x", "week"] {
        assert_eq!(RoundKey::parse(bad), None, "{bad:?}");
        assert!(serde_json::from_str::<RoundKey>(&format!("\"{bad}\"")).is_err());
    }
}

#[test]
fn keys_order_by_season() {
    assert!(RoundKey::from(Month::Jun) < RoundKey::from(Month::Jul));
    assert!(RoundKey::from(Month::Aug) < RoundKey::from(Month::Sep));
    assert!(RoundKey::Week(2) < RoundKey::Week(10));
}
```

- [ ] **Step 2:** `cargo test -p game --test round_key` → FAIL.
- [ ] **Step 3: `round.rs`** (deserializacija: `String::deserialize` pa `parse`, inače `de::Error::custom("nepoznat
  krug …")`), izvoz `pub use round::{MAX_WEEK, RoundKey};` u `lib.rs`.
- [ ] **Step 4: Zamjena tipa** u `game` (popis gore), pa `app`, pa `api`. Prevoditelj vodi: nakon svake datoteke
  `cargo check --workspace --all-targets`. Pozor: `BTreeMap` po `RoundKey` je u sezonskom redoslijedu (izvedeni `Ord`),
  kao i po `Month` — iteracije po mapi ostaju istog redoslijeda.
- [ ] **Step 5:** `cargo test --workspace; echo EXIT=$?` → sve zeleno, posebno `game --test v1_frozen` (6 passed).
  `corepack pnpm check:api; echo EXIT=$?` → EXIT=0 (`openapi.json` nepromijenjen).
- [ ] **Step 6: Sve provjere i commit** „Sloj 1/5: ključ kruga (RoundKey) umjesto mjeseca; v1 događaji bajt-isti".

**Prihvaćanje:** `git diff --stat -- openapi.json packages/api-client crates/game/tests/fixtures crates/sim-hotel` prazno;
`grep -rn "MonthCode" crates/game/src` pogađa samo `dto.rs` (kalendarski mjesec).

---

### Task 6: Jedno mjesto grananja — v2 igra (razdoblja, prenošenje plana, roboti, verzija)

**Veličina:** L, 5–7 h · **Ovisi o:** Task 3, 4, 5 (spojeni) · **Paralelno s:** ništa (VRATA) · **Model:** opus
(arhitektonski rez kroz cijeli `game`)

**Cilj:** igra se stvara i igra na v1 ili v2 prema snimku; v2 igra ima 4 mjesečna ili 17 tjednih krugova, plan se
prenosi iz kruga u krug, roboti igraju, verzija jezgre v2 se provjerava; sve grananje je u `model.rs`.

**Files:**
- Create: `crates/game/src/model.rs`, `crates/game/src/dto_v2.rs`, `crates/game/tests/model_v2.rs`,
  `crates/game/tests/equivalence.rs`
- Modify: `crates/game/Cargo.toml` (`sim-hotel-v2.workspace = true`), `crates/game/src/{lib,game,event,command,resolve,
  view,ai_input,error,kpi,talk,verdict}.rs`, `crates/app/src/error.rs`, `crates/app/src/commands.rs` (`set_plan` ide
  kroz `Game::plan_from_dto`), `crates/api/src/problem.rs` (samo novi kodovi grešaka), minimalno `crates/app/src/*.rs`
  i `crates/api/src/*.rs` da se prevede (bez novih ruta i polja)
- Mehaničke izmjene literala (Global Constraints; očekivanja se ne mijenjaju): `crates/game/tests/events.rs:43`
  (`GameCreated { …, rounds: vec![] }`) i `crates/game/tests/events.rs:85` (`HotelResolution { …, detail: None }`).
- Ne dirati: `crates/sim-hotel/**`, `crates/sim-hotel-v2/**` (ako fali javna funkcija → zapiši u izvještaj, dodaj
  najmanji getter), fixturei, `openapi.json` osim novih kodova grešaka.

**Interfaces:**
- Consumes: Task 3 (`resolve_period`, `PeriodPlan::{default_for, validate, carry_into, limits}`, `RefTable::compute`,
  `start_indices`, `period_indices`, `robot_plan`, `situations_for`), Task 4 (`Snapshot::model()`, `Content.v2`),
  Task 5 (`RoundKey`).
- Produces:

```rust
// crates/game/src/model.rs — JEDINO mjesto s `match` na model
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model { V1, V2 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreState { V1(sim_hotel::HotelState), V2(Box<sim_hotel_v2::HotelState>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorePlan { V1(sim_hotel::MonthPlan), V2(sim_hotel_v2::PeriodPlan) }
pub(crate) enum CoreRefs { V1(sim_hotel::RefTable), V2(sim_hotel_v2::RefTable) }
/// Jedan krug igre: ključ i kalendar (za v1 uvijek cijeli mjesec).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundInfo { pub key: RoundKey, pub period: sim_hotel_v2::Period }
pub(crate) struct Resolved { pub report: MonthReportDto, pub detail: Option<DetailDto>, pub indices: IndexSet }

impl Model {
    pub fn of(content: &content::Content, snapshot_model: content::Model) -> Model;
    /// "v1" | "v2" — jedini put kojim app/api pretvaraju model u svoj kod (bez `match` na `Model` izvan ove datoteke).
    pub fn code(self) -> &'static str;
    pub fn core_version(self) -> &'static str;              // sim_hotel::CORE_VERSION | sim_hotel_v2::CORE_VERSION
    pub(crate) fn start_state(self, c: &Content, p: Profile) -> Result<CoreState, GameError>;
    pub(crate) fn refs(self, c: &Content, category: Category, rounds: &[RoundInfo]) -> Result<CoreRefs, GameError>;
    pub(crate) fn start_indices(self, s: &CoreState, c: &Content, r: &CoreRefs) -> Result<IndexSet, GameError>;
    pub(crate) fn default_plan(self, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<CorePlan, GameError>;
    pub(crate) fn carry_plan(self, prev: &CorePlan, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<CorePlan, GameError>;
    pub(crate) fn validate_plan(self, p: &CorePlan, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<(), GameError>;
    pub(crate) fn robot_plan(self, st: Strategy, s: &CoreState, r: &RoundInfo, c: &Content, rng: &mut Rng) -> Result<CorePlan, GameError>;
    pub(crate) fn resolve(self, s: &CoreState, r: &RoundInfo, p: &CorePlan, o: &[SituationOutcome], c: &Content, refs: &CoreRefs) -> Result<Resolved, GameError>;
    pub(crate) fn state_after(self, res: &HotelResolution) -> Result<CoreState, GameError>;
    pub(crate) fn plan_from_dto(self, d: &PlanDto) -> Result<CorePlan, GameError>;
    pub(crate) fn plan_limits(self, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<PlanLimits, GameError>;
    pub(crate) fn situations_in<'a>(self, r: &RoundInfo, all: &'a [SituationTemplate]) -> Vec<&'a SituationTemplate>;
}
impl CoreState { pub fn summary(&self) -> sim_hotel::HotelState; pub fn category(&self) -> Category; }
impl CorePlan { pub fn to_dto(&self) -> PlanDto; }
impl From<sim_hotel::MonthPlan> for CorePlan;

// crates/game/src/command.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameSettings { pub kind: sim_hotel_v2::PeriodKind }
/// v1: snimak `model = v1` i `kind = Month` → lipanj i srpanj (kao danas); v2: cijela sezona (🟨 P6).
pub fn new_game_with(game_id: GameId, seed: u64, snapshot: &content::Snapshot, guest: GuestId, hotel_ids: [HotelId; 4], settings: GameSettings) -> Result<Vec<(Actor, GameEvent)>, GameError>;
// `new_game(...)` ostaje i zove `new_game_with(.., GameSettings { kind: PeriodKind::Month })`.

// crates/game/src/event.rs — samo dodano
pub struct GameCreated { /* …postojeće… */
    /// v2: kalendar krugova u trenutku stvaranja (igra ga nikad ne računa ponovno). v1: prazno.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds: Vec<RoundDefDto>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundDefDto { pub code: RoundKey, pub kind: String, pub index: u8, pub start_day: u16, pub days: u8 }
pub struct HotelResolution { /* …postojeće… */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailDto>,
}

// crates/game/src/dto_v2.rs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailDto { pub after: HotelStateV2Dto }
/// Serde-zrcalo `sim_hotel_v2::HotelState` (polje za polje; Task 11 ga mijenja zajedno sa stanjem).
pub struct HotelStateV2Dto { /* polja v2 stanja */ }

// crates/game/src/error.rs — dodano
GameError::Outdated { created: String, current: String }   // v2 igra druge verzije jezgre v2
GameError::WrongModel                                      // metoda samo za drugi model
GameError::InvalidSettings                                 // npr. v1 + tjedan
GameError::PlanV2(sim_hotel_v2::PlanError)                 // greška plana v2 (v1 ostaje `Plan(sim_hotel::PlanError)`)

// crates/game/src/game.rs
impl Game {
    pub fn model(&self) -> Model;
    pub fn round_info(&self, r: impl Into<RoundKey>) -> Result<RoundInfo, GameError>;
    pub fn rounds_info(&self) -> Vec<RoundInfo>;
    pub fn state_at(&self, h: HotelId, r: impl Into<RoundKey>) -> Result<CoreState, GameError>;
    pub fn latest_state(&self, h: HotelId) -> Result<CoreState, GameError>;
    pub fn effective_plan(&self, h: HotelId, r: impl Into<RoundKey>) -> Result<CorePlan, GameError>;
    pub fn set_plan(&self, h: HotelId, r: impl Into<RoundKey>, plan: impl Into<CorePlan>) -> Result<GameEvent, GameError>;
    /// Plan iz tijela zahtjeva (app `set_plan`) za model ove igre; javni omotač oko `Model::plan_from_dto`.
    /// `h` i `r` koristi Task 11: v2 plan bez `departments` uzima postavke odjela iz `effective_plan(h, r)`.
    pub fn plan_from_dto(&self, h: HotelId, r: impl Into<RoundKey>, d: &PlanDto) -> Result<CorePlan, GameError>;
}
// crates/game/src/talk.rs — zamjenjuje `situation_month` (talk.rs:39-46)
impl Game {
    /// Krug u kojem se situacija igra i njen predložak, kroz `Model::situations_in` za svaki krug igre
    /// (`UnknownSituation` ako nije ni u jednom krugu ove igre).
    pub fn situation_round(&self, code: &str) -> Result<(RoundKey, &SituationTemplate), GameError>;
}
```

Situacija → krug: tri mjesta danas traže situaciju uvjetom `self.months.contains(&t.month)` — `situation_month`
(`crates/game/src/talk.rs:39-46`), provjera slobodnog odgovora (`crates/game/src/verdict.rs:117`) i `choose_answer`
(`crates/game/src/command.rs:160`); u tjednoj igri bi svaka situacija bila `UnknownSituation` (i e2e Taska 22 bi pao).
Sva tri (i `assess_input` u `ai_input.rs`) prelaze na `Game::situation_round`, a ključ odgovora/razgovora je vraćeni
`RoundKey`.

Pravila:
- **Model** dolazi iz snimka u `GameCreated` (`Snapshot::from_value(..).model()`); `core_version` mora biti
  `Model::core_version()`; za **v2** razliku `fold` vraća `GameError::Outdated` (🟨 P11), za **v1** sve ostaje kao
  danas (provjera samo u `resolve_job`).
- **Krugovi:** v1 → iz `GameCreated.months` (mjeseci) i `month_period`; v2 → iz `GameCreated.rounds` (spremljeni
  kalendar), a `months` sadrži iste kodove (provjera pri `fold`: isti popis, inače `corrupt`).
- **Plan:** v1 — zadnji `PlanSet` za taj krug ili **zadani** (kao danas); v2 — zadnji `PlanSet` za taj krug, inače
  plan **najbližeg ranijeg kruga** kroz `carry_into`, inače zadani.
- **Sažetak:** `HotelResolution.report` uvijek v1 oblika (`MonthReportDto::from(&report.summary)` za v2);
  `HotelResolution.plan = CorePlan::to_dto()`; `detail` samo za v2.
- **Situacije u krugu:** v1 po mjesecu; v2 `sim_hotel_v2::season::situations_for`.
- **Tokovi kocke robota:** `game:robot-plan` s `[pozicija kruga, slot]` (pozicija = indeks u popisu krugova; u v1 je
  jednaka poziciji mjeseca, pa su v1 nizovi isti).

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/model_v2.rs` (pomoćnici u istoj datoteci):

```rust
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

fn v2_events(kind: PeriodKind) -> Vec<Recorded> {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    new_game_with(GameId(Uuid::from_u128(1)), 42, &snap, common::GUEST, common::hotel_ids(), GameSettings { kind })
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

fn play_all(mut g: Game) -> Game {
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else { panic!("čeka") };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

#[test]
fn v2_weekly_game_has_17_rounds_and_resolves_all_with_robots() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    assert_eq!(g.model(), Model::V2);
    assert_eq!(g.rounds.len(), 17);
    assert_eq!(g.rounds_info()[4].period.days, 7);
    let g = play_all(g);
    assert_eq!(g.resolved.len(), 17);
    let last = g.resolved.values().last().unwrap();
    assert!(last.hotels.iter().all(|h| h.detail.is_some()));
}

#[test]
fn v2_monthly_game_has_four_rounds() {
    let g = play_all(Game::fold(&v2_events(PeriodKind::Month)).unwrap());
    assert_eq!(g.resolved.len(), 4);
}

#[test]
fn v1_with_weeks_is_invalid_settings() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let r = new_game_with(GameId(Uuid::from_u128(1)), 42, &snap, common::GUEST, common::hotel_ids(), GameSettings { kind: PeriodKind::Week });
    assert_eq!(r.unwrap_err(), GameError::InvalidSettings);
}

#[test]
fn v2_plan_carries_over_without_renovation() {
    let mut g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r1 = g.current_round().unwrap();
    let CorePlan::V2(mut p) = g.effective_plan(h, r1).unwrap() else { panic!("v2") };
    assert_eq!(p.marketing_cents, 0); // zadani plan: marketing 0 — pa 300 000 u t02 može doći samo prenošenjem
    p.marketing_cents = 300_000; // ≤ najviše za tjedan: prorate(4 000 000, t01) = 933 333
    p.renovation = true;
    let ev = g.set_plan(h, r1, CorePlan::V2(p.clone())).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(r1).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r1, false).unwrap() else { panic!() };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let r2 = g.current_round().unwrap();
    let CorePlan::V2(carried) = g.effective_plan(h, r2).unwrap() else { panic!("v2") };
    assert_eq!(carried.marketing_cents, 300_000); // preneseno iz t01 (zadani plan bi dao 0)
    assert!(!carried.renovation);
}

#[test]
fn v1_round_without_plan_still_uses_the_default_plan() {
    let (mut g, h) = common::open_june_game();
    let mut p = match g.effective_plan(h, sim_hotel::Month::Jun).unwrap() {
        CorePlan::V1(p) => p,
        CorePlan::V2(_) => panic!("v1"),
    };
    // Marketing, ne plaća: v1 zadani plan kopira `state.wage_pct` (crates/sim-hotel/src/plan.rs:75), koji obračun lipnja
    // postavi iz plana (resolve.rs:134), pa plaća ne razlikuje „zadano" od „preneseno". Zadani marketing je uvijek 0.
    p.marketing_cents = 500_000;
    let ev = g.set_plan(h, sim_hotel::Month::Jun, p).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(sim_hotel::Month::Jun, false).unwrap() else { panic!() };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let CorePlan::V1(jul) = g.effective_plan(h, sim_hotel::Month::Jul).unwrap() else { panic!() };
    assert_eq!(jul.marketing_cents, 0); // v1: zadani plan, ne prošlomjesečni (koji bi dao 500 000)
}

#[test]
fn v2_game_from_other_core_version_is_outdated() {
    let mut log = v2_events(PeriodKind::Week);
    let GameEvent::GameCreated(c) = &mut log[0].event else { panic!() };
    c.core_version = "2.0.0-staro".into();
    assert!(matches!(Game::fold(&log), Err(GameError::Outdated { .. })));
}

#[test]
fn v1_game_is_never_outdated_by_v2_version() {
    let (g, _) = common::open_june_game();
    assert_eq!(g.model(), Model::V1);
    assert_eq!(g.core_version, sim_hotel::CORE_VERSION);
}

#[test]
fn v2_situations_appear_in_the_round_containing_the_first_of_the_month() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let in_round = |code: &str| g.situations_in(RoundKey::parse(code).unwrap()).len();
    assert!(in_round("t01") > 0); // lipanj
    assert_eq!(in_round("t02"), 0);
    assert!(in_round("t05") > 0); // srpanj
}

#[test]
fn v2_weekly_situation_is_answered_in_its_round() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let code = g.situations_in(RoundKey::Week(1))[0].code.clone();
    let (round, _) = g.situation_round(&code).unwrap();
    assert_eq!(round, RoundKey::Week(1));
    let GameEvent::AnswerChosen(a) = g.choose_answer(h, &code, 0).unwrap() else { panic!("AnswerChosen") };
    assert_eq!(a.month, RoundKey::Week(1)); // JSON ime polja ostaje `month`, vrijednost "t01"
    assert!(matches!(g.situation_round("NEMA"), Err(GameError::UnknownSituation)));
}

#[test]
fn dispatch_lives_in_one_place() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap() == "model.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for needle in ["Model::V1", "Model::V2", "CoreState::V1", "CoreState::V2", "CorePlan::V1", "CorePlan::V2", "CoreRefs::"] {
            assert!(!text.contains(needle), "{} sadrži {needle} — grananje ide u model.rs", path.display());
        }
    }
}
```

`crates/game/tests/equivalence.rs`:

```rust
mod common;

use content::{Snapshot, repo_dir};
use game::*;
use sim_hotel::Month;

/// v2 u degeneriranom katalogu (mjesečno, kao v1) daje iste obračune kao v1 za istu igru (MODEL_V2 §2 t. 6).
#[test]
fn v2_monthly_degenerate_game_matches_v1_game() {
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    let v2 = v1.degenerate_v2();
    let play = |snap: &Snapshot| {
        let mut g = Game::fold(&common::new_game_events(snap)).unwrap();
        let h = common::hotel_ids()[0];
        let me = Actor::Guest(common::GUEST);
        let ev = g.choose_answer(h, "S1", 0).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        let ev = g.choose_answer(h, "S3", 1).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        for m in [Month::Jun, Month::Jul] {
            let ev = g.close_round(m).unwrap().unwrap();
            common::apply_ev(&mut g, me.clone(), ev);
            let ResolveStep::Done(evs) = g.resolve_job(m, false).unwrap() else { panic!() };
            for (a, e) in evs {
                common::apply_ev(&mut g, a, e);
            }
        }
        g
    };
    let (a, b) = (play(&v1), play(&v2));
    assert_eq!(b.model(), Model::V2);
    for m in [Month::Jun, Month::Jul] {
        let (ra, rb) = (&a.resolved[&m.into()], &b.resolved[&m.into()]);
        for (ha, hb) in ra.hotels.iter().zip(&rb.hotels) {
            assert_eq!(ha.plan, hb.plan, "{m:?}");
            assert_eq!(ha.report, hb.report, "{m:?}");
            assert_eq!(ha.indices, hb.indices, "{m:?}");
            assert_eq!(ha.progress_hundredths, hb.progress_hundredths, "{m:?}");
        }
    }
}
```

Napomena: `new_game_events` iz `common` zove `new_game` (mjesečne postavke). Za v1 snimak to je lipanj–srpanj, a za v2
snimak mjesečna v2 igra od 4 kruga (🟨 P6), pa test uspoređuje samo prva dva kruga. Referentna tablica u obje igre
pokriva sva četiri mjeseca, pa su i početni indeksi isti.

- [ ] **Step 2:** `cargo test -p game --test model_v2 --test equivalence` → FAIL.
- [ ] **Step 3: `model.rs`, `dto_v2.rs`, promjene u `game.rs`/`command.rs`/`resolve.rs`/`view.rs`/`ai_input.rs`/
  `kpi.rs`.** `HotelRecord.start_state: CoreState`, `plans: BTreeMap<RoundKey, CorePlan>`; `Game` dobiva
  `model: Model`, `rounds_info: Vec<RoundInfo>`, `refs: BTreeMap<Category, CoreRefs>`. Pogledi i mentor koriste
  `CoreState::summary()` (v1 oblik) — zato rade i za v2. `plan_limits` (v1 `PlanLimits` iz `command.rs`) za v2 dolazi
  iz `PeriodPlan::limits` kroz `Model::plan_limits`. `kpi.rs` (sloj 0): dani iz `round_info(..).period.days`.
- [ ] **Step 4: Greške.** `GameError::PlanV2(p)` mapira se u `crates/app/src/error.rs` (uz postojeći `GameError::Plan`,
  `error.rs:160-169`) varijantu po varijantu na **iste** `ErrorCode` kao v1 (`PriceNotWholeEuro`, `PriceOutOfRange`,
  `MarketingOutOfRange`, `SeasonalOutOfRange`, `SeasonalStepTooBig`, `WageOutOfRange`, `RenovationTwice`,
  `Calc → Internal`) — u ovom tasku `sim_hotel_v2::PlanError` ima točno te varijante (kopija iz Taska 2), bez `_ =>`
  (bio bi nedostižan i `clippy -D warnings` pada). API (`problem.rs`) mapira `ErrorCode`, pa za `PlanV2` ne treba ništa
  novo. Odjelne varijante dodaje Task 11, a u `department_plan_invalid` ih mapira Task 15.
  App `set_plan` (`crates/app/src/commands.rs:14-39`) više ne radi `MonthPlan::from(&plan)`, nego u `decide` zatvoru
  `g.plan_from_dto(hotel, round, &plan)?` pa `g.set_plan(hotel, round, core)` — isto ponašanje za v1.
  `app::ErrorCode::{GameOutdated, InvalidSettings, WrongModel}`; `api::ProblemCode`:
  `game_outdated` 409 „Igra je stvorena starijom verzijom igre. Započni novu igru.", `invalid_settings` 422 „Te postavke
  igre nisu dopuštene.", `model_v1` 409 „Ova igra je na starom modelu i nema ovaj prikaz.". Regeneriraj
  `corepack pnpm gen:api` (u `openapi.json` se mijenja samo popis kodova grešaka).
- [ ] **Step 5:** `cargo test --workspace; echo EXIT=$?` → zeleno, uključujući `v1_frozen` (6) i sve stare testove.
- [ ] **Step 6: Sve provjere i commit** „Sloj 1/6: jedno mjesto grananja v1/v2; v2 igra po tjednima ili mjesecima".

**Prihvaćanje:** `dispatch_lives_in_one_place` zelen; ekvivalencija igre zelena; v1 fixturei nepromijenjeni;
`git diff --stat -- crates/sim-hotel` prazno. **Tek nakon spajanja ovog taska počinje Faza B.**

---

## Faza B — nakon vrata (Taskovi 7–22)
