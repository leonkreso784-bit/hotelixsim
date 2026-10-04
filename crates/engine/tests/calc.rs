use engine::calc::{self, CalcError};

#[test]
fn div_round_is_half_away_from_zero() {
    let cases: [(i128, i128, i128); 10] = [
        (1, 2, 1),
        (-1, 2, -1),
        (3, 2, 2),
        (-3, 2, -2),
        (5, 3, 2),
        (4, 3, 1),
        (-4, 3, -1),
        (-5, 3, -2),
        (0, 7, 0),
        (-2_000, 400, -5),
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
