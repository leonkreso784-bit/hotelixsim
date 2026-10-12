//! Brana regresije izvlačenja (J-2 popravak 1): tjedni rezultati za scenarije u kojima ponavljanje ne traži
//! novo pravilo ostaju bajt-isti kao na `52ce161` (otisci zabilježeni PRIJE izmjena). Svi razmaci u scenarijima su
//! barem 4 tjedna, pa Leonov najmanji razmak (4 tjedna) ne mijenja ništa; ne miješa se ni stanje sidara.

mod situations_common;

use sim_hotel_v2::situations::{DrawnSituation, Kind, MajorSide, Repeat, Scope, TemplateSet};
use sim_hotel_v2::{Month, PeriodKind, Profile};
use situations_common::{Hotel, content, general, indices, season, start, with_general};

/// FNV-1a 64, da otisak ne ovisi o verziji standardne knjižnice.
fn fnv(acc: u64, s: &str) -> u64 {
    s.bytes().fold(acc, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

const FNV_START: u64 = 0xcbf2_9ce4_8422_2325;

/// Opis primjerka; `roles` = s imenima i opisima likova.
fn describe(d: &DrawnSituation, roles: bool) -> String {
    let mut s = format!(
        "{}|{}|{}|{:?}|{}|{:?}|{:?}|{:?}|{:?}",
        d.code,
        d.template,
        d.period.code(),
        d.kind,
        d.weight,
        d.scope,
        d.major,
        d.department.as_ref().map(|c| c.as_str().to_string()),
        d.numbers
    );
    if roles {
        for (k, r) in &d.roles {
            s.push_str(&format!("|{k}={}/{:?}", r.name, r.role_type));
        }
    }
    s.push('\n');
    s
}

fn at_least_four_weeks(mut set: TemplateSet) -> TemplateSet {
    for t in &mut set.templates {
        t.repeat.gap_rounds = t.repeat.gap_rounds.max(4);
    }
    set
}

/// Svaki predložak najviše jednom u sezoni; uz kolektivne i veće.
fn no_repeat_set() -> TemplateSet {
    let (_, mut set) = content();
    let once = Repeat {
        max_per_season: 1,
        gap_rounds: 4,
    };
    for t in &mut set.templates {
        t.repeat = once;
    }
    let mut set = with_general(set, 24, once);
    for i in 0..2 {
        let mut svi = general(&format!("P-SVI-{i}"), Kind::Crisis, once);
        svi.scope = Scope::All;
        svi.when.months = vec![Month::Jul, Month::Aug];
        set.templates.push(svi);
    }
    for (i, side) in [
        MajorSide::High,
        MajorSide::High,
        MajorSide::Low,
        MajorSide::Low,
    ]
    .into_iter()
    .enumerate()
    {
        let mut t = general(&format!("P-VECA-{i}"), Kind::Opportunity, once);
        t.major = Some(side);
        set.templates.push(t);
    }
    set
}

/// Ponavljanje uz razmak barem 4 tjedna.
fn repeat_set() -> TemplateSet {
    let (_, set) = content();
    let set = at_least_four_weeks(set);
    let mut set = with_general(
        set,
        16,
        Repeat {
            max_per_season: 3,
            gap_rounds: 4,
        },
    );
    for i in 0..2 {
        set.templates.push(general(
            &format!("P-PRILIKA-{i}"),
            Kind::Opportunity,
            Repeat {
                max_per_season: 4,
                gap_rounds: 5,
            },
        ));
    }
    set
}

fn fingerprint(set: &TemplateSet, avg: u16, worn: bool, roles: bool) -> u64 {
    let (cat, _) = content();
    let mut h = FNV_START;
    for profile in [Profile::A, Profile::B, Profile::C] {
        let mut state = start(&cat, profile);
        if worn {
            for d in state.departments.values_mut() {
                d.condition_pts = 4_500;
            }
        }
        let hotel = Hotel {
            state: &state,
            indices: indices(avg),
            slot: 2,
        };
        for seed in 0..100 {
            for drawn in season(&cat, set, &hotel, PeriodKind::Week, seed) {
                for d in &drawn {
                    h = fnv(h, &describe(d, roles));
                }
                h = fnv(h, "--\n");
            }
        }
    }
    h
}

#[test]
fn weekly_draw_without_repeats_is_unchanged_since_52ce161() {
    let set = no_repeat_set();
    let got = [
        fingerprint(&set, 5_000, false, true),
        fingerprint(&set, 9_000, false, true),
        fingerprint(&set, 3_000, true, true),
    ];
    assert_eq!(
        got,
        [
            5987980432523894875,
            3209560799208187443,
            6467276241176834710
        ],
        "otisci"
    );
}

/// Ponovljeni primjerci od popravka dobivaju drugo ime (M2), a to pomiče i imena kasnijih primjeraka istog
/// kruga; zato ovdje otisak bez likova: predložak, šifra, krug, odjel, vrsta i brojevi.
#[test]
fn weekly_draw_with_four_week_repeats_is_unchanged_since_52ce161() {
    let set = repeat_set();
    let got = [
        fingerprint(&set, 5_000, false, false),
        fingerprint(&set, 5_000, true, false),
    ];
    assert_eq!(got, [8840238509046403872, 12105749331275259138], "otisci");
}
