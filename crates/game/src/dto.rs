//! Serde-zrcala tipova jezgre (jezgra nema serde). Polje za polje; iscrpni matchevi bez `_`, da nova varijanta
//! u jezgri ne prevede tiho.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sim_hotel::{
    Category, Cause, GroupBlock, GuestOutcome, HotelState, Index, IndexSet, Line, Maintenance,
    MarketOutcome, MoneyOutcome, Month, MonthPlan, MonthReport, Pending, PeopleOutcome, Training,
    Unit, Var,
};

use crate::dto_v2::DeptPlanDto;
use crate::error::{GameError, corrupt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonthCode {
    Jun,
    Jul,
    Aug,
    Sep,
}

impl From<Month> for MonthCode {
    fn from(m: Month) -> Self {
        match m {
            Month::Jun => MonthCode::Jun,
            Month::Jul => MonthCode::Jul,
            Month::Aug => MonthCode::Aug,
            Month::Sep => MonthCode::Sep,
        }
    }
}

impl From<MonthCode> for Month {
    fn from(m: MonthCode) -> Self {
        match m {
            MonthCode::Jun => Month::Jun,
            MonthCode::Jul => Month::Jul,
            MonthCode::Aug => Month::Aug,
            MonthCode::Sep => Month::Sep,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingCode {
    None,
    Basic,
    Full,
}

impl From<Training> for TrainingCode {
    fn from(t: Training) -> Self {
        match t {
            Training::None => TrainingCode::None,
            Training::Basic => TrainingCode::Basic,
            Training::Full => TrainingCode::Full,
        }
    }
}

impl From<TrainingCode> for Training {
    fn from(t: TrainingCode) -> Self {
        match t {
            TrainingCode::None => Training::None,
            TrainingCode::Basic => Training::Basic,
            TrainingCode::Full => Training::Full,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceCode {
    Deferred,
    Regular,
    Intensive,
}

impl From<Maintenance> for MaintenanceCode {
    fn from(m: Maintenance) -> Self {
        match m {
            Maintenance::Deferred => MaintenanceCode::Deferred,
            Maintenance::Regular => MaintenanceCode::Regular,
            Maintenance::Intensive => MaintenanceCode::Intensive,
        }
    }
}

impl From<MaintenanceCode> for Maintenance {
    fn from(m: MaintenanceCode) -> Self {
        match m {
            MaintenanceCode::Deferred => Maintenance::Deferred,
            MaintenanceCode::Regular => Maintenance::Regular,
            MaintenanceCode::Intensive => Maintenance::Intensive,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexCode {
    G,
    R,
    Z,
    O,
    F,
}

impl From<Index> for IndexCode {
    fn from(i: Index) -> Self {
        match i {
            Index::Guests => IndexCode::G,
            Index::Reviews => IndexCode::R,
            Index::Staff => IndexCode::Z,
            Index::Operations => IndexCode::O,
            Index::Finance => IndexCode::F,
        }
    }
}

/// Redoslijed `Index::ALL`; nizovi `[u16; 5]` u događajima slijede ga.
pub const INDEX_ORDER: [IndexCode; 5] = [
    IndexCode::G,
    IndexCode::R,
    IndexCode::Z,
    IndexCode::O,
    IndexCode::F,
];

pub fn indices_dto(s: &IndexSet) -> [u16; 5] {
    Index::ALL.map(|i| s.get(i).hundredths())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDto {
    pub price_cents: i64,
    pub marketing_cents: i64,
    pub seasonal_target: u16,
    pub wage_pct: u16,
    pub training: TrainingCode,
    pub maintenance: MaintenanceCode,
    pub renovation: bool,
    pub reply_reviews: bool,
    /// Postavke po odjelu (samo v2, ključ je kod odjela); v1 plan ga nema, pa se njegov JSON ne mijenja.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub departments: BTreeMap<String, DeptPlanDto>,
}

impl From<&MonthPlan> for PlanDto {
    fn from(p: &MonthPlan) -> Self {
        PlanDto {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
            departments: BTreeMap::new(),
        }
    }
}

impl From<&PlanDto> for MonthPlan {
    fn from(p: &PlanDto) -> Self {
        MonthPlan {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingDto {
    pub experience_pts: i32,
    pub morale_pts: i32,
    pub renovation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupBlockDto {
    pub month: MonthCode,
    pub room_nights: u32,
    pub price_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelStateDto {
    pub category: String,
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
    pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
}

impl From<&HotelState> for HotelStateDto {
    fn from(s: &HotelState) -> Self {
        HotelStateDto {
            category: s.category.code().to_owned(),
            rooms: s.rooms,
            permanent: s.permanent,
            seasonal: s.seasonal,
            wage_pct: s.wage_pct,
            extra_wage_cents: s.extra_wage_cents,
            morale_pts: s.morale_pts,
            experience_pts: s.experience_pts,
            equipment_pts: s.equipment_pts,
            satisfaction_pts: s.satisfaction_pts,
            rating: s.rating,
            cash_cents: s.cash_cents,
            fixed_cents: s.fixed_cents,
            load_hundredths: s.load_hundredths,
            turnover_bp: s.turnover_bp,
            renovated: s.renovated,
            pending: PendingDto {
                experience_pts: s.pending.experience_pts,
                morale_pts: s.pending.morale_pts,
                renovation: s.pending.renovation,
            },
            groups: s
                .groups
                .iter()
                .map(|g| GroupBlockDto {
                    month: g.month.into(),
                    room_nights: g.room_nights,
                    price_cents: g.price_cents,
                })
                .collect(),
        }
    }
}

impl TryFrom<&HotelStateDto> for HotelState {
    type Error = GameError;

    fn try_from(d: &HotelStateDto) -> Result<Self, GameError> {
        let category = Category::from_code(&d.category)
            .ok_or_else(|| corrupt(format!("kategorija {}", d.category)))?;
        Ok(HotelState {
            category,
            rooms: d.rooms,
            permanent: d.permanent,
            seasonal: d.seasonal,
            wage_pct: d.wage_pct,
            extra_wage_cents: d.extra_wage_cents,
            morale_pts: d.morale_pts,
            experience_pts: d.experience_pts,
            equipment_pts: d.equipment_pts,
            satisfaction_pts: d.satisfaction_pts,
            rating: d.rating,
            cash_cents: d.cash_cents,
            fixed_cents: d.fixed_cents,
            load_hundredths: d.load_hundredths,
            turnover_bp: d.turnover_bp,
            renovated: d.renovated,
            pending: Pending {
                experience_pts: d.pending.experience_pts,
                morale_pts: d.pending.morale_pts,
                renovation: d.pending.renovation,
            },
            groups: d
                .groups
                .iter()
                .map(|g| GroupBlock {
                    month: g.month.into(),
                    room_nights: g.room_nights,
                    price_cents: g.price_cents,
                })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketDto {
    pub rooms_out: u16,
    pub available: u32,
    pub f_price_bp: i32,
    pub f_reputation_bp: i32,
    pub f_marketing_bp: i32,
    pub base_demand: u32,
    pub demand: u32,
    pub staff_capacity: u32,
    pub group_room_nights: u32,
    pub sold: u32,
    pub group_sold: u32,
    pub individual_sold: u32,
    pub group_revenue_cents: i64,
}

impl From<&MarketOutcome> for MarketDto {
    fn from(m: &MarketOutcome) -> Self {
        MarketDto {
            rooms_out: m.rooms_out,
            available: m.available,
            f_price_bp: m.f_price_bp,
            f_reputation_bp: m.f_reputation_bp,
            f_marketing_bp: m.f_marketing_bp,
            base_demand: m.base_demand,
            demand: m.demand,
            staff_capacity: m.staff_capacity,
            group_room_nights: m.group_room_nights,
            sold: m.sold,
            group_sold: m.group_sold,
            individual_sold: m.individual_sold,
            group_revenue_cents: m.group_revenue_cents,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeopleDto {
    pub hires: u16,
    pub released: u16,
    pub staff: u16,
    pub load_hundredths: u16,
    pub load_penalty_pts: i32,
    pub morale_target_pts: i32,
    pub turnover_bp: u16,
    pub departures: u16,
    pub experience_mix_pts: i32,
}

impl From<&PeopleOutcome> for PeopleDto {
    fn from(p: &PeopleOutcome) -> Self {
        PeopleDto {
            hires: p.hires,
            released: p.released,
            staff: p.staff,
            load_hundredths: p.load_hundredths,
            load_penalty_pts: p.load_penalty_pts,
            morale_target_pts: p.morale_target_pts,
            turnover_bp: p.turnover_bp,
            departures: p.departures,
            experience_mix_pts: p.experience_mix_pts,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyDto {
    pub rooms_revenue_cents: i64,
    pub other_revenue_cents: i64,
    pub labor_cents: i64,
    pub costs_cents: i64,
    pub gop_cents: i64,
    pub capex_cents: i64,
    pub interest_cents: i64,
    pub goppar_cents: i64,
}

impl From<&MoneyOutcome> for MoneyDto {
    fn from(m: &MoneyOutcome) -> Self {
        MoneyDto {
            rooms_revenue_cents: m.rooms_revenue_cents,
            other_revenue_cents: m.other_revenue_cents,
            labor_cents: m.labor_cents,
            costs_cents: m.costs_cents,
            gop_cents: m.gop_cents,
            capex_cents: m.capex_cents,
            interest_cents: m.interest_cents,
            goppar_cents: m.goppar_cents,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestDto {
    pub quality_pts: i32,
    pub value_pts: i32,
    pub satisfaction_target_pts: i32,
    pub rating_target: u16,
}

impl From<&GuestOutcome> for GuestDto {
    fn from(g: &GuestOutcome) -> Self {
        GuestDto {
            quality_pts: g.quality_pts,
            value_pts: g.value_pts,
            satisfaction_target_pts: g.satisfaction_target_pts,
            rating_target: g.rating_target,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarCode {
    Sold,
    Revenue,
    Costs,
    Cash,
    Staff,
    Morale,
    Experience,
    Equipment,
    Satisfaction,
    Rating,
}

impl From<&Var> for VarCode {
    fn from(v: &Var) -> Self {
        match v {
            Var::Sold => VarCode::Sold,
            Var::Revenue => VarCode::Revenue,
            Var::Costs => VarCode::Costs,
            Var::Cash => VarCode::Cash,
            Var::Staff => VarCode::Staff,
            Var::Morale => VarCode::Morale,
            Var::Experience => VarCode::Experience,
            Var::Equipment => VarCode::Equipment,
            Var::Satisfaction => VarCode::Satisfaction,
            Var::Rating => VarCode::Rating,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseDto {
    Baseline,
    Price,
    Reputation,
    Marketing,
    Groups,
    Capacity,
    OtherRevenue,
    Wage,
    Load,
    Training,
    Hires,
    Release,
    Turnover,
    Maintenance,
    Renovation,
    ReplyReviews,
    FixedCosts,
    Interest,
    Situation(String),
}

impl From<&Cause> for CauseDto {
    fn from(c: &Cause) -> Self {
        match c {
            Cause::Baseline => CauseDto::Baseline,
            Cause::Price => CauseDto::Price,
            Cause::Reputation => CauseDto::Reputation,
            Cause::Marketing => CauseDto::Marketing,
            Cause::Groups => CauseDto::Groups,
            Cause::Capacity => CauseDto::Capacity,
            Cause::OtherRevenue => CauseDto::OtherRevenue,
            Cause::Wage => CauseDto::Wage,
            Cause::Load => CauseDto::Load,
            Cause::Training => CauseDto::Training,
            Cause::Hires => CauseDto::Hires,
            Cause::Release => CauseDto::Release,
            Cause::Turnover => CauseDto::Turnover,
            Cause::Maintenance => CauseDto::Maintenance,
            Cause::Renovation => CauseDto::Renovation,
            Cause::ReplyReviews => CauseDto::ReplyReviews,
            Cause::FixedCosts => CauseDto::FixedCosts,
            Cause::Interest => CauseDto::Interest,
            Cause::Situation(code) => CauseDto::Situation(code.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitCode {
    RoomNights,
    Cents,
    People,
    Points,
    RatingHundredths,
    Bp,
}

impl From<&Unit> for UnitCode {
    fn from(u: &Unit) -> Self {
        match u {
            Unit::RoomNights => UnitCode::RoomNights,
            Unit::Cents => UnitCode::Cents,
            Unit::People => UnitCode::People,
            Unit::Points => UnitCode::Points,
            Unit::RatingHundredths => UnitCode::RatingHundredths,
            Unit::Bp => UnitCode::Bp,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineDto {
    pub var: VarCode,
    pub cause: CauseDto,
    pub amount: i64,
    pub unit: UnitCode,
}

impl From<&Line> for LineDto {
    fn from(l: &Line) -> Self {
        LineDto {
            var: (&l.var).into(),
            cause: (&l.cause).into(),
            amount: l.amount,
            unit: (&l.unit).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonthReportDto {
    pub month: MonthCode,
    pub before: HotelStateDto,
    pub after: HotelStateDto,
    pub market: MarketDto,
    pub people: PeopleDto,
    pub money: MoneyDto,
    pub guests: GuestDto,
    pub lines: Vec<LineDto>,
}

impl From<&MonthReport> for MonthReportDto {
    fn from(r: &MonthReport) -> Self {
        MonthReportDto {
            month: r.month.into(),
            before: (&r.before).into(),
            after: (&r.after).into(),
            market: (&r.market).into(),
            people: (&r.people).into(),
            money: (&r.money).into(),
            guests: (&r.guests).into(),
            lines: r.lines.iter().map(LineDto::from).collect(),
        }
    }
}
