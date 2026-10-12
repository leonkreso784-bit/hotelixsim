//! Izvlačenje situacija iz predložaka (J-2, plan situacija §4): determinizam, ponavljanje, raznolikost, uvjeti i
//! pojačanja iz stanja, veća i kolektivna situacija, uloge i mjesta. Predlošci iz repoa (pretvoreni u jezgreni
//! oblik) uz testne „opće" predloške, jer ih repo za sada ima samo tri.

use std::collections::{BTreeMap, BTreeSet};

use content::{Snapshot, repo_dir};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{
    Chance, Conditions, DeptTarget, Draw, DrawView, DrawnSituation, Gender, Kind, MajorSide,
    NumberParam, Repeat, RoleSource, RoleSpec, Scope, SituationSpec, TemplateSet, draw_situations,
    situation_count,
};
use sim_hotel_v2::{
    Catalog, HotelState, IndexSet, Month, Period, PeriodKind, Profile, season_periods,
};

fn content() -> (Catalog, TemplateSet) {
    let c = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let set = c
        .situation_templates
        .expect("v2 snimak ima predloške")
        .to_core();
    (c.v2.expect("v2 katalog"), set)
}

fn indices(avg_hundredths: u16) -> IndexSet {
    IndexSet::from_hundredths([avg_hundredths; 5]).unwrap()
}

/// Opći predložak bez uvjeta (cijeli hotel).
fn general(code: &str, kind: Kind, repeat: Repeat) -> SituationSpec {
    SituationSpec {
        code: code.to_string(),
        kind,
        department: DeptTarget::Hotel,
        weight: 1,
        scope: Scope::Hotel,
        major: None,
        when: Conditions::default(),
        chance: Chance {
            base: 1,
            boost: vec![],
        },
        repeat,
        roles: BTreeMap::from([(
            "gost".to_string(),
            RoleSpec {
                source: RoleSource::Guest,
                gender: Gender::M,
                types: vec![],
            },
        )]),
        numbers: BTreeMap::new(),
    }
}

fn with_general(mut set: TemplateSet, n: usize, repeat: Repeat) -> TemplateSet {
    let kinds = [Kind::Request, Kind::Breakdown, Kind::Crisis, Kind::Staff];
    for i in 0..n {
        set.templates.push(general(
            &format!("P-OPCI-{i}"),
            kinds[i % kinds.len()],
            repeat,
        ));
    }
    set
}

struct Hotel<'a> {
    state: &'a HotelState,
    indices: IndexSet,
    slot: u8,
}

fn round(
    cat: &Catalog,
    set: &TemplateSet,
    h: &Hotel,
    p: &Period,
    history: &[DrawnSituation],
    seed: u64,
    anchors: u8,
) -> Vec<DrawnSituation> {
    let view = DrawView {
        state: h.state,
        previous: None,
        indices: &h.indices,
    };
    let draw = Draw {
        period: p,
        templates: set,
        history,
        seed,
        slot: h.slot,
        anchors,
    };
    draw_situations(&view, &draw, cat).unwrap()
}

/// Cijela sezona jednog hotela uz nepromijenjeno početno stanje; povijest raste krug po krug.
fn season(
    cat: &Catalog,
    set: &TemplateSet,
    h: &Hotel,
    kind: PeriodKind,
    seed: u64,
) -> Vec<Vec<DrawnSituation>> {
    let mut history: Vec<DrawnSituation> = Vec::new();
    let mut out = Vec::new();
    for p in season_periods(kind) {
        let drawn = round(cat, set, h, &p, &history, seed, 0);
        history.extend(drawn.iter().cloned());
        out.push(drawn);
    }
    out
}

fn start(cat: &Catalog, profile: Profile) -> HotelState {
    HotelState::from_profile(cat, profile).unwrap()
}

#[test]
fn same_seed_gives_same_instances() {
    let (cat, set) = content();
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
    let a = season(&cat, &set, &h, PeriodKind::Week, 7);
    assert_eq!(a, season(&cat, &set, &h, PeriodKind::Week, 7));
    assert_ne!(a, season(&cat, &set, &h, PeriodKind::Week, 8));
    let other_slot = Hotel { slot: 2, ..h };
    assert_ne!(a, season(&cat, &set, &other_slot, PeriodKind::Week, 7));
}

/// 1000 sjemena: isti predložak nikad dvaput u krugu, `max_per_season` i razmak poštovani, najviše onoliko
/// situacija koliko krug ima, najviše dvije iz istog odjela u tjednom krugu.
#[test]
fn thousand_seeds_respect_repeat_and_round_rules() {
    let (cat, set) = content();
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
    for profile in [Profile::A, Profile::B, Profile::C] {
        let state = start(&cat, profile);
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        for seed in 0..1_000 {
            let rounds = season(&cat, &set, &h, PeriodKind::Week, seed);
            let mut seen: BTreeMap<String, Vec<u16>> = BTreeMap::new();
            for (p, drawn) in season_periods(PeriodKind::Week).iter().zip(&rounds) {
                let n = situation_count(&state, p, &cat).unwrap();
                assert!(
                    drawn.len() <= usize::from(n),
                    "{}: {} > {n}",
                    p.code(),
                    drawn.len()
                );
                let codes: BTreeSet<&str> = drawn.iter().map(|d| d.template.as_str()).collect();
                assert_eq!(
                    codes.len(),
                    drawn.len(),
                    "isti predložak dvaput u {}",
                    p.code()
                );
                let mut per_dept: BTreeMap<&DeptCode, u8> = BTreeMap::new();
                for d in drawn {
                    assert_eq!(d.period, *p);
                    assert_eq!(d.code, format!("{}.{}", d.template, p.code()));
                    if let Some(dept) = &d.department {
                        *per_dept.entry(dept).or_default() += 1;
                    }
                    seen.entry(d.template.clone())
                        .or_default()
                        .push(p.start_day);
                }
                assert!(
                    per_dept.values().all(|&c| c <= 2),
                    "{}: {per_dept:?}",
                    p.code()
                );
            }
            for (code, days) in &seen {
                let r = rules[code];
                assert!(
                    days.len() <= usize::from(r.max_per_season),
                    "{code}: {days:?}"
                );
                for w in days.windows(2) {
                    assert!(
                        w[1] - w[0] >= u16::from(r.gap_rounds) * 7,
                        "{code}: razmak {days:?} (sjeme {seed})"
                    );
                }
            }
        }
    }
}

/// Kad dopuštenih predložaka ima dovoljno, krug dobiva točno svoj broj, bez sidara i sa sidrima.
#[test]
fn round_is_filled_to_its_count_minus_anchors() {
    let (cat, set) = content();
    let set = with_general(
        set,
        6,
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
    assert_eq!(
        round(&cat, &set, &h, &t01, &[], 1, 2).len(),
        1,
        "S1 i S3 su dva od tri"
    );
    assert!(round(&cat, &set, &h, &t01, &[], 1, 3).is_empty());
}

/// Poredak po važnosti: 4 prva.
#[test]
fn instances_are_ordered_by_importance() {
    let (cat, mut set) = content();
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
    for seed in 0..100 {
        for drawn in season(&cat, &set, &h, PeriodKind::Week, seed) {
            let w: Vec<u8> = drawn.iter().map(|d| d.weight).collect();
            assert!(w.windows(2).all(|x| x[0] >= x[1]), "{w:?}");
        }
    }
}

/// Stanje prostora gostinskih odjela ispod 40 (pojačanje P-DOTRAJALO ×3) povećava udio pritužbi na prostor;
/// iznad praga pojave (55) ih nema.
#[test]
fn worn_departments_raise_share_of_wear_complaints() {
    let (cat, mut set) = content();
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
            for d in round(&cat, &set, &h, &t06, &[], seed, 0) {
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
    let (cat, set) = content();
    let set = with_general(
        set,
        4,
        Repeat {
            max_per_season: 17,
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

/// Kolektivna situacija (`svi`): ista za sve hotele (utor 0), u prvom krugu mjeseca, najviše jedna mjesečno.
#[test]
fn collective_situation_is_the_same_for_every_hotel() {
    let (cat, mut set) = content();
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
        6,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    let states: Vec<HotelState> = [Profile::A, Profile::B, Profile::C]
        .into_iter()
        .map(|p| start(&cat, p))
        .collect();
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
                season(&cat, &set, &h, PeriodKind::Week, seed)
            })
            .collect();
        for (i, p) in season_periods(PeriodKind::Week).iter().enumerate() {
            let collective: Vec<Vec<&DrawnSituation>> = seasons
                .iter()
                .map(|s| s[i].iter().filter(|d| d.scope == Scope::All).collect())
                .collect();
            assert!(collective.windows(2).all(|w| w[0] == w[1]), "{}", p.code());
            let first_of_month = season_periods(PeriodKind::Week)
                .iter()
                .find(|q| q.start_month() == p.start_month())
                == Some(p);
            let expected = first_of_month && [Month::Jul, Month::Aug].contains(&p.start_month());
            assert_eq!(collective[0].len(), usize::from(expected), "{}", p.code());
        }
    }
}

/// Veća situacija: samo kad je prosjek pokazatelja ≥ 85 (`high`) ili ≤ 35 (`low`), najviše jedna mjesečno.
#[test]
fn major_situation_only_in_extremes_and_once_a_month() {
    let (cat, mut set) = content();
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
        6,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    let state = start(&cat, Profile::C);
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
            let rounds = season(&cat, &set, &h, PeriodKind::Week, seed);
            let mut per_month: BTreeMap<Month, Vec<&str>> = BTreeMap::new();
            for (p, drawn) in season_periods(PeriodKind::Week).iter().zip(&rounds) {
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

/// Raznolikost: barem dvije vrste u krugu i barem jedna prilika u svaka tri kruga, kad ih ima.
#[test]
fn kinds_vary_and_an_opportunity_comes_every_three_rounds() {
    let (cat, set) = content();
    let mut set = with_general(
        set,
        6,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    for t in set
        .templates
        .iter_mut()
        .filter(|t| t.code.starts_with("P-OPCI"))
    {
        t.kind = Kind::Request;
    }
    let mut prilika = general(
        "P-PRILIKA",
        Kind::Opportunity,
        Repeat {
            max_per_season: 17,
            gap_rounds: 1,
        },
    );
    prilika.chance.base = 1;
    set.templates.push(prilika);
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
    let (cat, set) = content();
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
