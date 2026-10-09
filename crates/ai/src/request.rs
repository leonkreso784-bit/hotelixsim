//! Tijela zahtjeva za Claude Messages API: čiste funkcije, testabilne bez mreže.
//! Ne šalju se `thinking`, `temperature`, `top_p` ni `budget_tokens` (modeli ih odbijaju s 400).

use game::{AssessInput, CharacterInput, ChatLine, DebriefInput, Speaker};
use serde_json::{Value, json};
use std::fmt::Write;

use crate::prompts;

pub const API_VERSION: &str = "2023-06-01";
pub const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

/// `<` → `‹`, `>` → `›`: studentov tekst ne može zatvoriti oznaku u kojoj stoji.
pub fn quote_student(text: &str) -> String {
    text.replace('<', "‹").replace('>', "›")
}

fn render_transcript(lines: &[ChatLine], character: &str) -> String {
    let mut out = String::new();
    for l in lines {
        match l.from {
            Speaker::Student => {
                let _ = writeln!(out, "Student: {}", quote_student(&l.text));
            }
            Speaker::Character => {
                let _ = writeln!(out, "{character}: {}", l.text);
            }
        }
    }
    out
}

/// Korisnička poruka za presudu: situacija, rubrika, ponuđeni odgovori, razgovor i studentov odgovor.
pub fn render_assess(input: &AssessInput) -> String {
    let mut out = format!("Situacija: {}\n\nRubrika:\n", input.title);
    for (i, r) in input.rubric.iter().enumerate() {
        let _ = writeln!(out, "{}. {r}", i + 1);
    }
    out.push_str("\nPonuđeni odgovori:\n");
    for o in &input.offered {
        let met = o
            .criteria
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            out,
            "[{}] {} — ispunjeni kriteriji: {met}",
            o.index, o.label
        );
    }
    if !input.transcript.is_empty() {
        let name = input.character.as_deref().unwrap_or("Lik");
        let _ = write!(
            out,
            "\n<razgovor>\n{}</razgovor>\n",
            render_transcript(&input.transcript, name)
        );
    }
    let _ = write!(
        out,
        "\n<odgovor_studenta>\n{}\n</odgovor_studenta>\n",
        quote_student(&input.response)
    );
    out
}

pub fn assess_body(model: &str, input: &AssessInput) -> Value {
    json!({
        "model": model,
        "max_tokens": 16000,
        "system": [{ "type": "text", "text": prompts::JUDGE.text, "cache_control": { "type": "ephemeral" } }],
        "messages": [{ "role": "user", "content": render_assess(input) }],
        "output_config": { "effort": "high", "format": { "type": "json_schema", "schema": verdict_schema(input.offered.len()) } },
        "fallbacks": "default"
    })
}

/// Preduvjet (jamči pozivatelj): prijepis počinje studentovom porukom, izmjenjuje se i završava studentovom
/// porukom; uvodna rečenica lika nije u prijepisu nego u uputi.
pub fn character_body(model: &str, input: &CharacterInput) -> Value {
    let c = &input.character;
    let system = prompts::CHARACTER
        .text
        .replace("{lik}", &format!("{}, {}. {}", c.name, c.role, c.brief))
        .replace("{uvod}", &c.opening);
    let messages: Vec<Value> = input
        .transcript
        .iter()
        .map(|l| match l.from {
            Speaker::Student => json!({
                "role": "user",
                "content": format!("<poruka_studenta>{}</poruka_studenta>", quote_student(&l.text))
            }),
            Speaker::Character => json!({ "role": "assistant", "content": l.text }),
        })
        .collect();
    json!({
        "model": model,
        "max_tokens": 2000,
        "system": system,
        "messages": messages,
        "output_config": { "effort": "low" },
        "fallbacks": "default"
    })
}

pub fn debrief_body(model: &str, input: &DebriefInput) -> Value {
    let data = serde_json::to_string_pretty(input).unwrap_or_default();
    json!({
        "model": model,
        "max_tokens": 4000,
        "system": prompts::MENTOR.text,
        "messages": [{ "role": "user", "content": data }],
        "output_config": { "effort": "low" },
        "fallbacks": "default"
    })
}

/// JSON Schema presude; zrcali `game::VerdictDraft`. `offered` = broj ponuđenih odgovora.
pub fn verdict_schema(offered: usize) -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["criteria", "nearest_answer", "rationale"],
        "properties": {
            "criteria": { "type": "array", "items": {
                "type": "object", "additionalProperties": false, "required": ["n", "met", "reason"],
                "properties": {
                    "n": { "type": "integer", "enum": [1, 2, 3, 4, 5] },
                    "met": { "type": "boolean" },
                    "reason": { "type": "string" }
                }
            }},
            "nearest_answer": { "type": "integer", "enum": (0..offered).collect::<Vec<_>>() },
            "rationale": { "type": "string" }
        }
    })
}
