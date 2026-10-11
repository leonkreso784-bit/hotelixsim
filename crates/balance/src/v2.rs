//! Balans modela v2: ista pitanja kao v1 (DEMO §11) nad `sim_hotel_v2`, za tjedne i mjesečne krugove, plus kriteriji
//! sloja 1 (marže odjela, udio troška rada, ostali prihodi profila C, tjedno prema mjesečnom). Zajednički dijelovi
//! (medijani, provjere §11.1/3/4/5, ispis) žive u `lib.rs` i dijele se s v1.

use engine::calc;
use sim_hotel_v2::calendar::{avg_i64, avg_u16, first_period_of_month, prorate};
use sim_hotel_v2::department::{DeptCode, UsaliClass};
use sim_hotel_v2::usali::{self, UsaliStatement};
use sim_hotel_v2::{
    Catalog, CatalogError, Category, HotelState, IndexSet, Maintenance, Month, Period, PeriodKind,
    PeriodPlan, PlanLimits, Profile, RefTable, ResolveError, SeasonResult, Strategy, Training,
    run_season, run_season_with, season_periods, start_indices,
};

use crate::{
    Probe, REPRO_SEED, all_reproducible, check_dominant, check_fairness, check_repro, check_start,
    hundredths, middle_index, profile_row, push_medians, push_probes, push_starts, ratio_bp,
    reference_profile, season_medians,
};

pub struct OptionsV2 {
    pub seeds: u64,
    pub kind: PeriodKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportV2 {
    pub starts: Vec<(Profile, IndexSet)>,
    pub reference_season_gop_margin_bp: i32,
    pub reference_season_ebitda_margin_bp: i32,
    pub reference_dept_margins_bp: Vec<(DeptCode, UsaliClass, Option<i32>)>,
    pub reference_labor_share_bp: i32,
    /// Profil C, sezona, zadani plan: Σ prihoda svih odjela osim Smještaja / prihod soba, u bp.
    pub reference_other_revenue_share_bp: i32,
    pub medians: Vec<(Profile, Strategy, i32)>,
    pub profile_medians: Vec<(Profile, i32)>,
    pub probes: Vec<(String, Vec<(Profile, i32)>)>,
    /// Medijan napretka tjedno, mjesečno (obje vrste krugova, bez obzira na `OptionsV2::kind`).
    pub week_vs_month: Vec<(Profile, Strategy, i32, i32)>,
    pub reproducible: bool,
    /// Robot „brižni" (sjeme ponovljivosti): Σ sezonaca·dani / Σ sezonskog plana kategorije·dani, u bp, po profilu.
    pub caring_seasonal_vs_plan_bp: Vec<(Profile, i32)>,
}

type RefTables = Vec<(Category, RefTable)>;

fn category_refs(cat: &Catalog, kind: PeriodKind) -> Result<RefTables, ResolveError> {
    let periods = season_periods(kind);
    let mut out = Vec::new();
    for c in Category::ALL {
        if cat.base.categories.contains_key(&c) {
            out.push((c, RefTable::compute(cat, c, &periods)?));
        }
    }
    Ok(out)
}

fn refs_for<'a>(
    cat: &Catalog,
    refs: &'a RefTables,
    profile: Profile,
) -> Result<&'a RefTable, ResolveError> {
    let category = cat.base.profile(profile)?.category;
    refs.iter()
        .find(|(c, _)| *c == category)
        .map(|(_, r)| r)
        .ok_or_else(|| {
            ResolveError::Catalog(CatalogError::Missing(format!(
                "categories.{}",
                category.code()
            )))
        })
}

/// Račun dobiti cijele sezone (zbroj krugova) uz zadani plan, bez situacija.
fn season_statement(
    profile: Profile,
    kind: PeriodKind,
    cat: &Catalog,
    refs: &RefTable,
) -> Result<UsaliStatement, ResolveError> {
    let r = run_season_with(
        profile,
        kind,
        cat,
        &[],
        refs,
        &mut |s, p| Ok(PeriodPlan::default_for(s, p, cat)?),
        &mut |_| Err(ResolveError::BadChoice),
    )?;
    let statements = r
        .periods
        .iter()
        .map(|(report, _)| usali::statement(&report.accounts))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(usali::sum_statements(&statements)?)
}

/// Sezonci nakon svakog kruga prema sezonskom planu kategorije za taj krug, ponderirano danima kruga (bp).
fn seasonal_vs_plan_bp(
    r: &SeasonResult,
    profile: Profile,
    cat: &Catalog,
) -> Result<i32, ResolveError> {
    let spec = cat.base.category(cat.base.profile(profile)?.category)?;
    let (mut held, mut planned) = (0_i128, 0_i128);
    for (rep, _) in &r.periods {
        let days = i128::from(rep.period.days);
        held = calc::add(
            held,
            calc::mul(i128::from(rep.summary.after.seasonal), days)?,
        )?;
        let plan = avg_u16(&spec.seasonal_plan, &rep.period)?;
        planned = calc::add(planned, calc::mul(i128::from(plan), days)?)?;
    }
    Ok(ratio_bp(held, planned)?)
}

fn narrow_bp(x: i64) -> Result<i32, ResolveError> {
    Ok(calc::narrow(i128::from(x))?)
}

fn opt_bp(x: Option<i64>) -> Result<i32, ResolveError> {
    x.map_or(Ok(0), narrow_bp)
}

/// Σ plaća (odjeli i neraspoređeni) / ukupni prihod, u bp.
fn labor_share_bp(st: &UsaliStatement) -> Result<i32, ResolveError> {
    let mut payroll: i128 = 0;
    for d in &st.departments {
        payroll = calc::add(payroll, i128::from(d.payroll_cents))?;
    }
    for u in &st.undistributed {
        payroll = calc::add(payroll, i128::from(u.payroll_cents))?;
    }
    Ok(ratio_bp(payroll, i128::from(st.total_revenue_cents))?)
}

/// Prihodi svih odjela osim Smještaja / prihod Smještaja, u bp.
fn other_revenue_share_bp(st: &UsaliStatement) -> Result<i32, ResolveError> {
    let (mut rooms, mut other): (i128, i128) = (0, 0);
    for d in &st.departments {
        let v = i128::from(d.revenue_cents);
        if d.class == UsaliClass::Rooms {
            rooms = calc::add(rooms, v)?;
        } else {
            other = calc::add(other, v)?;
        }
    }
    Ok(ratio_bp(other, rooms)?)
}

/// Probe v2: sve probe v1 prenesene na `PeriodPlan` plus odjelne.
#[derive(Clone, Copy)]
enum ProbeV2 {
    Base(Probe),
    CloseAll,
    PriceLevel(u16),
    MaintenanceAll(Maintenance),
    SeasonalRoomsOnly,
}

impl ProbeV2 {
    fn all() -> Vec<ProbeV2> {
        let mut out: Vec<ProbeV2> = Probe::ALL.iter().map(|&p| ProbeV2::Base(p)).collect();
        out.extend([
            ProbeV2::CloseAll,
            ProbeV2::PriceLevel(120),
            ProbeV2::PriceLevel(90),
            ProbeV2::MaintenanceAll(Maintenance::Deferred),
            ProbeV2::MaintenanceAll(Maintenance::Intensive),
            ProbeV2::SeasonalRoomsOnly,
        ]);
        out
    }

    fn name(self) -> String {
        match self {
            ProbeV2::Base(p) => p.name().to_string(),
            ProbeV2::CloseAll => "zatvori sve odjele koji se smiju zatvoriti".to_string(),
            ProbeV2::PriceLevel(120) => "razina cijene 120 % u svim odjelima".to_string(),
            ProbeV2::PriceLevel(pct) => format!("razina cijene {pct} %"),
            ProbeV2::MaintenanceAll(Maintenance::Deferred) => {
                "održavanje odgođeno svugdje".to_string()
            }
            ProbeV2::MaintenanceAll(_) => "održavanje pojačano svugdje".to_string(),
            ProbeV2::SeasonalRoomsOnly => "sezonci samo u Smještaju".to_string(),
        }
    }

    /// Zadani plan kruga s ugrađenom probom; svaka proba daje valjan plan (granice kruga).
    fn plan(self, s: &HotelState, p: &Period, cat: &Catalog) -> Result<PeriodPlan, ResolveError> {
        let mut plan = PeriodPlan::default_for(s, p, cat)?;
        let l = PeriodPlan::limits(s, p, cat)?;
        match self {
            ProbeV2::Base(probe) => base_probe(probe, &mut plan, (s, p, cat), &l)?,
            ProbeV2::CloseAll => {
                // Zatvaranje je dopušteno tek kad odjel nema sezonaca; do tada ih se otpušta koliko korak dopušta.
                for (code, d) in plan.departments.iter_mut() {
                    if l.departments.get(code).is_some_and(|x| x.can_close) {
                        d.seasonal_target = 0;
                        d.open = s
                            .departments
                            .get(code)
                            .is_some_and(|x| x.open && x.seasonal > 0);
                    }
                }
                plan = plan.carry_into(s, p, cat)?;
            }
            ProbeV2::PriceLevel(pct) => {
                for (code, d) in plan.departments.iter_mut() {
                    if let Some(x) = l.departments.get(code) {
                        d.price_level_pct =
                            pct.max(x.price_level_min_pct).min(x.price_level_max_pct);
                    }
                }
            }
            ProbeV2::MaintenanceAll(level) => {
                for d in plan.departments.values_mut() {
                    d.maintenance = level;
                }
            }
            ProbeV2::SeasonalRoomsOnly => {
                let total = plan.seasonal_total();
                for (code, d) in plan.departments.iter_mut() {
                    d.seasonal_target = if *code == DeptCode::rooms() { total } else { 0 };
                }
                plan = plan.carry_into(s, p, cat)?;
            }
        }
        Ok(plan)
    }
}

/// Probe v1 na planu kruga: ista pravila kao u v1, svedena na granice kruga (`PlanLimits`).
fn base_probe(
    probe: Probe,
    plan: &mut PeriodPlan,
    (s, p, cat): (&HotelState, &Period, &Catalog),
    l: &PlanLimits,
) -> Result<(), ResolveError> {
    match probe {
        Probe::Price60 | Probe::Price150 => {
            let pct: i128 = if matches!(probe, Probe::Price60) {
                60
            } else {
                150
            };
            let reference =
                i128::from(avg_i64(&cat.base.category(s.category)?.price_ref_cents, p)?);
            let euros = reference * pct / 10_000;
            let cents: i64 = calc::narrow(euros * 100)?;
            plan.price_cents = cents
                .max(l.price_min_cents)
                .min(l.price_max_cents.max(l.price_min_cents));
        }
        Probe::Marketing => {
            plan.marketing_cents = prorate(4_000_000, p)?.min(l.marketing_max_cents)
        }
        Probe::SeasonalUp => {
            *plan = plan.with_totals(s, p, cat, l.seasonal_max, plan.rooms_maintenance())?;
        }
        Probe::SeasonalZero => {
            *plan = plan.with_totals(s, p, cat, l.seasonal_min, plan.rooms_maintenance())?;
        }
        Probe::Wage115 => plan.wage_pct = 115.max(l.wage_min_pct).min(l.wage_max_pct),
        Probe::Wage90 => plan.wage_pct = 90.max(l.wage_min_pct).min(l.wage_max_pct),
        Probe::TrainingFull => plan.training = Training::Full,
        // v1 održavanje je održavanje Smještaja; sve odjele odjednom mijenja odjelna proba.
        Probe::MaintIntensive | Probe::MaintDeferred => {
            let level = if matches!(probe, Probe::MaintIntensive) {
                Maintenance::Intensive
            } else {
                Maintenance::Deferred
            };
            if let Some(d) = plan.departments.get_mut(&DeptCode::rooms()) {
                d.maintenance = level;
            }
        }
        Probe::Reply => plan.reply_reviews = true,
        Probe::RenovationJune => {
            plan.renovation =
                first_period_of_month(p.kind, Month::Jun) == *p && l.renovation_available;
        }
    }
    Ok(())
}

fn run_probe(
    probe: ProbeV2,
    profile: Profile,
    kind: PeriodKind,
    cat: &Catalog,
    situations: &[sim_hotel_v2::SituationTemplate],
    refs: &RefTable,
) -> Result<i32, ResolveError> {
    let r = run_season_with(
        profile,
        kind,
        cat,
        situations,
        refs,
        &mut |s, p| probe.plan(s, p, cat),
        &mut |t| middle_index(t).ok_or(ResolveError::BadChoice),
    )?;
    Ok(r.progress_pts)
}

type Medians = (Vec<(Profile, Strategy, i32)>, Vec<(Profile, i32)>);

fn kind_medians(
    content: &content::Content,
    cat: &Catalog,
    kind: PeriodKind,
    refs: &RefTables,
    seeds: u64,
) -> Result<Medians, ResolveError> {
    season_medians::<ResolveError>(seeds, &mut |p, s, seed| {
        Ok(run_season(
            p,
            s,
            seed,
            kind,
            cat,
            &content.situations,
            refs_for(cat, refs, p)?,
        )?
        .progress_pts)
    })
}

pub fn run_v2(content: &content::Content, opts: &OptionsV2) -> Result<ReportV2, ResolveError> {
    let cat = content
        .v2
        .as_ref()
        .ok_or_else(|| ResolveError::Catalog(CatalogError::Missing("v2".to_string())))?;
    let situations = &content.situations;
    let kind = opts.kind;
    let refs = category_refs(cat, kind)?;

    let mut starts = Vec::new();
    for p in Profile::ALL {
        let state = HotelState::from_profile(cat, p)?;
        starts.push((p, start_indices(&state, cat, refs_for(cat, &refs, p)?)?));
    }

    let reference = reference_profile(&cat.base)?;
    let ref_statement = season_statement(reference, kind, cat, refs_for(cat, &refs, reference)?)?;
    let c_statement = if reference == Profile::C {
        ref_statement.clone()
    } else {
        season_statement(Profile::C, kind, cat, refs_for(cat, &refs, Profile::C)?)?
    };
    let reference_dept_margins_bp = ref_statement
        .departments
        .iter()
        .map(|d| {
            let bp = d.margin_bp.map(narrow_bp).transpose()?;
            Ok((d.code.clone(), d.class, bp))
        })
        .collect::<Result<Vec<_>, ResolveError>>()?;

    let (medians, profile_medians) = kind_medians(content, cat, kind, &refs, opts.seeds)?;
    let other_kind = match kind {
        PeriodKind::Week => PeriodKind::Month,
        PeriodKind::Month => PeriodKind::Week,
    };
    let other_refs = category_refs(cat, other_kind)?;
    let (other_medians, _) = kind_medians(content, cat, other_kind, &other_refs, opts.seeds)?;
    let (week, month) = match kind {
        PeriodKind::Week => (&medians, &other_medians),
        PeriodKind::Month => (&other_medians, &medians),
    };
    let week_vs_month = week
        .iter()
        .zip(month.iter())
        .map(|((p, s, w), (_, _, m))| (*p, *s, *w, *m))
        .collect();

    let reproducible = all_reproducible::<ResolveError>(&mut |p, s| {
        let r = refs_for(cat, &refs, p)?;
        let a = run_season(p, s, REPRO_SEED, kind, cat, situations, r)?;
        let b = run_season(p, s, REPRO_SEED, kind, cat, situations, r)?;
        Ok(a == b)
    })?;

    let mut caring_seasonal_vs_plan_bp = Vec::new();
    for p in Profile::ALL {
        let r = run_season(
            p,
            Strategy::Caring,
            REPRO_SEED,
            kind,
            cat,
            situations,
            refs_for(cat, &refs, p)?,
        )?;
        caring_seasonal_vs_plan_bp.push((p, seasonal_vs_plan_bp(&r, p, cat)?));
    }

    let mut probes = Vec::new();
    for probe in ProbeV2::all() {
        let row = profile_row::<ResolveError>(&mut |p| {
            run_probe(probe, p, kind, cat, situations, refs_for(cat, &refs, p)?)
        })?;
        probes.push((probe.name(), row));
    }

    Ok(ReportV2 {
        starts,
        reference_season_gop_margin_bp: opt_bp(ref_statement.gop_margin_bp)?,
        reference_season_ebitda_margin_bp: opt_bp(ref_statement.ebitda_margin_bp)?,
        reference_dept_margins_bp,
        reference_labor_share_bp: labor_share_bp(&ref_statement)?,
        reference_other_revenue_share_bp: other_revenue_share_bp(&c_statement)?,
        medians,
        profile_medians,
        probes,
        week_vs_month,
        reproducible,
        caring_seasonal_vs_plan_bp,
    })
}

/// Pojas marže odjela u bp (sezona, referentni profil, zadani plan) po razredu; `None` = razred bez pojasa.
fn dept_band(class: UsaliClass) -> Option<(i32, i32)> {
    match class {
        UsaliClass::Rooms => Some((6_500, 8_000)),
        UsaliClass::FoodBeverage => Some((1_500, 3_500)),
        UsaliClass::OtherOperated => Some((2_000, 4_000)),
        _ => None,
    }
}

/// Najmanji udio sezonskog plana hotela koji brižni robot drži kroz sezonu (ZP-4).
const CARING_MIN_SEASONAL_VS_PLAN_BP: i32 = 8_000;

/// Prazno = sve provjere prolaze.
pub fn failures_v2(r: &ReportV2) -> Vec<String> {
    let mut f = Vec::new();
    check_start(&mut f, &r.starts);

    let gop = r.reference_season_gop_margin_bp;
    if !(3_500..=4_500).contains(&gop) {
        f.push(format!(
            "§11.2 ekonomija: GOP marža sezone je {gop} bp (cilj 3500–4500)"
        ));
    }
    let ebitda = r.reference_season_ebitda_margin_bp;
    if ebitda >= gop {
        f.push(format!(
            "§11.2 ekonomija: EBITDA marža {ebitda} bp nije ispod GOP marže {gop} bp"
        ));
    }
    for (code, class, margin) in &r.reference_dept_margins_bp {
        let Some((lo, hi)) = dept_band(*class) else {
            continue;
        };
        match margin {
            Some(bp) if (lo..=hi).contains(bp) => {}
            Some(bp) => f.push(format!(
                "sloj 1 marža odjela: {} ({}) je {bp} bp (cilj {lo}–{hi})",
                code.as_str(),
                class.code()
            )),
            None => f.push(format!(
                "sloj 1 marža odjela: {} ({}) nema prihoda (cilj {lo}–{hi} bp)",
                code.as_str(),
                class.code()
            )),
        }
    }
    let labor = r.reference_labor_share_bp;
    if !(3_000..=4_000).contains(&labor) {
        f.push(format!(
            "sloj 1 udio rada: trošak rada je {labor} bp ukupnog prihoda (cilj 3000–4000)"
        ));
    }
    let other = r.reference_other_revenue_share_bp;
    if !(2_500..=3_500).contains(&other) {
        f.push(format!(
            "sloj 1 ostali prihodi C: {other} bp prihoda soba (cilj 2500–3500)"
        ));
    }

    check_fairness(&mut f, &r.profile_medians, &r.medians);
    check_dominant(&mut f, &r.probes);

    for (p, s, week, month) in &r.week_vs_month {
        if (week - month).abs() > 300 {
            f.push(format!(
                "sloj 1 tjedno ≈ mjesečno: {} {} tjedno {} mjesečno {} (razlika najviše 300 stotinki)",
                p.code(),
                s.code(),
                week,
                month
            ));
        }
    }
    for (p, bp) in &r.caring_seasonal_vs_plan_bp {
        if *bp < CARING_MIN_SEASONAL_VS_PLAN_BP {
            f.push(format!(
                "sloj 1 brižni robot: {} drži {bp} bp sezonskog plana hotela (najmanje {CARING_MIN_SEASONAL_VS_PLAN_BP})",
                p.code()
            ));
        }
    }
    check_repro(&mut f, r.reproducible);
    f
}

/// Postotak s jednom decimalom i zarezom: 2940 → „29,4 %".
fn pct(bp: i32) -> String {
    let sign = if bp < 0 { "-" } else { "" };
    let tenths = (bp.abs() + 5) / 10;
    format!("{sign}{},{} %", tenths / 10, tenths % 10)
}

pub fn render_v2(r: &ReportV2) -> String {
    let mut s = String::new();
    push_starts(&mut s, &r.starts);
    s.push_str("== Ekonomija referentnog hotela, sezona, zadani plan ==\n");
    s.push_str(&format!(
        "  GOP marža: {}\n  EBITDA marža: {}\n",
        pct(r.reference_season_gop_margin_bp),
        pct(r.reference_season_ebitda_margin_bp)
    ));
    s.push_str("== Marže odjela (sezona, referentni profil) ==\n");
    for (code, class, margin) in &r.reference_dept_margins_bp {
        let value = margin.map_or("bez prihoda".to_string(), pct);
        s.push_str(&format!(
            "  {:<14} {:<15} {}\n",
            code.as_str(),
            class.code(),
            value
        ));
    }
    s.push_str(&format!(
        "  Udio troška rada u prihodu: {}\n  Ostali prihodi C / prihod soba: {}\n",
        pct(r.reference_labor_share_bp),
        pct(r.reference_other_revenue_share_bp)
    ));
    push_medians(&mut s, &r.medians, &r.profile_medians);
    push_probes(&mut s, &r.probes);
    s.push_str("== Tjedno prema mjesečnom, medijan napretka (tjedno / mjesečno) ==\n");
    for (p, st, week, month) in &r.week_vs_month {
        s.push_str(&format!(
            "  {} {:<10} {} / {}\n",
            p.code(),
            st.code(),
            hundredths(*week),
            hundredths(*month)
        ));
    }
    s.push_str("== Brižni robot: sezonci prema sezonskom planu hotela ==\n");
    for (p, bp) in &r.caring_seasonal_vs_plan_bp {
        s.push_str(&format!("  {}: {}\n", p.code(), pct(*bp)));
    }
    s.push_str(&format!(
        "== Ponovljivost (§11.5): {} ==\n",
        if r.reproducible { "da" } else { "ne" }
    ));
    s
}
