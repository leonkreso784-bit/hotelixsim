//! Broj situacija u krugu (plan situacija §3; Leonove odluke 13.10.). Broj gleda samo **očekivanu popunjenost
//! kruga** (više gostiju → više posla), nikad stanje hotela; stanje mijenja koje situacije dolaze (`draw`).
//! Sidra S1–S4 i veća situacija ulaze u taj broj, ne dodaju se povrh. Koliko je to po razini popunjenosti, postavka
//! je igre (`SituationCounts`); zadano tjedno 3/4/5, mjesečno 4/5/6 (ADR-042 t. 13).

use engine::calc as c;

use crate::calendar::{Period, PeriodKind, avg_u16};
use crate::catalog::Catalog;
use crate::market::f_reputation_bp;
use crate::resolve::ResolveError;
use crate::state::HotelState;

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

/// Broj situacija po krugu kao postavka igre (ADR-042 t. 13–14): `low` ispod 75 % očekivane popunjenosti, `mid` za
/// 75–89 %, `high` od 90 %. Uvijek `MIN ≤ low ≤ mid ≤ high ≤ MAX` (više gostiju → više posla, nikad manje); polja
/// su privatna, pa se invarijanta drži samo kroz `new`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SituationCounts {
    low: u8,
    mid: u8,
    high: u8,
}

impl SituationCounts {
    /// Najmanje: koliko je obveznih situacija (`game::MANDATORY_SITUATIONS`).
    pub const MIN: u8 = 3;
    pub const MAX: u8 = 8;
    /// Zadano za tjedni krug: 3 / 4 / 5.
    pub const WEEK: SituationCounts = SituationCounts {
        low: 3,
        mid: 4,
        high: 5,
    };
    /// Zadano za mjesečni krug: 4 / 5 / 6 (jedan krug, ne zbroj tjedana).
    pub const MONTH: SituationCounts = SituationCounts {
        low: 4,
        mid: 5,
        high: 6,
    };

    /// `None` kad broj izlazi iz `MIN..=MAX` ili pada s popunjenošću.
    pub fn new(low: u8, mid: u8, high: u8) -> Option<SituationCounts> {
        (Self::MIN <= low && low <= mid && mid <= high && high <= Self::MAX)
            .then_some(SituationCounts { low, mid, high })
    }

    /// Zadani broj za vrstu kruga.
    pub fn default_for(kind: PeriodKind) -> SituationCounts {
        match kind {
            PeriodKind::Week => Self::WEEK,
            PeriodKind::Month => Self::MONTH,
        }
    }

    /// `(low, mid, high)`.
    pub fn levels(self) -> (u8, u8, u8) {
        (self.low, self.mid, self.high)
    }

    /// Broj za očekivanu popunjenost: < 75 % → `low`, 75–89 % → `mid`, ≥ 90 % → `high`.
    pub fn for_occupancy(self, occupancy_bp: u16) -> u8 {
        match occupancy_bp {
            0..7_500 => self.low,
            7_500..9_000 => self.mid,
            _ => self.high,
        }
    }
}

/// Koliko situacija hotel ima u krugu uz postavku `c` (sidra i veća situacija uključeni).
pub fn situation_count_with(
    s: &HotelState,
    p: &Period,
    cat: &Catalog,
    c: SituationCounts,
) -> Result<u8, ResolveError> {
    Ok(c.for_occupancy(expected_occupancy_bp(s, p, cat)?))
}

/// Koliko situacija hotel ima u krugu uz zadani broj za vrstu kruga (`SituationCounts::default_for`).
pub fn situation_count(state: &HotelState, p: &Period, cat: &Catalog) -> Result<u8, ResolveError> {
    situation_count_with(state, p, cat, SituationCounts::default_for(p.kind))
}
