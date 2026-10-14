//! Račun dobiti po USALI-ju (11. izdanje, bez naknade za upravljanje i informatike — 🟨 P10) iz `PeriodAccounts` i
//! usporedba ostvarenog s budžetom po retku. Čisto: samo cijeli brojevi kroz `engine::calc`.

use std::collections::{BTreeMap, BTreeSet};

use engine::calc::{self as c, Calc};

use crate::department::{
    DeptCode, DeptOutcome, PeriodAccounts, UndistributedKind, UndistributedOutcome, UsaliClass,
};

fn w(x: impl Into<i128>) -> i128 {
    x.into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptAccount {
    pub code: DeptCode,
    pub class: UsaliClass,
    pub revenue_cents: i64,
    pub payroll_cents: i64,
    pub cost_of_sales_cents: i64,
    pub other_cents: i64,
    pub expenses_cents: i64,
    pub profit_cents: i64,
    pub margin_bp: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndistributedAccount {
    pub code: DeptCode,
    pub kind: UndistributedKind,
    pub payroll_cents: i64,
    pub other_cents: i64,
    pub total_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsaliStatement {
    pub days: u32,
    pub room_nights: u32,
    /// Smještaj, hrana i piće, ostali operativni, ostali prihodi (najam, parking) — redom `UsaliClass::rank`, pa kod.
    pub departments: Vec<DeptAccount>,
    pub total_revenue_cents: i64,
    pub total_departmental_expenses_cents: i64,
    pub departmental_profit_cents: i64,
    pub undistributed: Vec<UndistributedAccount>,
    pub total_undistributed_cents: i64,
    pub gop_cents: i64,
    pub gop_margin_bp: Option<i64>,
    pub goppar_cents: Option<i64>,
    pub fixed_charges: Vec<(String, i64)>,
    pub total_fixed_charges_cents: i64,
    pub ebitda_cents: i64,
    pub ebitda_margin_bp: Option<i64>,
    /// Amortizacija opreme (odjeli) i zgrade iz obnove v2, ispod EBITDA; EBIT = EBITDA − amortizacija.
    pub depreciation_equipment_cents: i64,
    pub depreciation_building_cents: i64,
    pub depreciation_cents: i64,
    pub ebit_cents: i64,
    pub ebit_margin_bp: Option<i64>,
    pub interest_cents: i64,
    /// Ulaganja (stara obnova i obnova v2) plaćena iz blagajne; nisu trošak, u račun ulazi samo amortizacija.
    pub capex_cents: i64,
}

/// Iznosi ispod EBITDA koje račun nosi uz retke: kamata, ulaganja, amortizacija opreme i zgrade.
struct BelowEbitda {
    interest: i64,
    capex: i64,
    depreciation_equipment: i64,
    depreciation_building: i64,
}

/// `div_round(part·10 000, whole)`; `None` kad je `whole` 0 (marža bez prihoda nije definirana).
fn margin_bp(part: i64, whole: i64) -> Calc<Option<i64>> {
    if whole == 0 {
        return Ok(None);
    }
    Ok(Some(c::narrow(c::div_round(
        c::mul(w(part), 10_000)?,
        w(whole),
    )?)?))
}

fn total(mut xs: impl Iterator<Item = i64>) -> Calc<i64> {
    c::narrow(xs.try_fold(0_i128, |acc, x| c::add(acc, w(x)))?)
}

fn add64(a: i64, b: i64) -> Calc<i64> {
    c::narrow(c::add(w(a), w(b))?)
}

fn sub64(a: i64, b: i64) -> Calc<i64> {
    c::narrow(c::sub(w(a), w(b))?)
}

fn dept_account(o: &DeptOutcome) -> Calc<DeptAccount> {
    finish_dept(DeptAccount {
        code: o.code.clone(),
        class: o.class,
        revenue_cents: o.revenue_cents,
        payroll_cents: o.payroll_cents,
        cost_of_sales_cents: o.cost_of_sales_cents,
        other_cents: o.other_direct_cents,
        expenses_cents: 0,
        profit_cents: 0,
        margin_bp: None,
    })
}

/// Rashodi, rezultat i marža odjela iz njegovih izvornih iznosa.
fn finish_dept(mut d: DeptAccount) -> Calc<DeptAccount> {
    d.expenses_cents = c::narrow(c::sum(&[
        w(d.payroll_cents),
        w(d.cost_of_sales_cents),
        w(d.other_cents),
    ])?)?;
    d.profit_cents = sub64(d.revenue_cents, d.expenses_cents)?;
    d.margin_bp = margin_bp(d.profit_cents, d.revenue_cents)?;
    Ok(d)
}

fn undistributed_account(u: &UndistributedOutcome) -> Calc<UndistributedAccount> {
    let a = &u.assigned;
    let other = c::sum(&[
        w(u.fixed_cents),
        w(u.per_room_cents),
        w(a.marketing_cents),
        w(a.reply_cents),
        w(a.maintenance_cents),
        w(a.training_cents),
        w(a.hiring_cents),
        w(a.situation_cents),
    ])?;
    Ok(UndistributedAccount {
        code: u.code.clone(),
        kind: u.kind,
        payroll_cents: u.payroll_cents,
        other_cents: c::narrow(other)?,
        total_cents: c::narrow(c::add(w(u.payroll_cents), other)?)?,
    })
}

/// Zajednički završetak: zbrojevi, GOP, EBITDA, EBIT i marže iz već složenih redaka.
fn assemble(
    (days, room_nights): (u32, u32),
    mut departments: Vec<DeptAccount>,
    mut undistributed: Vec<UndistributedAccount>,
    fixed_charges: Vec<(String, i64)>,
    below: BelowEbitda,
) -> Calc<UsaliStatement> {
    departments.sort_by(|a, b| (a.class.rank(), &a.code).cmp(&(b.class.rank(), &b.code)));
    undistributed.sort_by(|a, b| {
        (UsaliClass::Undistributed(a.kind).rank(), &a.code)
            .cmp(&(UsaliClass::Undistributed(b.kind).rank(), &b.code))
    });
    let total_revenue = total(departments.iter().map(|d| d.revenue_cents))?;
    let total_expenses = total(departments.iter().map(|d| d.expenses_cents))?;
    let dept_profit = sub64(total_revenue, total_expenses)?;
    let total_undist = total(undistributed.iter().map(|u| u.total_cents))?;
    let gop = sub64(dept_profit, total_undist)?;
    let total_fixed = total(fixed_charges.iter().map(|(_, v)| *v))?;
    let ebitda = sub64(gop, total_fixed)?;
    let depreciation = add64(below.depreciation_equipment, below.depreciation_building)?;
    let ebit = sub64(ebitda, depreciation)?;
    let goppar = if room_nights == 0 {
        None
    } else {
        Some(c::narrow(c::div_round(w(gop), w(room_nights))?)?)
    };
    Ok(UsaliStatement {
        days,
        room_nights,
        departments,
        total_revenue_cents: total_revenue,
        total_departmental_expenses_cents: total_expenses,
        departmental_profit_cents: dept_profit,
        undistributed,
        total_undistributed_cents: total_undist,
        gop_cents: gop,
        gop_margin_bp: margin_bp(gop, total_revenue)?,
        goppar_cents: goppar,
        fixed_charges,
        total_fixed_charges_cents: total_fixed,
        ebitda_cents: ebitda,
        ebitda_margin_bp: margin_bp(ebitda, total_revenue)?,
        depreciation_equipment_cents: below.depreciation_equipment,
        depreciation_building_cents: below.depreciation_building,
        depreciation_cents: depreciation,
        ebit_cents: ebit,
        ebit_margin_bp: margin_bp(ebit, total_revenue)?,
        interest_cents: below.interest,
        capex_cents: below.capex,
    })
}

/// Račun dobiti jednog razdoblja. Rashodi odjela = plaće + nabavna vrijednost + ostali izravni (održavanje je u
/// „Održavanju"); `room_nights = sobe · dani`; ispod EBITDA amortizacija (daje EBIT, koji ulazi u indeks F kad
/// sadržaj tako kaže). Kamata i ulaganja (capex) nisu retci računa: ulaganja su plaćena iz blagajne (ispod EBITDA,
/// kao promjena blagajne), a u račun dobiti ulaze samo kroz amortizaciju.
pub fn statement(a: &PeriodAccounts) -> Calc<UsaliStatement> {
    let departments = a
        .departments
        .values()
        .map(dept_account)
        .collect::<Calc<Vec<_>>>()?;
    let undistributed = a
        .undistributed
        .values()
        .map(undistributed_account)
        .collect::<Calc<Vec<_>>>()?;
    let fixed: Vec<(String, i64)> = a
        .fixed_charges
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    let days = u32::from(a.days);
    let room_nights: u32 = c::narrow(c::mul(w(a.rooms), w(days))?)?;
    assemble(
        (days, room_nights),
        departments,
        undistributed,
        fixed,
        BelowEbitda {
            interest: a.interest_cents,
            capex: a.capex_cents,
            depreciation_equipment: a.depreciation_equipment_cents,
            depreciation_building: a.depreciation_building_cents,
        },
    )
}

/// Zbroj više krugova (npr. sezona do sada); marže i GOPPAR iz zbrojeva. Prazan popis daje prazan račun.
pub fn sum_statements(xs: &[UsaliStatement]) -> Calc<UsaliStatement> {
    let mut depts: BTreeMap<DeptCode, DeptAccount> = BTreeMap::new();
    let mut undist: BTreeMap<DeptCode, UndistributedAccount> = BTreeMap::new();
    let mut fixed: BTreeMap<String, i64> = BTreeMap::new();
    let (mut days, mut nights) = (0_i128, 0_i128);
    let mut below = BelowEbitda {
        interest: 0,
        capex: 0,
        depreciation_equipment: 0,
        depreciation_building: 0,
    };
    for x in xs {
        days = c::add(days, w(x.days))?;
        nights = c::add(nights, w(x.room_nights))?;
        below.interest = add64(below.interest, x.interest_cents)?;
        below.capex = add64(below.capex, x.capex_cents)?;
        below.depreciation_equipment =
            add64(below.depreciation_equipment, x.depreciation_equipment_cents)?;
        below.depreciation_building =
            add64(below.depreciation_building, x.depreciation_building_cents)?;
        for d in &x.departments {
            let e = depts.entry(d.code.clone()).or_insert_with(|| DeptAccount {
                revenue_cents: 0,
                payroll_cents: 0,
                cost_of_sales_cents: 0,
                other_cents: 0,
                ..d.clone()
            });
            e.revenue_cents = add64(e.revenue_cents, d.revenue_cents)?;
            e.payroll_cents = add64(e.payroll_cents, d.payroll_cents)?;
            e.cost_of_sales_cents = add64(e.cost_of_sales_cents, d.cost_of_sales_cents)?;
            e.other_cents = add64(e.other_cents, d.other_cents)?;
        }
        for u in &x.undistributed {
            let e = undist
                .entry(u.code.clone())
                .or_insert_with(|| UndistributedAccount {
                    payroll_cents: 0,
                    other_cents: 0,
                    total_cents: 0,
                    ..u.clone()
                });
            e.payroll_cents = add64(e.payroll_cents, u.payroll_cents)?;
            e.other_cents = add64(e.other_cents, u.other_cents)?;
            e.total_cents = add64(e.total_cents, u.total_cents)?;
        }
        for (k, v) in &x.fixed_charges {
            let e = fixed.entry(k.clone()).or_insert(0);
            *e = add64(*e, *v)?;
        }
    }
    let departments = depts
        .into_values()
        .map(finish_dept)
        .collect::<Calc<Vec<_>>>()?;
    assemble(
        (c::narrow(days)?, c::narrow(nights)?),
        departments,
        undist.into_values().collect(),
        fixed.into_iter().collect(),
        below,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Revenue,
    DepartmentalExpenses,
    DepartmentalProfit,
    Undistributed,
    Gop,
    FixedCharges,
    Ebitda,
    /// Amortizacija iz obnove v2 (oprema odjela, zgrada, ukupno), ispod EBITDA.
    Depreciation,
    /// EBIT = EBITDA − amortizacija.
    Ebit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Revenue,
    Expense,
    Result,
}

/// `is_total` označava samo retke "total" sekcija s više redaka; GOP, EBITDA i EBIT su pojedinačni retci vrste `Result`
/// s `is_total = false` — sučelje ih mora isticati po sekciji/vrsti, ne po `is_total`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarianceLine {
    pub section: Section,
    pub code: String,
    pub kind: LineKind,
    pub is_total: bool,
    pub actual_cents: i64,
    pub budget_cents: i64,
    pub variance_cents: i64,
    pub favorable: bool,
}

/// Povoljno: prihod i rezultat ≥ 0, trošak ≤ 0. Razlika je zasićena: iznosi računa su daleko od granice `i64`, a
/// funkcija po potpisu ne može javiti grešku.
fn line(
    section: Section,
    code: &str,
    kind: LineKind,
    is_total: bool,
    (actual, budget): (i64, i64),
) -> VarianceLine {
    let variance = actual.saturating_sub(budget);
    let favorable = match kind {
        LineKind::Revenue | LineKind::Result => variance >= 0,
        LineKind::Expense => variance <= 0,
    };
    VarianceLine {
        section,
        code: code.to_string(),
        kind,
        is_total,
        actual_cents: actual,
        budget_cents: budget,
        variance_cents: variance,
        favorable,
    }
}

/// Redoci jedne sekcije: po jedan za svaki ključ iz unije (kojeg nema → 0), pa redak zbroja.
fn section_lines(
    out: &mut Vec<VarianceLine>,
    section: Section,
    kind: LineKind,
    rows: &BTreeMap<(u8, String), (i64, i64)>,
    total: (i64, i64),
) {
    for ((_, code), pair) in rows {
        out.push(line(section, code, kind, false, *pair));
    }
    out.push(line(section, "total", kind, true, total));
}

/// Retci oba izvješća (unija kodova; kojeg nema → 0), redom sekcija; `variance = actual − budget`. Svaka sekcija s
/// više redaka ima i redak `is_total` (kod "total"); Gop, Ebitda i Ebit imaju po jedan redak ("gop", "ebitda",
/// "ebit", `is_total` false). Amortizacija ima retke "equipment" i "building" pa "total" (obnova v2; budžet je zadani
/// plan bez obnove, pa 0). Odjeli su poredani kao u računu dobiti: rang razreda, pa kod.
pub fn budget_variance(actual: &UsaliStatement, budget: &UsaliStatement) -> Vec<VarianceLine> {
    let dept_rows = |pick: fn(&DeptAccount) -> i64| {
        let mut m: BTreeMap<(u8, String), (i64, i64)> = BTreeMap::new();
        for d in &actual.departments {
            m.entry((d.class.rank(), d.code.0.clone())).or_default().0 = pick(d);
        }
        for d in &budget.departments {
            m.entry((d.class.rank(), d.code.0.clone())).or_default().1 = pick(d);
        }
        m
    };
    let mut undist: BTreeMap<(u8, String), (i64, i64)> = BTreeMap::new();
    for u in &actual.undistributed {
        let key = (UsaliClass::Undistributed(u.kind).rank(), u.code.0.clone());
        undist.entry(key).or_default().0 = u.total_cents;
    }
    for u in &budget.undistributed {
        let key = (UsaliClass::Undistributed(u.kind).rank(), u.code.0.clone());
        undist.entry(key).or_default().1 = u.total_cents;
    }
    let names: BTreeSet<&String> = actual
        .fixed_charges
        .iter()
        .chain(&budget.fixed_charges)
        .map(|(k, _)| k)
        .collect();
    let fixed_of = |s: &UsaliStatement, k: &str| {
        s.fixed_charges
            .iter()
            .find(|(n, _)| n == k)
            .map_or(0, |(_, v)| *v)
    };

    let mut out = Vec::new();
    section_lines(
        &mut out,
        Section::Revenue,
        LineKind::Revenue,
        &dept_rows(|d| d.revenue_cents),
        (actual.total_revenue_cents, budget.total_revenue_cents),
    );
    section_lines(
        &mut out,
        Section::DepartmentalExpenses,
        LineKind::Expense,
        &dept_rows(|d| d.expenses_cents),
        (
            actual.total_departmental_expenses_cents,
            budget.total_departmental_expenses_cents,
        ),
    );
    section_lines(
        &mut out,
        Section::DepartmentalProfit,
        LineKind::Result,
        &dept_rows(|d| d.profit_cents),
        (
            actual.departmental_profit_cents,
            budget.departmental_profit_cents,
        ),
    );
    section_lines(
        &mut out,
        Section::Undistributed,
        LineKind::Expense,
        &undist,
        (
            actual.total_undistributed_cents,
            budget.total_undistributed_cents,
        ),
    );
    out.push(line(
        Section::Gop,
        "gop",
        LineKind::Result,
        false,
        (actual.gop_cents, budget.gop_cents),
    ));
    for k in names {
        out.push(line(
            Section::FixedCharges,
            k,
            LineKind::Expense,
            false,
            (fixed_of(actual, k), fixed_of(budget, k)),
        ));
    }
    out.push(line(
        Section::FixedCharges,
        "total",
        LineKind::Expense,
        true,
        (
            actual.total_fixed_charges_cents,
            budget.total_fixed_charges_cents,
        ),
    ));
    out.push(line(
        Section::Ebitda,
        "ebitda",
        LineKind::Result,
        false,
        (actual.ebitda_cents, budget.ebitda_cents),
    ));
    let depreciation = BTreeMap::from([
        (
            (0, "equipment".to_string()),
            (
                actual.depreciation_equipment_cents,
                budget.depreciation_equipment_cents,
            ),
        ),
        (
            (1, "building".to_string()),
            (
                actual.depreciation_building_cents,
                budget.depreciation_building_cents,
            ),
        ),
    ]);
    section_lines(
        &mut out,
        Section::Depreciation,
        LineKind::Expense,
        &depreciation,
        (actual.depreciation_cents, budget.depreciation_cents),
    );
    out.push(line(
        Section::Ebit,
        "ebit",
        LineKind::Result,
        false,
        (actual.ebit_cents, budget.ebit_cents),
    ));
    out
}
