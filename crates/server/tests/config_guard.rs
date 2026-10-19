//! Pilot-2 r2 (I4, M1): produkcija ne smije krenuti bez vrata zbog prazne zastavice; lozinka s razmakom na rubu.
use std::collections::HashMap;

use server::config::{ConfigError, Settings};

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let m: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |k| m.get(k).cloned()
}

#[test]
fn non_local_address_without_password_is_refused_whatever_the_production_flag() {
    for flag in ["", "0", "false", "1"] {
        let r = Settings::from_lookup(lookup(&[
            ("HOTELIXSIM_PRODUKCIJA", flag),
            ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
            ("DATABASE_URL", "postgres://x"),
            ("HOTELIXSIM_WEB", "/app/web"),
        ]));
        assert!(r.is_err(), "zastavica {flag:?}: start mora biti odbijen");
    }
    match Settings::from_lookup(lookup(&[("HOTELIXSIM_ADDR", "0.0.0.0:8080")])) {
        Err(e @ ConfigError::OpenWithoutPassword { .. }) => {
            assert!(e.to_string().contains("HOTELIXSIM_LOZINKA"), "{e}");
        }
        other => panic!("očekivano OpenWithoutPassword, dobiveno {other:?}"),
    }
}

#[test]
fn non_local_address_is_fine_with_a_password_and_local_ones_need_none() {
    assert!(
        Settings::from_lookup(lookup(&[
            ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
            ("HOTELIXSIM_LOZINKA", "tajna123"),
        ]))
        .is_ok()
    );
    for addr in [
        "127.0.0.1:8080",
        "[::1]:8080",
        "localhost:8080",
        "127.0.0.2:1",
    ] {
        assert!(
            Settings::from_lookup(lookup(&[("HOTELIXSIM_ADDR", addr)])).is_ok(),
            "{addr}"
        );
    }
    for addr in [
        "0.0.0.0:8080",
        "192.168.1.5:8080",
        "[::]:8080",
        "example.com:80",
    ] {
        assert!(
            Settings::from_lookup(lookup(&[("HOTELIXSIM_ADDR", addr)])).is_err(),
            "{addr}"
        );
    }
}

#[test]
fn password_with_edge_whitespace_is_invalid_and_never_echoed() {
    for pw in ["tajna123 ", " tajna123", "tajna123\n"] {
        match Settings::from_lookup(lookup(&[("HOTELIXSIM_LOZINKA", pw)])) {
            Err(
                e @ ConfigError::Invalid {
                    var: "HOTELIXSIM_LOZINKA",
                    ..
                },
            ) => {
                let text = e.to_string();
                assert!(text.contains("HOTELIXSIM_LOZINKA"), "{text}");
                assert!(!text.contains("tajna123"), "lozinka u poruci: {text}");
            }
            other => panic!("{pw:?}: očekivano Invalid, dobiveno {other:?}"),
        }
    }
}
