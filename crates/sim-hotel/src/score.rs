//! Relativna ocjena (DEMO §8, P31): napredak se mjeri prema prostoru za rast, ne prema zaradi.

use engine::calc::{self as c, Calc};

use crate::index::{Index, IndexSet};

/// Dobitak u bazama: rast se mjeri prema preostalom prostoru do 100, pad prema početnoj vrijednosti.
pub fn gain_bp(start: u16, end: u16) -> Calc<i32> {
    let (s, e) = (i128::from(start), i128::from(end));
    let delta = c::mul(10_000, c::sub(e, s)?)?;
    let g = if e >= s {
        if s == 10_000 {
            0
        } else {
            c::div_round(delta, c::sub(10_000, s)?)?
        }
    } else {
        c::div_round(delta, s)?
    };
    c::narrow(g)
}

/// Prosjek dobitaka pet indeksa, u bazama.
pub fn progress_pts(start: &IndexSet, end: &IndexSet) -> Calc<i32> {
    let mut total: i128 = 0;
    for i in Index::ALL {
        let g = gain_bp(start.get(i).hundredths(), end.get(i).hundredths())?;
        total = c::add(total, i128::from(g))?;
    }
    c::narrow(c::div_round(total, 5)?)
}
