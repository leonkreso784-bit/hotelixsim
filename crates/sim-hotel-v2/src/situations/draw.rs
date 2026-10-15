//! Izvlačenje situacija kruga (plan situacija §4). Kolektivna (isti primjerak za SVE hotele igre) izvlači se jednom
//! po krugu za sve hotele zajedno (`draw_collective`: ulazi su svi hoteli, ne smije probiti granicu nijednog).
//! Zatim svaki hotel: veća (samo u krajnostima, najviše jedna mjesečno), pa obične do broja kruga. Krug se izvlači
//! kao jedan blok i u mjesečnoj igri (ADR-042 t. 13): svi primjerci imaju dan početka kruga, a isti predložak dolazi
//! najviše jednom u krugu. Sva slučajnost ide kroz tok `v2:situacije`; kolektivna koristi utor 0, ostalo utor hotela.

use std::collections::{BTreeMap, BTreeSet};

use engine::calc::CalcError;
use engine::{Rng, label};
use sim_hotel::{CatalogError, Month};

use super::count::{SituationCounts, expected_occupancy_bp};
use super::eligible::{
    DrawView, Facts, WEEK_DAYS, dept_facts, major_side, repeat_allows, space_condition, weight,
};
use super::fill::{FillCtx, Place, fill};
use super::template::{Kind, Scope, SituationSpec, TemplateSet};
use super::{Anchor, DrawnSituation};
use crate::calendar::{Period, season_periods};
use crate::catalog::Catalog;
use crate::department::DeptCode;
use crate::resolve::ResolveError;
use crate::streams::{Purpose, stream};

/// Utor kolektivnih situacija: isti za sve hotele igre.
const COLLECTIVE_SLOT: u8 = 0;

/// Što izvući: krug, predlošci igre, povijest hotela, sjeme, utor i sidra.
#[derive(Clone, Copy, Debug)]
pub struct Draw<'a> {
    pub period: &'a Period,
    pub templates: &'a TemplateSet,
    /// Primjerci OVOG hotela iz prijašnjih krugova sezone i kolektivni (utor 0), svaki jednom. Primjerak drugog
    /// hotela ili ponovljena šifra je greška (`CatalogError::OutOfRange`), ne tiho krivo brojanje.
    pub history: &'a [DrawnSituation],
    pub seed: u64,
    /// Utor hotela u igri.
    pub slot: u8,
    /// Sidra kruga (S1–S4): broje se u broj situacija, u granicu odjela i u vrste kruga.
    pub anchors: &'a [Anchor],
    /// Mjeseci dolaska grupa koje nude sidra, od početka sezone do ovog kruga uključivo: broje se u granice grupa
    /// (sidra nisu u povijesti generatora).
    pub anchor_groups: &'a [Month],
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

/// Smije li doći primjerak koji nudi grupu s dolaskom u `month` (`None` = ne nudi grupu, uvijek smije): granica
/// sezone i granica po mjesecu; broje se sidra, povijest i već izvučeni u ovom krugu, a `reserve` su grupe koje
/// kolektivne još mogu donijeti svim hotelima (mjesta za njih hotel ne smije potrošiti vlastitim primjercima).
fn group_room(d: &Draw, out: &[DrawnSituation], month: Option<Month>, reserve: &[Month]) -> bool {
    let Some(m) = month else {
        return true;
    };
    let set = d.templates;
    let month_of = |code: &str| {
        set.templates
            .iter()
            .find(|t| t.code == code)
            .and_then(|t| t.group_month)
    };
    let months: Vec<Month> = d
        .anchor_groups
        .iter()
        .copied()
        .chain(
            d.history
                .iter()
                .chain(out)
                .filter_map(|h| month_of(&h.template)),
        )
        .collect();
    let season_ok = set
        .group_limit
        .is_none_or(|l| months.len().saturating_add(reserve.len()) < usize::from(l));
    let month_ok = set.groups_per_month.is_none_or(|l| {
        let held = months.iter().chain(reserve).filter(|x| **x == m).count();
        held < usize::from(l)
    });
    season_ok && month_ok
}

/// Najviše grupa koje kolektivne još mogu donijeti NAKON ovog kruga (gornja granica, ista za sve hotele): po
/// predlošku koji nudi grupu preostalo ponavljanje, ne više od prvih krugova mjeseca (u mjesecima pojave) koji tek
/// dolaze. Zato vlastiti primjerci hotela ne popune granicu prije kolektivne i svaki hotel ima mjesta za nju.
fn reserved_groups(d: &Draw, out: &[DrawnSituation]) -> Vec<Month> {
    let p = d.period;
    let future: Vec<Period> = season_periods(p.kind)
        .into_iter()
        .filter(|q| q.index > p.index && first_round_of_month(q))
        .collect();
    let mut reserve = Vec::new();
    for t in &d.templates.templates {
        let (Scope::All, None, Some(month)) = (t.scope, t.major, t.group_month) else {
            continue;
        };
        let used = d
            .history
            .iter()
            .chain(out)
            .filter(|h| h.template == t.code)
            .count();
        let left = usize::from(t.repeat.max_per_season).saturating_sub(used);
        let rounds = future
            .iter()
            .filter(|q| t.when.months.is_empty() || t.when.months.contains(&q.start_month()))
            .count();
        reserve.extend(std::iter::repeat_n(month, left.min(rounds)));
    }
    reserve
}

/// Povijest smije imati samo primjerke ovog hotela i kolektivne, svaku šifru jednom (ugovor s igrom).
fn check_history(d: &Draw) -> Result<(), ResolveError> {
    let mut codes = BTreeSet::new();
    for h in d.history {
        let own = match h.scope {
            Scope::Hotel => h.slot == d.slot,
            Scope::All => h.slot == COLLECTIVE_SLOT,
        };
        if !own || !codes.insert(h.code.as_str()) {
            return Err(CatalogError::OutOfRange(format!(
                "povijest situacija za utor {}: primjerak {} (utor {}) nije ovog hotela ili se ponavlja",
                d.slot, h.code, h.slot
            ))
            .into());
        }
    }
    Ok(())
}

/// Raniji primjerci predloška `code`: povijest i već izvučeni primjerci ovog kruga.
fn earlier<'h>(d: &Draw<'h>, out: &'h [DrawnSituation], code: &str) -> Vec<&'h DrawnSituation> {
    d.history
        .iter()
        .chain(out)
        .filter(|h| h.template == code)
        .collect()
}

/// Broj situacija kruga za hotel s očekivanom popunjenošću `occupancy_bp`: postavka igre ili zadano za vrstu kruga.
fn round_count(d: &Draw, occupancy_bp: u16) -> u8 {
    d.templates
        .counts
        .unwrap_or_else(|| SituationCounts::default_for(d.period.kind))
        .for_occupancy(occupancy_bp)
}

/// Dopušteni predlošci za hotel na dan `day`: uvjeti pojave (po odjelu za skupine), otvoren odjel, ponavljanje
/// prema povijesti; predložak koji je već izvučen u ovom krugu ne ulazi (najviše jednom u krugu, ADR-042 t. 12).
fn candidates<'a>(
    specs: impl Iterator<Item = &'a SituationSpec>,
    view: &DrawView,
    facts: &Facts,
    (d, out, day): (&Draw, &[DrawnSituation], u16),
    cat: &Catalog,
) -> Result<Vec<Candidate<'a>>, ResolveError> {
    let gap = d.templates.min_gap_weeks;
    let mut found = Vec::new();
    for spec in specs.filter(|t| {
        !out.iter().any(|s| s.template == t.code)
            && repeat_allows(&t.repeat, gap, &t.code, day, d.history.iter().chain(out))
    }) {
        let mut options = Vec::new();
        match spec.department.members(cat) {
            None => {
                let space = space_condition(spec, view, None);
                options.extend(weight(spec, facts, None, space)?.map(|w| (None, w)));
            }
            Some(depts) => {
                for code in depts {
                    let Some(df) = dept_facts(view, &code) else {
                        continue;
                    };
                    let space = space_condition(spec, view, Some(&df));
                    options.extend(weight(spec, facts, Some(&df), space)?.map(|w| (Some(code), w)));
                }
            }
        }
        options.retain(|(_, w)| *w > 0);
        if !options.is_empty() {
            found.push(Candidate { spec, options });
        }
    }
    Ok(found)
}

/// Kolektivni predlošci: samo mjesec i ponavljanje (sadržaj im ne dopušta uvjete stanja), odjel iz kataloga i to
/// samo onaj koji se ne može zatvoriti, pa je izbor isti za svaki hotel i nikad na zatvorenom odjelu.
fn collective<'a>(
    set: &'a TemplateSet,
    facts: &Facts,
    d: &Draw,
    cat: &Catalog,
) -> Result<Vec<Candidate<'a>>, ResolveError> {
    let mut found = Vec::new();
    for spec in set.templates.iter().filter(|t| {
        t.scope == Scope::All
            && t.major.is_none()
            && repeat_allows(
                &t.repeat,
                set.min_gap_weeks,
                &t.code,
                d.period.start_day,
                d.history.iter(),
            )
    }) {
        let Some(w) = weight(spec, facts, None, None)?.filter(|w| *w > 0) else {
            continue;
        };
        let options = match spec.department.members(cat) {
            None => vec![(None, w)],
            Some(depts) => {
                let mut open = Vec::new();
                for c in depts {
                    if !cat.dept(&c)?.can_close {
                        open.push((Some(c), w));
                    }
                }
                open
            }
        };
        if !options.is_empty() {
            found.push(Candidate { spec, options });
        }
    }
    Ok(found)
}

/// Prvi krug koji počinje u mjesecu u kojem počinje i ovaj.
fn first_round_of_month(p: &Period) -> bool {
    season_periods(p.kind)
        .into_iter()
        .find(|q| q.start_month() == p.start_month())
        == Some(*p)
}

/// Najviše situacija iz istog odjela u krugu s `n` situacija: ⌈2n/5⌉, barem 2.
fn dept_limit(n: u8) -> usize {
    usize::from(n.saturating_mul(2).div_ceil(5).max(2))
}

/// Situacije po odjelu: sidra i primjerci.
fn per_dept<'x>(
    anchors: &'x [Anchor],
    items: impl Iterator<Item = &'x DrawnSituation>,
) -> BTreeMap<&'x DeptCode, usize> {
    let mut m: BTreeMap<&DeptCode, usize> = BTreeMap::new();
    let depts = anchors
        .iter()
        .filter_map(|a| a.department.as_ref())
        .chain(items.filter_map(|s| s.department.as_ref()));
    for code in depts {
        let e = m.entry(code).or_default();
        *e = e.saturating_add(1);
    }
    m
}

/// Prošlih `every_weeks − 1` tjedana bez prilike → u ovom krugu treba jedna (ako je koja dopuštena). U tjednoj igri
/// isto što i „prošla dva kruga" (zadano 3 tjedna).
fn needs_opportunity<'h>(
    day: u16,
    every_weeks: u8,
    mut earlier: impl Iterator<Item = &'h DrawnSituation>,
) -> bool {
    let window = u16::from(every_weeks.saturating_sub(1)).saturating_mul(WEEK_DAYS);
    let since = day.saturating_sub(window);
    day >= window && !earlier.any(|h| h.kind == Kind::Opportunity && h.day >= since && h.day < day)
}

/// Kolektivna situacija kruga, JEDNOM za sve hotele (`hotels`: pogled i ulaz svakog hotela igre). Izvlači se samo u
/// prvom krugu mjeseca, kad nijedan hotel u tom mjesecu još nije imao kolektivnu, svaki hotel ima slobodno mjesto
/// (broj kruga − sidra > 0), a predložak koji nudi grupu ulazi u izbor samo ako svaki hotel ima mjesta za grupu i
/// ponavljanje ga dopušta svima. Tako je kolektivna ista kod svih i nikome ne probija ni broj kruga ni granicu grupa;
/// izvlači je isti tok (`svi`, utor 0) bez obzira na broj hotela. Sezona robota zove ovo s jednim hotelom.
pub fn draw_collective(
    hotels: &[(DrawView, Draw)],
    cat: &Catalog,
) -> Result<Option<DrawnSituation>, ResolveError> {
    let Some((view, d)) = hotels.first() else {
        return Ok(None);
    };
    let p = d.period;
    if !first_round_of_month(p) {
        return Ok(None);
    }
    let month = p.start_month();
    for (v, h) in hotels {
        check_history(h)?;
        let n = round_count(h, expected_occupancy_bp(v.state, p, cat)?);
        let this_month = h
            .history
            .iter()
            .any(|x| x.scope == Scope::All && x.period.start_month() == month);
        if usize::from(n) <= h.anchors.len() || this_month {
            return Ok(None);
        }
    }
    let facts = Facts::new(view, p, cat)?;
    let mut pool = collective(d.templates, &facts, d, cat)?;
    pool.retain(|c| {
        hotels.iter().all(|(_, h)| {
            repeat_allows(
                &c.spec.repeat,
                h.templates.min_gap_weeks,
                &c.spec.code,
                p.start_day,
                h.history.iter(),
            ) && group_room(h, &[], c.spec.group_month, &[])
        })
    });
    let mut rng = stream(
        d.seed,
        Purpose::Situations,
        p.index,
        COLLECTIVE_SLOT,
        label("svi"),
    );
    let weights: Vec<u64> = pool.iter().map(|c| best(&c.options)).collect();
    let Some(c) = pick(&mut rng, &weights)?.and_then(|i| pool.get(i)) else {
        return Ok(None);
    };
    let department = choose(&mut rng, &c.options)?;
    let ctx = FillCtx {
        p,
        seed: d.seed,
        slot: COLLECTIVE_SLOT,
        names: &d.templates.names,
    };
    let at = Place {
        department,
        day: p.start_day,
        occurrence: 1,
    };
    let s = fill(
        c.spec,
        at,
        &ctx,
        &earlier(d, &[], &c.spec.code),
        &mut BTreeSet::new(),
    )?;
    Ok(Some(s))
}

/// Primjerci kruga jednog hotela, kolektivna samo za taj hotel (`draw_collective` s jednim hotelom).
pub fn draw_situations(
    view: &DrawView,
    d: &Draw,
    cat: &Catalog,
) -> Result<Vec<DrawnSituation>, ResolveError> {
    let shared = draw_collective(&[(*view, *d)], cat)?;
    draw_situations_with(view, d, cat, shared.as_ref())
}

/// Primjerci kruga jednog hotela uz zadanu kolektivnu (jedna za cijelu igru, `draw_collective`; `None` = nema je):
/// ona je u izlazu kao primjerak utora 0, pa veća i obične do broja kruga, poredani po važnosti (4 prva; jednaki
/// redom izvlačenja). Može ih biti manje od broja kruga kad dopuštenih predložaka nema dovoljno.
pub fn draw_situations_with(
    view: &DrawView,
    d: &Draw,
    cat: &Catalog,
    shared: Option<&DrawnSituation>,
) -> Result<Vec<DrawnSituation>, ResolveError> {
    check_history(d)?;
    let p = d.period;
    let n = round_count(d, expected_occupancy_bp(view.state, p, cat)?);
    let slots = usize::from(n).saturating_sub(d.anchors.len());
    let facts = Facts::new(view, p, cat)?;
    let names = &d.templates.names;
    let hotel = FillCtx {
        p,
        seed: d.seed,
        slot: d.slot,
        names,
    };
    let mut used = BTreeSet::new();
    let mut out: Vec<DrawnSituation> = Vec::new();
    if let Some(s) = shared {
        used.extend(s.roles.values().map(|r| r.name.clone()));
        out.push(s.clone());
    }
    let reserve = reserved_groups(d, &out);
    let month = p.start_month();
    let in_month = |pred: &dyn Fn(&DrawnSituation) -> bool| {
        d.history
            .iter()
            .filter(|h| h.period.start_month() == month && pred(h))
            .count()
    };

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
        let mut pool = candidates(specs, view, &facts, (d, &out, p.start_day), cat)?;
        pool.retain(|c| group_room(d, &out, c.spec.group_month, &reserve));
        let mut rng = stream(d.seed, Purpose::Situations, p.index, d.slot, label("veca"));
        let weights: Vec<u64> = pool.iter().map(|c| best(&c.options)).collect();
        if let Some(c) = pick(&mut rng, &weights)?.and_then(|i| pool.get(i)) {
            let department = choose(&mut rng, &c.options)?;
            let at = Place {
                department,
                day: p.start_day,
                occurrence: 1,
            };
            let s = fill(
                c.spec,
                at,
                &hotel,
                &earlier(d, &out, &c.spec.code),
                &mut used,
            )?;
            out.push(s);
        }
    }

    // Obične do broja kruga, jedan blok i u mjesečnoj igri. Sidra, kolektivna i veća zauzimaju prva mjesta; razmak
    // ponavljanja gleda povijest, a predložak već izvučen u krugu ne dolazi opet.
    let mut rng = stream(d.seed, Purpose::Situations, p.index, d.slot, label("izbor"));
    let day = p.start_day;
    let target = usize::from(n);
    let pre = d.anchors.len().saturating_add(out.len()).min(target);
    let limit = dept_limit(n);
    let specs = d
        .templates
        .templates
        .iter()
        .filter(|t| t.scope == Scope::Hotel && t.major.is_none());
    let mut pool = candidates(specs, view, &facts, (d, &out, day), cat)?;
    let need_opportunity = needs_opportunity(
        day,
        d.templates.opportunity_every_weeks,
        d.history.iter().chain(&out),
    );
    let mut drawn: usize = 0;
    while pre.saturating_add(drawn) < target {
        let in_round = per_dept(d.anchors, out.iter());
        let room = |x: &DeptCode| in_round.get(x).copied().unwrap_or(0) < limit;
        // Granica po odjelu je stroga: radije manje situacija nego previše iz istog odjela. Kad je granica grupa
        // puna (sezona ili mjesec dolaska), predlošci koji nude takvu grupu ne dolaze.
        let allowed: Vec<(usize, Options)> = pool
            .iter()
            .enumerate()
            .filter(|(_, c)| group_room(d, &out, c.spec.group_month, &reserve))
            .map(|(i, c)| {
                let opts: Options = c
                    .options
                    .iter()
                    .filter(|(dept, _)| dept.as_ref().is_none_or(room))
                    .cloned()
                    .collect();
                (i, opts)
            })
            .filter(|(_, opts)| !opts.is_empty())
            .collect();
        if allowed.is_empty() {
            break;
        }
        // Raznolikost vrsta (meka): prilika kad je dugo nije bilo, a zadnje mjesto kruga druge vrste od
        // dosadašnjih (sidra uključena).
        let kinds: BTreeSet<Kind> = d
            .anchors
            .iter()
            .map(|a| a.kind)
            .chain(out.iter().map(|s| s.kind))
            .collect();
        let kind_of = |i: usize| pool.get(i).map(|c| c.spec.kind);
        let has =
            |f: &dyn Fn(Kind) -> bool| allowed.iter().any(|(i, _)| kind_of(*i).is_some_and(f));
        let keep: Box<dyn Fn(Kind) -> bool> = if need_opportunity
            && !kinds.contains(&Kind::Opportunity)
            && has(&|k| k == Kind::Opportunity)
        {
            Box::new(|k| k == Kind::Opportunity)
        } else if pre.saturating_add(drawn).saturating_add(1) == target
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
        let department = choose(&mut rng, opts)?;
        let c = pool.remove(*i);
        let at = Place {
            department,
            day,
            occurrence: 1,
        };
        let s = fill(
            c.spec,
            at,
            &hotel,
            &earlier(d, &out, &c.spec.code),
            &mut used,
        )?;
        out.push(s);
        drawn = drawn.saturating_add(1);
    }

    out.sort_by_key(|s| std::cmp::Reverse(s.weight));
    Ok(out)
}
