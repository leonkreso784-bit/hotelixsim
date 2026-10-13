### Task 2: Kocka (`engine::rng`) sa zlatnim vektorima

**Files:**
- Create: `crates/engine/src/rng.rs`, `crates/engine/tests/rng_golden.rs`
- Modify: `crates/engine/src/lib.rs`

**Interfaces:**
- Produces:
  - `engine::rng::Rng` — `Rng::from_seed(seed: u64) -> Rng`, `Rng::derive(seed: u64, labels: &[u64]) -> Rng`, `rng.next_u64() -> u64`, `rng.range(lo: u64, hi: u64) -> u64` (poluotvoreno `[lo, hi)`, panika ako `lo >= hi`), `rng.chance_bp(bp: u32) -> bool` (bazni bodovi `0..=10_000`, panika iznad).
  - `engine::rng::label(s: &str) -> u64` — FNV-1a 64 nad UTF-8 bajtovima.
  - Ponovni izvoz: `engine::Rng`, `engine::label`.

Algoritam: SplitMix64 (Vigna). Zlatni vektori izračunati su **nezavisnom** BigInt izvedbom u Nodeu (`golden.mjs`, 04.10.); vektor za sjeme 1234567 poklapa se s objavljenim referentnim izlazom SplitMix64.

- [ ] **Step 1: Napiši test koji pada — `crates/engine/tests/rng_golden.rs`**

```rust
use engine::{Rng, label};

#[test]
fn next_u64_matches_reference_seed_1234567() {
    let mut rng = Rng::from_seed(1_234_567);
    let got: Vec<u64> = (0..5).map(|_| rng.next_u64()).collect();
    assert_eq!(
        got,
        [
            6_457_827_717_110_365_317,
            3_203_168_211_198_807_973,
            9_817_491_932_198_370_423,
            4_593_380_528_125_082_431,
            16_408_922_859_458_223_821,
        ]
    );
}

#[test]
fn next_u64_matches_reference_seed_0() {
    let mut rng = Rng::from_seed(0);
    let got: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
    assert_eq!(got, [16_294_208_416_658_607_535, 7_960_286_522_194_355_700, 487_617_019_471_545_679]);
}

#[test]
fn label_is_fnv1a_64() {
    assert_eq!(label(""), 14_695_981_039_346_656_037);
    assert_eq!(label("trigger:T1"), 17_546_483_036_339_428_452);
}

#[test]
fn derive_matches_reference() {
    let mut rng = Rng::derive(42, &[3, 7, label("trigger:T1")]);
    let got: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
    assert_eq!(got, [15_554_128_182_792_330_207, 9_332_619_251_129_265_620, 12_170_803_838_678_117_234]);
}

#[test]
fn derive_streams_are_independent() {
    // Dodavanje novog bacanja u jednom toku ne smije pomaknuti drugi tok (ARHITEKTURA §7.2).
    let mut a = Rng::derive(42, &[3, 7, label("trigger:T1")]);
    let mut b = Rng::derive(42, &[3, 7, label("trigger:T2")]);
    let _ = b.next_u64();
    let _ = b.next_u64();
    assert_eq!(a.next_u64(), 15_554_128_182_792_330_207);
}

#[test]
fn range_matches_reference() {
    let mut rng = Rng::from_seed(99);
    let got: Vec<u64> = (0..5).map(|_| rng.range(0, 10_000)).collect();
    assert_eq!(got, [403, 4_564, 9_627, 807, 5_076]);

    let mut rng = Rng::from_seed(99);
    let got: Vec<u64> = (0..5).map(|_| rng.range(7, 13)).collect();
    assert_eq!(got, [12, 7, 8, 12, 11]);
}

#[test]
fn range_stays_in_bounds() {
    let mut rng = Rng::from_seed(5);
    for _ in 0..10_000 {
        let x = rng.range(10, 13);
        assert!((10..13).contains(&x), "izvan raspona: {x}");
    }
}

#[test]
fn chance_bp_extremes() {
    let mut rng = Rng::from_seed(7);
    assert!((0..1_000).all(|_| !rng.chance_bp(0)));
    assert!((0..1_000).all(|_| rng.chance_bp(10_000)));
}

#[test]
#[should_panic(expected = "lo < hi")]
fn range_rejects_empty() {
    Rng::from_seed(1).range(5, 5);
}

#[test]
#[should_panic(expected = "bp <= 10000")]
fn chance_bp_rejects_over_100_percent() {
    Rng::from_seed(1).chance_bp(10_001);
}
```

- [ ] **Step 2: Pokreni — mora pasti**

Run: `cargo test -p engine; echo EXIT=$?`
Expected: greška prevođenja `unresolved imports engine::Rng, engine::label`, `EXIT=101`.

- [ ] **Step 3: Izvedba — `crates/engine/src/rng.rs`**

```rust
//! Kocka: SplitMix64 (Vigna), neovisni tokovi iz sjemena. Ne ovisi o vanjskoj biblioteci, jer `rand`
//! dopušta da nova manja verzija promijeni niz brojeva (istrazivanje/RUST_ALATI_2026-10.md).
//! Sva `wrapping_*` aritmetika ovdje je namjerna: dio je definicije algoritma.

const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn finalize(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mix64(x: u64) -> u64 {
    finalize(x.wrapping_add(GOLDEN_GAMMA))
}

/// FNV-1a 64 nad UTF-8 bajtovima; služi za stabilne oznake tokova (npr. `label("trigger:T1")`).
pub fn label(s: &str) -> u64 {
    s.bytes().fold(FNV_OFFSET, |h, b| (h ^ u64::from(b)).wrapping_mul(FNV_PRIME))
}

/// Ponovljiva kocka. Isto sjeme i iste oznake daju isti niz na svakom računalu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Neovisan tok izveden iz sjemena igre i oznaka, npr. `(runda, hotel, label("trigger:T1"))`.
    pub fn derive(seed: u64, labels: &[u64]) -> Self {
        let state = labels.iter().fold(mix64(seed), |s, l| mix64(s ^ l));
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        finalize(self.state)
    }

    /// Jednoliko iz `[lo, hi)`, bez pristranosti (odbacivanje ispod praga).
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(lo < hi, "range: potrebno lo < hi (lo={lo}, hi={hi})");
        let span = hi.checked_sub(lo).expect("lo < hi");
        let threshold = span.wrapping_neg().checked_rem(span).expect("span > 0");
        loop {
            let r = self.next_u64();
            if r >= threshold {
                let offset = r.checked_rem(span).expect("span > 0");
                return lo.checked_add(offset).expect("lo + offset < hi");
            }
        }
    }

    /// Vjerojatnost u baznim bodovima: 0 = nikad, 10_000 = uvijek.
    pub fn chance_bp(&mut self, bp: u32) -> bool {
        assert!(bp <= 10_000, "chance_bp: potrebno bp <= 10000 (bp={bp})");
        self.range(0, 10_000) < u64::from(bp)
    }
}
```

`crates/engine/src/lib.rs` (cijeli sadržaj):

```rust
//! Generička jezgra: kocka, a kasnije dnevnik i životni ciklus runde. Bez I/O-a, sata i vanjske slučajnosti.
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]

pub mod rng;

pub use rng::{Rng, label};
```

- [ ] **Step 4: Pokreni — mora proći**

Run: `cargo test -p engine; echo EXIT=$?`
Expected: `test result: ok. 10 passed; 0 failed`, `EXIT=0`.
Run: `cargo clippy -p engine --all-targets -- -D warnings; echo EXIT=$?` → Expected: `EXIT=0`. Ako clippy javi `arithmetic_side_effects` na `>>`, `^` ili `wrapping_*`: ti operatori ne smiju biti uzrok (nemaju prekoračenje); pročitaj točan redak i zamijeni samo stvarno neprovjerenu operaciju `checked_*` oblikom.

- [ ] **Step 5: Commit** — `git add crates/engine` · poruka: „Kostur 2/8: kocka SplitMix64 sa zlatnim vektorima"

---

