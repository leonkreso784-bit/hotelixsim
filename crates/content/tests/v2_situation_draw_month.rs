//! Mjesečni krug (ADR-042 t. 12–13; mjesečno 4/5/6 situacija): izvlači se kao jedan krug, ne kao niz virtualnih
//! tjedana. Isti predložak dolazi najviše jednom u krugu, a iz kruga u krug (razmak 30 dana ≥ 28) smije se ponoviti,
//! s drugim imenom gosta.

mod situations_common;

use std::collections::{BTreeMap, BTreeSet};

use sim_hotel_v2::situations::{DrawnSituation, NumberParam, Repeat, TemplateSet, situation_count};
use sim_hotel_v2::{PeriodKind, Profile, season_periods};
use situations_common::{Hotel, frozen, indices, season, start, with_general};

/// Samo testni opći predlošci (bez uvjeta), dovoljno ih za najveći krug (6) uz razmak od 4 tjedna.
fn month_set() -> TemplateSet {
    let (_, mut set) = frozen();
    set.templates.clear();
    let mut set = with_general(
        set,
        24,
        Repeat {
            max_per_season: 8,
            gap_rounds: 1,
        },
    );
    for t in &mut set.templates {
        t.numbers
            .insert("broj".to_string(), NumberParam::Range(1, 1_000_000));
    }
    set
}

/// Mjesečni krug je jedan krug (ne niz virtualnih tjedana; ADR-042 t. 13): točno svoj broj situacija, svaki predložak
/// najviše jednom u krugu, svi primjerci imaju dan početka kruga, a iz kruga u krug smije se ponoviti (t. 12).
#[test]
fn monthly_round_draws_a_template_at_most_once() {
    let (cat, _) = frozen();
    let set = month_set();
    let mut consecutive = 0;
    for profile in [Profile::A, Profile::B, Profile::C] {
        let state = start(&cat, profile);
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        for seed in 0..300 {
            let rounds = season(&cat, &set, &h, PeriodKind::Month, seed);
            let mut codes: BTreeSet<&str> = BTreeSet::new();
            let mut earlier: BTreeMap<&str, &DrawnSituation> = BTreeMap::new();
            let mut previous: BTreeSet<&str> = BTreeSet::new();
            for (p, drawn) in season_periods(PeriodKind::Month).iter().zip(&rounds) {
                assert_eq!(
                    drawn.len(),
                    usize::from(situation_count(&state, p, &cat).unwrap()),
                    "{} (sjeme {seed})",
                    p.code()
                );
                let mut templates: BTreeSet<&str> = BTreeSet::new();
                for d in drawn {
                    assert!(
                        templates.insert(&d.template),
                        "{}: predložak {} dvaput u krugu (sjeme {seed})",
                        p.code(),
                        d.template
                    );
                    assert_eq!(d.day, p.start_day, "{}: dan početka kruga", d.code);
                    assert_eq!(d.occurrence, 1, "{}", d.code);
                    assert_eq!(d.code, format!("{}.{}", d.template, p.code()));
                    assert!(codes.insert(&d.code), "šifra {} dvaput", d.code);
                    if previous.contains(d.template.as_str()) {
                        consecutive += 1;
                    }
                    if let Some(e) = earlier.get(d.template.as_str()) {
                        assert_ne!(d.roles["gost"].name, e.roles["gost"].name, "{}", d.code);
                    }
                }
                previous = templates;
                for d in drawn {
                    earlier.insert(&d.template, d);
                }
            }
        }
    }
    assert!(
        consecutive > 0,
        "predložak smije doći dva mjeseca zaredom (ADR-042 t. 12) i to se događa"
    );
}

#[test]
fn monthly_draw_is_reproducible() {
    let (cat, _) = frozen();
    let set = month_set();
    let state = start(&cat, Profile::A);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 3,
    };
    for seed in [0, 1, 42] {
        let a = season(&cat, &set, &h, PeriodKind::Month, seed);
        assert_eq!(a, season(&cat, &set, &h, PeriodKind::Month, seed));
        assert_ne!(a, season(&cat, &set, &h, PeriodKind::Month, seed + 1000));
    }
}
