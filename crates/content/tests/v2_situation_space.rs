//! B1a: `economy.repair_urgent_condition_pts` u sadržaju v2 (ADR-042 t. 1): hitan popravak diže stanje prostora.
mod situations_common;

use content::{ContentError, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{
    Anchor, Bounds, Conditions, DeptTarget, DrawnSituation, Kind, Repeat,
};
use sim_hotel_v2::{
    CatalogError, HotelState, PeriodKind, PeriodPlan, Profile, RefTable, ResolveError,
    SeasonSituations, TemplateDraw, run_season_from, season_periods,
};
use situations_common::{Hotel, frozen, general, indices, round, start};

#[test]
fn repair_bonus_round_trips_and_old_snapshot_reads_zero() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let repo = Snapshot::from_value(&v)
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap();
    assert_eq!(
        repo.economy.repair_urgent_condition_pts, 100,
        "repo gradi 100 (privremeno, Leon 15.10.; pravi iznos u L1-13)"
    );

    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("repair_urgent_condition_pts");
    let old = v.clone();
    let none = Snapshot::from_value(&old).unwrap();
    assert_eq!(
        none.build()
            .unwrap()
            .v2
            .unwrap()
            .economy
            .repair_urgent_condition_pts,
        0,
        "stari snimak bez ključa: bonus 0"
    );
    let back = Snapshot::from_value(&old).unwrap();
    assert_eq!(
        fingerprint_value(&back.to_value()),
        fingerprint_value(&old),
        "snimak bez ključa se ne piše s ključem"
    );

    v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(1_000);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    s.build().unwrap().v2.unwrap().validate().unwrap();

    v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(20_000);
    assert!(
        Snapshot::from_value(&v).unwrap().build().is_err(),
        "iznad 10 000 bodova: provjera kataloga v2 odbija"
    );
}

const OFTEN: Repeat = Repeat {
    max_per_season: 17,
    gap_rounds: 1,
};

/// B1b (ADR-042 t. 7): `condition_of` kaže čije stanje čita `dept_condition_pts`. Obrazac: `boost_multiplies_weight_exactly`.
/// Kvar na cijelom hotelu s uvjetom „stanje Smještaja ispod 4 000": izvlači se samo kad Smještaj stvarno loše stoji.
#[test]
fn condition_of_reads_the_condition_of_that_space() {
    let (cat, mut set) = frozen();
    let mut lift = general("P-LIFT", Kind::Breakdown, OFTEN);
    lift.when = Conditions {
        dept_condition_pts: Some(Bounds {
            at_least: None,
            below: Some(4_000),
        }),
        ..Conditions::default()
    };
    let t06 = season_periods(PeriodKind::Week)[5];
    let drawn = |set: &sim_hotel_v2::situations::TemplateSet, rooms: i32, seed: u64| {
        let mut state: HotelState = start(&cat, Profile::C);
        state
            .departments
            .get_mut(&DeptCode::rooms())
            .unwrap()
            .condition_pts = rooms;
        let n = sim_hotel_v2::situations::situation_count(&state, &t06, &cat).unwrap();
        let anchors: Vec<Anchor> = [Kind::Staff, Kind::Complaint, Kind::Crisis]
            .into_iter()
            .cycle()
            .take(usize::from(n - 1))
            .map(|kind| Anchor {
                kind,
                department: None,
            })
            .collect();
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        round(&cat, set, &h, &t06, &[], seed, &anchors)
    };

    // Karakterizacija: bez `condition_of` cijeli hotel ne vidi stanje odjela, pa se uvjet nikad ne ispuni.
    set.templates = vec![lift.clone()];
    for seed in 0..50 {
        assert!(
            drawn(&set, 3_500, seed).is_empty(),
            "sjeme {seed}: bez condition_of hotel nema stanje odjela"
        );
    }

    lift.condition_of = Some(DeptCode::rooms());
    set.templates = vec![lift];
    for seed in 0..50 {
        let d = drawn(&set, 3_500, seed);
        assert_eq!(
            d.len(),
            1,
            "sjeme {seed}: Smještaj 3 500 < 4 000 pušta kvar"
        );
        assert_eq!(d[0].template, "P-LIFT");
        assert!(
            drawn(&set, 6_000, seed).is_empty(),
            "sjeme {seed}: Smještaj 6 000 >= 4 000 ne pušta kvar"
        );
    }
}

fn v2_value() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn tpl<'a>(v: &'a mut Value, code: &str) -> &'a mut Value {
    let i = v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["code"] == code)
        .unwrap();
    &mut v["v2"]["situation_templates"]["templates"][i]
}

fn build_with(change: impl FnOnce(&mut Value)) -> Result<(), ContentError> {
    let mut v = v2_value();
    change(&mut v);
    Snapshot::from_value(&v).unwrap().build().map(|_| ())
}

/// `condition_of` je dopušten samo uz prostor koji se troši; uvjet stanja na cijelom hotelu traži `condition_of`.
#[test]
fn condition_of_must_name_a_space_that_wears() {
    let below_6000 = json!({"dept_condition_pts": {"below": 6000}});
    let with_cond = |v: &mut Value| tpl(v, "P-DIZALO")["when"] = below_6000.clone();
    let r = build_with(|v| {
        with_cond(v);
        tpl(v, "P-DIZALO")["condition_of"] = json!("rooms");
    });
    assert!(r.is_ok(), "P-DIZALO + condition_of rooms + uvjet: {r:?}");

    let r = build_with(with_cond);
    assert!(
        matches!(&r, Err(ContentError::Invalid(m)) if m.contains("uvjet odjela traži predložak s odjelom")),
        "bez condition_of: {r:?}"
    );

    for bad in ["maintenance", "nepostoji"] {
        let r = build_with(|v| {
            with_cond(v);
            tpl(v, "P-DIZALO")["condition_of"] = json!(bad);
        });
        assert!(
            matches!(&r, Err(ContentError::Invalid(m))
                if m.contains("condition_of") && !m.contains("unknown field")),
            "'{bad}' mora odbiti provjera kataloga, ne parser: {r:?}"
        );
    }

    let v = v2_value();
    assert!(
        !v.to_string().contains("condition_of"),
        "repo snimak ne smije imati ključ (B4 ga upisuje)"
    );
}

/// B1b, Minor 1: predložak s vlastitim odjelom (grana `Some(depts)` izvlačenja) i `condition_of` čita stanje
/// `condition_of`, ne stanje svog odjela.
#[test]
fn condition_of_beats_own_department_in_the_draw() {
    let (cat, mut set) = frozen();
    let maintenance = DeptCode("maintenance".to_string());
    let mut t = general("P-BRAVA", Kind::Breakdown, OFTEN);
    t.department = DeptTarget::One(maintenance.clone());
    t.condition_of = Some(DeptCode::rooms());
    t.when = Conditions {
        dept_condition_pts: Some(Bounds {
            at_least: None,
            below: Some(4_000),
        }),
        ..Conditions::default()
    };
    set.templates = vec![t];
    let t06 = season_periods(PeriodKind::Week)[5];
    let drawn = |rooms: i32, upkeep: i32, seed: u64| {
        let mut state: HotelState = start(&cat, Profile::C);
        for (code, pts) in [(DeptCode::rooms(), rooms), (maintenance.clone(), upkeep)] {
            state.departments.get_mut(&code).unwrap().condition_pts = pts;
        }
        let n = sim_hotel_v2::situations::situation_count(&state, &t06, &cat).unwrap();
        let anchors: Vec<Anchor> = [Kind::Staff, Kind::Complaint, Kind::Crisis]
            .into_iter()
            .cycle()
            .take(usize::from(n - 1))
            .map(|kind| Anchor {
                kind,
                department: None,
            })
            .collect();
        let h = Hotel {
            state: &state,
            indices: indices(5_000),
            slot: 1,
        };
        round(&cat, &set, &h, &t06, &[], seed, &anchors)
    };
    for seed in 0..50 {
        let d = drawn(3_500, 9_000, seed);
        assert_eq!(d.len(), 1, "sjeme {seed}: Smještaj loš, Održavanje dobro");
        assert_eq!(d[0].department, Some(maintenance.clone()));
        assert!(
            drawn(6_000, 3_000, seed).is_empty(),
            "sjeme {seed}: Smještaj dobar, Održavanje loše: vlastiti odjel se ne gleda"
        );
    }
}

/// B1b, Minor 2: put prostora u sezoni robota (`season.rs`). Dvije sezone nad istim snimkom (P-INSPEKCIJA bez `major`),
/// uz i bez `condition_of: rooms`, svugdje prvi odgovor (hitan popravak). Do prvog kruga s P-INSPEKCIJA sezone su iste;
/// u tom krugu popravak ide u Smještaj (uz polje) odnosno u Restoran (bez njega).
#[test]
fn season_repair_goes_to_condition_of_space() {
    let season = |with_field: bool| {
        let mut v = v2_value();
        let t = tpl(&mut v, "P-INSPEKCIJA").as_object_mut().unwrap();
        t.remove("major");
        if with_field {
            t.insert("condition_of".into(), json!("rooms"));
        }
        let c = Snapshot::from_value(&v).unwrap().build().unwrap();
        let cat = c.v2.as_ref().unwrap();
        let tpls = c.situation_templates.as_ref().unwrap();
        let set = tpls.to_core();
        let category = cat.base.profile(Profile::C).unwrap().category;
        let periods = season_periods(PeriodKind::Week);
        let refs = RefTable::compute(cat, category, &periods).unwrap();
        let instance = |d: &DrawnSituation| {
            tpls.instance(d, &cat.base, category)
                .map_err(|e| ResolveError::Catalog(CatalogError::OutOfRange(format!("{e:?}"))))
        };
        let draw = TemplateDraw {
            anchors: &c.situations,
            templates: &set,
            seed: 11,
            slot: 1,
            instance: &instance,
        };
        let r = run_season_from(
            Profile::C,
            PeriodKind::Week,
            cat,
            SeasonSituations::Drawn(&draw),
            &refs,
            &mut |s, p| PeriodPlan::default_for(s, p, cat).map_err(ResolveError::from),
            &mut |_| Ok(0),
        )
        .unwrap();
        let bonus = cat.economy.repair_urgent_condition_pts;
        (r, bonus)
    };
    let (with, bonus) = season(true);
    let (without, _) = season(false);
    let k = with
        .drawn
        .iter()
        .position(|round| round.iter().any(|d| d.template == "P-INSPEKCIJA"))
        .expect("sezona sa sjemenom 11 nema P-INSPEKCIJA: promijeni sjeme");
    assert!(
        without.drawn[k]
            .iter()
            .any(|d| d.template == "P-INSPEKCIJA"),
        "do tog kruga sezone su iste"
    );
    let after = |r: &sim_hotel_v2::SeasonResult, d: &DeptCode| {
        r.periods[k].0.after.departments[d].condition_pts
    };
    let (rooms, restaurant) = (DeptCode::rooms(), DeptCode("restaurant".to_string()));
    assert!(
        after(&with, &rooms) - after(&without, &rooms) >= bonus,
        "Smještaj: {} / {}",
        after(&with, &rooms),
        after(&without, &rooms)
    );
    assert!(
        after(&without, &restaurant) - after(&with, &restaurant) >= bonus,
        "Restoran: {} / {}",
        after(&with, &restaurant),
        after(&without, &restaurant)
    );
}
