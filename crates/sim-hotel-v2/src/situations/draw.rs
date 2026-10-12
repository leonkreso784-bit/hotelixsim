//! Izvlačenje situacija kruga za jedan hotel (plan situacija §4). Redom: kolektivna (isti primjerak za sve
//! hotele), veća (samo u krajnostima, najviše jedna mjesečno), pa obične do broja kruga. Sva slučajnost ide kroz
//! tok `v2:situacije`; kolektivna koristi utor 0, ostalo utor hotela.

use std::collections::{BTreeMap, BTreeSet};

use engine::calc::CalcError;
use engine::{Rng, label};

use super::DrawnSituation;
use super::count::situation_count;
use super::eligible::{DrawView, Facts, dept_facts, major_side, repeat_allows, weight};
use super::fill::fill;
use super::template::{Kind, Scope, SituationSpec, TemplateSet};
use crate::calendar::{Period, season_periods};
use crate::catalog::Catalog;
use crate::department::DeptCode;
use crate::resolve::ResolveError;
use crate::streams::{Purpose, stream};

/// Utor kolektivnih situacija: isti za sve hotele igre.
const COLLECTIVE_SLOT: u8 = 0;
/// Prilika barem jednom u ovoliko uzastopnih krugova (kad je dopuštena).
const OPPORTUNITY_EVERY: u8 = 3;

/// Što izvući: krug, predlošci igre, povijest hotela, sjeme i utor.
#[derive(Clone, Copy, Debug)]
pub struct Draw<'a> {
    pub period: &'a Period,
    pub templates: &'a TemplateSet,
    /// Primjerci ovog hotela iz prijašnjih krugova sezone (i kolektivni).
    pub history: &'a [DrawnSituation],
    pub seed: u64,
    /// Utor hotela u igri.
    pub slot: u8,
    /// Sidra kruga (S1–S4) koja se broje u broj situacija.
    pub anchors: u8,
}

/// Mogućnosti predloška: odjel (ili cijeli hotel, `None`) s težinom u izvlačenju.
type Options = Vec<(Option<DeptCode>, u64)>;

/// Dopušteni predložak i njegove mogućnosti.
struct Candidate<'a> {
    spec: &'a SituationSpec,
    options: Options,
}

fn best(options: &[(Option<DeptCode>, u64)]) -> u64 {
    options.iter().map(|(_, w)| *w).max().unwrap_or(0)
}

/// Indeks po težinama (`None` kad je zbroj 0).
fn pick(rng: &mut Rng, weights: &[u64]) -> Result<Option<usize>, ResolveError> {
    let total = weights
        .iter()
        .try_fold(0_u64, |a, w| a.checked_add(*w))
        .ok_or(CalcError::Overflow)?;
    if total == 0 {
        return Ok(None);
    }
    let mut r = rng.range(0, total);
    for (i, w) in weights.iter().enumerate() {
        if r < *w {
            return Ok(Some(i));
        }
        r = r.saturating_sub(*w);
    }
    Ok(None)
}

fn choose(
    rng: &mut Rng,
    options: &[(Option<DeptCode>, u64)],
) -> Result<Option<DeptCode>, ResolveError> {
    let weights: Vec<u64> = options.iter().map(|(_, w)| *w).collect();
    let i = pick(rng, &weights)?.ok_or(CalcError::BadDivisor)?;
    Ok(options.get(i).and_then(|(d, _)| d.clone()))
}

/// Dopušteni predlošci za hotel: uvjeti pojave (po odjelu za skupine), otvoren odjel, ponavljanje.
fn candidates<'a>(
    specs: impl Iterator<Item = &'a SituationSpec>,
    view: &DrawView,
    facts: &Facts,
    d: &Draw,
    cat: &Catalog,
) -> Result<Vec<Candidate<'a>>, ResolveError> {
    let mut out = Vec::new();
    for spec in specs.filter(|t| repeat_allows(&t.repeat, &t.code, d.period, d.history)) {
        let mut options = Vec::new();
        match spec.department.members(cat) {
            None => options.extend(weight(spec, facts, None)?.map(|w| (None, w))),
            Some(depts) => {
                for code in depts {
                    let Some(df) = dept_facts(view, &code) else {
                        continue;
                    };
                    options.extend(weight(spec, facts, Some(&df))?.map(|w| (Some(code), w)));
                }
            }
        }
        options.retain(|(_, w)| *w > 0);
        if !options.is_empty() {
            out.push(Candidate { spec, options });
        }
    }
    Ok(out)
}

/// Kolektivni predlošci: samo mjesec i ponavljanje (sadržaj im ne dopušta uvjete stanja), odjel iz kataloga, pa
/// je izbor isti za svaki hotel.
fn collective<'a>(
    set: &'a TemplateSet,
    facts: &Facts,
    d: &Draw,
    cat: &Catalog,
) -> Result<Vec<Candidate<'a>>, ResolveError> {
    let mut out = Vec::new();
    for spec in set.templates.iter().filter(|t| {
        t.scope == Scope::All
            && t.major.is_none()
            && repeat_allows(&t.repeat, &t.code, d.period, d.history)
    }) {
        let Some(w) = weight(spec, facts, None)?.filter(|w| *w > 0) else {
            continue;
        };
        let options = match spec.department.members(cat) {
            None => vec![(None, w)],
            Some(depts) => depts.into_iter().map(|c| (Some(c), w)).collect(),
        };
        if !options.is_empty() {
            out.push(Candidate { spec, options });
        }
    }
    Ok(out)
}

/// Prvi krug koji počinje u mjesecu u kojem počinje i ovaj.
fn first_round_of_month(p: &Period) -> bool {
    season_periods(p.kind)
        .into_iter()
        .find(|q| q.start_month() == p.start_month())
        == Some(*p)
}

/// Najviše situacija iz istog odjela u krugu: 2 u tjednu, razmjerno više u dužem krugu (⌈2n/5⌉).
fn dept_limit(n: u8) -> usize {
    usize::from(n.saturating_mul(2).div_ceil(5).max(2))
}

/// Primjerci kruga, poredani po važnosti (4 prva; jednaki redom izvlačenja). Može ih biti manje od broja kruga
/// kad dopuštenih predložaka nema dovoljno.
pub fn draw_situations(
    view: &DrawView,
    d: &Draw,
    cat: &Catalog,
) -> Result<Vec<DrawnSituation>, ResolveError> {
    let p = d.period;
    let n = situation_count(view.state, p, cat)?;
    let slots = usize::from(n.saturating_sub(d.anchors));
    let facts = Facts::new(view, p, cat)?;
    let names = &d.templates.names;
    let mut used = BTreeSet::new();
    let mut out: Vec<DrawnSituation> = Vec::new();
    let month = p.start_month();
    let in_month = |pred: &dyn Fn(&DrawnSituation) -> bool| {
        d.history
            .iter()
            .filter(|h| h.period.start_month() == month && pred(h))
            .count()
    };

    // Kolektivna: prvi krug mjeseca, najviše jedna mjesečno (🟨), isti tok i utor za sve hotele.
    if slots > 0 && first_round_of_month(p) && in_month(&|h| h.scope == Scope::All) == 0 {
        let pool = collective(d.templates, &facts, d, cat)?;
        let mut rng = stream(
            d.seed,
            Purpose::Situations,
            p.index,
            COLLECTIVE_SLOT,
            label("svi"),
        );
        let weights: Vec<u64> = pool.iter().map(|c| best(&c.options)).collect();
        if let Some(c) = pick(&mut rng, &weights)?.and_then(|i| pool.get(i)) {
            let dept = choose(&mut rng, &c.options)?;
            out.push(fill(
                c.spec,
                dept,
                p,
                d.seed,
                COLLECTIVE_SLOT,
                names,
                &mut used,
            )?);
        }
    }

    // Veća: prosjek pokazatelja u krajnosti, najviše `max_per_month` u mjesecu.
    let major_quota = usize::from(d.templates.major.max_per_month);
    if out.len() < slots
        && in_month(&|h| h.major.is_some()) < major_quota
        && let Some(side) = major_side(&d.templates.major, view.indices)?
    {
        let specs = d
            .templates
            .templates
            .iter()
            .filter(|t| t.scope == Scope::Hotel && t.major == Some(side));
        let pool = candidates(specs, view, &facts, d, cat)?;
        let mut rng = stream(d.seed, Purpose::Situations, p.index, d.slot, label("veca"));
        let weights: Vec<u64> = pool.iter().map(|c| best(&c.options)).collect();
        if let Some(c) = pick(&mut rng, &weights)?.and_then(|i| pool.get(i)) {
            let dept = choose(&mut rng, &c.options)?;
            out.push(fill(c.spec, dept, p, d.seed, d.slot, names, &mut used)?);
        }
    }

    // Obične do broja kruga.
    let specs = d
        .templates
        .templates
        .iter()
        .filter(|t| t.scope == Scope::Hotel && t.major.is_none());
    let mut pool = candidates(specs, view, &facts, d, cat)?;
    let mut rng = stream(d.seed, Purpose::Situations, p.index, d.slot, label("izbor"));
    // Prošla dva kruga bez prilike → u ovom treba jedna (ako je koja dopuštena).
    let window = OPPORTUNITY_EVERY.saturating_sub(1);
    let since = p.index.saturating_sub(window);
    let need_opportunity = p.index >= window
        && !d.history.iter().any(|h| {
            h.kind == Kind::Opportunity && h.period.index >= since && h.period.index < p.index
        });
    let limit = dept_limit(n);
    while out.len() < slots {
        let mut per_dept: BTreeMap<&DeptCode, usize> = BTreeMap::new();
        for s in &out {
            if let Some(code) = &s.department {
                let e = per_dept.entry(code).or_default();
                *e = e.saturating_add(1);
            }
        }
        // Granica po odjelu je stroga: radije manje situacija nego tri iz istog odjela.
        let allowed: Vec<(usize, Options)> = pool
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let opts: Options = c
                    .options
                    .iter()
                    .filter(|(dept, _)| {
                        dept.as_ref()
                            .is_none_or(|x| per_dept.get(x).copied().unwrap_or(0) < limit)
                    })
                    .cloned()
                    .collect();
                (i, opts)
            })
            .filter(|(_, opts)| !opts.is_empty())
            .collect();
        if allowed.is_empty() {
            break;
        }
        // Raznolikost vrsta (meka): prilika kad je dugo nije bilo, a zadnje mjesto druge vrste od dosadašnjih.
        let kinds: BTreeSet<Kind> = out.iter().map(|s| s.kind).collect();
        let kind_of = |i: usize| pool.get(i).map(|c| c.spec.kind);
        let has =
            |f: &dyn Fn(Kind) -> bool| allowed.iter().any(|(i, _)| kind_of(*i).is_some_and(f));
        let keep: Box<dyn Fn(Kind) -> bool> = if need_opportunity
            && !kinds.contains(&Kind::Opportunity)
            && has(&|k| k == Kind::Opportunity)
        {
            Box::new(|k| k == Kind::Opportunity)
        } else if out.len().saturating_add(1) == slots
            && kinds.len() == 1
            && has(&|k| !kinds.contains(&k))
        {
            Box::new(|k| !kinds.contains(&k))
        } else {
            Box::new(|_| true)
        };
        let chosen: Vec<&(usize, Options)> = allowed
            .iter()
            .filter(|(i, _)| kind_of(*i).is_some_and(&keep))
            .collect();
        let weights: Vec<u64> = chosen.iter().map(|(_, o)| best(o)).collect();
        let Some((i, opts)) = pick(&mut rng, &weights)?
            .and_then(|k| chosen.get(k))
            .copied()
        else {
            break;
        };
        let dept = choose(&mut rng, opts)?;
        let c = pool.remove(*i);
        out.push(fill(c.spec, dept, p, d.seed, d.slot, names, &mut used)?);
    }

    out.sort_by_key(|s| std::cmp::Reverse(s.weight));
    Ok(out)
}
