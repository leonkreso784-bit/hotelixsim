//! Situacije iz predložaka (plan situacija §3–§4): koliko ih krug ima (`count`) i koje se izvlače (`draw`).
//! Jezgra ne zna za tekstove predložaka (oni su u sadržaju); ovdje su pravila pojave, broj i popunjeni primjerci.

use std::collections::BTreeMap;

use sim_hotel::SituationOutcome;

use crate::calendar::Period;
use crate::department::DeptCode;

mod count;
mod draw;
mod eligible;
mod fill;
mod template;

pub use count::{SituationCounts, expected_occupancy_bp, situation_count, situation_count_with};
pub use draw::{Draw, draw_collective, draw_situations, draw_situations_with};
pub use eligible::DrawView;
pub use template::{
    Boost, Bounds, Chance, Conditions, DeptTarget, Gender, Kind, MajorRule, MajorSide, NameList,
    NamePool, NumberParam, Repeat, RoleSource, RoleSpec, Scope, SituationSpec, TemplateSet,
};

/// Sidro kruga (S1–S4): zauzima mjesto u broju situacija i broji se u granicu odjela i vrste kruga.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub kind: Kind,
    /// Odjel sidra; `None` za cijeli hotel.
    pub department: Option<DeptCode>,
}

/// Primjerak: predložak složen za jedan hotel u jednom krugu. Tekst iz sadržaja s ovim ulogama i brojevima daje
/// konkretnu situaciju; mjesta s iznosom i tekstom po odjelu ne traže kocku pa ih ovdje nema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawnSituation {
    /// Šifra primjerka, jedinstvena u igri: predložak i krug (`P-KLIMA.t03`). Isti predložak dolazi najviše jednom
    /// u krugu; redni broj u šifri (`P-KLIMA.aug.2`) imaju samo primjerci iz starih dnevnika.
    pub code: String,
    pub template: String,
    pub period: Period,
    /// Dan sezone primjerka (0 = 1. lipnja) za razmak ponavljanja: početak kruga (i u mjesečnom krugu).
    pub day: u16,
    /// Redni broj primjerka istog predloška u krugu (1 = prvi; novo izvlačenje uvijek 1).
    pub occurrence: u8,
    /// Utor hotela; 0 za kolektivnu (ista za sve hotele).
    pub slot: u8,
    pub kind: Kind,
    pub weight: u8,
    pub scope: Scope,
    pub major: Option<MajorSide>,
    /// Odjel situacije; `None` za cijeli hotel.
    pub department: Option<DeptCode>,
    pub roles: BTreeMap<String, RoleFill>,
    /// Brojevna mjesta (`range`, `pick`).
    pub numbers: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleFill {
    pub name: String,
    /// Opis lika iz `types` predloška; `None` kad ga predložak nema.
    pub role_type: Option<String>,
}

/// Kvaliteta bez učinka: bodovi svih mehanika su `(kvaliteta − 3) × k` uz `k ≥ 0`.
const NEUTRAL_QUALITY: u8 = 3;

/// Ishod situacije „samo nagrada" (S-4, Leon 12.10.: pohvala je čista prilika): kvaliteta 1–2 računa se kao
/// neutralna 3. Bodovi mehanika su `(q − 3) × k` s `k ≥ 0`, pa je to isto što i donja granica 0 za svaki pokazatelj
/// (zadovoljstvo, moral, ocjena, oprema); trošak odluke (nagrada, kompenzacija) ostaje. Vrijedi za ponuđen,
/// slobodan i neodgovoren ishod; kvaliteta izvan 1–5 ostaje kakva jest, pa je obračun i dalje odbija.
pub fn reward_only(mut o: SituationOutcome) -> SituationOutcome {
    if (1..NEUTRAL_QUALITY).contains(&o.quality) {
        o.quality = NEUTRAL_QUALITY;
    }
    o
}

/// Ishod situacije u obračunu v2 (ADR-042 t. 1): v1 ishod i odjel čiji prostor popravak mijenja;
/// `None` (sidra, cijeli hotel) i neraspoređeni odjel = Smještaj; nepoznat odjel = greška kataloga.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundSituation {
    pub outcome: SituationOutcome,
    pub space: Option<DeptCode>,
}

impl From<SituationOutcome> for RoundSituation {
    /// Ishod bez odjela (sidro, cijeli hotel): učinak na prostor ide u Smještaj.
    fn from(outcome: SituationOutcome) -> Self {
        RoundSituation {
            outcome,
            space: None,
        }
    }
}
