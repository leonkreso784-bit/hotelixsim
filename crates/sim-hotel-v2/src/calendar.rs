//! Kalendar sezone (MODEL_V2 §4): krug je tjedan ili mjesec; dan 0 = 1. lipnja, sezona ima 122 dana.
//! Tjedni: 16 × 7 dana + zadnje razdoblje od 10 dana (21.–30. 9.) — pretpostavka 🟨 P1 (MODEL_V2 §9.2 N3),
//! potvrđena u ADR-028.

use engine::calc::{self as c, Calc};
use sim_hotel::{Month, MonthTable};

pub const SEASON_DAYS: u16 = 122;
const FULL_WEEKS: u8 = 16;
const WEEK_DAYS: u8 = 7;
const LAST_DAYS: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PeriodKind {
    Month,
    Week,
}

impl PeriodKind {
    pub const ALL: [PeriodKind; 2] = [PeriodKind::Month, PeriodKind::Week];

    /// Stabilan kod (JSON, API); nikad se ne mijenja.
    pub fn code(self) -> &'static str {
        match self {
            PeriodKind::Month => "month",
            PeriodKind::Week => "week",
        }
    }

    pub fn from_code(s: &str) -> Option<PeriodKind> {
        match s {
            "month" => Some(PeriodKind::Month),
            "week" => Some(PeriodKind::Week),
            _ => None,
        }
    }
}

/// Jedan krug igre: `index` od 0, `start_day` od 0 (= 1. lipnja).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Period {
    pub kind: PeriodKind,
    pub index: u8,
    pub start_day: u16,
    pub days: u8,
}

/// Dio razdoblja unutar jednog mjeseca; `from_day` od 0 unutar mjeseca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonthPart {
    pub month: Month,
    pub from_day: u8,
    pub days: u8,
}

fn month_start(m: Month) -> u16 {
    match m {
        Month::Jun => 0,
        Month::Jul => 30,
        Month::Aug => 61,
        Month::Sep => 92,
    }
}

fn month_index(m: Month) -> u8 {
    match m {
        Month::Jun => 0,
        Month::Jul => 1,
        Month::Aug => 2,
        Month::Sep => 3,
    }
}

/// Mjesec u kojem je dan sezone (dan izvan sezone pada u prvi ili zadnji mjesec).
fn month_of_day(day: u16) -> Month {
    Month::SEASON
        .iter()
        .rev()
        .copied()
        .find(|&m| month_start(m) <= day)
        .unwrap_or(Month::Jun)
}

pub fn month_period(m: Month) -> Period {
    Period {
        kind: PeriodKind::Month,
        index: month_index(m),
        start_day: month_start(m),
        days: m.days(),
    }
}

pub fn season_periods(kind: PeriodKind) -> Vec<Period> {
    match kind {
        PeriodKind::Month => Month::SEASON.iter().map(|&m| month_period(m)).collect(),
        PeriodKind::Week => {
            let mut out = Vec::with_capacity(usize::from(FULL_WEEKS.saturating_add(1)));
            let mut start: u16 = 0;
            for index in 0..FULL_WEEKS {
                out.push(Period {
                    kind,
                    index,
                    start_day: start,
                    days: WEEK_DAYS,
                });
                start = start.saturating_add(u16::from(WEEK_DAYS));
            }
            out.push(Period {
                kind,
                index: FULL_WEEKS,
                start_day: start,
                days: LAST_DAYS,
            });
            out
        }
    }
}

impl Period {
    /// Prvi dan nakon razdoblja (isključen).
    fn end_day(&self) -> u16 {
        self.start_day.saturating_add(u16::from(self.days))
    }

    /// Stabilan kod kruga: mjesec `"jun"`…, tjedan `"t01"`…`"t17"` (`index + 1`).
    pub fn code(&self) -> String {
        match self.kind {
            PeriodKind::Month => self.start_month().code().to_string(),
            PeriodKind::Week => format!("t{:02}", self.index.saturating_add(1)),
        }
    }

    /// Dijelovi razdoblja po mjesecima, redom (jedan ili dva).
    pub fn parts(&self) -> Vec<MonthPart> {
        let (start, end) = (self.start_day, self.end_day());
        Month::SEASON
            .iter()
            .filter_map(|&m| {
                let ms = month_start(m);
                let me = ms.saturating_add(u16::from(m.days()));
                let (lo, hi) = (start.max(ms), end.min(me));
                if lo >= hi {
                    return None;
                }
                Some(MonthPart {
                    month: m,
                    from_day: u8::try_from(lo.saturating_sub(ms)).ok()?,
                    days: u8::try_from(hi.saturating_sub(lo)).ok()?,
                })
            })
            .collect()
    }

    pub fn start_month(&self) -> Month {
        month_of_day(self.start_day)
    }

    /// D₀: broj dana mjeseca u kojem razdoblje počinje.
    pub fn month_days_at_start(&self) -> u8 {
        self.start_month().days()
    }

    /// Zadnji dan razdoblja je zadnji dan mjeseca.
    pub fn ends_month(&self) -> bool {
        self.parts().last().is_some_and(part_ends_month)
    }

    /// Mjesec čiji je zadnji dan unutar ovog razdoblja (najviše jedan, jer je krug kraći od mjeseca).
    pub fn month_ending_in(&self) -> Option<Month> {
        self.parts()
            .into_iter()
            .find(part_ends_month)
            .map(|part| part.month)
    }

    /// Razdoblje je točno jedan cijeli kalendarski mjesec (svaki mjesečni krug; nijedan tjedni).
    pub fn is_whole_month(&self) -> bool {
        matches!(self.parts().as_slice(), [part] if part.from_day == 0 && part_ends_month(part))
    }

    /// Mjesec čiji je prvi dan unutar ovog razdoblja (najviše jedan). To je mjesec situacija koje krug nosi
    /// (`season::situations_for`, 🟨 P13), pa se s njim obračunavaju njihovi učinci.
    pub fn month_starting_in(&self) -> Option<Month> {
        self.parts()
            .into_iter()
            .find(|part| part.from_day == 0)
            .map(|part| part.month)
    }

    /// Prvi dan razdoblja: (dan u mjesecu od 1, mjesec).
    pub fn first_date(&self) -> (u8, Month) {
        match self.parts().first() {
            Some(part) => (part.from_day.saturating_add(1), part.month),
            None => (1, self.start_month()),
        }
    }

    /// Zadnji dan razdoblja: (dan u mjesecu od 1, mjesec).
    pub fn last_date(&self) -> (u8, Month) {
        match self.parts().last() {
            Some(part) => (part.from_day.saturating_add(part.days), part.month),
            None => (1, self.start_month()),
        }
    }
}

fn part_ends_month(part: &MonthPart) -> bool {
    part.from_day.saturating_add(part.days) == part.month.days()
}

/// Krugovi iste vrste od ovog (uključivo) do kraja mjeseca u kojem krug počinje (tjedan koji prelazi mjesec pripada
/// mjesecu u kojem počinje): lipanj t01 → 5, srpanj t06 → 4; mjesečni krug → 1.
pub fn rounds_left_in_month(p: &Period) -> u8 {
    let n = season_periods(p.kind)
        .iter()
        .filter(|q| q.index >= p.index && q.start_month() == p.start_month())
        .count();
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// Krug je zadnji krug svog mjeseca (plaća ostatak obnove).
pub fn is_last_round_of_month(p: &Period) -> bool {
    rounds_left_in_month(p) == 1
}

pub fn period_by_code(kind: PeriodKind, code: &str) -> Option<Period> {
    season_periods(kind).into_iter().find(|p| p.code() == code)
}

pub fn period_containing(kind: PeriodKind, day: u16) -> Option<Period> {
    season_periods(kind)
        .into_iter()
        .find(|p| p.start_day <= day && day < p.end_day())
}

/// Razdoblje koje sadrži 1. dan mjeseca.
pub fn first_period_of_month(kind: PeriodKind, m: Month) -> Period {
    // Kalendar pokriva sve dane sezone, pa je zamjena (cijeli mjesec) nedostižna.
    period_containing(kind, month_start(m)).unwrap_or_else(|| month_period(m))
}

/// Razmjerni dio mjesečnog iznosa za dio mjeseca: `⌊M·(od+dana)/D⌋ − ⌊M·od/D⌋`. Svi dijelovi jednog mjeseca
/// zajedno daju točno `M` (i za negativne iznose, jer je `div_floor` euklidski).
pub fn prorate_part(monthly: i64, part: &MonthPart) -> Calc<i64> {
    let d = i128::from(part.month.days());
    let from = i128::from(part.from_day);
    let to = c::add(from, i128::from(part.days))?;
    let m = i128::from(monthly);
    c::narrow(c::sub(
        c::div_floor(c::mul(m, to)?, d)?,
        c::div_floor(c::mul(m, from)?, d)?,
    )?)
}

/// Razmjerni dio mjesečnog iznosa za cijelo razdoblje (zbroj po dijelovima mjeseci).
pub fn prorate(monthly: i64, p: &Period) -> Calc<i64> {
    let mut total: i128 = 0;
    for part in p.parts() {
        total = c::add(total, i128::from(prorate_part(monthly, &part)?))?;
    }
    c::narrow(total)
}

/// Prosjek mjesečne tablice po danima razdoblja, zaokružen (`div_round`).
fn day_weighted<T: Copy + Into<i128>>(t: &MonthTable<T>, p: &Period) -> Calc<i128> {
    let mut total: i128 = 0;
    for part in p.parts() {
        let v: i128 = t.get(part.month).into();
        total = c::add(total, c::mul(v, i128::from(part.days))?)?;
    }
    c::div_round(total, i128::from(p.days))
}

pub fn avg_i64(t: &MonthTable<i64>, p: &Period) -> Calc<i64> {
    c::narrow(day_weighted(t, p)?)
}

pub fn avg_u16(t: &MonthTable<u16>, p: &Period) -> Calc<u16> {
    c::narrow(day_weighted(t, p)?)
}

/// Iznos razdoblja preračunat na mjesec u kojem razdoblje počinje: `div_round(x·D₀, p.days)`.
pub fn per_month_equiv(x: i64, p: &Period) -> Calc<i64> {
    c::narrow(c::mul_div(
        i128::from(x),
        i128::from(p.month_days_at_start()),
        i128::from(p.days),
    )?)
}
