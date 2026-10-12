//! Kraj sezone v2 igre (kraj-sezone-spec §5–§6.1): konačni poredak napretka, krivulja, indeksi početak → kraj i,
//! kad je sezona gotova, financije i odluke svakog hotela (Leon 13.10.: s brojkama, tek na kraju). Tuđi ljudski
//! hotel i dalje pokazuje samo indekse (ARHITEKTURA §12). Ništa se ne računa novo: samo zbrojevi i prosjeci polja koja
//! je obračun već zapisao (`RoundResolved`), kroz `engine::calc`.

use engine::calc as c;
use sim_hotel_v2::usali::{UsaliStatement, statement, sum_statements};

use crate::dto::{MaintenanceCode, TrainingCode};
use crate::error::{GameError, corrupt};
use crate::event::{Controller, HotelResolution};
use crate::game::{Game, HotelRecord};
use crate::ids::HotelId;
use crate::model::RoundInfo;
use crate::round::RoundKey;
use crate::view::Seat;

/// Napredak hotela na kraju jednog kruga (isti broj kao `PublicHotel::months`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurvePoint {
    pub round: RoundKey,
    pub progress_hundredths: i32,
}

/// Novac sezone: prihod i GOP isti su brojevi kao „Račun dobiti → Sezona".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonFinances {
    pub total_revenue_cents: i64,
    pub gop_cents: i64,
    /// GOP kao udio prihoda u baznim bodovima; `None` bez prihoda.
    pub gop_margin_bp: Option<i64>,
    pub cash_start_cents: i64,
    pub cash_end_cents: i64,
}

/// Odluke hotela kroz sezonu, iz obračunatih planova i situacija.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonDecisions {
    /// Prosjek cijene sobe po krugovima (zaokruženo na cent).
    pub avg_price_cents: i64,
    pub marketing_total_cents: i64,
    pub wage_pct_min: u16,
    pub wage_pct_max: u16,
    /// Najviše sezonskih radnika istodobno (na kraju kruga).
    pub seasonal_peak: u16,
    /// Krugovi s obukom (osnovnom ili punom).
    pub training_rounds: u8,
    /// Krugovi u kojima je održavanje bar jednog odjela odgođeno.
    pub maintenance_deferred_rounds: u8,
    /// Krug obnove, ako je bila.
    pub renovation_round: Option<RoundKey>,
    pub reply_reviews_rounds: u8,
    /// Sve situacije sezone i koliko ih je ostalo bez odgovora.
    pub situations: u16,
    pub unanswered: u16,
    /// Prosjek ocjena 1–5 ×100; `None` bez situacija.
    pub avg_quality_hundredths: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonHotel {
    pub hotel_id: HotelId,
    pub slot: u8,
    pub name: String,
    pub seat: Seat,
    /// Hotel onoga tko gleda.
    pub mine: bool,
    /// 1..=4; jednak konačni napredak dijeli mjesto.
    pub rank: u8,
    /// Pobjednik je prvi po napretku (Leon 13.10.); kod dijeljenog prvog mjesta ih je više.
    pub winner: bool,
    pub progress_hundredths: i32,
    pub start: [u16; 5],
    pub end: [u16; 5],
    pub curve: Vec<CurvePoint>,
    /// `None` za tuđi ljudski hotel.
    pub finances: Option<SeasonFinances>,
    /// `None` za tuđi ljudski hotel.
    pub decisions: Option<SeasonDecisions>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonView {
    pub rounds: Vec<RoundInfo>,
    /// Redom poretka: mjesto, pa slot.
    pub hotels: Vec<SeasonHotel>,
    /// Sezonski mentor hotela koji gleda; `None` dok ga radnik ne zapiše (ili za igru završenu prije KS-2).
    pub mentor: Option<String>,
}

fn calc(e: c::CalcError) -> GameError {
    corrupt(format!("kraj sezone: {e:?}"))
}

fn count(n: usize) -> Result<u8, GameError> {
    u8::try_from(n).map_err(|_| corrupt("kraj sezone: previše krugova"))
}

/// Mjesto po napretku: 1 + broj hotela sa strogo većim napretkom.
fn rank_of(progress: i32, all: &[i32]) -> Result<u8, GameError> {
    count(all.iter().filter(|&&p| p > progress).count())?
        .checked_add(1)
        .ok_or_else(|| corrupt("kraj sezone: mjesto"))
}

impl Game {
    /// Kraj sezone gledan iz hotela `viewer`. v1 igra nema ovaj prikaz (`WrongModel`); dok svi krugovi nisu
    /// obračunati, `SeasonNotOver`.
    pub fn season(&self, viewer: HotelId) -> Result<SeasonView, GameError> {
        self.model().require_v2(&self.content)?;
        let me = self.hotel(viewer)?;
        if self.current_round().is_some() {
            return Err(GameError::SeasonNotOver);
        }
        let mut hotels = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            hotels.push(self.season_hotel(h, viewer)?);
        }
        let all: Vec<i32> = hotels.iter().map(|h| h.progress_hundredths).collect();
        for h in &mut hotels {
            h.rank = rank_of(h.progress_hundredths, &all)?;
            h.winner = h.rank == 1;
        }
        hotels.sort_by_key(|h| (h.rank, h.slot));
        Ok(SeasonView {
            rounds: self.rounds_info(),
            hotels,
            mentor: me.season_debrief.as_ref().map(|d| d.text.clone()),
        })
    }

    fn season_hotel(&self, h: &HotelRecord, viewer: HotelId) -> Result<SeasonHotel, GameError> {
        let id = h.setup.hotel_id;
        let public = self.public_hotel(id)?;
        let last = public
            .months
            .last()
            .ok_or_else(|| corrupt("kraj sezone bez obračunatog kruga"))?;
        let (progress_hundredths, end) = (last.progress_hundredths, last.indices);
        let curve = public
            .months
            .iter()
            .map(|m| CurvePoint {
                round: m.round,
                progress_hundredths: m.progress_hundredths,
            })
            .collect();
        let mine = id == viewer;
        let hidden = !mine && matches!(h.setup.controller, Controller::Human { .. });
        let (finances, decisions) = if hidden {
            (None, None)
        } else {
            let res = self.season_resolutions(id)?;
            (
                Some(self.season_finances(id, &res)?),
                Some(decisions(&res)?),
            )
        };
        Ok(SeasonHotel {
            hotel_id: id,
            slot: public.slot,
            name: public.name,
            seat: public.seat,
            mine,
            rank: 0,
            winner: false,
            progress_hundredths,
            start: public.start,
            end,
            curve,
            finances,
            decisions,
        })
    }

    /// Obračun hotela u svakom krugu, redom sezone (svi su obračunati).
    fn season_resolutions(
        &self,
        id: HotelId,
    ) -> Result<Vec<(RoundKey, &HotelResolution)>, GameError> {
        self.rounds
            .iter()
            .map(|r| {
                self.resolved
                    .get(r)
                    .and_then(|rr| rr.hotels.iter().find(|x| x.hotel_id == id))
                    .map(|x| (*r, x))
                    .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))
            })
            .collect()
    }

    fn season_finances(
        &self,
        id: HotelId,
        res: &[(RoundKey, &HotelResolution)],
    ) -> Result<SeasonFinances, GameError> {
        let st = self.season_statement(id)?;
        let (Some((_, first)), Some((_, last))) = (res.first(), res.last()) else {
            return Err(corrupt("kraj sezone bez obračunatog kruga"));
        };
        Ok(SeasonFinances {
            total_revenue_cents: st.total_revenue_cents,
            gop_cents: st.gop_cents,
            gop_margin_bp: st.gop_margin_bp,
            cash_start_cents: first.report.before.cash_cents,
            cash_end_cents: last.report.after.cash_cents,
        })
    }

    /// Ostvareni račun dobiti cijele sezone: isti zbroj kao `profit_and_loss(h, None, Season).actual`, bez budžeta.
    fn season_statement(&self, id: HotelId) -> Result<UsaliStatement, GameError> {
        let each = self
            .resolved_accounts(id)?
            .iter()
            .map(|(_, a)| statement(a).map_err(calc))
            .collect::<Result<Vec<_>, _>>()?;
        match each.as_slice() {
            [one] => Ok(one.clone()),
            many => sum_statements(many).map_err(calc),
        }
    }
}

fn decisions(res: &[(RoundKey, &HotelResolution)]) -> Result<SeasonDecisions, GameError> {
    let rounds = i128::try_from(res.len()).map_err(|_| corrupt("kraj sezone: krugovi"))?;
    let mut price = 0_i128;
    let mut marketing = 0_i128;
    let (mut wage_min, mut wage_max) = (u16::MAX, u16::MIN);
    let mut seasonal_peak = 0_u16;
    let (mut training, mut deferred, mut replies) = (0_usize, 0_usize, 0_usize);
    let mut renovation_round = None;
    let (mut situations, mut unanswered) = (0_usize, 0_usize);
    let mut quality = 0_i128;
    for &(round, r) in res {
        let p = &r.plan;
        price = c::add(price, i128::from(p.price_cents)).map_err(calc)?;
        marketing = c::add(marketing, i128::from(p.marketing_cents)).map_err(calc)?;
        wage_min = wage_min.min(p.wage_pct);
        wage_max = wage_max.max(p.wage_pct);
        seasonal_peak = seasonal_peak.max(r.report.after.seasonal);
        if p.training != TrainingCode::None {
            training = training.saturating_add(1);
        }
        let deferred_here = if p.departments.is_empty() {
            p.maintenance == MaintenanceCode::Deferred
        } else {
            p.departments
                .values()
                .any(|d| d.maintenance == MaintenanceCode::Deferred)
        };
        if deferred_here {
            deferred = deferred.saturating_add(1);
        }
        if p.reply_reviews {
            replies = replies.saturating_add(1);
        }
        if p.renovation && renovation_round.is_none() {
            renovation_round = Some(round);
        }
        for s in &r.situations {
            situations = situations.saturating_add(1);
            if s.answer.is_none() && !s.free_text {
                unanswered = unanswered.saturating_add(1);
            }
            quality = c::add(quality, i128::from(s.quality)).map_err(calc)?;
        }
    }
    if res.is_empty() {
        return Err(corrupt("kraj sezone bez obračunatog kruga"));
    }
    let avg_quality_hundredths = if situations == 0 {
        None
    } else {
        let n = i128::try_from(situations).map_err(|_| corrupt("kraj sezone: situacije"))?;
        let q = c::div_round(c::mul(quality, 100).map_err(calc)?, n).map_err(calc)?;
        Some(c::narrow(q).map_err(calc)?)
    };
    let small = |n: usize| u16::try_from(n).map_err(|_| corrupt("kraj sezone: situacije"));
    Ok(SeasonDecisions {
        avg_price_cents: c::narrow(c::div_round(price, rounds).map_err(calc)?).map_err(calc)?,
        marketing_total_cents: c::narrow(marketing).map_err(calc)?,
        wage_pct_min: wage_min,
        wage_pct_max: wage_max,
        seasonal_peak,
        training_rounds: count(training)?,
        maintenance_deferred_rounds: count(deferred)?,
        renovation_round,
        reply_reviews_rounds: count(replies)?,
        situations: small(situations)?,
        unanswered: small(unanswered)?,
        avg_quality_hundredths,
    })
}
