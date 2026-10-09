//! Čitanje odgovora Messages API-ja; sve što nije uredan tekst je `AiError`.

use app::AiError;
use game::VerdictDraft;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub text: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Deserialize)]
struct Block {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct StopDetails {
    category: Option<String>,
}

#[derive(Deserialize, Default)]
struct Usage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

#[derive(Deserialize)]
struct Body {
    #[serde(default)]
    model: String,
    #[serde(default)]
    content: Vec<Block>,
    stop_reason: Option<String>,
    stop_details: Option<StopDetails>,
    #[serde(default)]
    usage: Usage,
}

pub fn parse_reply(status: u16, body: &str) -> Result<Reply, AiError> {
    if !(200..300).contains(&status) {
        let head: String = body.chars().take(300).collect();
        return Err(AiError::Unavailable(format!("HTTP {status}: {head}")));
    }
    let b: Body = serde_json::from_str(body)
        .map_err(|e| AiError::BadOutput(format!("odgovor nije JSON: {e}")))?;
    match b.stop_reason.as_deref() {
        Some("refusal") => {
            let cat = b
                .stop_details
                .and_then(|d| d.category)
                .unwrap_or_else(|| "bez kategorije".into());
            return Err(AiError::Refused(cat));
        }
        Some("max_tokens") => return Err(AiError::BadOutput("odgovor odrezan".into())),
        _ => {}
    }
    let text: String = b
        .content
        .iter()
        .filter(|c| c.kind == "text")
        .filter_map(|c| c.text.as_deref())
        .collect();
    if text.trim().is_empty() {
        return Err(AiError::BadOutput("odgovor nema teksta".into()));
    }
    Ok(Reply {
        text,
        model: b.model,
        input_tokens: b.usage.input_tokens,
        output_tokens: b.usage.output_tokens,
    })
}

pub fn parse_verdict(reply: &Reply) -> Result<VerdictDraft, AiError> {
    serde_json::from_str::<VerdictDraft>(&reply.text)
        .map_err(|e| AiError::BadOutput(format!("presuda nije u očekivanom obliku: {e}")))
}
