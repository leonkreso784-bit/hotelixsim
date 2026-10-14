// Obnova v2 (O-2): brojke obnove u sadržaju `content/hotel/v2/*.json` (obnova-v2-spec §0, O1–O15).
use content::{Snapshot, repo_dir};
use sim_hotel_v2::Maintenance;
use sim_hotel_v2::department::DeptCode;

fn v2() -> sim_hotel_v2::Catalog {
    Snapshot::read_v2(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap()
}

/// `engine::calc::div_round` za pozitivne brojeve (pola ide gore).
fn div_round(a: i64, b: i64) -> i64 {
    (2 * a + b) / (2 * b)
}

fn code(s: &str) -> DeptCode {
    DeptCode(s.to_string())
}

#[test]
fn content_v2_with_renewal_numbers_validates() {
    v2().validate().unwrap();
}

/// O7, O8, O11: redovno održavanje svakog odjela koji se troši pokriva 75 % trošenja (`div_round(trošenje·7 500,
/// 10 000)`), odgođeno ne donosi bodove.
#[test]
fn regular_maintenance_covers_three_quarters_of_wear() {
    let c = v2();
    let mut worn = 0;
    for (k, d) in &c.departments {
        if d.wear_pts_per_month == 0 {
            continue;
        }
        worn += 1;
        let regular = d.maintenance_level(Maintenance::Regular).equipment_pts;
        let want = div_round(i64::from(d.wear_pts_per_month) * 7_500, 10_000);
        assert_eq!(i64::from(regular), want, "{}", k.as_str());
        assert_eq!(
            d.maintenance_level(Maintenance::Deferred).equipment_pts,
            0,
            "{}",
            k.as_str()
        );
    }
    // Smještaj, Restoran, Lobby bar, Rooftop bar, Bar uz bazen, Wellness, Parking, Lokal.
    assert_eq!(worn, 8);
}

/// O7, O8, O11 doslovno: redovno nakon spuštanja na 75 % i najmovi.
#[test]
fn maintenance_numbers_match_the_table() {
    let c = v2();
    let level = |k: &str, m: Maintenance| {
        let l = c.dept(&code(k)).unwrap().maintenance_level(m);
        (l.cost_cents, l.equipment_pts)
    };
    let regular = [
        ("rooms", 1_500_000, 300),
        ("restaurant", 400_000, 225),
        ("lobby_bar", 150_000, 188),
        ("rooftop_bar", 200_000, 263),
        ("pool_bar", 150_000, 338),
        ("spa", 250_000, 188),
        ("parking", 50_000, 38),
        ("retail_lease", 30_000, 38),
    ];
    for (k, cost, pts) in regular {
        assert_eq!(level(k, Maintenance::Regular), (cost, pts), "{k}");
    }
    // Najmovi (O8): trošenje 0,5 boda mjesečno; pojačano parking 1.500 €/mj = +3, lokal 800 €/mj = +2.
    for k in ["parking", "retail_lease"] {
        assert_eq!(c.dept(&code(k)).unwrap().wear_pts_per_month, 50, "{k}");
        assert_eq!(level(k, Maintenance::Deferred), (0, 0), "{k}");
    }
    assert_eq!(level("parking", Maintenance::Intensive), (150_000, 300));
    assert_eq!(level("retail_lease", Maintenance::Intensive), (80_000, 200));
    // Bar uz bazen (O7) troši 4,5 boda mjesečno; pojačano ostaje kao prije (O11).
    assert_eq!(c.dept(&code("pool_bar")).unwrap().wear_pts_per_month, 450);
    assert_eq!(level("pool_bar", Maintenance::Intensive), (300_000, 600));
}

/// Najmovi (korak b′, Leonova anketa 13.10. (3)): parking je 5 % osjećaja sobe, lokal ima popust na najamninu ispod
/// početnog stanja (2 % po bodu, najviše 30 %); nitko drugi nema ni jedno ni drugo.
#[test]
fn lease_rules_in_content() {
    use sim_hotel_v2::department::RentCondition;
    let c = v2();
    for (k, d) in &c.departments {
        let (share, rule) = match k.as_str() {
            "parking" => (500, None),
            "retail_lease" => (
                0,
                Some(RentCondition {
                    threshold_pts: None,
                    cut_bp_per_pt: 200,
                    max_cut_bp: 3_000,
                }),
            ),
            _ => (0, None),
        };
        assert_eq!(
            (d.guest_share_bp, d.rent_condition),
            (share, rule),
            "{}",
            k.as_str()
        );
    }
}

/// Ručni primjer na sadržaju: profil B, mjesečna sezona, zadani plan uz odgođeno održavanje lokala. Lokal počinje
/// na 60 i uz odgođeno održavanje gubi div_round(50 · 12 500, 10 000) = 63 stotinke boda mjesečno (korak e), pa
/// popust raste za 126 bp mjesečno: najamnina (6.000 €) 5.924,40, 5.848,80, 5.773,20, 5.697,60 €.
#[test]
fn lease_rent_falls_with_deferred_upkeep_by_hand() {
    use sim_hotel_v2::{Category, PeriodKind, PeriodPlan, Profile, RefTable};
    let c = v2();
    let lokal = code("retail_lease");
    let periods = sim_hotel_v2::season_periods(PeriodKind::Month);
    let refs = RefTable::compute(&c, Category::Four, &periods).unwrap();
    let r = sim_hotel_v2::run_season_with(
        Profile::B,
        PeriodKind::Month,
        &c,
        &[],
        &refs,
        &mut |s, p| {
            let mut plan = PeriodPlan::default_for(s, p, &c)?;
            plan.departments.get_mut(&lokal).unwrap().maintenance = Maintenance::Deferred;
            Ok(plan)
        },
        &mut |_| Err(sim_hotel_v2::ResolveError::BadChoice),
    )
    .unwrap();
    let got: Vec<(i32, i64)> = r
        .periods
        .iter()
        .map(|(rep, _)| {
            let o = &rep.accounts.departments[&lokal];
            (o.condition_after_pts, o.revenue_cents)
        })
        .collect();
    assert_eq!(
        got,
        [
            (5_937, 592_440),
            (5_874, 584_880),
            (5_811, 577_320),
            (5_748, 569_760)
        ]
    );
}

/// Nova polja najmova se čitaju i pišu bez gubitka; bez ključeva odjel nema udio ni pravilo najamnine.
#[test]
fn lease_fields_round_trip() {
    use serde_json::json;
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["departments"]["retail_lease"]["rent_condition"]["threshold_pts"] = json!(6500);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    let c = s.build().unwrap().v2.unwrap();
    let rule = c
        .dept(&code("retail_lease"))
        .unwrap()
        .rent_condition
        .unwrap();
    assert_eq!(rule.threshold_pts, Some(6_500));
    let d = v["v2"]["departments"].as_object_mut().unwrap();
    d["parking"]
        .as_object_mut()
        .unwrap()
        .remove("guest_share_bp");
    d["retail_lease"]
        .as_object_mut()
        .unwrap()
        .remove("rent_condition");
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    let c = s.build().unwrap().v2.unwrap();
    assert!(
        c.departments
            .values()
            .all(|d| d.guest_share_bp == 0 && d.rent_condition.is_none())
    );
}

/// O1, O2, O5, O10: krivulja obnove (G 50 bodova, K), najveći mjesečni iznos i korak, vijek za amortizaciju; svi
/// odjeli s gostima i parking je imaju, lokal i neraspoređeni nemaju.
#[test]
fn renewal_curves_and_limits_per_department() {
    let c = v2();
    let want: [(&str, i64, i64, i64, u16); 7] = [
        ("rooms", 15_000_000, 15_000_000, 500_000, 84),
        ("restaurant", 5_500_000, 5_000_000, 500_000, 84),
        ("lobby_bar", 2_500_000, 5_000_000, 500_000, 84),
        ("rooftop_bar", 2_500_000, 5_000_000, 500_000, 84),
        ("pool_bar", 2_000_000, 5_000_000, 500_000, 84),
        ("spa", 4_000_000, 5_000_000, 500_000, 120),
        ("parking", 6_000_000, 5_000_000, 500_000, 180),
    ];
    for (k, half, max, step, life) in want {
        let r = &c.dept(&code(k)).unwrap().renewal;
        assert_eq!(
            (
                r.gain_pts,
                r.half_cents,
                r.max_monthly_cents,
                r.step_cents,
                r.life_months
            ),
            (5_000, half, max, step, life),
            "{k}"
        );
    }
    for (k, d) in &c.departments {
        if !want.iter().any(|w| w.0 == k.as_str()) {
            assert_eq!(d.renewal, Default::default(), "{}", k.as_str());
            assert!(
                k.as_str() == "retail_lease" || d.class.is_undistributed(),
                "{} nema obnovu",
                k.as_str()
            );
        }
    }
}

/// O4, O5, O9, O1, O2, O3/O15: zgrada (trošenje 0,5/mj, udio 20 %, K 150.000 €, najviše 200.000 € u koraku
/// 10.000 €, vijek 40 g.), stanje zgrade po profilu i smjernica pričuve 4 %.
#[test]
fn building_and_reserve() {
    use sim_hotel_v2::Profile;
    let c = v2();
    let b = &c.economy.building;
    assert_eq!(
        (
            b.name.as_str(),
            b.wear_pts_per_month,
            b.satisfaction_share_bp
        ),
        ("Zgrada", 50, 2_000)
    );
    let r = &b.renewal;
    assert_eq!(
        (
            r.gain_pts,
            r.half_cents,
            r.max_monthly_cents,
            r.step_cents,
            r.life_months
        ),
        (5_000, 15_000_000, 20_000_000, 1_000_000, 480)
    );
    assert_eq!(c.economy.ffe_reserve_bp, 400);
    let pts: Vec<i32> = Profile::ALL
        .iter()
        .map(|p| c.profile_v2(*p).unwrap().building_pts)
        .collect();
    assert_eq!(pts, [8_000, 4_500, 6_500]);
    assert!(c.renewal_enabled());
}

/// Korak d′ (Leonova odluka): F iz GOP-a nakon ulaganja — GOP po sobi i danu umanjen za amortizaciju obnova, bez
/// fiksnih naknada; nazivnik ostaje GOPPAR referentnog hotela (O12).
#[test]
fn finance_index_is_gop_after_investment() {
    assert_eq!(
        v2().economy.finance_index,
        sim_hotel_v2::FinanceBasis::GopAfterInvestment
    );
}

/// Korak e (jezgra): faktor trošenja uz odgođeno održavanje čita se iz `economy.deferred_wear_bp`; bez ključa je
/// 10 000 (kao redovno).
#[test]
fn deferred_wear_factor_round_trips() {
    use serde_json::json;
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("deferred_wear_bp");
    let none = Snapshot::from_value(&v).unwrap();
    assert_eq!(
        none.build().unwrap().v2.unwrap().economy.deferred_wear_bp,
        10_000
    );
    v["v2"]["economy"]["deferred_wear_bp"] = json!(15_000);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    let c = s.build().unwrap().v2.unwrap();
    assert_eq!(c.economy.deferred_wear_bp, 15_000);
    c.validate().unwrap();
}

/// Korak e (presuda kontrolora 13.10.): zapušten prostor uz odgođeno održavanje troši se 1,25 × brže.
#[test]
fn deferred_wear_factor_in_content() {
    assert_eq!(v2().economy.deferred_wear_bp, 12_500);
}
