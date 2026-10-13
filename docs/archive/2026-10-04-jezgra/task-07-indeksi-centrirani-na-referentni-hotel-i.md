### Task 7: Indeksi centrirani na referentni hotel i relativna ocjena (§4, §8, ADR-013 t. 1–2)

**Files:**
- Create: `crates/sim-hotel/src/indices.rs`, `crates/sim-hotel/src/score.rs`, `crates/sim-hotel/tests/indices.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawIndices { pub g: i32, pub r: i32, pub z: i32, pub o: i32, pub f: i32 } // stotinke, R smije biti < 0
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefPoint { pub raw: RawIndices, pub goppar_cents: i64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefTable { pub start: RefPoint, pub months: MonthTable<RefPoint> }
impl RefTable { pub fn compute(cat: &Catalog, category: Category) -> Result<RefTable, ResolveError>; }
pub fn raw_indices(s: &HotelState, goppar_cents: i64, labor_cents: i64, ref_goppar_cents: i64, cat: &Catalog) -> Calc<RawIndices>;
pub fn centred(raw: &RawIndices, reference: &RawIndices) -> Calc<IndexSet>;
pub fn base_labor_cents(s: &HotelState, cat: &Catalog) -> Result<i64, ResolveError>;   // staff·(wage/100)·wage_pct + extra
pub fn start_indices(s: &HotelState, cat: &Catalog, refs: &RefTable) -> Result<IndexSet, ResolveError>;
pub fn month_indices(r: &MonthReport, cat: &Catalog, refs: &RefTable) -> Result<IndexSet, ResolveError>;
// score.rs
pub fn gain_bp(start: u16, end: u16) -> Calc<i32>;
pub fn progress_pts(start: &IndexSet, end: &IndexSet) -> Calc<i32>;
```

- [ ] **Step 1: Formule** (P30 + centriranje): `retention = clamp(10000 − div_round(25·turnover, 2), 0, 10000)` ·
  `g = satisfaction` · `r = div_round((rating − 700)·10000, 280)` · `z = div_round(6000·morale + 4000·retention, 10000)` ·
  `o = div_round(4500·equipment + 3000·experience + 2500·(10000 − penalty(load)), 10000)` ·
  `gi = clamp(5000 + div_round(5000·(goppar − ref_goppar), max(ref_goppar, goppar_floor)), 0, 10000)` ·
  `liq = cash ≤ 0 ? 0 : clamp(div_round(5000·cash, fixed + labor), 0, 10000)` · `f = div_round(7000·gi + 3000·liq, 10000)`.
  `centred`: svaki indeks `clamp(5000 + raw − ref, 0, 10000)`.
  `RefTable::compute`: referentni profil kategorije → `HotelState::from_profile`; `start` = sirovo stanje uz GOPPAR
  prognoze lipnja (zadani plan, bez situacija) i `base_labor`, s `ref_goppar` = isti GOPPAR; zatim mjeseci redom sa
  zadanim planom, `months[m]` = sirovo nakon mjeseca uz vlastiti GOPPAR kao referencu i `labor` mjeseca.
  `start_indices`: GOPPAR prognoze lipnja ovog hotela, `base_labor`, `refs.start`. `month_indices`: `r.after`,
  `r.money.goppar_cents`, `r.money.labor_cents`, `refs.months[r.month]`.
  `gain_bp` i `progress_pts` po P31 (`start = 10000 = end` → 0; nazivnik druge grane `start ≥ 1`).

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/indices.rs`:

```rust
mod common;
use common::*;
use sim_hotel::*;

fn raw(g: i32, r: i32, z: i32, o: i32, f: i32) -> RawIndices { RawIndices { g, r, z, o, f } }
fn five(v: [u16; 5]) -> IndexSet { IndexSet::from_hundredths(v).unwrap() }

#[test]
fn reference_table_for_test_catalog() {
    let t = RefTable::compute(&test_catalog(), Category::Four).unwrap();
    assert_eq!(t.start, RefPoint { raw: raw(7_000, 5_000, 5_425, 7_000, 5_457), goppar_cents: 4_163 });
    assert_eq!(t.months.get(Month::Jun), RefPoint { raw: raw(6_874, 4_964, 5_425, 7_042, 6_396), goppar_cents: 4_163 });
    assert_eq!(t.months.get(Month::Jul), RefPoint { raw: raw(6_753, 4_893, 5_282, 6_862, 6_500), goppar_cents: 8_340 });
    assert_eq!(t.months.get(Month::Aug), RefPoint { raw: raw(6_591, 4_786, 4_939, 6_556, 6_500), goppar_cents: 10_364 });
    assert_eq!(t.months.get(Month::Sep), RefPoint { raw: raw(6_658, 4_714, 5_084, 7_068, 6_500), goppar_cents: 4_283 });
}

#[test]
fn starting_indices_of_profiles() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let start = |p: ProfileSpec| start_indices(&HotelState::from_profile(&p).unwrap(), &cat, &refs).unwrap();
    assert_eq!(start(profile_c()), five([5_000; 5]));
    assert_eq!(start(profile_a()), five([5_800, 7_500, 3_245, 4_625, 3_351]));
    assert_eq!(start(profile_b()), five([5_000, 3_571, 5_405, 3_950, 6_620]));
}

#[test]
fn passive_reference_hotel_stays_at_fifty_all_season() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let mut s = state_c();
    for m in Month::SEASON {
        let r = resolve_month(&s, m, &MonthPlan::default_for(&s, m, &cat).unwrap(), &[], &cat).unwrap();
        assert_eq!(month_indices(&r, &cat, &refs).unwrap(), five([5_000; 5]), "{m:?}");
        s = r.after;
    }
    assert_eq!(s.cash_cents, 129_984_850);
}

#[test]
fn june_variant_indices_and_progress() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let s = state_c();
    let plan = MonthPlan { price_cents: 15_000, marketing_cents: 1_500_000, seasonal_target: 30, wage_pct: 105,
        training: Training::Basic, maintenance: Maintenance::Intensive, renovation: false, reply_reviews: true };
    let r = resolve_month(&s, Month::Jun, &plan, &[], &cat).unwrap();
    let idx = month_indices(&r, &cat, &refs).unwrap();
    assert_eq!(idx, five([4_883, 5_107, 5_405, 4_988, 3_390]));
    assert_eq!(progress_pts(&five([5_000; 5]), &idx), Ok(-491));
}

#[test]
fn gain_follows_room_for_growth() {
    assert_eq!(gain_bp(5_000, 6_000), Ok(2_000));
    assert_eq!(gain_bp(5_000, 4_000), Ok(-2_000));
    assert_eq!(gain_bp(10_000, 10_000), Ok(0));
    assert_eq!(gain_bp(0, 100), Ok(100));
    assert_eq!(gain_bp(7_500, 8_000), Ok(2_000));
    assert_eq!(gain_bp(3_500, 4_000), Ok(769));
    assert_eq!(gain_bp(0, 0), Ok(0));
    assert_eq!(progress_pts(&five([5_000; 5]), &five([6_000, 4_000, 5_000, 5_500, 5_100])), Ok(240));
}

#[test]
fn liquidity_is_zero_when_cash_negative() {
    let cat = test_catalog();
    let mut s = state_c();
    s.cash_cents = -1;
    let raw = raw_indices(&s, 4_163, 11_000_000, 4_163, &cat).unwrap();
    assert_eq!(raw.f, 3_500); // 0,7 × 50 + 0,3 × 0
}
```

- [ ] **Step 3–5:** pokreni (pada) → implementiraj → pokreni `cargo test -p sim-hotel; echo EXIT=$?` i clippy → 0.
- [ ] **Step 6: Commit** — „Jezgra 7/12: indeksi centrirani na referentni hotel i relativna ocjena".

---

