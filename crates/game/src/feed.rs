//! Dnevnik jednog hotela kao popis stavki za ekran (DEMO §3): samo događaji koji se tiču tog hotela.

use crate::actor::Actor;
use crate::dto::{LineDto, MonthCode, PlanDto};
use crate::error::{GameError, corrupt};
use crate::event::{AiSource, GameEvent, VerdictRecorded};
use crate::game::Game;
use crate::ids::HotelId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedItem {
    HotelReceived {
        seq: i64,
        name: String,
        profile: String,
    },
    PlanChanged {
        seq: i64,
        month: MonthCode,
        plan: PlanDto,
        by: Actor,
    },
    AnswerChosen {
        seq: i64,
        month: MonthCode,
        situation: String,
        title: String,
        answer: u8,
        label: String,
        by: Actor,
    },
    RoundClosed {
        seq: i64,
        month: MonthCode,
        by: Actor,
    },
    SituationIgnored {
        seq: i64,
        month: MonthCode,
        situation: String,
        title: String,
    },
    MonthResolved {
        seq: i64,
        month: MonthCode,
        indices: [u16; 5],
        progress_hundredths: i32,
    },
    Change {
        seq: i64,
        month: MonthCode,
        line: LineDto,
    },
    /// Slobodan odgovor igrača na situaciju (tekst je u pogledu situacije, ne u feedu).
    Response {
        seq: i64,
        month: MonthCode,
        situation: String,
        title: String,
    },
    /// Presuda na slobodni odgovor; `neutral` = rezervna (bez AI-ja), `late` = stigla nakon obračuna.
    Verdict {
        seq: i64,
        month: MonthCode,
        situation: String,
        title: String,
        quality: u8,
        neutral: bool,
        late: bool,
    },
    /// Mentorov osvrt na mjesec.
    Mentor {
        seq: i64,
        month: MonthCode,
        text: String,
    },
}

impl Game {
    /// Kronološki feed hotela; sve stavke istog događaja nose isti `seq`.
    pub fn feed(&self, hotel: HotelId) -> Result<Vec<FeedItem>, GameError> {
        let h = self.hotel(hotel)?;
        let mut out = Vec::new();
        for r in &self.history {
            let seq = r.seq;
            match &r.event {
                GameEvent::GameCreated(_) => out.push(FeedItem::HotelReceived {
                    seq,
                    name: h.setup.name.clone(),
                    profile: h.setup.profile.clone(),
                }),
                GameEvent::PlanSet(p) if p.hotel_id == hotel => out.push(FeedItem::PlanChanged {
                    seq,
                    month: p.month,
                    plan: p.plan.clone(),
                    by: r.actor.clone(),
                }),
                GameEvent::AnswerChosen(a) if a.hotel_id == hotel => {
                    let (title, label) = self.answer_texts(&a.situation, a.answer)?;
                    out.push(FeedItem::AnswerChosen {
                        seq,
                        month: a.month,
                        situation: a.situation.clone(),
                        title,
                        answer: a.answer,
                        label,
                        by: r.actor.clone(),
                    });
                }
                GameEvent::RoundClosed(c) => out.push(FeedItem::RoundClosed {
                    seq,
                    month: c.month,
                    by: r.actor.clone(),
                }),
                GameEvent::RoundResolved(rr) => {
                    let res = rr
                        .hotels
                        .iter()
                        .find(|x| x.hotel_id == hotel)
                        .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
                    for s in res
                        .situations
                        .iter()
                        .filter(|s| s.answer.is_none() && !s.free_text)
                    {
                        out.push(FeedItem::SituationIgnored {
                            seq,
                            month: rr.month,
                            situation: s.code.clone(),
                            title: self.situation_title(&s.code)?,
                        });
                    }
                    out.push(FeedItem::MonthResolved {
                        seq,
                        month: rr.month,
                        indices: res.indices,
                        progress_hundredths: res.progress_hundredths,
                    });
                    out.extend(res.report.lines.iter().map(|l| FeedItem::Change {
                        seq,
                        month: rr.month,
                        line: l.clone(),
                    }));
                }
                GameEvent::ResponseSubmitted(e) if e.hotel_id == hotel => {
                    out.push(FeedItem::Response {
                        seq,
                        month: e.month,
                        situation: e.situation.clone(),
                        title: self.situation_title(&e.situation)?,
                    });
                }
                GameEvent::VerdictRecorded(v) if v.hotel_id == hotel => {
                    out.push(self.verdict_item(seq, v, false)?);
                }
                GameEvent::VerdictLate(v) if v.hotel_id == hotel => {
                    out.push(self.verdict_item(seq, v, true)?);
                }
                GameEvent::DebriefRecorded(d) if d.hotel_id == hotel => {
                    out.push(FeedItem::Mentor {
                        seq,
                        month: d.month,
                        text: d.text.clone(),
                    });
                }
                // Razgovor je dio situacije, ne feeda; ostalo je tuđi hotel.
                GameEvent::PlanSet(_)
                | GameEvent::AnswerChosen(_)
                | GameEvent::ChatMessageSent(_)
                | GameEvent::CharacterReplied(_)
                | GameEvent::ResponseSubmitted(_)
                | GameEvent::VerdictRecorded(_)
                | GameEvent::VerdictLate(_)
                | GameEvent::DebriefRecorded(_) => {}
            }
        }
        Ok(out)
    }

    fn verdict_item(
        &self,
        seq: i64,
        v: &VerdictRecorded,
        late: bool,
    ) -> Result<FeedItem, GameError> {
        Ok(FeedItem::Verdict {
            seq,
            month: v.month,
            situation: v.situation.clone(),
            title: self.situation_title(&v.situation)?,
            quality: v.quality,
            neutral: v.source == AiSource::Fallback,
            late,
        })
    }

    fn situation_title(&self, code: &str) -> Result<String, GameError> {
        self.content
            .situations
            .iter()
            .find(|t| t.code == code)
            .map(|t| t.title.clone())
            .ok_or_else(|| corrupt(format!("nepoznata situacija {code}")))
    }

    fn answer_texts(&self, code: &str, answer: u8) -> Result<(String, String), GameError> {
        let t = self
            .content
            .situations
            .iter()
            .find(|t| t.code == code)
            .ok_or_else(|| corrupt(format!("nepoznata situacija {code}")))?;
        let a = t
            .answers
            .get(usize::from(answer))
            .ok_or_else(|| corrupt("indeks odgovora"))?;
        Ok((t.title.clone(), a.label.clone()))
    }
}
