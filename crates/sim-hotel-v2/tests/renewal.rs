// Obnova v2 (O-1): krivulja, raspored plaćanja, amortizacija, zgrada, granica 100, objašnjenje, indeks F.
mod common;

use sim_hotel_v2::calendar::{
    PeriodKind, is_last_round_of_month, month_period, period_by_code, prorate_part,
    rounds_left_in_month, season_periods,
};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::renewal::{AssetBook, depreciation, gain_pts, renew_round};
use sim_hotel_v2::usali::{statement, sum_statements};
use sim_hotel_v2::{
    Catalog, FinanceBasis, HotelState, Month, Period, PeriodPlan, Profile, RefTable, Strategy,
    period_indices, resolve_period, run_season, run_season_with, start_indices,
};

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

fn eur(x: i64) -> i64 {
    x * 100
}

fn week(c: &str) -> Period {
    period_by_code(PeriodKind::Week, c).unwrap()
}

fn spec(c: &Catalog, d: &str) -> sim_hotel_v2::RenewalSpec {
    c.dept(&code(d)).unwrap().renewal.clone()
}

// ------------------------------------------------------------------ krivulja (F3)

#[test]
fn curve_matches_the_table_in_whole_numbers() {
    let c = common::catalog_with_renewal();
    let building = c.economy.building.renewal.clone();
    let cases = [
        ("rooms", 10_000, 313),
        ("rooms", 20_000, 588),
        ("rooms", 50_000, 1_250),
        ("rooms", 100_000, 2_000),
        ("rooms", 150_000, 2_500),
        ("restaurant", 10_000, 769),
        ("restaurant", 20_000, 1_333),
        ("restaurant", 40_000, 2_105),
        ("restaurant", 50_000, 2_381),
        ("lobby_bar", 10_000, 1_429),
        ("lobby_bar", 50_000, 3_333),
        ("spa", 10_000, 1_000),
        ("spa", 50_000, 2_778),
        ("parking", 10_000, 714),
        ("parking", 50_000, 2_273),
    ];
    for (d, a, e) in cases {
        assert_eq!(gain_pts(&spec(&c, d), eur(a)).unwrap(), e, "{d} {a}");
    }
    assert_eq!(gain_pts(&building, eur(200_000)).unwrap(), 2_857);
    assert_eq!(gain_pts(&building, eur(10_000)).unwrap(), 313);
    // Dvije faze po 20.000 € u restoranu daju više od 40.000 € u jednom mjesecu.
    let r = spec(&c, "restaurant");
    assert_eq!(2 * gain_pts(&r, eur(20_000)).unwrap(), 2_666);
}

#[test]
fn curve_is_half_at_k_grows_and_never_reaches_g() {
    let c = common::catalog_with_renewal();
    for d in ["rooms", "restaurant", "lobby_bar", "spa", "parking"] {
        let s = spec(&c, d);
        assert_eq!(gain_pts(&s, 0).unwrap(), 0);
        assert_eq!(gain_pts(&s, s.half_cents).unwrap(), s.gain_pts / 2, "{d}");
        let mut last = 0;
        for k in 1..=200_i64 {
            let e = gain_pts(&s, k * eur(5_000)).unwrap();
            assert!(e >= last && e < s.gain_pts, "{d} {k}");
            last = e;
        }
        assert!(gain_pts(&s, eur(100_000_000)).unwrap() < s.gain_pts);
    }
}

// ------------------------------------------------------------------ raspored plaćanja (F4, F5)

#[test]
fn rounds_left_in_month_follow_the_calendar() {
    let left: Vec<u8> = season_periods(PeriodKind::Week)
        .iter()
        .map(rounds_left_in_month)
        .collect();
    assert_eq!(left, [5, 4, 3, 2, 1, 4, 3, 2, 1, 5, 4, 3, 2, 1, 3, 2, 1]);
    for p in season_periods(PeriodKind::Month) {
        assert_eq!(rounds_left_in_month(&p), 1);
        assert!(is_last_round_of_month(&p));
    }
    assert!(is_last_round_of_month(&week("t05")) && !is_last_round_of_month(&week("t04")));
}

/// Krugovi istog mjeseca od kruga odluke do kraja mjeseca.
fn rest_of_month(p: &Period) -> Vec<Period> {
    season_periods(p.kind)
        .into_iter()
        .filter(|q| q.index >= p.index && q.start_month() == p.start_month())
        .collect()
}

#[test]
fn payments_add_up_to_the_amount_for_every_decision_round() {
    let c = common::catalog_with_renewal();
    let s = spec(&c, "rooms");
    for kind in PeriodKind::ALL {
        for p in season_periods(kind) {
            for k in 1..=40_i64 {
                let a = k * eur(5_000);
                let mut book = AssetBook::default();
                let (mut paid, mut gained, mut pays) = (0_i64, 0_i32, vec![]);
                for (i, q) in rest_of_month(&p).iter().enumerate() {
                    // Odluka vrijedi mjesec: u kasnijim krugovima plan nosi isti iznos.
                    let r = renew_round(&mut book, &s, a, q).unwrap();
                    paid += r.cents;
                    gained += r.delta_pts;
                    pays.push(r.cents);
                    let locked = book.locked_in(q.start_month()).unwrap();
                    assert_eq!(locked.paid_cents, paid, "{} {a} {i}", q.code());
                }
                assert_eq!(paid, a, "{} {a}", p.code());
                assert_eq!(gained, gain_pts(&s, a).unwrap(), "{} {a}", p.code());
                let n = i64::try_from(pays.len()).unwrap();
                assert_eq!(n, i64::from(rounds_left_in_month(&p)));
                let installment = a / n;
                assert!(pays[..pays.len() - 1].iter().all(|x| *x == installment));
                assert_eq!(*pays.last().unwrap(), a - installment * (n - 1));
                if kind == PeriodKind::Month {
                    assert_eq!(pays, [a]);
                }
            }
        }
    }
}

#[test]
fn restaurant_example_from_the_spec() {
    let c = common::catalog_with_renewal();
    let s = spec(&c, "restaurant");
    let pays = |from: &str| {
        let mut book = AssetBook::default();
        rest_of_month(&week(from))
            .iter()
            .map(|q| renew_round(&mut book, &s, eur(40_000), q).unwrap().cents)
            .collect::<Vec<_>>()
    };
    assert_eq!(pays("t06"), [eur(10_000); 4]);
    assert_eq!(pays("t07"), [1_333_333, 1_333_333, 1_333_334]);
}

#[test]
fn book_rolls_into_capitalized_in_the_next_month() {
    let c = common::catalog_with_renewal();
    let s = spec(&c, "rooms");
    let mut book = AssetBook::default();
    renew_round(&mut book, &s, eur(50_000), &month_period(Month::Jun)).unwrap();
    assert_eq!(book.invested_cents(), eur(50_000));
    assert_eq!(book.capitalized_cents, 0);
    let r = renew_round(&mut book, &s, 0, &month_period(Month::Jul)).unwrap();
    assert_eq!((r.cents, r.delta_pts), (0, 0));
    assert!(book.renewal.is_none());
    assert_eq!(book.capitalized_cents, eur(50_000));
    assert_eq!(book.invested_cents(), eur(50_000));
}

// ------------------------------------------------------------------ amortizacija (F9, O13)

#[test]
fn depreciation_starts_the_month_after_the_renewal() {
    let c = common::catalog_with_renewal();
    let s = spec(&c, "rooms");
    let monthly = (eur(150_000) + 42) / 84; // div_round(15 000 000, 84) = 178 571
    assert_eq!(monthly, 178_571);
    // Tjedna igra: 150.000 € odlučeno u t01 (lipanj).
    let mut book = AssetBook::default();
    let mut by_round = vec![];
    for p in season_periods(PeriodKind::Week) {
        let a = if p.start_month() == Month::Jun {
            eur(150_000)
        } else {
            0
        };
        renew_round(&mut book, &s, a, &p).unwrap();
        by_round.push((p, depreciation(&book, s.life_months, &p).unwrap()));
    }
    for (p, d) in &by_round[..4] {
        assert_eq!(*d, 0, "{}", p.code());
    }
    // t05 (29. 6. – 5. 7.): srpanjski dio (5 dana) već nosi lipanjsku obnovu.
    let t05 = &by_round[4];
    let july_part = t05.0.parts()[1];
    assert_eq!(july_part.month, Month::Jul);
    assert_eq!(t05.1, prorate_part(monthly, &july_part).unwrap());
    // Mjesečna igra: isti mjesečni iznos od srpnja; tjedno po mjesecu = zbroj dijelova krugova u tom mjesecu.
    let mut month_book = AssetBook::default();
    for m in [Month::Jun, Month::Jul, Month::Aug, Month::Sep] {
        let mp = month_period(m);
        let a = if m == Month::Jun { eur(150_000) } else { 0 };
        renew_round(&mut month_book, &s, a, &mp).unwrap();
        let expected = if m == Month::Jun { 0 } else { monthly };
        assert_eq!(
            depreciation(&month_book, s.life_months, &mp).unwrap(),
            expected,
            "{m:?}"
        );
    }
    // Krugovi koji ne prelaze mjesec: zbroj kruga = zbroj njegovih dijelova (osnovica stalna u mjesecu).
    for (p, d) in &by_round[5..] {
        let parts: i64 = p
            .parts()
            .iter()
            .map(|part| prorate_part(monthly, part).unwrap())
            .sum();
        assert_eq!(*d, parts, "{}", p.code());
    }
    let season_weekly: i64 = by_round.iter().map(|(_, d)| d).sum();
    assert_eq!(season_weekly, 3 * monthly);
}

#[test]
fn september_renewal_is_not_depreciated_in_the_season() {
    let c = common::catalog_with_renewal();
    let s = spec(&c, "rooms");
    for kind in PeriodKind::ALL {
        let mut book = AssetBook::default();
        let mut total = 0;
        for p in season_periods(kind) {
            let a = if p.start_month() == Month::Sep {
                eur(150_000)
            } else {
                0
            };
            renew_round(&mut book, &s, a, &p).unwrap();
            total += depreciation(&book, s.life_months, &p).unwrap();
        }
        assert_eq!(total, 0, "{kind:?}");
        assert_eq!(book.invested_cents(), eur(150_000));
    }
}

// ------------------------------------------------------------------ obračun kruga s obnovom

/// Zadani plan profila u krugu s iznosima obnove (odjel → €) i obnovom zgrade (€).
fn plan_with(
    s: &HotelState,
    p: &Period,
    c: &Catalog,
    depts: &[(&str, i64)],
    building: i64,
) -> PeriodPlan {
    let mut plan = common::hold_plan(s, p, c);
    for (d, a) in depts {
        plan.departments.get_mut(&code(d)).unwrap().renewal_cents = eur(*a);
    }
    plan.building_renewal_cents = eur(building);
    plan
}

#[test]
fn renewal_raises_condition_and_is_capex_not_cost() {
    let c = common::catalog_with_renewal();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let plain = resolve_period(&s, &p, &plan_with(&s, &p, &c, &[], 0), &[], &c).unwrap();
    let plan = plan_with(
        &s,
        &p,
        &c,
        &[("restaurant", 40_000), ("rooms", 50_000)],
        100_000,
    );
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    let rest = &r.accounts.departments[&code("restaurant")];
    let rest0 = &plain.accounts.departments[&code("restaurant")];
    assert_eq!((rest.renewal_cents, rest.renewal_pts), (eur(40_000), 2_105));
    assert_eq!(rest.condition_after_pts, rest0.condition_after_pts + 2_105);
    let rooms = &r.accounts.departments[&DeptCode::rooms()];
    let rooms0 = &plain.accounts.departments[&DeptCode::rooms()];
    assert_eq!(
        (rooms.renewal_cents, rooms.renewal_pts),
        (eur(50_000), 1_250)
    );
    assert_eq!(
        rooms.condition_after_pts,
        rooms0.condition_after_pts + 1_250
    );
    let b = &r.accounts.building;
    assert_eq!(
        (b.before_pts, b.wear_pts, b.renewal_pts, b.renewal_cents),
        (6_500, -50, 2_000, eur(100_000))
    );
    assert_eq!(b.after_pts, 6_500 - 50 + 2_000);
    assert_eq!(r.after.building.condition_pts, b.after_pts);
    // Capex: ne ulazi u GOP, ide iz blagajne.
    let capex = eur(40_000 + 50_000 + 100_000);
    assert_eq!(r.accounts.capex_cents, capex);
    assert_eq!(r.summary.money.capex_cents, capex);
    assert_eq!(r.summary.money.costs_cents, plain.summary.money.costs_cents);
    let gop_diff = r.summary.money.gop_cents - plain.summary.money.gop_cents;
    assert_eq!(
        r.after.cash_cents - plain.after.cash_cents,
        gop_diff - capex
    );
    // Obnova u lipnju se ne amortizira u lipnju.
    assert_eq!(r.accounts.depreciation_equipment_cents, 0);
    assert_eq!(r.accounts.depreciation_building_cents, 0);
    assert_eq!(r.finance.depreciation_cents, 0);
    // Knjige: zaključano za lipanj.
    let locked = r.after.departments[&code("restaurant")]
        .book
        .locked_in(Month::Jun)
        .unwrap();
    assert_eq!(
        (locked.total_cents, locked.paid_cents),
        (eur(40_000), eur(40_000))
    );
}

#[test]
fn capped_at_100_counts_only_what_was_gained() {
    let c = common::catalog_with_renewal();
    let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
    s.departments
        .get_mut(&code("restaurant"))
        .unwrap()
        .condition_pts = 9_800;
    let p = month_period(Month::Jun);
    let plain = resolve_period(&s, &p, &plan_with(&s, &p, &c, &[], 0), &[], &c).unwrap();
    let r = resolve_period(
        &s,
        &p,
        &plan_with(&s, &p, &c, &[("restaurant", 50_000)], 0),
        &[],
        &c,
    )
    .unwrap();
    let o = &r.accounts.departments[&code("restaurant")];
    let base = plain.accounts.departments[&code("restaurant")].condition_after_pts;
    assert_eq!(base, 9_800); // redovno održavanje pokriva trošenje u testnom katalogu
    assert_eq!(o.condition_after_pts, 10_000);
    assert_eq!(o.renewal_pts, 10_000 - base);
    // Bez obnove (Δ = 0) stanje je točno kao u katalogu bez obnove.
    let no_renewal = common::catalog_with_departments();
    let s0 = HotelState::from_profile(&no_renewal, Profile::C).unwrap();
    let r0 = resolve_period(
        &s0,
        &p,
        &common::hold_plan(&s0, &p, &no_renewal),
        &[],
        &no_renewal,
    )
    .unwrap();
    let c_plain = HotelState::from_profile(&c, Profile::C).unwrap();
    let r1 = resolve_period(&c_plain, &p, &common::hold_plan(&c_plain, &p, &c), &[], &c).unwrap();
    for (k, d) in &r0.accounts.departments {
        let d1 = &r1.accounts.departments[k];
        assert_eq!(
            d.condition_after_pts,
            d1.condition_after_pts,
            "{}",
            k.as_str()
        );
        assert_eq!(d1.renewal_pts, 0);
    }
}

#[test]
fn explanation_adds_up_with_renewal_of_other_departments() {
    let c = common::catalog_with_renewal();
    for profile in [Profile::A, Profile::C] {
        let s = HotelState::from_profile(&c, profile).unwrap();
        for p in [month_period(Month::Jun), week("t01")] {
            let plan = plan_with(
                &s,
                &p,
                &c,
                &[
                    ("restaurant", 20_000),
                    ("lobby_bar", 10_000),
                    ("rooms", 30_000),
                ],
                50_000,
            );
            let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
            common::assert_lines_explain(&r.summary);
            let cash_renovation: i64 = r
                .summary
                .lines
                .iter()
                .filter(|l| {
                    l.var == sim_hotel_v2::Var::Cash && l.cause == sim_hotel_v2::Cause::Renovation
                })
                .map(|l| l.amount)
                .sum();
            assert_eq!(cash_renovation, -r.accounts.capex_cents);
            let renov: i64 = r
                .summary
                .lines
                .iter()
                .filter(|l| {
                    l.var == sim_hotel_v2::Var::Equipment
                        && l.cause == sim_hotel_v2::Cause::Renovation
                })
                .map(|l| l.amount)
                .sum();
            assert!(renov > 0, "{profile:?} {}", p.code());
        }
    }
}

// ------------------------------------------------------------------ Smještaj čita svoj odjel (§3.1)

#[test]
fn rooms_read_their_department_spec() {
    let c = common::catalog_with_departments();
    let mut worn = c.clone();
    worn.departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .wear_pts_per_month = 800;
    let mut cheap = c.clone();
    cheap
        .departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .maintenance[1]
        .equipment_pts = 100;
    cheap
        .departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .maintenance[1]
        .cost_cents = 200_000;
    let p = month_period(Month::Jun);
    let run = |cat: &Catalog| {
        let s = HotelState::from_profile(cat, Profile::C).unwrap();
        resolve_period(&s, &p, &common::hold_plan(&s, &p, cat), &[], cat).unwrap()
    };
    let (a, b, d) = (run(&c), run(&worn), run(&cheap));
    let rooms = |r: &sim_hotel_v2::PeriodReport| r.accounts.departments[&DeptCode::rooms()].clone();
    assert_eq!(
        rooms(&a).condition_after_pts - rooms(&b).condition_after_pts,
        400
    );
    assert_eq!(rooms(&b).wear_pts, -800);
    assert_eq!(
        rooms(&a).condition_after_pts - rooms(&d).condition_after_pts,
        300
    );
    assert_eq!(rooms(&d).maintenance_cents, 200_000);
}

// ------------------------------------------------------------------ zgrada (F7, F8)

#[test]
fn building_wear_per_month_is_the_same_weekly_and_monthly() {
    let c = common::catalog_with_renewal();
    for kind in PeriodKind::ALL {
        let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
        let mut per_month = [0_i64; 4];
        for p in season_periods(kind) {
            let r = resolve_period(&s, &p, &common::hold_plan(&s, &p, &c), &[], &c).unwrap();
            let parts: i64 = p
                .parts()
                .iter()
                .map(|part| prorate_part(-50, part).unwrap())
                .sum();
            assert_eq!(
                i64::from(r.accounts.building.wear_pts),
                parts,
                "{}",
                p.code()
            );
            for part in p.parts() {
                per_month[part.month.position()] += prorate_part(-50, &part).unwrap();
            }
            s = r.after;
        }
        assert_eq!(per_month, [-50; 4], "{kind:?}");
        assert_eq!(s.building.condition_pts, 6_500 - 200, "{kind:?}");
    }
}

#[test]
fn building_share_zero_leaves_targets_unchanged() {
    let plain = common::catalog_with_departments();
    let mut zero = common::catalog_with_renewal();
    zero.economy.building.satisfaction_share_bp = 0;
    let p = month_period(Month::Jun);
    let run = |cat: &Catalog| {
        let s = HotelState::from_profile(cat, Profile::C).unwrap();
        resolve_period(&s, &p, &common::hold_plan(&s, &p, cat), &[], cat).unwrap()
    };
    let (a, b) = (run(&plain), run(&zero));
    assert_eq!(a.summary, b.summary);
    for (k, d) in &a.accounts.departments {
        let e = &b.accounts.departments[k];
        assert_eq!(
            d.satisfaction_target_pts,
            e.satisfaction_target_pts,
            "{}",
            k.as_str()
        );
        assert_eq!(
            e.felt_condition_pts,
            e.condition_after_pts,
            "{}",
            k.as_str()
        );
    }
}

#[test]
fn building_share_mixes_into_the_satisfaction_target() {
    let c = common::catalog_with_renewal();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let r = resolve_period(&s, &p, &common::hold_plan(&s, &p, &c), &[], &c).unwrap();
    let z = i64::from(r.accounts.building.after_pts);
    let round = |n: i64, d: i64| (2 * n + d) / (2 * d); // div_round za pozitivne
    for (k, o) in &r.accounts.departments {
        if o.guest_weight == 0 {
            continue;
        }
        let felt = round(i64::from(o.condition_after_pts) * 8_000 + z * 2_000, 10_000);
        assert_eq!(i64::from(o.felt_condition_pts), felt, "{}", k.as_str());
        let target = round(
            4_500 * i64::from(o.quality_pts) + 3_000 * felt + 2_500 * i64::from(o.value_pts),
            10_000,
        );
        assert_eq!(
            i64::from(o.satisfaction_target_pts),
            target,
            "{}",
            k.as_str()
        );
    }
}

// ------------------------------------------------------------------ račun dobiti (EBIT)

#[test]
fn statement_has_depreciation_and_ebit() {
    let c = common::catalog_with_renewal();
    let mut s = HotelState::from_profile(&c, Profile::A).unwrap();
    let mut statements = vec![];
    for p in season_periods(PeriodKind::Week) {
        let plan = if p.start_month() == Month::Jun {
            plan_with(&s, &p, &c, &[("rooms", 150_000), ("spa", 50_000)], 200_000)
        } else {
            common::hold_plan(&s, &p, &c)
        };
        let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
        let st = statement(&r.accounts).unwrap();
        assert_eq!(
            st.depreciation_cents,
            st.depreciation_equipment_cents + st.depreciation_building_cents
        );
        assert_eq!(
            st.ebit_cents,
            st.ebitda_cents - st.depreciation_cents,
            "{}",
            p.code()
        );
        assert_eq!(st.ebit_cents, r.finance.ebit_cents);
        assert_eq!(st.ebitda_cents, r.finance.ebitda_cents);
        let nights = i64::from(r.accounts.rooms) * i64::from(p.days);
        assert_eq!(
            r.finance.ebitpar_cents,
            (2 * st.ebit_cents + nights * st.ebit_cents.signum()) / (2 * nights)
        );
        statements.push(st);
        s = r.after;
    }
    let season = sum_statements(&statements).unwrap();
    let dep: i64 = statements.iter().map(|x| x.depreciation_cents).sum();
    let equip: i64 = statements
        .iter()
        .map(|x| x.depreciation_equipment_cents)
        .sum();
    // Smještaj 150.000 €/84 + wellness 50.000 €/120 kroz srpanj–rujan, zgrada 200.000 €/480.
    assert_eq!(equip, 3 * (178_571 + 41_667));
    assert_eq!(dep - equip, 3 * 41_667);
    assert_eq!(season.depreciation_cents, dep);
    assert_eq!(season.ebit_cents, season.ebitda_cents - dep);
    assert!(season.ebit_margin_bp.is_some());
}

// ------------------------------------------------------------------ indeks F (F10, F11, O12)

#[test]
fn finance_index_with_gop_basis_is_unchanged_and_with_ebit_follows_f11() {
    let mut gop = common::catalog_with_renewal();
    gop.economy.finance_index = FinanceBasis::Gop;
    let mut ebit = gop.clone();
    ebit.economy.finance_index = FinanceBasis::Ebit;
    let periods = season_periods(PeriodKind::Month);
    let category = gop.base.profile(Profile::A).unwrap().category;
    for (cat, basis) in [(&gop, FinanceBasis::Gop), (&ebit, FinanceBasis::Ebit)] {
        let refs = RefTable::compute(cat, category, &periods).unwrap();
        let mut s = HotelState::from_profile(cat, Profile::A).unwrap();
        for p in &periods {
            let plan = if p.start_month() == Month::Jun {
                plan_with(&s, p, cat, &[("rooms", 150_000)], 200_000)
            } else {
                common::hold_plan(&s, p, cat)
            };
            let r = resolve_period(&s, p, &plan, &[], cat).unwrap();
            let idx = period_indices(&r, cat, &refs).unwrap();
            let point = &refs.points[usize::from(p.index)];
            let money = &r.summary.money;
            let labor = sim_hotel_v2::calendar::per_month_equiv(money.labor_cents, p).unwrap();
            let x = match basis {
                FinanceBasis::Gop => money.goppar_cents,
                FinanceBasis::Ebit => {
                    point.goppar_cents
                        + (r.finance.ebitpar_cents - refs.ebitpar_cents[usize::from(p.index)])
                }
                FinanceBasis::GopAfterInvestment => unreachable!("samo GOP i EBIT"),
            };
            let raw =
                sim_hotel::raw_indices(&r.summary.after, x, labor, point.goppar_cents, &cat.base)
                    .unwrap();
            let expected = sim_hotel::centred(&raw, &point.raw).unwrap();
            assert_eq!(idx, expected, "{basis:?} {}", p.code());
            if basis == FinanceBasis::Ebit {
                // Ručno po F11: gi s razlikom EBITPAR-a i nazivnikom GOPPAR referentnog hotela, likvidnost kao v1.
                let round = |n: i64, d: i64| (2 * n + d * n.signum()) / (2 * d);
                let floor = cat.base.economy.goppar_floor_cents;
                let diff = r.finance.ebitpar_cents - refs.ebitpar_cents[usize::from(p.index)];
                let gi =
                    (5_000 + round(5_000 * diff, point.goppar_cents.max(floor))).clamp(0, 10_000);
                let after = &r.summary.after;
                let liq = if after.cash_cents <= 0 {
                    0
                } else {
                    round(5_000 * after.cash_cents, after.fixed_cents + labor)
                };
                assert_eq!(
                    i64::from(raw.f),
                    round(7_000 * gi + 3_000 * liq, 10_000),
                    "{}",
                    p.code()
                );
            }
            s = r.after;
        }
    }
}

#[test]
fn reference_hotel_against_itself_stays_at_50_with_ebit() {
    let mut c = common::catalog_with_renewal();
    c.economy.finance_index = FinanceBasis::Ebit;
    for kind in PeriodKind::ALL {
        let periods = season_periods(kind);
        let category = c.base.profile(Profile::C).unwrap().category;
        let refs = RefTable::compute(&c, category, &periods).unwrap();
        let s = HotelState::from_profile(&c, Profile::C).unwrap();
        let start = start_indices(&s, &c, &refs).unwrap();
        assert!(
            start.iter().all(|(_, v)| v.hundredths() == 5_000),
            "{kind:?} {start:?}"
        );
        let r = run_season_with(
            Profile::C,
            kind,
            &c,
            &[],
            &refs,
            &mut |s, p| Ok(PeriodPlan::default_for(s, p, &c)?),
            &mut |_| Ok(0),
        )
        .unwrap();
        for (rep, idx) in &r.periods {
            assert!(
                idx.iter().all(|(_, v)| v.hundredths() == 5_000),
                "{}",
                rep.period.code()
            );
        }
    }
}

// ------------------------------------------------------------------ cijela sezona

#[test]
fn weekly_season_with_renewal_runs_for_every_profile_and_strategy() {
    let c = common::catalog_with_renewal();
    for kind in PeriodKind::ALL {
        let periods = season_periods(kind);
        for profile in Profile::ALL {
            let category = c.base.profile(profile).unwrap().category;
            let refs = RefTable::compute(&c, category, &periods).unwrap();
            for strategy in Strategy::ALL {
                for seed in [1_u64, 7, 42] {
                    let r = run_season(profile, strategy, seed, kind, &c, &[], &refs);
                    assert!(r.is_ok(), "{kind:?} {profile:?} {strategy:?} {seed}: {r:?}");
                    let capex: i64 = r
                        .unwrap()
                        .periods
                        .iter()
                        .map(|(rep, _)| rep.accounts.capex_cents)
                        .sum();
                    // B: Smještaj počinje na 40, pa ga brižni i lovac obnove; štediša i nasumični nikad.
                    if profile == Profile::B {
                        let renews =
                            matches!(strategy, Strategy::Caring | Strategy::OccupancyHunter);
                        assert_eq!(capex > 0, renews, "{kind:?} {strategy:?} {seed}");
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ indeks F iz GOP-a nakon ulaganja (O-2 d′)

/// Leonova odluka (O-2 d′): brojnik F je GOP po sobi i danu umanjen za amortizaciju obnova (od mjeseca nakon obnove),
/// prema referentnom hotelu; fiksne naknade ne ulaze; nazivnik ostaje GOPPAR referentnog hotela. Na početku nema
/// amortizacije, pa su početni indeksi točno oni iz GOP-a.
#[test]
fn gop_after_investment_start_is_the_gop_start() {
    let mut gop = common::catalog_with_renewal();
    gop.economy.finance_index = FinanceBasis::Gop;
    let mut gai = gop.clone();
    gai.economy.finance_index = FinanceBasis::GopAfterInvestment;
    for kind in PeriodKind::ALL {
        let periods = season_periods(kind);
        for profile in Profile::ALL {
            let category = gop.base.profile(profile).unwrap().category;
            let start = |c: &Catalog| {
                let refs = RefTable::compute(c, category, &periods).unwrap();
                let s = HotelState::from_profile(c, profile).unwrap();
                start_indices(&s, c, &refs).unwrap()
            };
            assert_eq!(start(&gai), start(&gop), "{profile:?} {kind:?}");
        }
    }
}

/// Ručni primjer, profil A, mjesečna igra: obnova Smještaja 150.000 € u lipnju. U lipnju nema amortizacije (O13), pa
/// je F isti kao iz GOP-a. Od srpnja je amortizacija div_round(15.000.000, 84) = 178.571 centi mjesečno, po sobi i
/// danu div_round(178.571, 120 · 31) = 48 centi u srpnju; F dobiva GOPPAR − 48 centi (referentni hotel ne obnavlja).
#[test]
fn gop_after_investment_lowers_f_by_depreciation_per_room_day() {
    let mut gop = common::catalog_with_renewal();
    gop.economy.finance_index = FinanceBasis::Gop;
    let mut gai = gop.clone();
    gai.economy.finance_index = FinanceBasis::GopAfterInvestment;
    let periods = season_periods(PeriodKind::Month);
    let category = gai.base.profile(Profile::A).unwrap().category;
    let refs = RefTable::compute(&gai, category, &periods).unwrap();
    let gop_refs = RefTable::compute(&gop, category, &periods).unwrap();
    assert_eq!(refs.points, gop_refs.points);
    assert!(refs.deppar_cents.iter().all(|d| *d == 0));
    let mut s = HotelState::from_profile(&gai, Profile::A).unwrap();
    for p in &periods {
        let plan = if p.start_month() == Month::Jun {
            plan_with(&s, p, &gai, &[("rooms", 150_000)], 0)
        } else {
            common::hold_plan(&s, p, &gai)
        };
        let r = resolve_period(&s, p, &plan, &[], &gai).unwrap();
        let point = &refs.points[usize::from(p.index)];
        let money = &r.summary.money;
        let nights = i64::from(r.accounts.rooms) * i64::from(p.days);
        let dep = r.finance.depreciation_cents;
        let deppar = (2 * dep + nights) / (2 * nights);
        match p.start_month() {
            Month::Jun => assert_eq!(dep, 0),
            Month::Jul => {
                assert_eq!((dep, nights, deppar), (178_571, 3_720, 48));
            }
            _ => assert_eq!(dep, 178_571),
        }
        let labor = sim_hotel_v2::calendar::per_month_equiv(money.labor_cents, p).unwrap();
        let raw = sim_hotel::raw_indices(
            &r.summary.after,
            money.goppar_cents - deppar,
            labor,
            point.goppar_cents,
            &gai.base,
        )
        .unwrap();
        let idx = period_indices(&r, &gai, &refs).unwrap();
        assert_eq!(
            idx,
            sim_hotel::centred(&raw, &point.raw).unwrap(),
            "{}",
            p.code()
        );
        let with_gop = period_indices(&r, &gop, &gop_refs).unwrap();
        let f = |i: &sim_hotel::IndexSet| i.get(sim_hotel_v2::Index::Finance).hundredths();
        if dep == 0 {
            assert_eq!(idx, with_gop, "{}", p.code());
        } else {
            assert!(f(&idx) < f(&with_gop), "{}", p.code());
        }
        s = r.after;
    }
}

#[test]
fn finance_basis_codes_round_trip() {
    for b in [
        FinanceBasis::Gop,
        FinanceBasis::Ebit,
        FinanceBasis::GopAfterInvestment,
    ] {
        assert_eq!(FinanceBasis::from_code(b.code()), Some(b));
    }
    assert_eq!(
        FinanceBasis::GopAfterInvestment.code(),
        "gop_after_investment"
    );
}
