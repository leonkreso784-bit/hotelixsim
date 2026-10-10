//! Sezona jednog hotela (kopija v1; Task 3 uvodi krugove): plan i odgovori na situacije dolaze od igrača ili
//! robota. Robotski tokovi kocke su v1 (`robot:plan`, kod situacije), da mjesečna kopija daje iste brojke.

use engine::calc::CalcError;
use engine::{Rng, label};
use sim_hotel::{
    IndexSet, Month, Profile, SituationOutcome, SituationTemplate, Strategy, progress_pts,
};

use crate::catalog::Catalog;
use crate::indices::{RefTable, period_indices, start_indices};
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_month};
use crate::robots::robot_plan;
use crate::state::HotelState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonResult {
    pub start: IndexSet,
    pub periods: Vec<(PeriodReport, IndexSet)>,
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
    let category = cat.base.profile(profile)?.category;
    run_season_with(
        profile,
        cat,
        situations,
        refs,
        &mut |s, m| {
            let mut rng = Rng::derive(seed, &[position(m)?, label("robot:plan")]);
            robot_plan(strategy, s, m, cat, &mut rng)
        },
        &mut |t| {
            let mut rng = Rng::derive(seed, &[position(t.month)?, label(&t.code)]);
            Ok(strategy.choose(&t.answers, &cat.base, category, &mut rng)?)
        },
    )
}

/// Sezona s vlastitim planerom i izborom odgovora; slučajnost, ako je ima, nose same funkcije.
pub fn run_season_with(
    profile: Profile,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, Month) -> Result<PeriodPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>,
) -> Result<SeasonResult, ResolveError> {
    let mut state = HotelState::from_profile(cat.base.profile(profile)?)?;
    let start = start_indices(&state, cat, refs)?;
    let mut periods: Vec<(PeriodReport, IndexSet)> = Vec::with_capacity(Month::SEASON.len());
    for m in Month::SEASON {
        let plan = planner(&state, m)?;
        let mut outcomes = Vec::new();
        for t in situations.iter().filter(|t| t.month == m) {
            let i = chooser(t)?;
            let answer = t.answers.get(i).ok_or(ResolveError::BadChoice)?;
            outcomes.push(SituationOutcome::from_answer(&t.code, answer));
        }
        let report = resolve_month(&state, m, &plan, &outcomes, cat)?;
        let idx = period_indices(&report, cat, refs)?;
        state = report.after.clone();
        periods.push((report, idx));
    }
    let end = periods.last().map_or(&start, |(_, i)| i);
    let progress = progress_pts(&start, end)?;
    Ok(SeasonResult {
        start,
        periods,
        progress_pts: progress,
    })
}
