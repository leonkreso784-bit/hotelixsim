### Task 3: Pet indeksa i profili A/B/C (`sim-hotel`)

**Files:**
- Create: `crates/sim-hotel/src/index.rs`, `crates/sim-hotel/src/profile.rs`
- Modify: `crates/sim-hotel/src/lib.rs`

**Interfaces:**
- Produces:
  - `sim_hotel::Index` — `enum { Guests, Reviews, Staff, Operations, Finance }`, `Index::ALL: [Index; 5]` (redoslijed G, R, Z, O, F), `index.code() -> &'static str` (`"G"`, `"R"`, `"Z"`, `"O"`, `"F"`).
  - `sim_hotel::IndexValue` — stotinke `0..=10_000`; `IndexValue::new(hundredths: u16) -> Option<IndexValue>`, `value.hundredths() -> u16`.
  - `sim_hotel::IndexSet` — `IndexSet::from_hundredths([u16; 5]) -> Option<IndexSet>` (redoslijed `Index::ALL`), `set.get(Index) -> IndexValue`, `set.iter() -> impl Iterator<Item = (Index, IndexValue)>`, `set.total_hundredths() -> u32`.
  - `sim_hotel::Profile` — `enum { A, B, C }`, `Profile::ALL: [Profile; 3]`, `Profile::from_code(&str) -> Option<Profile>` (samo velika slova `"A"`, `"B"`, `"C"`), `profile.code() -> &'static str`, `profile.starting_indices() -> IndexSet`.

Početne vrijednosti su **ciljevi iz DEMO §5**, zapisani kao konstante. U planu jezgre (sljedeći plan) profili prelaze u `content/hotel/` i računaju se iz stvarnog sloja.

- [ ] **Step 1: Napiši testove koji padaju — na dnu `crates/sim-hotel/src/profile.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;

    #[test]
    fn every_profile_totals_250() {
        for p in Profile::ALL {
            assert_eq!(p.starting_indices().total_hundredths(), 25_000, "profil {}", p.code());
        }
    }

    #[test]
    fn profile_c_is_fifty_everywhere() {
        let c = Profile::C.starting_indices();
        assert!(c.iter().all(|(_, v)| v.hundredths() == 5_000));
    }

    #[test]
    fn profiles_match_demo_section_5() {
        let a = Profile::A.starting_indices();
        assert_eq!(a.get(Index::Reviews).hundredths(), 7_500);
        assert_eq!(a.get(Index::Staff).hundredths(), 3_000);
        let b = Profile::B.starting_indices();
        assert_eq!(b.get(Index::Finance).hundredths(), 7_500);
        assert_eq!(b.get(Index::Reviews).hundredths(), 3_500);
    }

    #[test]
    fn from_code_accepts_only_known_uppercase_codes() {
        assert_eq!(Profile::from_code("A"), Some(Profile::A));
        assert_eq!(Profile::from_code("C"), Some(Profile::C));
        assert_eq!(Profile::from_code("a"), None);
        assert_eq!(Profile::from_code("D"), None);
        assert_eq!(Profile::from_code(""), None);
    }
}
```

I na dnu `crates/sim-hotel/src/index.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_value_rejects_over_100() {
        assert!(IndexValue::new(10_000).is_some());
        assert!(IndexValue::new(10_001).is_none());
    }

    #[test]
    fn index_codes_are_stable_and_ordered() {
        let codes: Vec<&str> = Index::ALL.iter().map(|i| i.code()).collect();
        assert_eq!(codes, ["G", "R", "Z", "O", "F"]);
    }

    #[test]
    fn index_set_rejects_invalid_value() {
        assert!(IndexSet::from_hundredths([5_000, 5_000, 5_000, 5_000, 10_001]).is_none());
    }
}
```

- [ ] **Step 2: Pokreni — mora pasti**

Run: `cargo test -p sim-hotel; echo EXIT=$?` → Expected: greške prevođenja (tipovi ne postoje), `EXIT=101`.

- [ ] **Step 3: Izvedba**

`crates/sim-hotel/src/index.rs` (iznad testova):

```rust
//! Pet indeksa 0–100 (DEMO §4). U jezgri se drže u stotinkama: 0..=10_000.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
    Guests,
    Reviews,
    Staff,
    Operations,
    Finance,
}

impl Index {
    pub const ALL: [Index; 5] = [Index::Guests, Index::Reviews, Index::Staff, Index::Operations, Index::Finance];

    /// Stabilan kod za API i prijevode; nikad se ne mijenja.
    pub fn code(self) -> &'static str {
        match self {
            Index::Guests => "G",
            Index::Reviews => "R",
            Index::Staff => "Z",
            Index::Operations => "O",
            Index::Finance => "F",
        }
    }

    fn position(self) -> usize {
        match self {
            Index::Guests => 0,
            Index::Reviews => 1,
            Index::Staff => 2,
            Index::Operations => 3,
            Index::Finance => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexValue(u16);

impl IndexValue {
    pub const MAX_HUNDREDTHS: u16 = 10_000;

    pub fn new(hundredths: u16) -> Option<Self> {
        (hundredths <= Self::MAX_HUNDREDTHS).then_some(Self(hundredths))
    }

    pub fn hundredths(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexSet([IndexValue; 5]);

impl IndexSet {
    /// Vrijednosti u redoslijedu `Index::ALL` (G, R, Z, O, F).
    pub fn from_hundredths(values: [u16; 5]) -> Option<Self> {
        let mut out = [IndexValue(0); 5];
        for (slot, v) in out.iter_mut().zip(values) {
            *slot = IndexValue::new(v)?;
        }
        Some(Self(out))
    }

    pub fn get(&self, index: Index) -> IndexValue {
        self.0[index.position()]
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, IndexValue)> + '_ {
        Index::ALL.into_iter().map(|i| (i, self.get(i)))
    }

    pub fn total_hundredths(&self) -> u32 {
        self.0.iter().map(|v| u32::from(v.0)).sum()
    }
}
```

Napomena za izvođača: `#[derive(Hash)]` na `Index` je dopušten (ne koristi se `HashMap`). Indeksiranje `self.0[index.position()]` je sigurno jer `position()` vraća 0–4; ako clippy `indexing_slicing` nije uključen, ne treba ništa dodatno.

`crates/sim-hotel/src/profile.rs` (iznad testova):

```rust
//! Profili A/B/C (DEMO §5): isti zbroj 250, drukčije raspoređene snage.
//! Privremeno konstante; u planu jezgre prelaze u content/hotel i računaju se iz stvarnog sloja.

use crate::index::IndexSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    A,
    B,
    C,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::A, Profile::B, Profile::C];

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" => Some(Profile::A),
            "B" => Some(Profile::B),
            "C" => Some(Profile::C),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Profile::A => "A",
            Profile::B => "B",
            Profile::C => "C",
        }
    }

    /// Ciljane početne vrijednosti u stotinkama, redoslijed G, R, Z, O, F.
    pub fn starting_indices(self) -> IndexSet {
        let values = match self {
            Profile::A => [6_000, 7_500, 3_000, 5_000, 3_500],
            Profile::B => [4_500, 3_500, 5_500, 4_000, 7_500],
            Profile::C => [5_000, 5_000, 5_000, 5_000, 5_000],
        };
        IndexSet::from_hundredths(values).expect("profili su u rasponu 0..=10_000")
    }
}
```

`crates/sim-hotel/src/lib.rs` (cijeli sadržaj):

```rust
//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]

pub mod index;
pub mod profile;

pub use index::{Index, IndexSet, IndexValue};
pub use profile::Profile;
```

- [ ] **Step 4: Pokreni — mora proći**

Run: `cargo test -p sim-hotel; echo EXIT=$?` → Expected: `7 passed; 0 failed`, `EXIT=0`.
Run: `cargo clippy --workspace --all-targets -- -D warnings; echo EXIT=$?` → `EXIT=0`. Ako `arithmetic_side_effects` javi `.sum()` u `total_hundredths`, zamijeni s `fold(0u32, |acc, v| acc.checked_add(u32::from(v.0)).expect("najviše 50 000"))`.
Run: `node scripts/check-core-deps.mjs; echo EXIT=$?` → `zabranjenih ovisnosti: 0`, `EXIT=0`.

- [ ] **Step 5: Commit** — `git add crates/sim-hotel` · poruka: „Kostur 3/8: pet indeksa i profili A/B/C"

---

