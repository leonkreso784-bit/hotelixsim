//! Izvlačenje situacija iz predložaka (J-2, plan situacija §4): determinizam, ponavljanje, raznolikost, uvjeti i
//! pojačanja iz stanja, veća i kolektivna situacija, uloge i mjesta, u tjednoj i mjesečnoj igri. Zamrznuti
//! predlošci s `52ce161` (tri predloška S-1, `situations_common::frozen`) uz testne „opće" predloške, da novi
//! predlošci u repou ne pomiču pravila. Najmanji razmak između dva pojavljivanja istog predloška je 4 tjedna
//! (Leon), i kad predložak traži manje.

mod situations_common;

use std::collections::{BTreeMap, BTreeSet};

use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{
    DeptTarget, DrawnSituation, Gender, Kind, MajorSide, NumberParam, Repeat, RoleSource, Scope,
    situation_count,
};
use sim_hotel_v2::{HotelState, Month, PeriodKind, Profile, season_periods};
use situations_common::{
    Hotel, anchor, frozen, general, indices, round, season, start, with_general,
};

const KINDS: [PeriodKind; 2] = [PeriodKind::Week, PeriodKind::Month];

/// Najmanji razmak u danima sezone: `gap_rounds` tjedana, ali barem 4 tjedna.
fn min_gap_days(r: Repeat) -> u16 {
    u16::from(r.gap_rounds.max(4)) * 7
}

/// Najviše iz istog odjela u cijelom krugu: 2 u tjednu, ⌈2n/5⌉ u mjesecu.
fn round_dept_limit(n: u8) -> u8 {
    (n * 2).div_ceil(5).max(2)
}

#[test]
fn same_seed_gives_same_instances() {
    let (cat, set) = frozen();
    let set = with_general(
        set,
        8,
        Repeat {
            max_per_season: 3,
            gap_rounds: 2,
        },
    );
    let state = start(&cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for kind in KINDS {
        let a = season(&cat, &set, &h, kind, 7);
        assert_eq!(a, season(&cat, &set, &h, kind, 7));
        assert_ne!(a, season(&cat, &set, &h, kind, 8));
        let other_slot = Hotel {
            slot: 2,
            indices: indices(5_000),
            ..h
        };
        assert_ne!(a, season(&cat, &set, &other_slot, kind, 7));
    }
}

/// 1000 sjemena × tjedna i mjesečna igra: `max_per_season` i razmak (barem 4 tjedna) po danima sezone, šifre
/// jedinstvene u igri, najviše onoliko situacija koliko krug ima, najviše dvije iz istog odjela u (virtualnom)
/// tjednu i ⌈2n/5⌉ u krugu.
#[test]
fn thousand_seeds_respect_repeat_and_round_rules() {
    let (cat, set) = frozen();
    let set = with_general(
        set,
        8,
        Repeat {
            max_per_season: 3,
            gap_rounds: 2,
        },
    );
    let rules: BTreeMap<String, Repeat> = set
        .templates
        .iter()
        .map(|t| (t.code.clone(), t.repeat))
        .collect();
    for kind in KINDS {
        for profile in [Profile::A, Profile::B, Profile::C] {
            let state = start(&cat, profile);
            let h = Hotel {
                state: &state,
                indices: indices(5_000),
                slot: 1,
            };
            for seed in 0..1_000 {
                let rounds = season(&cat, &set, &h, kind, seed);
                let mut seen: BTreeMap<String, Vec<u16>> = BTreeMap::new();
                let mut codes: BTreeSet<&str> = BTreeSet::new();
                for (p, drawn) in season_periods(kind).iter().zip(&rounds) {
                    let n = situation_count(&state, p, &cat).unwrap();
                    assert!(
                        drawn.len() <= usize::from(n),
                        "{}: {} > {n}",
                        p.code(),
                        drawn.len()
                    );
                    let mut per_week: BTreeMap<(u16, &DeptCode), u8> = BTreeMap::new();
                    let mut per_round: BTreeMap<&DeptCode, u8> = BTreeMap::new();
                    for d in drawn {
                        assert_eq!(d.period, *p);
                        assert!(codes.insert(&d.code), "šifra {} dvaput u igri", d.code);
                        assert!(d.day >= p.start_day && d.day < p.start_day + u16::from(p.days));
                        assert_eq!((d.day - p.start_day) % 7, 0, "{}", d.code);
                        if let Some(dept) = &d.department {
                            *per_week.entry((d.day, dept)).or_default() += 1;
                            *per_round.entry(dept).or_default() += 1;
                        }
                        seen.entry(d.template.clone()).or_default().push(d.day);
                    }
                    // Tjedni krug: najviše 2 iz istog odjela. Mjesečni je jedan blok (B2a) s jednom granicom
                    // `round_dept_limit(n)` ispod, pa se tjedna granica gleda samo u tjednom krugu.
                    assert!(
                        kind != PeriodKind::Week || per_week.values().all(|&c| c <= 2),
                        "{}: {per_week:?}",
                        p.code()
                    );
                    assert!(
                        per_round.values().all(|&c| c <= round_dept_limit(n)),
                        "{}: {per_round:?}",
                        p.code()
                    );
                }
                for (code, days) in &seen {
                    let r = rules[code];
                    assert!(
                        days.len() <= usize::from(r.max_per_season),
                        "{code}: {days:?}"
                    );
                    let mut sorted = days.clone();
                    sorted.sort_unstable();
                    for w in sorted.windows(2) {
                        assert!(
                            w[1] - w[0] >= min_gap_days(r),
                            "{code}: razmak {days:?} (sjeme {seed}, {kind:?})"
                        );
                    }
                }
            }
        }
    }
}

/// Kad dopuštenih predložaka ima dovoljno, tjedni krug dobiva točno svoj broj, bez sidara i sa sidrima. Uz
/// razmak od 4 tjedna vrhunac (5 tjedno) traži barem 20 različitih dopuštenih predložaka.
#[test]
fn round_is_filled_to_its_count_minus_anchors() {
    let (cat, set) = frozen();
    let set = with_general(
        set,
        24,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    let state = start(&cat, Profile::A);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for seed in 0..200 {
        for (p, drawn) in season_periods(PeriodKind::Week).iter().zip(season(
            &cat,
            &set,
            &h,
            PeriodKind::Week,
            seed,
        )) {
            assert_eq!(
                drawn.len(),
                usize::from(situation_count(&state, p, &cat).unwrap())
            );
        }
    }
    let t01 = season_periods(PeriodKind::Week)[0];
    assert_eq!(situation_count(&state, &t01, &cat).unwrap(), 3);
    let two = [anchor(Kind::Complaint), anchor(Kind::Staff)];
    assert_eq!(
        round(&cat, &set, &h, &t01, &[], 1, &two).len(),
        1,
        "S1 i S3 su dva od tri"
    );
    let three = [
        anchor(Kind::Complaint),
        anchor(Kind::Staff),
        anchor(Kind::Crisis),
    ];
    assert!(round(&cat, &set, &h, &t01, &[], 1, &three).is_empty());
}

/// Poredak po važnosti: 4 prva.
#[test]
fn instances_are_ordered_by_importance() {
    let (cat, mut set) = frozen();
    let repeat = Repeat {
        max_per_season: 17,
        gap_rounds: 1,
    };
    for (i, w) in [1_u8, 4, 2, 3, 1, 4].into_iter().enumerate() {
        let mut t = general(&format!("P-VAZNO-{i}"), Kind::Request, repeat);
        t.weight = w;
        set.templates.push(t);
    }
    let state = start(&cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for kind in KINDS {
        for seed in 0..100 {
            for drawn in season(&cat, &set, &h, kind, seed) {
                let w: Vec<u8> = drawn.iter().map(|d| d.weight).collect();
                assert!(w.windows(2).all(|x| x[0] >= x[1]), "{w:?}");
            }
        }
    }
}

/// Stanje prostora gostinskih odjela ispod 40 (pojačanje P-DOTRAJALO ×3) povećava udio pritužbi na prostor;
/// iznad praga pojave (55) ih nema.
#[test]
fn worn_departments_raise_share_of_wear_complaints() {
    let (cat, mut set) = frozen();
    // Samo pritužba na prostor uz mnogo općih, da udio prati težinu (bez drugih pojačanih predložaka).
    set.templates.retain(|t| t.code == "P-DOTRAJALO");
    let set = with_general(
        set,
        30,
        Repeat {
            max_per_season: 3,
            gap_rounds: 2,
        },
    );
    let t06 = season_periods(PeriodKind::Week)[5];
    let share = |condition: i32, only: Option<&str>| -> (u32, BTreeMap<String, u32>) {
        let mut state = start(&cat, Profile::C);
        for (code, d) in state.departments.iter_mut() {
            if only.is_none_or(|o| o == code.as_str()) {
                d.condition_pts = condition;
            } else {
                d.condition_pts = 5_000;
            }
        }
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        let mut hits = 0;
        let mut depts: BTreeMap<String, u32> = BTreeMap::new();
        for seed in 0..1_000 {
            for d in round(&cat, &set, &h, &t06, &[], seed, &[]) {
                if d.template == "P-DOTRAJALO" {
                    hits += 1;
                    let dept = d.department.expect("pritužba na prostor ima odjel");
                    *depts.entry(dept.as_str().to_string()).or_default() += 1;
                }
            }
        }
        (hits, depts)
    };
    let (none, _) = share(6_000, None);
    let (mild, _) = share(5_000, None);
    let (worn, _) = share(3_500, None);
    assert_eq!(none, 0, "stanje 60 je iznad praga pojave");
    assert!(mild > 0);
    assert!(worn * 10 >= mild * 15, "pojačanje: {worn} prema {mild}");
    let (_, depts) = share(3_500, Some("spa"));
    let spa = depts.get("spa").copied().unwrap_or(0);
    assert!(
        depts.iter().all(|(k, &v)| k == "spa" || v < spa),
        "{depts:?}"
    );
}

#[test]
fn roles_and_numbers_are_filled_from_templates() {
    let (cat, set) = frozen();
    // Najviše 3 pojave po predlošku: ranija imena istog predloška (najviše 2) i imena kruga (najviše 4) ne
    // potroše popis od 8 imena, pa se ime u tjednom krugu ne ponavlja.
    let set = with_general(
        set,
        4,
        Repeat {
            max_per_season: 3,
            gap_rounds: 1,
        },
    );
    let state = start(&cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    let klima = set
        .templates
        .iter()
        .find(|t| t.code == "P-KLIMA")
        .unwrap()
        .clone();
    let guests_f = set.names.list(RoleSource::Guest, Gender::F).to_vec();
    let mut found = 0;
    for seed in 0..300 {
        for drawn in season(&cat, &set, &h, PeriodKind::Week, seed) {
            let names: Vec<&str> = drawn
                .iter()
                .flat_map(|d| d.roles.values().map(|r| r.name.as_str()))
                .collect();
            let unique: BTreeSet<&str> = names.iter().copied().collect();
            assert_eq!(
                unique.len(),
                names.len(),
                "ime se ponavlja u krugu: {names:?}"
            );
            for d in drawn.iter().filter(|d| d.template == "P-KLIMA") {
                found += 1;
                assert_eq!(d.department, Some(DeptCode::rooms()));
                let gost = &d.roles["gost"];
                assert!(guests_f.contains(&gost.name));
                let types = &klima.roles["gost"].types;
                assert!(types.contains(gost.role_type.as_ref().unwrap()));
                assert!((101..=520).contains(&d.numbers["soba"]));
                assert!([32, 34, 36].contains(&d.numbers["temp"]));
                assert_eq!(d.numbers.len(), 2, "iznosi nisu brojevna mjesta");
            }
        }
    }
    assert!(found > 0);
    assert!(matches!(
        klima.numbers["soba"],
        NumberParam::Range(101, 520)
    ));
}

/// Kolektivna situacija (`svi`): ista za sve hotele (utor 0), u prvom krugu mjeseca, najviše jedna mjesečno —
/// u tjednoj i u mjesečnoj igri (u mjesečnoj je svaki krug prvi u svom mjesecu).
#[test]
fn collective_situation_is_the_same_for_every_hotel() {
    let (cat, mut set) = frozen();
    let mut svi = general(
        "P-SVI",
        Kind::Crisis,
        Repeat {
            max_per_season: 4,
            gap_rounds: 1,
        },
    );
    svi.scope = Scope::All;
    svi.when.months = vec![Month::Jul, Month::Aug];
    svi.numbers
        .insert("broj".to_string(), NumberParam::Range(1, 1_000_000));
    set.templates.push(svi);
    let set = with_general(
        set,
        24,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    let states: Vec<HotelState> = [Profile::A, Profile::B, Profile::C]
        .into_iter()
        .map(|p| start(&cat, p))
        .collect();
    for kind in KINDS {
        for seed in 0..50 {
            let seasons: Vec<Vec<Vec<DrawnSituation>>> = states
                .iter()
                .zip(1_u8..)
                .map(|(s, slot)| {
                    let h = Hotel {
                        state: s,
                        indices: indices(5_000),
                        slot,
                    };
                    season(&cat, &set, &h, kind, seed)
                })
                .collect();
            for (i, p) in season_periods(kind).iter().enumerate() {
                let collective: Vec<Vec<&DrawnSituation>> = seasons
                    .iter()
                    .map(|s| s[i].iter().filter(|d| d.scope == Scope::All).collect())
                    .collect();
                assert!(collective.windows(2).all(|w| w[0] == w[1]), "{}", p.code());
                let first_of_month = season_periods(kind)
                    .iter()
                    .find(|q| q.start_month() == p.start_month())
                    == Some(p);
                let expected =
                    first_of_month && [Month::Jul, Month::Aug].contains(&p.start_month());
                assert_eq!(collective[0].len(), usize::from(expected), "{}", p.code());
                for c in &collective[0] {
                    assert_eq!(c.day, p.start_day, "kolektivna je u prvom tjednu kruga");
                    assert_eq!(c.code, format!("P-SVI.{}", p.code()));
                }
            }
        }
    }
}

/// Veća situacija: samo kad je prosjek pokazatelja ≥ 85 (`high`) ili ≤ 35 (`low`), najviše jedna mjesečno.
#[test]
fn major_situation_only_in_extremes_and_once_a_month() {
    let (cat, mut set) = frozen();
    for (code, side) in [
        ("P-VECA-DOBRO", MajorSide::High),
        ("P-VECA-LOSE", MajorSide::Low),
    ] {
        let mut t = general(
            code,
            Kind::Opportunity,
            Repeat {
                max_per_season: 17,
                gap_rounds: 1,
            },
        );
        t.major = Some(side);
        set.templates.push(t);
    }
    let set = with_general(
        set,
        24,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    let state = start(&cat, Profile::C);
    for kind in KINDS {
        for (avg, expected) in [
            (9_000, Some("P-VECA-DOBRO")),
            (5_000, None),
            (3_000, Some("P-VECA-LOSE")),
        ] {
            let h = Hotel {
                state: &state,
                indices: indices(avg),
                slot: 1,
            };
            for seed in 0..50 {
                let rounds = season(&cat, &set, &h, kind, seed);
                let mut per_month: BTreeMap<Month, Vec<&str>> = BTreeMap::new();
                for (p, drawn) in season_periods(kind).iter().zip(&rounds) {
                    for d in drawn.iter().filter(|d| d.major.is_some()) {
                        per_month
                            .entry(p.start_month())
                            .or_default()
                            .push(d.template.as_str());
                    }
                }
                match expected {
                    None => assert!(per_month.is_empty()),
                    Some(code) => {
                        assert_eq!(per_month.len(), 4, "jedna u svakom mjesecu: {per_month:?}");
                        assert!(
                            per_month.values().all(|v| v == &vec![code]),
                            "{per_month:?}"
                        );
                    }
                }
            }
        }
    }
}

/// Raznolikost: barem dvije vrste u krugu i barem jedna prilika u svaka tri kruga, kad ih ima. Prilika ima
/// osam (svaka se smije ponoviti tek nakon 4 tjedna).
#[test]
fn kinds_vary_and_an_opportunity_comes_every_three_rounds() {
    let (cat, set) = frozen();
    let repeat = Repeat {
        max_per_season: 17,
        gap_rounds: 1,
    };
    let mut set = with_general(set, 24, repeat);
    for t in set
        .templates
        .iter_mut()
        .filter(|t| t.code.starts_with("P-OPCI"))
    {
        t.kind = Kind::Request;
    }
    for i in 0..8 {
        set.templates.push(general(
            &format!("P-PRILIKA-{i}"),
            Kind::Opportunity,
            repeat,
        ));
    }
    let state = start(&cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for seed in 0..300 {
        let rounds = season(&cat, &set, &h, PeriodKind::Week, seed);
        for drawn in &rounds {
            let kinds: BTreeSet<Kind> = drawn.iter().map(|d| d.kind).collect();
            assert!(kinds.len() >= 2, "{kinds:?}");
        }
        for w in rounds.windows(3) {
            assert!(
                w.iter().flatten().any(|d| d.kind == Kind::Opportunity),
                "tri kruga bez prilike (sjeme {seed})"
            );
        }
    }
}

/// Pretvorba iz sadržaja čuva pravila pojave.
#[test]
fn content_templates_convert_to_core_rules() {
    let (cat, set) = frozen();
    let klima = set.templates.iter().find(|t| t.code == "P-KLIMA").unwrap();
    assert_eq!(klima.kind, Kind::Complaint);
    assert_eq!(klima.department, DeptTarget::One(DeptCode::rooms()));
    assert_eq!(klima.when.months, vec![Month::Jun, Month::Jul, Month::Aug]);
    assert_eq!(klima.when.occupancy_bp.unwrap().at_least, Some(6_000));
    assert_eq!(klima.chance.boost[0].times, 3);
    assert_eq!(
        klima.repeat,
        Repeat {
            max_per_season: 2,
            gap_rounds: 5
        }
    );
    let povisica = set
        .templates
        .iter()
        .find(|t| t.code == "P-POVISICA")
        .unwrap();
    assert_eq!(povisica.department, DeptTarget::Staffed);
    assert_eq!(povisica.roles["radnik"].source, RoleSource::Staff);
    assert_eq!(set.major.high_avg_min, 85);
    let dotrajalo = set
        .templates
        .iter()
        .find(|t| t.code == "P-DOTRAJALO")
        .unwrap();
    let members = dotrajalo.department.members(&cat).unwrap();
    assert!(
        members.contains(&DeptCode::rooms()) && members.len() == 6,
        "{members:?}"
    );
}
