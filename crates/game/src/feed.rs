//! Dnevnik jednog hotela kao popis stavki za ekran (DEMO §3): samo događaji koji se tiču tog hotela.

use crate::actor::Actor;
use crate::dto::{LineDto, MonthCode, PlanDto};
use crate::error::{GameError, corrupt};
use crate::event::GameEvent;
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
                    for s in res.situations.iter().filter(|s| s.answer.is_none()) {
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
                GameEvent::PlanSet(_) | GameEvent::AnswerChosen(_) => {}
            }
        }
        Ok(out)
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
