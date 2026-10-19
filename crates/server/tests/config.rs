//! Tipizirane postavke (pilot, dio 2 e): jedno mjesto čita varijable okoline u strukturu.
//! Čita se iz IZVORA varijabli (funkcija), ne iz stvarne okoline procesa, pa su testovi neovisni o računalu.
use std::collections::HashMap;
use std::path::PathBuf;

use server::config::{ConfigError, Settings};

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let m: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |k| m.get(k).cloned()
}

fn missing(r: Result<Settings, ConfigError>) -> Vec<&'static str> {
    match r {
        Err(ConfigError::Missing(v)) => v,
        other => panic!("očekivano Missing, dobiveno {other:?}"),
    }
}

const FULL_PRODUCTION: [(&str, &str); 5] = [
    ("HOTELIXSIM_PRODUKCIJA", "1"),
    (
        "DATABASE_URL",
        "postgres://u:p@baza.railway.internal:5432/railway",
    ),
    ("HOTELIXSIM_LOZINKA", "tajna123"),
    ("HOTELIXSIM_WEB", "/app/web"),
    ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
];

#[test]
fn local_mode_needs_nothing_and_has_todays_defaults() {
    let s = Settings::from_lookup(lookup(&[])).expect("lokalno sve neobavezno");
    assert!(!s.production);
    assert_eq!(s.addr, "127.0.0.1:8080");
    assert_eq!(s.content_dir, None);
    assert_eq!(s.database_url, None);
    assert_eq!(s.site_password, None);
    assert_eq!(s.web_dir, None);
}

#[test]
fn local_mode_reads_what_is_set() {
    let s = Settings::from_lookup(lookup(&[
        ("HOTELIXSIM_ADDR", "127.0.0.1:9999"),
        ("HOTELIXSIM_CONTENT", "/x/content/hotel"),
        ("DATABASE_URL", "postgres://lokalno"),
        ("HOTELIXSIM_LOZINKA", "tajna123"),
        ("HOTELIXSIM_WEB", "/x/dist"),
    ]))
    .unwrap();
    assert!(!s.production);
    assert_eq!(s.addr, "127.0.0.1:9999");
    assert_eq!(s.content_dir, Some(PathBuf::from("/x/content/hotel")));
    assert_eq!(s.database_url.as_deref(), Some("postgres://lokalno"));
    assert_eq!(s.site_password.as_deref(), Some("tajna123"));
    assert_eq!(s.web_dir, Some(PathBuf::from("/x/dist")));
}

#[test]
fn blank_values_count_as_not_set() {
    let s = Settings::from_lookup(lookup(&[
        ("HOTELIXSIM_LOZINKA", "  "),
        ("HOTELIXSIM_WEB", ""),
        ("HOTELIXSIM_ADDR", ""),
        ("HOTELIXSIM_PRODUKCIJA", ""),
    ]))
    .unwrap();
    assert!(!s.production);
    assert_eq!(s.site_password, None);
    assert_eq!(s.web_dir, None);
    assert_eq!(s.addr, "127.0.0.1:8080", "prazna adresa = zadana");
}

#[test]
fn full_production_is_accepted() {
    let s = Settings::from_lookup(lookup(&FULL_PRODUCTION)).unwrap();
    assert!(s.production);
    assert_eq!(s.addr, "0.0.0.0:8080");
    assert_eq!(s.web_dir, Some(PathBuf::from("/app/web")));
}

#[test]
fn production_refuses_start_for_each_missing_required_variable() {
    for gone in ["HOTELIXSIM_LOZINKA", "HOTELIXSIM_WEB", "DATABASE_URL"] {
        let pairs: Vec<(&str, &str)> = FULL_PRODUCTION
            .iter()
            .copied()
            .filter(|(k, _)| *k != gone)
            .collect();
        assert_eq!(missing(Settings::from_lookup(lookup(&pairs))), vec![gone]);
    }
}

#[test]
fn production_treats_blank_required_as_missing_and_reports_all_at_once() {
    let r = Settings::from_lookup(lookup(&[
        ("HOTELIXSIM_PRODUKCIJA", "1"),
        ("HOTELIXSIM_LOZINKA", "   "),
    ]));
    let mut v = missing(r);
    v.sort_unstable();
    assert_eq!(
        v,
        vec!["DATABASE_URL", "HOTELIXSIM_LOZINKA", "HOTELIXSIM_WEB"]
    );
}

#[test]
fn error_text_names_the_variable() {
    let e = Settings::from_lookup(lookup(&[("HOTELIXSIM_PRODUKCIJA", "1")])).unwrap_err();
    let text = e.to_string();
    for var in ["DATABASE_URL", "HOTELIXSIM_LOZINKA", "HOTELIXSIM_WEB"] {
        assert!(text.contains(var), "{text}");
    }
}

#[test]
fn production_flag_accepts_one_and_true_and_rejects_nonsense() {
    for on in ["1", "true", "TRUE"] {
        let mut pairs = FULL_PRODUCTION.to_vec();
        pairs[0] = ("HOTELIXSIM_PRODUKCIJA", on);
        assert!(
            Settings::from_lookup(lookup(&pairs)).unwrap().production,
            "{on}"
        );
    }
    for off in ["0", "false"] {
        let s = Settings::from_lookup(lookup(&[("HOTELIXSIM_PRODUKCIJA", off)])).unwrap();
        assert!(!s.production, "{off}");
    }
    // Zabuna u slovu ne smije tiho isključiti zaštitu.
    match Settings::from_lookup(lookup(&[("HOTELIXSIM_PRODUKCIJA", "da")])) {
        Err(ConfigError::Invalid { var, .. }) => assert_eq!(var, "HOTELIXSIM_PRODUKCIJA"),
        other => panic!("očekivano Invalid, dobiveno {other:?}"),
    }
}

#[test]
fn production_turns_on_secure_cookie_and_passes_gate_and_web_to_router() {
    let s = Settings::from_lookup(lookup(&FULL_PRODUCTION)).unwrap();
    let c = s.router_config();
    assert!(c.secure_cookie, "produkcija = Secure kolačić");
    assert_eq!(c.site_password.as_deref(), Some("tajna123"));
    assert_eq!(c.web_dir, Some(PathBuf::from("/app/web")));
}

#[test]
fn local_router_config_has_no_secure_cookie_no_gate_no_web() {
    let c = Settings::from_lookup(lookup(&[])).unwrap().router_config();
    assert!(!c.secure_cookie);
    assert_eq!(c.site_password, None);
    assert_eq!(c.web_dir, None);
}

#[test]
fn settings_debug_never_prints_the_password() {
    let s = Settings::from_lookup(lookup(&FULL_PRODUCTION)).unwrap();
    let text = format!("{s:?}");
    assert!(!text.contains("tajna123"), "lozinka u Debug ispisu: {text}");
}

// --- Pilot, dio 4: najviše igara po gostu (ADR-062 t. 9) ---
// 🟨 odluka čuvara: u produkciji zadano 3; lokalno bez granice (None) dok varijabla nije postavljena, da razvoj i
// automatski testovi ne udaraju o granicu. Vrijednost mora biti cijeli broj ≥ 1; sve ostalo (i 0) je Invalid.

#[test]
fn games_per_guest_defaults_to_three_in_production_and_unlimited_locally() {
    let prod = Settings::from_lookup(lookup(&FULL_PRODUCTION)).unwrap();
    assert_eq!(prod.games_per_guest, Some(3));
    let local = Settings::from_lookup(lookup(&[])).unwrap();
    assert_eq!(local.games_per_guest, None);
}

#[test]
fn games_per_guest_is_read_from_the_variable_in_both_modes() {
    let mut pairs = FULL_PRODUCTION.to_vec();
    pairs.push(("HOTELIXSIM_IGARA_PO_GOSTU", "5"));
    assert_eq!(
        Settings::from_lookup(lookup(&pairs))
            .unwrap()
            .games_per_guest,
        Some(5)
    );
    let local = Settings::from_lookup(lookup(&[("HOTELIXSIM_IGARA_PO_GOSTU", "2")])).unwrap();
    assert_eq!(local.games_per_guest, Some(2));
    let blank = Settings::from_lookup(lookup(&[("HOTELIXSIM_IGARA_PO_GOSTU", "  ")])).unwrap();
    assert_eq!(blank.games_per_guest, None, "prazno = nepostavljeno");
}

#[test]
fn games_per_guest_rejects_nonsense_naming_the_variable() {
    for bad in ["0", "-1", "tri", "3.5", "3 igre"] {
        match Settings::from_lookup(lookup(&[("HOTELIXSIM_IGARA_PO_GOSTU", bad)])) {
            Err(ConfigError::Invalid { var, value }) => {
                assert_eq!(var, "HOTELIXSIM_IGARA_PO_GOSTU", "{bad}");
                assert_eq!(value, bad);
            }
            other => panic!("{bad}: očekivano Invalid, dobiveno {other:?}"),
        }
    }
    let e = Settings::from_lookup(lookup(&[("HOTELIXSIM_IGARA_PO_GOSTU", "tri")])).unwrap_err();
    assert!(e.to_string().contains("HOTELIXSIM_IGARA_PO_GOSTU"), "{e}");
}
