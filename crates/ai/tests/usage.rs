//! Pilot, dio 3: brojač troška vidi SVAKI odgovor Anthropica, i odrezan (`max_tokens`) i neparsabilan, i upis/čitanje
//! predmemorije upita. Greške `AiError` ne nose potrošnju, pa brojač sjedi ISPOD parsiranja: klijent javlja `UsageSink`-u
//! potrošnju čim stigne tijelo odgovora 2xx, prije nego što se odgovor tumači.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti uz obrazloženje):
//! - `ai::response::parse_usage(body: &str) -> Option<app::spend::ApiUsage>`: `Some` za svako tijelo koje je JSON s poljem
//!   `usage` (bez obzira na `stop_reason`: `end_turn`, `max_tokens`, `refusal`); `None` za ne-JSON i tijelo bez `usage`.
//!   Model je iz tijela (`""` ako ga nema; `Anthropic` tada koristi model iz zahtjeva). Polja `usage`:
//!   `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens` (sva neobavezna, zadano 0).
//! - `Reply` dobiva `cache_creation_input_tokens: u32` i `cache_read_input_tokens: u32`.
//! - `Anthropic::with_base_url(self, url: String) -> Anthropic` (probni poslužitelj) i
//!   `Anthropic::with_usage_sink(self, Arc<dyn app::spend::UsageSink>) -> Anthropic`.
//! - HTTP greške (ne-2xx) nemaju potrošnju: sink se ne poziva.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use ai::{Anthropic, Models};
use app::spend::{ApiUsage, TokenUsage, UsageSink};
use app::{AiError, AiPort, BoxFut};
use game::{CharacterCard, CharacterInput, ChatLine, Speaker};

const FULL: &str = r#"{"model":"claude-sonnet-5-5","content":[{"type":"text","text":"Hvala."}],"stop_reason":"end_turn","usage":{"input_tokens":900,"output_tokens":210,"cache_creation_input_tokens":4000,"cache_read_input_tokens":12000}}"#;
const CUT: &str = r#"{"model":"claude-haiku-5-5","content":[{"type":"text","text":"{\"crit"}],"stop_reason":"max_tokens","usage":{"input_tokens":50,"output_tokens":16000,"cache_creation_input_tokens":0,"cache_read_input_tokens":300}}"#;
const REFUSAL: &str = r#"{"model":"claude-sonnet-5-5","content":[],"stop_reason":"refusal","stop_details":{"type":"refusal","category":null,"explanation":"x"},"usage":{"input_tokens":7,"output_tokens":0}}"#;
const PROSE_VERDICT: &str = r#"{"model":"claude-sonnet-5-5","content":[{"type":"text","text":"Mislim da je dobro."}],"stop_reason":"end_turn","usage":{"input_tokens":1500,"output_tokens":20,"cache_read_input_tokens":8000}}"#;

fn tokens(input: u32, output: u32, cache_write: u32, cache_read: u32) -> TokenUsage {
    TokenUsage {
        input,
        output,
        cache_write,
        cache_read,
    }
}

#[test]
fn parse_usage_reads_all_four_counters() {
    let u = ai::response::parse_usage(FULL).expect("usage");
    assert_eq!(u.model, "claude-sonnet-5-5");
    assert_eq!(u.tokens, tokens(900, 210, 4000, 12000));
}

#[test]
fn parse_usage_counts_truncated_refused_and_missing_cache_fields() {
    let cut = ai::response::parse_usage(CUT).expect("odrezan odgovor ima potrošnju");
    assert_eq!(cut.model, "claude-haiku-5-5");
    assert_eq!(cut.tokens, tokens(50, 16000, 0, 300));
    let refusal = ai::response::parse_usage(REFUSAL).expect("odbijanje ima potrošnju");
    assert_eq!(
        refusal.tokens,
        tokens(7, 0, 0, 0),
        "polja predmemorije izostavljena = 0"
    );
}

#[test]
fn parse_usage_is_none_without_usage_or_json() {
    assert_eq!(ai::response::parse_usage("nije json"), None);
    assert_eq!(ai::response::parse_usage("{}"), None);
    assert_eq!(
        ai::response::parse_usage(r#"{"error":{"type":"overloaded_error"}}"#),
        None
    );
}

#[test]
fn reply_carries_cache_tokens_and_errors_stay_errors() {
    let r = ai::response::parse_reply(200, FULL).unwrap();
    assert_eq!((r.input_tokens, r.output_tokens), (900, 210));
    assert_eq!(
        (r.cache_creation_input_tokens, r.cache_read_input_tokens),
        (4000, 12000)
    );
    // odrezan odgovor je i dalje greška (jezgra se ne mijenja), ali se broji u `parse_usage`
    assert!(matches!(
        ai::response::parse_reply(200, CUT),
        Err(AiError::BadOutput(_))
    ));
}

// ---------- kroz stvarni klijent (probni HTTP poslužitelj na slobodnom portu) ----------

#[derive(Default)]
struct Collect(Mutex<Vec<ApiUsage>>);

impl UsageSink for Collect {
    fn record(&self, usage: ApiUsage) -> BoxFut<'_, ()> {
        self.0.lock().unwrap().push(usage);
        Box::pin(async {})
    }
}

/// Jedan odgovor pa gasi poslužitelj; vraća adresu `http://127.0.0.1:port`.
fn serve_once(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = s.read(&mut chunk).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..pos]).to_ascii_lowercase();
                let len: usize = head
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:"))
                    .map_or(0, |v| v.trim().parse().unwrap_or(0));
                if buf.len() >= pos + 4 + len {
                    break;
                }
            }
        }
        let resp = format!(
            "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        s.write_all(resp.as_bytes()).unwrap();
    });
    format!("http://{addr}")
}

fn client(url: String, sink: Arc<Collect>) -> Anthropic {
    Anthropic::new("test-kljuc-nije-pravi".into(), Models::default())
        .with_base_url(url)
        .with_usage_sink(sink)
}

fn sample_character() -> CharacterInput {
    CharacterInput {
        situation: "S1".into(),
        title: "Gost se žali na buku".into(),
        character: CharacterCard {
            name: "Gospođa Horvat".into(),
            role: "gost".into(),
            brief: "Ljuta zbog buke.".into(),
            opening: "Cijelu noć nisam oka sklopila!".into(),
        },
        transcript: vec![ChatLine {
            from: Speaker::Student,
            text: "Dobar dan.".into(),
        }],
        message_no: 1,
        message_max: 6,
    }
}

fn sample_assess() -> game::AssessInput {
    game::AssessInput {
        situation: "S1".into(),
        title: "Gost se žali na buku".into(),
        rubric: (1..=5).map(|n| format!("kriterij {n}")).collect(),
        character: None,
        offered: vec![],
        transcript: vec![],
        response: "Ispričavam se.".into(),
    }
}

#[tokio::test]
async fn successful_reply_reports_usage_including_cache_to_the_sink() {
    let sink = Arc::new(Collect::default());
    let ai = client(serve_once(200, FULL), sink.clone());
    let out = ai.character(sample_character()).await.expect("uspjeh");
    assert_eq!(out.value, "Hvala.");
    let got = sink.0.lock().unwrap().clone();
    assert_eq!(
        got,
        vec![ApiUsage {
            model: "claude-sonnet-5-5".into(),
            tokens: tokens(900, 210, 4000, 12000),
        }]
    );
}

#[tokio::test]
async fn truncated_reply_is_an_error_but_its_tokens_are_counted() {
    let sink = Arc::new(Collect::default());
    let ai = client(serve_once(200, CUT), sink.clone());
    let r = ai.character(sample_character()).await;
    assert!(matches!(r, Err(AiError::BadOutput(_))), "{r:?}");
    let got = sink.0.lock().unwrap().clone();
    assert_eq!(got.len(), 1, "odrezan odgovor mora biti brojan");
    assert_eq!(got[0].tokens, tokens(50, 16000, 0, 300));
}

#[tokio::test]
async fn unparsable_verdict_is_an_error_but_its_tokens_are_counted() {
    let sink = Arc::new(Collect::default());
    let ai = client(serve_once(200, PROSE_VERDICT), sink.clone());
    let r = ai.assess(sample_assess()).await;
    assert!(matches!(r, Err(AiError::BadOutput(_))), "{r:?}");
    let got = sink.0.lock().unwrap().clone();
    assert_eq!(
        got.len(),
        1,
        "presuda koju nije moguće pročitati ipak košta"
    );
    assert_eq!(got[0].tokens, tokens(1500, 20, 0, 8000));
}

#[tokio::test]
async fn refused_reply_is_counted_but_http_errors_are_not() {
    let sink = Arc::new(Collect::default());
    let r = client(serve_once(200, REFUSAL), sink.clone())
        .character(sample_character())
        .await;
    assert!(matches!(r, Err(AiError::Refused(_))), "{r:?}");
    assert_eq!(sink.0.lock().unwrap().len(), 1);

    let sink = Arc::new(Collect::default());
    let r = client(
        serve_once(529, r#"{"error":{"type":"overloaded_error"}}"#),
        sink.clone(),
    )
    .character(sample_character())
    .await;
    assert!(matches!(r, Err(AiError::Unavailable(_))), "{r:?}");
    assert!(
        sink.0.lock().unwrap().is_empty(),
        "HTTP greška nema potrošnju"
    );
}
