//! Situacije iz predložaka u igri (G-1): događaj `SituationsDrawn` pri otvaranju kruga, primjerci po hotelu iz
//! dnevnika i `situations_in` koji ih čita. Izvlačenje je isti poziv generatora kao u sezoni robota
//! (`sim_hotel_v2::draw_for_round`); igra mu daje stanje, obračun i pokazatelje prošlog kruga i povijest iz
//! dnevnika. Igra bez ikakvog `SituationsDrawn` (v1, v2 bez predložaka, v2 nastala prije G-1) igra po starom pravilu.

use std::collections::BTreeMap;

use sim_hotel::{IndexSet, SituationTemplate};
use sim_hotel_v2::Period;
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{DrawnSituation, Kind, MajorSide, RoleFill, Scope};

use crate::actor::Actor;
use crate::error::{GameError, corrupt};
use crate::event::{DrawnItem, DrawnRole, GameEvent, Recorded, SituationsDrawn};
use crate::game::{Game, HotelRecord, RoundStatus};
use crate::ids::HotelId;
use crate::model::DrawInput;
use crate::round::RoundKey;

/// Primjerak izvučen za jedan hotel: jezgreni oblik (povijest generatora), situacija s ponuđenim odgovorima za
/// kategoriju tog hotela i oznaka „samo nagrada“ njegova predloška (S-4).
#[derive(Clone, Debug)]
pub struct DrawnInstance {
    pub drawn: DrawnSituation,
    pub situation: SituationTemplate,
    /// Podaci za AI sloj iz popunjenog primjerka (lik, rubrika, kriteriji); lik ima samo predložak s likom.
    pub ai: content::SituationAi,
    pub no_penalty: bool,
    /// Opis situacije s popunjenim mjestima (`content::Instance::description`).
    pub description: String,
}

/// Uloga primjerka za pogled: ključ iz predloška, ime i rečenica uloge — samo ono što je primjerak već popunio
/// (lik: rečenica kao u liku; ostale uloge: vrsta iz izvlačenja ili naziv iz predloška, inače `None`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceRole {
    pub key: String,
    pub name: String,
    pub role: Option<String>,
}

/// Što pogled situacije zna o izvučenom primjerku. Sidra S1–S4 i v1 to nemaju (sve `None`, uloge prazne) 🟨.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SituationCard {
    /// Kod odjela primjerka; `None` za situaciju cijelog hotela.
    pub department: Option<String>,
    pub kind: Option<Kind>,
    pub weight: Option<u8>,
    pub description: Option<String>,
    pub roles: Vec<InstanceRole>,
}

const KINDS: [(Kind, &str); 6] = [
    (Kind::Complaint, "complaint"),
    (Kind::Staff, "staff"),
    (Kind::Opportunity, "opportunity"),
    (Kind::Breakdown, "breakdown"),
    (Kind::Crisis, "crisis"),
    (Kind::Request, "request"),
];

fn kind_code(k: Kind) -> &'static str {
    KINDS
        .iter()
        .find(|(x, _)| *x == k)
        .map_or("complaint", |(_, c)| c)
}

fn kind_of(code: &str) -> Result<Kind, GameError> {
    KINDS
        .iter()
        .find(|(_, c)| *c == code)
        .map(|(k, _)| *k)
        .ok_or_else(|| corrupt(format!("vrsta situacije {code}")))
}

fn item_of(d: &DrawnSituation) -> DrawnItem {
    DrawnItem {
        code: d.code.clone(),
        template: d.template.clone(),
        day: d.day,
        slot: d.slot,
        occurrence: d.occurrence,
        kind: kind_code(d.kind).to_owned(),
        weight: d.weight,
        scope: match d.scope {
            Scope::Hotel => "hotel",
            Scope::All => "all",
        }
        .to_owned(),
        major: d.major.map(|m| {
            match m {
                MajorSide::High => "high",
                MajorSide::Low => "low",
            }
            .to_owned()
        }),
        department: d.department.as_ref().map(|c| c.as_str().to_owned()),
        roles: d
            .roles
            .iter()
            .map(|(k, r)| {
                (
                    k.clone(),
                    DrawnRole {
                        name: r.name.clone(),
                        role_type: r.role_type.clone(),
                    },
                )
            })
            .collect(),
        numbers: d.numbers.clone(),
    }
}

fn drawn_of(it: &DrawnItem, period: Period) -> Result<DrawnSituation, GameError> {
    let scope = match it.scope.as_str() {
        "hotel" => Scope::Hotel,
        "all" => Scope::All,
        other => return Err(corrupt(format!("doseg situacije {other}"))),
    };
    if (scope == Scope::All) != (it.slot == 0) {
        return Err(corrupt(format!(
            "primjerak {}: kolektivna je samo u utoru 0",
            it.code
        )));
    }
    let major = match it.major.as_deref() {
        None => None,
        Some("high") => Some(MajorSide::High),
        Some("low") => Some(MajorSide::Low),
        Some(other) => return Err(corrupt(format!("veća situacija {other}"))),
    };
    Ok(DrawnSituation {
        code: it.code.clone(),
        template: it.template.clone(),
        period,
        day: it.day,
        occurrence: it.occurrence,
        slot: it.slot,
        kind: kind_of(&it.kind)?,
        weight: it.weight,
        scope,
        major,
        department: it.department.clone().map(DeptCode),
        roles: it
            .roles
            .iter()
            .map(|(k, r)| {
                (
                    k.clone(),
                    RoleFill {
                        name: r.name.clone(),
                        role_type: r.role_type.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
        numbers: it.numbers.clone(),
    })
}

impl Game {
    /// Je li krug izvučen (dnevnik ima `SituationsDrawn` za njega).
    pub fn is_drawn(&self, r: impl Into<RoundKey>) -> bool {
        self.drawn_rounds.contains(&r.into())
    }

    /// Primjena zapisanog `SituationsDrawn`: primjerci svakog hotela (njegovi i kolektivni) postaju situacije
    /// kruga, a povijest generatora toga hotela raste. Pravilo je provjereno pri upisu; prekršaj je pokvaren dnevnik.
    pub(crate) fn apply_drawn(&mut self, d: &SituationsDrawn) -> Result<(), GameError> {
        let m = d.month;
        self.expect_open(m)?;
        if !self.model().can_draw(&self.content) {
            return Err(corrupt("izvlačenje situacija u igri bez predložaka"));
        }
        if self.drawn_rounds.contains(&m) {
            return Err(corrupt("dvostruko izvlačenje situacija kruga"));
        }
        let first = *self
            .rounds
            .first()
            .ok_or_else(|| corrupt("igra bez krugova"))?;
        if m != first && !self.drawn_rounds.contains(&first) {
            return Err(corrupt("izvlačenje u igri koja ga nije imala od početka"));
        }
        let period = self.round_info(m)?.period;
        let items = d
            .situations
            .iter()
            .map(|it| drawn_of(it, period))
            .collect::<Result<Vec<_>, _>>()?;
        let slots: Vec<u8> = self.hotels.iter().map(|h| h.setup.slot).collect();
        if let Some(bad) = items
            .iter()
            .find(|i| i.slot != 0 && !slots.contains(&i.slot))
        {
            return Err(corrupt(format!(
                "primjerak {}: nema hotela s utorom {}",
                bad.code, bad.slot
            )));
        }
        let tpl = self
            .content
            .situation_templates
            .as_ref()
            .ok_or_else(|| corrupt("predlošci situacija"))?;
        let base = self.model().catalog(&self.content)?;
        let dept_names = self.content.v2.as_ref().map(|c| &c.departments);
        let mut built: Vec<Vec<DrawnInstance>> = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            let category = h.start_state.category();
            let mut own = Vec::new();
            for i in items
                .iter()
                .filter(|i| i.slot == 0 || i.slot == h.setup.slot)
            {
                let name = |c: &DeptCode| {
                    dept_names
                        .and_then(|d| d.get(c))
                        .map_or_else(|| c.as_str().to_owned(), |d| d.name.clone())
                };
                let filled = tpl
                    .instance_with(i, base, category, &name)
                    .map_err(|e| corrupt(format!("{e:?}")))?;
                own.push(DrawnInstance {
                    drawn: i.clone(),
                    situation: filled.situation,
                    ai: filled.ai,
                    no_penalty: tpl.no_penalty(&i.template),
                    description: filled.description,
                });
            }
            built.push(own);
        }
        for (h, mut own) in self.hotels.iter_mut().zip(built) {
            // Dnevnik ostaje redom događaja; popis situacija hotela ide po važnosti silazno (stabilno), pa
            // kolektivna ne stoji ispred važnijih primjeraka samo zato što je u događaju prva.
            h.drawn_log.extend(own.iter().map(|i| i.drawn.clone()));
            own.sort_by_key(|i| std::cmp::Reverse(i.drawn.weight));
            h.drawn.insert(m, own);
        }
        self.drawn_rounds.insert(m);
        Ok(())
    }

    /// Pokazatelji na početku kruga `r` za hotel (početni ili iz obračuna prošlog kruga).
    fn indices_at(&self, hotel: HotelId, r: RoundKey) -> Result<IndexSet, GameError> {
        let pos = self.position(r)?;
        let Some(prev) = pos.checked_sub(1).and_then(|p| self.rounds.get(p)) else {
            return Ok(self.hotel(hotel)?.start_indices.clone());
        };
        let res = self
            .resolved
            .get(prev)
            .and_then(|rr| rr.hotels.iter().find(|x| x.hotel_id == hotel))
            .ok_or_else(|| corrupt("prošli krug nije obračunat"))?;
        IndexSet::from_hundredths(res.indices).ok_or_else(|| corrupt("pokazatelji u obračunu"))
    }

    /// Događaj `SituationsDrawn` za krug koji se otvara; `None` kad igra ne izvlači (v1, v2 bez predložaka, v2
    /// bez izvlačenja od prvog kruga), krug nije otvoren ili je već izvučen. Kolektivna se izvlači jednom za sve
    /// hotele i prva je u događaju; zatim primjerci hotela redom po utoru.
    pub fn draw_round(&self, r: impl Into<RoundKey>) -> Result<Option<GameEvent>, GameError> {
        let r = r.into();
        let model = self.model();
        if !model.can_draw(&self.content)
            || self.drawn_rounds.contains(&r)
            || self.status(r)? != RoundStatus::Open
        {
            return Ok(None);
        }
        let first = *self
            .rounds
            .first()
            .ok_or_else(|| corrupt("igra bez krugova"))?;
        if r != first && !self.drawn_rounds.contains(&first) {
            return Ok(None);
        }
        let tpl = self
            .content
            .situation_templates
            .as_ref()
            .ok_or_else(|| corrupt("predlošci situacija"))?;
        let mut set = tpl.to_core();
        set.counts = Some(self.situation_counts());
        let info = self.round_info(r)?;
        let pos = self.position(r)?;
        let prev = pos.checked_sub(1).and_then(|p| self.rounds.get(p)).copied();
        // Ulazi svih hotela: kolektivna se izvlači jednom za sve (granica nijednog hotela se ne probija).
        let mut views = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            let id = h.setup.hotel_id;
            let previous = match prev {
                Some(p) => Some(self.period_report(h, p)?),
                None => None,
            };
            views.push((self.state_at(id, r)?, self.indices_at(id, r)?, previous));
        }
        let inputs: Vec<DrawInput> = self
            .hotels
            .iter()
            .zip(&views)
            .map(|(h, (state, indices, previous))| DrawInput {
                state,
                previous: previous.as_ref(),
                indices,
                round: &info,
                history: &h.drawn_log,
                seed: self.seed,
                slot: h.setup.slot,
            })
            .collect();
        let shared = model.draw_collective(&self.content, &set, &inputs)?;
        let mut items: Vec<DrawnItem> = shared.iter().map(item_of).collect();
        for i in &inputs {
            let found = model.draw_situations(&self.content, &set, i, shared.as_ref())?;
            items.extend(found.iter().filter(|s| s.slot != 0).map(item_of));
        }
        Ok(Some(GameEvent::SituationsDrawn(SituationsDrawn {
            month: r,
            situations: items,
        })))
    }

    /// Otvaranje kruga: izvlačenje situacija (ako igra izvlači), pa potezi robota koji već vide primjerke.
    pub fn open_round(&self, r: impl Into<RoundKey>) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        let r = r.into();
        let mut out = Vec::new();
        match self.draw_round(r)? {
            Some(ev) => {
                let mut next = self.clone();
                next.apply(Recorded {
                    seq: next.last_seq + 1,
                    actor: Actor::System,
                    event: ev.clone(),
                })?;
                out.push((Actor::System, ev));
                out.extend(next.robot_moves(r)?);
            }
            None => out.extend(self.robot_moves(r)?),
        }
        Ok(out)
    }

    /// Situacije kruga za jedan hotel: sidra po starom pravilu (S1–S4), pa primjerci izvučeni za taj hotel.
    pub(crate) fn hotel_situations<'a>(
        &'a self,
        h: &'a HotelRecord,
        r: RoundKey,
    ) -> Vec<&'a SituationTemplate> {
        let mut v = match self.round_info(r) {
            Ok(info) => self.model().situations_in(&info, &self.content.situations),
            Err(_) => return Vec::new(),
        };
        if let Some(list) = h.drawn.get(&r) {
            // Odgovori se vode po (krug, šifra): šifra koju već nosi sidro nije druga situacija.
            for i in list {
                if !v.iter().any(|t| t.code == i.situation.code) {
                    v.push(&i.situation);
                }
            }
        }
        v
    }

    /// Situacije kruga za hotel `hotel`.
    pub fn situations_of_hotel(
        &self,
        hotel: HotelId,
        r: impl Into<RoundKey>,
    ) -> Result<Vec<&SituationTemplate>, GameError> {
        let h = self.hotel(hotel)?;
        Ok(self.hotel_situations(h, r.into()))
    }

    /// Krug i situacija hotela `hotel` s tom šifrom (šifra primjerka je jedinstvena samo unutar hotela: dva
    /// hotela mogu u istom krugu dobiti isti predložak).
    pub fn situation_round_for(
        &self,
        hotel: HotelId,
        code: &str,
    ) -> Result<(RoundKey, &SituationTemplate), GameError> {
        let h = self.hotel(hotel)?;
        self.rounds
            .iter()
            .find_map(|&r| {
                self.hotel_situations(h, r)
                    .into_iter()
                    .find(|t| t.code == code)
                    .map(|t| (r, t))
            })
            .ok_or(GameError::UnknownSituation)
    }

    /// Oznaka „samo nagrada“: primjerak iz dnevnika nosi je od svog predloška; šifra koja nije izvučeni primjerak
    /// (sidra, sadržaj sa šifrom oblika `P-X.krug`) ide na provjeru po šifri (`SituationTemplates::no_penalty`).
    pub(crate) fn no_penalty(&self, h: &HotelRecord, r: RoundKey, code: &str) -> bool {
        match h
            .drawn
            .get(&r)
            .and_then(|l| l.iter().find(|i| i.situation.code == code))
        {
            Some(i) => i.no_penalty,
            None => self
                .content
                .situation_templates
                .as_ref()
                .is_some_and(|t| t.no_penalty(code)),
        }
    }

    /// AI podaci situacije s tom šifrom: za šifru iz sadržaja (S1–S4) njezini; za izvučeni primjerak podaci
    /// njegova popunjenog predloška (lik, rubrika, kriteriji; slobodan odgovor ima samo predložak s likom). Šifra
    /// primjerka može postojati kod više hotela s različitim imenima likova, pa je za razgovor i presudu
    /// mjerodavan `ai_entry_for`; ovdje se uzima prvi primjerak (dovoljno za „ima li slobodan odgovor").
    pub fn ai_entry(&self, code: &str) -> Option<&content::SituationAi> {
        self.content.ai.get(code).or_else(|| {
            self.hotels
                .iter()
                .flat_map(|h| h.drawn.values().flatten())
                .find(|i| i.situation.code == code)
                .map(|i| &i.ai)
        })
    }

    /// AI podaci situacije za primjerak hotela `hotel` (njegova imena likova i brojevi).
    pub fn ai_entry_for(&self, hotel: HotelId, code: &str) -> Option<&content::SituationAi> {
        self.content.ai.get(code).or_else(|| {
            self.hotel(hotel)
                .ok()?
                .drawn
                .values()
                .flatten()
                .find(|i| i.situation.code == code)
                .map(|i| &i.ai)
        })
    }

    /// Kartica situacije hotela u krugu `r` (odjel, vrsta, važnost, opis, uloge); prazna za sidra i v1.
    pub fn situation_card(
        &self,
        hotel: HotelId,
        r: RoundKey,
        code: &str,
    ) -> Result<SituationCard, GameError> {
        let h = self.hotel(hotel)?;
        let Some(i) = h
            .drawn
            .get(&r)
            .and_then(|l| l.iter().find(|i| i.situation.code == code))
        else {
            return Ok(SituationCard::default());
        };
        let template = self
            .content
            .situation_templates
            .as_ref()
            .and_then(|t| t.templates.iter().find(|t| t.code == i.drawn.template));
        let character_key = template
            .and_then(|t| t.character.as_ref())
            .map(|c| c.role.as_str());
        let roles = i
            .drawn
            .roles
            .iter()
            .map(|(key, fill)| {
                let role = match (&i.ai.character, character_key == Some(key.as_str())) {
                    (Some(c), true) => Some(c.role.clone()),
                    _ => fill.role_type.clone().or_else(|| {
                        template
                            .and_then(|t| t.roles.get(key))
                            .and_then(|r| r.title.clone())
                    }),
                };
                InstanceRole {
                    key: key.clone(),
                    name: fill.name.clone(),
                    role,
                }
            })
            .collect();
        Ok(SituationCard {
            department: i.drawn.department.as_ref().map(|d| d.as_str().to_owned()),
            kind: Some(i.drawn.kind),
            weight: Some(i.drawn.weight),
            description: Some(i.description.clone()),
            roles,
        })
    }

    /// Voditelj kojem student može prepustiti situaciju: odjel situacije; za situaciju cijelog hotela (predložak
    /// razine `hotel`, sidra) odjel s najviše zaposlenih, a pri jednakosti prvi po kodu odjela 🟨. `None` u v1 igri
    /// i kad nijedan odjel nije otvoren.
    pub fn situation_leader(
        &self,
        hotel: HotelId,
        code: &str,
    ) -> Result<Option<String>, GameError> {
        let h = self.hotel(hotel)?;
        let (r, _) = self.situation_round_for(hotel, code)?;
        let own = h
            .drawn
            .get(&r)
            .and_then(|l| l.iter().find(|i| i.situation.code == code))
            .and_then(|i| i.drawn.department.as_ref());
        if let Some(d) = own {
            return Ok(Some(d.as_str().to_owned()));
        }
        let state = self.state_at(hotel, r)?;
        let Ok(s) = self.model().v2_state(&state) else {
            return Ok(None);
        };
        let mut best: Option<(&DeptCode, u32)> = None;
        for (code, d) in s.departments.iter().filter(|(_, d)| d.open) {
            let staff = u32::from(d.permanent) + u32::from(d.seasonal);
            if best.is_none_or(|(_, b)| staff > b) {
                best = Some((code, staff));
            }
        }
        Ok(best.map(|(c, _)| c.as_str().to_owned()))
    }
}
