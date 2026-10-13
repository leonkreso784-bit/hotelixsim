### Task 2: Paket `sim-hotel-v2` — kopija, kalendar razdoblja, tokovi kocke, brane

**Veličina:** M, 3–4 h · **Ovisi o:** sloj 0 spojen · **Paralelno s:** Task 1 · **Model:** sonnet

**Cilj:** novi jezgreni paket s istim branama kao v1, čistim kalendarom sezone (tjedan/mjesec) i tokovima kocke po
svrsi, te **mjesečnom kopijom** v1 obračuna koja već prolazi ekvivalencijski test. Task 3 tek tada uvodi razdoblje.

**Files:**
- Create: `crates/sim-hotel-v2/Cargo.toml`, `crates/sim-hotel-v2/clippy.toml` (kopija `crates/sim-hotel/clippy.toml`,
  bajt-ista), `src/{lib,calendar,streams,catalog,state,plan,market,resolve,explain,indices,robots,season}.rs`,
  `tests/common/mod.rs`, `tests/calendar.rs`, `tests/streams.rs`, `tests/equivalence.rs`
- Modify: `Cargo.toml` (član `crates/sim-hotel-v2`, `sim-hotel-v2 = { path = "crates/sim-hotel-v2" }` u
  `[workspace.dependencies]`), `Cargo.lock`, `scripts/check-core-deps.mjs`, `scripts/probe-mutacija.mjs`
- Ne dirati: `crates/sim-hotel/**`, `crates/game/**`, `crates/content/**`.

**Interfaces (Produces):**

```rust
// crates/sim-hotel-v2/src/calendar.rs
pub const SEASON_DAYS: u16 = 122;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PeriodKind { Month, Week }
impl PeriodKind {
    pub const ALL: [PeriodKind; 2];
    pub fn code(self) -> &'static str;                 // "month" | "week"
    pub fn from_code(s: &str) -> Option<PeriodKind>;
}
/// Jedan krug igre: `index` od 0, `start_day` od 0 (= 1. lipnja).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Period { pub kind: PeriodKind, pub index: u8, pub start_day: u16, pub days: u8 }
/// Dio razdoblja unutar jednog mjeseca; `from_day` od 0 unutar mjeseca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthPart { pub month: Month, pub from_day: u8, pub days: u8 }
impl Period {
    pub fn code(&self) -> String;                      // mjesec: "jun"…; tjedan: "t01"…"t17" (index + 1)
    pub fn parts(&self) -> Vec<MonthPart>;             // 1 ili 2 dijela, redom
    pub fn start_month(&self) -> Month;
    pub fn month_days_at_start(&self) -> u8;           // D₀
    pub fn ends_month(&self) -> bool;                  // zadnji dan razdoblja je zadnji dan mjeseca
    pub fn month_ending_in(&self) -> Option<Month>;    // mjesec čiji je zadnji dan u ovom razdoblju (mentor, Task 8)
    pub fn first_date(&self) -> (u8, Month);           // (dan u mjesecu od 1, mjesec)
    pub fn last_date(&self) -> (u8, Month);
}
pub fn season_periods(kind: PeriodKind) -> Vec<Period>;
pub fn month_period(m: Month) -> Period;
pub fn period_by_code(kind: PeriodKind, code: &str) -> Option<Period>;
pub fn period_containing(kind: PeriodKind, day: u16) -> Option<Period>;
pub fn first_period_of_month(kind: PeriodKind, m: Month) -> Period; // razdoblje koje sadrži 1. dan mjeseca
pub fn prorate_part(monthly: i64, part: &MonthPart) -> Calc<i64>;
pub fn prorate(monthly: i64, p: &Period) -> Calc<i64>;
pub fn avg_i64(t: &MonthTable<i64>, p: &Period) -> Calc<i64>;
pub fn avg_u16(t: &MonthTable<u16>, p: &Period) -> Calc<u16>;
pub fn per_month_equiv(x: i64, p: &Period) -> Calc<i64>;   // div_round(x·D₀, p.days)

// crates/sim-hotel-v2/src/streams.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Purpose { Departures, Breakdowns, Reviews, LabourMarket }
impl Purpose { pub const ALL: [Purpose; 4]; pub fn label(self) -> &'static str; }
pub fn stream(seed: u64, purpose: Purpose, period_index: u8, slot: u8, key: u64) -> engine::Rng;

// crates/sim-hotel-v2/src/catalog.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodCoefficients { pub morale_step_bp: u16, pub satisfaction_step_bp: u16, pub rating_step_bp: u16, pub seasonal_step: u16 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog { pub base: sim_hotel::Catalog, pub periods: BTreeMap<PeriodKind, PeriodCoefficients> }
impl Catalog {
    pub fn coeffs(&self, k: PeriodKind) -> Result<&PeriodCoefficients, CatalogError>;
    pub fn validate(&self) -> Result<(), CatalogError>;
}
pub fn degenerate(base: &sim_hotel::Catalog) -> Catalog;

// mjesečna kopija (Task 3 je pretvara u razdoblja)
pub struct PeriodPlan { /* polja kao sim_hotel::MonthPlan */ }
pub struct HotelState { /* polja kao sim_hotel::HotelState */ }
impl HotelState { pub fn from_profile(spec: &ProfileSpec) -> Calc<HotelState>; pub fn summary(&self) -> sim_hotel::HotelState; }
pub struct PeriodReport { pub period: Period, pub before: HotelState, pub after: HotelState, pub summary: sim_hotel::MonthReport }
pub enum ResolveError { Calc(CalcError), Plan(PlanError), Catalog(CatalogError), Mechanic(MechanicError), BadChoice }
pub fn resolve_month(state: &HotelState, month: Month, plan: &PeriodPlan, situations: &[SituationOutcome], cat: &Catalog) -> Result<PeriodReport, ResolveError>;
pub struct RefTable { pub start: sim_hotel::RefPoint, pub points: Vec<sim_hotel::RefPoint> }
pub struct SeasonResult { pub start: IndexSet, pub periods: Vec<(PeriodReport, IndexSet)>, pub progress_pts: i32 }
pub fn run_season(profile: Profile, strategy: Strategy, seed: u64, cat: &Catalog, situations: &[SituationTemplate], refs: &RefTable) -> Result<SeasonResult, ResolveError>;
pub const CORE_VERSION: &str; // "2.0.0"
```

- [ ] **Step 1: Kostur paketa.** `crates/sim-hotel-v2/Cargo.toml`:

```toml
[package]
name = "sim-hotel-v2"
version = "2.0.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
engine.workspace = true
sim-hotel.workspace = true

[lints]
workspace = true
```

`src/lib.rs`:

```rust
//! Model v2 (MODEL_V2.md): hotel od stvari, razdoblje od N dana (tjedan ili mjesec). v1 (`sim_hotel`) je zamrznut;
//! odande se ponovno koriste katalog (`Catalog::base`), mehanike situacija, indeksi, ocjena napretka i izlazne
//! strukture izvješća, a svako razdoblje daje i v1 sažetak (`PeriodReport::summary`).
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod calendar;
pub mod catalog;
pub mod explain;
pub mod indices;
pub mod market;
pub mod plan;
pub mod resolve;
pub mod robots;
pub mod season;
pub mod state;
pub mod streams;

pub use calendar::{MonthPart, Period, PeriodKind, season_periods};
pub use catalog::{Catalog, PeriodCoefficients, degenerate};
pub use indices::{RefTable, period_indices, start_indices};
pub use plan::{PeriodPlan, PlanError};
pub use resolve::{PeriodReport, ResolveError};
pub use season::{SeasonResult, run_season, run_season_with};
pub use sim_hotel::{
    CatalogError, Category, Cause, Effects, GroupBlock, GuestOutcome, Index, IndexSet, Line, Maintenance,
    MarketOutcome, Mechanic, MechanicError, MoneyOutcome, Month, MonthReport, MonthTable, OfferedAnswer, Pending,
    PeopleOutcome, Profile, ProfileSpec, SituationOutcome, SituationTemplate, Strategy, Training, Unit, Var,
    gain_bp, progress_pts,
};
pub use state::HotelState;

/// Verzija jezgre v2 u `GameCreated` (MODEL_V2 §6.2 t. 3); raste po sloju, ne unutar sloja 1.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
```

- [ ] **Step 2: Testovi kalendara (padaju)** — `tests/calendar.rs`:

```rust
use sim_hotel_v2::calendar::*;
use sim_hotel_v2::{Month, MonthTable};

#[test]
fn weekly_season_is_sixteen_weeks_and_ten_days() {
    let w = season_periods(PeriodKind::Week);
    assert_eq!(w.len(), 17);
    assert!(w[..16].iter().all(|p| p.days == 7));
    assert_eq!(w[16].days, 10);
    assert_eq!(w[16].start_day, 112);
    assert_eq!(w[16].first_date(), (21, Month::Sep));
    assert_eq!(w[16].last_date(), (30, Month::Sep));
    assert_eq!(w.iter().map(|p| u16::from(p.days)).sum::<u16>(), SEASON_DAYS);
    let codes: Vec<String> = w.iter().map(Period::code).collect();
    assert_eq!(codes.first().map(String::as_str), Some("t01"));
    assert_eq!(codes.last().map(String::as_str), Some("t17"));
}

#[test]
fn monthly_periods_match_month_days() {
    let m = season_periods(PeriodKind::Month);
    let days: Vec<u8> = m.iter().map(|p| p.days).collect();
    assert_eq!(days, [30, 31, 31, 30]);
    let codes: Vec<String> = m.iter().map(Period::code).collect();
    assert_eq!(codes, ["jun", "jul", "aug", "sep"]);
    assert!(m.iter().all(Period::ends_month));
    assert_eq!(month_period(Month::Aug), m[2]);
}

#[test]
fn week_five_spans_june_and_july() {
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    assert_eq!(t05.first_date(), (29, Month::Jun));
    assert_eq!(t05.last_date(), (5, Month::Jul));
    assert_eq!(
        t05.parts(),
        vec![
            MonthPart { month: Month::Jun, from_day: 28, days: 2 },
            MonthPart { month: Month::Jul, from_day: 0, days: 5 },
        ]
    );
    assert_eq!(t05.month_days_at_start(), 30);
    assert!(!t05.ends_month());
}

#[test]
fn prorate_is_exact_within_each_month() {
    for kind in PeriodKind::ALL {
        for monthly in [15_000_000_i64, 220_000, 300, -400, -100, 7, 1, 0] {
            for m in Month::SEASON {
                let total: i64 = season_periods(kind)
                    .iter()
                    .flat_map(Period::parts)
                    .filter(|part| part.month == m)
                    .map(|part| prorate_part(monthly, &part).unwrap())
                    .sum();
                assert_eq!(total, monthly, "{kind:?} {m:?} {monthly}");
            }
        }
    }
}

#[test]
fn prorate_of_full_month_is_the_monthly_amount() {
    for m in Month::SEASON {
        assert_eq!(prorate(1_234_567, &month_period(m)).unwrap(), 1_234_567);
    }
}

#[test]
fn average_is_day_weighted_and_exact_for_full_month() {
    let t = MonthTable([13_500_i64, 16_500, 17_500, 13_500]);
    assert_eq!(avg_i64(&t, &month_period(Month::Jul)).unwrap(), 16_500);
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    // (2·13 500 + 5·16 500) / 7 = 15 642,86 → 15 643
    assert_eq!(avg_i64(&t, &t05).unwrap(), 15_643);
    let bp = MonthTable([7_200_u16, 7_800, 10_600, 7_300]);
    // (2·7 200 + 5·7 800) / 7 = 7 628,57 → 7 629
    assert_eq!(avg_u16(&bp, &t05).unwrap(), 7_629);
}

#[test]
fn first_period_of_each_month_contains_its_first_day() {
    let codes: Vec<String> = Month::SEASON
        .iter()
        .map(|&m| first_period_of_month(PeriodKind::Week, m).code())
        .collect();
    assert_eq!(codes, ["t01", "t05", "t09", "t14"]);
    assert_eq!(first_period_of_month(PeriodKind::Month, Month::Aug).code(), "aug");
}

#[test]
fn month_ends_fall_in_t05_t09_t14_and_t17() {
    let ends: Vec<(String, Month)> = season_periods(PeriodKind::Week)
        .iter()
        .filter_map(|p| p.month_ending_in().map(|m| (p.code(), m)))
        .collect();
    assert_eq!(
        ends,
        [("t05".into(), Month::Jun), ("t09".into(), Month::Jul), ("t14".into(), Month::Aug), ("t17".into(), Month::Sep)]
    );
    assert!(season_periods(PeriodKind::Month).iter().all(|p| p.month_ending_in().is_some()));
}

#[test]
fn codes_parse_back_and_unknown_codes_do_not() {
    for kind in PeriodKind::ALL {
        for p in season_periods(kind) {
            assert_eq!(period_by_code(kind, &p.code()), Some(p));
        }
    }
    assert_eq!(period_by_code(PeriodKind::Week, "t18"), None);
    assert_eq!(period_by_code(PeriodKind::Week, "jun"), None);
    assert_eq!(period_by_code(PeriodKind::Month, "t01"), None);
}

#[test]
fn per_month_equivalent_scales_by_start_month_days() {
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    assert_eq!(per_month_equiv(700, &t02).unwrap(), 3_000); // 700 · 30 / 7
    assert_eq!(per_month_equiv(700, &month_period(Month::Jul)).unwrap(), 700);
}
```

Run: `cd /c/Users/leonk/Documents/hotelixsim && cargo test -p sim-hotel-v2 --test calendar; echo EXIT=$?` → FAIL (modul ne postoji).

- [ ] **Step 3: Napiši `src/calendar.rs`.** Bez `+ - * / %` i `as` (lintovi): `checked_*`/`saturating_*` za `u8/u16`,
`engine::calc` za novac. Jezgra:

```rust
//! Kalendar sezone (MODEL_V2 §4): krug je tjedan ili mjesec; dan 0 = 1. lipnja, sezona ima 122 dana.
//! Tjedni: 16 × 7 dana + zadnje razdoblje od 10 dana (21.–30. 9.) — pretpostavka 🟨 P1 (MODEL_V2 §9.2 N3).

use engine::calc::{self as c, Calc};
use sim_hotel::{Month, MonthTable};

pub const SEASON_DAYS: u16 = 122;
const FULL_WEEKS: u8 = 16;
const LAST_DAYS: u8 = 10;

fn month_start(m: Month) -> u16 {
    match m {
        Month::Jun => 0,
        Month::Jul => 30,
        Month::Aug => 61,
        Month::Sep => 92,
    }
}

fn month_index(m: Month) -> u8 {
    match m {
        Month::Jun => 0,
        Month::Jul => 1,
        Month::Aug => 2,
        Month::Sep => 3,
    }
}

pub fn month_period(m: Month) -> Period {
    Period { kind: PeriodKind::Month, index: month_index(m), start_day: month_start(m), days: m.days() }
}

pub fn season_periods(kind: PeriodKind) -> Vec<Period> {
    match kind {
        PeriodKind::Month => Month::SEASON.iter().map(|&m| month_period(m)).collect(),
        PeriodKind::Week => {
            let mut out = Vec::with_capacity(17);
            let mut start: u16 = 0;
            for index in 0..FULL_WEEKS {
                out.push(Period { kind, index, start_day: start, days: 7 });
                start = start.saturating_add(7);
            }
            out.push(Period { kind, index: FULL_WEEKS, start_day: start, days: LAST_DAYS });
            out
        }
    }
}

impl Period {
    fn end_day(&self) -> u16 {
        self.start_day.saturating_add(u16::from(self.days))
    }

    pub fn parts(&self) -> Vec<MonthPart> {
        let (start, end) = (self.start_day, self.end_day());
        Month::SEASON
            .iter()
            .filter_map(|&m| {
                let ms = month_start(m);
                let me = ms.saturating_add(u16::from(m.days()));
                let (lo, hi) = (start.max(ms), end.min(me));
                if lo >= hi {
                    return None;
                }
                Some(MonthPart {
                    month: m,
                    from_day: u8::try_from(lo.saturating_sub(ms)).ok()?,
                    days: u8::try_from(hi.saturating_sub(lo)).ok()?,
                })
            })
            .collect()
    }
    // code, start_month, month_days_at_start, ends_month, first_date, last_date: iz `parts()`
    // (prvi dio → početak, zadnji dio → kraj; `from_day + 1` je dan u mjesecu).
}

pub fn prorate_part(monthly: i64, part: &MonthPart) -> Calc<i64> {
    let d = i128::from(part.month.days());
    let from = i128::from(part.from_day);
    let to = c::add(from, i128::from(part.days))?;
    let m = i128::from(monthly);
    c::narrow(c::sub(c::div_floor(c::mul(m, to)?, d)?, c::div_floor(c::mul(m, from)?, d)?)?)
}

pub fn prorate(monthly: i64, p: &Period) -> Calc<i64> {
    let mut total: i128 = 0;
    for part in p.parts() {
        total = c::add(total, i128::from(prorate_part(monthly, &part)?))?;
    }
    c::narrow(total)
}
```

`avg_i64`: `div_round(Σ t.get(part.month)·part.days, p.days)`; `avg_u16` isto pa `narrow`; `per_month_equiv`:
`div_round(x·D₀, p.days)`; `period_by_code`/`period_containing`/`first_period_of_month` traže u `season_periods(kind)`.
`div_floor` je euklidski, pa se razlike i za negativne iznose zbrajaju točno na mjesečni iznos.

- [ ] **Step 4: `src/streams.rs` i test** — `tests/streams.rs`:

```rust
use engine::{Rng, label};
use sim_hotel_v2::streams::{Purpose, stream};

#[test]
fn every_purpose_has_its_own_stable_v2_label() {
    let labels: Vec<&str> = Purpose::ALL.iter().map(|p| p.label()).collect();
    assert_eq!(labels, ["v2:odlasci", "v2:kvarovi", "v2:recenzije", "v2:burza-rada"]);
}

#[test]
fn stream_is_derive_over_purpose_period_slot_and_key() {
    let mut a = stream(7, Purpose::Departures, 3, 2, 99);
    let mut b = Rng::derive(7, &[label("v2:odlasci"), 3, 2, 99]);
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn streams_differ_when_any_input_differs() {
    let first = |r: &mut Rng| r.next_u64();
    let base = first(&mut stream(7, Purpose::Departures, 3, 2, 99));
    for other in [
        stream(8, Purpose::Departures, 3, 2, 99),
        stream(7, Purpose::Reviews, 3, 2, 99),
        stream(7, Purpose::Departures, 4, 2, 99),
        stream(7, Purpose::Departures, 3, 1, 99),
        stream(7, Purpose::Departures, 3, 2, 98),
    ] {
        let mut o = other;
        assert_ne!(first(&mut o), base);
    }
}
```

`stream` = `Rng::derive(seed, &[label(purpose.label()), u64::from(period_index), u64::from(slot), key])`. Oznake se
nikad ne mijenjaju (mijenjale bi niz brojeva budućih igara); u sloju 1 nijedna formula ne zove `stream`.

- [ ] **Step 5: `src/catalog.rs`** (tipovi gore). `validate`: `base.validate()?`; za obje vrste razdoblja koeficijenti
pomaka u `1..=10_000`, `seasonal_step ≥ 1`, inače `CatalogError::OutOfRange("v2.periods.<vrsta>.<polje>")`.
`degenerate(base)`: `month` = `{3_000, 5_000, 2_500, base.actions.seasonal_step}` (v1 konstante iz
`crates/sim-hotel/src/resolve.rs`), `week` = `{800, 1_500, 700, 5}` [PROC].

- [ ] **Step 6: Mjesečna kopija v1 obračuna.** `cp crates/sim-hotel/src/{state,plan,market,resolve,explain,indices,robots,season}.rs crates/sim-hotel-v2/src/`
pa uredi:
  - `state.rs`: zadrži `HotelState` (ista polja) i `from_profile`, `staff()`; **obriši** `GroupBlock`, `Pending`,
    `turnover_bp` (uvozi iz `sim_hotel`); dodaj `summary()` (polje po polje u `sim_hotel::HotelState`).
  - `plan.rs`: `MonthPlan` → `PeriodPlan`; `Maintenance`/`Training` iz `sim_hotel`; `PlanError` vlastita kopija.
  - `market.rs`: vraća `sim_hotel::MarketOutcome` (obriši vlastitu strukturu); prima `&sim_hotel::Catalog`.
  - `resolve.rs`: obriši strukture `PeopleOutcome/MoneyOutcome/GuestOutcome/MonthReport` (iz `sim_hotel`), zadrži
    privatne korake i `pub(crate)` strukture koraka; `MonthReport` → `PeriodReport { period: month_period(month),
    before, after, summary: sim_hotel::MonthReport { month, before: before.summary(), after: after.summary(), market,
    people, money, guests, lines } }`; `ResolveError` kopija + `From<sim_hotel::ResolveError>` (za `Strategy::choose`).
  - `explain.rs`: zadrži `Lines`, `Explain`, `explain`; `Line/Var/Cause/Unit` iz `sim_hotel`.
  - `indices.rs`: zadrži samo `RefTable { start, points: Vec<RefPoint> }`, `RefTable::compute`, `start_indices`,
    `month_indices` → `period_indices(&PeriodReport, …)`; `raw_indices`, `centred`, `base_labor_cents`, `RefPoint`
    iz `sim_hotel` nad `summary()`.
  - `robots.rs`: `pub fn robot_plan(strategy: Strategy, s: &HotelState, m: Month, cat: &Catalog, rng: &mut Rng)
    -> Result<PeriodPlan, ResolveError>` (tijelo iz `Strategy::plan`); izbor odgovora ostaje `Strategy::choose` iz v1.
  - `season.rs`: `SeasonResult.periods`.
  - Unutarnje funkcije i dalje primaju `&sim_hotel::Catalog`; javni ulazi primaju `&Catalog` (v2) i predaju `&cat.base`.

- [ ] **Step 7: Ekvivalencija (mora proći odmah)** — `tests/common/mod.rs` je kopija
`crates/sim-hotel/tests/common/mod.rs` (testni katalog) plus:

```rust
/// Dvije situacije za ekvivalenciju: oporavak gosta u lipnju i grupa za kolovoz prihvaćena u lipnju.
pub fn test_situations() -> Vec<SituationTemplate> {
    vec![
        SituationTemplate {
            code: "T1".into(),
            month: Month::Jun,
            title: "Pritužba".into(),
            answers: vec![
                OfferedAnswer { label: "Isprika".into(), mechanics: vec![Mechanic::ServiceRecovery { compensation_cents: 5_000 }], quality: 4 },
                OfferedAnswer { label: "Ništa".into(), mechanics: vec![Mechanic::ServiceRecovery { compensation_cents: 0 }], quality: 1 },
            ],
            unanswered: vec![Mechanic::ServiceRecovery { compensation_cents: 0 }],
        },
        SituationTemplate {
            code: "T2".into(),
            month: Month::Jun,
            title: "Grupa".into(),
            answers: vec![
                OfferedAnswer { label: "Da".into(), mechanics: vec![Mechanic::GroupBooking { accept: true, rooms: 20, nights: 5, discount_bp: 1_500, month: Month::Aug }], quality: 3 },
                OfferedAnswer { label: "Ne".into(), mechanics: vec![Mechanic::GroupBooking { accept: false, rooms: 20, nights: 5, discount_bp: 1_500, month: Month::Aug }], quality: 3 },
            ],
            unanswered: vec![Mechanic::GroupBooking { accept: false, rooms: 20, nights: 5, discount_bp: 1_500, month: Month::Aug }],
        },
    ]
}
```

`tests/equivalence.rs`:

```rust
mod common;

use sim_hotel_v2::{Profile, Strategy};

#[test]
fn v2_copy_equals_v1_for_every_profile_strategy_and_seed() {
    let base = common::test_catalog();
    let v2 = sim_hotel_v2::degenerate(&base);
    let situations = common::test_situations();
    for p in Profile::ALL {
        let category = base.profile(p).unwrap().category;
        let r1 = sim_hotel::RefTable::compute(&base, category).unwrap();
        let r2 = sim_hotel_v2::RefTable::compute(&v2, category).unwrap();
        for s in Strategy::ALL {
            for seed in [7_u64, 42] {
                for sits in [&[][..], &situations[..]] {
                    let a = sim_hotel::run_season(p, s, seed, &base, sits, &r1).unwrap();
                    let b = sim_hotel_v2::run_season(p, s, seed, &v2, sits, &r2).unwrap();
                    assert_eq!(a.start, b.start, "{p:?} {s:?} {seed}");
                    assert_eq!(a.progress_pts, b.progress_pts, "{p:?} {s:?} {seed}");
                    assert_eq!(a.months.len(), b.periods.len());
                    for ((ra, ia), (rb, ib)) in a.months.iter().zip(&b.periods) {
                        assert_eq!(ra, &rb.summary, "{p:?} {s:?} {seed} {:?}", ra.month);
                        assert_eq!(ia, ib);
                    }
                }
            }
        }
    }
}
```

(Ako `test_catalog()` nema profile A i B, test ide samo po profilima koje ima — provjeri u kopiji `common`.)

Run: `cd /c/Users/leonk/Documents/hotelixsim && cargo test -p sim-hotel-v2; echo EXIT=$?` → svi prolaze.

- [ ] **Step 8: Brane.** `scripts/check-core-deps.mjs`: `CORE = ["engine", "sim-hotel", "sim-hotel-v2"]`,
`ALLOWED = new Set(["engine", "sim-hotel", "sim-hotel-v2"])`, provjera float literala i za `crates/sim-hotel-v2`.
`scripts/probe-mutacija.mjs`: nova mutacija „HashSet u sim-hotel-v2" (isti obrazac kao za `sim-hotel`, datoteka
`crates/sim-hotel-v2/src/lib.rs`, `nadi: "pub mod calendar;"`), `CLIPPY` dobiva `-p sim-hotel-v2`, provjera
identičnosti `clippy.toml` za sva tri jezgrena paketa.

Run: `cd /c/Users/leonk/Documents/hotelixsim && node scripts/check-core-deps.mjs; echo EXIT=$?` → `prekršaja: 0`;
`node scripts/probe-mutacija.mjs; echo EXIT=$?` → svaka proba `OK`, EXIT=0.

- [ ] **Step 9: Sve provjere i commit** „Sloj 1/2: paket sim-hotel-v2 — kalendar razdoblja, tokovi kocke, mjesečna kopija v1".

**Prihvaćanje:** `cargo test -p sim-hotel-v2` zeleno (kalendar 10, tokovi 3, ekvivalencija 1); brane EXIT=0;
`git diff --stat -- crates/sim-hotel` prazno.

---

