# 2026-10-10-v2-sloj1-kartice — dio 4

Arhiva (zamrznuto). Indeks: [2026-10-10-v2-sloj1-kartice.md](../2026-10-10-v2-sloj1-kartice.md).

### Task 10: Sadržaj v2 — odjeli, profili s odjelima, raspad fiksnih troškova

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 9 · **Paralelno s:** Task 8, Task 13, Task 18 (val 4) · **Model:** sonnet

**Cilj:** stvarni katalog odjela za profile A/B/C (MODEL_V2 §3.2, brojke [PROC] — kalibrira ih Task 17) i
graditelj u `crates/content`.

**Files:**
- Create: `content/hotel/v2/departments.json`, `content/hotel/v2/profiles.json`, `content/hotel/v2/economy.json`,
  `crates/content/tests/v2_departments.rs`
- Modify: `crates/content/src/v2.rs` (DTO-i i graditelj), `crates/content/src/lib.rs` (`read_v2` čita još tri datoteke;
  `degenerate_v2` puni nove dijelove iz `sim_hotel_v2::degenerate`)
- Ne dirati: v1 sadržaj, `crates/sim-hotel-v2/**` (osim ako validacija iz Taska 9 ima grešku — tada stani i javi).

**Interfaces:**
- Consumes: Task 9 tipovi (`DeptSpec`, `RevenueModel`, `UsaliClass`, `DeptStart`, `ProfileV2`, `EconomyV2`,
  `FixedChargeSpec`, `Catalog::validate`).
- Produces: JSON oblik (kodovi odjela i naknada su stabilni ID-jevi):

`content/hotel/v2/economy.json`:

```json
{
  "guests_per_room_hundredths": 180,
  "price_level_min_pct": 90,
  "price_level_max_pct": 120,
  "fixed_charges": {
    "insurance": { "name": "Osiguranje" },
    "lease": { "name": "Najam zgrade i opreme" },
    "property_tax": { "name": "Porez na imovinu" }
  }
}
```

`content/hotel/v2/departments.json` (prihod: `{"type": "rooms" | "per_guest" | "fixed" | "share_of_rooms" | "none", …}`;
održavanje `[odgođeno, redovno, pojačano]` kao `{ "deferred": {…}, "regular": {…}, "intensive": {…} }`):

```json
{
  "rooms": {
    "name": "Smještaj", "class": "rooms", "revenue": { "type": "rooms" },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 2200, "wage_cents": 200000, "capacity_tenths_per_staff_day": 55,
    "wear_pts_per_month": 400,
    "maintenance": { "deferred": { "cost_cents": 300000, "equipment_pts": 0 }, "regular": { "cost_cents": 1500000, "equipment_pts": 400 }, "intensive": { "cost_cents": 3000000, "equipment_pts": 800 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 30,
    "seasonal_plan": { "jun": 14, "jul": 17, "aug": 22, "sep": 17 }, "can_close": false
  },
  "restaurant": {
    "name": "Restoran", "class": "food_beverage",
    "revenue": { "type": "per_guest", "usage_bp": { "jun": 4500, "jul": 4800, "aug": 5000, "sep": 4500 }, "check_cents": 3200, "price_elasticity_bp": -15000, "satisfaction_elasticity_bp": 5000 },
    "cost_of_sales_bp": 3200, "direct_per_use_cents": 0, "wage_cents": 210000, "capacity_tenths_per_staff_day": 50,
    "wear_pts_per_month": 300,
    "maintenance": { "deferred": { "cost_cents": 50000, "equipment_pts": 0 }, "regular": { "cost_cents": 400000, "equipment_pts": 300 }, "intensive": { "cost_cents": 800000, "equipment_pts": 600 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 20,
    "seasonal_plan": { "jun": 10, "jul": 12, "aug": 15, "sep": 12 }, "can_close": false
  },
  "lobby_bar": {
    "name": "Lobby bar", "class": "food_beverage",
    "revenue": { "type": "per_guest", "usage_bp": { "jun": 4000, "jul": 4200, "aug": 4500, "sep": 4000 }, "check_cents": 1200, "price_elasticity_bp": -12000, "satisfaction_elasticity_bp": 4000 },
    "cost_of_sales_bp": 2500, "direct_per_use_cents": 0, "wage_cents": 190000, "capacity_tenths_per_staff_day": 120,
    "wear_pts_per_month": 250,
    "maintenance": { "deferred": { "cost_cents": 20000, "equipment_pts": 0 }, "regular": { "cost_cents": 150000, "equipment_pts": 250 }, "intensive": { "cost_cents": 300000, "equipment_pts": 500 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 8,
    "seasonal_plan": { "jun": 3, "jul": 4, "aug": 5, "sep": 4 }, "can_close": true
  },
  "rooftop_bar": {
    "name": "Rooftop bar", "class": "food_beverage",
    "revenue": { "type": "per_guest", "usage_bp": { "jun": 1500, "jul": 2500, "aug": 3000, "sep": 1500 }, "check_cents": 2200, "price_elasticity_bp": -15000, "satisfaction_elasticity_bp": 6000 },
    "cost_of_sales_bp": 2800, "direct_per_use_cents": 0, "wage_cents": 200000, "capacity_tenths_per_staff_day": 100,
    "wear_pts_per_month": 350,
    "maintenance": { "deferred": { "cost_cents": 30000, "equipment_pts": 0 }, "regular": { "cost_cents": 200000, "equipment_pts": 350 }, "intensive": { "cost_cents": 400000, "equipment_pts": 700 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 6,
    "seasonal_plan": { "jun": 2, "jul": 3, "aug": 4, "sep": 2 }, "can_close": true
  },
  "pool_bar": {
    "name": "Bar uz bazen", "class": "food_beverage",
    "revenue": { "type": "per_guest", "usage_bp": { "jun": 2500, "jul": 3500, "aug": 4000, "sep": 2500 }, "check_cents": 1000, "price_elasticity_bp": -12000, "satisfaction_elasticity_bp": 4000 },
    "cost_of_sales_bp": 2700, "direct_per_use_cents": 0, "wage_cents": 185000, "capacity_tenths_per_staff_day": 120,
    "wear_pts_per_month": 450,
    "maintenance": { "deferred": { "cost_cents": 20000, "equipment_pts": 0 }, "regular": { "cost_cents": 150000, "equipment_pts": 300 }, "intensive": { "cost_cents": 300000, "equipment_pts": 600 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 6,
    "seasonal_plan": { "jun": 3, "jul": 4, "aug": 5, "sep": 3 }, "can_close": true
  },
  "spa": {
    "name": "Wellness i spa", "class": "other_operated",
    "revenue": { "type": "per_guest", "usage_bp": { "jun": 800, "jul": 900, "aug": 1000, "sep": 900 }, "check_cents": 4500, "price_elasticity_bp": -10000, "satisfaction_elasticity_bp": 6000 },
    "cost_of_sales_bp": 1200, "direct_per_use_cents": 300, "wage_cents": 230000, "capacity_tenths_per_staff_day": 60,
    "wear_pts_per_month": 250,
    "maintenance": { "deferred": { "cost_cents": 30000, "equipment_pts": 0 }, "regular": { "cost_cents": 250000, "equipment_pts": 250 }, "intensive": { "cost_cents": 500000, "equipment_pts": 500 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 6,
    "seasonal_plan": { "jun": 1, "jul": 2, "aug": 2, "sep": 1 }, "can_close": true
  },
  "parking": {
    "name": "Parking", "class": "rentals", "revenue": { "type": "fixed", "monthly_cents": 0, "per_guest_night_cents": 150 },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 0, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 100,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 50000, "equipment_pts": 100 }, "intensive": { "cost_cents": 100000, "equipment_pts": 200 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  },
  "retail_lease": {
    "name": "Najam lokala", "class": "rentals", "revenue": { "type": "fixed", "monthly_cents": 600000, "per_guest_night_cents": 0 },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 0, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 0,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 0, "equipment_pts": 0 }, "intensive": { "cost_cents": 0, "equipment_pts": 0 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  },
  "admin": {
    "name": "Uprava", "class": "admin_general", "revenue": { "type": "none" },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 300000, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 0,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 0, "equipment_pts": 0 }, "intensive": { "cost_cents": 0, "equipment_pts": 0 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  },
  "sales": {
    "name": "Prodaja i marketing", "class": "sales_marketing", "revenue": { "type": "none" },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 280000, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 0,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 0, "equipment_pts": 0 }, "intensive": { "cost_cents": 0, "equipment_pts": 0 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  },
  "maintenance": {
    "name": "Održavanje", "class": "maintenance", "revenue": { "type": "none" },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 240000, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 0,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 0, "equipment_pts": 0 }, "intensive": { "cost_cents": 0, "equipment_pts": 0 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 0, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  },
  "energy": {
    "name": "Energija i voda", "class": "utilities", "revenue": { "type": "none" },
    "cost_of_sales_bp": 0, "direct_per_use_cents": 0, "wage_cents": 0, "capacity_tenths_per_staff_day": 0,
    "wear_pts_per_month": 0,
    "maintenance": { "deferred": { "cost_cents": 0, "equipment_pts": 0 }, "regular": { "cost_cents": 0, "equipment_pts": 0 }, "intensive": { "cost_cents": 0, "equipment_pts": 0 } },
    "fixed_monthly_cents": 0, "per_occupied_room_cents": 600, "seasonal_max": 0,
    "seasonal_plan": { "jun": 0, "jul": 0, "aug": 0, "sep": 0 }, "can_close": false
  }
}
```

`content/hotel/v2/profiles.json` (Σ stalnih i sezonaca = v1 profil; stalni troškovi ≈ v1 `fixed_cents` uz energiju po
noći; `name` mijenja ime odjela za taj profil):

```json
{
  "A": {
    "departments": {
      "rooms": { "open": true, "permanent": 11, "seasonal": 13, "condition_pts": 7500, "satisfaction_pts": 8000 },
      "restaurant": { "open": true, "permanent": 7, "seasonal": 9, "condition_pts": 7800, "satisfaction_pts": 8200 },
      "lobby_bar": { "open": true, "permanent": 2, "seasonal": 2, "condition_pts": 7600, "satisfaction_pts": 8000 },
      "spa": { "open": true, "permanent": 3, "seasonal": 2, "condition_pts": 8000, "satisfaction_pts": 8400 },
      "rooftop_bar": { "open": true, "permanent": 0, "seasonal": 2, "condition_pts": 8200, "satisfaction_pts": 8500 },
      "parking": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 7000, "satisfaction_pts": 7000 },
      "admin": { "open": true, "permanent": 2, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 3300000 },
      "sales": { "open": true, "permanent": 1, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 1200000 },
      "maintenance": { "open": true, "permanent": 2, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 1800000 },
      "energy": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 2800000 }
    },
    "fixed_charges_cents": { "insurance": 900000, "lease": 4500000, "property_tax": 800000 }
  },
  "B": {
    "departments": {
      "rooms": { "open": true, "permanent": 13, "seasonal": 15, "condition_pts": 4000, "satisfaction_pts": 6500 },
      "restaurant": { "open": true, "permanent": 8, "seasonal": 10, "condition_pts": 5000, "satisfaction_pts": 6400 },
      "pool_bar": { "open": true, "permanent": 2, "seasonal": 3, "condition_pts": 3500, "satisfaction_pts": 6000 },
      "parking": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 6000, "satisfaction_pts": 6500 },
      "retail_lease": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 6000, "satisfaction_pts": 6500 },
      "admin": { "open": true, "permanent": 2, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 2900000 },
      "sales": { "open": true, "permanent": 1, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 800000 },
      "maintenance": { "open": true, "permanent": 2, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 1400000 },
      "energy": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 2400000 }
    },
    "fixed_charges_cents": { "insurance": 800000, "lease": 4000000, "property_tax": 700000 }
  },
  "C": {
    "departments": {
      "rooms": { "open": true, "permanent": 12, "seasonal": 14, "condition_pts": 6000, "satisfaction_pts": 7000 },
      "restaurant": { "open": true, "permanent": 8, "seasonal": 10, "condition_pts": 6500, "satisfaction_pts": 7000 },
      "lobby_bar": { "open": true, "permanent": 2, "seasonal": 3, "condition_pts": 6500, "satisfaction_pts": 7000 },
      "spa": { "name": "Mali wellness", "open": true, "permanent": 2, "seasonal": 1, "condition_pts": 6500, "satisfaction_pts": 7000 },
      "parking": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 6500, "satisfaction_pts": 7000 },
      "admin": { "open": true, "permanent": 2, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 3000000 },
      "sales": { "open": true, "permanent": 1, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 1000000 },
      "maintenance": { "open": true, "permanent": 3, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 1500000 },
      "energy": { "open": true, "permanent": 0, "seasonal": 0, "condition_pts": 10000, "satisfaction_pts": 0, "fixed_monthly_cents": 2500000 }
    },
    "fixed_charges_cents": { "insurance": 800000, "lease": 4000000, "property_tax": 700000 }
  }
}
```

(Provjera zbrojeva: A stalni 11+7+2+3+0+2+1+2 = 28, sezonci 13+9+2+2+2 = 28; B 13+8+2+2+1+2 = 28 i 15+10+3 = 28;
C 12+8+2+2+2+1+3 = 30 i 14+10+3+1 = 28 — jednako v1 profilima.)

- [ ] **Step 1: Testovi (padaju)** — `crates/content/tests/v2_departments.rs`:

```rust
use content::{Snapshot, repo_dir};
use sim_hotel_v2::department::{DeptCode, UsaliClass};
use sim_hotel_v2::Profile;

fn v2() -> sim_hotel_v2::Catalog {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap().v2.unwrap()
}

#[test]
fn content_v2_loads_and_validates() {
    v2().validate().unwrap();
}

#[test]
fn profiles_have_the_departments_from_model_v2() {
    let c = v2();
    let codes = |p: Profile| -> Vec<String> {
        c.profile_v2(p).unwrap().departments.keys().map(|k| k.as_str().to_owned())
            .filter(|k| !["rooms", "admin", "sales", "maintenance", "energy"].contains(&k.as_str())).collect()
    };
    assert_eq!(codes(Profile::A), ["lobby_bar", "parking", "restaurant", "rooftop_bar", "spa"]);
    assert_eq!(codes(Profile::B), ["parking", "pool_bar", "restaurant", "retail_lease"]);
    assert_eq!(codes(Profile::C), ["lobby_bar", "parking", "restaurant", "spa"]);
    assert_eq!(c.dept_name(Profile::C, &DeptCode("spa".into())).unwrap(), "Mali wellness");
    assert_eq!(c.dept_name(Profile::A, &DeptCode("spa".into())).unwrap(), "Wellness i spa");
}

#[test]
fn staff_per_profile_matches_v1_profile() {
    let c = v2();
    for p in Profile::ALL {
        let v = c.profile_v2(p).unwrap();
        let perm: u16 = v.departments.values().map(|d| d.permanent).sum();
        let seas: u16 = v.departments.values().map(|d| d.seasonal).sum();
        let base = c.base.profile(p).unwrap();
        assert_eq!((perm, seas), (base.permanent, base.seasonal), "{p:?}");
    }
}

#[test]
fn fixed_charges_are_below_gop_items_with_names() {
    let c = v2();
    let names: Vec<&str> = c.economy.fixed_charges.values().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Osiguranje", "Najam zgrade i opreme", "Porez na imovinu"]);
    assert!(c.departments.values().any(|d| d.class == UsaliClass::Rooms));
}

#[test]
fn unknown_department_in_profile_is_rejected() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["profiles"]["C"]["departments"]["casino"] = v["v2"]["profiles"]["C"]["departments"]["parking"].clone();
    assert!(Snapshot::from_value(&v).unwrap().build().is_err());
}

#[test]
fn degenerate_v2_still_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    assert_eq!(c.v2.unwrap(), sim_hotel_v2::degenerate(&c.catalog));
}
```

(Redoslijed u `fixed_charges` je abecedni po kodu: `insurance`, `lease`, `property_tax`.)

- [ ] **Step 2:** `cargo test -p content --test v2_departments` → FAIL.
- [ ] **Step 3: DTO-i** (`deny_unknown_fields`, `revenue` kao `#[serde(tag = "type", rename_all = "snake_case")]`) i
  graditelj; razredi iz `UsaliClass::from_code`; nepoznat kod → `ContentError::Invalid` s imenom polja.
- [ ] **Step 4:** `cargo test -p content; cargo test --workspace` → zeleno (igra još ne čita odjele, Task 11).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/10: sadržaj v2 — odjeli po profilu i raspad fiksnih troškova".

---

### Task 11: Odjeli u stanju i planu v2 (oblik, Smještaj, roboti, DTO)

**Veličina:** M, 3–4 h · **Ovisi o:** Task 9, Task 10 · **Paralelno s:** ništa iz jezgre (Task 13 smije) · **Model:** sonnet

**Cilj:** v2 stanje drži **odjele** umjesto agregata `permanent/seasonal/equipment_pts/fixed_cents`; plan drži
**postavke po odjelu**; v1 sažetak se izvodi (MODEL_V2 §3.8). Obračun u ovom tasku koristi samo odjel Smještaj (ostali
se nose, a prihod im dolazi u Tasku 12); degenerirana ekvivalencija mora ostati zelena.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{state,plan,resolve,market,indices,robots,season,explain}.rs`,
  `crates/sim-hotel-v2/tests/{common/mod.rs,period.rs}`, `crates/game/src/{model,dto_v2,dto,game}.rs` (`game.rs`: samo
  `Game::plan_from_dto` — dopuna odjela, vidi niže), `crates/game/tests/model_v2.rs` (jedan novi test),
  `crates/app/src/error.rs` (samo mapiranje šest novih varijanti `sim_hotel_v2::PlanError` — vidi niže)
- Create: `crates/sim-hotel-v2/tests/departments_state.rs`
- Mehaničke izmjene literala (Global Constraints; novo polje `PlanDto.departments: BTreeMap::new()`, očekivanja se ne
  mijenjaju): `crates/api/src/schema.rs:176` (`impl From<&PlanBody> for PlanDto`), `crates/ai/tests/request.rs:152`,
  `crates/game/tests/events.rs:18`, `crates/game/tests/mentor.rs:32` i `crates/game/tests/mentor.rs:724` (retci prema
  `main` @ 5d44614 — ako ih je raniji task pomaknuo, isti literal `PlanDto {` u istoj datoteci).
- Ne dirati: `department.rs` (osim ako fali `pub` — dodaj), `crates/content/**`, `crates/api/**` (osim gornjeg
  retka), `crates/game/src/{view,ai_input,kpi}.rs` (`summary` ostaje bez kataloga, pa se pozivi ne mijenjaju), web.

**Interfaces:**
- Consumes: Task 9 (`DeptCode`, `DeptState`, `DeptPlan`, `allocate_seasonal`, `Catalog::{dept, profile_v2}`),
  Task 10 (stvarni sadržaj za igru).
- Produces:

```rust
// crates/sim-hotel-v2/src/state.rs
pub struct HotelState {
    pub category: Category, pub rooms: u16, pub wage_pct: u16, pub extra_wage_cents: i64,
    pub morale_pts: i32, pub experience_pts: i32, pub satisfaction_pts: i32, pub rating: u16, pub cash_cents: i64,
    pub load_hundredths: u16, pub turnover_bp: u16, pub renovated: bool, pub pending: Pending, pub groups: Vec<GroupBlock>,
    pub departments: BTreeMap<DeptCode, DeptState>,
    /// Mjesečni stalni dio neraspoređenih odjela (iz profila) i mjesečne fiksne naknade.
    pub undistributed_fixed_cents: BTreeMap<DeptCode, i64>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
}
impl HotelState {
    pub fn from_profile(cat: &Catalog, p: Profile) -> Result<HotelState, ResolveError>;
    pub fn staff(&self) -> u16;                 // Σ stalni + sezonci svih odjela
    pub fn rooms_state(&self) -> Result<&DeptState, ResolveError>;
    /// v1 sažetak: permanent/seasonal = zbrojevi; equipment = Smještaj ako nema drugih operativnih odjela
    /// (`DeptState.class` ∈ {FoodBeverage, OtherOperated}), inače div_round(7 000·Smještaj + 3 000·prosjek ostalih
    /// operativnih, 10 000); fixed_cents = Σ neraspoređeni + Σ naknade.
    /// **Potpis iz Taska 2 ostaje** (bez kataloga, bez `Result`) — `CoreState::summary(&self)` iz Taska 6 i njegovi
    /// pozivi u `game` se ne mijenjaju. Razred odjela čita iz `DeptState.class` (Task 9). Zbrojevi `saturating_add`;
    /// 70/30 kroz `engine::calc` s `.unwrap_or(stanje Smještaja)` (vrijednosti su u 0..=10 000 po `Catalog::validate`,
    /// pa ta grana ne radi — komentar to kaže).
    pub fn summary(&self) -> sim_hotel::HotelState;
}
// crates/sim-hotel-v2/src/plan.rs
pub struct PeriodPlan {
    pub price_cents: i64, pub marketing_cents: i64, pub wage_pct: u16, pub training: Training,
    pub renovation: bool, pub reply_reviews: bool,
    pub departments: BTreeMap<DeptCode, DeptPlan>,
}
impl PeriodPlan {
    pub fn seasonal_total(&self) -> u16;
    pub fn rooms_maintenance(&self) -> Maintenance;
    // default_for / validate / carry_into / limits — isti potpisi kao Task 3
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeptLimits { pub seasonal_min: u16, pub seasonal_max: u16, pub price_level_min_pct: u16, pub price_level_max_pct: u16, pub can_close: bool }
#[derive(Clone, Debug, PartialEq, Eq)] // bez `Copy` (mapa)
pub struct PlanLimits { /* Task 3 polja */ pub seasonal_step_total: u16, pub departments: BTreeMap<DeptCode, DeptLimits> }
pub enum PlanError { /* Task 3 … */ DepartmentUnknown, DepartmentMissing, DepartmentCannotClose, DepartmentSeasonalOutOfRange, ClosedWithSeasonal, PriceLevelOutOfRange }
```

Pravila plana: plan mora imati točno odjele stanja; Smještaj uvijek otvoren i razine cijene 100; `seasonal_target ≤
seasonal_max` odjela; zatvoren odjel ima 0 sezonaca i smije se zatvoriti samo uz `can_close`; razina cijene u
`[price_level_min_pct, price_level_max_pct]`; Σ |cilj − trenutno| po odjelima ≤ `coeffs.seasonal_step`.

**Redoslijed provjera u `validate`** (vraća se prva pogođena greška; `resolve_period` zove `validate` za svaki plan,
kao v1 `resolve.rs:451`): (1) provjere iz Taska 3 bez sezonaca — cijena (cijeli euro, raspon), marketing, plaća,
obnova; (2) **struktura**: odjel kojeg nema u stanju → `DepartmentUnknown`, odjel stanja kojeg nema u planu →
`DepartmentMissing`; (3) **pravila odjela**, redom koda odjela: zatvoren bez `can_close` (i zatvoren Smještaj) →
`DepartmentCannotClose`, zatvoren s `seasonal_target > 0` → `ClosedWithSeasonal`, `seasonal_target > seasonal_max`
odjela → `DepartmentSeasonalOutOfRange`, razina cijene izvan granica (Smještaj ≠ 100) → `PriceLevelOutOfRange`, pa
ukupno `seasonal_total() > base.actions.seasonal_max` → `SeasonalOutOfRange`; (4) **korak** (zadnji): Σ |cilj −
trenutno| > `coeffs(p.kind).seasonal_step` → `SeasonalStepTooBig`. Zato test koji zatvara odjel dobiva grešku odjela
čak i kad bi plan prešao i korak. Plan koji **mora proći** obračun smije u jednom krugu pomaknuti sezonce najviše za
korak (tjedan 5, mjesec 10): testovi koji režu odjel polaze od `common::hold_plan` (svi odjeli na trenutnim
sezoncima, Σ|Δ| = 0) i režu za najviše korak po krugu (`trenutno.saturating_sub(step)`).

Zadani plan:
`total = clamp(Σ avg_u16(plan_odjela, p), Σtrenutno − step, Σtrenutno + step)`, raspodjela `allocate_seasonal`, razine
100, održavanje redovno svugdje, otvorenost kao u stanju. `carry_into`: postavke odjela se prenose (zatvoreni ostaju
zatvoreni, sezonci stisnuti na `seasonal_max`). Robot: v1 strategija daje ukupne sezonce i razinu održavanja kao u
Tasku 3; raspodjela `allocate_seasonal` prema planu odjela; ista razina održavanja u svim odjelima; razina cijene 100
(Nasumični: jednoliko u granicama, u koracima od 5).

Obračun u ovom tasku: osoblje po odjelu iz plana (najam/puštanje po odjelu, moral puštanja kao v1 ako je ikoga
pušteno); kapacitet tržišta = osoblje Smještaja × dani × `capacity_tenths_per_staff_day` Smještaja / 10;
opterećenje i kazna morala iz Smještaja; oprema u obračunu = stanje Smještaja (pravila v1: trošenje, održavanje
razine Smještaja, obnova, situacije); plaće = Σ plaća svih odjela (svaki svojom `wage_cents`); fiksno = Σ
neraspoređeni (razmjerno); ostali prihod = Σ `ShareOfRooms` odjela. U degeneriranom katalogu to je točno v1.

`crates/game`: `HotelStateV2Dto` dobiva `departments: BTreeMap<String, DeptStateDto>`, `undistributed_fixed_cents`,
`fixed_charges_cents` (bez `permanent/seasonal/equipment_pts/fixed_cents`); `PlanDto` (v1 tip u `dto.rs`) dobiva
`#[serde(default, skip_serializing_if = "BTreeMap::is_empty")] pub departments: BTreeMap<String, DeptPlanDto>` — v1
JSON se ne mijenja (Task 1 test to provjerava); za v2 `PlanDto.seasonal_target = seasonal_total()` i
`maintenance = rooms_maintenance()`; `DeptPlanDto { open: bool, seasonal_target: u16, price_level_pct: u16,
maintenance: MaintenanceCode }`; `DeptStateDto` nosi i `class` (kod iz `UsaliClass::code`). `Model::plan_from_dto`
za v2 traži neprazne `departments` (`seasonal_target` i `maintenance` iz DTO-a se za v2 ne čitaju — izvode se iz
odjela). **`Game::plan_from_dto(h, r, d)`** (Task 6) za v2 s **praznim** `d.departments` prije toga upiše postavke
odjela iz `effective_plan(h, r)` — tako `PUT plans/{round}` bez odjela (do Taska 15 `PlanBody` ih nema; test Taska 7
`round_paths_accept_week_codes` to radi) i dalje prolazi i zadržava postavke odjela.

Greške: šest novih varijanti `sim_hotel_v2::PlanError` (`DepartmentUnknown` … `PriceLevelOutOfRange`) u
`crates/app/src/error.rs` (mapiranje `GameError::PlanV2` iz Taska 6) privremeno idu u `ErrorCode::Internal` s
komentarom „Task 15: `department_plan_invalid`" — kroz API su do Taska 15 nedostižne (odjeli se tada samo dopunjuju iz
važećeg plana); bez `_ =>` grane.

- [ ] **Step 1: Testovi (padaju)** — `tests/departments_state.rs`:

```rust
mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::*;

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

#[test]
fn summary_counts_staff_from_departments() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let sum = s.summary();
    let base = c.base.profile(Profile::C).unwrap(); // testni profil C (30, 20), ne sadržaj (30, 28)
    assert_eq!((sum.permanent, sum.seasonal), (base.permanent, base.seasonal));
}

#[test]
fn summary_equipment_is_seventy_thirty_rooms_and_operated_departments() {
    let c = common::catalog_with_departments();
    let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
    for (k, d) in s.departments.iter_mut() {
        d.condition_pts = if k.as_str() == "rooms" { 6_000 } else { 8_000 };
    }
    assert_eq!(s.summary().equipment_pts, 6_600);
}

#[test]
fn summary_fixed_is_undistributed_plus_charges() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let undist: i64 = s.undistributed_fixed_cents.values().sum();
    let charges: i64 = s.fixed_charges_cents.values().sum();
    assert_eq!(s.summary().fixed_cents, undist + charges);
}

#[test]
fn default_plan_moves_seasonal_within_step_toward_department_plans() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let moved: u16 = plan.departments.iter().map(|(k, d)| d.seasonal_target.abs_diff(s.departments[k].seasonal)).sum();
    assert!(moved <= c.coeffs(PeriodKind::Week).unwrap().seasonal_step);
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
}

#[test]
fn closing_a_department_that_cannot_close_is_rejected() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.departments.get_mut(&code("restaurant")).unwrap().open = false;
    plan.departments.get_mut(&code("restaurant")).unwrap().seasonal_target = 0;
    // Uz pomake zadanog plana ovo prelazi i korak 10 — pravilo odjela se provjerava prije koraka.
    assert_eq!(plan.validate(&s, &p, &c), Err(PlanError::DepartmentCannotClose));
}

#[test]
fn closed_department_with_seasonal_is_rejected() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let bar = plan.departments.get_mut(&code("lobby_bar")).unwrap();
    bar.open = false;
    bar.seasonal_target = 1; // izričito > 0, ne ovisi o raspodjeli zadanog plana
    assert_eq!(plan.validate(&s, &p, &c), Err(PlanError::ClosedWithSeasonal));
}

#[test]
fn hold_plan_is_valid_and_moves_nobody() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let plan = common::hold_plan(&s, &p, &c);
    assert!(plan.departments.iter().all(|(k, d)| d.seasonal_target == s.departments[k].seasonal));
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
}

#[test]
fn carried_plan_respects_department_limits() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.departments.get_mut(&code("spa")).unwrap().price_level_pct = 120;
    let carried = plan.carry_into(&s, &month_period(Month::Jul), &c).unwrap();
    assert_eq!(carried.departments[&code("spa")].price_level_pct, 120);
    assert_eq!(carried.validate(&s, &month_period(Month::Jul), &c), Ok(()));
}

#[test]
fn robots_produce_valid_department_plans_every_week() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &calendar::season_periods(PeriodKind::Week)).unwrap();
    for s in Strategy::ALL {
        run_season(Profile::A, s, 3, PeriodKind::Week, &c, &[], &r).unwrap();
    }
}
```

`tests/common/mod.rs` dobiva `pub fn catalog_with_departments() -> Catalog`: `degenerate(&test_catalog())` pa odjeli
`restaurant`, `lobby_bar`, `spa`, `parking`, `rooftop_bar` i neraspoređeni s brojkama iz `content/hotel/v2/departments.json`
(Task 10, prepisane u kod — ovaj paket ne smije ovisiti o `content`) i profili C/A s raspodjelom osoblja iz
`profiles.json` prilagođenom testnom katalogu (Σ = v1 testni profil: C 30 stalnih i 20 sezonaca,
`crates/sim-hotel-v2/tests/common/mod.rs` = kopija `crates/sim-hotel/tests/common/mod.rs:7-23`). U profilu C
`restaurant` ima 10 sezonaca i `lobby_bar` 3 (kao `profiles.json`), razliku do Σ nosi `rooms` — testovi Taska 12 na
to računaju. I `pub fn hold_plan(s: &HotelState, p: &Period, c: &Catalog) -> PeriodPlan`: `PeriodPlan::default_for`,
pa svakom odjelu `seasonal_target = s.departments[k].seasonal` (Σ|Δ| = 0; plan koji uvijek prolazi korak).
Ekvivalencijski testovi (Task 2–3) i dalje koriste `degenerate` i moraju ostati zeleni.

`crates/game/tests/model_v2.rs` — novi test (dopuna odjela u `Game::plan_from_dto`):

```rust
#[test]
fn v2_plan_dto_without_departments_takes_them_from_the_effective_plan() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let CorePlan::V2(eff) = g.effective_plan(h, r).unwrap() else { panic!("v2") };
    let mut dto = CorePlan::V2(eff.clone()).to_dto();
    assert!(!dto.departments.is_empty());
    dto.departments.clear();
    let CorePlan::V2(p) = g.plan_from_dto(h, r, &dto).unwrap() else { panic!("v2") };
    assert_eq!(p.departments, eff.departments);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2` → FAIL (novi testovi i prevođenje).
- [ ] **Step 3: Implementiraj** jezgru, pa `crates/game` (DTO i `model.rs`); `cargo test --workspace` mora proći,
  uključujući `game --test equivalence`, `game --test model_v2`, `game --test v1_frozen`.
- [ ] **Step 4: Sve provjere** (i `probe-mutacija`) i commit „Sloj 1/11: odjeli u stanju i planu v2; v1 sažetak izveden".

---

### Task 12: Odjeli u obračunu — računi odjela, neraspoređeni troškovi, fiksne naknade, sažetak

**Veličina:** M, 3–4 h · **Ovisi o:** Task 11 · **Model:** sonnet (opus ako se identiteti ne slažu)

**Cilj:** svaki operativni odjel dobiva svoj račun po formulama Taska 9; neraspoređeni odjeli i fiksne naknade ispod
GOP-a; v1 sažetak (prihod, troškovi, GOP, oprema, kvaliteta, zadovoljstvo, opterećenje) slaže se iz odjela; izvješće
razdoblja nosi `accounts: PeriodAccounts`.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{resolve,explain,market}.rs`
- Create: `crates/sim-hotel-v2/tests/accounts.rs`
- Ne dirati: `department.rs` formule (ako je formula kriva — stani, javi, Ruling), `crates/game/**` (osim ako se ne
  prevodi zbog novog polja — tada samo `..` u uzorku), sadržaj.

**Interfaces:**
- Consumes: Task 9 (`dept_step`, `undistributed_step`, `AssignedCosts`, `PeriodAccounts`), Task 11 (stanje, plan).
- Produces: `PeriodReport { period, before, after, summary, pub accounts: PeriodAccounts }`.

Slaganje (redoslijed u `resolve_period`):
1. Osoblje po odjelu (Task 11), tržište i Smještaj kao u Tasku 11 → `sold`, `guest_nights = div_round(sold·guests_per_room_hundredths, 100)`.
2. Ljudi (moral, iskustvo, odlasci) — kazna opterećenja = **ponderirana prema osoblju** kazna odjela s kapacitetom
   (`capacity_tenths_per_staff_day > 0` i osoblje > 0); sažetak `load_hundredths` = isti ponder nad opterećenjima.
   Odlasci po odjelu `div_round(osoblje·turnover·dani, 10 000·D₀)`; trošak zapošljavanja (najam + odlasci) × `hire_cents`.
3. `dept_step` za svaki odjel osim Smještaja i neraspoređenih (u redoslijedu koda); Smještaj dobiva `DeptOutcome` iz
   koraka tržišta (prihod soba, plaće Smještaja + `extra_wage_cents` 🟨 P12, `other_direct = CPOR·sold`, stanje po
   pravilima v1, zadovoljstvo odjelom: cilj po v1 formuli s vrijednošću iz cijene sobe, težina = `guest_nights`).
4. Kvaliteta hotela = Σ(kvaliteta·težina)/Σ težina; cilj zadovoljstva hotela = Σ(cilj odjela·težina)/Σ težina, pa v1
   pravilo pomaka (+ obnova, + situacije). U degeneriranom katalogu ima samo Smještaj → točno v1.
5. Neraspoređeni (`undistributed_step`) s dodjelom troškova odluka (Task 9).
6. Fiksne naknade: `prorate(mjesečni, p)` po kodu.
7. Sažetak: `other_revenue = Σ prihoda odjela osim Smještaja`; `labor = Σ plaća (svi odjeli) + zapošljavanje + obuka`;
   `costs = labor + Σ cost_of_sales + Σ other_direct + Σ održavanje + marketing + odgovaranje + Σ stalni neraspoređeni +
   Σ po noći + situacije`; `gop = prihod − costs`; `cash = cash + gop − Σ naknade − obnova`, pa kamata na negativnu
   blagajnu (Pravila razdoblja).
8. Retci objašnjenja sažetka (samo postojeći `Cause`): `Revenue/OtherRevenue = Σ ostalih prihoda`; `Costs/Baseline =
   Σ cost_of_sales + Σ other_direct + Σ po noći` (svih odjela; CPOR se **ne** dodaje posebno — on je već
   `other_direct` Smještaja = `CPOR·sold`, korak 3, pa bi se inače brojao dvaput); `Costs/Maintenance = Σ održavanje`; `Costs/FixedCosts =
   Σ stalni neraspoređeni`; **novi redak** `Cash/FixedCosts = −Σ naknade`; ostalo kao v1. Oprema u sažetku = 70/30
   (Task 11), a `Equipment` retci objašnjavaju promjenu **sažetka** (Baseline = promjena Smještaja, Load/Maintenance
   po Smještaju; razlika zbog ostalih odjela ide u redak `Equipment/Maintenance` — test provjerava da se retci zbrajaju).

- [ ] **Step 1: Testovi (padaju)** — `tests/accounts.rs`:

```rust
mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, season_periods};
use sim_hotel_v2::department::{DeptCode, UsaliClass};
use sim_hotel_v2::*;

fn june_c() -> (Catalog, PeriodReport) {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    (c, r)
}

fn lines(r: &PeriodReport, var: Var) -> i64 {
    r.summary.lines.iter().filter(|l| l.var == var).map(|l| l.amount).sum()
}

#[test]
fn accounts_add_up_to_summary() {
    let (_, r) = june_c();
    let a = &r.accounts;
    let m = &r.summary.money;
    let rooms = &a.departments[&DeptCode::rooms()];
    let other: i64 = a.departments.values().filter(|d| d.class != UsaliClass::Rooms).map(|d| d.revenue_cents).sum();
    assert_eq!(rooms.revenue_cents, m.rooms_revenue_cents);
    assert_eq!(other, m.other_revenue_cents);
    let dept_costs: i64 = a.departments.values().map(|d| d.payroll_cents + d.cost_of_sales_cents + d.other_direct_cents).sum();
    let undist: i64 = a.undistributed.values().map(|u| u.total_cents).sum();
    assert_eq!(dept_costs + undist, m.costs_cents);
    assert_eq!(m.rooms_revenue_cents + m.other_revenue_cents - m.costs_cents, m.gop_cents);
    let charges: i64 = a.fixed_charges.values().sum();
    assert_eq!(r.after.cash_cents - r.before.cash_cents, m.gop_cents - charges - m.capex_cents - m.interest_cents);
}

#[test]
fn cost_and_cash_lines_explain_the_summary() {
    let (_, r) = june_c();
    assert_eq!(lines(&r, Var::Costs), r.summary.money.costs_cents);
    assert_eq!(lines(&r, Var::Cash), r.after.cash_cents - r.before.cash_cents);
    assert_eq!(lines(&r, Var::Equipment), r.summary.after.equipment_pts - r.summary.before.equipment_pts);
}

#[test]
fn maintenance_is_booked_in_the_maintenance_service() {
    let (_, r) = june_c();
    let spent: i64 = r.accounts.departments.values().map(|d| d.maintenance_cents).sum();
    let pom = r.accounts.undistributed.values().find(|u| u.code.as_str() == "maintenance").unwrap();
    assert_eq!(pom.assigned.maintenance_cents, spent);
}

#[test]
fn closing_the_bar_stops_its_revenue_but_not_permanent_wages() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    // Svi ostali odjeli na trenutnim sezoncima, bar 3 → 0: Σ|Δ| = 3 ≤ korak 10, pa `resolve_period` (koji zove
    // `validate`) prolazi.
    let mut plan = common::hold_plan(&s, &p, &c);
    let bar = plan.departments.get_mut(&DeptCode("lobby_bar".into())).unwrap();
    bar.open = false;
    bar.seasonal_target = 0;
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    let o = &r.accounts.departments[&DeptCode("lobby_bar".into())];
    assert_eq!(o.revenue_cents, 0);
    assert!(o.payroll_cents > 0);
    assert!(!r.after.departments[&DeptCode("lobby_bar".into())].open);
}

#[test]
fn understaffed_restaurant_lowers_its_satisfaction_and_next_week_usage() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    let rest = DeptCode("restaurant".into());
    let step = c.coeffs(PeriodKind::Week).unwrap().seasonal_step; // 5
    // Oba puta svi odjeli stoje na trenutnim sezoncima; „cut" svaki tjedan reže restoran za najviše korak
    // (10 → 5 → 0 → 0 …), pa svaki plan prolazi `validate` u `resolve_period`.
    let run = |cut: bool| {
        run_season_with(Profile::C, PeriodKind::Week, &c, &[], &r, &mut |s, p| {
            let mut plan = common::hold_plan(s, p, &c);
            if cut {
                let now = s.departments[&rest].seasonal;
                plan.departments.get_mut(&rest).unwrap().seasonal_target = now.saturating_sub(step);
            }
            Ok(plan)
        }, &mut |_| Ok(0)).unwrap()
    };
    let (full, cut) = (run(false), run(true));
    let at = |s: &SeasonResult, i: usize| s.periods[i].0.accounts.departments[&rest].clone();
    assert!(at(&cut, 3).quality_pts < at(&full, 3).quality_pts);
    assert!(at(&cut, 3).satisfaction_after_pts < at(&full, 3).satisfaction_after_pts);
    assert!(at(&cut, 4).usage_bp < at(&full, 4).usage_bp);
}

#[test]
fn weekly_season_with_departments_is_reproducible() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    let a = run_season(Profile::A, Strategy::Random, 5, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    let b = run_season(Profile::A, Strategy::Random, 5, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    assert_eq!(a, b);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2 --test accounts` → FAIL.
- [ ] **Step 3: Implementiraj** po redoslijedu gore. Nakon svakog koraka `cargo test -p sim-hotel-v2 --test equivalence`
  mora biti zelen (degenerirani katalog nema operativnih odjela osim `ostalo` s `ShareOfRooms`).
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno (uključujući `game --test equivalence`).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/12: odjeli u obračunu — računi odjela, neraspoređeni troškovi, fiksne naknade".

---

### Task 13: Račun dobiti po USALI-ju i odstupanje od budžeta (čisto)

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 9 · **Paralelno s:** Task 10, 11, 12 · **Model:** sonnet

**Cilj:** čista funkcija koja od `PeriodAccounts` složi račun dobiti redoslijedom USALI-ja (11. izdanje, bez naknade za
upravljanje i informatike — 🟨 P10) i usporedi ostvareno s budžetom po retku.

**Files:**
- Create: `crates/sim-hotel-v2/src/usali.rs`, `crates/sim-hotel-v2/tests/usali.rs`,
  `crates/sim-hotel-v2/tests/common/usali.rs` (`pub fn june_accounts() -> PeriodAccounts`, ručno složen iz brojki
  niže; u podmapi `common/`, pa ga cargo ne prevodi kao zaseban test)
- Modify: `crates/sim-hotel-v2/src/lib.rs` (`pub mod usali;`)
- Ne dirati: sve ostalo (i `tests/common/mod.rs`).

**Interfaces (Produces):**

```rust
// crates/sim-hotel-v2/src/usali.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptAccount { pub code: DeptCode, pub class: UsaliClass, pub revenue_cents: i64, pub payroll_cents: i64, pub cost_of_sales_cents: i64, pub other_cents: i64, pub expenses_cents: i64, pub profit_cents: i64, pub margin_bp: Option<i64> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndistributedAccount { pub code: DeptCode, pub kind: UndistributedKind, pub payroll_cents: i64, pub other_cents: i64, pub total_cents: i64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsaliStatement {
    pub days: u32, pub room_nights: u32,
    /// Smještaj, hrana i piće, ostali operativni, ostali prihodi (najam, parking) — redom `UsaliClass::rank`, pa kod.
    pub departments: Vec<DeptAccount>,
    pub total_revenue_cents: i64, pub total_departmental_expenses_cents: i64, pub departmental_profit_cents: i64,
    pub undistributed: Vec<UndistributedAccount>, pub total_undistributed_cents: i64,
    pub gop_cents: i64, pub gop_margin_bp: Option<i64>, pub goppar_cents: Option<i64>,
    pub fixed_charges: Vec<(String, i64)>, pub total_fixed_charges_cents: i64,
    pub ebitda_cents: i64, pub ebitda_margin_bp: Option<i64>,
    pub interest_cents: i64, pub capex_cents: i64,
}
pub fn statement(a: &PeriodAccounts) -> Calc<UsaliStatement>;
/// Zbroj više krugova (npr. sezona do sada); marže i GOPPAR iz zbrojeva.
pub fn sum_statements(xs: &[UsaliStatement]) -> Calc<UsaliStatement>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section { Revenue, DepartmentalExpenses, DepartmentalProfit, Undistributed, Gop, FixedCharges, Ebitda }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind { Revenue, Expense, Result }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarianceLine { pub section: Section, pub code: String, pub kind: LineKind, pub is_total: bool, pub actual_cents: i64, pub budget_cents: i64, pub variance_cents: i64, pub favorable: bool }
/// Retci oba izvješća (unija kodova; kojeg nema → 0), redom sekcija; `variance = actual − budget`;
/// povoljno: prihod i rezultat ≥ 0, trošak ≤ 0. Svaka sekcija s više redaka ima i redak `is_total` (kod "total");
/// sekcija Gop ima jedan redak koda "gop", sekcija Ebitda jedan redak koda "ebitda".
pub fn budget_variance(actual: &UsaliStatement, budget: &UsaliStatement) -> Vec<VarianceLine>;
```

Pravila: rashodi odjela = plaće + `cost_of_sales` + `other_direct` (održavanje **nije** ovdje, nego u „Održavanju");
`margin_bp = div_round(profit·10 000, revenue)` ili `None` kad je prihod 0; `room_nights = rooms·days`;
`goppar = div_round(gop, room_nights)`; neraspoređeni: plaće = `payroll`, ostalo = `fixed + per_room + Σ assigned`;
`ebitda = gop − Σ naknade`; kamata i obnova su ispod EBITDA, samo za prikaz.

- [ ] **Step 1: Testovi (padaju)** — `tests/usali.rs` (ručno složeni `PeriodAccounts` s tri odjela: Smještaj prihod
  35 000 000, plaće 5 200 000, izravno 5 702 400; restoran iz Taska 9 (6 720 000 / 3 780 000 / 2 150 400 / 0);
  parking `Rentals` 699 900; neraspoređeni admin 5 905 000, sales 1 000 000 + marketing 500 000, maintenance
  1 500 000 + održavanje 2 300 000, energy 2 500 000 + 2 592·600; naknade insurance 800 000, lease 4 000 000,
  property_tax 700 000; 120 soba, 30 dana):

```rust
use sim_hotel_v2::usali::*;

// tests/common/usali.rs: pub fn june_accounts() -> PeriodAccounts (brojke iz opisa gore); podmapa se ne prevodi kao test.
#[path = "common/usali.rs"]
mod fixtures;

#[test]
fn statement_identities_hold() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    assert_eq!(s.total_revenue_cents, 35_000_000 + 6_720_000 + 699_900);
    let expenses: i64 = s.departments.iter().map(|d| d.expenses_cents).sum();
    assert_eq!(s.departmental_profit_cents, s.total_revenue_cents - expenses);
    assert_eq!(s.gop_cents, s.departmental_profit_cents - s.total_undistributed_cents);
    assert_eq!(s.total_fixed_charges_cents, 5_500_000);
    assert_eq!(s.ebitda_cents, s.gop_cents - 5_500_000);
    assert_eq!(s.room_nights, 3_600);
    assert_eq!(s.goppar_cents, Some((s.gop_cents * 2 + 3_600) / 7_200));
}

#[test]
fn rooms_come_first_and_rentals_have_no_departmental_expense() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let codes: Vec<&str> = s.departments.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["rooms", "restaurant", "parking"]);
    assert_eq!(s.departments[2].expenses_cents, 0);
    assert_eq!(s.departments[1].margin_bp, Some(1_175));
}

#[test]
fn maintenance_spend_sits_in_the_maintenance_service_not_in_departments() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let pom = s.undistributed.iter().find(|u| u.code.as_str() == "maintenance").unwrap();
    assert_eq!(pom.other_cents, 1_500_000 + 2_300_000);
}

#[test]
fn variance_signs_follow_line_kind() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut worse = fixtures::june_accounts();
    worse.departments.get_mut(&sim_hotel_v2::department::DeptCode::rooms()).unwrap().revenue_cents += 100_000;
    let b = statement(&worse).unwrap(); // budžet s većim prihodom soba
    let v = budget_variance(&a, &b);
    let rooms_rev = v.iter().find(|l| l.section == Section::Revenue && l.code == "rooms").unwrap();
    assert_eq!((rooms_rev.variance_cents, rooms_rev.favorable), (-100_000, false));
    let rooms_exp = v.iter().find(|l| l.section == Section::DepartmentalExpenses && l.code == "rooms").unwrap();
    assert_eq!((rooms_exp.variance_cents, rooms_exp.favorable), (0, true));
}

#[test]
fn variance_lists_departments_missing_from_the_budget_with_zero() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut no_parking = fixtures::june_accounts();
    no_parking.departments.remove(&sim_hotel_v2::department::DeptCode("parking".into()));
    let b = statement(&no_parking).unwrap();
    let line = budget_variance(&a, &b).into_iter().find(|l| l.section == Section::Revenue && l.code == "parking").unwrap();
    assert_eq!((line.actual_cents, line.budget_cents, line.favorable), (699_900, 0, true));
}

#[test]
fn four_weeks_sum_like_one_statement() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let sum = sum_statements(&[s.clone(), s.clone()]).unwrap();
    assert_eq!(sum.gop_cents, 2 * s.gop_cents);
    assert_eq!(sum.room_nights, 7_200);
    assert_eq!(sum.gop_margin_bp, s.gop_margin_bp);
}

#[test]
fn margins_are_none_without_revenue() {
    let mut a = fixtures::june_accounts();
    for d in a.departments.values_mut() {
        d.revenue_cents = 0;
    }
    assert_eq!(statement(&a).unwrap().gop_margin_bp, None);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2 --test usali` → FAIL.
- [ ] **Step 3: Implementiraj** `usali.rs` (sve kroz `engine::calc`).
- [ ] **Step 4:** `cargo test -p sim-hotel-v2; echo EXIT=$?`; `node scripts/probe-mutacija.mjs; echo EXIT=$?`.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/13: račun dobiti po USALI-ju i odstupanje od budžeta".

---
