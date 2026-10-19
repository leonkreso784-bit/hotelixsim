//! Granica igara po gostu (pilot, dio 4): zaboravljena zastavica HOTELIXSIM_PRODUKCIJA ne smije isključiti zaštitu.
use std::collections::HashMap;

use server::config::Settings;

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let m: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |k| m.get(k).cloned()
}

#[test]
fn non_local_addr_without_production_flag_defaults_to_three_games() {
    let s = Settings::from_lookup(lookup(&[
        ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
        ("HOTELIXSIM_LOZINKA", "tajna123"),
    ]))
    .unwrap();
    assert!(!s.production);
    assert_eq!(s.games_per_guest, Some(3));
}

#[test]
fn non_local_addr_respects_explicit_value() {
    let s = Settings::from_lookup(lookup(&[
        ("HOTELIXSIM_ADDR", "0.0.0.0:8080"),
        ("HOTELIXSIM_LOZINKA", "tajna123"),
        ("HOTELIXSIM_IGARA_PO_GOSTU", "5"),
    ]))
    .unwrap();
    assert_eq!(s.games_per_guest, Some(5));
}

#[test]
fn local_addr_without_production_has_no_limit() {
    let s = Settings::from_lookup(lookup(&[("HOTELIXSIM_ADDR", "127.0.0.1:9000")])).unwrap();
    assert_eq!(s.games_per_guest, None);
}
