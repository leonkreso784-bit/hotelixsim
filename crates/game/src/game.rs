use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use content::Content;
use sim_hotel::{
    Category, HotelState, IndexSet, Month, MonthPlan, Profile, RefTable, SituationTemplate,
    start_indices,
};

use crate::actor::{Actor, strategy_from_code};
use crate::error::{GameError, corrupt};
use crate::event::{Controller, GameCreated, GameEvent, HotelSetup, Recorded, RoundResolved};
use crate::ids::{GameId, HotelId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundStatus {
    /// ZAKAZANA (ARHITEKTURA §8.1): prethodni mjesec još nije obračunat.
    Scheduled,
    Open,
    /// ZATVARANJE: gumb pritisnut, posao obračuna čeka u redu.
    Closing,
    Resolved,
}

/// Jedan hotel u igri: postavke iz `GameCreated` plus ono što su igrač ili robot odlučili.
#[derive(Clone, Debug)]
pub struct HotelRecord {
    pub setup: HotelSetup,
    pub profile: Profile,
    pub start_state: HotelState,
    pub start_indices: IndexSet,
    pub plans: BTreeMap<Month, MonthPlan>,
    pub answers: BTreeMap<(Month, String), u8>,
}

/// Projekcija dnevnika: stanje igre dobiveno slijedom događaja (`fold`).
#[derive(Clone, Debug)]
pub struct Game {
    pub id: GameId,
    pub seed: u64,
    pub months: Vec<Month>,
    /// Verzija jezgre kojom je igra stvorena; obračun se odbija na drugoj (BACKLOG M3).
    pub core_version: String,
    pub content: Content,
    pub hotels: Vec<HotelRecord>,
    pub closed: BTreeMap<Month, Actor>,
    pub resolved: BTreeMap<Month, RoundResolved>,
    pub last_seq: i64,
    pub history: Vec<Recorded>,
    refs: BTreeMap<Category, RefTable>,
}

impl Game {
    pub fn fold(records: &[Recorded]) -> Result<Game, GameError> {
        let (first, rest) = records
            .split_first()
            .ok_or_else(|| corrupt("prazan dnevnik"))?;
        let GameEvent::GameCreated(c) = &first.event else {
            return Err(corrupt("prvi događaj nije GameCreated"));
        };
        if first.seq != 1 {
            return Err(corrupt("GameCreated nije na rednom broju 1"));
        }
        let mut g = Game::created(c, first)?;
        for r in rest {
            g.apply(r.clone())?;
        }
        Ok(g)
    }

    fn created(c: &GameCreated, r: &Recorded) -> Result<Game, GameError> {
        let snap = content::Snapshot::from_value(&c.snapshot)
            .map_err(|e| corrupt(format!("snimak: {e:?}")))?;
        let ok = content::fingerprint_value(&c.snapshot) == c.ruleset_id
            || snap.legacy_fingerprint() == c.ruleset_id;
        if !ok {
            return Err(corrupt("otisak pravila se ne slaže sa snimkom"));
        }
        let content = snap
            .build()
            .map_err(|e| corrupt(format!("snimak: {e:?}")))?;
        let mut setups: Vec<HotelSetup> = c.hotels.clone();
        setups.sort_by_key(|h| h.slot);
        let mut refs = BTreeMap::new();
        let mut hotels = Vec::with_capacity(setups.len());
        for setup in setups {
            let profile = Profile::from_code(&setup.profile)
                .ok_or_else(|| corrupt(format!("profil {}", setup.profile)))?;
            if let Controller::Robot { strategy } = &setup.controller
                && strategy_from_code(strategy).is_none()
            {
                return Err(corrupt(format!("strategija {strategy}")));
            }
            let spec = content
                .catalog
                .profile(profile)
                .map_err(|e| corrupt(format!("{e:?}")))?;
            let state = HotelState::from_profile(spec).map_err(|e| corrupt(format!("{e:?}")))?;
            if let Entry::Vacant(e) = refs.entry(state.category) {
                let t = RefTable::compute(&content.catalog, state.category)
                    .map_err(|e| corrupt(format!("{e:?}")))?;
                e.insert(t);
            }
            let table = refs
                .get(&state.category)
                .ok_or_else(|| corrupt("referentna tablica"))?;
            let start = start_indices(&state, &content.catalog, table)
                .map_err(|e| corrupt(format!("{e:?}")))?;
            hotels.push(HotelRecord {
                setup,
                profile,
                start_state: state,
                start_indices: start,
                plans: BTreeMap::new(),
                answers: BTreeMap::new(),
            });
        }
        Ok(Game {
            id: c.game_id,
            seed: c.seed,
            months: c.months.iter().map(|&m| m.into()).collect(),
            core_version: c.core_version.clone(),
            content,
            hotels,
            closed: BTreeMap::new(),
            resolved: BTreeMap::new(),
            last_seq: 1,
            history: vec![r.clone()],
            refs,
        })
    }

    /// Primijeni jedan zapisani događaj. Pravila su provjerena pri upisu; prekršaj ovdje znači pokvaren dnevnik.
    pub fn apply(&mut self, r: Recorded) -> Result<(), GameError> {
        if r.seq != self.last_seq + 1 {
            return Err(corrupt(format!(
                "redni broj {} nakon {}",
                r.seq, self.last_seq
            )));
        }
        match &r.event {
            GameEvent::GameCreated(_) => return Err(corrupt("drugi GameCreated")),
            GameEvent::PlanSet(p) => {
                let m = p.month.into();
                self.expect_open(m)?;
                let plan = MonthPlan::from(&p.plan);
                self.hotel_mut(p.hotel_id)?.plans.insert(m, plan);
            }
            GameEvent::AnswerChosen(a) => {
                let m = a.month.into();
                self.expect_open(m)?;
                let h = self.hotel_mut(a.hotel_id)?;
                if h.answers
                    .insert((m, a.situation.clone()), a.answer)
                    .is_some()
                {
                    return Err(corrupt(format!("dvostruki odgovor na {}", a.situation)));
                }
            }
            GameEvent::RoundClosed(c) => {
                let m = c.month.into();
                self.expect_open(m)?;
                self.closed.insert(m, r.actor.clone());
            }
            GameEvent::RoundResolved(rr) => {
                let m = rr.month.into();
                if self.status(m)? != RoundStatus::Closing {
                    return Err(corrupt("obračun runde koja nije zatvorena"));
                }
                self.resolved.insert(m, rr.clone());
            }
        }
        self.last_seq = r.seq;
        self.history.push(r);
        Ok(())
    }

    fn expect_open(&self, m: Month) -> Result<(), GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(()),
            s => Err(corrupt(format!("događaj za rundu u stanju {s:?}"))),
        }
    }

    pub fn hotel(&self, id: HotelId) -> Result<&HotelRecord, GameError> {
        self.hotels
            .iter()
            .find(|h| h.setup.hotel_id == id)
            .ok_or(GameError::UnknownHotel)
    }

    fn hotel_mut(&mut self, id: HotelId) -> Result<&mut HotelRecord, GameError> {
        self.hotels
            .iter_mut()
            .find(|h| h.setup.hotel_id == id)
            .ok_or(GameError::UnknownHotel)
    }

    pub fn current_month(&self) -> Option<Month> {
        self.months
            .iter()
            .copied()
            .find(|m| !self.resolved.contains_key(m))
    }

    pub fn status(&self, m: Month) -> Result<RoundStatus, GameError> {
        if !self.months.contains(&m) {
            return Err(GameError::UnknownMonth);
        }
        Ok(if self.resolved.contains_key(&m) {
            RoundStatus::Resolved
        } else if self.closed.contains_key(&m) {
            RoundStatus::Closing
        } else if self.current_month() == Some(m) {
            RoundStatus::Open
        } else {
            RoundStatus::Scheduled
        })
    }

    /// Stanje hotela na POČETKU mjeseca `m`: početno stanje ili `after` iz obračuna prethodnog mjeseca.
    pub fn state_at(&self, hotel: HotelId, m: Month) -> Result<HotelState, GameError> {
        let h = self.hotel(hotel)?;
        let pos = self
            .months
            .iter()
            .position(|&x| x == m)
            .ok_or(GameError::UnknownMonth)?;
        let Some(prev) = pos.checked_sub(1).and_then(|p| self.months.get(p)) else {
            return Ok(h.start_state.clone());
        };
        let rr = self.resolved.get(prev).ok_or(GameError::RoundNotOpen)?;
        let res = rr
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
        HotelState::try_from(&res.report.after)
    }

    /// Stanje nakon zadnjeg obračunatog mjeseca (ili početno).
    pub fn latest_state(&self, hotel: HotelId) -> Result<HotelState, GameError> {
        let h = self.hotel(hotel)?;
        let Some(last) = self
            .months
            .iter()
            .rev()
            .find(|m| self.resolved.contains_key(m))
        else {
            return Ok(h.start_state.clone());
        };
        let rr = self
            .resolved
            .get(last)
            .ok_or_else(|| corrupt("obračun mjeseca"))?;
        let res = rr
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
        HotelState::try_from(&res.report.after)
    }

    /// Zadnji postavljeni plan ili zadani (DEMO §6.1).
    pub fn effective_plan(&self, hotel: HotelId, m: Month) -> Result<MonthPlan, GameError> {
        let h = self.hotel(hotel)?;
        if let Some(p) = h.plans.get(&m) {
            return Ok(p.clone());
        }
        let state = self.state_at(hotel, m)?;
        MonthPlan::default_for(&state, m, &self.content.catalog)
            .map_err(|e| corrupt(format!("{e:?}")))
    }

    pub fn situations_in(&self, m: Month) -> Vec<&SituationTemplate> {
        self.content
            .situations
            .iter()
            .filter(|t| t.month == m)
            .collect()
    }

    pub fn unanswered_count(&self, hotel: HotelId, m: Month) -> Result<u8, GameError> {
        let h = self.hotel(hotel)?;
        let n = self
            .situations_in(m)
            .into_iter()
            .filter(|t| !h.answers.contains_key(&(m, t.code.clone())))
            .count();
        u8::try_from(n).map_err(|_| corrupt("broj neodgovorenih situacija"))
    }

    pub fn refs(&self, c: Category) -> Result<&RefTable, GameError> {
        self.refs
            .get(&c)
            .ok_or_else(|| corrupt("referentna tablica"))
    }
}
