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
    Anchor, Draw, DrawView, DrawnSituation, Kind, RoundSituation, TemplateSet, draw_collective,
    draw_situations_with, reward_only,
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

/// Hotel u izvlačenju kruga: pogled na stanje, njegova povijest primjeraka i utor.
#[derive(Clone, Copy)]
pub struct RoundHotel<'a> {
    pub view: DrawView<'a>,
    pub history: &'a [DrawnSituation],
    pub slot: u8,
}

/// Ulazi kruga isti za sve hotele: sidra kruga, njihove vrste i mjeseci grupa koje sidra nude do ovog kruga
/// uključivo (S3 u prvom krugu lipnja).
fn round_inputs<'a>(
    anchors: &'a [SituationTemplate],
    p: &Period,
) -> (Vec<&'a SituationTemplate>, Vec<Anchor>, Vec<Month>) {
    let here = situations_for(p, anchors);
    let kinds: Vec<Anchor> = here.iter().map(|t| anchor_of(t)).collect();
    let groups: Vec<Month> = anchors
        .iter()
        .filter(|t| first_period_of_month(p.kind, t.month).index <= p.index)
        .filter_map(group_month)
        .collect();
    (here, kinds, groups)
}

/// Kolektivna situacija kruga, jedna za sve hotele (`situations::draw_collective`): ulaz su svi hoteli igre, pa
/// ne probija ni broj kruga ni granicu grupa nijednog od njih. Sezona jednog hotela zove ovo s jednim hotelom.
pub fn collective_for_round(
    anchors: &[SituationTemplate],
    templates: &TemplateSet,
    seed: u64,
    p: &Period,
    hotels: &[RoundHotel],
    cat: &Catalog,
) -> Result<Option<DrawnSituation>, ResolveError> {
    let (_, kinds, groups) = round_inputs(anchors, p);
    let list: Vec<(DrawView, Draw)> = hotels
        .iter()
        .map(|h| {
            let draw = Draw {
                period: p,
                templates,
                history: h.history,
                seed,
                slot: h.slot,
                anchors: &kinds,
                anchor_groups: &groups,
            };
            (h.view, draw)
        })
        .collect();
    draw_collective(&list, cat)
}

/// Primjerci generatora za krug `p` i sidra kruga uz zadanu kolektivnu (`collective_for_round`, jedna za cijelu
/// igru); generator vidi stanje na početku kruga, obračun i pokazatelje prošlog kruga i sve ranije primjerke ovog
/// hotela. Isti poziv koristi sezona robota i igra (`game`), pa isto sjeme i isto stanje daju iste primjerke.
pub fn draw_for_round_with<'a>(
    anchors: &'a [SituationTemplate],
    templates: &TemplateSet,
    (seed, slot): (u64, u8),
    p: &Period,
    (view, history): (&DrawView, &[DrawnSituation]),
    (cat, shared): (&Catalog, Option<&DrawnSituation>),
) -> Result<(Vec<&'a SituationTemplate>, Vec<DrawnSituation>), ResolveError> {
    let (anchors_here, kinds, groups) = round_inputs(anchors, p);
    let draw = Draw {
        period: p,
        templates,
        history,
        seed,
        slot,
        anchors: &kinds,
        anchor_groups: &groups,
    };
    Ok((
        anchors_here,
        draw_situations_with(view, &draw, cat, shared)?,
    ))
}

/// Primjerci generatora za krug `p` jednog hotela; kolektivnu izvlači za taj hotel sam (sezona robota).
pub fn draw_for_round<'a>(
    anchors: &'a [SituationTemplate],
    templates: &TemplateSet,
    (seed, slot): (u64, u8),
    p: &Period,
    view: &DrawView,
    history: &[DrawnSituation],
    cat: &Catalog,
) -> Result<(Vec<&'a SituationTemplate>, Vec<DrawnSituation>), ResolveError> {
    let hotel = RoundHotel {
        view: *view,
        history,
        slot,
    };
    let shared = collective_for_round(anchors, templates, seed, p, &[hotel], cat)?;
    draw_for_round_with(
        anchors,
        templates,
        (seed, slot),
        p,
        (view, history),
        (cat, shared.as_ref()),
    )
}

/// Primjerci generatora za krug `p` (sezona robota, `TemplateDraw`).
fn draw_round<'a>(
    d: &TemplateDraw<'a>,
    p: &Period,
    view: &DrawView,
    history: &[DrawnSituation],
    cat: &Catalog,
) -> Result<(Vec<&'a SituationTemplate>, Vec<DrawnSituation>), ResolveError> {
    draw_for_round(
        d.anchors,
        d.templates,
        (d.seed, d.slot),
        p,
        view,
        history,
        cat,
    )
}

/// Situacija kruga za odgovarača sezone; `drawn` je primjerak generatora (važnost određuje poredak u igri), sidro ga nema.
#[derive(Clone, Copy)]
pub struct RoundItem<'a> {
    pub template: &'a SituationTemplate,
    pub drawn: Option<&'a DrawnSituation>,
}

/// Ishodi kruga, jedan po stavci i istim redom; „samo nagradu" (S-4) primjenjuje jezgra.
pub type Responder<'r> =
    dyn FnMut(&Period, &[RoundItem]) -> Result<Vec<SituationOutcome>, ResolveError> + 'r;

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
    let mut responder = |_: &Period, items: &[RoundItem]| {
        items
            .iter()
            .map(|item| {
                let t = item.template;
                let i = chooser(t)?;
                let answer = t.answers.get(i).ok_or(ResolveError::BadChoice)?;
                Ok(SituationOutcome::from_answer(&t.code, answer))
            })
            .collect()
    };
    run_season_responding(profile, kind, cat, source, refs, planner, &mut responder)
}

/// Sezona s vlastitim planerom i odgovaračem koji za cijeli krug vraća ishode (može i prepuštanje, šutnju).
pub fn run_season_responding(
    profile: Profile,
    kind: PeriodKind,
    cat: &Catalog,
    source: SeasonSituations,
    refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, &Period) -> Result<PeriodPlan, ResolveError>,
    responder: &mut Responder,
) -> Result<SeasonResult, ResolveError> {
    let mut state = HotelState::from_profile(cat, profile)?;
    let start = start_indices(&state, cat, refs)?;
    let all = season_periods(kind);
    let mut periods: Vec<(PeriodReport, IndexSet)> = Vec::with_capacity(all.len());
    let mut drawn: Vec<Vec<DrawnSituation>> = Vec::new();
    let mut history: Vec<DrawnSituation> = Vec::new();
    for p in &all {
        let plan = planner(&state, p)?;
        // Primjerci kruga i oznaka „samo nagrada" njihova predloška (S-4).
        let mut instances: Vec<(SituationTemplate, bool)> = Vec::new();
        let mut has_drawn = false;
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
                    instances.push(((d.instance)(s)?, d.templates.no_penalty(&s.template)));
                }
                history.extend(found.iter().cloned());
                drawn.push(found);
                has_drawn = true;
                anchors
            }
        };
        let round_drawn: &[DrawnSituation] = match drawn.last() {
            Some(found) if has_drawn => found,
            _ => &[],
        };
        let mut items: Vec<RoundItem> = fixed
            .into_iter()
            .map(|template| RoundItem {
                template,
                drawn: None,
            })
            .collect();
        let anchors_len = items.len();
        items.extend(
            instances
                .iter()
                .zip(round_drawn)
                .map(|((template, _), d)| RoundItem {
                    template,
                    drawn: Some(d),
                }),
        );
        let answers = responder(p, &items)?;
        if answers.len() != items.len() {
            return Err(ResolveError::BadChoice);
        }
        // Prostor ishoda je odjel primjerka (ADR-042 t. 1); sidra su cijeli hotel.
        let outcomes: Vec<RoundSituation> = answers
            .into_iter()
            .zip(&items)
            .enumerate()
            .map(|(k, (o, item))| {
                let no_penalty = k
                    .checked_sub(anchors_len)
                    .and_then(|j| instances.get(j))
                    .is_some_and(|(_, n)| *n);
                RoundSituation {
                    outcome: if no_penalty { reward_only(o) } else { o },
                    space: item.drawn.and_then(|d| d.department.clone()),
                }
            })
            .collect();
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
