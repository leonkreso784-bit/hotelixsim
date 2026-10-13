### Task 2: `engine::calc` — provjereni cjelobrojni račun

**Files:**
- Create: `crates/engine/src/calc.rs`, `crates/engine/tests/calc.rs`
- Modify: `crates/engine/src/lib.rs` (`pub mod calc;`), `crates/engine/src/rng.rs` (doc za `derive(seed, &[])`)
- Modify: `crates/engine/tests/rng_golden.rs` (BACKLOG: neovisnost tokova, grana odbacivanja)

**Interfaces:**
- Produces (svi u `engine::calc`):
  `enum CalcError { Overflow, BadDivisor }` · `type Calc<T> = Result<T, CalcError>` ·
  `fn add(a: i128, b: i128) -> Calc<i128>` · `fn sub(..)` · `fn mul(..)` · `fn sum(xs: &[i128]) -> Calc<i128>` ·
  `fn div_round(a: i128, b: i128) -> Calc<i128>` · `fn div_floor(a: i128, b: i128) -> Calc<i128>` ·
  `fn mul_div(a: i128, b: i128, c: i128) -> Calc<i128>` · `fn narrow<T: TryFrom<i128>>(x: i128) -> Calc<T>`

- [ ] **Step 1: Testovi** `crates/engine/tests/calc.rs`:

```rust
use engine::calc::{self, CalcError};

#[test]
fn div_round_is_half_away_from_zero() {
    let cases: [(i128, i128, i128); 10] = [
        (1, 2, 1), (-1, 2, -1), (3, 2, 2), (-3, 2, -2), (5, 3, 2),
        (4, 3, 1), (-4, 3, -1), (-5, 3, -2), (0, 7, 0), (-2_000, 400, -5),
    ];
    for (a, b, want) in cases {
        assert_eq!(calc::div_round(a, b), Ok(want), "{a}/{b}");
    }
}

#[test]
fn div_round_rejects_zero_and_negative_divisor() {
    assert_eq!(calc::div_round(7, 0), Err(CalcError::BadDivisor));
    assert_eq!(calc::div_round(7, -1), Err(CalcError::BadDivisor));
}

#[test]
fn div_round_reports_overflow_instead_of_wrapping() {
    assert_eq!(calc::div_round(i128::MAX, 3), Err(CalcError::Overflow));
    assert_eq!(calc::div_round(i128::MIN, 3), Err(CalcError::Overflow));
}

#[test]
fn div_floor_rounds_down_for_staff_capacity() {
    assert_eq!(calc::div_floor(50 * 30 * 24, 10), Ok(3_600));
    assert_eq!(calc::div_floor(19, 10), Ok(1));
    assert_eq!(calc::div_floor(5, 0), Err(CalcError::BadDivisor));
}

#[test]
fn mul_div_rounds_once() {
    // 0,3 × 34 992 000 centi = 10 497 600 (ostali prihod, lipanj profila C)
    assert_eq!(calc::mul_div(34_992_000, 3_000, 10_000), Ok(10_497_600));
    assert_eq!(calc::mul_div(i128::MAX, 2, 1), Err(CalcError::Overflow));
}

#[test]
fn checked_ops_and_sum() {
    assert_eq!(calc::add(i128::MAX, 1), Err(CalcError::Overflow));
    assert_eq!(calc::sub(i128::MIN, 1), Err(CalcError::Overflow));
    assert_eq!(calc::mul(i128::MAX, 2), Err(CalcError::Overflow));
    assert_eq!(calc::sum(&[1, 2, 3, -10]), Ok(-4));
}

#[test]
fn narrow_fails_when_value_does_not_fit() {
    assert_eq!(calc::narrow::<u16>(65_535), Ok(65_535_u16));
    assert_eq!(calc::narrow::<u16>(65_536), Err(CalcError::Overflow));
    assert_eq!(calc::narrow::<u32>(-1), Err(CalcError::Overflow));
}
```

- [ ] **Step 2: Pokreni** `cargo test -p engine --test calc; echo EXIT=$?` → ne prevodi se (`calc` ne postoji).

- [ ] **Step 3: Implementacija** `crates/engine/src/calc.rs`:

```rust
//! Cjelobrojni račun jezgre (DEMO §12, P1–P5): sve kroz `i128`, provjerene operacije, jedno zaokruživanje.
//! Greška se vraća, nikad se ne „prevrne" niti tiho postane 0.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcError {
    /// Rezultat ne stane u `i128` ili u ciljni tip (`narrow`).
    Overflow,
    /// Nazivnik je 0 ili negativan; u jezgri je to uvijek greška kataloga ili koda (P5).
    BadDivisor,
}

pub type Calc<T> = Result<T, CalcError>;

pub fn add(a: i128, b: i128) -> Calc<i128> {
    a.checked_add(b).ok_or(CalcError::Overflow)
}

pub fn sub(a: i128, b: i128) -> Calc<i128> {
    a.checked_sub(b).ok_or(CalcError::Overflow)
}

pub fn mul(a: i128, b: i128) -> Calc<i128> {
    a.checked_mul(b).ok_or(CalcError::Overflow)
}

pub fn sum(xs: &[i128]) -> Calc<i128> {
    xs.iter().try_fold(0_i128, |acc, &x| add(acc, x))
}

/// `a / b` zaokruženo polovicom od nule (P1): `(2a + b) / 2b` za `a ≥ 0`, simetrično za `a < 0`.
pub fn div_round(a: i128, b: i128) -> Calc<i128> {
    if b <= 0 {
        return Err(CalcError::BadDivisor);
    }
    let two_b = mul(2, b)?;
    let magnitude = mul(2, a)?.checked_abs().ok_or(CalcError::Overflow)?;
    let q = add(magnitude, b)?.checked_div(two_b).ok_or(CalcError::Overflow)?;
    if a >= 0 { Ok(q) } else { q.checked_neg().ok_or(CalcError::Overflow) }
}

/// `a / b` zaokruženo prema dolje; samo za kapacitet osoblja (P14), gdje se tvrda granica ne smije prekoračiti.
pub fn div_floor(a: i128, b: i128) -> Calc<i128> {
    if b <= 0 {
        return Err(CalcError::BadDivisor);
    }
    a.checked_div_euclid(b).ok_or(CalcError::Overflow)
}

/// `a × b / c` s jednim zaokruživanjem (P2).
pub fn mul_div(a: i128, b: i128, c: i128) -> Calc<i128> {
    div_round(mul(a, b)?, c)
}

/// Spremanje u uži tip; vrijednost koja ne stane je greška, ne rezanje.
pub fn narrow<T: TryFrom<i128>>(x: i128) -> Calc<T> {
    T::try_from(x).map_err(|_| CalcError::Overflow)
}
```
U `lib.rs`: `pub mod calc;` i komentar modula „Generička jezgra: kocka i cjelobrojni račun…".

- [ ] **Step 4: Pokreni** `cargo test -p engine; echo EXIT=$?` → svi prolaze; `cargo clippy -p engine --all-targets -- -D warnings; echo EXIT=$?` → 0.

- [ ] **Step 5: Kocka (BACKLOG [jezgra], tri stavke).** U `rng_golden.rs` dodaj:

```rust
#[test]
fn derived_streams_differ_by_label_order_and_seed() {
    let t1 = label("trigger:T1");
    let t2 = label("trigger:T2");
    let first = |mut r: Rng| r.next_u64();
    assert_ne!(first(Rng::derive(7, &[1, t1])), first(Rng::derive(7, &[1, t2])));
    assert_ne!(first(Rng::derive(7, &[1, t1])), first(Rng::derive(7, &[t1, 1])));
    assert_ne!(first(Rng::derive(7, &[1, t1])), first(Rng::derive(8, &[1, t1])));
}

#[test]
fn range_rejection_branch_keeps_values_in_bounds() {
    // raspon blizu 2^63 odbacuje gotovo polovicu izvlačenja, pa se grana odbacivanja stvarno izvršava
    let mut rng = Rng::from_seed(42);
    let hi = (1_u64 << 63) + 1;
    for _ in 0..1_000 {
        let v = rng.range(0, hi);
        assert!(v < hi);
    }
}
```
U `rng.rs` uz `derive` dodaj doc: „`derive(seed, &[])` NIJE `from_seed(seed)`: izvedeni tok počinje od `mix64(seed)`,
pa su prostori stanja različiti — za tokove igre uvijek koristi `derive`."

- [ ] **Step 6: Pokreni** `cargo test -p engine; echo EXIT=$?` → prolazi.

- [ ] **Step 7: Commit** — „Jezgra 2/12: engine::calc (polovica od nule, provjerene operacije) i testovi kocke".

---

