### Task 3: Obračun razdoblja (tjedan ili mjesec) i ekvivalencija s v1

**Veličina:** L, 5–6 h · **Ovisi o:** Task 2 · **Paralelno s:** Task 4, Task 5 · **Model:** sonnet (opus ako se
ekvivalencija ne poklapa nakon prvog pokušaja)

**Cilj:** `resolve_period` računa bilo koje razdoblje po **Pravilima razdoblja** (gore); mjesečna razdoblja daju
**točno** v1 brojke, tjedna sezona od 17 krugova radi bez greške i ponovljivo.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{plan,market,resolve,explain,indices,robots,season}.rs`
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
  `Rng::derive(seed, &[pozicija mjeseca situacije, label(&t.code)])` za izbor — u mjesečnoj igri isto kao v1.
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

