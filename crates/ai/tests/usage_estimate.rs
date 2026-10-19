//! Pilot-3 runda 2 (I3): istek vremena i prekid zahtjeva u letu upisuju se kao PROCJENA najgoreg slučaja
//! (`max_tokens` x cijena izlaza modela, ulaz 0); zahtjev koji nikad nije stigao se ne broji.

use std::io::Read;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ai::{Anthropic, Models};
use app::spend::{ApiUsage, TokenUsage, UsageSink, cost_nano_usd};
use app::{AiError, AiPort, BoxFut};
use game::{CharacterCard, CharacterInput, ChatLine, Speaker};

#[derive(Default)]
struct Collect(Mutex<Vec<ApiUsage>>);

impl UsageSink for Collect {
    fn record(&self, usage: ApiUsage) -> BoxFut<'_, ()> {
        self.0.lock().unwrap().push(usage);
        Box::pin(async {})
    }
}

/// Primi zahtjev, pročitaj ga i šuti `secs` sekundi (odgovor nikad ne stigne).
fn hang(secs: u64) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = listener.accept() {
            let mut buf = [0u8; 8192];
            let _ = s.read(&mut buf);
            std::thread::sleep(Duration::from_secs(secs));
        }
    });
    format!("http://{addr}")
}

fn input() -> CharacterInput {
    CharacterInput {
        situation: "S1".into(),
        title: "Gost".into(),
        character: CharacterCard {
            name: "Horvat".into(),
            role: "gost".into(),
            brief: "Ljuta.".into(),
            opening: "Dobar dan.".into(),
        },
        transcript: vec![ChatLine {
            from: Speaker::Student,
            text: "Dobar dan.".into(),
        }],
        message_no: 1,
        message_max: 6,
    }
}

async fn wait_for(sink: &Collect, n: usize) {
    for _ in 0..100 {
        if sink.0.lock().unwrap().len() >= n {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn expected_estimate() -> ApiUsage {
    let m = Models::default().character;
    let max_tokens = ai::request::character_body(&m, &input())["max_tokens"]
        .as_u64()
        .unwrap() as u32;
    ApiUsage {
        model: m,
        tokens: TokenUsage {
            output: max_tokens,
            ..Default::default()
        },
    }
}

#[tokio::test]
async fn timeout_is_recorded_as_worst_case_estimate() {
    let sink = Arc::new(Collect::default());
    let ai = Anthropic::new("test-kljuc".into(), Models::default())
        .with_base_url(hang(5))
        .with_timeout(Duration::from_millis(300))
        .with_usage_sink(sink.clone());
    let r = ai.character(input()).await;
    assert!(matches!(r, Err(AiError::Unavailable(_))), "{r:?}");
    wait_for(&sink, 1).await;
    let got = sink.0.lock().unwrap().clone();
    assert_eq!(got, vec![expected_estimate()]);
    assert!(got[0].tokens.output > 0);
    assert!(cost_nano_usd(&got[0].model, &got[0].tokens) > 0);
}

#[tokio::test]
async fn cancelled_request_is_recorded_as_worst_case_estimate() {
    let sink = Arc::new(Collect::default());
    let ai = Anthropic::new("test-kljuc".into(), Models::default())
        .with_base_url(hang(5))
        .with_usage_sink(sink.clone());
    // prekid pri gašenju: budućnost se odbacuje usred čekanja
    let r = tokio::time::timeout(Duration::from_millis(300), ai.character(input())).await;
    assert!(r.is_err(), "poziv je trebao ostati u letu");
    wait_for(&sink, 1).await;
    assert_eq!(sink.0.lock().unwrap().clone(), vec![expected_estimate()]);
}

#[tokio::test]
async fn connection_refused_is_not_counted() {
    let sink = Arc::new(Collect::default());
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let ai = Anthropic::new("test-kljuc".into(), Models::default())
        .with_base_url(format!("http://127.0.0.1:{port}"))
        .with_usage_sink(sink.clone());
    let r = ai.character(input()).await;
    assert!(matches!(r, Err(AiError::Unavailable(_))), "{r:?}");
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(sink.0.lock().unwrap().is_empty());
}
