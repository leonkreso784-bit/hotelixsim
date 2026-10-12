//! Cijene odluka u granicama plana (Leon 12.10.: obnova je koštala 150.000 € bez riječi na ekranu). Brojke su iz
//! kataloga (`content/hotel/catalog.json`, `content/hotel/v2/departments.json`), za krug kako ih knjiži obračun.
mod common;

#[test]
fn v1_plan_costs_are_the_monthly_catalog_amounts() {
    let (g, h) = common::open_june_game();
    let c = g.private_hotel(h).unwrap().plan.unwrap().limits.costs;
    assert_eq!(c.renovation_cents, 15_000_000);
    assert_eq!(c.renovation_equipment_pts, 2_500);
    assert_eq!(c.renovation_satisfaction_pts, 300);
    assert_eq!(c.reply_cents, 150_000);
    assert_eq!(c.training_basic.cost_cents, 500_000);
    assert_eq!(c.training_full.experience_pts, 700);
    assert_eq!(
        (
            c.maintenance.deferred_cents,
            c.maintenance.regular_cents,
            c.maintenance.intensive_cents
        ),
        (300_000, 1_500_000, 3_000_000)
    );
    assert!(c.payroll_week_cents > 0);
    // Mjesec je krug: plaće kruga su mjesečne, tjedne su 7/30 njih (lipanj, zaokruženo).
    assert_eq!(c.payroll_week_cents, (c.payroll_round_cents * 7 + 15) / 30);
}

#[test]
fn v2_weekly_costs_are_prorated_to_seven_of_thirty_days_and_renovation_is_full() {
    let g = common::v2_weekly_game_resolved_through("none");
    let l = g
        .private_hotel(common::hotel_ids()[0])
        .unwrap()
        .plan
        .unwrap()
        .limits;
    let c = &l.costs;
    // t01 = 1.–7. lipnja: 7/30 mjesečnog iznosa.
    assert_eq!(c.renovation_cents, 15_000_000);
    assert_eq!(c.reply_cents, 35_000);
    assert_eq!(c.training_basic.cost_cents, 116_666);
    assert_eq!(c.maintenance.regular_cents, 350_000);
    // Restoran: redovno održavanje 4.000 €/mjesec → 933,33 € za tjedan.
    assert_eq!(
        l.departments["restaurant"].maintenance.regular_cents,
        93_333
    );
    assert_eq!(l.departments["rooms"].maintenance, c.maintenance);
    assert!(c.payroll_week_cents > 0);
    // t01 ima 7 dana: plaće kruga i tjedna procjena razlikuju se najviše za zaokruživanje (1 cent).
    assert!((c.payroll_round_cents - c.payroll_week_cents).abs() <= 1);
    // Restoran: plaća 2.100 €/mjesec pri 100 % → 490 € za tjedan (7/30).
    assert_eq!(l.departments["restaurant"].seasonal_worker_cents, 49_000);
}

#[test]
fn v2_monthly_costs_are_full_month_and_match_what_resolution_books() {
    let h = common::hotel_ids()[0];
    let g = common::v2_monthly_game_resolved_through("none");
    let l = g.private_hotel(h).unwrap().plan.unwrap().limits;
    // Lipanj je cijeli krug: plaća jednog sezonskog radnika Restorana je mjesečna (2.100 € pri 100 %).
    assert_eq!(l.departments["restaurant"].seasonal_worker_cents, 210_000);
    // Plaće kruga = ljudi svakog odjela (otvoreni: stalni + sezonski) puta plaća jednog radnika tog odjela.
    let before = g.departments(h).unwrap();
    let sum: i64 = before
        .departments
        .iter()
        .map(|d| {
            let staff = i64::from(d.permanent) + if d.open { i64::from(d.seasonal) } else { 0 };
            staff * l.departments[&d.code].seasonal_worker_cents
        })
        .sum();
    assert_eq!(l.costs.payroll_round_cents, sum);

    // Obračun lipnja knjiži za svakog zaposlenog odjela upravo tu plaću (plan ne mijenja postotak plaće).
    let resolved = common::v2_monthly_game_resolved_through("jun");
    let mut checked = 0;
    for d in resolved.departments(h).unwrap().departments {
        let (staff, payroll) = match (&d.last, &d.undistributed_last) {
            (Some((_, o)), _) => (o.staff, o.payroll_cents),
            (None, Some((_, u))) => (u.staff, u.payroll_cents),
            (None, None) => continue,
        };
        assert_eq!(
            payroll,
            i64::from(staff) * l.departments[&d.code].seasonal_worker_cents,
            "{}",
            d.code
        );
        checked += 1;
    }
    assert!(checked >= 10, "provjereno samo {checked} odjela");
}
