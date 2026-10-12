//! Situacije iz predložaka (plan situacija §3–§4): koliko ih krug ima (`count`) i koje se izvlače (`draw`).
//! Jezgra ne zna za tekstove predložaka (oni su u sadržaju); ovdje su pravila pojave, broj i popunjeni primjerci.

use std::collections::BTreeMap;

use crate::calendar::Period;
use crate::department::DeptCode;

mod count;
mod draw;
mod eligible;
mod fill;
mod template;

pub use count::{expected_occupancy_bp, period_count, situation_count, weekly_count};
pub use draw::{Draw, draw_situations};
pub use eligible::DrawView;
pub use template::{
    Boost, Bounds, Chance, Conditions, DeptTarget, Gender, Kind, MajorRule, MajorSide, NameList,
    NamePool, NumberParam, Repeat, RoleSource, RoleSpec, Scope, SituationSpec, TemplateSet,
};

/// Primjerak: predložak složen za jedan hotel u jednom krugu. Tekst iz sadržaja s ovim ulogama i brojevima daje
/// konkretnu situaciju; mjesta s iznosom i tekstom po odjelu ne traže kocku pa ih ovdje nema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawnSituation {
    /// Šifra primjerka, jedinstvena u igri: predložak i krug (`P-KLIMA.t03`).
    pub code: String,
    pub template: String,
    pub period: Period,
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
