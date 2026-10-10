//! Stanje hotela na početku kruga (kopija v1 `HotelState`; Task 11 dodaje odjele). `summary()` daje v1 oblik
//! za indekse, KPI-je i v1 sažetak izvješća.

use engine::calc::Calc;
use sim_hotel::{Category, GroupBlock, Pending, ProfileSpec, turnover_bp};

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

    /// Granice drži plan (najviše `seasonal_max` sezonaca iz kataloga), zato je zbroj bez prelijevanja.
    pub fn staff(&self) -> u16 {
        self.permanent.saturating_add(self.seasonal)
    }

    /// v1 oblik stanja, polje po polje (novo polje v2 mora ovdje svjesno odlučiti gdje ide).
    pub fn summary(&self) -> sim_hotel::HotelState {
        sim_hotel::HotelState {
            category: self.category,
            rooms: self.rooms,
            permanent: self.permanent,
            seasonal: self.seasonal,
            wage_pct: self.wage_pct,
            extra_wage_cents: self.extra_wage_cents,
            morale_pts: self.morale_pts,
            experience_pts: self.experience_pts,
            equipment_pts: self.equipment_pts,
            satisfaction_pts: self.satisfaction_pts,
            rating: self.rating,
            cash_cents: self.cash_cents,
            fixed_cents: self.fixed_cents,
            load_hundredths: self.load_hundredths,
            turnover_bp: self.turnover_bp,
            renovated: self.renovated,
            pending: self.pending.clone(),
            groups: self.groups.clone(),
        }
    }
}
