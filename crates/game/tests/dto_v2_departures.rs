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
