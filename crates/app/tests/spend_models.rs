//! Pilot-3 runda 2 (I2): tablica cijena pokriva sve trenutne modele; nepoznat model je po komponentama najskuplji
//! iz CIJELE tablice, pa skuplji model od Sonneta nikad nije podcijenjen. Najdulji prefiks (Sonnet 5 / Sonnet 5.5).

use app::spend::{PRICES, TokenUsage, cost_nano_usd, is_priced};

fn t(input: u32, output: u32, cache_write: u32, cache_read: u32) -> TokenUsage {
    TokenUsage {
        input,
        output,
        cache_write,
        cache_read,
    }
}

fn u() -> TokenUsage {
    t(1_000, 500, 10_000, 20_000)
}

/// ulaz, izlaz, upis, čitanje (nano-$ po tokenu) pomnoženi s u()
fn expected(input: u64, output: u64, cache_write: u64, cache_read: u64) -> u64 {
    1_000 * input + 500 * output + 10_000 * cache_write + 20_000 * cache_read
}

#[test]
fn current_models_are_priced_from_the_published_table() {
    let cost = |m: &str| cost_nano_usd(m, &u());
    assert_eq!(cost("claude-opus-5-5"), expected(4_000, 20_000, 5_000, 200));
    assert_eq!(
        cost("claude-fable-5-1"),
        expected(10_000, 50_000, 12_500, 250)
    );
    assert_eq!(
        cost("claude-fable-5"),
        expected(10_000, 50_000, 12_500, 1_000)
    );
    assert_eq!(cost("claude-opus-5"), expected(5_000, 25_000, 6_250, 500));
    assert_eq!(cost("claude-haiku-4-5"), expected(1_000, 5_000, 1_250, 100));
}

#[test]
fn sonnet_5_fallback_is_not_swallowed_by_sonnet_5_5_nor_the_other_way() {
    let read = t(0, 0, 0, 1_000_000);
    assert_eq!(cost_nano_usd("claude-sonnet-5", &read), 200_000_000);
    assert_eq!(cost_nano_usd("claude-sonnet-5-5", &read), 100_000_000);
    assert_eq!(
        cost_nano_usd("claude-sonnet-5-20260101", &read),
        200_000_000
    );
    assert_eq!(
        cost_nano_usd("claude-sonnet-5-5-20261001", &read),
        100_000_000
    );
}

#[test]
fn unknown_model_is_never_cheaper_than_any_priced_model_per_component() {
    for (label, only) in [
        ("ulaz", t(1_000_000, 0, 0, 0)),
        ("izlaz", t(0, 1_000_000, 0, 0)),
        ("upis", t(0, 0, 1_000_000, 0)),
        ("čitanje", t(0, 0, 0, 1_000_000)),
    ] {
        let unknown = cost_nano_usd("claude-iz-buducnosti-9", &only);
        for p in PRICES {
            assert!(
                unknown >= cost_nano_usd(p.model_prefix, &only),
                "{label} {}",
                p.model_prefix
            );
        }
    }
}

#[test]
fn opus_is_not_underestimated_and_unknown_is_fable_level() {
    let opus = cost_nano_usd("claude-opus-5-5", &u());
    let sonnet = cost_nano_usd("claude-sonnet-5-5", &u());
    assert!(opus > sonnet, "Opus mora koštati više od Sonneta");
    let unknown = cost_nano_usd("neki-novi-model", &u());
    assert!(unknown >= cost_nano_usd("claude-fable-5-1", &u()));
    assert_eq!(unknown, expected(10_000, 50_000, 12_500, 1_000));
}

#[test]
fn is_priced_knows_table_models_only() {
    for m in [
        "claude-sonnet-5-5",
        "claude-haiku-5-5",
        "claude-opus-5-5",
        "claude-sonnet-5",
        "claude-fable-5-1",
    ] {
        assert!(is_priced(m), "{m}");
    }
    for m in ["", "gpt-x", "claude-opus-9", "sonnet"] {
        assert!(!is_priced(m), "{m:?}");
    }
}
