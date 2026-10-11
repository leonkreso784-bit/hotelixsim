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
