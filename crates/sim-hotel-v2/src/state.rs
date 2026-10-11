//! Stanje hotela na početku kruga: hotel od odjela (MODEL_V2 §3.8). Osoblje i stanje prostora žive u odjelima,
//! stalni troškovi u neraspoređenim odjelima i fiksnim naknadama; `summary()` iz njih izvodi v1 oblik za indekse,
//! KPI-je i v1 sažetak izvješća.

use std::collections::BTreeMap;

use engine::calc as c;
use sim_hotel::{CatalogError, Category, GroupBlock, Pending, Profile, turnover_bp};

use crate::catalog::Catalog;
use crate::department::{DeptCode, DeptState, UsaliClass};
use crate::resolve::ResolveError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotelState {
    pub category: Category,
    pub rooms: u16,
    pub wage_pct: u16,
    pub extra_wage_cents: i64,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub load_hundredths: u16,
    pub turnover_bp: u16,
    pub renovated: bool,
    pub pending: Pending,
    pub groups: Vec<GroupBlock>,
    pub departments: BTreeMap<DeptCode, DeptState>,
    /// Mjesečni stalni dio neraspoređenih odjela (iz profila) i mjesečne fiksne naknade.
    pub undistributed_fixed_cents: BTreeMap<DeptCode, i64>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
}

/// Operativni odjeli osim Smještaja (hrana i piće, ostali operativni) ulaze u v1 „opremu" sažetka.
fn is_operated(class: UsaliClass) -> bool {
    matches!(class, UsaliClass::FoodBeverage | UsaliClass::OtherOperated)
}

impl HotelState {
    /// Početno stanje profila: v1 polja iz `base`, odjeli i stalni troškovi iz profila v2.
    pub fn from_profile(cat: &Catalog, p: Profile) -> Result<HotelState, ResolveError> {
        let spec = cat.base.profile(p)?;
        let pv2 = cat.profile_v2(p)?;
        let mut departments = BTreeMap::new();
        let mut undistributed_fixed_cents = BTreeMap::new();
        for (code, start) in &pv2.departments {
            let d = cat.dept(code)?;
            departments.insert(
                code.clone(),
                DeptState {
                    class: d.class,
                    open: start.open,
                    permanent: start.permanent,
                    seasonal: start.seasonal,
                    condition_pts: start.condition_pts,
                    satisfaction_pts: start.satisfaction_pts,
                    season_departures: 0,
                },
            );
            if d.class.is_undistributed() {
                undistributed_fixed_cents.insert(
                    code.clone(),
                    start.fixed_monthly_cents.unwrap_or(d.fixed_monthly_cents),
                );
            }
        }
        Ok(HotelState {
            category: spec.category,
            rooms: spec.rooms,
            wage_pct: spec.wage_pct,
            extra_wage_cents: 0,
            morale_pts: spec.morale_pts,
            experience_pts: spec.experience_pts,
            satisfaction_pts: spec.satisfaction_pts,
            rating: spec.rating,
            cash_cents: spec.cash_cents,
            load_hundredths: spec.load_hundredths,
            turnover_bp: turnover_bp(spec.morale_pts)?,
            renovated: false,
            pending: Pending::default(),
            groups: Vec::new(),
            departments,
            undistributed_fixed_cents,
            fixed_charges_cents: pv2.fixed_charges_cents.clone(),
        })
    }

    /// Σ stalni + sezonci svih odjela. Granice drži plan (najviše `seasonal_max` sezonaca iz kataloga), zato je
    /// zbroj bez prelijevanja.
    pub fn staff(&self) -> u16 {
        self.departments.values().fold(0_u16, |a, d| {
            a.saturating_add(d.permanent).saturating_add(d.seasonal)
        })
    }

    /// Odjel Smještaj (katalog ga uvijek ima, `Catalog::validate`).
    pub fn rooms_state(&self) -> Result<&DeptState, ResolveError> {
        self.departments
            .get(&DeptCode::rooms())
            .ok_or_else(|| CatalogError::Missing("state.departments.rooms".to_string()).into())
    }

    /// v1 „oprema": stanje Smještaja ako nema drugih operativnih odjela, inače 70 % Smještaj i 30 % prosjek ostalih
    /// operativnih (hrana i piće, ostali operativni).
    fn equipment_summary(&self) -> i32 {
        let rooms = self
            .departments
            .get(&DeptCode::rooms())
            .map_or(0_i32, |d| d.condition_pts);
        let others: Vec<i128> = self
            .departments
            .iter()
            .filter(|(k, d)| **k != DeptCode::rooms() && is_operated(d.class))
            .map(|(_, d)| i128::from(d.condition_pts))
            .collect();
        if others.is_empty() {
            return rooms;
        }
        // Stanja su u 0..=10 000 (`Catalog::validate`, obračun ih stišće), pa ovaj račun ne može pasti; zamjena
        // (stanje Smještaja) je samo da potpis ostane bez `Result`.
        let mixed = i128::try_from(others.len())
            .map_err(|_| c::CalcError::Overflow)
            .and_then(|n| c::div_round(c::sum(&others)?, n))
            .and_then(|avg| {
                c::div_round(
                    c::add(c::mul(7_000, i128::from(rooms))?, c::mul(3_000, avg)?)?,
                    10_000,
                )
            })
            .and_then(c::narrow);
        mixed.unwrap_or(rooms)
    }

    /// v1 oblik stanja, polje po polje (novo polje v2 mora ovdje svjesno odlučiti gdje ide): stalni i sezonci su
    /// zbrojevi odjela, oprema je stanje prostora (`equipment_summary`), stalni trošak je Σ neraspoređeni + Σ
    /// fiksne naknade (mjesečno).
    pub fn summary(&self) -> sim_hotel::HotelState {
        let (permanent, seasonal) = self.departments.values().fold((0_u16, 0_u16), |(p, s), d| {
            (p.saturating_add(d.permanent), s.saturating_add(d.seasonal))
        });
        let fixed = self
            .undistributed_fixed_cents
            .values()
            .chain(self.fixed_charges_cents.values())
            .fold(0_i64, |a, v| a.saturating_add(*v));
        sim_hotel::HotelState {
            category: self.category,
            rooms: self.rooms,
            permanent,
            seasonal,
            wage_pct: self.wage_pct,
            extra_wage_cents: self.extra_wage_cents,
            morale_pts: self.morale_pts,
            experience_pts: self.experience_pts,
            equipment_pts: self.equipment_summary(),
            satisfaction_pts: self.satisfaction_pts,
            rating: self.rating,
            cash_cents: self.cash_cents,
            fixed_cents: fixed,
            load_hundredths: self.load_hundredths,
            turnover_bp: self.turnover_bp,
            renovated: self.renovated,
            pending: self.pending.clone(),
            groups: self.groups.clone(),
        }
    }
}
