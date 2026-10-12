//! Obnova v2 za ekran (obnova-v2-spec §4.4): izbor iznosa po odjelu i za zgradu (rate, učinak, amortizacija),
//! zaključana obnova mjeseca, stanje zgrade, smjernica ulaganja (FF&E pričuva) i obnove sezone. Sve brojke računa
//! `sim_hotel_v2::renewal` (iste funkcije kao obračun); ovdje se samo slažu. v1 igra nema ništa od ovoga (`None`).

use std::collections::BTreeMap;

use engine::calc as c;
use sim_hotel::Month;
use sim_hotel_v2::calendar::rounds_left_in_month;
use sim_hotel_v2::renewal::{
    AssetBook, Renewal, RenewalSpec, gain_pts, gained_pts, installments, monthly_depreciation,
};
use sim_hotel_v2::usali::statement;
use sim_hotel_v2::{Period, RenewalLimits};

use crate::error::{GameError, corrupt};
use crate::game::Game;
use crate::ids::HotelId;

/// Obnova odlučena u mjesecu, zaključana do kraja mjeseca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenewalStatus {
    pub month: Month,
    pub total_cents: i64,
    /// Rata svakog kruga mjeseca osim zadnjeg (zadnji nosi ostatak).
    pub installment_cents: i64,
    /// Plaćeno do sada.
    pub paid_cents: i64,
    /// Koliko krugova mjeseca još plaća (0 = plaćeno).
    pub rounds_left: u8,
    /// Učinak cijelog iznosa (bodovi 0–10 000), fiksan pri odluci.
    pub gain_pts: i32,
    /// Učinak plaćenog do sada (prije granice 100).
    pub gained_pts: i32,
}

/// Jedan iznos u izboru obnove.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenewalOption {
    pub amount_cents: i64,
    /// Plaća se ovaj i svaki idući krug mjeseca osim zadnjeg.
    pub installment_cents: i64,
    /// Zadnji krug mjeseca (ostatak); u mjesečnoj igri cijeli iznos.
    pub last_installment_cents: i64,
    /// Učinak cijelog iznosa za mjesec (F3).
    pub gain_pts: i32,
    /// `min(učinak, 10 000 − stanje sada)`: približno, jer se do kraja mjeseca stanje još troši i održava.
    pub gain_capped_pts: i32,
    /// Mjesečna amortizacija iznosa, od idućeg mjeseca.
    pub depreciation_monthly_cents: i64,
}

/// Izbor iznosa obnove jednog dijela (odjel ili zgrada) u krugu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenewalChoice {
    /// Mjesec kruga (odluka vrijedi taj mjesec).
    pub month: Month,
    /// Krugova mjeseca od ovoga (uključivo): na toliko se rata dijeli iznos odlučen sada.
    pub rounds_left: u8,
    pub max_monthly_cents: i64,
    pub step_cents: i64,
    pub life_months: u16,
    /// Stanje dijela sada (bodovi 0–10 000).
    pub condition_pts: i32,
    /// Obnova već odlučena u ovom mjesecu (ne mijenja se do kraja mjeseca).
    pub locked: Option<RenewalStatus>,
    /// 0, korak, 2·korak … najviše.
    pub options: Vec<RenewalOption>,
}

/// Smjernica ulaganja u sezoni: pričuva za obnovu (FF&E) kao udio prihoda sezone iz budžeta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenewalGuide {
    pub reserve_bp: u16,
    /// Prihod cijele sezone u budžetu (zadani plan, isti kao račun dobiti „budžet").
    pub season_revenue_budget_cents: i64,
    /// `div_round(prihod · udio, 10 000)`.
    pub guide_cents: i64,
    /// Uloženo do sada (svi odjeli i zgrada: kapitalizirano + plaćeno u tekućem mjesecu).
    pub invested_cents: i64,
}

/// Zgrada hotela (v2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingView {
    pub condition_pts: i32,
    pub start_pts: i32,
    pub wear_pts_per_month: i32,
    /// Obnova zgrade odlučena u mjesecu trenutnog kruga.
    pub locked: Option<RenewalStatus>,
    pub invested_cents: i64,
}

/// Obnova jednog dijela u jednom mjesecu sezone (kraj sezone).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonRenewal {
    /// Kod odjela ili `building`.
    pub asset_code: String,
    pub asset_name: String,
    pub month: Month,
    /// Plaćeno u tom mjesecu (zbroj rata = odlučeni iznos).
    pub amount_cents: i64,
}

/// Kod zgrade u popisima obnova.
pub const BUILDING_CODE: &str = "building";

fn calc(e: c::CalcError) -> GameError {
    corrupt(format!("obnova: {e:?}"))
}

/// Zaključana obnova s brojem krugova mjeseca koji još plaćaju.
pub(crate) fn status(r: &Renewal, rounds_left: u8) -> Result<RenewalStatus, GameError> {
    Ok(RenewalStatus {
        month: r.month,
        total_cents: r.total_cents,
        installment_cents: r.installment_cents,
        paid_cents: r.paid_cents,
        rounds_left,
        gain_pts: r.gain_pts,
        gained_pts: gained_pts(r).map_err(calc)?,
    })
}

/// Obnova iz knjige zaključana u mjesecu kruga `current` (trenutni krug još nije plaćen); bez kruga `None`.
pub(crate) fn locked_status(
    book: &AssetBook,
    current: Option<&Period>,
) -> Result<Option<RenewalStatus>, GameError> {
    let Some(p) = current else {
        return Ok(None);
    };
    book.locked_in(p.start_month())
        .map(|r| status(r, rounds_left_in_month(p)))
        .transpose()
}

fn option(
    spec: &RenewalSpec,
    lim: &RenewalLimits,
    condition_pts: i32,
    amount: i64,
) -> Result<RenewalOption, GameError> {
    let (installment, last) = installments(amount, lim.rounds_left).map_err(calc)?;
    let gain = gain_pts(spec, amount).map_err(calc)?;
    let room = c::narrow::<i32>(c::sub(10_000, i128::from(condition_pts)).map_err(calc)?)
        .map_err(calc)?
        .max(0);
    Ok(RenewalOption {
        amount_cents: amount,
        installment_cents: installment,
        last_installment_cents: last,
        gain_pts: gain,
        gain_capped_pts: gain.min(room),
        depreciation_monthly_cents: monthly_depreciation(amount, lim.life_months).map_err(calc)?,
    })
}

/// Izbor iznosa jednog dijela; `None` kad obnova nije ponuđena.
pub(crate) fn choice(
    spec: &RenewalSpec,
    lim: &RenewalLimits,
    condition_pts: i32,
) -> Result<Option<RenewalChoice>, GameError> {
    if !spec.offered() || lim.step_cents <= 0 {
        return Ok(None);
    }
    let mut options = Vec::new();
    let mut amount = 0_i64;
    while amount <= lim.max_monthly_cents {
        options.push(option(spec, lim, condition_pts, amount)?);
        amount = c::narrow(c::add(i128::from(amount), i128::from(lim.step_cents)).map_err(calc)?)
            .map_err(calc)?;
    }
    Ok(Some(RenewalChoice {
        month: lim.month,
        rounds_left: lim.rounds_left,
        max_monthly_cents: lim.max_monthly_cents,
        step_cents: lim.step_cents,
        life_months: lim.life_months,
        condition_pts,
        locked: lim
            .locked
            .map(|r| status(&r, lim.rounds_left))
            .transpose()?,
        options,
    }))
}

/// Izbori obnove v2 u granicama plana: zgrada i odjeli koji je nude (po kodu).
pub(crate) fn renewal_choices(
    s: &sim_hotel_v2::HotelState,
    l: &sim_hotel_v2::PlanLimits,
    cat: &sim_hotel_v2::Catalog,
) -> Result<(Option<RenewalChoice>, BTreeMap<String, RenewalChoice>), GameError> {
    let building = choice(
        &cat.economy.building.renewal,
        &l.building_renewal,
        s.building.condition_pts,
    )?;
    let mut depts = BTreeMap::new();
    for (code, d) in &l.departments {
        let spec = cat
            .dept(code)
            .map_err(|e| corrupt(format!("katalog v2: {e:?}")))?;
        let condition = s
            .departments
            .get(code)
            .map(|x| x.condition_pts)
            .ok_or_else(|| corrupt("odjel plana bez stanja"))?;
        if let Some(ch) = choice(&spec.renewal, &d.renewal, condition)? {
            depts.insert(code.as_str().to_owned(), ch);
        }
    }
    Ok((building, depts))
}

impl Game {
    /// Kalendar trenutnog (otvorenog ili sljedećeg) kruga; `None` kad je sezona gotova.
    fn current_period(&self) -> Result<Option<Period>, GameError> {
        self.current_round()
            .map(|r| self.round_info(r).map(|i| i.period))
            .transpose()
    }

    /// Katalog v2 ili `None` za v1 igru.
    fn v2_catalog_or_none(&self) -> Result<Option<&sim_hotel_v2::Catalog>, GameError> {
        match self.model().require_v2(&self.content) {
            Ok(cat) => Ok(Some(cat)),
            Err(GameError::WrongModel) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Zaključana obnova odjela u mjesecu trenutnog kruga (pogled odjela).
    pub(crate) fn dept_renewal_status(
        &self,
        book: &AssetBook,
    ) -> Result<Option<RenewalStatus>, GameError> {
        locked_status(book, self.current_period()?.as_ref())
    }

    /// Zgrada hotela (stanje na kraju zadnjeg obračunatog kruga); v1 igra je nema.
    pub(crate) fn building_view(&self, h: HotelId) -> Result<Option<BuildingView>, GameError> {
        let Some(cat) = self.v2_catalog_or_none()? else {
            return Ok(None);
        };
        let state = self.latest_state(h)?;
        let b = &self.model().v2_state(&state)?.building;
        Ok(Some(BuildingView {
            condition_pts: b.condition_pts,
            start_pts: b.start_pts,
            wear_pts_per_month: cat.economy.building.wear_pts_per_month,
            locked: locked_status(&b.book, self.current_period()?.as_ref())?,
            invested_cents: b.book.invested_cents(),
        }))
    }

    /// Smjernica ulaganja: udio `ffe_reserve_bp` prihoda sezone iz budžeta i uloženo do sada. `None` u v1 i u
    /// sadržaju bez obnove v2.
    pub(crate) fn renewal_guide(&self, h: HotelId) -> Result<Option<RenewalGuide>, GameError> {
        let Some(cat) = self.v2_catalog_or_none()? else {
            return Ok(None);
        };
        if !cat.renewal_enabled() {
            return Ok(None);
        }
        let rec = self.hotel(h)?;
        let budget = self.model().budget_accounts(
            &self.content,
            rec.profile,
            &self.rounds_info(),
            self.refs(rec.start_state.category())?,
        )?;
        let mut revenue = 0_i128;
        for a in &budget {
            let st = statement(a).map_err(calc)?;
            revenue = c::add(revenue, i128::from(st.total_revenue_cents)).map_err(calc)?;
        }
        let bp = cat.economy.ffe_reserve_bp;
        let guide =
            c::div_round(c::mul(revenue, i128::from(bp)).map_err(calc)?, 10_000).map_err(calc)?;
        let state = self.latest_state(h)?;
        let s = self.model().v2_state(&state)?;
        let mut invested = i128::from(s.building.book.invested_cents());
        for d in s.departments.values() {
            invested = c::add(invested, i128::from(d.book.invested_cents())).map_err(calc)?;
        }
        Ok(Some(RenewalGuide {
            reserve_bp: bp,
            season_revenue_budget_cents: c::narrow(revenue).map_err(calc)?,
            guide_cents: c::narrow(guide).map_err(calc)?,
            invested_cents: c::narrow(invested).map_err(calc)?,
        }))
    }

    /// Obnove v2 hotela u sezoni: po mjesecu i dijelu plaćeni iznos (iz računa obračunatih krugova), redom mjeseca,
    /// pa odjela kao u računu dobiti, zgrada zadnja. v1 igra ih nema.
    pub(crate) fn season_renewals(&self, h: HotelId) -> Result<Vec<SeasonRenewal>, GameError> {
        let Some(cat) = self.v2_catalog_or_none()? else {
            return Ok(Vec::new());
        };
        let profile = self.hotel(h)?.profile;
        let mut paid: BTreeMap<(Month, u8, String), i128> = BTreeMap::new();
        let mut add = |key: (Month, u8, String), cents: i64| -> Result<(), GameError> {
            if cents != 0 {
                let e = paid.entry(key).or_insert(0);
                *e = c::add(*e, i128::from(cents)).map_err(calc)?;
            }
            Ok(())
        };
        for (r, a) in self.resolved_accounts(h)? {
            let month = self.round_info(r)?.period.start_month();
            for (code, o) in &a.departments {
                add(
                    (month, o.class.rank(), code.as_str().to_owned()),
                    o.renewal_cents,
                )?;
            }
            add(
                (month, u8::MAX, BUILDING_CODE.to_owned()),
                a.building.renewal_cents,
            )?;
        }
        paid.into_iter()
            .map(|((month, _, code), cents)| {
                let asset_name = if code == BUILDING_CODE {
                    cat.economy.building.name.clone()
                } else {
                    cat.dept_name(profile, &sim_hotel_v2::department::DeptCode(code.clone()))
                        .map_err(|e| corrupt(format!("katalog v2: {e:?}")))?
                };
                Ok(SeasonRenewal {
                    asset_code: code,
                    asset_name,
                    month,
                    amount_cents: c::narrow(cents).map_err(calc)?,
                })
            })
            .collect()
    }
}
