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
    let q = add(magnitude, b)?
        .checked_div(two_b)
        .ok_or(CalcError::Overflow)?;
    if a >= 0 {
        Ok(q)
    } else {
        q.checked_neg().ok_or(CalcError::Overflow)
    }
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
