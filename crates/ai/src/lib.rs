//! Klijent za Claude Messages API (sirovi HTTP: za Rust nema službenog SDK-a) iza `app::AiPort`.
//! Ključ je samo ovdje i nikad se ne logira ni ne ispisuje.

pub mod benchmark;
pub mod prompts;
pub mod request;
pub mod response;

use app::{AiError, AiOut, AiPort, BoxFut};
use game::{AiMeta, AssessInput, CharacterInput, DebriefInput, VerdictDraft};
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
            judge: "claude-opus-5-5".into(),
            character: "claude-sonnet-5-5".into(),
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

pub struct Anthropic {
    http: reqwest::Client,
    key: String,
    models: Models,
    base_url: String,
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
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("reqwest klijent se mora izgraditi");
        Anthropic {
            http,
            key,
            models,
            base_url: "https://api.anthropic.com".into(),
        }
    }

    async fn post(&self, body: serde_json::Value) -> Result<Reply, AiError> {
        let resp = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.key)
            .header("anthropic-version", request::API_VERSION)
            .header("anthropic-beta", request::FALLBACK_BETA)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("mreža: {}", e.without_url())))?;
        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| AiError::Unavailable(format!("čitanje odgovora: {}", e.without_url())))?;
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
}
