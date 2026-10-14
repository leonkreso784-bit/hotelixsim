//! Činjenice obnove v2 za mentora i „Zašto" (obnova-v2-spec §7): obnove kruga po dijelu, zgrada, amortizacija i
//! dobit nakon ulaganja, najamnina umanjena zbog stanja prostora i brže propadanje uz odgođeno održavanje. Rust ih
//! računa istim funkcijama kao obračun (`sim_hotel_v2`), mentor ih samo prepričava. v1 igra nema ništa od ovoga.

use engine::calc as c;
use serde::Serialize;
use sim_hotel::Maintenance;
use sim_hotel_v2::FinanceBasis;
use sim_hotel_v2::calendar::{Period, prorate, rounds_left_in_month};
use sim_hotel_v2::department::{
    DeptCode, DeptOutcome, RevenueModel, rent_after_condition, wear_pts,
};
use sim_hotel_v2::renewal::{AssetBook, RenewalSpec, gain_pts, monthly_depreciation};
use sim_hotel_v2::usali::statement;

use crate::dto::MonthCode;
use crate::error::{GameError, corrupt};
use crate::event::HotelResolution;
use crate::game::Game;
use crate::ids::HotelId;

/// Obnova jednog dijela (odjel ili zgrada) koja se plaća u krugu.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RenewalFact {
    /// Ime odjela iz sadržaja ili ime zgrade.
    pub name: String,
    /// `true` za zgradu hotela.
    pub building: bool,
    /// Mjesec obnove (odluka vrijedi cijeli mjesec).
    pub month: MonthCode,
    /// Odlučeni iznos za mjesec.
    pub total_cents: i64,
    pub paid_this_round_cents: i64,
    /// Koliko krugova mjeseca nakon ovoga još plaća (0 = plaćeno do kraja).
    pub rounds_left: u8,
    /// Bodovi stanja (0–10 000) koje je obnova donijela u ovom krugu, nakon granice 100.
    pub gained_this_round_pts: i32,
    /// Učinak cijelog iznosa u mjesecu (krivulja F3).
    pub gain_month_pts: i32,
    /// Učinak istog iznosa u dvije mjesečne faze: `E(a₁) + E(A − a₁)`, `a₁ = ⌊A/2⌋` zaokruženo dolje na korak.
    pub split_gain_pts: i32,
    /// Vijek amortizacije u godinama (zaokruženo); 0 = bez amortizacije.
    pub life_years: u16,
    /// Mjesečna amortizacija iznosa, od idućeg mjeseca.
    pub depreciation_monthly_cents: i64,
    /// Odluka je donesena u ovom krugu; `false` = zaključana iz ranijeg kruga istog mjeseca (nije nova odluka).
    pub decided_this_round: bool,
}

/// Brže trošenje prostora odjela u krugu jer je održavanje bilo odgođeno.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeferredWearFact {
    pub name: String,
    /// Dodatni pad stanja prostora odjela (bodovi 0–10 000, negativno) prema trošenju uz redovno održavanje.
    pub extra_pts: i32,
    /// Faktor trošenja uz odgođeno održavanje (bp; 12 500 = 1,25 puta).
    pub factor_bp: u16,
}

/// Najamnina umanjena zbog stanja prostora (lokal).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RentCutFact {
    pub name: String,
    /// Za koliko je najamnina kruga manja od ugovorene.
    pub cut_cents: i64,
    /// Stanje prostora nakon kruga i prag ispod kojeg najamnina pada (bodovi 0–10 000).
    pub condition_pts: i32,
    pub threshold_pts: i32,
}

/// Sve činjenice v2 kruga za ulaz mentora; za v1 prazno.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DebriefV2 {
    pub renewals: Vec<RenewalFact>,
    pub building_before_pts: Option<i32>,
    pub building_after_pts: Option<i32>,
    pub depreciation_round_cents: Option<i64>,
    pub ebit_cents: Option<i64>,
    pub capex_cents: Option<i64>,
    pub gop_cents: Option<i64>,
    pub finance_basis: Option<FinanceBasis>,
    pub deferred_wear: Vec<DeferredWearFact>,
    pub rent_cuts: Vec<RentCutFact>,
}

fn calc(e: c::CalcError) -> GameError {
    corrupt(format!("mentor v2: {e:?}"))
}

fn catalog(e: impl std::fmt::Debug) -> GameError {
    corrupt(format!("katalog v2: {e:?}"))
}

/// Učinak iznosa raspoređenog na dva mjeseca: prva faza `⌊A/2⌋` zaokruženo dolje na korak, druga ostatak (isti
/// ukupni iznos). Bez koraka ili s iznosom manjim od dva koraka jednako je učinku cijelog iznosa.
pub(crate) fn split_gain_pts(spec: &RenewalSpec, amount_cents: i64) -> c::Calc<i32> {
    let step = i128::from(spec.step_cents.max(1));
    let half = c::div_floor(i128::from(amount_cents), 2)?;
    let first = c::mul(c::div_floor(half, step)?, step)?;
    let second = c::sub(i128::from(amount_cents), first)?;
    let total = c::add(
        i128::from(gain_pts(spec, c::narrow(first)?)?),
        i128::from(gain_pts(spec, c::narrow(second)?)?),
    )?;
    c::narrow(total)
}

/// Obnova dijela iz knjige nakon kruga, ako se plaća u mjesecu kruga.
fn renewal_fact(
    name: String,
    building: bool,
    book: &AssetBook,
    spec: &RenewalSpec,
    p: &Period,
    paid_cents: i64,
    gained_pts: i32,
) -> Result<Option<RenewalFact>, GameError> {
    let Some(r) = book.locked_in(p.start_month()) else {
        return Ok(None);
    };
    let paid_before = c::sub(i128::from(r.paid_cents), i128::from(paid_cents)).map_err(calc)?;
    Ok(Some(RenewalFact {
        name,
        building,
        month: r.month.into(),
        total_cents: r.total_cents,
        paid_this_round_cents: paid_cents,
        rounds_left: rounds_left_in_month(p).saturating_sub(1),
        gained_this_round_pts: gained_pts,
        gain_month_pts: r.gain_pts,
        split_gain_pts: split_gain_pts(spec, r.total_cents).map_err(calc)?,
        life_years: c::narrow(c::div_round(i128::from(spec.life_months), 12).map_err(calc)?)
            .map_err(calc)?,
        depreciation_monthly_cents: monthly_depreciation(r.total_cents, spec.life_months)
            .map_err(calc)?,
        decided_this_round: paid_before == 0,
    }))
}

impl Game {
    /// Činjenice obnove v2 obračunatog kruga `res`; v1 igra daje prazno.
    pub(crate) fn debrief_v2(
        &self,
        hotel: HotelId,
        p: &Period,
        res: &HotelResolution,
    ) -> Result<DebriefV2, GameError> {
        let accounts = match self.model().accounts(res) {
            Ok(a) => a,
            Err(GameError::WrongModel) => return Ok(DebriefV2::default()),
            Err(e) => return Err(e),
        };
        let cat = self.model().require_v2(&self.content)?;
        let profile = self.hotel(hotel)?.profile;
        let after = self.model().state_after(res)?;
        let state = self.model().v2_state(&after)?;
        let st = statement(&accounts).map_err(calc)?;

        // Redom računa dobiti (klasa USALI, pa šifra), zgrada zadnja — kao obnove sezone.
        let mut ranked: Vec<(&DeptCode, &DeptOutcome)> = accounts.departments.iter().collect();
        ranked.sort_by_key(|(code, o)| (o.class.rank(), *code));
        let mut renewals = Vec::new();
        let mut deferred_wear = Vec::new();
        let mut rent_cuts = Vec::new();
        for (code, o) in ranked {
            let spec = cat.dept(code).map_err(catalog)?;
            let ds = state
                .departments
                .get(code)
                .ok_or_else(|| corrupt("odjel iz računa bez stanja"))?;
            let name = cat.dept_name(profile, code).map_err(catalog)?;
            if let Some(f) = renewal_fact(
                name.clone(),
                false,
                &ds.book,
                &spec.renewal,
                p,
                o.renewal_cents,
                o.renewal_pts,
            )? {
                renewals.push(f);
            }
            let regular = wear_pts(
                spec.wear_pts_per_month,
                Maintenance::Regular,
                cat.economy.deferred_wear_bp,
                p,
            )
            .map_err(calc)?;
            let extra: i32 =
                c::narrow(c::sub(i128::from(o.wear_pts), i128::from(regular)).map_err(calc)?)
                    .map_err(calc)?;
            if extra < 0 {
                deferred_wear.push(DeferredWearFact {
                    name: name.clone(),
                    extra_pts: extra,
                    factor_bp: cat.economy.deferred_wear_bp,
                });
            }
            if let (RevenueModel::Fixed { monthly_cents, .. }, Some(rule)) =
                (&spec.revenue, spec.rent_condition.as_ref())
            {
                let full = prorate(*monthly_cents, p).map_err(calc)?;
                let cut = c::sub(
                    i128::from(full),
                    i128::from(
                        rent_after_condition(
                            full,
                            Some(rule),
                            ds.start_condition_pts,
                            o.condition_after_pts,
                        )
                        .map_err(calc)?,
                    ),
                )
                .map_err(calc)?;
                if cut > 0 {
                    rent_cuts.push(RentCutFact {
                        name,
                        cut_cents: c::narrow(cut).map_err(calc)?,
                        condition_pts: o.condition_after_pts,
                        threshold_pts: rule.threshold_pts.unwrap_or(ds.start_condition_pts),
                    });
                }
            }
        }
        let b = &accounts.building;
        if let Some(f) = renewal_fact(
            cat.economy.building.name.clone(),
            true,
            &state.building.book,
            &cat.economy.building.renewal,
            p,
            b.renewal_cents,
            b.renewal_pts,
        )? {
            renewals.push(f);
        }
        Ok(DebriefV2 {
            renewals,
            building_before_pts: Some(b.before_pts),
            building_after_pts: Some(b.after_pts),
            depreciation_round_cents: Some(st.depreciation_cents),
            ebit_cents: Some(st.ebit_cents),
            capex_cents: Some(st.capex_cents),
            gop_cents: Some(st.gop_cents),
            finance_basis: Some(cat.economy.finance_index),
            deferred_wear,
            rent_cuts,
        })
    }
}
