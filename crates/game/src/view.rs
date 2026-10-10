//! Pogledi na hotel (ARHITEKTURA §12): javni (tuđi hotel: ocjena i pet indeksa) i privatni (vlasnik: stvarne brojke).

use crate::command::{PlanLimits, plan_limits};
use crate::dto::{PlanDto, indices_dto};
use crate::error::{GameError, corrupt};
use crate::event::Controller;
use crate::game::{Game, RoundStatus};
use crate::ids::HotelId;
use crate::kpi::report_kpi;
use crate::round::RoundKey;

/// Tko vodi hotel; namjerno bez `guest_id`, tuđi gost se ne otkriva.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seat {
    Human,
    Robot { strategy: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthScore {
    pub round: RoundKey,
    pub indices: [u16; 5],
    pub progress_hundredths: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicHotel {
    pub hotel_id: HotelId,
    pub slot: u8,
    pub name: String,
    pub seat: Seat,
    pub start: [u16; 5],
    pub months: Vec<MonthScore>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealNumbers {
    pub rooms: u16,
    pub permanent: u16,
    pub seasonal: u16,
    pub wage_pct: u16,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub equipment_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub load_hundredths: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastMonth {
    pub round: RoundKey,
    pub sold: u32,
    pub available: u32,
    pub rooms_revenue_cents: i64,
    pub gop_cents: i64,
    pub goppar_cents: i64,
    /// Mentorov osvrt na taj mjesec (kad ga ima).
    pub mentor: Option<String>,
    /// Sloj 0: isti brojevi kao `/kpi` za taj mjesec (prodane noći ÷ sve sobe × dani; bp).
    pub occupancy_bp: Option<i64>,
    pub adr_cents: Option<i64>,
    pub revpar_cents: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanState {
    pub round: RoundKey,
    pub plan: PlanDto,
    pub limits: PlanLimits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateHotel {
    pub profile: String,
    pub now: RealNumbers,
    pub last_month: Option<LastMonth>,
    /// `Some` samo dok je trenutni mjesec otvoren.
    pub plan: Option<PlanState>,
}

impl Game {
    pub fn public_hotel(&self, id: HotelId) -> Result<PublicHotel, GameError> {
        let h = self.hotel(id)?;
        let seat = match &h.setup.controller {
            Controller::Human { .. } => Seat::Human,
            Controller::Robot { strategy } => Seat::Robot {
                strategy: strategy.clone(),
            },
        };
        let mut months = Vec::new();
        for m in &self.rounds {
            let Some(rr) = self.resolved.get(m) else {
                continue;
            };
            let res = rr
                .hotels
                .iter()
                .find(|x| x.hotel_id == id)
                .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
            months.push(MonthScore {
                round: rr.month,
                indices: res.indices,
                progress_hundredths: res.progress_hundredths,
            });
        }
        Ok(PublicHotel {
            hotel_id: id,
            slot: h.setup.slot,
            name: h.setup.name.clone(),
            seat,
            start: indices_dto(&h.start_indices),
            months,
        })
    }

    pub fn private_hotel(&self, id: HotelId) -> Result<PrivateHotel, GameError> {
        let h = self.hotel(id)?;
        let s = self.latest_state(id)?;
        let now = RealNumbers {
            rooms: s.rooms,
            permanent: s.permanent,
            seasonal: s.seasonal,
            wage_pct: s.wage_pct,
            morale_pts: s.morale_pts,
            experience_pts: s.experience_pts,
            equipment_pts: s.equipment_pts,
            satisfaction_pts: s.satisfaction_pts,
            rating: s.rating,
            cash_cents: s.cash_cents,
            load_hundredths: s.load_hundredths,
        };
        let mut last_month = None;
        if let Some(rr) = self.rounds.iter().rev().find_map(|m| self.resolved.get(m)) {
            let res = rr
                .hotels
                .iter()
                .find(|x| x.hotel_id == id)
                .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
            let k = report_kpi(&res.report, &self.content.catalog)?;
            last_month = Some(LastMonth {
                round: rr.month,
                sold: res.report.market.sold,
                available: res.report.market.available,
                rooms_revenue_cents: res.report.money.rooms_revenue_cents,
                gop_cents: res.report.money.gop_cents,
                goppar_cents: res.report.money.goppar_cents,
                mentor: h.debriefs.get(&rr.month).map(|d| d.text.clone()),
                occupancy_bp: k.occupancy_bp,
                adr_cents: k.adr_cents,
                revpar_cents: k.revpar_cents,
            });
        }
        let mut plan = None;
        if let Some(m) = self.current_round()
            && self.status(m)? == RoundStatus::Open
        {
            let state = self.state_at(id, m)?;
            plan = Some(PlanState {
                round: m,
                plan: (&self.effective_plan(id, m)?).into(),
                limits: plan_limits(&state, m.require_month()?, &self.content.catalog)?,
            });
        }
        Ok(PrivateHotel {
            profile: h.profile.code().to_owned(),
            now,
            last_month,
            plan,
        })
    }
}
