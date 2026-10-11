// Odlasci po odjelu u dnevniku (Task 23a): krugovi obračunati prije Taska 23 nemaju ključ `departures` i čitaju se
// kao 0; krug bez odlazaka zapisuje se bez ključa (bajt-isto kao prije), a krug s odlascima ga nosi.
use content::{Snapshot, repo_dir};
use game::dto_v2::{DeptOutcomeDto, PeriodAccountsDto, UndistributedOutcomeDto};
use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::department::PeriodAccounts;
use sim_hotel_v2::{HotelState, Month, PeriodPlan, Profile, resolve_period};

fn june_accounts() -> PeriodAccounts {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let state = HotelState::from_profile(cat, Profile::A).unwrap();
    let p = month_period(Month::Jun);
    let plan = PeriodPlan::default_for(&state, &p, cat).unwrap();
    resolve_period(&state, &p, &plan, &[], cat)
        .unwrap()
        .accounts
}

#[test]
fn department_departures_without_the_key_read_as_zero() {
    let accounts = june_accounts();
    let dto = PeriodAccountsDto::from(&accounts);
    let mut v = serde_json::to_value(&dto).unwrap();
    let mut removed = 0;
    for section in ["departments", "undistributed"] {
        for (_, d) in v[section].as_object_mut().unwrap() {
            if d.as_object_mut().unwrap().remove("departures").is_some() {
                removed += 1;
            }
        }
    }
    assert!(
        removed > 0,
        "lipanj profila A bez ijednog odlaska ne provjerava čitanje"
    );
    let old: PeriodAccountsDto = serde_json::from_value(v).unwrap();
    assert!(old.departments.values().all(|d| d.departures == 0));
    assert!(old.undistributed.values().all(|u| u.departures == 0));
    // S odlascima: ključ se zapisuje i vraća isti broj.
    let back: PeriodAccountsDto =
        serde_json::from_value(serde_json::to_value(&dto).unwrap()).unwrap();
    assert_eq!(back, dto);
}

#[test]
fn zero_departures_are_not_written() {
    let accounts = june_accounts();
    let dto = PeriodAccountsDto::from(&accounts);
    let dept: &DeptOutcomeDto = dto
        .departments
        .values()
        .find(|d| d.departures == 0)
        .unwrap();
    let service: &UndistributedOutcomeDto = dto
        .undistributed
        .values()
        .find(|u| u.departures == 0)
        .unwrap();
    assert!(
        serde_json::to_value(dept)
            .unwrap()
            .get("departures")
            .is_none()
    );
    assert!(
        serde_json::to_value(service)
            .unwrap()
            .get("departures")
            .is_none()
    );
}

#[test]
fn season_departures_in_state_are_optional_in_the_log() {
    // Stanje nakon lipnja nosi odlaske sezone po odjelu; stanje zapisano prije ZP-1 (bez ključa) čita se kao 0, a
    // odjel s 0 se zapisuje bez ključa (bajt-isto kao prije).
    use game::dto_v2::HotelStateV2Dto;
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let state = HotelState::from_profile(cat, Profile::A).unwrap();
    let p = month_period(Month::Jun);
    let plan = PeriodPlan::default_for(&state, &p, cat).unwrap();
    let after = resolve_period(&state, &p, &plan, &[], cat).unwrap().after;
    let total: u32 = after
        .departments
        .values()
        .map(|d| u32::from(d.season_departures))
        .sum();
    assert!(
        total > 0,
        "lipanj profila A bez odlazaka ne provjerava ništa"
    );
    let dto = HotelStateV2Dto::from(&after);
    let v = serde_json::to_value(&dto).unwrap();
    for (code, d) in v["departments"].as_object().unwrap() {
        let written = d.get("season_departures").and_then(|x| x.as_u64());
        let expected = after
            .departments
            .iter()
            .find(|(k, _)| k.as_str() == code)
            .map(|(_, s)| s.season_departures)
            .unwrap();
        assert_eq!(
            written,
            (expected > 0).then_some(u64::from(expected)),
            "{code}"
        );
    }
    let mut old = v.clone();
    for (_, d) in old["departments"].as_object_mut().unwrap() {
        d.as_object_mut().unwrap().remove("season_departures");
    }
    let read: HotelStateV2Dto = serde_json::from_value(old).unwrap();
    assert!(read.departments.values().all(|d| d.season_departures == 0));
    let back = HotelState::try_from(&dto).unwrap();
    assert_eq!(back, after);
}
