//! Sezona jednog hotela (krugovi su tjedni ili mjeseci): plan i odgovori na situacije dolaze od igrača ili robota.
//! Situacije kruga dolaze iz starog popisa (S1–S4 po mjesecu, P13) ili, kad pravila igre imaju predloške, od
//! generatora (`situations::draw_situations`, J-3) uz S1–S4 kao sidra.
//! Jedina imenovana iznimka od pravila „slučajnost samo kroz `streams::stream`": generator plana robota i izbora
//! odgovora izvodi se kao u v1 (`robot:plan`, kod situacije), da mjesečna sezona daje iste nizove kao v1.

use engine::calc::CalcError;
use engine::{Rng, label};
use sim_hotel::{
    IndexSet, Mechanic, Month, Profile, SituationOutcome, SituationTemplate, Strategy, progress_pts,
};

use crate::calendar::{Period, PeriodKind, first_period_of_month, season_periods};
use crate::catalog::Catalog;
use crate::indices::{RefTable, period_indices, start_indices};
use crate::plan::PeriodPlan;
use crate::resolve::{PeriodReport, ResolveError, resolve_period};
use crate::robots::robot_plan;
use crate::situations::{
    Anchor, Draw, DrawView, DrawnSituation, Kind, TemplateSet, draw_situations,
};
use crate::state::HotelState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonResult {
    pub start: IndexSet,
    pub periods: Vec<(PeriodReport, IndexSet)>,
    pub progress_pts: i32,
    /// Primjerci generatora po krugu, redom `periods`; prazno kad sezona igra stari popis situacija.
    pub drawn: Vec<Vec<DrawnSituation>>,
}

/// Odakle krug dobiva situacije.
#[derive(Clone, Copy)]
pub enum SeasonSituations<'a> {
    /// Stari popis: situacije mjeseca u krugu koji sadrži 1. dan mjeseca (`situations_for`, P13).
    Fixed(&'a [SituationTemplate]),
    /// Predlošci: stari popis ostaje kao sidra u svojim krugovima, ostatak do broja kruga izvlači generator.
    Drawn(&'a TemplateDraw<'a>),
}

/// Predlošci za izvlačenje u sezoni jednog hotela.
pub struct TemplateDraw<'a> {
    /// Stari popis (S1–S4): sidra kruga, broje se u broj situacija.
    pub anchors: &'a [SituationTemplate],
    pub templates: &'a TemplateSet,
    pub seed: u64,
    /// Utor hotela u igri (0 je kolektivni).
    pub slot: u8,
    /// Primjerak kao situacija s ponuđenim odgovorima (iznosi za kategoriju hotela i mjesec kruga); odgovore zna
    /// sadržaj, ne jezgra.
    pub instance: &'a dyn Fn(&DrawnSituation) -> Result<SituationTemplate, ResolveError>,
}

/// Sidro stare situacije za generator: vrsta po prvoj mehanici prvog odgovora (oporavak gosta → pritužba,
/// zaposlenik, grupa → zahtjev, kriza; 🟨 J-3), cijeli hotel.
pub fn anchor_of(t: &SituationTemplate) -> Anchor {
    let first = t
        .answers
        .first()
        .and_then(|a| a.mechanics.first())
        .or(t.unanswered.first());
    let kind = match first {
        Some(Mechanic::ServiceRecovery { .. }) | None => Kind::Complaint,
        Some(Mechanic::EquipmentRepair { .. }) => Kind::Breakdown,
        Some(Mechanic::StaffRetention { .. }) => Kind::Staff,
        Some(Mechanic::GroupBooking { .. }) => Kind::Request,
        Some(Mechanic::CrisisCommunication { .. }) => Kind::Crisis,
    };
    Anchor {
        kind,
        department: None,
    }
}

/// Mjesec dolaska grupe koju stara situacija nudi (prvi odgovor koji prihvaća grupu); `None` = ne nudi grupu.
/// Broji se u granice grupa.
pub fn group_month(t: &SituationTemplate) -> Option<Month> {
    t.answers
        .iter()
        .flat_map(|a| &a.mechanics)
        .find_map(|m| match m {
            Mechanic::GroupBooking {
                accept: true,
                month,
                ..
            } => Some(*month),
            _ => None,
        })
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
    run_robot_season(
        profile,
        strategy,
        seed,
        kind,
        cat,
        SeasonSituations::Fixed(situations),
        refs,
    )
}

/// Sezona robota: plan i izbor odgovora kao `run_season`, situacije iz `source` (stari popis ili predlošci).
pub fn run_robot_season(
    profile: Profile,
    strategy: Strategy,
    seed: u64,
    kind: PeriodKind,
    cat: &Catalog,
    source: SeasonSituations,
    refs: &RefTable,
) -> Result<SeasonResult, ResolveError> {
    let category = cat.base.profile(profile)?.category;
    run_season_from(
        profile,
        kind,
        cat,
        source,
        refs,
        &mut |s, p| {
            let mut rng = Rng::derive(seed, &[u64::from(p.index), label("robot:plan")]);
            robot_plan(strategy, s, p, cat, &mut rng)
        },
        &mut |t| {
            // Imenovana iznimka (v1 formula) vrijedi i za primjerke predložaka: šifra primjerka je jedinstvena u
            // igri (`P-X.t03`, ponovljeni u krugu `P-X.aug.2`) i nikad nije šifra stare situacije (S1–S4), pa svaki
            // izbor dobiva svoj niz.
            let mut rng = Rng::derive(seed, &[position(t.month)?, label(&t.code)]);
            Ok(strategy.choose(&t.answers, &cat.base, category, &mut rng)?)
        },
    )
}

/// Sezona s vlastitim planerom i izborom odgovora nad starim popisom situacija; slučajnost, ako je ima, nose
/// same funkcije.
pub fn run_season_with(
    profile: Profile,
    kind: PeriodKind,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>,
) -> Result<SeasonResult, ResolveError> {
    run_season_from(
        profile,
        kind,
        cat,
        SeasonSituations::Fixed(situations),
        refs,
        planner,
        chooser,
    )
}

/// Primjerci generatora za krug `p` i sidra kruga; generator vidi stanje na početku kruga, obračun i pokazatelje
/// prošlog kruga i sve ranije primjerke ovog hotela.
fn draw_round<'a>(
    d: &TemplateDraw<'a>,
    p: &Period,
    view: &DrawView,
    history: &[DrawnSituation],
    cat: &Catalog,
) -> Result<(Vec<&'a SituationTemplate>, Vec<DrawnSituation>), ResolveError> {
    let anchors_here = situations_for(p, d.anchors);
    let anchors: Vec<Anchor> = anchors_here.iter().map(|t| anchor_of(t)).collect();
    // Sidra koja nude grupu do ovog kruga uključivo (S3 u prvom krugu lipnja).
    let anchor_groups: Vec<Month> = d
        .anchors
        .iter()
        .filter(|t| first_period_of_month(p.kind, t.month).index <= p.index)
        .filter_map(group_month)
        .collect();
    let draw = Draw {
        period: p,
        templates: d.templates,
        history,
        seed: d.seed,
        slot: d.slot,
        anchors: &anchors,
        anchor_groups: &anchor_groups,
    };
    Ok((anchors_here, draw_situations(view, &draw, cat)?))
}

/// Sezona s vlastitim planerom i izborom odgovora; situacije iz `source`.
pub fn run_season_from(
    profile: Profile,
    kind: PeriodKind,
    cat: &Catalog,
    source: SeasonSituations,
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>,
) -> Result<SeasonResult, ResolveError> {
    let mut state = HotelState::from_profile(cat, profile)?;
    let start = start_indices(&state, cat, refs)?;
    let all = season_periods(kind);
    let mut periods: Vec<(PeriodReport, IndexSet)> = Vec::with_capacity(all.len());
    let mut drawn: Vec<Vec<DrawnSituation>> = Vec::new();
    let mut history: Vec<DrawnSituation> = Vec::new();
    for p in &all {
        let plan = planner(&state, p)?;
        let mut instances: Vec<SituationTemplate> = Vec::new();
        let fixed = match source {
            SeasonSituations::Fixed(list) => situations_for(p, list),
            SeasonSituations::Drawn(d) => {
                let previous = periods.last();
                let view = DrawView {
                    state: &state,
                    previous: previous.map(|(r, _)| r),
                    indices: previous.map_or(&start, |(_, i)| i),
                };
                let (anchors, found) = draw_round(d, p, &view, &history, cat)?;
                for s in &found {
                    instances.push((d.instance)(s)?);
                }
                history.extend(found.iter().cloned());
                drawn.push(found);
                anchors
            }
        };
        let mut outcomes = Vec::new();
        for t in fixed.into_iter().chain(&instances) {
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
        drawn,
    })
}
