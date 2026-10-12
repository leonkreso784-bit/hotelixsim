//! Serde-zrcala tipova jezgre v2 (jezgra nema serde). Polje za polje, kao `dto.rs`; v2 dijelovi koje v1 nema
//! (odjeli) dolaze ovamo zajedno sa stanjem.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sim_hotel::{Category, GroupBlock, Pending};
use sim_hotel_v2::department::{
    AssignedCosts, DeptCode, DeptOutcome, DeptPlan, DeptState, PeriodAccounts,
    UndistributedOutcome, UsaliClass,
};
use sim_hotel_v2::{AssetBook, Building, BuildingOutcome, HotelState, PeriodPlan, Renewal};

use crate::dto::{GroupBlockDto, MaintenanceCode, MonthCode, PendingDto, PlanDto};
use crate::error::{GameError, corrupt};

/// v2 dio obračuna kruga uz v1 sažetak (`HotelResolution.report`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailDto {
    /// Stanje v2 nakon kruga; iz njega kreće idući krug.
    pub after: HotelStateV2Dto,
    /// Računi odjela kruga (izvor računa dobiti i pogleda odjela); krugovi obračunati prije Taska 14 ga nemaju.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounts: Option<PeriodAccountsDto>,
}

/// Serde-zrcalo `sim_hotel_v2::department::DeptOutcome`; `class` je kod `UsaliClass::code`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeptOutcomeDto {
    pub code: String,
    pub class: String,
    pub open: bool,
    pub staff: u16,
    pub capacity_uses: u32,
    pub demand_uses: u32,
    pub served_uses: u32,
    pub usage_bp: u16,
    pub load_hundredths: u16,
    pub load_penalty_pts: i32,
    pub quality_pts: i32,
    pub price_level_pct: u16,
    pub revenue_cents: i64,
    pub payroll_cents: i64,
    pub cost_of_sales_cents: i64,
    pub other_direct_cents: i64,
    pub profit_cents: i64,
    pub condition_before_pts: i32,
    pub wear_pts: i32,
    pub load_wear_pts: i32,
    pub maintenance_pts: i32,
    pub condition_after_pts: i32,
    pub maintenance_cents: i64,
    pub value_pts: i32,
    pub satisfaction_before_pts: i32,
    pub satisfaction_target_pts: i32,
    pub satisfaction_after_pts: i32,
    pub guest_weight: u32,
    /// Odlasci zaposlenih odjela u krugu; krugovi obračunati prije Taska 23 ga nemaju (0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub departures: u16,
    /// Obnova v2 plaćena u krugu i bodovi koje je stvarno donijela; krugovi bez obnove ih nemaju (0).
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub renewal_cents: i64,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub renewal_pts: i32,
    /// Stanje koje gost osjeća (sa zgradom); krugovi obračunati prije obnove v2 ga nemaju (= stanje nakon kruga).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub felt_condition_pts: Option<i32>,
}

/// Serde-zrcalo `sim_hotel_v2::department::BuildingOutcome` (zgrada u krugu).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingOutcomeDto {
    pub before_pts: i32,
    pub wear_pts: i32,
    pub renewal_pts: i32,
    pub after_pts: i32,
    pub renewal_cents: i64,
}

/// Serde-zrcalo `sim_hotel_v2::Renewal` (obnova odlučena u mjesecu, zaključana do kraja mjeseca).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenewalDto {
    pub month: MonthCode,
    pub total_cents: i64,
    pub installment_cents: i64,
    pub paid_cents: i64,
    pub gain_pts: i32,
}

/// Serde-zrcalo `sim_hotel_v2::AssetBook`; prazna knjiga se ne zapisuje.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBookDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renewal: Option<RenewalDto>,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub capitalized_cents: i64,
}

/// Serde-zrcalo `sim_hotel_v2::Building` (stanje zgrade i knjiga).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingStateDto {
    pub condition_pts: i32,
    pub start_pts: i32,
    #[serde(default, skip_serializing_if = "AssetBookDto::is_empty")]
    pub book: AssetBookDto,
}

impl AssetBookDto {
    fn is_empty(&self) -> bool {
        *self == AssetBookDto::default()
    }
}

impl From<&AssetBook> for AssetBookDto {
    fn from(b: &AssetBook) -> Self {
        AssetBookDto {
            renewal: b.renewal.map(|r| RenewalDto {
                month: r.month.into(),
                total_cents: r.total_cents,
                installment_cents: r.installment_cents,
                paid_cents: r.paid_cents,
                gain_pts: r.gain_pts,
            }),
            capitalized_cents: b.capitalized_cents,
        }
    }
}

impl From<&AssetBookDto> for AssetBook {
    fn from(b: &AssetBookDto) -> Self {
        AssetBook {
            renewal: b.renewal.as_ref().map(|r| Renewal {
                month: r.month.into(),
                total_cents: r.total_cents,
                installment_cents: r.installment_cents,
                paid_cents: r.paid_cents,
                gain_pts: r.gain_pts,
            }),
            capitalized_cents: b.capitalized_cents,
        }
    }
}

impl From<&BuildingOutcome> for BuildingOutcomeDto {
    fn from(b: &BuildingOutcome) -> Self {
        BuildingOutcomeDto {
            before_pts: b.before_pts,
            wear_pts: b.wear_pts,
            renewal_pts: b.renewal_pts,
            after_pts: b.after_pts,
            renewal_cents: b.renewal_cents,
        }
    }
}

impl From<&BuildingOutcomeDto> for BuildingOutcome {
    fn from(b: &BuildingOutcomeDto) -> Self {
        BuildingOutcome {
            before_pts: b.before_pts,
            wear_pts: b.wear_pts,
            renewal_pts: b.renewal_pts,
            after_pts: b.after_pts,
            renewal_cents: b.renewal_cents,
        }
    }
}

/// Serde-zrcalo `sim_hotel_v2::department::AssignedCosts`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignedCostsDto {
    pub marketing_cents: i64,
    pub reply_cents: i64,
    pub maintenance_cents: i64,
    pub training_cents: i64,
    pub hiring_cents: i64,
    pub situation_cents: i64,
}

/// Serde-zrcalo `sim_hotel_v2::department::UndistributedOutcome`; `kind` je kod razreda (`UsaliClass::code`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UndistributedOutcomeDto {
    pub code: String,
    pub kind: String,
    pub staff: u16,
    pub payroll_cents: i64,
    pub fixed_cents: i64,
    pub per_room_cents: i64,
    pub assigned: AssignedCostsDto,
    pub total_cents: i64,
    /// Odlasci zaposlenih odjela u krugu; krugovi obračunati prije Taska 23 ga nemaju (0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub departures: u16,
}

/// Serde-zrcalo `sim_hotel_v2::department::PeriodAccounts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodAccountsDto {
    pub days: u8,
    pub rooms: u16,
    pub departments: BTreeMap<String, DeptOutcomeDto>,
    pub undistributed: BTreeMap<String, UndistributedOutcomeDto>,
    pub fixed_charges: BTreeMap<String, i64>,
    pub interest_cents: i64,
    pub capex_cents: i64,
    /// Zgrada u krugu; krugovi obračunati prije obnove v2 je nemaju.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building: Option<BuildingOutcomeDto>,
    /// Amortizacija opreme i zgrade (obnova v2); bez nje 0.
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub depreciation_equipment_cents: i64,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub depreciation_building_cents: i64,
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

fn is_zero_i64(n: &i64) -> bool {
    *n == 0
}

fn class_of(code: &str) -> Result<UsaliClass, GameError> {
    UsaliClass::from_code(code).ok_or_else(|| corrupt(format!("razred odjela {code}")))
}

impl From<&DeptOutcome> for DeptOutcomeDto {
    fn from(o: &DeptOutcome) -> Self {
        DeptOutcomeDto {
            code: o.code.as_str().to_owned(),
            class: o.class.code().to_owned(),
            open: o.open,
            staff: o.staff,
            capacity_uses: o.capacity_uses,
            demand_uses: o.demand_uses,
            served_uses: o.served_uses,
            usage_bp: o.usage_bp,
            load_hundredths: o.load_hundredths,
            load_penalty_pts: o.load_penalty_pts,
            quality_pts: o.quality_pts,
            price_level_pct: o.price_level_pct,
            revenue_cents: o.revenue_cents,
            payroll_cents: o.payroll_cents,
            cost_of_sales_cents: o.cost_of_sales_cents,
            other_direct_cents: o.other_direct_cents,
            profit_cents: o.profit_cents,
            condition_before_pts: o.condition_before_pts,
            wear_pts: o.wear_pts,
            load_wear_pts: o.load_wear_pts,
            maintenance_pts: o.maintenance_pts,
            condition_after_pts: o.condition_after_pts,
            maintenance_cents: o.maintenance_cents,
            value_pts: o.value_pts,
            satisfaction_before_pts: o.satisfaction_before_pts,
            satisfaction_target_pts: o.satisfaction_target_pts,
            satisfaction_after_pts: o.satisfaction_after_pts,
            guest_weight: o.guest_weight,
            departures: o.departures,
            renewal_cents: o.renewal_cents,
            renewal_pts: o.renewal_pts,
            felt_condition_pts: Some(o.felt_condition_pts),
        }
    }
}

impl TryFrom<&DeptOutcomeDto> for DeptOutcome {
    type Error = GameError;

    fn try_from(o: &DeptOutcomeDto) -> Result<Self, GameError> {
        Ok(DeptOutcome {
            code: DeptCode(o.code.clone()),
            class: class_of(&o.class)?,
            open: o.open,
            staff: o.staff,
            capacity_uses: o.capacity_uses,
            demand_uses: o.demand_uses,
            served_uses: o.served_uses,
            usage_bp: o.usage_bp,
            load_hundredths: o.load_hundredths,
            load_penalty_pts: o.load_penalty_pts,
            quality_pts: o.quality_pts,
            price_level_pct: o.price_level_pct,
            revenue_cents: o.revenue_cents,
            payroll_cents: o.payroll_cents,
            cost_of_sales_cents: o.cost_of_sales_cents,
            other_direct_cents: o.other_direct_cents,
            profit_cents: o.profit_cents,
            condition_before_pts: o.condition_before_pts,
            wear_pts: o.wear_pts,
            load_wear_pts: o.load_wear_pts,
            maintenance_pts: o.maintenance_pts,
            condition_after_pts: o.condition_after_pts,
            maintenance_cents: o.maintenance_cents,
            value_pts: o.value_pts,
            satisfaction_before_pts: o.satisfaction_before_pts,
            satisfaction_target_pts: o.satisfaction_target_pts,
            satisfaction_after_pts: o.satisfaction_after_pts,
            guest_weight: o.guest_weight,
            departures: o.departures,
            renewal_cents: o.renewal_cents,
            renewal_pts: o.renewal_pts,
            felt_condition_pts: o.felt_condition_pts.unwrap_or(o.condition_after_pts),
        })
    }
}

impl From<&AssignedCosts> for AssignedCostsDto {
    fn from(a: &AssignedCosts) -> Self {
        AssignedCostsDto {
            marketing_cents: a.marketing_cents,
            reply_cents: a.reply_cents,
            maintenance_cents: a.maintenance_cents,
            training_cents: a.training_cents,
            hiring_cents: a.hiring_cents,
            situation_cents: a.situation_cents,
        }
    }
}

impl From<&AssignedCostsDto> for AssignedCosts {
    fn from(a: &AssignedCostsDto) -> Self {
        AssignedCosts {
            marketing_cents: a.marketing_cents,
            reply_cents: a.reply_cents,
            maintenance_cents: a.maintenance_cents,
            training_cents: a.training_cents,
            hiring_cents: a.hiring_cents,
            situation_cents: a.situation_cents,
        }
    }
}

impl From<&UndistributedOutcome> for UndistributedOutcomeDto {
    fn from(u: &UndistributedOutcome) -> Self {
        UndistributedOutcomeDto {
            code: u.code.as_str().to_owned(),
            kind: UsaliClass::Undistributed(u.kind).code().to_owned(),
            staff: u.staff,
            payroll_cents: u.payroll_cents,
            fixed_cents: u.fixed_cents,
            per_room_cents: u.per_room_cents,
            assigned: (&u.assigned).into(),
            total_cents: u.total_cents,
            departures: u.departures,
        }
    }
}

impl TryFrom<&UndistributedOutcomeDto> for UndistributedOutcome {
    type Error = GameError;

    fn try_from(u: &UndistributedOutcomeDto) -> Result<Self, GameError> {
        let UsaliClass::Undistributed(kind) = class_of(&u.kind)? else {
            return Err(corrupt(format!("{} nije neraspoređeni odjel", u.kind)));
        };
        Ok(UndistributedOutcome {
            code: DeptCode(u.code.clone()),
            kind,
            staff: u.staff,
            payroll_cents: u.payroll_cents,
            fixed_cents: u.fixed_cents,
            per_room_cents: u.per_room_cents,
            assigned: (&u.assigned).into(),
            total_cents: u.total_cents,
            departures: u.departures,
        })
    }
}

impl From<&PeriodAccounts> for PeriodAccountsDto {
    fn from(a: &PeriodAccounts) -> Self {
        PeriodAccountsDto {
            days: a.days,
            rooms: a.rooms,
            departments: by_string(&a.departments, |d| DeptOutcomeDto::from(d)),
            undistributed: by_string(&a.undistributed, |u| UndistributedOutcomeDto::from(u)),
            fixed_charges: a.fixed_charges.clone(),
            interest_cents: a.interest_cents,
            capex_cents: a.capex_cents,
            building: Some(BuildingOutcomeDto::from(&a.building)),
            depreciation_equipment_cents: a.depreciation_equipment_cents,
            depreciation_building_cents: a.depreciation_building_cents,
        }
    }
}

impl TryFrom<&PeriodAccountsDto> for PeriodAccounts {
    type Error = GameError;

    fn try_from(a: &PeriodAccountsDto) -> Result<Self, GameError> {
        let mut departments = BTreeMap::new();
        for (k, v) in &a.departments {
            departments.insert(DeptCode(k.clone()), DeptOutcome::try_from(v)?);
        }
        let mut undistributed = BTreeMap::new();
        for (k, v) in &a.undistributed {
            undistributed.insert(DeptCode(k.clone()), UndistributedOutcome::try_from(v)?);
        }
        Ok(PeriodAccounts {
            days: a.days,
            rooms: a.rooms,
            departments,
            undistributed,
            fixed_charges: a.fixed_charges.clone(),
            interest_cents: a.interest_cents,
            capex_cents: a.capex_cents,
            building: a
                .building
                .as_ref()
                .map(BuildingOutcome::from)
                .unwrap_or_default(),
            depreciation_equipment_cents: a.depreciation_equipment_cents,
            depreciation_building_cents: a.depreciation_building_cents,
        })
    }
}

/// Serde-zrcalo `sim_hotel_v2::department::DeptState`; `class` je kod `UsaliClass::code`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeptStateDto {
    pub class: String,
    pub open: bool,
    pub permanent: u16,
    pub seasonal: u16,
    pub condition_pts: i32,
    pub satisfaction_pts: i32,
    /// Odlasci odjela u dosadašnjim krugovima sezone; stari dnevnici ga nemaju (čita se kao 0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub season_departures: u16,
    /// Početno stanje prostora iz profila (obnova v2); stari dnevnici ga nemaju (čita se kao stanje).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_condition_pts: Option<i32>,
    /// Knjiga imovine odjela (obnova v2); prazna se ne zapisuje.
    #[serde(default, skip_serializing_if = "AssetBookDto::is_empty")]
    pub book: AssetBookDto,
}

/// Serde-zrcalo `sim_hotel_v2::department::DeptPlan` (postavke jednog odjela u planu kruga).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeptPlanDto {
    pub open: bool,
    pub seasonal_target: u16,
    pub price_level_pct: u16,
    pub maintenance: MaintenanceCode,
}

/// Serde-zrcalo `sim_hotel_v2::HotelState` (polje za polje): odjeli umjesto v1 agregata osoblja, opreme i
/// stalnog troška (v1 oblik daje `HotelState::summary`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelStateV2Dto {
    pub category: String,
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
    pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
    pub departments: BTreeMap<String, DeptStateDto>,
    pub undistributed_fixed_cents: BTreeMap<String, i64>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
    /// Zgrada (obnova v2); stari dnevnici je nemaju (zgrada 100/100 bez knjige).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building: Option<BuildingStateDto>,
}

impl From<&DeptState> for DeptStateDto {
    fn from(d: &DeptState) -> Self {
        DeptStateDto {
            class: d.class.code().to_owned(),
            open: d.open,
            permanent: d.permanent,
            seasonal: d.seasonal,
            condition_pts: d.condition_pts,
            satisfaction_pts: d.satisfaction_pts,
            season_departures: d.season_departures,
            start_condition_pts: Some(d.start_condition_pts),
            book: AssetBookDto::from(&d.book),
        }
    }
}

impl TryFrom<&DeptStateDto> for DeptState {
    type Error = GameError;

    fn try_from(d: &DeptStateDto) -> Result<Self, GameError> {
        Ok(DeptState {
            class: UsaliClass::from_code(&d.class)
                .ok_or_else(|| corrupt(format!("razred odjela {}", d.class)))?,
            open: d.open,
            permanent: d.permanent,
            seasonal: d.seasonal,
            condition_pts: d.condition_pts,
            satisfaction_pts: d.satisfaction_pts,
            season_departures: d.season_departures,
            start_condition_pts: d.start_condition_pts.unwrap_or(d.condition_pts),
            book: AssetBook::from(&d.book),
        })
    }
}

impl From<&DeptPlan> for DeptPlanDto {
    fn from(d: &DeptPlan) -> Self {
        DeptPlanDto {
            open: d.open,
            seasonal_target: d.seasonal_target,
            price_level_pct: d.price_level_pct,
            maintenance: d.maintenance.into(),
        }
    }
}

impl From<&DeptPlanDto> for DeptPlan {
    fn from(d: &DeptPlanDto) -> Self {
        DeptPlan {
            open: d.open,
            seasonal_target: d.seasonal_target,
            price_level_pct: d.price_level_pct,
            maintenance: d.maintenance.into(),
            // Iznos obnove v2 u planu dolazi s Taskom O-3 (tijelo plana); dotad 0.
            renewal_cents: 0,
        }
    }
}

fn by_string<V, W>(m: &BTreeMap<DeptCode, V>, f: impl Fn(&V) -> W) -> BTreeMap<String, W> {
    m.iter()
        .map(|(k, v)| (k.as_str().to_owned(), f(v)))
        .collect()
}

impl From<&HotelState> for HotelStateV2Dto {
    fn from(s: &HotelState) -> Self {
        HotelStateV2Dto {
            category: s.category.code().to_owned(),
            rooms: s.rooms,
            wage_pct: s.wage_pct,
            extra_wage_cents: s.extra_wage_cents,
            morale_pts: s.morale_pts,
            experience_pts: s.experience_pts,
            satisfaction_pts: s.satisfaction_pts,
            rating: s.rating,
            cash_cents: s.cash_cents,
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
            departments: by_string(&s.departments, |d| DeptStateDto::from(d)),
            undistributed_fixed_cents: by_string(&s.undistributed_fixed_cents, |v| *v),
            fixed_charges_cents: s.fixed_charges_cents.clone(),
            building: Some(BuildingStateDto {
                condition_pts: s.building.condition_pts,
                start_pts: s.building.start_pts,
                book: AssetBookDto::from(&s.building.book),
            }),
        }
    }
}

impl TryFrom<&HotelStateV2Dto> for HotelState {
    type Error = GameError;

    fn try_from(d: &HotelStateV2Dto) -> Result<Self, GameError> {
        let category = Category::from_code(&d.category)
            .ok_or_else(|| corrupt(format!("kategorija {}", d.category)))?;
        let mut departments = BTreeMap::new();
        for (k, v) in &d.departments {
            departments.insert(DeptCode(k.clone()), DeptState::try_from(v)?);
        }
        Ok(HotelState {
            category,
            rooms: d.rooms,
            wage_pct: d.wage_pct,
            extra_wage_cents: d.extra_wage_cents,
            morale_pts: d.morale_pts,
            experience_pts: d.experience_pts,
            satisfaction_pts: d.satisfaction_pts,
            rating: d.rating,
            cash_cents: d.cash_cents,
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
            departments,
            undistributed_fixed_cents: d
                .undistributed_fixed_cents
                .iter()
                .map(|(k, v)| (DeptCode(k.clone()), *v))
                .collect(),
            fixed_charges_cents: d.fixed_charges_cents.clone(),
            building: d.building.as_ref().map_or_else(
                || Building {
                    condition_pts: 10_000,
                    start_pts: 10_000,
                    book: AssetBook::default(),
                },
                |b| Building {
                    condition_pts: b.condition_pts,
                    start_pts: b.start_pts,
                    book: AssetBook::from(&b.book),
                },
            ),
        })
    }
}

/// v2 plan u v1 obliku tijela plana: `seasonal_target` je zbroj odjela, `maintenance` razina Smještaja (za v1
/// klijente i prikaz); postavke odjela idu u `departments`.
impl From<&PeriodPlan> for PlanDto {
    fn from(p: &PeriodPlan) -> Self {
        PlanDto {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_total(),
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.rooms_maintenance().into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
            departments: by_string(&p.departments, |d| DeptPlanDto::from(d)),
        }
    }
}

/// v2 plan traži postavke odjela; `seasonal_target` i `maintenance` iz DTO-a se ne čitaju (izvode se iz odjela).
/// Prazni `departments` ovdje je pokvaren dnevnik: zahtjev bez odjela dopunjuje `Game::plan_from_dto`.
impl TryFrom<&PlanDto> for PeriodPlan {
    type Error = GameError;

    fn try_from(p: &PlanDto) -> Result<Self, GameError> {
        if p.departments.is_empty() {
            return Err(corrupt("v2 plan bez odjela"));
        }
        Ok(PeriodPlan {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
            departments: p
                .departments
                .iter()
                .map(|(k, v)| (DeptCode(k.clone()), DeptPlan::from(v)))
                .collect(),
            // Iznos obnove zgrade u planu dolazi s Taskom O-3 (tijelo plana); dotad 0.
            building_renewal_cents: 0,
        })
    }
}
