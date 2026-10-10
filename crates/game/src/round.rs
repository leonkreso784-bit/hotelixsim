//! Ključ kruga (MODEL_V2 §4.4): `Month(m)` → "jun"…; `Week(n)`, n = 1..=17 → "t01"…"t17".
//! Serde je kod (niz), isti kao stari kod mjeseca — v1 događaji su bajt-isti.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sim_hotel::Month;

use crate::error::GameError;

/// Najveći redni broj tjedna u sezoni.
pub const MAX_WEEK: u8 = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoundKey {
    Month(Month),
    Week(u8),
}

impl RoundKey {
    pub fn code(self) -> String {
        match self {
            RoundKey::Month(m) => m.code().to_owned(),
            RoundKey::Week(n) => format!("t{n:02}"),
        }
    }

    /// "jun".."sep" i "t01".."t17"; ništa drugo.
    pub fn parse(s: &str) -> Option<RoundKey> {
        if let Some(m) = Month::from_code(s) {
            return Some(RoundKey::Month(m));
        }
        let digits = s.strip_prefix('t')?;
        if digits.len() != 2 || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let n: u8 = digits.parse().ok()?;
        (1..=MAX_WEEK).contains(&n).then_some(RoundKey::Week(n))
    }

    /// Kalendarski mjesec mjesečnog kruga; za tjedan `None`.
    pub fn month(self) -> Option<Month> {
        match self {
            RoundKey::Month(m) => Some(m),
            RoundKey::Week(_) => None,
        }
    }

    /// Mjesec za jezgru v1; tjedan v1 igra ne poznaje.
    pub(crate) fn require_month(self) -> Result<Month, GameError> {
        self.month().ok_or(GameError::UnknownMonth)
    }
}

impl From<Month> for RoundKey {
    fn from(m: Month) -> Self {
        RoundKey::Month(m)
    }
}

impl Serialize for RoundKey {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.code())
    }
}

impl<'de> Deserialize<'de> for RoundKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        RoundKey::parse(&s).ok_or_else(|| de::Error::custom(format!("nepoznat krug {s:?}")))
    }
}
