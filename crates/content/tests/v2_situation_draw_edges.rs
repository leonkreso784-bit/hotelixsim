//! Rubni slučajevi izvlačenja (pregled J-2, I1 i I4, M3, M4, M6): zatvoren odjel, prazan popis, krug bez
//! dopuštenih, uvjeti iz obračuna prošlog kruga, točne granice veće situacije, kolektivna nikad na odjelu koji se
//! može zatvoriti, sidra u granici odjela i vrstama, povijest drugog hotela, točna težina pojačanja.

mod situations_common;

use std::collections::BTreeSet;

use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{
    Anchor, Boost, Bounds, Conditions, DeptTarget, Draw, DrawView, DrawnSituation, Kind, MajorSide,
    Repeat, Scope, TemplateSet, draw_situations, expected_occupancy_bp,
};
use sim_hotel_v2::{
    Catalog, HotelState, IndexSet, Month, PeriodKind, PeriodPlan, Profile, ResolveError,
    resolve_period, season_periods,
};
use situations_common::{Hotel, content, general, indices, round, season, start, with_general};

const KINDS: [PeriodKind; 2] = [PeriodKind::Week, PeriodKind::Month];

const OFTEN: Repeat = Repeat {
    max_per_season: 17,
    gap_rounds: 1,
};

fn closable(cat: &Catalog) -> BTreeSet<DeptCode> {
    cat.departments
        .iter()
        .filter(|(_, s)| s.can_close)
        .map(|(k, _)| k.clone())
        .collect()
}

#[test]
fn closed_department_never_gets_a_situation() {
    let (cat, set) = content();
    let set = with_general(set, 24, OFTEN);
    let closed = closable(&cat);
    assert!(!closed.is_empty());
    let mut state = start(&cat, Profile::C);
    for (code, d) in state.departments.iter_mut() {
        d.condition_pts = 3_500;
        if closed.contains(code) {
            d.open = false;
        }
    }
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for kind in KINDS {
        let mut with_dept = 0;
        for seed in 0..300 {
            for d in season(&cat, &set, &h, kind, seed).iter().flatten() {
                if let Some(dept) = &d.department {
                    with_dept += 1;
                    assert!(!closed.contains(dept), "{} na zatvorenom {dept:?}", d.code);
                }
            }
        }
        assert!(with_dept > 0, "situacije s odjelom ipak dolaze ({kind:?})");
    }
}

#[test]
fn empty_template_set_draws_nothing() {
    let (cat, mut set) = content();
    set.templates.clear();
    let state = start(&cat, Profile::A);
    let h = Hotel {
        state: &state,
        indices: indices(9_000),
        slot: 1,
    };
    for kind in KINDS {
        assert!(season(&cat, &set, &h, kind, 5).iter().all(Vec::is_empty));
    }
}

#[test]
fn round_without_allowed_templates_draws_nothing() {
    let (cat, mut set) = content();
    set.templates.clear();
    for i in 0..10 {
        let mut t = general(&format!("P-RUJAN-{i}"), Kind::Request, OFTEN);
        t.when.months = vec![Month::Sep];
        set.templates.push(t);
    }
    let state = start(&cat, Profile::A);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    for kind in KINDS {
        for p in season_periods(kind)
            .iter()
            .filter(|p| p.start_month() == Month::Jun)
        {
            assert!(
                round(&cat, &set, &h, p, &[], 3, &[]).is_empty(),
                "{}",
                p.code()
            );
        }
    }
    // Potrošen `max_per_season`: jedini predložak je već bio.
    let mut once = TemplateSet {
        templates: vec![general(
            "P-JEDNOM",
            Kind::Request,
            Repeat {
                max_per_season: 1,
                gap_rounds: 1,
            },
        )],
        ..set
    };
    let weeks = season_periods(PeriodKind::Week);
    let first = round(&cat, &once, &h, &weeks[0], &[], 3, &[]);
    assert_eq!(first.len(), 1);
    assert!(round(&cat, &once, &h, &weeks[10], &first, 3, &[]).is_empty());
    once.templates[0].repeat.max_per_season = 2;
    assert_eq!(round(&cat, &once, &h, &weeks[10], &first, 3, &[]).len(), 1);
}

fn bounds(at_least: Option<i64>, below: Option<i64>) -> Option<Bounds> {
    Some(Bounds { at_least, below })
}

/// Popunjenost i opterećenje odjela dolaze iz obračuna prošlog kruga (`previous`); bez njega popunjenost je
/// očekivana za ovaj krug, a opterećenje 0.
#[test]
fn previous_report_drives_occupancy_and_load_conditions() {
    let (cat, mut set) = content();
    let state = start(&cat, Profile::C);
    let weeks = season_periods(PeriodKind::Week);
    let (p1, p2) = (weeks[0], weeks[1]);
    let plan = PeriodPlan::default_for(&state, &p1, &cat).unwrap();
    let report = resolve_period(&state, &p1, &plan, &[], &cat).unwrap();
    let nights = i64::from(report.before.rooms) * i64::from(report.period.days);
    let sold = i64::from(report.summary.market.sold);
    let occ = (sold * 10_000 + nights / 2) / nights;
    let load = i64::from(report.accounts.departments[&DeptCode::rooms()].load_hundredths);
    assert!(load > 0, "prvi krug ima opterećenje smještaja");
    let hotel_tpl = |code: &str, when: Conditions| {
        let mut t = general(code, Kind::Request, OFTEN);
        t.when = when;
        t
    };
    let rooms_tpl = |code: &str, when: Conditions| {
        let mut t = hotel_tpl(code, when);
        t.department = DeptTarget::One(DeptCode::rooms());
        t
    };
    set.templates = vec![
        hotel_tpl(
            "P-POP-DA",
            Conditions {
                occupancy_bp: bounds(Some(occ), Some(occ + 1)),
                ..Conditions::default()
            },
        ),
        hotel_tpl(
            "P-POP-NE",
            Conditions {
                occupancy_bp: bounds(Some(occ + 1), None),
                ..Conditions::default()
            },
        ),
        rooms_tpl(
            "P-OPT-DA",
            Conditions {
                dept_load_hundredths: bounds(Some(load), Some(load + 1)),
                ..Conditions::default()
            },
        ),
        rooms_tpl(
            "P-OPT-NULA",
            Conditions {
                dept_load_hundredths: bounds(None, Some(1)),
                ..Conditions::default()
            },
        ),
    ];
    let after = report.after.clone();
    let idx = indices(5_000);
    let codes = |previous| -> BTreeSet<String> {
        let view = DrawView {
            state: &after,
            previous,
            indices: &idx,
        };
        let draw = Draw {
            period: &p2,
            templates: &set,
            history: &[],
            seed: 9,
            slot: 1,
            anchors: &[],
        };
        draw_situations(&view, &draw, &cat)
            .unwrap()
            .into_iter()
            .map(|d| d.template)
            .collect()
    };
    assert_eq!(
        codes(Some(&report)),
        BTreeSet::from(["P-POP-DA".to_string(), "P-OPT-DA".to_string()])
    );
    let expected = i64::from(expected_occupancy_bp(&after, &p2, &cat).unwrap());
    let mut without = BTreeSet::from(["P-OPT-NULA".to_string()]);
    if expected == occ {
        without.insert("P-POP-DA".to_string());
    }
    if expected > occ {
        without.insert("P-POP-NE".to_string());
    }
    assert_eq!(codes(None), without);
}

/// Granice veće situacije su uključive, a prosjek pet pokazatelja zaokružen prema dolje.
#[test]
fn major_thresholds_are_inclusive() {
    let (cat, mut set) = content();
    for (code, side) in [
        ("P-VECA-HI", MajorSide::High),
        ("P-VECA-LO", MajorSide::Low),
    ] {
        let mut t = general(code, Kind::Opportunity, OFTEN);
        t.major = Some(side);
        set.templates.push(t);
    }
    let set = with_general(set, 6, OFTEN);
    let state = start(&cat, Profile::C);
    let t01 = season_periods(PeriodKind::Week)[0];
    for (values, expected) in [
        ([8_500; 5], Some(MajorSide::High)),
        ([8_500, 8_500, 8_500, 8_500, 8_504], Some(MajorSide::High)),
        ([8_499; 5], None),
        ([8_499, 8_499, 8_499, 8_499, 8_503], None),
        ([3_500; 5], Some(MajorSide::Low)),
        ([3_501, 3_501, 3_501, 3_501, 3_500], Some(MajorSide::Low)),
        ([3_501; 5], None),
    ] {
        let h = Hotel {
            state: &state,
            indices: IndexSet::from_hundredths(values).unwrap(),
            slot: 1,
        };
        let majors: Vec<Option<MajorSide>> = round(&cat, &set, &h, &t01, &[], 4, &[])
            .iter()
            .filter(|d| d.major.is_some())
            .map(|d| d.major)
            .collect();
        assert_eq!(
            majors,
            expected.into_iter().map(Some).collect::<Vec<_>>(),
            "{values:?}"
        );
    }
}

/// Kolektivna je ista za sve hotele, pa smije pasti samo na odjel koji se ne može zatvoriti (inače bi hotel sa
/// zatvorenim odjelom dobio problem u odjelu koji ne radi).
#[test]
fn collective_never_targets_a_closable_department() {
    let (cat, mut set) = content();
    let closed = closable(&cat);
    for (code, dept) in [
        ("P-SVI-GOSTI", DeptTarget::GuestFacing),
        ("P-SVI-OSOBLJE", DeptTarget::Staffed),
        ("P-SVI-SPA", DeptTarget::One(DeptCode("spa".to_string()))),
    ] {
        let mut t = general(code, Kind::Crisis, OFTEN);
        t.scope = Scope::All;
        t.department = dept;
        set.templates.push(t);
    }
    let set = with_general(set, 24, OFTEN);
    let state = start(&cat, Profile::B);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 2,
    };
    let mut seen = BTreeSet::new();
    for kind in KINDS {
        for seed in 0..200 {
            for d in season(&cat, &set, &h, kind, seed)
                .iter()
                .flatten()
                .filter(|d| d.scope == Scope::All)
            {
                seen.insert(d.template.clone());
                let dept = d.department.as_ref().expect("skupina daje odjel");
                assert!(!closed.contains(dept), "{} na {dept:?}", d.code);
            }
        }
    }
    assert!(
        seen.contains("P-SVI-GOSTI") && seen.contains("P-SVI-OSOBLJE"),
        "{seen:?}"
    );
    assert!(!seen.contains("P-SVI-SPA"));
}

/// Sidra se broje u granicu odjela i u vrste kruga (M3).
#[test]
fn anchors_count_toward_department_limit_and_kinds() {
    let (cat, mut set) = content();
    set.templates.clear();
    for i in 0..6 {
        let mut t = general(&format!("P-SOBE-{i}"), Kind::Complaint, OFTEN);
        t.department = DeptTarget::One(DeptCode::rooms());
        set.templates.push(t);
        set.templates
            .push(general(&format!("P-HOTEL-{i}"), Kind::Request, OFTEN));
    }
    set.templates.push(general("P-KRIZA", Kind::Crisis, OFTEN));
    let state = start(&cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    let t06 = season_periods(PeriodKind::Week)[5];
    let rooms = |kind| Anchor {
        kind,
        department: Some(DeptCode::rooms()),
    };
    for seed in 0..200 {
        // Dva sidra iz smještaja: treće iz smještaja ne smije doći.
        let drawn = round(
            &cat,
            &set,
            &h,
            &t06,
            &[],
            seed,
            &[rooms(Kind::Staff), rooms(Kind::Staff)],
        );
        assert!(!drawn.is_empty());
        assert!(drawn.iter().all(|d| d.department.is_none()), "{drawn:?}");
        // Tri sidra iste vrste: zadnje mjesto mora biti druge vrste.
        let n = sim_hotel_v2::situations::situation_count(&state, &t06, &cat).unwrap();
        let same: Vec<Anchor> = (1..n)
            .map(|_| Anchor {
                kind: Kind::Request,
                department: None,
            })
            .collect();
        let last = round(&cat, &set, &h, &t06, &[], seed, &same);
        assert_eq!(last.len(), 1);
        assert_ne!(last[0].kind, Kind::Request, "sjeme {seed}");
    }
}

/// Povijest smije imati samo primjerke ovog hotela i kolektivne, svaku šifru jednom (M4).
#[test]
fn history_from_another_hotel_is_rejected() {
    let (cat, set) = content();
    let set = with_general(set, 24, OFTEN);
    let state = start(&cat, Profile::A);
    let t02 = season_periods(PeriodKind::Week)[1];
    let t01 = season_periods(PeriodKind::Week)[0];
    let other = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 2,
    };
    let foreign = round(&cat, &set, &other, &t01, &[], 1, &[]);
    assert!(!foreign.is_empty());
    let idx = indices(5_000);
    let attempt = |history: &[DrawnSituation]| {
        let view = DrawView {
            state: &state,
            previous: None,
            indices: &idx,
        };
        let draw = Draw {
            period: &t02,
            templates: &set,
            history,
            seed: 1,
            slot: 1,
            anchors: &[],
        };
        draw_situations(&view, &draw, &cat)
    };
    assert!(matches!(attempt(&foreign), Err(ResolveError::Catalog(_))));
    let own = round(
        &cat,
        &set,
        &Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        },
        &t01,
        &[],
        1,
        &[],
    );
    assert!(attempt(&own).is_ok());
    let twice: Vec<DrawnSituation> = own.iter().chain(own.iter()).cloned().collect();
    assert!(matches!(attempt(&twice), Err(ResolveError::Catalog(_))));
}

/// Pojačanje množi težinu točno (M6): jedno mjesto, dva predloška; pojačani (×3) dolazi u ¾ slučajeva.
#[test]
fn boost_multiplies_weight_exactly() {
    let (cat, mut set) = content();
    let plain = general("P-OBICNO", Kind::Request, OFTEN);
    let mut boosted = general("P-POJACANO", Kind::Request, OFTEN);
    boosted.department = DeptTarget::One(DeptCode::rooms());
    boosted.chance.boost = vec![Boost {
        when: Conditions {
            dept_condition_pts: bounds(None, Some(4_000)),
            ..Conditions::default()
        },
        times: 3,
    }];
    set.templates = vec![plain, boosted];
    let t06 = season_periods(PeriodKind::Week)[5];
    let share = |condition: i32| -> u32 {
        let mut state: HotelState = start(&cat, Profile::C);
        state
            .departments
            .get_mut(&DeptCode::rooms())
            .unwrap()
            .condition_pts = condition;
        let n = sim_hotel_v2::situations::situation_count(&state, &t06, &cat).unwrap();
        let anchors: Vec<Anchor> = [Kind::Staff, Kind::Complaint, Kind::Crisis, Kind::Breakdown]
            .into_iter()
            .take(usize::from(n - 1))
            .map(|kind| Anchor {
                kind,
                department: None,
            })
            .collect();
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        (0..4_000)
            .map(|seed| {
                let d = round(&cat, &set, &h, &t06, &[], seed, &anchors);
                assert_eq!(d.len(), 1);
                u32::from(d[0].template == "P-POJACANO")
            })
            .sum()
    };
    let boosted = share(3_500);
    assert!((2_850..=3_150).contains(&boosted), "×3 → ¾: {boosted}");
    let even = share(6_000);
    assert!((1_850..=2_150).contains(&even), "×1 → ½: {even}");
}
