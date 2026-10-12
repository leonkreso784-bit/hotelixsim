//! Što izvlačenje vidi na početku kruga (stanje hotela i odjela, prošli krug, pokazatelji) i kako iz toga slijede
//! uvjeti pojave, težina u izvlačenju i pravila ponavljanja jednog predloška.

use engine::calc::{self as c, CalcError};
use sim_hotel::{IndexSet, Month};

use super::DrawnSituation;
use super::count::expected_occupancy_bp;
use super::template::{Bounds, Conditions, MajorRule, MajorSide, Repeat, SituationSpec};
use crate::calendar::Period;
use crate::catalog::Catalog;
use crate::department::DeptCode;
use crate::resolve::{PeriodReport, ResolveError};
use crate::state::HotelState;

pub(super) const WEEK_DAYS: u16 = 7;

/// Stanje hotela kako ga vidi izvlačenje na početku kruga.
#[derive(Clone, Copy, Debug)]
pub struct DrawView<'a> {
    pub state: &'a HotelState,
    /// Obračun prošlog kruga; `None` u prvom krugu.
    pub previous: Option<&'a PeriodReport>,
    /// Pet pokazatelja na početku kruga (za veću situaciju).
    pub indices: &'a IndexSet,
}

/// Hotelske činjenice za uvjete, izračunate jednom po krugu.
pub(super) struct Facts {
    month: Month,
    /// Popunjenost prošlog kruga; u prvom krugu očekivana popunjenost ovog (J-1).
    occupancy_bp: i64,
    morale_pts: i64,
    cash_cents: i64,
    group_accepted: bool,
}

/// Činjenice jednog otvorenog odjela.
pub(super) struct DeptFacts {
    condition_pts: i64,
    satisfaction_pts: i64,
    /// Opterećenje prošlog kruga; u prvom krugu 0.
    load_hundredths: i64,
    staff: i64,
    departures: i64,
}

impl Facts {
    pub(super) fn new(view: &DrawView, p: &Period, cat: &Catalog) -> Result<Facts, ResolveError> {
        let occupancy_bp = match view.previous {
            Some(r) => {
                let nights = c::mul(i128::from(r.before.rooms), i128::from(r.period.days))?;
                if nights > 0 {
                    c::narrow(c::div_round(
                        c::mul(i128::from(r.summary.market.sold), 10_000)?,
                        nights,
                    )?)?
                } else {
                    0
                }
            }
            None => i64::from(expected_occupancy_bp(view.state, p, cat)?),
        };
        Ok(Facts {
            month: p.start_month(),
            occupancy_bp,
            morale_pts: i64::from(view.state.morale_pts),
            cash_cents: view.state.cash_cents,
            group_accepted: !view.state.groups.is_empty(),
        })
    }
}

/// Činjenice odjela ako postoji u hotelu i otvoren je.
pub(super) fn dept_facts(view: &DrawView, code: &DeptCode) -> Option<DeptFacts> {
    let d = view.state.departments.get(code).filter(|d| d.open)?;
    let load = view
        .previous
        .and_then(|r| r.accounts.departments.get(code))
        .map_or(0, |o| o.load_hundredths);
    Some(DeptFacts {
        condition_pts: i64::from(d.condition_pts),
        satisfaction_pts: i64::from(d.satisfaction_pts),
        load_hundredths: i64::from(load),
        staff: i64::from(d.permanent.saturating_add(d.seasonal)),
        departures: i64::from(d.season_departures),
    })
}

fn within(b: &Option<Bounds>, v: i64) -> bool {
    b.as_ref().is_none_or(|b| b.contains(v))
}

/// Uvjet nad odjelom bez odjela ne vrijedi (sadržaj ga i ne dopušta).
fn within_dept(b: &Option<Bounds>, d: Option<&DeptFacts>, f: fn(&DeptFacts) -> i64) -> bool {
    match (b, d) {
        (None, _) => true,
        (Some(b), Some(d)) => b.contains(f(d)),
        (Some(_), None) => false,
    }
}

pub(super) fn holds(c: &Conditions, h: &Facts, d: Option<&DeptFacts>) -> bool {
    (c.months.is_empty() || c.months.contains(&h.month))
        && within(&c.occupancy_bp, h.occupancy_bp)
        && within(&c.morale_pts, h.morale_pts)
        && within(&c.cash_cents, h.cash_cents)
        && c.group_accepted.is_none_or(|g| g == h.group_accepted)
        && within_dept(&c.dept_condition_pts, d, |d| d.condition_pts)
        && within_dept(&c.dept_satisfaction_pts, d, |d| d.satisfaction_pts)
        && within_dept(&c.dept_load_hundredths, d, |d| d.load_hundredths)
        && within_dept(&c.dept_staff, d, |d| d.staff)
        && within_dept(&c.dept_departures, d, |d| d.departures)
}

/// Težina u izvlačenju: `base` × pojačanja koja vrijede × važnost; `None` kad uvjeti pojave ne vrijede.
pub(super) fn weight(
    t: &SituationSpec,
    h: &Facts,
    d: Option<&DeptFacts>,
) -> Result<Option<u64>, ResolveError> {
    if !holds(&t.when, h, d) {
        return Ok(None);
    }
    let mut w = u64::from(t.chance.base)
        .checked_mul(u64::from(t.weight))
        .ok_or(CalcError::Overflow)?;
    for b in t.chance.boost.iter().filter(|b| holds(&b.when, h, d)) {
        w = w
            .checked_mul(u64::from(b.times))
            .ok_or(CalcError::Overflow)?;
    }
    Ok(Some(w))
}

/// Najmanji razmak između dva pojavljivanja istog predloška u tjednima (Leon): vrijedi i kad predložak traži manje.
const MIN_GAP_WEEKS: u8 = 4;

/// `max_per_season` i razmak od svih ranijih pojava (`earlier`: povijest i već izvučeni primjerci ovog kruga)
/// u danima sezone: barem `max(gap_rounds, 4)` tjedana. Isto mjerilo za tjednu i mjesečnu igru (u mjesečnoj je
/// dan primjerka početak njegova virtualnog tjedna).
pub(super) fn repeat_allows<'h>(
    r: &Repeat,
    code: &str,
    day: u16,
    earlier: impl Iterator<Item = &'h DrawnSituation>,
) -> bool {
    let past: Vec<&DrawnSituation> = earlier.filter(|h| h.template == code).collect();
    if past.len() >= usize::from(r.max_per_season) {
        return false;
    }
    let gap = u16::from(r.gap_rounds.max(MIN_GAP_WEEKS)).saturating_mul(WEEK_DAYS);
    past.iter()
        .all(|h| h.day < day && day.saturating_sub(h.day) >= gap)
}

/// Strana veće situacije za ovaj krug: prosjek pet pokazatelja na početku kruga prema pravilu.
pub(super) fn major_side(
    rule: &MajorRule,
    indices: &IndexSet,
) -> Result<Option<MajorSide>, ResolveError> {
    let avg = c::div_floor(i128::from(indices.total_hundredths()), 5)?;
    let high = c::mul(i128::from(rule.high_avg_min), 100)?;
    let low = c::mul(i128::from(rule.low_avg_max), 100)?;
    Ok(if avg >= high {
        Some(MajorSide::High)
    } else if avg <= low {
        Some(MajorSide::Low)
    } else {
        None
    })
}
