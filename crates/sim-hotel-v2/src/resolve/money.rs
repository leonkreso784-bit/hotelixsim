//! Faza novca (4): troškovi odluka kruga i v1 sažetak novca složen iz računa odjela (prihodi, troškovi, GOP), pa
//! blagajna nakon fiksnih naknada i ulaganja (stara obnova i obnova v2), kamata i GOPPAR kruga; zatim EBITDA,
//! amortizacija, EBIT i EBITPAR (`finance`).

use engine::calc::{self as c, Calc};
use sim_hotel::MoneyOutcome;

use super::departments::Decisions;
use super::{neg, training_effect, w};
use crate::calendar::{Period, prorate};
use crate::department::{DeptCode, DeptSpec, PeriodAccounts};
use crate::plan::PeriodPlan;
use crate::state::HotelState;

/// EBITDA, amortizacija, EBIT i EBIT po raspoloživoj sobi i danu kruga (obnova-v2-spec §3.9). `MoneyOutcome` je v1
/// tip i ne mijenja se, pa ovo ide uz njega.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinanceOutcome {
    pub ebitda_cents: i64,
    pub depreciation_cents: i64,
    pub ebit_cents: i64,
    pub ebitpar_cents: i64,
}

/// EBITDA = GOP − fiksne naknade; EBIT = EBITDA − amortizacija (oprema i zgrada); EBITPAR = `div_round(EBIT, sobe ·
/// dani)` (isti nazivnik kao GOPPAR).
pub(super) fn finance(gop: i64, a: &PeriodAccounts) -> Calc<FinanceOutcome> {
    let charges = a
        .fixed_charges
        .values()
        .try_fold(0_i128, |s, v| c::add(s, w(*v)))?;
    let ebitda = c::sub(w(gop), charges)?;
    let depreciation = c::add(
        w(a.depreciation_equipment_cents),
        w(a.depreciation_building_cents),
    )?;
    let ebit = c::sub(ebitda, depreciation)?;
    let ebitpar = c::div_round(ebit, c::mul(w(a.rooms), w(a.days))?)?;
    Ok(FinanceOutcome {
        ebitda_cents: c::narrow(ebitda)?,
        depreciation_cents: c::narrow(depreciation)?,
        ebit_cents: c::narrow(ebit)?,
        ebitpar_cents: c::narrow(ebitpar)?,
    })
}

/// Troškovi odluka kruga: marketing (iznos iz plana je za krug), odgovaranje na recenzije i obuka razmjerno,
/// zapošljavanje (najam i odlasci) i situacije u punom iznosu; održavanje je zbroj odjela (`maintenance`).
pub(super) fn decisions(
    plan: &PeriodPlan,
    (hires, departures): (u16, u16),
    maintenance: i64,
    sit_cost: i128,
    p: &Period,
    cat: &sim_hotel::Catalog,
) -> Calc<Decisions> {
    let hiring = c::mul(c::add(w(hires), w(departures))?, w(cat.economy.hire_cents))?;
    let reply = if plan.reply_reviews {
        prorate(cat.actions.reply_cents, p)?
    } else {
        0
    };
    Ok(Decisions {
        marketing: plan.marketing_cents,
        reply,
        maintenance,
        training: training_effect(plan.training, cat, p)?.cost_cents,
        hiring: c::narrow(hiring)?,
        situation: c::narrow(sit_cost)?,
    })
}

/// Trošak održavanja Smještaja u krugu (razina iz plana, cijena iz odjela Smještaj; degenerirani katalog: v1).
pub(super) fn rooms_maintenance_cost(plan: &PeriodPlan, p: &Period, rooms: &DeptSpec) -> Calc<i64> {
    prorate(
        rooms.maintenance_level(plan.rooms_maintenance()).cost_cents,
        p,
    )
}

pub(crate) struct MoneyStep {
    pub(crate) out: MoneyOutcome,
    pub(crate) cash: i128,
    /// Σ plaće svih odjela (Smještaj s povišicama iz situacija).
    pub(crate) wage: i128,
    pub(crate) hire_cost: i128,
    pub(crate) training_cost: i128,
    /// Σ nabavna vrijednost + Σ ostali izravni troškovi odjela + Σ trošak po prodanoj sobi neraspoređenih.
    pub(crate) direct: i128,
    /// Σ održavanje svih odjela (knjiženo u „Održavanju").
    pub(crate) maintenance_cost: i128,
    pub(crate) reply_cost: i128,
    /// Σ prihodi odjela osim Smještaja.
    pub(crate) other: i128,
    /// Σ stalni dio neraspoređenih odjela.
    pub(crate) fixed: i128,
    /// Σ fiksne naknade (ispod GOP-a).
    pub(crate) charges: i128,
}

pub(super) struct MoneyIn<'a> {
    pub(super) state: &'a HotelState,
    pub(super) plan: &'a PeriodPlan,
    pub(super) accounts: &'a PeriodAccounts,
    pub(super) decisions: &'a Decisions,
    pub(super) period: &'a Period,
    /// Obnova v2 plaćena u krugu (odjeli i zgrada); s njom ide stara obnova u ulaganja (capex).
    pub(super) renewal_cents: i128,
}

/// v1 sažetak novca iz računa odjela: prihod soba = Smještaj, ostali prihod = Σ ostalih odjela; trošak = rad (Σ
/// plaće + zapošljavanje + obuka) + izravni + održavanje + marketing + odgovaranje + stalni neraspoređeni +
/// situacije, što je točno Σ rashodi odjela + Σ neraspoređeni; GOP = prihod − trošak; blagajna = blagajna + GOP −
/// fiksne naknade − obnova, pa kamata na negativnu blagajnu.
pub(super) fn money(i: &MoneyIn, cat: &sim_hotel::Catalog) -> Calc<MoneyStep> {
    let p = i.period;
    let a = i.accounts;
    let d = i.decisions;
    let (mut rooms_revenue, mut other, mut wage, mut direct, mut maintenance_cost) =
        (0_i128, 0_i128, 0_i128, 0_i128, 0_i128);
    for (code, o) in &a.departments {
        if *code == DeptCode::rooms() {
            rooms_revenue = c::add(rooms_revenue, w(o.revenue_cents))?;
        } else {
            other = c::add(other, w(o.revenue_cents))?;
        }
        wage = c::add(wage, w(o.payroll_cents))?;
        direct = c::sum(&[direct, w(o.cost_of_sales_cents), w(o.other_direct_cents)])?;
        maintenance_cost = c::add(maintenance_cost, w(o.maintenance_cents))?;
    }
    let mut fixed: i128 = 0;
    for u in a.undistributed.values() {
        wage = c::add(wage, w(u.payroll_cents))?;
        direct = c::add(direct, w(u.per_room_cents))?;
        fixed = c::add(fixed, w(u.fixed_cents))?;
    }
    let mut charges: i128 = 0;
    for v in a.fixed_charges.values() {
        charges = c::add(charges, w(*v))?;
    }
    let (hire_cost, training_cost, reply_cost) = (w(d.hiring), w(d.training), w(d.reply));
    let labor = c::sum(&[wage, hire_cost, training_cost])?;
    let costs = c::sum(&[
        labor,
        direct,
        maintenance_cost,
        w(d.marketing),
        fixed,
        reply_cost,
        w(d.situation),
    ])?;
    let gop = c::sub(c::add(rooms_revenue, other)?, costs)?;
    let old = if i.plan.renovation {
        w(cat.actions.renovation_cents)
    } else {
        0
    };
    let capex = c::add(old, i.renewal_cents)?;
    let mut cash = c::sub(c::sub(c::add(w(i.state.cash_cents), gop)?, charges)?, capex)?;
    let mut interest = 0;
    if cash < 0 {
        // Mjesečna kamata razmjerno danima kruga (nazivnik su dani mjeseca u kojem krug počinje).
        interest = c::mul_div(
            c::mul(neg(cash)?, w(cat.economy.interest_bp))?,
            w(p.days),
            c::mul(10_000, w(p.month_days_at_start()))?,
        )?;
        cash = c::sub(cash, interest)?;
    }
    let goppar = c::div_round(gop, c::mul(w(i.state.rooms), w(p.days))?)?;
    Ok(MoneyStep {
        out: MoneyOutcome {
            rooms_revenue_cents: c::narrow(rooms_revenue)?,
            other_revenue_cents: c::narrow(other)?,
            labor_cents: c::narrow(labor)?,
            costs_cents: c::narrow(costs)?,
            gop_cents: c::narrow(gop)?,
            capex_cents: c::narrow(capex)?,
            interest_cents: c::narrow(interest)?,
            goppar_cents: c::narrow(goppar)?,
        },
        cash,
        wage,
        hire_cost,
        training_cost,
        direct,
        maintenance_cost,
        reply_cost,
        other,
        fixed,
        charges,
    })
}
