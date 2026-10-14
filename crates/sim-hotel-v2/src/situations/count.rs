//! Broj situacija u krugu (plan situacija §3; Leonove odluke 13.10.). Broj gleda samo **očekivanu popunjenost
//! kruga** (više gostiju → više posla), nikad stanje hotela; stanje mijenja koje situacije dolaze (`draw`).
//! Sidra S1–S4 i veća situacija ulaze u taj broj, ne dodaju se povrh.

use engine::calc::{self as c, Calc};

use crate::calendar::{Period, PeriodKind, avg_u16};
use crate::catalog::Catalog;
use crate::market::f_reputation_bp;
use crate::resolve::ResolveError;
use crate::state::HotelState;

const WEEK_DAYS: i128 = 7;

/// Očekivana popunjenost kruga (bp): osnovna popunjenost kataloga po danima kruga × faktor ugleda (ocjena na
/// početku kruga, ista formula kao tržište), najviše 100 %. Poznata je na početku kruga i ne ovisi o planu.
pub fn expected_occupancy_bp(
    state: &HotelState,
    p: &Period,
    cat: &Catalog,
) -> Result<u16, ResolveError> {
    let spec = cat.base.category(state.category)?;
    let base = i128::from(avg_u16(&spec.base_occupancy_bp, p)?);
    let f_rep = i128::from(f_reputation_bp(state.rating)?);
    let occ = c::div_round(c::mul(base, f_rep)?, 10_000)?.clamp(0, 10_000);
    Ok(c::narrow(occ)?)
}

/// Tjedni krug: < 75 % → 3, 75–89 % → 4, ≥ 90 % → 5 (najviše 5, nikad manje od 3).
pub fn weekly_count(occupancy_bp: u16) -> u8 {
    match occupancy_bp {
        0..7_500 => 3,
        7_500..9_000 => 4,
        _ => 5,
    }
}

/// Broj za krug: tjedni po pragovima; mjesečni razmjerno danima,
/// `round(tjedni broj × dani mjeseca / 7)` s polovicom prema gore (30 dana: 13 / 17 / 21; 31 dan: 13 / 18 / 22).
pub fn period_count(p: &Period, occupancy_bp: u16) -> Calc<u8> {
    let weekly = weekly_count(occupancy_bp);
    match p.kind {
        PeriodKind::Week => Ok(weekly),
        PeriodKind::Month => c::narrow(c::div_round(
            c::mul(i128::from(weekly), i128::from(p.days))?,
            WEEK_DAYS,
        )?),
    }
}

/// Koliko situacija hotel ima u krugu (sidra i veća situacija uključeni).
pub fn situation_count(state: &HotelState, p: &Period, cat: &Catalog) -> Result<u8, ResolveError> {
    Ok(period_count(p, expected_occupancy_bp(state, p, cat)?)?)
}

/// Virtualni tjedan kruga: od `offset_days` dana od početka kruga, s `count` situacija (sidra, kolektivna i veća
/// uključene u prvi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualWeek {
    pub offset_days: u8,
    pub count: u8,
}

/// Krug kao niz virtualnih tjedana, za razmak ponavljanja (Leon: isti predložak tek nakon 4 tjedna, i u
/// mjesečnoj igri). Tjedni krug je jedan dio s cijelim brojem; mjesečni su dijelovi od 7 dana od početka
/// mjeseca: svaki pun tjedan dobiva tjedni broj, zadnji dio ostatak do broja kruga (kolovoz 5+5+5+5+2).
pub fn virtual_weeks(p: &Period, occupancy_bp: u16) -> Calc<Vec<VirtualWeek>> {
    let n = period_count(p, occupancy_bp)?;
    if p.kind == PeriodKind::Week {
        return Ok(vec![VirtualWeek {
            offset_days: 0,
            count: n,
        }]);
    }
    let weekly = weekly_count(occupancy_bp);
    let mut out = Vec::new();
    let mut left = n;
    let mut offset: u8 = 0;
    while offset < p.days {
        let next = offset.saturating_add(7);
        let count = if next < p.days {
            weekly.min(left)
        } else {
            left
        };
        out.push(VirtualWeek {
            offset_days: offset,
            count,
        });
        left = left.saturating_sub(count);
        offset = next;
    }
    Ok(out)
}
