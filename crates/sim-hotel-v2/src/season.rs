//! Sezona jednog hotela (krugovi su tjedni ili mjeseci): plan i odgovori na situacije dolaze od igrača ili robota.
//! Jedina imenovana iznimka od pravila „slučajnost samo kroz `streams::stream`": generator plana robota i izbora
//! odgovora izvodi se kao u v1 (`robot:plan`, kod situacije), da mjesečna sezona daje iste nizove kao v1.

use engine::calc::CalcError;
use engine::{Rng, label};
use sim_hotel::{
    IndexSet, Month, Profile, SituationOutcome, SituationTemplate, Strategy, progress_pts,
};

use crate::calendar::{Period, PeriodKind, first_period_of_month, season_periods};
use crate::catalog::Catalog;
use crate::indices::{RefTable, period_indices, start_indices};
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_period};
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

/// Situacije čiji mjesec počinje u ovom razdoblju (razdoblje koje sadrži 1. dan mjeseca; 🟨 P13).
pub fn situations_for<'a>(p: &Period, all: &'a [SituationTemplate]) -> Vec<&'a SituationTemplate> {
    all.iter()
        .filter(|t| first_period_of_month(p.kind, t.month) == *p)
        .collect()
}

pub fn run_season(
    profile: Profile,
    strategy: Strategy,
    seed: u64,
    kind: PeriodKind,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
) -> Result<SeasonResult, ResolveError> {
    let category = cat.base.profile(profile)?.category;
    run_season_with(
        profile,
        kind,
        cat,
        situations,
        refs,
        &mut |s, p| {
            let mut rng = Rng::derive(seed, &[u64::from(p.index), label("robot:plan")]);
            robot_plan(strategy, s, p, cat, &mut rng)
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
    kind: PeriodKind,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>,
) -> Result<SeasonResult, ResolveError> {
    let mut state = HotelState::from_profile(cat.base.profile(profile)?)?;
    let start = start_indices(&state, cat, refs)?;
    let all = season_periods(kind);
    let mut periods: Vec<(PeriodReport, IndexSet)> = Vec::with_capacity(all.len());
    for p in &all {
        let plan = planner(&state, p)?;
        let mut outcomes = Vec::new();
        for t in situations_for(p, situations) {
            let i = chooser(t)?;
            let answer = t.answers.get(i).ok_or(ResolveError::BadChoice)?;
            outcomes.push(SituationOutcome::from_answer(&t.code, answer));
        }
        let report = resolve_period(&state, p, &plan, &outcomes, cat)?;
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
