//! Radnik (ARHITEKTURA §9.4): preuzima poslove iz reda — obračun runde, odgovor lika, presudu i mentora.
//! AI se zove IZVAN `decide`; unutar `decide` se nad svježom igrom ponovno provjerava smije li se rezultat upisati.
use game::RoundKey;
use game::{
    Actor, AiSource, Controller, GameError, GameEvent, GameId, HotelId, NEUTRAL_FAILED, ReplyBody,
    ResolveStep, RoundStatus,
};
use serde::{Deserialize, Serialize};

use crate::demo::{fold_raw, load_game, new_event};
use crate::{
    AiError, AppError, Decision, Deps, Job, JobClaim, MAX_ATTEMPTS, NewEvent, NewJob, RawEvent,
};

pub const RESOLVE_KIND: &str = "resolve_round";
pub const CHAT_KIND: &str = "character_reply";
pub const ASSESS_KIND: &str = "assess_response";
pub const DEBRIEF_KIND: &str = "debrief";
/// Zakup posla (ARHITEKTURA §9.4): radnik koji ga ne završi u roku vraća ga u red.
/// AI poziv za presudu može trajati desetke sekundi.
pub const LEASE_SECS: i32 = 180;
/// Od zatvaranja mjeseca do neutralne presude.
pub const VERDICT_DEADLINE_SECS: i64 = 120;
pub const VERDICT_WAIT_SECS: i32 = 5;

pub fn resolve_key(game: GameId, m: RoundKey) -> String {
    format!("resolve:{}:{}", game.0, m.code())
}

pub fn chat_key(g: GameId, h: HotelId, code: &str, n: u8) -> String {
    format!("chat:{}:{}:{code}:{n}", g.0, h.0)
}

pub fn assess_key(g: GameId, h: HotelId, code: &str) -> String {
    format!("assess:{}:{}:{code}", g.0, h.0)
}

pub fn debrief_key(g: GameId, m: RoundKey, h: HotelId) -> String {
    format!("debrief:{}:{}:{}", g.0, m.code(), h.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvePayload {
    pub month: RoundKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatPayload {
    pub hotel_id: HotelId,
    pub situation: String,
    pub reply_to: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessPayload {
    pub hotel_id: HotelId,
    pub situation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebriefPayload {
    pub hotel_id: HotelId,
    pub month: RoundKey,
}

/// Jedan korak radnika: preuzmi posao, obradi ga, a grešku zapiši u red (ponovni pokušaj ili `failed`).
/// `true` = obradio posao, `false` = red je prazan.
pub async fn run_next_job(deps: &Deps) -> Result<bool, AppError> {
    let Some(job) = deps.jobs.claim(LEASE_SECS).await? else {
        return Ok(false);
    };
    match run_job(deps, &job).await {
        Ok(()) => Ok(true),
        Err(e) => {
            // izvornu grešku posla ne smije progutati greška zapisivanja u red
            if let Err(f) = deps.jobs.fail(job.claim(), format!("{e}")).await {
                return Err(AppError {
                    code: e.code,
                    detail: format!("{e}; uz to zapis neuspjeha u red nije uspio: {f}"),
                });
            }
            Err(e)
        }
    }
}

async fn run_job(deps: &Deps, job: &Job) -> Result<(), AppError> {
    match job.kind.as_str() {
        RESOLVE_KIND => run_resolve(deps, job).await,
        CHAT_KIND => run_chat(deps, job).await,
        ASSESS_KIND => run_assess(deps, job).await,
        DEBRIEF_KIND => run_debrief(deps, job).await,
        other => Err(AppError::internal(format!("nepoznata vrsta posla {other}"))),
    }
}

/// Samo za testove: pokreće jedan posao mimo reda (npr. spori AI koji je završio nakon obračuna).
#[doc(hidden)]
pub async fn run_job_for_test(deps: &Deps, job: &Job) -> Result<(), AppError> {
    run_job(deps, job).await
}

/// Isključen AI i odbijanje zahtjeva se ne ponavljaju (ponavljanje ih ne mijenja): odmah rezerva.
/// Ostale greške do zadnjeg pokušaja.
fn retry(job: &Job, e: &AiError) -> bool {
    !matches!(e, AiError::Disabled | AiError::Refused(_)) && job.attempts < MAX_ATTEMPTS
}

fn payload<T: serde::de::DeserializeOwned>(job: &Job) -> Result<T, AppError> {
    serde_json::from_value(job.payload.clone()).map_err(|e| AppError::internal(e.to_string()))
}

fn system_events(evs: &[GameEvent]) -> Vec<NewEvent> {
    evs.iter().map(|e| new_event(&Actor::System, e)).collect()
}

/// Zatvori posao bez događaja (nema više što raditi).
async fn finish(deps: &Deps, game: GameId, claim: JobClaim) -> Result<(), AppError> {
    deps.log
        .decide(
            game,
            Box::new(move |_raw: &[RawEvent]| {
                Ok(Decision {
                    finish_job: Some(claim),
                    ..Decision::default()
                })
            }),
        )
        .await?;
    Ok(())
}

/// Presuda je gotova: AI-jeva je već upisana, ili je uz neutralnu već stigla i zakašnjela.
/// Neutralna presuda sama NE znači gotovo: AI-jeva koja stigne poslije upisuje se kao zakašnjela.
fn verdict_settled(g: &game::Game, hotel: HotelId, code: &str) -> Result<bool, GameError> {
    let (m, _) = g.situation_round(code)?;
    let fr = g
        .hotel(hotel)?
        .responses
        .get(&(m, code.to_owned()))
        .ok_or(GameError::UnknownSituation)?;
    Ok(fr
        .verdict
        .as_ref()
        .is_some_and(|v| v.source == AiSource::Ai)
        || !fr.late.is_empty())
}

async fn run_resolve(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: ResolvePayload = payload(job)?;
    let m = p.month;
    let claim = job.claim();
    let deadline_passed = job.age_secs >= VERDICT_DEADLINE_SECS;
    let g = load_game(deps, job.game_id).await?;
    if !deadline_passed && !g.pending_verdicts(m).is_empty() {
        deps.jobs.defer(claim, VERDICT_WAIT_SECS).await?;
        return Ok(());
    }
    let game_id = job.game_id;
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                let evs = match g.resolve_job(m, deadline_passed)? {
                    ResolveStep::Done(evs) => evs,
                    ResolveStep::WaitForVerdicts(n) => {
                        return Err(AppError::internal(format!(
                            "{n} presuda čeka unutar zaključavanja"
                        )));
                    }
                };
                let resolved = evs
                    .iter()
                    .any(|(_, e)| matches!(e, GameEvent::RoundResolved(_)));
                let mut enqueue = Vec::new();
                if resolved {
                    for h in g
                        .hotels
                        .iter()
                        .filter(|h| matches!(h.setup.controller, Controller::Human { .. }))
                    {
                        let hotel_id = h.setup.hotel_id;
                        enqueue.push(NewJob {
                            key: debrief_key(game_id, m, hotel_id),
                            kind: DEBRIEF_KIND.to_owned(),
                            game_id,
                            payload: serde_json::to_value(DebriefPayload { hotel_id, month: m })
                                .map_err(|e| AppError::internal(e.to_string()))?,
                        });
                    }
                }
                Ok(Decision {
                    events: evs.iter().map(|(a, e)| new_event(a, e)).collect(),
                    enqueue,
                    finish_job: Some(claim),
                })
            }),
        )
        .await?;
    Ok(())
}

async fn run_chat(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: ChatPayload = payload(job)?;
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if g.status(g.situation_round(&p.situation)?.0)? != RoundStatus::Open {
        return finish(deps, job.game_id, claim).await;
    }
    let input = g.character_input(p.hotel_id, &p.situation)?;
    let reply = match deps.ai.character(input).await {
        Ok(out) => ReplyBody {
            text: out.value,
            source: AiSource::Ai,
            meta: Some(out.meta),
        },
        Err(e) if retry(job, &e) => return Err(AppError::internal(e.to_string())),
        Err(_) => {
            let text = g
                .content
                .ai
                .get(&p.situation)
                .and_then(|a| a.character.as_ref())
                .map(|c| c.fallback_reply.clone())
                .ok_or_else(|| AppError::internal("situacija bez lika"))?;
            ReplyBody {
                text,
                source: AiSource::Fallback,
                meta: None,
            }
        }
    };
    let (hotel, code, n) = (p.hotel_id, p.situation, p.reply_to);
    deps.log
        .decide(
            job.game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                let ev = g.record_reply(hotel, &code, n, reply)?;
                Ok(Decision {
                    events: system_events(ev.as_slice()),
                    finish_job: Some(claim),
                    ..Decision::default()
                })
            }),
        )
        .await?;
    Ok(())
}

async fn run_assess(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: AssessPayload = payload(job)?;
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if verdict_settled(&g, p.hotel_id, &p.situation)? {
        return finish(deps, job.game_id, claim).await; // AI presuda već postoji
    }
    let input = g.assess_input(p.hotel_id, &p.situation)?;
    let judged = match deps.ai.assess(input).await {
        Ok(out) => g
            .validate_verdict(&p.situation, out.value)
            .map(|v| (v, out.meta))
            .map_err(|e| {
                (
                    format!("presuda nije prošla provjeru: {e:?}"),
                    job.attempts < MAX_ATTEMPTS,
                )
            }),
        Err(e) => Err((e.to_string(), retry(job, &e))),
    };
    let judged = match judged {
        Ok(ok) => Some(ok),
        Err((msg, true)) => return Err(AppError::internal(msg)),
        Err(_) => None, // zadnji pokušaj: neutralna presuda po pravilu
    };
    let (hotel, code) = (p.hotel_id, p.situation);
    deps.log
        .decide(
            job.game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                let ev = match judged {
                    Some(_) if verdict_settled(&g, hotel, &code)? => None,
                    // VerdictRecorded dok odgovor čeka, VerdictLate ako je u međuvremenu došla neutralna presuda
                    Some((v, meta)) => Some(g.record_verdict(hotel, &code, v, meta)?),
                    None if g.response_pending(hotel, &code)? => {
                        Some(g.neutral_verdict(hotel, &code, NEUTRAL_FAILED)?)
                    }
                    None => None,
                };
                Ok(Decision {
                    events: system_events(ev.as_slice()),
                    finish_job: Some(claim),
                    ..Decision::default()
                })
            }),
        )
        .await?;
    Ok(())
}

async fn run_debrief(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: DebriefPayload = payload(job)?;
    let m = p.month;
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if g.hotel(p.hotel_id)?.debriefs.contains_key(&m) {
        return finish(deps, job.game_id, claim).await;
    }
    let input = g.debrief_input(p.hotel_id, m)?;
    let (text, source, meta) = match deps.ai.debrief(input.clone()).await {
        Ok(out) => (out.value, AiSource::Ai, Some(out.meta)),
        Err(e) if retry(job, &e) => return Err(AppError::internal(e.to_string())),
        Err(_) => (game::fallback_debrief(&input), AiSource::Fallback, None),
    };
    let hotel = p.hotel_id;
    deps.log
        .decide(
            job.game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                let ev = g.record_debrief(hotel, m, text, source, meta)?;
                Ok(Decision {
                    events: system_events(ev.as_slice()),
                    finish_job: Some(claim),
                    ..Decision::default()
                })
            }),
        )
        .await?;
    Ok(())
}
