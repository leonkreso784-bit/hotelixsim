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
