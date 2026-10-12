//! Mjesečni krug (Leon: isti predložak smije se ponoviti uz razmak od barem 4 tjedna, i u mjesečnoj igri s
//! 13–22 situacije): krug se izvlači kao niz virtualnih tjedana (7 dana od početka mjeseca, zadnji dio ostatak),
//! pa predložak iz prvog tjedna smije doći opet od 28. dana mjeseca, s drugim imenom i drugim brojevima.

mod situations_common;

use std::collections::{BTreeMap, BTreeSet};

use sim_hotel_v2::situations::{
    DrawnSituation, NumberParam, Repeat, TemplateSet, expected_occupancy_bp, situation_count,
    virtual_weeks,
};
use sim_hotel_v2::{Month, PeriodKind, Profile, season_periods};
use situations_common::{Hotel, content, indices, season, start, with_general};

/// Samo testni opći predlošci (bez uvjeta), dovoljno ih za kolovoz uz razmak od 4 tjedna.
fn month_set() -> TemplateSet {
    let (_, mut set) = content();
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

fn period(m: Month) -> sim_hotel_v2::Period {
    *season_periods(PeriodKind::Month)
        .iter()
        .find(|p| p.start_month() == m)
        .unwrap()
}

#[test]
fn virtual_weeks_split_the_month_count() {
    let (cat, _) = content();
    let split = |profile: Profile, m: Month| -> Vec<(u8, u8)> {
        let state = start(&cat, profile);
        let p = period(m);
        let occ = expected_occupancy_bp(&state, &p, &cat).unwrap();
        virtual_weeks(&p, occ)
            .unwrap()
            .iter()
            .map(|w| (w.offset_days, w.count))
            .collect()
    };
    // Kolovoz (31 dan, 100 %): 22 = 5+5+5+5+2; lipanj C (30 dana, < 75 %): 13 = 3+3+3+3+1.
    assert_eq!(
        split(Profile::A, Month::Aug),
        vec![(0, 5), (7, 5), (14, 5), (21, 5), (28, 2)]
    );
    assert_eq!(
        split(Profile::C, Month::Jun),
        vec![(0, 3), (7, 3), (14, 3), (21, 3), (28, 1)]
    );
    let state = start(&cat, Profile::A);
    for p in season_periods(PeriodKind::Month) {
        let occ = expected_occupancy_bp(&state, &p, &cat).unwrap();
        let sum: u8 = virtual_weeks(&p, occ)
            .unwrap()
            .iter()
            .map(|w| w.count)
            .sum();
        assert_eq!(
            sum,
            situation_count(&state, &p, &cat).unwrap(),
            "{}",
            p.code()
        );
    }
    let t05 = season_periods(PeriodKind::Week)[4];
    let occ = expected_occupancy_bp(&state, &t05, &cat).unwrap();
    let weeks = virtual_weeks(&t05, occ).unwrap();
    assert_eq!(weeks.len(), 1, "tjedni krug je jedan tjedan");
    assert_eq!(weeks[0].offset_days, 0);
    assert_eq!(weeks[0].count, situation_count(&state, &t05, &cat).unwrap());
}

/// Srpanj i kolovoz (i cijela mjesečna sezona): krug ima točno svoj broj, isti predložak se u krugu ponavlja
/// samo uz razmak od barem 28 dana, šifre su jedinstvene u igri, a ponovljeni primjerak ima redni sufiks, drugo
/// ime i druge brojeve.
#[test]
fn monthly_round_is_filled_and_repeats_keep_28_days() {
    let (cat, _) = content();
    let set = month_set();
    let mut repeated = 0;
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
            let mut days: BTreeMap<&str, Vec<u16>> = BTreeMap::new();
            let mut earlier: BTreeMap<&str, Vec<&DrawnSituation>> = BTreeMap::new();
            for (p, drawn) in season_periods(PeriodKind::Month).iter().zip(&rounds) {
                assert_eq!(
                    drawn.len(),
                    usize::from(situation_count(&state, p, &cat).unwrap()),
                    "{} (sjeme {seed})",
                    p.code()
                );
                let mut by_day: Vec<&DrawnSituation> = drawn.iter().collect();
                by_day.sort_by_key(|d| d.day);
                for d in by_day {
                    assert!(codes.insert(&d.code), "šifra {} dvaput", d.code);
                    let expected = if d.occurrence == 1 {
                        format!("{}.{}", d.template, p.code())
                    } else {
                        format!("{}.{}.{}", d.template, p.code(), d.occurrence)
                    };
                    assert_eq!(d.code, expected);
                    let before = earlier.entry(&d.template).or_default();
                    for e in before.iter() {
                        assert!(
                            d.day - e.day >= 28,
                            "{}: {} pa {} (sjeme {seed})",
                            d.template,
                            e.day,
                            d.day
                        );
                        assert_ne!(d.roles["gost"].name, e.roles["gost"].name, "{}", d.code);
                    }
                    if d.occurrence > 1 {
                        repeated += 1;
                        let first = before
                            .iter()
                            .find(|e| e.period == *p && e.occurrence == 1)
                            .expect("prvi primjerak u istom krugu");
                        assert_ne!(d.numbers["broj"], first.numbers["broj"], "{}", d.code);
                    }
                    before.push(d);
                    days.entry(&d.template).or_default().push(d.day);
                }
            }
            if seed == 0 && profile == Profile::A {
                let aug = period(Month::Aug);
                let per_week: Vec<usize> = (0..5)
                    .map(|k| {
                        rounds[2]
                            .iter()
                            .filter(|d| d.day == aug.start_day + 7 * k)
                            .count()
                    })
                    .collect();
                assert_eq!(
                    per_week,
                    vec![5, 5, 5, 5, 2],
                    "kolovoz po virtualnim tjednima"
                );
            }
        }
    }
    assert!(repeated > 0, "ponavljanje u mjesecu se doista događa");
}

#[test]
fn monthly_draw_is_reproducible() {
    let (cat, _) = content();
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
