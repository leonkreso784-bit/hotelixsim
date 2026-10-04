//! Sezona jednog hotela: četiri mjeseca, plan i odgovori na situacije dolaze od igrača ili robota.

use engine::calc::CalcError;
use engine::{Rng, label};

use crate::catalog::Catalog;
use crate::index::IndexSet;
use crate::indices::{RefTable, month_indices, start_indices};
use crate::mechanic::{SituationOutcome, SituationTemplate};
use crate::month::Month;
use crate::plan::MonthPlan;
use crate::profile::Profile;
use crate::resolve::{MonthReport, ResolveError, resolve_month};
use crate::robots::Strategy;
use crate::score::progress_pts;
use crate::state::HotelState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonResult {
    pub start: IndexSet,
    pub months: Vec<(MonthReport, IndexSet)>,
    pub progress_pts: i32,
}

fn position(m: Month) -> Result<u64, ResolveError> {
    u64::try_from(m.position()).map_err(|_| ResolveError::Calc(CalcError::Overflow))
}

pub fn run_season(
    profile: Profile,
    strategy: Strategy,
    seed: u64,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
) -> Result<SeasonResult, ResolveError> {
    let category = cat.profile(profile)?.category;
    run_season_with(
        profile,
        cat,
        situations,
        refs,
        &mut |s, m| {
            let mut rng = Rng::derive(seed, &[position(m)?, label("robot:plan")]);
            strategy.plan(s, m, cat, &mut rng)
        },
        &mut |t| {
            let mut rng = Rng::derive(seed, &[position(t.month)?, label(&t.code)]);
            strategy.choose(&t.answers, cat, category, &mut rng)
        },
    )
}

/// Sezona s vlastitim planerom i izborom odgovora; slučajnost, ako je ima, nose same funkcije.
pub fn run_season_with(
    profile: Profile,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, Month) -> Result<MonthPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>,
) -> Result<SeasonResult, ResolveError> {
    let mut state = HotelState::from_profile(cat.profile(profile)?)?;
    let start = start_indices(&state, cat, refs)?;
    let mut months: Vec<(MonthReport, IndexSet)> = Vec::with_capacity(Month::SEASON.len());
    for m in Month::SEASON {
        let plan = planner(&state, m)?;
        let mut outcomes = Vec::new();
        for t in situations.iter().filter(|t| t.month == m) {
            let i = chooser(t)?;
            let answer = t.answers.get(i).ok_or(ResolveError::BadChoice)?;
            outcomes.push(SituationOutcome::from_answer(&t.code, answer));
        }
        let report = resolve_month(&state, m, &plan, &outcomes, cat)?;
        let idx = month_indices(&report, cat, refs)?;
        state = report.after.clone();
        months.push((report, idx));
    }
    let end = months.last().map_or(&start, |(_, i)| i);
    let progress = progress_pts(&start, end)?;
    Ok(SeasonResult {
        start,
        months,
        progress_pts: progress,
    })
}
