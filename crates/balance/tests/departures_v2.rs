// Odlasci po odjelima (Task 23a): u svakom krugu cijele sezone zbroj odlazaka svih odjela (Smještaj, operativni,
// neraspoređeni) jednak je hotelskom broju odlazaka kruga, za sva tri profila i obje vrste krugova.
use content::{Snapshot, repo_dir};
use sim_hotel_v2::{Category, PeriodKind, Profile, RefTable, Strategy, run_season, season_periods};

#[test]
fn department_departures_add_up_to_hotel_departures_every_round() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let refs = RefTable::compute(cat, Category::Four, &season_periods(kind)).unwrap();
        let mut any = 0_u32;
        for profile in Profile::ALL {
            for strategy in Strategy::ALL {
                let r = run_season(profile, strategy, 7, kind, cat, &content.situations, &refs)
                    .unwrap();
                for (rep, _) in &r.periods {
                    let a = &rep.accounts;
                    let by_dept: u32 = a
                        .departments
                        .values()
                        .map(|o| u32::from(o.departures))
                        .chain(a.undistributed.values().map(|u| u32::from(u.departures)))
                        .sum();
                    let hotel = u32::from(rep.summary.people.departures);
                    assert_eq!(
                        by_dept,
                        hotel,
                        "{profile:?} {strategy:?} {}",
                        rep.period.code()
                    );
                    any += hotel;
                    // Odjel bez ljudi nema odlazaka.
                    for o in a.departments.values().filter(|o| o.staff == 0) {
                        assert_eq!(o.departures, 0, "{}", o.code.as_str());
                    }
                }
            }
        }
        assert!(
            any > 0,
            "{kind:?}: sezona bez ijednog odlaska ne provjerava ništa"
        );
    }
}
