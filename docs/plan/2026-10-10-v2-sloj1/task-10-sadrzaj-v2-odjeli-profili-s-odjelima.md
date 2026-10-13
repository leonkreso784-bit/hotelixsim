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

