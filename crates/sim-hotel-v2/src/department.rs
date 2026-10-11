//! Odjeli po USALI razredima (MODEL_V2 §3.2): tipovi i čiste formule jednog odjela u jednom razdoblju.
//! Još nisu spojeni u obračun razdoblja (Task 11–12). Sve kroz `engine::calc`, samo cijeli brojevi.

use std::collections::BTreeMap;

use engine::calc::{self as c, Calc, CalcError};
use sim_hotel::{Category, LevelEffect, Maintenance, MonthTable, load_penalty};

use crate::calendar::{Period, avg_u16, prorate};

/// Stabilan kod odjela (`rooms`, `restaurant`, …); ključ svih mapa odjela.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeptCode(pub String);

impl DeptCode {
    pub fn rooms() -> DeptCode {
        DeptCode("rooms".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UndistributedKind {
    AdminGeneral,
    SalesMarketing,
    Maintenance,
    Utilities,
}

/// USALI razred odjela (MODEL_V2 §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsaliClass {
    Rooms,
    FoodBeverage,
    OtherOperated,
    Rentals,
    Undistributed(UndistributedKind),
}

impl UsaliClass {
    /// rooms, food_beverage, other_operated, rentals, admin_general, sales_marketing, maintenance, utilities
    pub fn code(self) -> &'static str {
        match self {
            UsaliClass::Rooms => "rooms",
            UsaliClass::FoodBeverage => "food_beverage",
            UsaliClass::OtherOperated => "other_operated",
            UsaliClass::Rentals => "rentals",
            UsaliClass::Undistributed(UndistributedKind::AdminGeneral) => "admin_general",
            UsaliClass::Undistributed(UndistributedKind::SalesMarketing) => "sales_marketing",
            UsaliClass::Undistributed(UndistributedKind::Maintenance) => "maintenance",
            UsaliClass::Undistributed(UndistributedKind::Utilities) => "utilities",
        }
    }

    pub fn from_code(s: &str) -> Option<UsaliClass> {
        match s {
            "rooms" => Some(UsaliClass::Rooms),
            "food_beverage" => Some(UsaliClass::FoodBeverage),
            "other_operated" => Some(UsaliClass::OtherOperated),
            "rentals" => Some(UsaliClass::Rentals),
            "admin_general" => Some(UsaliClass::Undistributed(UndistributedKind::AdminGeneral)),
            "sales_marketing" => Some(UsaliClass::Undistributed(UndistributedKind::SalesMarketing)),
            "maintenance" => Some(UsaliClass::Undistributed(UndistributedKind::Maintenance)),
            "utilities" => Some(UsaliClass::Undistributed(UndistributedKind::Utilities)),
            _ => None,
        }
    }

    pub fn is_undistributed(self) -> bool {
        matches!(self, UsaliClass::Undistributed(_))
    }

    /// Redoslijed u računu dobiti: Smještaj, hrana i piće, ostali operativni, ostali prihodi, A&G, S&M, POM, energija.
    pub fn rank(self) -> u8 {
        match self {
            UsaliClass::Rooms => 0,
            UsaliClass::FoodBeverage => 1,
            UsaliClass::OtherOperated => 2,
            UsaliClass::Rentals => 3,
            UsaliClass::Undistributed(UndistributedKind::AdminGeneral) => 4,
            UsaliClass::Undistributed(UndistributedKind::SalesMarketing) => 5,
            UsaliClass::Undistributed(UndistributedKind::Maintenance) => 6,
            UsaliClass::Undistributed(UndistributedKind::Utilities) => 7,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevenueModel {
    /// Prihod soba (samo Smještaj): računa ga tržište.
    Rooms,
    /// Noćenja gostiju × udio koji koristi odjel × prosječni račun.
    PerGuest {
        usage_bp: MonthTable<u16>,
        check_cents: i64,
        price_elasticity_bp: i32,
        satisfaction_elasticity_bp: i32,
    },
    /// Stalni iznos mjesečno + iznos po noćenju (parking, najam lokala).
    Fixed {
        monthly_cents: i64,
        per_guest_night_cents: i64,
    },
    /// Udio prihoda soba — samo degenerirani katalog (v1 `other_revenue_bp`).
    ShareOfRooms { bp: u16 },
    /// Neraspoređeni odjeli.
    None,
}

/// Norma osoblja odjela za jednu kategoriju hotela: raspon ljudi (stalni + sezonci) na 100 soba, u desetinkama
/// (Smještaj 4*: 170–250 = 17–25 ljudi na 100 soba). Provjerava se samo na početnom stanju profila
/// (`Catalog::validate`); obračun kruga je ne koristi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffingNorm {
    pub min_tenths_per_100_rooms: u16,
    pub max_tenths_per_100_rooms: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptSpec {
    pub name: String,
    pub class: UsaliClass,
    pub revenue: RevenueModel,
    pub cost_of_sales_bp: u16,
    pub direct_per_use_cents: i64,
    pub wage_cents: i64,
    /// Korištenja po zaposlenom na dan, u desetinkama (Smještaj 24 = 2,4 prodane noći kao u v1).
    pub capacity_tenths_per_staff_day: u16,
    /// Trošenje stanja prostora mjesečno (pozitivan broj, oduzima se).
    pub wear_pts_per_month: i32,
    /// Razine održavanja: [odgođeno, redovno, pojačano] — mjesečni trošak i bodovi stanja.
    pub maintenance: [LevelEffect; 3],
    pub fixed_monthly_cents: i64,
    pub per_occupied_room_cents: i64,
    pub seasonal_max: u16,
    pub seasonal_plan: MonthTable<u16>,
    pub can_close: bool,
    /// Norma osoblja po kategoriji hotela; prazno = odjel bez norme (hrana i piće, energija, degenerirani katalog).
    pub staffing: BTreeMap<Category, StaffingNorm>,
}

/// `class` se prepisuje iz `DeptSpec` pri stvaranju stanja i ne mijenja se; služi da `HotelState::summary(&self)`
/// (Task 11) bez kataloga zna koji su odjeli operativni.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptState {
    pub class: UsaliClass,
    pub open: bool,
    pub permanent: u16,
    pub seasonal: u16,
    pub condition_pts: i32,
    pub satisfaction_pts: i32,
    /// Odlasci zaposlenih ovog odjela u dosadašnjim krugovima sezone (`split_season_departures`); počinje od 0.
    pub season_departures: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptPlan {
    pub open: bool,
    pub seasonal_target: u16,
    pub price_level_pct: u16,
    pub maintenance: Maintenance,
}

pub struct DeptInput<'a> {
    pub code: DeptCode,
    pub period: &'a Period,
    pub spec: &'a DeptSpec,
    pub state: &'a DeptState,
    pub plan: &'a DeptPlan,
    pub guest_nights: u32,
    pub rooms_revenue_cents: i64,
    pub experience_pts: i32,
    pub morale_pts: i32,
    pub wage_pct: u16,
    pub satisfaction_step_bp: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptOutcome {
    pub code: DeptCode,
    pub class: UsaliClass,
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
    /// Trošak održavanja ovog odjela — knjiži se u neraspoređeni odjel „Održavanje" (🟨 P9), ne u rezultat odjela.
    pub maintenance_cents: i64,
    pub value_pts: i32,
    pub satisfaction_before_pts: i32,
    pub satisfaction_target_pts: i32,
    pub satisfaction_after_pts: i32,
    /// Težina u kvaliteti i zadovoljstvu hotela: korištenja (Smještaj: noćenja gostiju); 0 = ne dotiče goste.
    pub guest_weight: u32,
    /// Odlasci zaposlenih ovog odjela u krugu (dio hotelskog broja, `split_departures`); popunjava ih obračun kruga.
    pub departures: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssignedCosts {
    pub marketing_cents: i64,
    pub reply_cents: i64,
    pub maintenance_cents: i64,
    pub training_cents: i64,
    pub hiring_cents: i64,
    pub situation_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndistributedOutcome {
    pub code: DeptCode,
    pub kind: UndistributedKind,
    pub staff: u16,
    pub payroll_cents: i64,
    pub fixed_cents: i64,
    pub per_room_cents: i64,
    pub assigned: AssignedCosts,
    pub total_cents: i64,
    /// Odlasci zaposlenih ovog odjela u krugu (dio hotelskog broja, `split_departures`); popunjava ih obračun kruga.
    pub departures: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodAccounts {
    pub days: u8,
    pub rooms: u16,
    /// Smještaj + operativni + ostali prihodi.
    pub departments: BTreeMap<DeptCode, DeptOutcome>,
    pub undistributed: BTreeMap<DeptCode, UndistributedOutcome>,
    /// Osiguranje, najam, porez — razmjerno razdoblju.
    pub fixed_charges: BTreeMap<String, i64>,
    pub interest_cents: i64,
    pub capex_cents: i64,
}

fn w(x: impl Into<i128>) -> i128 {
    x.into()
}

/// Plaća po zaposlenom (v1: `⌊wage/100⌋·wage_pct`) za `staff` ljudi, razmjerno razdoblju.
fn payroll(staff: u16, wage_cents: i64, wage_pct: u16, p: &Period) -> Calc<i64> {
    let monthly = c::mul(
        c::mul(w(staff), c::div_floor(w(wage_cents), 100)?)?,
        w(wage_pct),
    )?;
    prorate(c::narrow(monthly)?, p)
}

fn level_index(m: Maintenance) -> usize {
    match m {
        Maintenance::Deferred => 0,
        Maintenance::Regular => 1,
        Maintenance::Intensive => 2,
    }
}

/// Jedan operativni odjel ili odjel ostalih prihoda (ne Smještaj, ne neraspoređeni) u razdoblju.
pub fn dept_step(i: &DeptInput) -> Calc<DeptOutcome> {
    let (p, spec, st, pl) = (i.period, i.spec, i.state, i.plan);
    let open = pl.open;
    let level = w(pl.price_level_pct);
    let sat_before = w(st.satisfaction_pts);

    // 1–4 udio gostiju koji koristi odjel
    let usage: i128 = match &spec.revenue {
        RevenueModel::PerGuest {
            usage_bp,
            price_elasticity_bp,
            satisfaction_elasticity_bp,
            ..
        } => {
            let base = w(avg_u16(usage_bp, p)?);
            let f_price = c::add(
                10_000,
                c::div_round(c::mul(w(*price_elasticity_bp), c::sub(level, 100)?)?, 100)?,
            )?
            .clamp(0, 20_000);
            let f_sat = c::add(
                10_000,
                c::div_round(
                    c::mul(w(*satisfaction_elasticity_bp), c::sub(sat_before, 7_000)?)?,
                    10_000,
                )?,
            )?
            .clamp(0, 20_000);
            c::div_round(c::mul(c::mul(base, f_price)?, f_sat)?, 100_000_000)?.clamp(0, 10_000)
        }
        _ => 0,
    };
    let is_per_guest = matches!(spec.revenue, RevenueModel::PerGuest { .. });

    // 5 potražnja
    let demand: i128 = if open && is_per_guest {
        c::div_round(c::mul(w(i.guest_nights), usage)?, 10_000)?
    } else {
        0
    };

    // 6 ljudi i kapacitet
    let staff: u16 = c::narrow(c::add(
        w(st.permanent),
        if open { w(pl.seasonal_target) } else { 0 },
    )?)?;
    let capacity = c::div_floor(
        c::mul(
            c::mul(w(staff), w(p.days))?,
            w(spec.capacity_tenths_per_staff_day),
        )?,
        10,
    )?;

    // 7 usluženo
    let served = demand.min(capacity);

    // 8 opterećenje (pri punom kapacitetu 240, kao u v1)
    let denom = c::mul(
        c::mul(w(staff), w(p.days))?,
        w(spec.capacity_tenths_per_staff_day),
    )?;
    let load: i128 = if denom == 0 {
        if served > 0 { 300 } else { 0 }
    } else {
        c::div_round(c::mul(served, 2_400)?, denom)?.clamp(0, 300)
    };
    let load_hundredths: u16 = c::narrow(load)?;

    // 9 kazna i kvaliteta
    let penalty = load_penalty(load_hundredths)?;
    let quality = c::div_round(
        c::sum(&[
            c::mul(4_000, w(i.experience_pts))?,
            c::mul(3_000, w(i.morale_pts))?,
            c::mul(3_000, c::sub(10_000, w(penalty))?)?,
        ])?,
        10_000,
    )?;

    // 10 prihod
    let revenue: i64 = match &spec.revenue {
        RevenueModel::PerGuest { check_cents, .. } => c::narrow(c::mul(
            served,
            c::div_round(c::mul(w(*check_cents), level)?, 100)?,
        )?)?,
        RevenueModel::Fixed {
            monthly_cents,
            per_guest_night_cents,
        } => c::narrow(c::add(
            w(prorate(*monthly_cents, p)?),
            c::mul(w(i.guest_nights), w(*per_guest_night_cents))?,
        )?)?,
        RevenueModel::ShareOfRooms { bp } => c::narrow(c::div_round(
            c::mul(w(i.rooms_revenue_cents), w(*bp))?,
            10_000,
        )?)?,
        RevenueModel::Rooms | RevenueModel::None => 0,
    };

    // 11 troškovi
    let cost_of_sales: i64 = c::narrow(c::div_round(
        c::mul(w(revenue), w(spec.cost_of_sales_bp))?,
        10_000,
    )?)?;
    let other_direct: i64 = c::narrow(c::mul(served, w(spec.direct_per_use_cents))?)?;
    let payroll_cents = payroll(staff, spec.wage_cents, i.wage_pct, p)?;

    // 12 rezultat
    let profit = c::sub(
        c::sub(c::sub(w(revenue), w(payroll_cents))?, w(cost_of_sales))?,
        w(other_direct),
    )?;

    // 13 stanje prostora
    let lvl = &spec.maintenance[level_index(pl.maintenance)];
    let wear_pts: i32 = c::narrow(w(prorate(
        c::narrow(c::sub(0, w(spec.wear_pts_per_month))?)?,
        p,
    )?))?;
    let load_wear_pts: i32 = if load > 180 {
        c::narrow(w(prorate(-100, p)?))?
    } else {
        0
    };
    let maintenance_pts: i32 = c::narrow(w(prorate(i64::from(lvl.equipment_pts), p)?))?;
    let maintenance_cents = prorate(lvl.cost_cents, p)?;
    let condition_before = w(st.condition_pts);
    let condition_after: i32 = c::narrow(
        c::sum(&[
            condition_before,
            w(wear_pts),
            w(load_wear_pts),
            w(maintenance_pts),
        ])?
        .clamp(0, 10_000),
    )?;

    // 14 vrijednost za novac
    let value: i32 = c::narrow(c::sub(7_000, c::mul(100, c::sub(level, 100)?)?)?.clamp(0, 10_000))?;

    // 15 zadovoljstvo odjelom
    let target: i32 = c::narrow(c::div_round(
        c::sum(&[
            c::mul(4_500, quality)?,
            c::mul(3_000, w(condition_after))?,
            c::mul(2_500, w(value))?,
        ])?,
        10_000,
    )?)?;
    let frozen = !open
        || spec.class == UsaliClass::Rentals
        || matches!(spec.revenue, RevenueModel::ShareOfRooms { .. });
    let sat_after: i32 = if frozen {
        st.satisfaction_pts
    } else {
        c::narrow(
            c::add(
                sat_before,
                c::div_round(
                    c::mul(w(i.satisfaction_step_bp), c::sub(w(target), sat_before)?)?,
                    10_000,
                )?,
            )?
            .clamp(0, 10_000),
        )?
    };

    // 16 težina
    let guest_weight: u32 = if is_per_guest && open {
        c::narrow(served)?
    } else {
        0
    };

    Ok(DeptOutcome {
        code: i.code.clone(),
        class: spec.class,
        open,
        staff,
        capacity_uses: c::narrow(capacity)?,
        demand_uses: c::narrow(demand)?,
        served_uses: c::narrow(served)?,
        usage_bp: c::narrow(usage)?,
        load_hundredths,
        load_penalty_pts: penalty,
        quality_pts: c::narrow(quality)?,
        price_level_pct: pl.price_level_pct,
        revenue_cents: revenue,
        payroll_cents,
        cost_of_sales_cents: cost_of_sales,
        other_direct_cents: other_direct,
        profit_cents: c::narrow(profit)?,
        condition_before_pts: st.condition_pts,
        wear_pts,
        load_wear_pts,
        maintenance_pts,
        condition_after_pts: condition_after,
        maintenance_cents,
        value_pts: value,
        satisfaction_before_pts: st.satisfaction_pts,
        satisfaction_target_pts: target,
        satisfaction_after_pts: sat_after,
        guest_weight,
        departures: 0,
    })
}

/// Neraspoređeni odjel: plaće + stalni dio + po prodanoj noći + troškovi odluka koje mu pripadaju.
/// `code` i `kind` (iz `spec.class`) ulaze u ishod; razred koji nije neraspoređen je greška koda/kataloga.
#[allow(clippy::too_many_arguments)]
pub fn undistributed_step(
    code: &DeptCode,
    spec: &DeptSpec,
    state: &DeptState,
    fixed_monthly_cents: i64,
    p: &Period,
    sold: u32,
    wage_pct: u16,
    assigned: AssignedCosts,
) -> Calc<UndistributedOutcome> {
    let UsaliClass::Undistributed(kind) = spec.class else {
        return Err(CalcError::BadDivisor);
    };
    let staff: u16 = c::narrow(c::add(w(state.permanent), w(state.seasonal))?)?;
    let payroll_cents = payroll(staff, spec.wage_cents, wage_pct, p)?;
    let fixed_cents = prorate(fixed_monthly_cents, p)?;
    let per_room_cents: i64 = c::narrow(c::mul(w(sold), w(spec.per_occupied_room_cents))?)?;
    let total = c::sum(&[
        w(payroll_cents),
        w(fixed_cents),
        w(per_room_cents),
        w(assigned.marketing_cents),
        w(assigned.reply_cents),
        w(assigned.maintenance_cents),
        w(assigned.training_cents),
        w(assigned.hiring_cents),
        w(assigned.situation_cents),
    ])?;
    Ok(UndistributedOutcome {
        code: code.clone(),
        kind,
        staff,
        payroll_cents,
        fixed_cents,
        per_room_cents,
        assigned,
        total_cents: c::narrow(total)?,
        departures: 0,
    })
}

/// Raspodjela hotelskih odlazaka kruga po odjelima prema broju ljudi odjela (metoda najvećeg ostatka): odjel dobiva
/// cijeli dio kvote `ukupno·ljudi/svi`, a preostali odlasci idu odjelima s najvećim ostatkom; jednaki ostaci →
/// manji kod. Zbroj je točno `total`; odjel bez ljudi dobiva 0. Bez slučajnosti, pa ne mijenja nijedan tok kocke.
pub fn split_departures(
    total: u16,
    staff: &BTreeMap<DeptCode, u16>,
) -> Calc<BTreeMap<DeptCode, u16>> {
    let all = staff.values().try_fold(0_i128, |a, n| c::add(a, w(*n)))?;
    let mut out: BTreeMap<DeptCode, u16> = staff.keys().map(|k| (k.clone(), 0)).collect();
    if total == 0 {
        return Ok(out);
    }
    if all == 0 {
        return Err(CalcError::BadDivisor);
    }
    let mut given: i128 = 0;
    let mut remainders: Vec<(i128, &DeptCode)> = Vec::with_capacity(staff.len());
    for (k, n) in staff.iter().filter(|(_, n)| **n > 0) {
        let share = c::mul(w(total), w(*n))?;
        let whole = c::div_floor(share, all)?;
        given = c::add(given, whole)?;
        out.insert(k.clone(), c::narrow(whole)?);
        remainders.push((c::sub(share, c::mul(whole, all)?)?, k));
    }
    // Najveći ostatak prvi; `sort_by_key` je stabilan, a popis ide po kodu, pa jednaki ostaci ostaju redom koda.
    remainders.sort_by_key(|r| std::cmp::Reverse(r.0));
    let left: usize = c::narrow(c::sub(w(total), given)?)?;
    for (_, k) in remainders.into_iter().take(left) {
        if let Some(v) = out.get_mut(k) {
            *v = v.checked_add(1).ok_or(CalcError::Overflow)?;
        }
    }
    Ok(out)
}

/// Hotelski odlasci kruga po odjelima kroz cijelu sezonu (`so_far` = odlasci odjela u dosadašnjim krugovima).
/// Zbroj odlazaka sezone do sada, uključujući ovaj krug (`T`), dijeli se po ljudima odjela u ovom krugu
/// (`split_departures`), a odjel u krugu dobiva `kvota − već dodijeljeno` (najmanje 0, najviše koliko ljudi ima).
/// Ako zbroj tada nije `total`: višak se oduzima odjelu s najvećim viškom prema točnom udjelu
/// (`(dodijeljeno + krug)·svi − T·ljudi`), manjak se dodaje odjelu s najvećim manjkom; jednako → manji kod.
/// Zbroj je uvijek `total` (`total` ≤ svi ljudi, obračun ga stišće). Bez slučajnosti, kao `split_departures`;
/// tako i jedan tjedni odlazak s vremenom stigne do svakog odjela s ljudima, ne uvijek najvećem.
pub fn split_season_departures(
    total: u16,
    staff: &BTreeMap<DeptCode, u16>,
    so_far: &BTreeMap<DeptCode, u16>,
) -> Calc<BTreeMap<DeptCode, u16>> {
    let mut out: BTreeMap<DeptCode, u16> = staff.keys().map(|k| (k.clone(), 0)).collect();
    if total == 0 {
        return Ok(out);
    }
    let done = |k: &DeptCode| w(so_far.get(k).copied().unwrap_or(0));
    let all = staff.values().try_fold(0_i128, |a, n| c::add(a, w(*n)))?;
    let before = staff.keys().try_fold(0_i128, |a, k| c::add(a, done(k)))?;
    let season_total = c::add(before, w(total))?;
    let quota = split_departures(c::narrow(season_total)?, staff)?;
    for (k, n) in staff {
        let q = w(quota.get(k).copied().unwrap_or(0));
        let give = c::sub(q, done(k))?.max(0).min(w(*n));
        out.insert(k.clone(), c::narrow(give)?);
    }
    // Manjak odjela prema točnom udjelu sezone, u jedinicama 1/svi (negativno = višak).
    let gap = |k: &DeptCode, n: u16, given: u16| -> Calc<i128> {
        c::sub(
            c::mul(season_total, w(n))?,
            c::mul(c::add(done(k), w(given))?, all)?,
        )
    };
    let sum = |m: &BTreeMap<DeptCode, u16>| m.values().try_fold(0_i128, |a, v| c::add(a, w(*v)));
    let goal = w(total);
    while sum(&out)? != goal {
        let surplus = sum(&out)? > goal;
        // BTreeMap ide po kodu rastuće, pa strogo bolji ključ čuva manji kod pri izjednačenju.
        let mut best: Option<(&DeptCode, i128)> = None;
        for (k, n) in staff {
            let given = out.get(k).copied().unwrap_or(0);
            let movable = if surplus { given > 0 } else { given < *n };
            if !movable {
                continue;
            }
            let g = gap(k, *n, given)?;
            let key = if surplus { c::sub(0, g)? } else { g };
            if best.is_none_or(|(_, b)| key > b) {
                best = Some((k, key));
            }
        }
        let (k, _) = best.ok_or(CalcError::BadDivisor)?;
        let v = out.get_mut(k).ok_or(CalcError::BadDivisor)?;
        *v = if surplus {
            v.checked_sub(1)
        } else {
            v.checked_add(1)
        }
        .ok_or(CalcError::Overflow)?;
    }
    Ok(out)
}

/// Raspodjela ukupnog broja sezonaca po odjelima. Zatvoreni odjel treba proslijediti s `max` 0 (ili bez zapisa u
/// `max`): funkcija ne zna za zatvaranje.
/// Kreni od `current` (do `max`; odjel bez `max` ima najviše 0);
/// dok je zbroj manji od cilja i postoji odjel ispod `max`: +1 odjelu s najvećim `plan − trenutno` (jednaki:
/// manji kod); dok je veći: −1 odjelu s najvećim `trenutno − plan` među onima s više od 0 (jednaki: manji kod).
pub fn allocate_seasonal(
    current: &BTreeMap<DeptCode, u16>,
    plan: &BTreeMap<DeptCode, u16>,
    max: &BTreeMap<DeptCode, u16>,
    total_target: u16,
) -> BTreeMap<DeptCode, u16> {
    let cap = |k: &DeptCode| max.get(k).copied().unwrap_or(0);
    let mut out: BTreeMap<DeptCode, u16> = BTreeMap::new();
    for k in current.keys().chain(max.keys()) {
        let cur = current.get(k).copied().unwrap_or(0);
        out.insert(k.clone(), cur.min(cap(k)));
    }
    let wanted = |k: &DeptCode| i32::from(plan.get(k).copied().unwrap_or(0));
    let total = |m: &BTreeMap<DeptCode, u16>| {
        m.values()
            .fold(0_u32, |a, v| a.saturating_add(u32::from(*v)))
    };
    let goal = u32::from(total_target);
    while total(&out) < goal {
        // BTreeMap ide po kodu rastuće, pa strogo veći jaz čuva manji kod pri izjednačenju.
        let mut best: Option<(&DeptCode, i32)> = None;
        for (k, v) in &out {
            if *v >= cap(k) {
                continue;
            }
            let gap = wanted(k).saturating_sub(i32::from(*v));
            if best.is_none_or(|(_, g)| gap > g) {
                best = Some((k, gap));
            }
        }
        let Some((k, _)) = best else { break };
        let k = k.clone();
        if let Some(v) = out.get_mut(&k) {
            *v = v.saturating_add(1);
        }
    }
    while total(&out) > goal {
        let mut best: Option<(&DeptCode, i32)> = None;
        for (k, v) in &out {
            if *v == 0 {
                continue;
            }
            let surplus = i32::from(*v).saturating_sub(wanted(k));
            if best.is_none_or(|(_, s)| surplus > s) {
                best = Some((k, surplus));
            }
        }
        let Some((k, _)) = best else { break };
        let k = k.clone();
        if let Some(v) = out.get_mut(&k) {
            *v = v.saturating_sub(1);
        }
    }
    out
}
