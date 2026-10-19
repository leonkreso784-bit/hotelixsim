//! Pilot, dio 3: stropovi AI-ja iz postavki. Imena po uzoru na dio 2 (`HOTELIXSIM_*`, hrvatski nazivi):
//! `HOTELIXSIM_AI_STROP_DAN_USD` i `HOTELIXSIM_AI_STROP_MJESEC_USD` (decimalni broj s točkom, dolari).
//!
//! PRETPOSTAVKE O SUČELJU: `Settings` dobiva `ai_cap_day_usd: Option<f64>`, `ai_cap_month_usd: Option<f64>` i
//! `anthropic_api_key: Option<String>` (ključ iz `ANTHROPIC_API_KEY`; u `Debug`-u se nikad ne ispisuje).
//! U produkciji su oba stropa obavezna (i bez ključa: zaboravljen strop ne smije proći tiho); lokalno bez stropa = kao danas.
use std::collections::HashMap;

use server::config::{ConfigError, Settings};

const DAY: &str = "HOTELIXSIM_AI_STROP_DAN_USD";
const MONTH: &str = "HOTELIXSIM_AI_STROP_MJESEC_USD";

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let m: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |k| m.get(k).cloned()
}

const PRODUCTION: [(&str, &str); 5] = [
    ("HOTELIXSIM_PRODUKCIJA", "1"),
    (
        "DATABASE_URL",
        "postgres://u:p@baza.railway.internal:5432/railway",
    ),
    ("HOTELIXSIM_LOZINKA", "tajna123"),
    ("HOTELIXSIM_WEB", "/app/web"),
    ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
];

fn production_with(extra: &[(&'static str, &'static str)]) -> Result<Settings, ConfigError> {
    let mut pairs = PRODUCTION.to_vec();
    pairs.extend_from_slice(extra);
    Settings::from_lookup(lookup(&pairs))
}

#[test]
fn local_mode_without_caps_is_as_today() {
    let s = Settings::from_lookup(lookup(&[])).unwrap();
    assert_eq!(s.ai_cap_day_usd, None);
    assert_eq!(s.ai_cap_month_usd, None);
    assert_eq!(s.anthropic_api_key, None);
}

#[test]
fn caps_are_read_as_decimal_dollars() {
    let s = Settings::from_lookup(lookup(&[(DAY, "5"), (MONTH, "30.5")])).unwrap();
    assert_eq!(s.ai_cap_day_usd, Some(5.0));
    assert_eq!(s.ai_cap_month_usd, Some(30.5));
    let s = Settings::from_lookup(lookup(&[(DAY, " 0.25 ")])).unwrap();
    assert_eq!(s.ai_cap_day_usd, Some(0.25));
    assert_eq!(
        s.ai_cap_month_usd, None,
        "svaki strop je zasebno neobavezan lokalno"
    );
}

#[test]
fn blank_cap_counts_as_not_set() {
    let s = Settings::from_lookup(lookup(&[(DAY, "  "), (MONTH, "")])).unwrap();
    assert_eq!((s.ai_cap_day_usd, s.ai_cap_month_usd), (None, None));
}

#[test]
fn production_with_both_caps_is_accepted() {
    let s = production_with(&[(DAY, "5"), (MONTH, "30")]).unwrap();
    assert_eq!(s.ai_cap_day_usd, Some(5.0));
    assert_eq!(s.ai_cap_month_usd, Some(30.0));
}

#[test]
fn production_refuses_start_without_each_cap() {
    for (given, gone) in [((DAY, "5"), MONTH), ((MONTH, "30"), DAY)] {
        match production_with(&[given]) {
            Err(ConfigError::Missing(v)) => assert_eq!(v, vec![gone]),
            other => panic!("očekivano Missing([{gone}]), dobiveno {other:?}"),
        }
    }
    match production_with(&[]) {
        Err(ConfigError::Missing(mut v)) => {
            v.sort_unstable();
            assert_eq!(v, vec![DAY, MONTH]);
        }
        other => panic!("očekivano Missing, dobiveno {other:?}"),
    }
    // prazna vrijednost = nedostaje
    match production_with(&[(DAY, " "), (MONTH, "30")]) {
        Err(ConfigError::Missing(v)) => assert_eq!(v, vec![DAY]),
        other => panic!("očekivano Missing, dobiveno {other:?}"),
    }
}

#[test]
fn invalid_cap_values_are_refused_in_both_modes() {
    for bad in ["abc", "-1", "NaN", "inf", "5,5", "1e400", "$5", "5 USD"] {
        for var in [DAY, MONTH] {
            match Settings::from_lookup(lookup(&[(var, bad)])) {
                Err(ConfigError::Invalid { var: v, .. }) => assert_eq!(v, var, "{bad}"),
                other => panic!("{var}={bad:?}: očekivano Invalid, dobiveno {other:?}"),
            }
            let other: &'static str = if var == DAY { MONTH } else { DAY };
            match production_with(&[(var, bad), (other, "30")]) {
                Err(ConfigError::Invalid { var: v, .. }) => assert_eq!(v, var, "{bad}"),
                other => panic!("produkcija {var}={bad:?}: očekivano Invalid, dobiveno {other:?}"),
            }
        }
    }
}

#[test]
fn invalid_cap_error_text_names_the_variable() {
    let e = Settings::from_lookup(lookup(&[(MONTH, "puno")])).unwrap_err();
    assert!(e.to_string().contains(MONTH), "{e}");
}

#[test]
fn api_key_is_read_but_never_printed() {
    let s = Settings::from_lookup(lookup(&[("ANTHROPIC_API_KEY", "sk-ant-tajni-kljuc")])).unwrap();
    assert_eq!(s.anthropic_api_key.as_deref(), Some("sk-ant-tajni-kljuc"));
    let text = format!("{s:?}");
    assert!(
        !text.contains("sk-ant-tajni-kljuc"),
        "ključ u Debug ispisu: {text}"
    );
    let blank = Settings::from_lookup(lookup(&[("ANTHROPIC_API_KEY", "  ")])).unwrap();
    assert_eq!(blank.anthropic_api_key, None);
}

#[test]
fn caps_may_be_printed_in_debug() {
    let s = production_with(&[(DAY, "5"), (MONTH, "30")]).unwrap();
    let text = format!("{s:?}");
    assert!(text.contains("ai_cap_day_usd"), "{text}");
}
