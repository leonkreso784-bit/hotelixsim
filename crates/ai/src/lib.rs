//! Klijent za Claude Messages API (sirovi HTTP: za Rust nema službenog SDK-a) iza `app::AiPort`.
//! Ključ je samo ovdje i nikad se ne logira ni ne ispisuje.

pub mod benchmark;
pub mod prompts;
pub mod request;
pub mod response;

use app::spend::UsageSink;
use app::{AiError, AiOut, AiPort, BoxFut};
use game::{AiMeta, AssessInput, CharacterInput, DebriefInput, SeasonDebriefInput, VerdictDraft};
use std::sync::Arc;
use std::time::Duration;

pub use app::DisabledAi as Disabled;
use response::Reply;

/// Vremensko ograničenje jednog zahtjeva; manje od zakupa posla (180 s).
const TIMEOUT: Duration = Duration::from_secs(150);

#[derive(Clone, Debug)]
pub struct Models {
    pub judge: String,
    pub character: String,
    pub mentor: String,
}

impl Default for Models {
    fn default() -> Self {
        Models {
            judge: "claude-sonnet-5-5".into(),
            character: "claude-haiku-5-5".into(),
            mentor: "claude-sonnet-5-5".into(),
        }
    }
}

impl Models {
    /// Zadani modeli, nadjačani varijablama okoline (isti izvor za server i mjerilo).
    pub fn from_env() -> Models {
        let d = Models::default();
        let pick = |var: &str, def: String| {
            std::env::var(var)
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or(def)
        };
        Models {
            judge: pick("HOTELIXSIM_AI_MODEL_PRESUDA", d.judge),
            character: pick("HOTELIXSIM_AI_MODEL_LIK", d.character),
            mentor: pick("HOTELIXSIM_AI_MODEL_MENTOR", d.mentor),
        }
    }
}

fn http_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("reqwest klijent se mora izgraditi")
}

/// Zahtjev u letu: ako završi bez odgovora (istek vremena, prekid pri gašenju), Anthropic je možda ipak generirao
/// i naplatio odgovor. Tada brojaču ide PROCJENA najgoreg slučaja: `max_tokens` × cijena izlaza modela, ulaz 0 (🟨).
/// Odgovor s HTTP greškom i greška spajanja (zahtjev nije stigao) se ne broje: čuvar se tada razoružava.
struct InFlight {
    sink: Option<Arc<dyn UsageSink>>,
    model: String,
    max_tokens: u32,
}

impl InFlight {
    fn disarm(&mut self) {
        self.sink = None;
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        let Some(sink) = self.sink.take() else { return };
        let usage = app::spend::ApiUsage {
            model: std::mem::take(&mut self.model),
            tokens: app::spend::TokenUsage {
                output: self.max_tokens,
                ..Default::default()
            },
        };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move { sink.record(usage).await });
        }
    }
}

pub struct Anthropic {
    http: reqwest::Client,
    key: String,
    models: Models,
    base_url: String,
    /// Prima potrošnju svakog odgovora 2xx prije tumačenja (strop troška, pilot dio 3).
    usage_sink: Option<Arc<dyn UsageSink>>,
}

impl std::fmt::Debug for Anthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Anthropic")
            .field("models", &self.models)
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl Anthropic {
    pub fn new(key: String, models: Models) -> Anthropic {
        Anthropic {
            http: http_client(TIMEOUT),
            key,
            models,
            base_url: "https://api.anthropic.com".into(),
            usage_sink: None,
        }
    }

    /// Druga adresa umjesto api.anthropic.com (probni poslužitelj u testovima).
    #[must_use]
    pub fn with_base_url(mut self, url: String) -> Anthropic {
        self.base_url = url;
        self
    }

    /// Kraće ili dulje vremensko ograničenje zahtjeva (testovi).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Anthropic {
        self.http = http_client(timeout);
        self
    }

    /// Brojač troška: dobiva potrošnju svakog odgovora 2xx, i odrezanog i neparsabilnog.
    #[must_use]
    pub fn with_usage_sink(mut self, sink: Arc<dyn UsageSink>) -> Anthropic {
        self.usage_sink = Some(sink);
        self
    }

    async fn post(&self, body: serde_json::Value) -> Result<Reply, AiError> {
        let mut req = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.key)
            .header("anthropic-version", request::API_VERSION)
            .header("content-type", "application/json");
        if let Some(beta) = request::beta_header(&body) {
            req = req.header("anthropic-beta", beta);
        }
        let mut in_flight = InFlight {
            sink: self.usage_sink.clone(),
            model: body["model"].as_str().unwrap_or_default().to_owned(),
            max_tokens: body["max_tokens"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .unwrap_or(0),
        };
        let resp = match req.json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                if !e.is_timeout() {
                    in_flight.disarm();
                }
                return Err(AiError::Unavailable(format!("mreža: {}", e.without_url())));
            }
        };
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            in_flight.disarm();
        }
        let text = resp
            .text()
            .await
            .map_err(|e| AiError::Unavailable(format!("čitanje odgovora: {}", e.without_url())))?;
        in_flight.disarm();
        if (200..300).contains(&status)
            && let Some(sink) = &self.usage_sink
            && let Some(mut usage) = response::parse_usage(&text)
        {
            if usage.model.is_empty() {
                usage.model = body["model"].as_str().unwrap_or_default().to_owned();
            }
            sink.record(usage).await;
        }
        response::parse_reply(status, &text)
    }
}

fn meta(reply: &Reply, prompt: &prompts::Prompt) -> AiMeta {
    AiMeta {
        model: reply.model.clone(),
        prompt: prompt.version.to_owned(),
        input_tokens: reply.input_tokens,
        output_tokens: reply.output_tokens,
    }
}

impl AiPort for Anthropic {
    fn assess(&self, input: AssessInput) -> BoxFut<'_, Result<AiOut<VerdictDraft>, AiError>> {
        Box::pin(async move {
            let reply = self
                .post(request::assess_body(&self.models.judge, &input))
                .await?;
            let value = response::parse_verdict(&reply)?;
            Ok(AiOut {
                meta: meta(&reply, &prompts::JUDGE),
                value,
            })
        })
    }

    fn character(&self, input: CharacterInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            let reply = self
                .post(request::character_body(&self.models.character, &input))
                .await?;
            Ok(AiOut {
                meta: meta(&reply, &prompts::CHARACTER),
                value: reply.text,
            })
        })
    }

    fn debrief(&self, input: DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            let reply = self
                .post(request::debrief_body(&self.models.mentor, &input))
                .await?;
            Ok(AiOut {
                meta: meta(&reply, &prompts::MENTOR),
                value: reply.text,
            })
        })
    }

    fn season_debrief(
        &self,
        input: SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            let reply = self
                .post(request::season_debrief_body(&self.models.mentor, &input))
                .await?;
            Ok(AiOut {
                meta: meta(&reply, &prompts::MENTOR_SEASON),
                value: reply.text,
            })
        })
    }
}
