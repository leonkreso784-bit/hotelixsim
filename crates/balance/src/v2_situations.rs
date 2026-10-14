//! Situacije iz predložaka u balansu v2 (J-3): sezona robota izvlači primjerke istim generatorom kao igra kad
//! sadržaj ima predloške (bez njih stari popis S1–S4), pa udio situacija u napretku, broj situacija po krugu,
//! granica kruga i raznolikost predložaka.

use std::collections::BTreeMap;

use sim_hotel_v2::situations::{DrawnSituation, TemplateSet};
use sim_hotel_v2::{
    Catalog, CatalogError, Cause, Period, PeriodKind, PeriodPlan, Profile, RefTable, ResolveError,
    SeasonResult, SeasonSituations, SituationTemplate, Strategy, TemplateDraw, Var, group_month,
    run_robot_season, run_season_from, season_periods, situations_for,
};

use crate::median;

/// Utor hotela robota u balansu (0 je kolektivni).
pub const HOTEL_SLOT: u8 = 1;

/// Leon (plan situacija §6, P8): situacije zajedno ~⅓ napretka; pojas provjere 20–40 %.
pub(crate) const SHARE_BAND_BP: (i32, i32) = (2_000, 4_000);
/// Granica kruga iz plana situacija §6 (±bodova); za ispis koliko krugova je udari.
pub(crate) const ROUND_BOUND_PTS: i64 = 800;
/// Najveći udio jednog predloška u svim pojavama (provjera d).
pub(crate) const DOMINANT_MAX_BP: i32 = 1_500;
/// Strategije čiji se učinak situacija mjeri: Brižnom udio u pojasu, Štediši smjer i veličina (Leon 13.10.).
pub(crate) const SHARE_STRATEGIES: [Strategy; 2] = [Strategy::Saver, Strategy::Caring];
/// Štediši situacije smanjuju napredak, najviše za 6 bodova (stotinke).
pub(crate) const SAVER_MAX_LOSS: i32 = 600;

/// Situacije sezone: predlošci iz sadržaja (u jezgrenom obliku) ili stari popis.
pub(crate) struct Situations<'a> {
    content: &'a content::Content,
    set: Option<TemplateSet>,
}

type Planner<'p> =
    dyn FnMut(&sim_hotel_v2::HotelState, &Period) -> Result<PeriodPlan, ResolveError> + 'p;
type Chooser<'c> = dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError> + 'c;

impl<'a> Situations<'a> {
    pub(crate) fn new(content: &'a content::Content) -> Situations<'a> {
        Situations {
            content,
            set: content.situation_templates.as_ref().map(|t| t.to_core()),
        }
    }

    pub(crate) fn has_templates(&self) -> bool {
        self.set.is_some()
    }

    pub(crate) fn group_limit(&self) -> Option<u8> {
        self.set.as_ref().and_then(|s| s.group_limit)
    }

    /// Nudi li primjerak predloška `code` grupu.
    fn template_offers_group(&self, code: &str) -> bool {
        self.set.as_ref().is_some_and(|s| {
            s.templates
                .iter()
                .any(|t| t.code == code && t.group_month.is_some())
        })
    }

    /// Stari popis (S1–S4).
    pub(crate) fn fixed(&self) -> &'a [SituationTemplate] {
        &self.content.situations
    }

    fn with_source<T>(
        &self,
        profile: Profile,
        seed: u64,
        cat: &Catalog,
        run: &mut dyn FnMut(SeasonSituations) -> Result<T, ResolveError>,
    ) -> Result<T, ResolveError> {
        let (Some(tpl), Some(set)) = (&self.content.situation_templates, &self.set) else {
            return run(SeasonSituations::Fixed(self.fixed()));
        };
        let category = cat.base.profile(profile)?.category;
        let instance = |d: &DrawnSituation| {
            tpl.instance(d, &cat.base, category)
                .map_err(|e| ResolveError::Catalog(CatalogError::OutOfRange(format!("{e:?}"))))
        };
        let draw = TemplateDraw {
            anchors: self.fixed(),
            templates: set,
            seed,
            slot: HOTEL_SLOT,
            instance: &instance,
        };
        run(SeasonSituations::Drawn(&draw))
    }

    /// Sezona robota sa situacijama (generator kad ih sadržaj ima).
    pub(crate) fn robot(
        &self,
        (profile, strategy, seed): (Profile, Strategy, u64),
        kind: PeriodKind,
        cat: &Catalog,
        refs: &RefTable,
    ) -> Result<SeasonResult, ResolveError> {
        self.with_source(profile, seed, cat, &mut |src| {
            run_robot_season(profile, strategy, seed, kind, cat, src, refs)
        })
    }

    /// Sezona s vlastitim planerom i izborom; primjerke izvlači sjeme `seed`.
    pub(crate) fn with(
        &self,
        (profile, seed): (Profile, u64),
        kind: PeriodKind,
        cat: &Catalog,
        refs: &RefTable,
        planner: &mut Planner,
        chooser: &mut Chooser,
    ) -> Result<SeasonResult, ResolveError> {
        self.with_source(profile, seed, cat, &mut |src| {
            run_season_from(profile, kind, cat, src, refs, planner, chooser)
        })
    }
}

/// Što sezone sa situacijama pokazuju o broju, granici i raznolikosti (sve strategije, svi profili, sva sjemena).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DrawStats {
    /// Krugova ukupno.
    pub rounds: u64,
    /// Najmanje i najviše situacija u krugu (sidra uključena) i zbroj za prosjek.
    pub min: u64,
    pub max: u64,
    pub total: u64,
    /// Krugovi čiji zbroj situacija udari u granicu kruga (|Σ| ≥ 800): zadovoljstvo, moral.
    pub bound_satisfaction: u64,
    pub bound_morale: u64,
    /// Pojave po predlošku (bez sidara).
    pub templates: BTreeMap<String, u64>,
    /// Najviše ponuda grupe u jednoj sezoni (sidra uključena).
    pub groups_max: u64,
    /// Broj sezona po broju ponuda grupe (sidra uključena).
    pub groups_seasons: BTreeMap<u64, u64>,
}

impl DrawStats {
    /// Dodaj jednu sezonu.
    pub(crate) fn add(&mut self, r: &SeasonResult, sits: &Situations) {
        let fixed = sits.fixed();
        let mut groups: u64 = 0;
        for ((report, _), drawn) in r.periods.iter().zip(&r.drawn) {
            let here = situations_for(&report.period, fixed);
            let g = here
                .iter()
                .filter(|t| group_month(t).is_some())
                .count()
                .saturating_add(
                    drawn
                        .iter()
                        .filter(|s| sits.template_offers_group(&s.template))
                        .count(),
                );
            groups = groups.saturating_add(u64::try_from(g).unwrap_or(u64::MAX));
            let n = u64::try_from(here.len())
                .unwrap_or(u64::MAX)
                .saturating_add(u64::try_from(drawn.len()).unwrap_or(u64::MAX));
            self.min = if self.rounds == 0 { n } else { self.min.min(n) };
            self.max = self.max.max(n);
            self.total = self.total.saturating_add(n);
            self.rounds = self.rounds.saturating_add(1);
            let sum = |var: Var| -> i64 {
                report
                    .summary
                    .lines
                    .iter()
                    .filter(|l| l.var == var && matches!(l.cause, Cause::Situation(_)))
                    .map(|l| l.amount)
                    .sum()
            };
            if sum(Var::Satisfaction).abs() >= ROUND_BOUND_PTS {
                self.bound_satisfaction = self.bound_satisfaction.saturating_add(1);
            }
            if sum(Var::Morale).abs() >= ROUND_BOUND_PTS {
                self.bound_morale = self.bound_morale.saturating_add(1);
            }
            for s in drawn {
                let e = self.templates.entry(s.template.clone()).or_default();
                *e = e.saturating_add(1);
            }
        }
        self.groups_max = self.groups_max.max(groups);
        let e = self.groups_seasons.entry(groups).or_default();
        *e = e.saturating_add(1);
    }

    /// Predložak s najviše pojava i njegov udio u svim pojavama (bp).
    pub fn top(&self) -> Option<(String, i32)> {
        let all: u64 = self.templates.values().sum();
        let (code, n) = self.templates.iter().max_by_key(|(_, n)| **n)?;
        let bp = n.saturating_mul(10_000).checked_div(all).unwrap_or(0);
        Some((code.clone(), i32::try_from(bp).unwrap_or(i32::MAX)))
    }
}

/// Udio situacija u napretku jedne strategije na jednom profilu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Share {
    pub profile: Profile,
    pub strategy: Strategy,
    /// Medijan napretka sa situacijama i bez ijedne situacije (isto sjeme), stotinke boda.
    pub with: i32,
    pub without: i32,
    /// Medijan razlike po sjemenu (sa − bez).
    pub delta: i32,
    /// |Δ| / (|Δ| + |bez|) iz medijana, bp — koliki dio puta od početka do kraja sezone nose situacije.
    pub share_bp: i32,
}

/// `|Δ| / (|Δ| + |bez|)` u bp; bez ikakvog pomaka 0.
pub(crate) fn share_bp(delta: i32, without: i32) -> i32 {
    let d = i64::from(delta).abs();
    let whole = d.saturating_add(i64::from(without).abs());
    i32::try_from(d.saturating_mul(10_000).checked_div(whole).unwrap_or(0)).unwrap_or(i32::MAX)
}

/// Udio situacija za Štedišu i Brižnog na svakom profilu: ista sjemena sa situacijama i bez njih.
pub(crate) fn shares(
    sits: &Situations,
    kind: PeriodKind,
    cat: &Catalog,
    refs_for: &dyn Fn(Profile) -> Result<RefTable, ResolveError>,
    seeds: u64,
) -> Result<Vec<Share>, ResolveError> {
    let mut out = Vec::new();
    for profile in Profile::ALL {
        let refs = refs_for(profile)?;
        for strategy in SHARE_STRATEGIES {
            let (mut with, mut without, mut delta) = (Vec::new(), Vec::new(), Vec::new());
            for seed in 0..seeds {
                let a = sits.robot((profile, strategy, seed), kind, cat, &refs)?;
                let b = run_robot_season(
                    profile,
                    strategy,
                    seed,
                    kind,
                    cat,
                    SeasonSituations::Fixed(&[]),
                    &refs,
                )?;
                with.push(a.progress_pts);
                without.push(b.progress_pts);
                delta.push(a.progress_pts.saturating_sub(b.progress_pts));
            }
            let (w, wo, d) = (median(&mut with), median(&mut without), median(&mut delta));
            out.push(Share {
                profile,
                strategy,
                with: w,
                without: wo,
                delta: d,
                share_bp: share_bp(d, wo),
            });
        }
    }
    Ok(out)
}

/// Najmanje i najviše situacija koje krug smije imati: tjedno 3–5, mjesečno zbroj tjednih (13–22).
pub(crate) fn count_band(kind: PeriodKind) -> (u64, u64) {
    match kind {
        PeriodKind::Week => (3, 5),
        PeriodKind::Month => (13, 22),
    }
}

/// Broj krugova sezone (za ispis prosjeka po sezoni).
pub(crate) fn rounds_per_season(kind: PeriodKind) -> u64 {
    u64::try_from(season_periods(kind).len()).unwrap_or(1)
}
