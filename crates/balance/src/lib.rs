//! Alat za balans: pokreće jezgru preko profila, strategija i sjemena i provjerava kriterije DEMO §11.
//! Nije jezgra, pa smije std; sve račune drži u cijelim brojevima.

use engine::calc;
pub mod v2;

use sim_hotel::{
    Catalog, CatalogError, Category, HotelState, Index, IndexSet, Maintenance, Mechanic, Month,
    MonthPlan, Profile, RefTable, ResolveError, SituationOutcome, SituationTemplate, Strategy,
    Training, effects, resolve_month, run_season, run_season_with, start_indices,
};

pub struct Options {
    pub seeds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub starts: Vec<(Profile, IndexSet)>,
    pub reference_margin_bp: Vec<(Month, i32)>,
    pub reference_season_margin_bp: i32,
    pub august_displacement: bool,
    /// Dobit GOP-a (centi) od prihvaćanja ponude iz S3 „kako je ponuđeno" u kolovozu i u lipnju (GOP s grupom −
    /// GOP bez nje, ostalo jednako, zadani plan, referentni profil). Provjera §7: kolovoz < lipanj. `None` bez S3.
    pub s3_gop_gain_cents: Option<S3Gain>,
    pub medians: Vec<(Profile, Strategy, i32)>,
    pub profile_medians: Vec<(Profile, i32)>,
    pub probes: Vec<(String, Vec<(Profile, i32)>)>,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct S3Gain {
    pub august_cents: i64,
    pub june_cents: i64,
}

pub(crate) const REPRO_SEED: u64 = 7;

fn calc_err(e: calc::CalcError) -> ResolveError {
    ResolveError::Calc(e)
}

/// Medijan: za paran broj niži od dva srednja.
pub(crate) fn median(values: &mut [i32]) -> i32 {
    values.sort_unstable();
    match values.len().checked_sub(1) {
        Some(last) => values.get(last / 2).copied().unwrap_or(0),
        None => 0,
    }
}

/// `part · 10 000 / whole` u baznim bodovima; bez prihoda (`whole ≤ 0`) 0.
pub(crate) fn ratio_bp(part: i128, whole: i128) -> Result<i32, calc::CalcError> {
    if whole <= 0 {
        return Ok(0);
    }
    calc::narrow(calc::div_round(calc::mul(part, 10_000)?, whole)?)
}

fn margin_bp(gop: i128, revenue: i128) -> Result<i32, ResolveError> {
    ratio_bp(gop, revenue).map_err(calc_err)
}

pub(crate) fn reference_profile(cat: &Catalog) -> Result<Profile, ResolveError> {
    let category = cat.profile(Profile::C)?.category;
    cat.reference_profiles
        .get(&category)
        .copied()
        .ok_or_else(|| {
            ResolveError::Catalog(CatalogError::Missing(format!(
                "reference_profiles.{}",
                category.code()
            )))
        })
}

type RefMargins = (Vec<(Month, i32)>, i32, HotelState, HotelState);

/// Marže referentnog profila uz zadani plan, bez situacija; i stanja prije lipnja i prije kolovoza.
fn reference_margins(cat: &Catalog) -> Result<RefMargins, ResolveError> {
    let mut state = HotelState::from_profile(cat.profile(reference_profile(cat)?)?)?;
    let mut out = Vec::new();
    let (mut gop, mut rev) = (0i128, 0i128);
    let before_jun = state.clone();
    let mut before_aug = state.clone();
    for m in Month::SEASON {
        if m == Month::Aug {
            before_aug = state.clone();
        }
        let plan = MonthPlan::default_for(&state, m, cat)?;
        let r = resolve_month(&state, m, &plan, &[], cat)?;
        let g = i128::from(r.money.gop_cents);
        let v = i128::from(r.money.rooms_revenue_cents) + i128::from(r.money.other_revenue_cents);
        out.push((m, margin_bp(g, v)?));
        gop += g;
        rev += v;
        state = r.after;
    }
    Ok((out, margin_bp(gop, rev)?, before_jun, before_aug))
}

/// Ponuda „kako je ponuđeno" iz S3 (ADR-014): prihvaćena kolovoška grupa s najvećim popustom, tj. bez
/// pregovora (25 soba × 7 noći uz −25 %). Istu ponudu koriste i provjera istiskivanja i dobit GOP-a.
fn s3_offer_mechanic(s3: &SituationTemplate) -> Option<Mechanic> {
    s3.answers
        .iter()
        .flat_map(|a| a.mechanics.iter())
        .filter(|m| {
            matches!(
                m,
                Mechanic::GroupBooking {
                    accept: true,
                    month: Month::Aug,
                    ..
                }
            )
        })
        .max_by_key(|m| match m {
            Mechanic::GroupBooking { discount_bp, .. } => *discount_bp,
            _ => 0,
        })
        .cloned()
}

/// Ista grupa, ali prodaje se u mjesecu `target`; blok kako ga daje `effects` u mjesecu situacije.
fn s3_group(
    cat: &Catalog,
    s3: &SituationTemplate,
    mech: &Mechanic,
    target: Month,
    category: Category,
) -> Result<Option<sim_hotel::GroupBlock>, ResolveError> {
    let Mechanic::GroupBooking {
        accept,
        rooms,
        nights,
        discount_bp,
        ..
    } = mech
    else {
        return Ok(None);
    };
    let moved = Mechanic::GroupBooking {
        accept: *accept,
        rooms: *rooms,
        nights: *nights,
        discount_bp: *discount_bp,
        month: target,
    };
    let quality = s3
        .answers
        .iter()
        .find(|a| a.mechanics.contains(mech))
        .map_or(3, |a| a.quality);
    let o = SituationOutcome {
        code: s3.code.clone(),
        mechanics: vec![moved],
        quality,
    };
    let e = effects(&o, s3.month, cat, category).map_err(ResolveError::Mechanic)?;
    Ok(e.groups.into_iter().find(|g| g.month == target))
}

/// Grupa iz S3 u kolovozu istiskuje individualne goste (ADR-013 posljedice).
fn august_displacement(
    cat: &Catalog,
    situations: &[SituationTemplate],
    before_aug: &HotelState,
) -> Result<bool, ResolveError> {
    let Some(s3) = situations.iter().find(|t| t.code == "S3") else {
        return Ok(false);
    };
    let Some(mech) = s3_offer_mechanic(s3) else {
        return Ok(false);
    };
    let Some(group) = s3_group(cat, s3, &mech, Month::Aug, before_aug.category)? else {
        return Ok(false);
    };
    let plan = MonthPlan::default_for(before_aug, Month::Aug, cat)?;
    let without = resolve_month(before_aug, Month::Aug, &plan, &[], cat)?;
    let mut with_state = before_aug.clone();
    with_state.groups.push(group);
    let with = resolve_month(&with_state, Month::Aug, &plan, &[], cat)?;
    Ok(with.market.individual_sold < without.market.individual_sold)
}

/// GOP s grupom − GOP bez nje u mjesecu `month`, iz stanja `before`, uz zadani plan.
fn group_gop_gain(
    cat: &Catalog,
    s3: &SituationTemplate,
    mech: &Mechanic,
    month: Month,
    before: &HotelState,
) -> Result<i64, ResolveError> {
    let Some(group) = s3_group(cat, s3, mech, month, before.category)? else {
        return Ok(0);
    };
    let plan = MonthPlan::default_for(before, month, cat)?;
    let without = resolve_month(before, month, &plan, &[], cat)?;
    let mut with_state = before.clone();
    with_state.groups.push(group);
    let with = resolve_month(&with_state, month, &plan, &[], cat)?;
    calc::narrow(
        calc::sub(
            i128::from(with.money.gop_cents),
            i128::from(without.money.gop_cents),
        )
        .map_err(calc_err)?,
    )
    .map_err(calc_err)
}

fn s3_gop_gain(
    cat: &Catalog,
    situations: &[SituationTemplate],
    before_jun: &HotelState,
    before_aug: &HotelState,
) -> Result<Option<S3Gain>, ResolveError> {
    let Some(s3) = situations.iter().find(|t| t.code == "S3") else {
        return Ok(None);
    };
    let Some(mech) = s3_offer_mechanic(s3) else {
        return Ok(None);
    };
    Ok(Some(S3Gain {
        august_cents: group_gop_gain(cat, s3, &mech, Month::Aug, before_aug)?,
        june_cents: group_gop_gain(cat, s3, &mech, Month::Jun, before_jun)?,
    }))
}

#[derive(Clone, Copy)]
pub(crate) enum Probe {
    Price60,
    Price150,
    Marketing,
    SeasonalUp,
    SeasonalZero,
    Wage115,
    Wage90,
    TrainingFull,
    MaintIntensive,
    MaintDeferred,
    Reply,
    RenovationJune,
}

impl Probe {
    pub(crate) const ALL: [Probe; 12] = [
        Probe::Price60,
        Probe::Price150,
        Probe::Marketing,
        Probe::SeasonalUp,
        Probe::SeasonalZero,
        Probe::Wage115,
        Probe::Wage90,
        Probe::TrainingFull,
        Probe::MaintIntensive,
        Probe::MaintDeferred,
        Probe::Reply,
        Probe::RenovationJune,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Probe::Price60 => "cijena 60 %",
            Probe::Price150 => "cijena 150 %",
            Probe::Marketing => "marketing 40 000 EUR",
            Probe::SeasonalUp => "sezonci +korak",
            Probe::SeasonalZero => "sezonci −korak svaki mjesec",
            Probe::Wage115 => "plaća 115",
            Probe::Wage90 => "plaća 90",
            Probe::TrainingFull => "obuka Full",
            Probe::MaintIntensive => "održavanje Intensive",
            Probe::MaintDeferred => "održavanje Deferred",
            Probe::Reply => "odgovaranje da",
            Probe::RenovationJune => "obnova u lipnju",
        }
    }

    fn apply(
        self,
        plan: &mut MonthPlan,
        state: &HotelState,
        month: Month,
        cat: &Catalog,
    ) -> Result<(), ResolveError> {
        let a = &cat.actions;
        let spec = cat.category(state.category)?;
        match self {
            Probe::Price60 | Probe::Price150 => {
                let pct: i128 = if matches!(self, Probe::Price60) {
                    60
                } else {
                    150
                };
                let reference = i128::from(spec.price_ref_cents.get(month));
                let lo = (i128::from(a.price_min_pct) * reference + 9_999) / 10_000;
                let hi = i128::from(a.price_max_pct) * reference / 10_000;
                let euros = (reference * pct / 10_000).max(lo).min(hi.max(lo));
                plan.price_cents = calc::narrow(euros * 100).map_err(calc_err)?;
            }
            Probe::Marketing => plan.marketing_cents = a.marketing_max_cents.min(4_000_000),
            Probe::SeasonalUp => {
                plan.seasonal_target = state
                    .seasonal
                    .saturating_add(a.seasonal_step)
                    .min(a.seasonal_max);
            }
            Probe::SeasonalZero => {
                plan.seasonal_target = state.seasonal.saturating_sub(a.seasonal_step);
            }
            Probe::Wage115 => plan.wage_pct = 115.max(a.wage_min_pct).min(a.wage_max_pct),
            Probe::Wage90 => plan.wage_pct = 90.max(a.wage_min_pct).min(a.wage_max_pct),
            Probe::TrainingFull => plan.training = Training::Full,
            Probe::MaintIntensive => plan.maintenance = Maintenance::Intensive,
            Probe::MaintDeferred => plan.maintenance = Maintenance::Deferred,
            Probe::Reply => plan.reply_reviews = true,
            Probe::RenovationJune => {
                plan.renovation = month == Month::Jun && !state.renovated;
            }
        }
        Ok(())
    }
}

/// Odgovor „srednji po q": stabilno sortirano po kvaliteti, indeks len/2.
pub(crate) fn middle_index(t: &SituationTemplate) -> Option<usize> {
    let mut order: Vec<usize> = (0..t.answers.len()).collect();
    order.sort_by_key(|&i| t.answers.get(i).map_or(0, |a| a.quality));
    order.get(order.len() / 2).copied()
}

fn middle_by_quality(t: &SituationTemplate) -> Result<usize, ResolveError> {
    middle_index(t).ok_or(ResolveError::BadChoice)
}

fn run_probe(
    probe: Probe,
    profile: Profile,
    cat: &Catalog,
    situations: &[SituationTemplate],
    refs: &RefTable,
) -> Result<i32, ResolveError> {
    let r = run_season_with(
        profile,
        cat,
        situations,
        refs,
        &mut |s, m| {
            let mut plan = MonthPlan::default_for(s, m, cat)?;
            probe.apply(&mut plan, s, m, cat)?;
            Ok(plan)
        },
        &mut middle_by_quality,
    )?;
    Ok(r.progress_pts)
}

fn category_refs(cat: &Catalog) -> Result<Vec<(Category, RefTable)>, ResolveError> {
    let mut out = Vec::new();
    for c in Category::ALL {
        if cat.categories.contains_key(&c) {
            out.push((c, RefTable::compute(cat, c)?));
        }
    }
    Ok(out)
}

/// Medijani napretka po profilu i strategiji i po profilu (sve strategije zajedno); `season` vraća napredak
/// jedne sezone za (profil, strategija, sjeme). Zajedničko v1 i v2.
#[allow(clippy::type_complexity)]
pub(crate) fn season_medians<E>(
    seeds: u64,
    season: &mut dyn FnMut(Profile, Strategy, u64) -> Result<i32, E>,
) -> Result<(Vec<(Profile, Strategy, i32)>, Vec<(Profile, i32)>), E> {
    let mut medians = Vec::new();
    let mut per_profile: Vec<(Profile, Vec<i32>)> =
        Profile::ALL.iter().map(|&p| (p, Vec::new())).collect();
    for p in Profile::ALL {
        for s in Strategy::ALL {
            let mut pts = Vec::new();
            for seed in 0..seeds {
                pts.push(season(p, s, seed)?);
            }
            if let Some((_, all)) = per_profile.iter_mut().find(|(q, _)| *q == p) {
                all.extend(pts.iter().copied());
            }
            medians.push((p, s, median(&mut pts)));
        }
    }
    let profile_medians = per_profile
        .into_iter()
        .map(|(p, mut v)| (p, median(&mut v)))
        .collect();
    Ok((medians, profile_medians))
}

/// §11.5: za svaki profil i strategiju `same` kaže jesu li dva izvršavanja sa sjemenom 7 jednaka.
pub(crate) fn all_reproducible<E>(
    same: &mut dyn FnMut(Profile, Strategy) -> Result<bool, E>,
) -> Result<bool, E> {
    let mut reproducible = true;
    for p in Profile::ALL {
        for s in Strategy::ALL {
            reproducible &= same(p, s)?;
        }
    }
    Ok(reproducible)
}

/// Jedna vrijednost po profilu (redak probe).
pub(crate) fn profile_row<E>(
    value: &mut dyn FnMut(Profile) -> Result<i32, E>,
) -> Result<Vec<(Profile, i32)>, E> {
    Profile::ALL.iter().map(|&p| Ok((p, value(p)?))).collect()
}

pub fn run(content: &content::Content, opts: &Options) -> Result<Report, ResolveError> {
    let cat = &content.catalog;
    let situations = &content.situations;
    let all_refs = category_refs(cat)?;
    let refs_for = |profile: Profile| -> Result<&RefTable, ResolveError> {
        let category = cat.profile(profile)?.category;
        all_refs
            .iter()
            .find(|(c, _)| *c == category)
            .map(|(_, r)| r)
            .ok_or_else(|| {
                ResolveError::Catalog(CatalogError::Missing(format!(
                    "categories.{}",
                    category.code()
                )))
            })
    };

    let mut starts = Vec::new();
    for p in Profile::ALL {
        let state = HotelState::from_profile(cat.profile(p)?)?;
        starts.push((p, start_indices(&state, cat, refs_for(p)?)?));
    }

    let (reference_margin_bp, reference_season_margin_bp, before_jun, before_aug) =
        reference_margins(cat)?;
    let august_displacement = august_displacement(cat, situations, &before_aug)?;
    let s3_gop_gain_cents = s3_gop_gain(cat, situations, &before_jun, &before_aug)?;

    let (medians, profile_medians) =
        season_medians::<ResolveError>(opts.seeds, &mut |p, s, seed| {
            Ok(run_season(p, s, seed, cat, situations, refs_for(p)?)?.progress_pts)
        })?;
    let reproducible = all_reproducible::<ResolveError>(&mut |p, s| {
        let refs = refs_for(p)?;
        let a = run_season(p, s, REPRO_SEED, cat, situations, refs)?;
        let b = run_season(p, s, REPRO_SEED, cat, situations, refs)?;
        Ok(a == b)
    })?;

    let mut probes = Vec::new();
    for probe in Probe::ALL {
        let row = profile_row::<ResolveError>(&mut |p| {
            run_probe(probe, p, cat, situations, refs_for(p)?)
        })?;
        probes.push((probe.name().to_string(), row));
    }

    Ok(Report {
        starts,
        reference_margin_bp,
        reference_season_margin_bp,
        august_displacement,
        s3_gop_gain_cents,
        medians,
        profile_medians,
        probes,
        reproducible,
    })
}

pub(crate) fn lookup<T: Copy>(row: &[(Profile, T)], p: Profile, default: T) -> T {
    row.iter()
        .find(|(q, _)| *q == p)
        .map_or(default, |(_, v)| *v)
}

/// §11.1 početak: indeksi na početku blizu ciljeva profila (C ±100, A/B ±200).
pub(crate) fn check_start(f: &mut Vec<String>, starts: &[(Profile, IndexSet)]) {
    for (p, set) in starts {
        let tol = if *p == Profile::C { 100i32 } else { 200 };
        let target = p.starting_indices();
        for i in Index::ALL {
            let got = i32::from(set.get(i).hundredths());
            let want = i32::from(target.get(i).hundredths());
            if (got - want).abs() > tol {
                f.push(format!(
                    "§11.1 početak: profil {} indeks {} je {} (cilj {} ±{})",
                    p.code(),
                    i.code(),
                    got,
                    want,
                    tol
                ));
            }
        }
    }
}

/// §11.3 pravednost: razlika medijana profila i nijedan profil najbolji pod svim strategijama.
pub(crate) fn check_fairness(
    f: &mut Vec<String>,
    profile_medians: &[(Profile, i32)],
    medians: &[(Profile, Strategy, i32)],
) {
    let hi = profile_medians.iter().map(|(_, v)| *v).max();
    let lo = profile_medians.iter().map(|(_, v)| *v).min();
    if let (Some(hi), Some(lo)) = (hi, lo)
        && hi - lo >= 500
    {
        f.push(format!(
            "§11.3 pravednost: razlika medijana profila je {} (najviše 499)",
            hi - lo
        ));
    }
    for p in Profile::ALL {
        let wins_all = Strategy::ALL.iter().all(|s| {
            let of = |q: Profile| {
                medians
                    .iter()
                    .find(|(pp, ss, _)| *pp == q && ss == s)
                    .map_or(i32::MIN, |(_, _, v)| *v)
            };
            Profile::ALL
                .iter()
                .filter(|q| **q != p)
                .all(|q| of(p) > of(*q))
        });
        if wins_all {
            f.push(format!(
                "§11.3 pravednost: profil {} ima najveći medijan pod svim strategijama",
                p.code()
            ));
        }
    }
}

/// §11.4 dominantna radnja: strogo najveći napredak za sva tri profila.
pub(crate) fn check_dominant(f: &mut Vec<String>, probes: &[(String, Vec<(Profile, i32)>)]) {
    for (name, row) in probes {
        let dominant = Profile::ALL.iter().all(|p| {
            let mine = lookup(row, *p, i32::MIN);
            probes
                .iter()
                .filter(|(n, _)| n != name)
                .all(|(_, r)| mine > lookup(r, *p, i32::MIN))
        });
        if dominant {
            f.push(format!(
                "§11.4 dominantna radnja: \"{name}\" ima najveći napredak za sva tri profila"
            ));
        }
    }
}

/// §11.5 ponovljivost.
pub(crate) fn check_repro(f: &mut Vec<String>, reproducible: bool) {
    if !reproducible {
        f.push(
            "§11.5 ponovljivost: ista sezona sa sjemenom 7 dala je različit rezultat".to_string(),
        );
    }
}

/// Prazno = sve provjere prolaze.
pub fn failures(report: &Report, _cat: &Catalog) -> Vec<String> {
    let mut f = Vec::new();

    check_start(&mut f, &report.starts);

    // §11.2 ekonomija
    let season = report.reference_season_margin_bp;
    if !(3_500..=4_500).contains(&season) {
        f.push(format!(
            "§11.2 ekonomija: sezonska marža VI–IX je {season} bp (cilj 3500–4500)"
        ));
    }
    for (m, bp) in &report.reference_margin_bp {
        if *bp > 5_000 {
            f.push(format!(
                "§11.2 ekonomija: marža u mjesecu {} je {bp} bp (najviše 5000)",
                m.code()
            ));
        }
    }
    if !report.august_displacement {
        f.push("§11.2 istiskivanje: grupa u kolovozu ne smanjuje individualne goste".to_string());
    }

    // §7 S3 (ADR-014): ista ponuda je dobra u lipnju, a loša u rasprodanom kolovozu
    match &report.s3_gop_gain_cents {
        Some(g) if g.august_cents >= g.june_cents => f.push(format!(
            "§7 S3: prihvaćanje grupe u kolovozu donosi {} GOP-a, a u lipnju {} (kolovoz mora biti manji)",
            euros(g.august_cents),
            euros(g.june_cents)
        )),
        Some(_) => {}
        None => f.push("§7 S3: nema ponude grupe za kolovoz, lekcija se ne može provjeriti".to_string()),
    }

    check_fairness(&mut f, &report.profile_medians, &report.medians);
    check_dominant(&mut f, &report.probes);
    check_repro(&mut f, report.reproducible);
    f
}

pub(crate) fn hundredths(v: i32) -> String {
    let sign = if v < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", v.abs() / 100, v.abs() % 100)
}

fn euros(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    let whole = (abs / 100).to_string();
    let mut grouped = String::new();
    for (i, ch) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(ch);
    }
    format!("{sign}{grouped} EUR")
}

pub(crate) fn push_starts(s: &mut String, starts: &[(Profile, IndexSet)]) {
    s.push_str("== Početak (§11.1): G R Z O F ==\n");
    for (p, set) in starts {
        let vals: Vec<String> = Index::ALL
            .iter()
            .map(|i| hundredths(i32::from(set.get(*i).hundredths())))
            .collect();
        s.push_str(&format!("  {}: {}\n", p.code(), vals.join("  ")));
    }
}

pub(crate) fn push_medians(
    s: &mut String,
    medians: &[(Profile, Strategy, i32)],
    profile_medians: &[(Profile, i32)],
) {
    s.push_str("== Medijan napretka po profilu i strategiji (§11.3), bodova ==\n");
    for (p, st, v) in medians {
        s.push_str(&format!(
            "  {} {:<10} {}\n",
            p.code(),
            st.code(),
            hundredths(*v)
        ));
    }
    for (p, v) in profile_medians {
        s.push_str(&format!(
            "  profil {} (sve strategije): {}\n",
            p.code(),
            hundredths(*v)
        ));
    }
}

pub(crate) fn push_probes(s: &mut String, probes: &[(String, Vec<(Profile, i32)>)]) {
    s.push_str("== Probe krajnjih radnji (§11.4), napredak A / B / C ==\n");
    for (name, row) in probes {
        let vals: Vec<String> = row
            .iter()
            .map(|(p, v)| format!("{}={}", p.code(), hundredths(*v)))
            .collect();
        s.push_str(&format!("  {:<24} {}\n", name, vals.join("  ")));
    }
}

pub fn render(report: &Report) -> String {
    let mut s = String::new();
    push_starts(&mut s, &report.starts);
    s.push_str("== Ekonomija referentnog hotela (§11.2), marža GOP-a ==\n");
    for (m, bp) in &report.reference_margin_bp {
        s.push_str(&format!("  {}: {} %\n", m.code(), hundredths(*bp)));
    }
    s.push_str(&format!(
        "  sezona VI–IX: {} %\n  kolovoz: grupa istiskuje individualne goste: {}\n",
        hundredths(report.reference_season_margin_bp),
        if report.august_displacement {
            "da"
        } else {
            "ne"
        }
    ));
    match &report.s3_gop_gain_cents {
        Some(g) => s.push_str(&format!(
            "  S3 grupa, dobit GOP-a od prihvaćanja: kolovoz {}, lipanj {} (§7: kolovoz < lipanj)\n",
            euros(g.august_cents),
            euros(g.june_cents)
        )),
        None => s.push_str("  S3 grupa, dobit GOP-a od prihvaćanja: nema grupe u kolovozu\n"),
    }
    push_medians(&mut s, &report.medians, &report.profile_medians);
    push_probes(&mut s, &report.probes);
    s.push_str(&format!(
        "== Ponovljivost (§11.5): {} ==\n",
        if report.reproducible { "da" } else { "ne" }
    ));
    s
}
