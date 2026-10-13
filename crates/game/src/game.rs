use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use content::Content;
use sim_hotel::{Category, IndexSet, Profile, SituationTemplate};
use sim_hotel_v2::situations::DrawnSituation;

use crate::actor::{Actor, strategy_from_code};
use crate::drawn::DrawnInstance;
use crate::dto::PlanDto;
use crate::error::{GameError, corrupt};
use crate::event::{
    Controller, DebriefRecorded, GameCreated, GameEvent, HotelResolution, HotelSetup, Recorded,
    RoundResolved, SeasonDebriefRecorded, VerdictRecorded,
};
use crate::ids::{GameId, HotelId};
use crate::model::{CorePlan, CoreRefs, CoreState, Model, RoundInfo};
use crate::round::RoundKey;
use crate::talk::MAX_CHAT_MESSAGES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundStatus {
    /// ZAKAZANA (ARHITEKTURA §8.1): prethodni mjesec još nije obračunat.
    Scheduled,
    Open,
    /// ZATVARANJE: gumb pritisnut, posao obračuna čeka u redu.
    Closing,
    Resolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Speaker {
    Student,
    Character,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ChatLine {
    pub from: Speaker,
    pub text: String,
}

/// Slobodan odgovor na situaciju, njegova presuda i presude koje su stigle prekasno.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeResponse {
    pub text: String,
    pub verdict: Option<VerdictRecorded>,
    pub late: Vec<VerdictRecorded>,
}

pub(crate) fn student_count(lines: &[ChatLine]) -> usize {
    lines.iter().filter(|l| l.from == Speaker::Student).count()
}

/// Jedan hotel u igri: postavke iz `GameCreated` plus ono što su igrač ili robot odlučili.
#[derive(Clone, Debug)]
pub struct HotelRecord {
    pub setup: HotelSetup,
    pub profile: Profile,
    pub start_state: CoreState,
    pub start_indices: IndexSet,
    pub plans: BTreeMap<RoundKey, CorePlan>,
    pub answers: BTreeMap<(RoundKey, String), u8>,
    pub chats: BTreeMap<(RoundKey, String), Vec<ChatLine>>,
    pub responses: BTreeMap<(RoundKey, String), FreeResponse>,
    pub debriefs: BTreeMap<RoundKey, DebriefRecorded>,
    /// Sezonski mentor (samo v2, nakon zadnjeg kruga).
    pub season_debrief: Option<SeasonDebriefRecorded>,
    /// Primjerci situacija izvučeni za ovaj hotel (i kolektivni) po krugu, iz `SituationsDrawn` (G-1).
    pub drawn: BTreeMap<RoundKey, Vec<DrawnInstance>>,
    /// Povijest generatora ovog hotela: njegovi i kolektivni primjerci, redom dnevnika.
    pub drawn_log: Vec<DrawnSituation>,
}

/// Projekcija dnevnika: stanje igre dobiveno slijedom događaja (`fold`).
#[derive(Clone, Debug)]
pub struct Game {
    pub id: GameId,
    pub seed: u64,
    pub rounds: Vec<RoundKey>,
    /// Verzija jezgre kojom je igra stvorena; obračun se odbija na drugoj (BACKLOG M3).
    pub core_version: String,
    pub content: Content,
    pub hotels: Vec<HotelRecord>,
    pub closed: BTreeMap<RoundKey, Actor>,
    pub resolved: BTreeMap<RoundKey, RoundResolved>,
    pub last_seq: i64,
    pub history: Vec<Recorded>,
    model: Model,
    /// Krugovi za koje dnevnik ima `SituationsDrawn`; ostali v2 krugovi igraju po starom pravilu (S1–S4 po mjesecu).
    pub(crate) drawn_rounds: BTreeSet<RoundKey>,
    /// Krugovi igre s kalendarom, redom `rounds`.
    rounds_info: Vec<RoundInfo>,
    refs: BTreeMap<Category, CoreRefs>,
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
        let model = Model::of(&content, snap.model());
        model.check_version(&c.core_version)?;
        let rounds_info = model.rounds(&c.months, &c.rounds)?;
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
            let state = model.start_state(&content, profile)?;
            let category = state.category();
            if let Entry::Vacant(e) = refs.entry(category) {
                e.insert(model.refs(&content, category, &rounds_info)?);
            }
            let table = refs
                .get(&category)
                .ok_or_else(|| corrupt("referentna tablica"))?;
            let start = model.start_indices(&state, &content, table)?;
            hotels.push(HotelRecord {
                setup,
                profile,
                start_state: state,
                start_indices: start,
                plans: BTreeMap::new(),
                answers: BTreeMap::new(),
                chats: BTreeMap::new(),
                responses: BTreeMap::new(),
                debriefs: BTreeMap::new(),
                season_debrief: None,
                drawn: BTreeMap::new(),
                drawn_log: Vec::new(),
            });
        }
        Ok(Game {
            id: c.game_id,
            seed: c.seed,
            rounds: c.months.clone(),
            core_version: c.core_version.clone(),
            content,
            hotels,
            closed: BTreeMap::new(),
            resolved: BTreeMap::new(),
            last_seq: 1,
            history: vec![r.clone()],
            model,
            drawn_rounds: BTreeSet::new(),
            rounds_info,
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
                let m = p.month;
                self.expect_open(m)?;
                let plan = self.model.plan_from_dto(&p.plan)?;
                self.hotel_mut(p.hotel_id)?.plans.insert(m, plan);
            }
            GameEvent::AnswerChosen(a) => {
                let m = a.month;
                self.expect_open(m)?;
                let h = self.hotel_mut(a.hotel_id)?;
                if h.responses.contains_key(&(m, a.situation.clone())) {
                    return Err(corrupt(format!(
                        "ponuđeni odgovor uz slobodni na {}",
                        a.situation
                    )));
                }
                if h.answers
                    .insert((m, a.situation.clone()), a.answer)
                    .is_some()
                {
                    return Err(corrupt(format!("dvostruki odgovor na {}", a.situation)));
                }
            }
            GameEvent::ChatMessageSent(c) => {
                let m = c.month;
                self.expect_open(m)?;
                self.expect_free_text(c.hotel_id, m, &c.situation)?;
                let h = self.hotel_mut(c.hotel_id)?;
                let key = (m, c.situation.clone());
                if h.answers.contains_key(&key) || h.responses.contains_key(&key) {
                    return Err(corrupt("poruka nakon odgovora"));
                }
                let lines = h.chats.entry(key).or_default();
                if student_count(lines) >= usize::from(MAX_CHAT_MESSAGES)
                    || lines.last().is_some_and(|l| l.from == Speaker::Student)
                {
                    return Err(corrupt("poruka izvan reda razgovora"));
                }
                lines.push(ChatLine {
                    from: Speaker::Student,
                    text: c.text.clone(),
                });
            }
            GameEvent::CharacterReplied(c) => {
                let m = c.month;
                self.expect_open(m)?;
                let h = self.hotel_mut(c.hotel_id)?;
                let lines = h.chats.entry((m, c.situation.clone())).or_default();
                if usize::from(c.reply_to) != student_count(lines)
                    || lines.last().is_none_or(|l| l.from != Speaker::Student)
                {
                    return Err(corrupt("odgovor lika izvan reda razgovora"));
                }
                lines.push(ChatLine {
                    from: Speaker::Character,
                    text: c.text.clone(),
                });
            }
            GameEvent::ResponseSubmitted(s) => {
                let m = s.month;
                self.expect_open(m)?;
                self.expect_free_text(s.hotel_id, m, &s.situation)?;
                let h = self.hotel_mut(s.hotel_id)?;
                let key = (m, s.situation.clone());
                if h.answers.contains_key(&key) || h.responses.contains_key(&key) {
                    return Err(corrupt(format!("dvostruki odgovor na {}", s.situation)));
                }
                h.responses.insert(
                    key,
                    FreeResponse {
                        text: s.text.clone(),
                        verdict: None,
                        late: Vec::new(),
                    },
                );
            }
            GameEvent::VerdictRecorded(v) => {
                let m = v.month;
                if !matches!(self.status(m)?, RoundStatus::Open | RoundStatus::Closing) {
                    return Err(corrupt("presuda za rundu koja nije otvorena ni zatvorena"));
                }
                let h = self.hotel_mut(v.hotel_id)?;
                let fr = h
                    .responses
                    .get_mut(&(m, v.situation.clone()))
                    .ok_or_else(|| corrupt("presuda bez slobodnog odgovora"))?;
                if fr.verdict.is_some() {
                    return Err(corrupt("dvostruka presuda"));
                }
                fr.verdict = Some(v.clone());
            }
            GameEvent::VerdictLate(v) => {
                let m = v.month;
                let resolved = self.status(m)? == RoundStatus::Resolved;
                let h = self.hotel_mut(v.hotel_id)?;
                let fr = h
                    .responses
                    .get_mut(&(m, v.situation.clone()))
                    .ok_or_else(|| corrupt("kasna presuda bez slobodnog odgovora"))?;
                if fr.verdict.is_none() && !resolved {
                    return Err(corrupt("kasna presuda bez ranije presude"));
                }
                fr.late.push(v.clone());
            }
            GameEvent::DebriefRecorded(d) => {
                let m = d.month;
                if self.status(m)? != RoundStatus::Resolved {
                    return Err(corrupt("mentor za rundu koja nije obračunata"));
                }
                let h = self.hotel_mut(d.hotel_id)?;
                if h.debriefs.insert(m, d.clone()).is_some() {
                    return Err(corrupt("dvostruki mentor"));
                }
            }
            GameEvent::SeasonDebriefRecorded(d) => {
                if self.model.require_v2(&self.content).is_err() {
                    return Err(corrupt("sezonski mentor u v1 igri"));
                }
                if self.current_round().is_some() {
                    return Err(corrupt("sezonski mentor prije kraja sezone"));
                }
                let h = self.hotel_mut(d.hotel_id)?;
                if h.season_debrief.replace(d.clone()).is_some() {
                    return Err(corrupt("dvostruki sezonski mentor"));
                }
            }
            GameEvent::SituationsDrawn(d) => self.apply_drawn(d)?,
            GameEvent::RoundClosed(c) => {
                let m = c.month;
                self.expect_open(m)?;
                self.closed.insert(m, r.actor.clone());
            }
            GameEvent::RoundResolved(rr) => {
                let m = rr.month;
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

    pub(crate) fn expect_open(&self, m: RoundKey) -> Result<(), GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(()),
            s => Err(corrupt(format!("događaj za rundu u stanju {s:?}"))),
        }
    }

    fn expect_free_text(&self, hotel: HotelId, m: RoundKey, code: &str) -> Result<(), GameError> {
        let ok = self
            .situation_round_for(hotel, code)
            .is_ok_and(|(r, _)| r == m)
            && self.ai_entry(code).is_some_and(|a| a.free_text);
        if ok {
            Ok(())
        } else {
            Err(corrupt(format!("situacija {code} nema slobodni odgovor")))
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

    pub fn current_round(&self) -> Option<RoundKey> {
        self.rounds
            .iter()
            .copied()
            .find(|m| !self.resolved.contains_key(m))
    }

    pub fn status(&self, m: impl Into<RoundKey>) -> Result<RoundStatus, GameError> {
        let m = m.into();
        if !self.rounds.contains(&m) {
            return Err(GameError::UnknownMonth);
        }
        Ok(if self.resolved.contains_key(&m) {
            RoundStatus::Resolved
        } else if self.closed.contains_key(&m) {
            RoundStatus::Closing
        } else if self.current_round() == Some(m) {
            RoundStatus::Open
        } else {
            RoundStatus::Scheduled
        })
    }

    pub fn model(&self) -> Model {
        self.model
    }

    /// Krug igre s kalendarom (`UnknownMonth` ako krug nije u ovoj igri).
    pub fn round_info(&self, r: impl Into<RoundKey>) -> Result<RoundInfo, GameError> {
        let r = r.into();
        self.rounds_info
            .iter()
            .find(|i| i.key == r)
            .copied()
            .ok_or(GameError::UnknownMonth)
    }

    pub fn rounds_info(&self) -> Vec<RoundInfo> {
        self.rounds_info.clone()
    }

    /// Pozicija kruga u igri (od 0).
    pub(crate) fn position(&self, r: RoundKey) -> Result<usize, GameError> {
        self.rounds
            .iter()
            .position(|&x| x == r)
            .ok_or(GameError::UnknownMonth)
    }

    /// Obračun hotela u obračunatom krugu.
    fn resolution(&self, hotel: HotelId, r: &RoundKey) -> Result<&HotelResolution, GameError> {
        self.resolved
            .get(r)
            .ok_or(GameError::RoundNotOpen)?
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))
    }

    /// Stanje hotela na POČETKU kruga `m`: početno stanje ili stanje nakon obračuna prethodnog kruga.
    pub fn state_at(&self, hotel: HotelId, m: impl Into<RoundKey>) -> Result<CoreState, GameError> {
        let h = self.hotel(hotel)?;
        let pos = self.position(m.into())?;
        let Some(prev) = pos.checked_sub(1).and_then(|p| self.rounds.get(p)) else {
            return Ok(h.start_state.clone());
        };
        self.model.state_after(self.resolution(hotel, prev)?)
    }

    /// Stanje nakon zadnjeg obračunatog kruga (ili početno).
    pub fn latest_state(&self, hotel: HotelId) -> Result<CoreState, GameError> {
        let h = self.hotel(hotel)?;
        let Some(last) = self
            .rounds
            .iter()
            .rev()
            .find(|m| self.resolved.contains_key(m))
        else {
            return Ok(h.start_state.clone());
        };
        self.model.state_after(self.resolution(hotel, last)?)
    }

    /// Plan kruga: zadnji postavljeni; inače plan najbližeg ranijeg kruga s planom, prenesen u ovaj krug
    /// (`Model::carry_plan`; v1 ne prenosi, nego daje zadani, DEMO §6.1); inače zadani.
    pub fn effective_plan(
        &self,
        hotel: HotelId,
        m: impl Into<RoundKey>,
    ) -> Result<CorePlan, GameError> {
        let m = m.into();
        let h = self.hotel(hotel)?;
        if let Some(p) = h.plans.get(&m) {
            return Ok(p.clone());
        }
        let state = self.state_at(hotel, m)?;
        let info = self.round_info(m)?;
        let pos = self.position(m)?;
        let earlier = self
            .rounds
            .iter()
            .take(pos)
            .rev()
            .find_map(|r| h.plans.get(r));
        match earlier {
            Some(prev) => self.model.carry_plan(prev, &state, &info, &self.content),
            None => self.model.default_plan(&state, &info, &self.content),
        }
    }

    /// Plan iz tijela zahtjeva (app `set_plan`) za model ove igre; javni omotač oko `Model::plan_from_dto`.
    /// v2 plan bez `departments` (do Taska 15 `PlanBody` ih nema) uzima postavke odjela iz `effective_plan(h, r)`,
    /// a ukupne sezonce i održavanje iz tijela (`Model::plan_from_dto_without_departments`).
    pub fn plan_from_dto(
        &self,
        h: HotelId,
        r: impl Into<RoundKey>,
        d: &PlanDto,
    ) -> Result<CorePlan, GameError> {
        let r = r.into();
        self.hotel(h)?;
        self.round_info(r)?;
        self.model.check_plan_body(d)?;
        if self.model.needs_departments(d) {
            let eff = self.effective_plan(h, r)?;
            let state = self.state_at(h, r)?;
            let info = self.round_info(r)?;
            return self.model.plan_from_dto_without_departments(
                d,
                &eff,
                &state,
                &info,
                &self.content,
            );
        }
        self.model.plan_from_dto(d)
    }

    /// Situacije kruga za hotel igrača (prvi utor): sidra S1–S4 po starom pravilu i, kad je krug izvučen,
    /// njegovi primjerci (`Game::situations_of_hotel` za bilo koji hotel). Krug koji nije u ovoj igri nema situacija.
    pub fn situations_in(&self, r: impl Into<RoundKey>) -> Vec<&SituationTemplate> {
        match self.hotels.first() {
            Some(h) => self.hotel_situations(h, r.into()),
            None => Vec::new(),
        }
    }

    pub fn unanswered_count(
        &self,
        hotel: HotelId,
        m: impl Into<RoundKey>,
    ) -> Result<u8, GameError> {
        let m = m.into();
        let h = self.hotel(hotel)?;
        let n = self
            .hotel_situations(h, m)
            .into_iter()
            .filter(|t| {
                let key = (m, t.code.clone());
                !h.answers.contains_key(&key) && !h.responses.contains_key(&key)
            })
            .count();
        u8::try_from(n).map_err(|_| corrupt("broj neodgovorenih situacija"))
    }

    pub(crate) fn refs(&self, c: Category) -> Result<&CoreRefs, GameError> {
        self.refs
            .get(&c)
            .ok_or_else(|| corrupt("referentna tablica"))
    }
}
