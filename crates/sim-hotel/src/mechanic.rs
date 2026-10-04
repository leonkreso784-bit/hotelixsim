//! Mehanike situacija (DEMO §6.2): samo deklaracije tipova; logiku dodaje Task 6.

use engine::calc::CalcError;

use crate::month::Month;
use crate::state::GroupBlock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mechanic {
    ServiceRecovery {
        compensation_cents: i64,
    },
    EquipmentRepair {
        urgent: bool,
    },
    StaffRetention {
        raise_one_pct: u16,
        bonus_cents: i64,
    },
    GroupBooking {
        accept: bool,
        rooms: u16,
        nights: u16,
        discount_bp: u16,
        month: Month,
    },
    CrisisCommunication {
        backup_cents: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationOutcome {
    pub code: String,
    pub mechanics: Vec<Mechanic>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedAnswer {
    pub label: String,
    pub mechanics: Vec<Mechanic>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationTemplate {
    pub code: String,
    pub month: Month,
    pub title: String,
    pub answers: Vec<OfferedAnswer>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub satisfaction_pts: i32,
    pub morale_pts: i32,
    pub rating: i32,
    pub equipment_pts: i32,
    pub cost_cents: i64,
    pub extra_wage_cents: i64,
    pub groups: Vec<GroupBlock>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanicError {
    BadQuality(u8),
    GroupInPast,
    BadGroup,
    Calc(CalcError),
    /// Privremeno (Task 5): situacije još nisu podržane. Task 6 briše varijantu i granu.
    NotYetSupported,
}

impl From<CalcError> for MechanicError {
    fn from(e: CalcError) -> Self {
        MechanicError::Calc(e)
    }
}
