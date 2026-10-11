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

/// Tjedna sezona profila A uz zadani plan (sjeme bira odgovore na situacije): odlasci se kroz sezonu raspoređuju po
/// ljudima, ne svaki krug najvećem odjelu. Udio Smještaja u odlascima sezone smije biti iznad njegova udjela ljudi
/// (prosjek po krugovima) najviše za jedan odlazak (najveći ostatak nad zbrojem sezone) i 5 postotnih bodova za
/// promjenu sastava osoblja kroz sezonu (sezonci po odjelima rastu i padaju različito).
#[test]
fn weekly_departures_spread_over_departments_through_the_season() {
    use sim_hotel_v2::department::DeptCode;
    use sim_hotel_v2::{PeriodPlan, run_season_with};
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let cat = content.v2.as_ref().unwrap();
    let refs = RefTable::compute(cat, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    for seed in [1_u64, 7, 42, 1_000] {
        let r = run_season_with(
            Profile::A,
            PeriodKind::Week,
            cat,
            &content.situations,
            &refs,
            &mut |s, p| Ok(PeriodPlan::default_for(s, p, cat)?),
            &mut |t| {
                let mut rng = engine::Rng::derive(seed, &[engine::label(&t.code)]);
                let n = u64::try_from(t.answers.len()).unwrap();
                Ok(usize::try_from(rng.range(0, n)).unwrap())
            },
        )
        .unwrap();
        let (mut rooms_people, mut all_people) = (0_u64, 0_u64);
        let (mut rooms_out, mut all_out) = (0_u64, 0_u64);
        let mut depts_with_departures = std::collections::BTreeSet::new();
        for (rep, _) in &r.periods {
            let a = &rep.accounts;
            for o in a.departments.values() {
                all_people += u64::from(o.staff);
                all_out += u64::from(o.departures);
                if o.code == DeptCode::rooms() {
                    rooms_people += u64::from(o.staff);
                    rooms_out += u64::from(o.departures);
                }
                if o.departures > 0 {
                    depts_with_departures.insert(o.code.clone());
                }
            }
            for u in a.undistributed.values() {
                all_people += u64::from(u.staff);
                all_out += u64::from(u.departures);
                if u.departures > 0 {
                    depts_with_departures.insert(u.code.clone());
                }
            }
        }
        eprintln!(
            "sjeme {seed}: odlasci {all_out}, Smještaj {rooms_out}, ljudi Smještaja {rooms_people}/{all_people}, \
             odjeli s odlaskom {depts_with_departures:?}"
        );
        assert!(
            depts_with_departures.len() >= 3,
            "sjeme {seed}: odlazak samo u {depts_with_departures:?}"
        );
        // rooms_out/all_out ≤ rooms_people/all_people + 1/all_out + 5 %, u cijelim brojevima (·100·all_out·all_people).
        assert!(all_out > 0);
        let lhs = 100 * rooms_out * all_people;
        let rhs = 100 * rooms_people * all_out + 100 * all_people + 5 * all_out * all_people;
        assert!(
            lhs <= rhs,
            "sjeme {seed}: Smještaj {rooms_out}/{all_out} odlazaka, a ljudi {rooms_people}/{all_people}"
        );
    }
}
