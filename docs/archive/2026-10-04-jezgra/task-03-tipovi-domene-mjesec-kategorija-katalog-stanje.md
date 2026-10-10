### Task 3: Tipovi domene — mjesec, kategorija, katalog, stanje hotela

**Files:**
- Create: `crates/sim-hotel/src/month.rs`, `catalog.rs`, `state.rs`
- Modify: `crates/sim-hotel/src/lib.rs` (moduli i `pub use`), `crates/sim-hotel/src/index.rs` (`IndexSet::from_named`)
- Create: `crates/sim-hotel/tests/common/mod.rs`, `crates/sim-hotel/tests/domain.rs`

**Interfaces:**
- Produces:

```rust
// month.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Month { Jun, Jul, Aug, Sep }
impl Month {
    pub const SEASON: [Month; 4] = [Month::Jun, Month::Jul, Month::Aug, Month::Sep];
    pub fn days(self) -> u8;                 // 30, 31, 31, 30
    pub fn code(self) -> &'static str;       // "jun", "jul", "aug", "sep" (stabilno: JSON, API)
    pub fn from_code(s: &str) -> Option<Month>;
    pub fn position(self) -> usize;          // 0..=3
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthTable<T>(pub [T; 4]);
impl<T: Copy> MonthTable<T> { pub fn get(&self, m: Month) -> T; }

// catalog.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category { Three, Four, Five }      // code(): "3", "4", "5"; from_code
pub struct CategorySpec {
    pub price_ref_cents: MonthTable<i64>, pub base_occupancy_bp: MonthTable<u16>, pub seasonal_plan: MonthTable<u16>,
    pub elasticity_bp: i32, pub wage_cents: i64, pub cpor_cents: i64,
}
pub struct Economy {
    pub other_revenue_bp: u16, pub hire_cents: i64, pub interest_bp: u16,
    pub marketing_half_cents: i64, pub marketing_max_bp: u16, pub goppar_floor_cents: i64,
}
pub struct LevelEffect { pub cost_cents: i64, pub equipment_pts: i32 }               // održavanje
pub struct TrainingEffect { pub cost_cents: i64, pub experience_pts: i32, pub morale_pts: i32 }
pub struct ActionSpec {
    pub maintenance_deferred: LevelEffect, pub maintenance_regular: LevelEffect, pub maintenance_intensive: LevelEffect,
    pub training_basic: TrainingEffect, pub training_full: TrainingEffect,
    pub renovation_cents: i64, pub renovation_equipment_pts: i32, pub renovation_satisfaction_pts: i32,
    pub reply_cents: i64, pub reply_rating: i32,
    pub price_min_pct: u16, pub price_max_pct: u16, pub marketing_max_cents: i64,
    pub seasonal_max: u16, pub seasonal_step: u16, pub wage_min_pct: u16, pub wage_max_pct: u16,
    pub release_morale_pts: i32,                                                       // −300
}
pub struct MechanicSpec {
    pub recovery_pts_per_q: i32, pub retention_pts_per_q: i32, pub crisis_pts_per_q: i32,   // 200, 300, 300
    pub repair_urgent_cents: i64, pub repair_temporary_cents: i64, pub repair_temporary_equipment_pts: i32,
    pub satisfaction_bound_pts: i32, pub morale_bound_pts: i32, pub rating_bound: i32,      // 800, 800, 15
    pub group_good_quality_cut_bp: u16,                                                    // 500
}
pub struct ProfileSpec {
    pub category: Category, pub rooms: u16, pub permanent: u16, pub seasonal: u16, pub wage_pct: u16,
    pub morale_pts: i32, pub experience_pts: i32, pub equipment_pts: i32, pub satisfaction_pts: i32,
    pub rating: u16, pub cash_cents: i64, pub fixed_cents: i64, pub load_hundredths: u16,
}
pub struct Catalog {
    pub categories: BTreeMap<Category, CategorySpec>, pub economy: Economy, pub actions: ActionSpec,
    pub mechanics: MechanicSpec, pub profiles: BTreeMap<Profile, ProfileSpec>,
    pub reference_profiles: BTreeMap<Category, Profile>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError { Missing(String), NotPositive(String), OutOfRange(String) }
impl Catalog {
    pub fn validate(&self) -> Result<(), CatalogError>;
    pub fn category(&self, c: Category) -> Result<&CategorySpec, CatalogError>;
    pub fn profile(&self, p: Profile) -> Result<&ProfileSpec, CatalogError>;
}

// state.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupBlock { pub month: Month, pub room_nights: u32, pub price_cents: i64 }
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Pending { pub experience_pts: i32, pub morale_pts: i32, pub renovation: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotelState {
    pub category: Category, pub rooms: u16, pub permanent: u16, pub seasonal: u16, pub wage_pct: u16,
    pub extra_wage_cents: i64, pub morale_pts: i32, pub experience_pts: i32, pub equipment_pts: i32,
    pub satisfaction_pts: i32, pub rating: u16, pub cash_cents: i64, pub fixed_cents: i64,
    pub load_hundredths: u16, pub turnover_bp: u16, pub renovated: bool,
    pub pending: Pending, pub groups: Vec<GroupBlock>,
}
impl HotelState {
    pub fn from_profile(spec: &ProfileSpec) -> engine::calc::Calc<HotelState>;  // turnover iz morala (P21)
    pub fn staff(&self) -> u16;            // permanent.saturating_add(seasonal); granice drži plan (≤ 40 sezonaca)
}
pub fn turnover_bp(morale_pts: i32) -> engine::calc::Calc<u16>;               // 300 + div_round(15·max(0, 6000−m), 100)

// index.rs
pub struct NamedIndices { pub guests: u16, pub reviews: u16, pub staff: u16, pub operations: u16, pub finance: u16 }
impl IndexSet { pub fn from_named(v: NamedIndices) -> Option<IndexSet>; }
```
Svi strukturni tipovi kataloga: `#[derive(Clone, Debug, PartialEq, Eq)]`.

- [ ] **Step 1: Testni katalog** `crates/sim-hotel/tests/common/mod.rs` — funkcije `test_catalog() -> Catalog`,
  `profile_c() -> ProfileSpec`, `profile_a()`, `profile_b()`, `state_c() -> HotelState`. Vrijednosti su TOČNO
  `TEST_CAT`, `PROFILE_A/B/C` iz `scripts/oracle-jezgra.mjs`:
  4*: cijene 13 500/16 500/17 500/13 500 centi · bazna popunjenost 7 200/7 800/8 300/7 300 bp · plan sezonaca 20/20/20/20
  · ε −8 000 · plaća 220 000 · CPOR 2 200. Economy: ostali prihod 3 000 bp · zapošljavanje 150 000 · kamata 100 bp ·
  marketing pola 1 500 000 · marketing max 1 500 bp · pod GOPPAR-a 1 000. Akcije: održavanje 300 000/0, 1 500 000/400,
  3 000 000/800 · obuka 500 000/400/200 i 1 000 000/700/200 · obnova 15 000 000, +2 500 opreme, +300 zadovoljstva ·
  odgovaranje 150 000, +5 ocjene · cijena 60–150 % · marketing ≤ 4 000 000 · sezonci ≤ 40, korak 10 · plaća 90–115 ·
  puštanje −300. Mehanike: 200/300/300 po q · popravak hitno 25 000, privremeno 8 000 i −100 opreme · granice 800/800/15
  · rez popusta 500 bp. Profil C: 4*, 120 soba, 30 + 20, plaća 100, moral 5 500, iskustvo 6 000, oprema 6 000,
  zadovoljstvo 7 000, ocjena 840, blagajna 30 000 000, fiksni 12 000 000, opterećenje 173. A = C uz ocjena 910,
  zadovoljstvo 7 800, moral 4 200, opterećenje 195, blagajna 12 000 000, fiksni 15 000 000. B = C uz ocjena 800,
  oprema 4 000, iskustvo 5 500, blagajna 60 000 000, plaća 97, moral 5 800, fiksni 9 000 000. Referentni: {4* → C}.
  Na vrhu datoteke komentar: „Mijenja se samo zajedno s `scripts/oracle-jezgra.mjs` — zlatne vrijednosti ovise o tome."

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/domain.rs`:

```rust
mod common;
use common::*;
use sim_hotel::*;

#[test]
fn month_days_and_codes_are_stable() {
    let days: Vec<u8> = Month::SEASON.iter().map(|m| m.days()).collect();
    assert_eq!(days, [30, 31, 31, 30]);
    let codes: Vec<&str> = Month::SEASON.iter().map(|m| m.code()).collect();
    assert_eq!(codes, ["jun", "jul", "aug", "sep"]);
    assert_eq!(Month::from_code("aug"), Some(Month::Aug));
    assert_eq!(Month::from_code("Aug"), None);
}

#[test]
fn month_position_matches_season_order() {
    for (i, m) in Month::SEASON.iter().enumerate() { assert_eq!(m.position(), i); }
}

#[test]
fn test_catalog_is_valid() {
    assert_eq!(test_catalog().validate(), Ok(()));
}

#[test]
fn catalog_rejects_zero_reference_price() {
    let mut cat = test_catalog();
    cat.categories.get_mut(&Category::Four).unwrap().price_ref_cents.0[0] = 0;
    assert!(matches!(cat.validate(), Err(CatalogError::NotPositive(_))));
}

#[test]
fn catalog_rejects_profile_without_category_tables() {
    let mut cat = test_catalog();
    cat.profiles.get_mut(&Profile::A).unwrap().category = Category::Five;
    assert!(matches!(cat.validate(), Err(CatalogError::Missing(_))));
}

#[test]
fn catalog_rejects_missing_reference_profile() {
    let mut cat = test_catalog();
    cat.reference_profiles.clear();
    assert!(matches!(cat.validate(), Err(CatalogError::Missing(_))));
}

#[test]
fn state_from_profile_c_derives_turnover() {
    let s = HotelState::from_profile(&profile_c()).unwrap();
    assert_eq!(s.turnover_bp, 375);         // 3 % + 0,15 % × (60 − 55) = 3,75 %
    assert_eq!(s.staff(), 50);
    assert!(s.groups.is_empty());
}

#[test]
fn turnover_rounds_half_away_from_zero() {
    assert_eq!(sim_hotel::turnover_bp(6_000), Ok(300));
    assert_eq!(sim_hotel::turnover_bp(4_200), Ok(570));
    assert_eq!(sim_hotel::turnover_bp(0), Ok(1_200));
}

#[test]
fn index_set_from_named_keeps_meaning() {
    let s = IndexSet::from_named(NamedIndices { guests: 6_000, reviews: 7_500, staff: 3_000, operations: 5_000, finance: 3_500 }).unwrap();
    assert_eq!(s.get(Index::Reviews).hundredths(), 7_500);
    assert_eq!(s.get(Index::Finance).hundredths(), 3_500);
    assert!(IndexSet::from_named(NamedIndices { guests: 10_001, reviews: 0, staff: 0, operations: 0, finance: 0 }).is_none());
}

#[test]
fn index_set_rejects_invalid_value_in_every_position() {
    for i in 0_usize..5 {
        let v: [u16; 5] = std::array::from_fn(|j| if j == i { 10_001 } else { 5_000 });
        assert!(IndexSet::from_hundredths(v).is_none(), "mjesto {i}");
    }
}

#[test]
fn profile_targets_match_demo_section_5_fully() {
    let full = |p: Profile| p.starting_indices().iter().map(|(_, v)| v.hundredths()).collect::<Vec<_>>();
    assert_eq!(full(Profile::A), [6_000, 7_500, 3_000, 5_000, 3_500]);
    assert_eq!(full(Profile::B), [4_500, 3_500, 5_500, 4_000, 7_500]);
    assert_eq!(full(Profile::C), [5_000; 5]);
}
```
(`Profile::starting_indices` ostaje: to su CILJANE vrijednosti iz DEMO §5, testno proročanstvo za §11.1 — dopiši to
u njegov doc-komentar.)

- [ ] **Step 3: Pokreni** `cargo test -p sim-hotel; echo EXIT=$?` → ne prevodi se.

- [ ] **Step 4: Implementacija.** `validate()` provjerava (P5): za svaku kategoriju sve cijene, `wage_cents`,
  `cpor_cents` > 0 i bazna popunjenost 1..=10 000; `economy.marketing_half_cents`, `goppar_floor_cents`, `hire_cents` > 0;
  `price_min_pct` < `price_max_pct`, `wage_min_pct` ≤ 100 ≤ `wage_max_pct`; svaki profil: kategorija postoji u
  `categories`, `rooms` > 0, `fixed_cents` > 0, skale 0..=10 000, ocjena 100..=1 000, plaća u granicama; svaka
  kategorija nekog profila ima referentni profil koji postoji i iste je kategorije. Poruka u `CatalogError` imenuje polje
  (npr. `"categories.4.price_ref_cents.jun"`). Sva aritmetika kroz `engine::calc` (`i128::from(..)`, `calc::narrow`).

- [ ] **Step 5: Pokreni** `cargo test -p sim-hotel; echo EXIT=$?` i clippy → 0.

- [ ] **Step 6: Commit** — „Jezgra 3/12: tipovi domene — mjesec, kategorija, katalog s provjerom, stanje hotela".

---

