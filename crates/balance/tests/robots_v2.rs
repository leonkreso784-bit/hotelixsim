// Robot „brižni" v2 (ZP-2): potreba za ljudima je ⌈sobe·popunjenost/1,8⌉ zaposlenih koji služe goste, pa se od nje
// oduzimaju samo stalni izvan neraspoređenih odjela (Uprava, Prodaja, Održavanje ne služe goste i nisu u
// opterećenju hotela). Profil A u lipnju: 120 soba · 72 % / 1,8 = 48; stalni u operativnim odjelima 11 + 7 + 2 + 3 =
// 23 → cilj 25 sezonaca (v1: 48 − 28 = 20; prije popravka 48 − 39 = 9, stisnuto korakom na 28 − 10 = 18).
use content::{Snapshot, repo_dir};
use engine::{Rng, label};
use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::department::UsaliClass;
use sim_hotel_v2::{HotelState, Month, Profile, Strategy, robot_plan};

#[test]
fn caring_robot_counts_only_people_who_serve_guests() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let state = HotelState::from_profile(cat, Profile::A).unwrap();
    let serving_permanent: u16 = state
        .departments
        .values()
        .filter(|d| !matches!(d.class, UsaliClass::Undistributed(_)))
        .map(|d| d.permanent)
        .sum();
    assert_eq!(serving_permanent, 23);
    let p = month_period(Month::Jun);
    let mut rng = Rng::derive(1, &[u64::from(p.index), label("robot:plan")]);
    let plan = robot_plan(Strategy::Caring, &state, &p, cat, &mut rng).unwrap();
    let seasonal: u16 = plan.departments.values().map(|d| d.seasonal_target).sum();
    assert_eq!(seasonal, 25);
}

/// Tjedna sezona brižnog za svaki profil: (krug, sezonci prije, sezonci poslije, potreba kruga, cilj iz pravila
/// ZP-6). Potreba je ⌈sobe·popunjenost/1,8⌉ − stalni koji služe goste; cilj je
/// `min(max(potreba_t, max_k(potreba_{t+k} − 3k)), min_k(potreba_{t+k} + 3k))`.
/// Krug brižnog: (krug, sezonci prije, sezonci poslije, potreba kruga, cilj iz pravila).
type CaringRound = (sim_hotel_v2::Period, i64, i64, i64, i64);

fn weekly_caring_rounds() -> Vec<(Profile, Vec<CaringRound>)> {
    use sim_hotel_v2::calendar::avg_u16;
    use sim_hotel_v2::{Category, PeriodKind, RefTable, run_season, season_periods};
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let spec = cat.base.category(Category::Four).unwrap();
    let step = i64::from(cat.coeffs(PeriodKind::Week).unwrap().seasonal_step);
    let periods = season_periods(PeriodKind::Week);
    let refs = RefTable::compute(cat, Category::Four, &periods).unwrap();
    let mut out = Vec::new();
    for profile in Profile::ALL {
        let start = HotelState::from_profile(cat, profile).unwrap();
        let serving: i64 = start
            .departments
            .values()
            .filter(|d| !matches!(d.class, UsaliClass::Undistributed(_)))
            .map(|d| i64::from(d.permanent))
            .sum();
        let need = |p: &sim_hotel_v2::Period| {
            let occ = i64::from(avg_u16(&spec.base_occupancy_bp, p).unwrap());
            ((i64::from(start.rooms) * occ + 17_999) / 18_000 - serving).max(0)
        };
        let r = run_season(
            profile,
            Strategy::Caring,
            7,
            PeriodKind::Week,
            cat,
            &content.situations,
            &refs,
        )
        .unwrap();
        let mut before = i64::from(start.summary().seasonal);
        let mut rows = Vec::new();
        for (rep, _) in &r.periods {
            let p = rep.period;
            let later: Vec<i64> = periods
                .iter()
                .filter(|q| q.index > p.index)
                .map(need)
                .collect();
            let up = later
                .iter()
                .zip(1_i64..)
                .map(|(n, k)| n - k * step)
                .fold(need(&p), i64::max);
            let down = later
                .iter()
                .zip(1_i64..)
                .map(|(n, k)| n + k * step)
                .fold(i64::MAX, i64::min);
            let now = i64::from(rep.summary.after.seasonal);
            rows.push((p, before, now, need(&p), up.min(down)));
            before = now;
        }
        out.push((profile, rows));
    }
    out
}

/// ZP-6 (Leonova odluka 11.10.): tjedni brižni planira u oba smjera, a krug nikad ne izlazi iz onoga što pravilo i
/// korak ±3 dopuštaju: sezonci = cilj stisnut na [prije − 3, prije + 3] (i najviše 50). Prije kolovoza diže sezonce
/// iznad potrebe svog kruga (zapošljava unaprijed za vrhunac).
#[test]
fn weekly_caring_robot_hires_ahead_of_the_peak() {
    for (profile, rows) in weekly_caring_rounds() {
        let mut ahead = false;
        for (p, before, now, need, target) in &rows {
            let expected = (*target).clamp(before - 3, before + 3).min(50);
            assert_eq!(
                *now,
                expected,
                "{profile:?} {}: potreba {need}, cilj {target}",
                p.code()
            );
            let early = matches!(p.start_month(), Month::Jun | Month::Jul);
            if early && now > before && now > need {
                ahead = true;
            }
        }
        assert!(
            ahead,
            "{profile:?}: brižni ne zapošljava unaprijed prije kolovoza"
        );
    }
}

/// ZP-6: tjedni brižni počinje spuštati sezonce već u kolovozu (prije pada u rujnu) i ispod potrebe svog kruga,
/// da u rujnu ne plaća višak ljudi; mjesečni krug ostaje v1 pravilo (bez gledanja unaprijed).
#[test]
fn weekly_caring_robot_releases_before_september() {
    for (profile, rows) in weekly_caring_rounds() {
        let released_in_august = rows.iter().any(|(p, before, now, need, _)| {
            p.start_month() == Month::Aug && now < before && now < need
        });
        assert!(
            released_in_august,
            "{profile:?}: brižni ne počinje otpuštati prije rujna"
        );
    }
}

/// Obnova v2 (O6, obnova-v2-spec §5.1) na sadržaju: brižni u profilu B obnavlja Smještaj (počinje na 40, ispod 60)
/// najmanjim iznosom u koraku koji ga vraća na 60, i to u prvom krugu sezone; lovac isto samo Smještaj; štediša i
/// nasumični nikad; zgradu nitko.
#[test]
fn caring_robot_in_b_renews_rooms() {
    use sim_hotel_v2::department::DeptCode;
    use sim_hotel_v2::renewal::smallest_amount_for;
    use sim_hotel_v2::{Category, PeriodKind, RefTable, run_season, season_periods};
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let rooms = DeptCode::rooms();
    let start = HotelState::from_profile(cat, Profile::B).unwrap();
    let need = 6_000 - start.departments[&rooms].condition_pts;
    let amount = smallest_amount_for(&cat.dept(&rooms).unwrap().renewal, need).unwrap();
    assert!(amount > 0);
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let refs = RefTable::compute(cat, Category::Four, &season_periods(kind)).unwrap();
        let run = |s: Strategy| {
            run_season(Profile::B, s, 7, kind, cat, &content.situations, &refs).unwrap()
        };
        let caring = run(Strategy::Caring);
        let (first, _) = &caring.periods[0];
        assert!(first.accounts.departments[&rooms].renewal_cents > 0);
        let invested = caring.periods.last().unwrap().0.after.departments[&rooms]
            .book
            .invested_cents();
        assert_eq!(invested, amount, "{kind:?}");
        for s in Strategy::ALL {
            let r = run(s);
            let building: i64 = r
                .periods
                .iter()
                .map(|(rep, _)| rep.accounts.building.renewal_cents)
                .sum();
            assert_eq!(building, 0, "{s:?} {kind:?}");
            let others: i64 = r
                .periods
                .iter()
                .flat_map(|(rep, _)| rep.accounts.departments.iter())
                .filter(|(k, _)| **k != rooms)
                .map(|(_, o)| o.renewal_cents)
                .sum();
            let rooms_paid: i64 = r
                .periods
                .iter()
                .map(|(rep, _)| rep.accounts.departments[&rooms].renewal_cents)
                .sum();
            match s {
                Strategy::Saver | Strategy::Random => assert_eq!(rooms_paid + others, 0, "{s:?}"),
                Strategy::OccupancyHunter => assert_eq!(others, 0, "{s:?}"),
                Strategy::Caring => assert!(rooms_paid > 0),
            }
        }
    }
}

/// Mjesečna sezona brižnog za svaki profil (RB-1, ADR-041): po krugu (krug, sezonci prije, sezonci poslije, potreba
/// kruga, gornja granica `down` iz ZP-6, gornji korak i najviše sezonaca). `down = min_k(potreba_{t+k} + k·korak)`
/// nad preostalim mjesecima sezone.
type MonthlyRound = (sim_hotel_v2::Period, i64, i64, i64, i64);

fn monthly_caring_rounds() -> (i64, i64, Vec<(Profile, Vec<MonthlyRound>)>) {
    use sim_hotel_v2::calendar::avg_u16;
    use sim_hotel_v2::{Category, PeriodKind, RefTable, run_season, season_periods};
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let spec = cat.base.category(Category::Four).unwrap();
    let step = i64::from(cat.coeffs(PeriodKind::Month).unwrap().seasonal_step);
    let max = i64::from(cat.base.actions.seasonal_max);
    let periods = season_periods(PeriodKind::Month);
    let refs = RefTable::compute(cat, Category::Four, &periods).unwrap();
    let mut out = Vec::new();
    for profile in Profile::ALL {
        let start = HotelState::from_profile(cat, profile).unwrap();
        let serving: i64 = start
            .departments
            .values()
            .filter(|d| !matches!(d.class, UsaliClass::Undistributed(_)))
            .map(|d| i64::from(d.permanent))
            .sum();
        let need = |p: &sim_hotel_v2::Period| {
            let occ = i64::from(avg_u16(&spec.base_occupancy_bp, p).unwrap());
            ((i64::from(start.rooms) * occ + 17_999) / 18_000 - serving).max(0)
        };
        let r = run_season(
            profile,
            Strategy::Caring,
            7,
            PeriodKind::Month,
            cat,
            &content.situations,
            &refs,
        )
        .unwrap();
        let mut before = i64::from(start.summary().seasonal);
        let mut rows = Vec::new();
        for (rep, _) in &r.periods {
            let p = rep.period;
            let down = periods
                .iter()
                .filter(|q| q.index > p.index)
                .zip(1_i64..)
                .map(|(q, k)| need(q) + k * step)
                .fold(i64::MAX, i64::min);
            let now = i64::from(rep.summary.after.seasonal);
            rows.push((p, before, now, need(&p), down));
            before = now;
        }
        out.push((profile, rows));
    }
    (step, max, out)
}

/// RB-1 (ADR-041): mjesečni brižni otpušta sezonce prije pada kao tjedni: cilj kruga je `min(potreba, down)`, a krug
/// ga slijedi do koraka (±korak, najviše `seasonal_max`). Na kodu prije RB-1 mjesečni drži `potreba` (B u kolovozu
/// 41 sezonac, a granica prema rujnu je niža), pa ga u rujnu korak zaustavi iznad potrebe.
#[test]
fn monthly_caring_robot_releases_before_the_september_drop() {
    let (step, max, all) = monthly_caring_rounds();
    // Izravno: u kolovozu mjesečni B brižni ne drži više sezonaca nego što mu dopušta rujan + korak.
    let (_, rows) = all.iter().find(|(pr, _)| *pr == Profile::B).unwrap();
    let (p, _, now, need, down) = rows
        .iter()
        .find(|(p, ..)| p.start_month() == Month::Aug)
        .unwrap();
    assert!(
        down < need,
        "kolovoz B: potreba {need} ne smije biti ispod granice {down} (inače test ništa ne mjeri)"
    );
    assert!(
        now <= down,
        "{profile:?} {}: {now} sezonaca u kolovozu, gornja granica prema rujnu {down} (potreba {need})",
        p.code(),
        profile = Profile::B
    );
    // Svi profili, svi mjeseci: cilj i korak.
    for (profile, rows) in &all {
        for (p, before, now, need, down) in rows {
            let target = (*need).min(*down);
            let expected = target.clamp(before - step, before + step).min(max);
            assert_eq!(
                *now,
                expected,
                "{profile:?} {}: potreba {need}, gornja granica {down}, cilj {target}",
                p.code()
            );
        }
    }
}

/// Kontrola RB-1: mjesečni brižni NE zapošljava unaprijed (v1 ekvivalencija, MODEL_V2 §2 t. 6): prije kolovoza
/// krug je `potreba` stisnuta korakom, nikad iznad nje, i nikad iznad onoga što bi dao zadnji v1 oblik pravila.
#[test]
fn monthly_caring_robot_does_not_hire_ahead() {
    let (step, max, all) = monthly_caring_rounds();
    for (profile, rows) in &all {
        for (p, before, now, need, _) in rows {
            if matches!(p.start_month(), Month::Jun | Month::Jul) {
                let expected = (*need).clamp(before - step, before + step).min(max);
                assert_eq!(
                    *now,
                    expected,
                    "{profile:?} {}: mjesečni brižni mora slijediti potrebu {need}",
                    p.code()
                );
                assert!(
                    now <= &(*need).max(*before),
                    "{profile:?} {}: zaposlio unaprijed ({now} > potreba {need})",
                    p.code()
                );
            }
        }
    }
}
