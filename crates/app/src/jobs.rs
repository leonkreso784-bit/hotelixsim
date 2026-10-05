//! Radnik obračuna (ARHITEKTURA §9.4): preuzima poslove iz reda i obračunava rundu.
use game::GameId;
use game::dto::MonthCode;
use serde::{Deserialize, Serialize};
use sim_hotel::Month;

use crate::demo::{fold_raw, new_event};
use crate::{AppError, Decision, Deps, Job, RawEvent};

pub const RESOLVE_KIND: &str = "resolve_round";
/// Zakup posla (ARHITEKTURA §9.4): radnik koji ga ne završi u roku vraća ga u red.
pub const LEASE_SECS: i32 = 30;

pub fn resolve_key(game: GameId, m: Month) -> String {
    format!("resolve:{}:{}", game.0, m.code())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvePayload {
    pub month: MonthCode,
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
            deps.jobs.fail(job.id, format!("{e}")).await?;
            Err(e)
        }
    }
}

async fn run_job(deps: &Deps, job: &Job) -> Result<(), AppError> {
    if job.kind != RESOLVE_KIND {
        return Err(AppError::internal(format!(
            "nepoznata vrsta posla {}",
            job.kind
        )));
    }
    let p: ResolvePayload = serde_json::from_value(job.payload.clone())
        .map_err(|e| AppError::internal(e.to_string()))?;
    let id = job.id;
    deps.log
        .decide(
            job.game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                // idempotentno: već obračunata runda daje praznu odluku, ali se posao ipak zatvara
                let events = g
                    .resolve_job(p.month.into())?
                    .iter()
                    .map(|(a, e)| new_event(a, e))
                    .collect();
                Ok(Decision {
                    events,
                    enqueue: None,
                    finish_job: Some(id),
                })
            }),
        )
        .await?;
    Ok(())
}
