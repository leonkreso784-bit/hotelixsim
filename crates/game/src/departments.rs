//! Pogledi odjela i račun dobiti hotela za v2 igru (MODEL_V2 §3, USALI). Ništa se ne sprema osim računa odjela koje
//! je obračun već zapisao u dnevnik (`DetailDto::accounts`); budžet (🟨 P8) i sve zbrojeve računamo pri čitanju.
//! v1 igra nema odjela: svaki pogled vraća `WrongModel` (grananje je u `model.rs`).

use std::collections::BTreeMap;

use sim_hotel_v2::department::{DeptOutcome, PeriodAccounts, UndistributedOutcome, UsaliClass};
use sim_hotel_v2::usali::{
    UsaliStatement, VarianceLine, budget_variance, statement, sum_statements,
};

use crate::error::{GameError, corrupt};
use crate::game::Game;
use crate::ids::HotelId;
use crate::round::RoundKey;

/// Jedan odjel hotela: stanje na kraju zadnjeg obračunatog kruga i njegov zadnji rezultat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentView {
    pub code: String,
    pub name: String,
    pub class: UsaliClass,
    pub open: bool,
    pub can_close: bool,
    pub permanent: u16,
    pub seasonal: u16,
    pub condition_pts: i32,
    /// Zadovoljstvo gostiju OVIM odjelom (`DeptState::satisfaction_pts`), 0–10 000. Nije hotelsko zadovoljstvo
    /// (indeks): hotelsko uključuje i obnovu i situacije, odjelno ne. Za Smještaj je to također odjelna brojka.
    /// `None` za odjele bez gostiju (neraspoređeni, najam, parking).
    pub guest_satisfaction_pts: Option<i32>,
    pub last: Option<(RoundKey, DeptOutcome)>,
    pub undistributed_last: Option<(RoundKey, UndistributedOutcome)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentsView {
    /// Zadnji obračunati krug; `None` prije prvog obračuna.
    pub round: Option<RoundKey>,
    pub departments: Vec<DepartmentView>,
}

/// Jedan odjel s poviješću rezultata po obračunatim krugovima (neraspoređeni odjeli nemaju rezultat odjela, pa im
/// je povijest prazna; njihov zadnji trošak je u `DepartmentView::undistributed_last`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentDetail {
    pub department: DepartmentView,
    pub history: Vec<(RoundKey, DeptOutcome)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlScope {
    /// Samo traženi krug.
    Round,
    /// Zbroj krugova od prvog do traženog.
    Season,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlView {
    pub round: RoundKey,
    pub scope: PlScope,
    pub actual: UsaliStatement,
    pub budget: UsaliStatement,
    pub lines: Vec<VarianceLine>,
    /// Ime za svaki kod retka (odjeli iz profila, naknade iz kataloga).
    pub names: BTreeMap<String, String>,
}

fn calc(e: engine::calc::CalcError) -> GameError {
    corrupt(format!("račun dobiti: {e:?}"))
}

fn catalog(e: impl std::fmt::Debug) -> GameError {
    corrupt(format!("katalog v2: {e:?}"))
}

impl Game {
    /// Računi odjela obračunatih krugova hotela, redom sezone.
    fn resolved_accounts(&self, h: HotelId) -> Result<Vec<(RoundKey, PeriodAccounts)>, GameError> {
        let mut out = Vec::new();
        for r in &self.rounds {
            let Some(rr) = self.resolved.get(r) else {
                continue;
            };
            let res = rr
                .hotels
                .iter()
                .find(|x| x.hotel_id == h)
                .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
            out.push((rr.month, self.model().accounts(res)?));
        }
        Ok(out)
    }

    /// Odjeli hotela sa stanjem na kraju zadnjeg obračunatog kruga (prije prvog obračuna: početno stanje bez rezultata).
    pub fn departments(&self, h: HotelId) -> Result<DepartmentsView, GameError> {
        let rec = self.hotel(h)?;
        let cat = self.model().require_v2(&self.content)?;
        let state = self.latest_state(h)?;
        let state = self.model().v2_state(&state)?;
        let accounts = self.resolved_accounts(h)?;
        let last = accounts.last();
        let mut departments = Vec::with_capacity(state.departments.len());
        for (code, ds) in &state.departments {
            let spec = cat.dept(code).map_err(catalog)?;
            let guest_satisfaction_pts = match ds.class {
                UsaliClass::Rooms | UsaliClass::FoodBeverage | UsaliClass::OtherOperated => {
                    Some(ds.satisfaction_pts)
                }
                UsaliClass::Rentals | UsaliClass::Undistributed(_) => None,
            };
            departments.push(DepartmentView {
                code: code.as_str().to_owned(),
                name: cat.dept_name(rec.profile, code).map_err(catalog)?,
                class: ds.class,
                open: ds.open,
                can_close: spec.can_close,
                permanent: ds.permanent,
                seasonal: ds.seasonal,
                condition_pts: ds.condition_pts,
                guest_satisfaction_pts,
                last: last.and_then(|(r, a)| a.departments.get(code).map(|o| (*r, o.clone()))),
                undistributed_last: last
                    .and_then(|(r, a)| a.undistributed.get(code).map(|o| (*r, o.clone()))),
            });
        }
        departments.sort_by(|a, b| (a.class.rank(), &a.code).cmp(&(b.class.rank(), &b.code)));
        Ok(DepartmentsView {
            round: last.map(|(r, _)| *r),
            departments,
        })
    }

    /// Jedan odjel s poviješću; nepoznat kod je `UnknownDepartment`.
    pub fn department(&self, h: HotelId, code: &str) -> Result<DepartmentDetail, GameError> {
        let department = self
            .departments(h)?
            .departments
            .into_iter()
            .find(|d| d.code == code)
            .ok_or(GameError::UnknownDepartment)?;
        let history = self
            .resolved_accounts(h)?
            .into_iter()
            .filter_map(|(r, a)| {
                a.departments
                    .iter()
                    .find(|(c, _)| c.as_str() == code)
                    .map(|(_, o)| (r, o.clone()))
            })
            .collect();
        Ok(DepartmentDetail {
            department,
            history,
        })
    }

    /// Račun dobiti kruga (ili sezone do tog kruga) uz budžet. `round = None` je zadnji obračunati krug; ako nijedan
    /// nije obračunat, `NothingResolved`; zadani krug koji još nije obračunat je `RoundNotOpen`.
    pub fn profit_and_loss(
        &self,
        h: HotelId,
        round: Option<RoundKey>,
        scope: PlScope,
    ) -> Result<PlView, GameError> {
        let rec = self.hotel(h)?;
        let cat = self.model().require_v2(&self.content)?;
        let accounts = self.resolved_accounts(h)?;
        let round = match round {
            Some(r) => {
                self.round_info(r)?;
                r
            }
            None => accounts
                .last()
                .map(|(r, _)| *r)
                .ok_or(GameError::NothingResolved)?,
        };
        let pos = accounts
            .iter()
            .position(|(r, _)| *r == round)
            .ok_or(if accounts.is_empty() {
                GameError::NothingResolved
            } else {
                GameError::RoundNotOpen
            })?;
        let from = match scope {
            PlScope::Round => pos,
            PlScope::Season => 0,
        };
        let budget_all = self.model().budget_accounts(
            &self.content,
            rec.profile,
            &self.rounds_info(),
            self.refs(rec.start_state.category())?,
        )?;
        let statements = |xs: &[PeriodAccounts]| -> Result<UsaliStatement, GameError> {
            let each = xs
                .iter()
                .map(|a| statement(a).map_err(calc))
                .collect::<Result<Vec<_>, _>>()?;
            match each.as_slice() {
                [one] => Ok(one.clone()),
                many => sum_statements(many).map_err(calc),
            }
        };
        let actual_accounts: Vec<PeriodAccounts> = accounts[from..=pos]
            .iter()
            .map(|(_, a)| a.clone())
            .collect();
        let budget_slice = budget_all
            .get(from..=pos)
            .ok_or_else(|| corrupt("budžet je kraći od obračunatih krugova"))?;
        let actual = statements(&actual_accounts)?;
        let budget = statements(budget_slice)?;
        let lines = budget_variance(&actual, &budget);

        let mut names = BTreeMap::new();
        let codes = actual
            .departments
            .iter()
            .map(|d| &d.code)
            .chain(actual.undistributed.iter().map(|u| &u.code))
            .chain(budget.departments.iter().map(|d| &d.code))
            .chain(budget.undistributed.iter().map(|u| &u.code));
        for code in codes {
            let name = cat.dept_name(rec.profile, code).map_err(catalog)?;
            names.insert(code.as_str().to_owned(), name);
        }
        for (key, _) in actual.fixed_charges.iter().chain(&budget.fixed_charges) {
            let name = cat
                .economy
                .fixed_charges
                .get(key)
                .map_or_else(|| key.clone(), |f| f.name.clone());
            names.insert(key.clone(), name);
        }
        Ok(PlView {
            round,
            scope,
            actual,
            budget,
            lines,
            names,
        })
    }
}
