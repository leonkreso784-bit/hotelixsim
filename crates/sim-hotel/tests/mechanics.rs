mod common;
use common::*;
use sim_hotel::*;

fn out(code: &str, q: u8, mechanics: Vec<Mechanic>) -> SituationOutcome {
    SituationOutcome {
        code: code.into(),
        mechanics,
        quality: q,
    }
}

#[test]
fn june_with_s1_s2_s3_like_outcomes() {
    let cat = test_catalog();
    let s = state_c();
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let sits = [
        out(
            "S1",
            5,
            vec![Mechanic::ServiceRecovery {
                compensation_cents: 12_000,
            }],
        ),
        out(
            "S2",
            2,
            vec![Mechanic::StaffRetention {
                raise_one_pct: 0,
                bonus_cents: 0,
            }],
        ),
        out(
            "S3",
            3,
            vec![Mechanic::GroupBooking {
                accept: true,
                rooms: 25,
                nights: 7,
                discount_bp: 2_500,
                month: Month::Jun,
            }],
        ),
    ];
    let r = resolve_month(&s, Month::Jun, &plan, &sits, &cat).unwrap();
    assert_eq!(
        (r.market.sold, r.market.group_sold, r.market.individual_sold),
        (2_767, 175, 2_592)
    );
    assert_eq!(
        (
            r.money.rooms_revenue_cents,
            r.money.other_revenue_cents,
            r.money.costs_cents
        ),
        (36_763_875, 11_029_163, 30_899_400)
    );
    assert_eq!(
        (r.money.gop_cents, r.money.goppar_cents),
        (16_893_638, 4_693)
    );
    assert_eq!(
        (
            r.people.load_hundredths,
            r.people.load_penalty_pts,
            r.people.morale_target_pts,
            r.people.turnover_bp
        ),
        (184, 400, 5_000, 398)
    );
    assert_eq!(
        (r.after.satisfaction_pts, r.after.morale_pts, r.after.rating),
        (7_221, 5_350, 842)
    );
    assert!(
        r.lines
            .iter()
            .any(|l| l.cause == Cause::Situation("S1".into()) && l.var == Var::Satisfaction)
    );
    assert_lines_explain(&r);
}

#[test]
fn group_booked_in_june_sells_in_august_with_good_quality_discount_cut() {
    let cat = test_catalog();
    let s = state_c();
    let sits = [
        out(
            "S3",
            4,
            vec![Mechanic::GroupBooking {
                accept: true,
                rooms: 25,
                nights: 7,
                discount_bp: 2_500,
                month: Month::Aug,
            }],
        ),
        out(
            "S2",
            5,
            vec![Mechanic::StaffRetention {
                raise_one_pct: 8,
                bonus_cents: 0,
            }],
        ),
    ];
    let june = resolve_month(
        &s,
        Month::Jun,
        &MonthPlan::default_for(&s, Month::Jun, &cat).unwrap(),
        &sits,
        &cat,
    )
    .unwrap();
    assert_lines_explain(&june);
    assert_eq!(
        june.after.groups,
        vec![GroupBlock {
            month: Month::Aug,
            room_nights: 175,
            price_cents: 14_000
        }]
    );
    assert_eq!(
        (june.after.extra_wage_cents, june.after.morale_pts),
        (17_600, 5_680)
    );
    let july = resolve_month(
        &june.after,
        Month::Jul,
        &MonthPlan::default_for(&june.after, Month::Jul, &cat).unwrap(),
        &[],
        &cat,
    )
    .unwrap();
    let aug = resolve_month(
        &july.after,
        Month::Aug,
        &MonthPlan::default_for(&july.after, Month::Aug, &cat).unwrap(),
        &[],
        &cat,
    )
    .unwrap();
    assert_eq!(
        (
            aug.market.group_room_nights,
            aug.market.sold,
            aug.market.individual_sold,
            aug.market.f_reputation_bp
        ),
        (175, 3_258, 3_083, 9_984)
    );
    assert_eq!(
        (
            aug.money.rooms_revenue_cents,
            aug.money.labor_cents,
            aug.money.gop_cents
        ),
        (56_402_500, 11_317_600, 41_338_050)
    );
    assert!(aug.after.groups.is_empty());
}

#[test]
fn situation_bounds_cap_effects() {
    let cat = test_catalog();
    let e = effects(
        &out(
            "X",
            5,
            vec![
                Mechanic::ServiceRecovery {
                    compensation_cents: 0,
                },
                Mechanic::CrisisCommunication { backup_cents: 0 },
            ],
        ),
        Month::Jun,
        &cat,
        Category::Four,
    )
    .unwrap();
    assert_eq!(e.satisfaction_pts, 800); // 400 + 600 -> granica 800
}

#[test]
fn bad_quality_rejected() {
    let cat = test_catalog();
    for q in [0_u8, 6] {
        let r = effects(
            &out(
                "X",
                q,
                vec![Mechanic::ServiceRecovery {
                    compensation_cents: 0,
                }],
            ),
            Month::Jun,
            &cat,
            Category::Four,
        );
        assert_eq!(r, Err(MechanicError::BadQuality(q)));
    }
}

#[test]
fn group_in_past_rejected() {
    let cat = test_catalog();
    let r = effects(
        &out(
            "S3",
            3,
            vec![Mechanic::GroupBooking {
                accept: true,
                rooms: 25,
                nights: 7,
                discount_bp: 2_500,
                month: Month::Jun,
            }],
        ),
        Month::Jul,
        &cat,
        Category::Four,
    );
    assert_eq!(r, Err(MechanicError::GroupInPast));
}

#[test]
fn rejected_group_has_no_effect() {
    let cat = test_catalog();
    let e = effects(
        &out(
            "S3",
            2,
            vec![Mechanic::GroupBooking {
                accept: false,
                rooms: 25,
                nights: 7,
                discount_bp: 2_500,
                month: Month::Aug,
            }],
        ),
        Month::Jun,
        &cat,
        Category::Four,
    )
    .unwrap();
    assert_eq!(e, Effects::default());
}

#[test]
fn temporary_repair_is_cheaper_but_wears_equipment() {
    let cat = test_catalog();
    let e = effects(
        &out("S1", 3, vec![Mechanic::EquipmentRepair { urgent: false }]),
        Month::Jun,
        &cat,
        Category::Four,
    )
    .unwrap();
    assert_eq!((e.cost_cents, e.equipment_pts), (8_000, -100));
    let u = effects(
        &out("S1", 3, vec![Mechanic::EquipmentRepair { urgent: true }]),
        Month::Jun,
        &cat,
        Category::Four,
    )
    .unwrap();
    assert_eq!((u.cost_cents, u.equipment_pts), (25_000, 0));
}

#[test]
fn direct_cost_for_robots() {
    let cat = test_catalog();
    let c = |m: Mechanic| m.direct_cost_cents(&cat, Category::Four).unwrap();
    assert_eq!(
        c(Mechanic::StaffRetention {
            raise_one_pct: 8,
            bonus_cents: 200_000
        }),
        217_600
    );
    assert_eq!(
        c(Mechanic::GroupBooking {
            accept: true,
            rooms: 25,
            nights: 7,
            discount_bp: 2_500,
            month: Month::Aug
        }),
        0
    );
}

fn bad_mechanics() -> Vec<(&'static str, Mechanic)> {
    vec![
        (
            "compensation_cents",
            Mechanic::ServiceRecovery {
                compensation_cents: -1,
            },
        ),
        (
            "bonus_cents",
            Mechanic::StaffRetention {
                raise_one_pct: 0,
                bonus_cents: -1,
            },
        ),
        (
            "raise_one_pct",
            Mechanic::StaffRetention {
                raise_one_pct: 101,
                bonus_cents: 0,
            },
        ),
        (
            "backup_cents",
            Mechanic::CrisisCommunication { backup_cents: -1 },
        ),
        (
            "discount_bp",
            Mechanic::GroupBooking {
                accept: true,
                rooms: 25,
                nights: 7,
                discount_bp: 10_001,
                month: Month::Aug,
            },
        ),
    ]
}

#[test]
fn bad_parameters_rejected_with_field_name() {
    let cat = test_catalog();
    for (field, mech) in bad_mechanics() {
        let r = effects(&out("X", 3, vec![mech]), Month::Jun, &cat, Category::Four);
        assert_eq!(r, Err(MechanicError::BadParameter(field)), "{field}");
    }
}

#[test]
fn resolve_month_propagates_bad_parameter() {
    let cat = test_catalog();
    let s = state_c();
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    for (field, mech) in bad_mechanics() {
        let r = resolve_month(&s, Month::Jun, &plan, &[out("X", 3, vec![mech])], &cat);
        assert_eq!(
            r.err(),
            Some(ResolveError::Mechanic(MechanicError::BadParameter(field))),
            "{field}"
        );
    }
}

#[test]
fn unanswered_is_quality_one_with_template_mechanics() {
    let cat = test_catalog();
    let t = SituationTemplate {
        code: "S1".into(),
        month: Month::Jun,
        title: "Klima".into(),
        answers: vec![],
        unanswered: vec![
            Mechanic::ServiceRecovery {
                compensation_cents: 0,
            },
            Mechanic::EquipmentRepair { urgent: false },
        ],
    };
    let o = SituationOutcome::unanswered(&t);
    assert_eq!(o.code, "S1");
    assert_eq!(o.quality, 1);
    assert_eq!(o.mechanics, t.unanswered);
    let e = effects(&o, Month::Jun, &cat, Category::Four).unwrap();
    // DEMO §6.2: zadovoljstvo Δ = 2 × (q − 3) bodova = −4,00 → −400 stotinki
    assert_eq!(e.satisfaction_pts, cat.mechanics.recovery_pts_per_q * -2);
    assert_eq!(e.satisfaction_pts, -400);
}
