//! Sezona: lipanj–rujan (DEMO §2). Mjesec je ključ svih tablica.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Month {
    Jun,
    Jul,
    Aug,
    Sep,
}

impl Month {
    pub const SEASON: [Month; 4] = [Month::Jun, Month::Jul, Month::Aug, Month::Sep];

    pub fn days(self) -> u8 {
        match self {
            Month::Jun | Month::Sep => 30,
            Month::Jul | Month::Aug => 31,
        }
    }

    /// Stabilan kod (JSON, API); nikad se ne mijenja.
    pub fn code(self) -> &'static str {
        match self {
            Month::Jun => "jun",
            Month::Jul => "jul",
            Month::Aug => "aug",
            Month::Sep => "sep",
        }
    }

    pub fn from_code(s: &str) -> Option<Month> {
        match s {
            "jun" => Some(Month::Jun),
            "jul" => Some(Month::Jul),
            "aug" => Some(Month::Aug),
            "sep" => Some(Month::Sep),
            _ => None,
        }
    }

    pub fn position(self) -> usize {
        match self {
            Month::Jun => 0,
            Month::Jul => 1,
            Month::Aug => 2,
            Month::Sep => 3,
        }
    }
}

/// Jedna vrijednost po mjesecu sezone, redoslijedom `Month::SEASON`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthTable<T>(pub [T; 4]);

impl<T: Copy> MonthTable<T> {
    pub fn get(&self, m: Month) -> T {
        self.0[m.position()]
    }
}
