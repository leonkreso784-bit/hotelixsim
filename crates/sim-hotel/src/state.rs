//! Stanje hotela na početku mjeseca (DEMO §2.2) i izvedeni odlazak osoblja (P21).

use engine::calc::{self, Calc};

use crate::catalog::{Category, ProfileSpec};
use crate::month::Month;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupBlock {
    pub month: Month,
    pub room_nights: u32,
    pub price_cents: i64,
}

/// Učinci odluka iz prošlog mjeseca koji stižu tek u sljedećem.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Pending {
    pub experience_pts: i32,
    pub morale_pts: i32,
    pub renovation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotelState {
    pub category: Category,
    pub rooms: u16,
    pub permanent: u16,
    pub seasonal: u16,
    pub wage_pct: u16,
    pub extra_wage_cents: i64,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub equipment_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub fixed_cents: i64,
    pub load_hundredths: u16,
    pub turnover_bp: u16,
    pub renovated: bool,
    pub pending: Pending,
    pub groups: Vec<GroupBlock>,
}

impl HotelState {
    pub fn from_profile(spec: &ProfileSpec) -> Calc<HotelState> {
        Ok(HotelState {
            category: spec.category,
            rooms: spec.rooms,
            permanent: spec.permanent,
            seasonal: spec.seasonal,
            wage_pct: spec.wage_pct,
            extra_wage_cents: 0,
            morale_pts: spec.morale_pts,
            experience_pts: spec.experience_pts,
            equipment_pts: spec.equipment_pts,
            satisfaction_pts: spec.satisfaction_pts,
            rating: spec.rating,
            cash_cents: spec.cash_cents,
            fixed_cents: spec.fixed_cents,
            load_hundredths: spec.load_hundredths,
            turnover_bp: turnover_bp(spec.morale_pts)?,
            renovated: false,
            pending: Pending::default(),
            groups: Vec::new(),
        })
    }

    /// Granice drži plan (najviše 40 sezonaca), zato je zbroj bez prelijevanja.
    pub fn staff(&self) -> u16 {
        self.permanent.saturating_add(self.seasonal)
    }
}

/// Odlazak u bazama: 300 + 15·max(0, 6000 − moral) / 100, zaokruženo (P21).
pub fn turnover_bp(morale_pts: i32) -> Calc<u16> {
    let gap = calc::sub(6_000, i128::from(morale_pts))?.max(0);
    let extra = calc::div_round(calc::mul(15, gap)?, 100)?;
    calc::narrow(calc::add(300, extra)?)
}
