//! Tokovi kocke po svrsi (MODEL_V2 §6.2): svaka svrha ima svoj niz brojeva izveden iz sjemena igre, kruga,
//! utora i ključa, pa novo bacanje nikad ne pomiče postojeća. Oznake se nikad ne mijenjaju (promijenile bi
//! brojke budućih igara). U sloju 1 nijedna formula obračuna ne zove `stream`; tok situacija koristi samo
//! izvlačenje situacija iz predložaka (`situations::draw`).

use engine::{Rng, label};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Purpose {
    Departures,
    Breakdowns,
    Reviews,
    LabourMarket,
    /// Izvlačenje situacija iz predložaka i popunjavanje uloga i mjesta (J-2).
    Situations,
}

impl Purpose {
    pub const ALL: [Purpose; 5] = [
        Purpose::Departures,
        Purpose::Breakdowns,
        Purpose::Reviews,
        Purpose::LabourMarket,
        Purpose::Situations,
    ];

    /// Stabilna oznaka toka; nikad se ne mijenja.
    pub fn label(self) -> &'static str {
        match self {
            Purpose::Departures => "v2:odlasci",
            Purpose::Breakdowns => "v2:kvarovi",
            Purpose::Reviews => "v2:recenzije",
            Purpose::LabourMarket => "v2:burza-rada",
            Purpose::Situations => "v2:situacije",
        }
    }
}

/// Tok za jednu svrhu u jednom krugu; `slot` i `key` razlikuju bacanja unutar kruga (npr. odjel, osoba).
pub fn stream(seed: u64, purpose: Purpose, period_index: u8, slot: u8, key: u64) -> Rng {
    Rng::derive(
        seed,
        &[
            label(purpose.label()),
            u64::from(period_index),
            u64::from(slot),
            key,
        ],
    )
}
