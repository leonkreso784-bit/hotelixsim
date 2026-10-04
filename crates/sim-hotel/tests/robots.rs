mod common;
use common::*;
use engine::Rng;
use sim_hotel::*;

#[test]
fn robot_plans_in_june_from_profile_c() {
    let cat = test_catalog();
    let s = state_c();
    let mut rng = Rng::from_seed(1);
    let p = |st: Strategy, rng: &mut Rng| st.plan(&s, Month::Jun, &cat, rng).unwrap();
    let saver = p(Strategy::Saver, &mut rng);
    assert_eq!(
        (
            saver.price_cents,
            saver.seasonal_target,
            saver.wage_pct,
            saver.maintenance
        ),
        (13_500, 15, 95, Maintenance::Deferred)
    );
    let caring = p(Strategy::Caring, &mut rng);
    assert_eq!(
        (
            caring.price_cents,
            caring.marketing_cents,
            caring.seasonal_target,
            caring.wage_pct,
            caring.training
        ),
        (14_200, 500_000, 18, 103, Training::Basic)
    );
    let hunter = p(Strategy::OccupancyHunter, &mut rng);
    assert_eq!(
        (
            hunter.price_cents,
            hunter.marketing_cents,
            hunter.seasonal_target
        ),
        (11_500, 2_500_000, 20)
    );
    let aug = Strategy::Caring
        .plan(&s, Month::Aug, &cat, &mut rng)
        .unwrap();
    assert_eq!(aug.seasonal_target, 26); // ceil(120 · 8 300 / 18 000) = 56 − 30
}

#[test]
fn random_robot_plans_are_valid() {
    let cat = test_catalog();
    let s = state_c();
    for seed in 0..200 {
        let mut rng = Rng::derive(seed, &[]);
        for m in Month::SEASON {
            let p = Strategy::Random.plan(&s, m, &cat, &mut rng).unwrap();
            assert_eq!(
                p.validate(&s, m, &cat),
                Ok(()),
                "sjeme {seed}, {m:?}: {p:?}"
            );
        }
    }
}

fn s1() -> Vec<OfferedAnswer> {
    let a = |q: u8, comp: i64, urgent: bool| OfferedAnswer {
        label: format!("q{q}"),
        quality: q,
        mechanics: vec![
            Mechanic::ServiceRecovery {
                compensation_cents: comp,
            },
            Mechanic::EquipmentRepair { urgent },
        ],
    };
    vec![
        a(5, 12_000, true),
        a(3, 0, true),
        a(1, 94_500, false),
        a(1, 0, false),
    ]
}

#[test]
fn robots_choose_answers_by_their_rule() {
    let cat = test_catalog();
    let mut rng = Rng::from_seed(3);
    let answers = s1();
    let pick = |st: Strategy, rng: &mut Rng| {
        let i = st.choose(&answers, &cat, Category::Four, rng).unwrap();
        answers[i].label.clone()
    };
    assert_eq!(pick(Strategy::Saver, &mut rng), "q1"); // 8 000 centi, najjeftiniji
    assert_eq!(
        Strategy::Saver.choose(&answers, &cat, Category::Four, &mut rng),
        Ok(3)
    );
    assert_eq!(pick(Strategy::Caring, &mut rng), "q5");
    assert_eq!(pick(Strategy::OccupancyHunter, &mut rng), "q3"); // srednji po q: [q1, q1, q3, q5][2]
}

#[test]
fn hunter_accepts_group_on_agency_terms() {
    let cat = test_catalog();
    let g = |q: u8, accept: bool, disc: u16, m: Month| OfferedAnswer {
        label: format!("{q}-{disc}"),
        quality: q,
        mechanics: vec![Mechanic::GroupBooking {
            accept,
            rooms: 25,
            nights: 7,
            discount_bp: disc,
            month: m,
        }],
    };
    let answers = vec![
        g(5, true, 2_000, Month::Sep),
        g(3, true, 1_500, Month::Aug),
        g(1, true, 2_500, Month::Aug),
        g(2, false, 2_500, Month::Aug),
    ];
    let mut rng = Rng::from_seed(0);
    let i = Strategy::OccupancyHunter
        .choose(&answers, &cat, Category::Four, &mut rng)
        .unwrap();
    assert_eq!(answers[i].label, "1-2500");
}

#[test]
fn season_is_reproducible() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let sits = vec![SituationTemplate {
        code: "S1".into(),
        month: Month::Jun,
        title: "Klima".into(),
        answers: s1(),
    }];
    for st in Strategy::ALL {
        let a = run_season(Profile::A, st, 99, &cat, &sits, &refs).unwrap();
        let b = run_season(Profile::A, st, 99, &cat, &sits, &refs).unwrap();
        assert_eq!(a, b, "{st:?}");
        assert_eq!(a.months.len(), 4);
    }
}

#[test]
fn passive_profile_c_season_scores_zero() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let r = run_season_with(
        Profile::C,
        &cat,
        &[],
        &refs,
        &mut |s, m| Ok(MonthPlan::default_for(s, m, &cat)?),
        &mut |_| Ok(0),
    )
    .unwrap();
    assert_eq!(r.start, IndexSet::from_hundredths([5_000; 5]).unwrap());
    assert_eq!(r.progress_pts, 0);
    assert_eq!(r.months.len(), 4);
}

#[test]
fn fixed_robot_plans_are_valid_from_every_profile() {
    let cat = test_catalog();
    for p in [Profile::A, Profile::B, Profile::C] {
        let s = HotelState::from_profile(cat.profile(p).unwrap()).unwrap();
        for st in [Strategy::Saver, Strategy::Caring, Strategy::OccupancyHunter] {
            let mut rng = Rng::from_seed(1);
            for m in Month::SEASON {
                let plan = st.plan(&s, m, &cat, &mut rng).unwrap();
                assert_eq!(plan.validate(&s, m, &cat), Ok(()), "{p:?} {st:?} {m:?}");
            }
        }
    }
}

#[test]
fn choose_on_empty_answers_is_an_error() {
    let cat = test_catalog();
    let mut rng = Rng::from_seed(0);
    for st in Strategy::ALL {
        assert_eq!(
            st.choose(&[], &cat, Category::Four, &mut rng),
            Err(ResolveError::BadChoice),
            "{st:?}"
        );
    }
}

#[test]
fn random_seasons_differ_between_seeds() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let a = run_season(Profile::C, Strategy::Random, 1, &cat, &[], &refs).unwrap();
    let b = run_season(Profile::C, Strategy::Random, 2, &cat, &[], &refs).unwrap();
    assert_ne!(a, b);
}
