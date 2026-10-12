//! Mentor i „Zašto" za obnovu v2 (obnova-v2-spec §7, §8 O-5): ulaz mentora s obnovama kruga (Restoran 40.000 €:
//! učinak 2105 → 2666 bodova u dvije mjesečne faze), rezervni tekst s obnovom, zaključana obnova nije nova odluka,
//! dobit iz poslovanja nakon ulaganja uz indeks Financije (samo v2), redak za brže propadanje uz odgođeno održavanje,
//! redak za najamninu umanjenu zbog stanja prostora i sezonski mentor s ulaganjima i amortizacijom. Bez AI-ja.
mod common;

use common::{GUEST, apply_ev, hotel_ids, play_through, renewal_snapshot, v2_game_from};
use content::{Snapshot, repo_dir};
use game::*;
use sim_hotel::{Maintenance, Profile};
use sim_hotel_v2::department::{DeptCode, PeriodAccounts};
use sim_hotel_v2::{PeriodKind, PeriodPlan};

fn me() -> HotelId {
    hotel_ids()[0]
}

fn week(n: u8) -> RoundKey {
    RoundKey::Week(n)
}

fn plan_v2(g: &Game, r: RoundKey) -> PeriodPlan {
    let CorePlan::V2(p) = g.effective_plan(me(), r).unwrap() else {
        panic!("v2 plan")
    };
    p
}

fn set_plan(g: &mut Game, r: RoundKey, p: PeriodPlan) {
    let ev = g.set_plan(me(), r, CorePlan::V2(p)).unwrap();
    apply_ev(g, Actor::Guest(GUEST), ev);
}

/// Tjedna igra s obnovom: Restoran 40.000 € odlučen u t06 (prvi tjedan koji počinje u srpnju; srpanj ima t06–t09),
/// obračunato do uključivo `through`.
fn restaurant_july_game(through: &str) -> Game {
    let g = v2_game_from(&renewal_snapshot(), 42, PeriodKind::Week);
    let mut g = play_through(g, "t05");
    let mut p = plan_v2(&g, week(6));
    p.departments
        .get_mut(&DeptCode("restaurant".into()))
        .unwrap()
        .renewal_cents = 4_000_000;
    set_plan(&mut g, week(6), p);
    play_through(g, through)
}

fn accounts(g: &Game, r: RoundKey) -> PeriodAccounts {
    let res = g.resolved[&r]
        .hotels
        .iter()
        .find(|x| x.hotel_id == me())
        .unwrap();
    PeriodAccounts::try_from(res.detail.as_ref().unwrap().accounts.as_ref().unwrap()).unwrap()
}

fn driver<'a>(f: &'a DebriefFacts, what: &str) -> &'a DriverFact {
    f.indices
        .iter()
        .flat_map(|i| &i.drivers)
        .find(|d| d.what == what)
        .unwrap_or_else(|| panic!("nema pokretača {what}"))
}

#[test]
fn debrief_input_carries_round_renewals_and_the_split_gain() {
    let g = restaurant_july_game("t06");
    let input = g.debrief_input(me(), week(6)).unwrap();
    let [r] = input.renewals.as_slice() else {
        panic!("jedna obnova: {:?}", input.renewals)
    };
    assert_eq!(r.name, "Restoran");
    assert!(!r.building);
    assert_eq!(r.month, dto::MonthCode::Jul);
    assert_eq!(r.total_cents, 4_000_000);
    assert_eq!(r.paid_this_round_cents, 1_000_000, "četiri tjedna srpnja");
    assert_eq!(r.rounds_left, 3);
    // Krivulja Restorana (G 50 bodova, K 55.000 €): E(40.000 €) = 2105, a 20.000 € + 20.000 € = 1333 + 1333.
    assert_eq!(r.gain_month_pts, 2105);
    assert_eq!(r.split_gain_pts, 2666);
    assert_eq!(r.life_years, 7);
    assert_eq!(r.depreciation_monthly_cents, 47_619);
    assert!(r.decided_this_round);
    let a = accounts(&g, week(6));
    let o = &a.departments[&DeptCode("restaurant".into())];
    assert_eq!(r.gained_this_round_pts, o.renewal_pts);
    assert_eq!(input.capex_cents, Some(1_000_000));
    assert_eq!(input.building_before_pts, Some(a.building.before_pts));
    assert_eq!(input.building_after_pts, Some(a.building.after_pts));
    // Obnova iz srpnja amortizira se tek od kolovoza; prije nje nije bilo ulaganja.
    assert_eq!(input.depreciation_round_cents, Some(0));
}

#[test]
fn plan_and_fallback_name_the_renewal_by_part_and_month() {
    let g = restaurant_july_game("t06");
    let input = g.debrief_input(me(), week(6)).unwrap();
    let f = debrief_facts(&input);
    assert_eq!(
        f.plan.renovation,
        "Restoran 40.000 € u srpnju (10.000 € ovaj tjedan, još 3 tjedna); zgrada ne"
    );
    let text = fallback_debrief(&input);
    assert!(
        text.contains(
            "Obnova: Restoran 40.000 € u srpnju (10.000 € ovaj tjedan, još 3 tjedna); zgrada ne."
        ),
        "{text}"
    );
    assert!(
        text.contains("isti iznos raspoređen na dva mjeseca dao bi +27 bodova umjesto +21 bod"),
        "{text}"
    );
    assert!(
        text.contains(
            "blagajna plaća cijeli iznos, a račun dobiti nosi samo amortizaciju (476 € mjesečno kroz 7 godina, od idućeg mjeseca)"
        ),
        "{text}"
    );
    assert!(text.ends_with("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki."));
    let json = serde_json::to_value(&f).unwrap();
    let o = &json["ulaganja"]["obnove"][0];
    assert_eq!(o["dio"], "Restoran");
    assert_eq!(o["odluka"], "nova odluka ovog tjedna");
    assert_eq!(
        o["u_dvije_mjesecne_faze"],
        "isti iznos raspoređen na dva mjeseca dao bi +27 bodova umjesto +21 bod"
    );
    let sent = serde_json::to_string(&f).unwrap();
    for internal in ["cents", "_pts", "4000000", "restaurant"] {
        assert!(!sent.contains(internal), "{internal} u činjenicama mentora");
    }
}

#[test]
fn locked_renewal_from_an_earlier_week_is_not_a_new_decision() {
    let g = restaurant_july_game("t07");
    let input = g.debrief_input(me(), week(7)).unwrap();
    let [r] = input.renewals.as_slice() else {
        panic!("zaključana obnova se i dalje plaća")
    };
    assert!(!r.decided_this_round);
    assert_eq!(r.rounds_left, 2);
    let f = debrief_facts(&input);
    assert_eq!(
        f.plan.renovation,
        "Restoran 40.000 € u srpnju (10.000 € ovaj tjedan, još 2 tjedna; odlučeno u ranijem tjednu, zaključano do kraja mjeseca); zgrada ne"
    );
    let json = serde_json::to_value(&f).unwrap();
    let decision = json["ulaganja"]["obnove"][0]["odluka"].as_str().unwrap();
    assert!(decision.contains("nije nova odluka"), "{decision}");
    // Rezerva ne ponavlja „blagajna plaća cijeli iznos" kao da je odluka nova.
    assert!(!fallback_debrief(&input).contains("blagajna plaća cijeli iznos"));
}

#[test]
fn finance_note_only_in_v2_and_names_profit_after_investment() {
    let v1 = common::june_resolved_game();
    let f1 = debrief_facts(
        &v1.debrief_input(common::my_hotel(&v1), sim_hotel::Month::Jun)
            .unwrap(),
    );
    assert!(f1.finance_note.is_none());
    assert!(f1.investments.is_none());

    // Pravi sadržaj v2: indeks Financije iz GOP-a nakon ulaganja (Leonova odluka O-2 d′).
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut g = v2_game_from(&snap, 42, PeriodKind::Month);
    let mut p = plan_v2(&g, RoundKey::parse("jun").unwrap());
    p.departments
        .get_mut(&DeptCode("restaurant".into()))
        .unwrap()
        .renewal_cents = 4_000_000;
    set_plan(&mut g, RoundKey::parse("jun").unwrap(), p);
    let g = play_through(g, "jul");
    let input = g
        .debrief_input(me(), RoundKey::parse("jul").unwrap())
        .unwrap();
    let note = debrief_facts(&input).finance_note.unwrap();
    let dep = input.depreciation_round_cents.unwrap();
    assert!(dep > 0, "lipanjska obnova amortizira se u srpnju");
    assert_eq!(note.depreciation_cents, dep);
    assert_eq!(
        note.gop_after_investment_cents,
        input.gop_cents.unwrap() - dep
    );
    assert_eq!(note.ebit_cents, input.ebit_cents.unwrap());
    assert!(
        note.text
            .starts_with("Indeks Financije gleda dobit iz poslovanja nakon ulaganja"),
        "{}",
        note.text
    );
    assert!(
        note.text
            .contains("Najam zgrade, osiguranje i porez ne ulaze u ovaj indeks."),
        "{}",
        note.text
    );
}

#[test]
fn deferred_maintenance_gets_its_own_why_row() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut g = v2_game_from(&snap, 42, PeriodKind::Month);
    let jun = RoundKey::parse("jun").unwrap();
    let mut p = plan_v2(&g, jun);
    p.departments
        .get_mut(&DeptCode("restaurant".into()))
        .unwrap()
        .maintenance = Maintenance::Deferred;
    set_plan(&mut g, jun, p);
    let g = play_through(g, "jun");
    let input = g.debrief_input(me(), jun).unwrap();
    let [d] = input.deferred_wear.as_slice() else {
        panic!(
            "samo Restoran ima odgođeno održavanje: {:?}",
            input.deferred_wear
        )
    };
    assert_eq!(d.name, "Restoran");
    assert_eq!(d.factor_bp, 12_500);
    // Restoran se troši 300 bodova mjesečno; odgođeno ×1,25 = 375, pa je dodatni pad 75 (0,75 → 0,8 boda).
    assert_eq!(d.extra_pts, -75);
    let f = debrief_facts(&input);
    let row = driver(&f, "Oprema")
        .changes
        .iter()
        .find(|c| c.cause.contains("zapušten prostor propada brže"))
        .expect("redak za brže propadanje");
    assert_eq!(
        row.cause,
        "zapušten prostor propada brže: Restoran (održavanje odgođeno)"
    );
    assert_eq!(row.amount, "\u{2212}0,8 boda stanja prostora");
    assert_eq!(row.effect, Effect::Down);
    assert!(fallback_debrief(&input).contains(
        "Održavanje odjela Restoran je odgođeno, pa zapušten prostor propada 1,25 puta brže"
    ),);
}

#[test]
fn rent_cut_by_shop_condition_gets_its_own_why_row() {
    // Lokal ima samo profil B: sjeme u kojem igrač vodi hotel profila B.
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut g = (0..60)
        .map(|seed| v2_game_from(&snap, seed, PeriodKind::Month))
        .find(|g| {
            g.hotels
                .iter()
                .any(|h| h.setup.hotel_id == me() && h.profile == Profile::B)
        })
        .expect("sjeme s igračem profila B");
    let jun = RoundKey::parse("jun").unwrap();
    let mut p = plan_v2(&g, jun);
    p.departments
        .get_mut(&DeptCode("retail_lease".into()))
        .unwrap()
        .maintenance = Maintenance::Deferred;
    set_plan(&mut g, jun, p);
    let g = play_through(g, "jun");
    let input = g.debrief_input(me(), jun).unwrap();
    let [cut] = input.rent_cuts.as_slice() else {
        panic!("lokal ispod početnog stanja: {:?}", input.rent_cuts)
    };
    // Lipanj je cijeli mjesec: ugovorena najamnina 6.000 € umanjena je točno za iznos retka.
    let a = accounts(&g, jun);
    let revenue = a.departments[&DeptCode("retail_lease".into())].revenue_cents;
    assert_eq!(revenue + cut.cut_cents, 600_000);
    assert!(cut.condition_pts < cut.threshold_pts);
    let f = debrief_facts(&input);
    let row = driver(&f, "Prihod")
        .changes
        .iter()
        .find(|c| {
            c.cause
                .starts_with("najamnina umanjena zbog stanja prostora")
        })
        .expect("redak najamnine");
    assert_eq!(
        row.cause,
        format!("najamnina umanjena zbog stanja prostora: {}", cut.name)
    );
    assert!(row.amount.starts_with('\u{2212}'), "{}", row.amount);
    assert_eq!(row.effect, Effect::Down);
    assert!(fallback_debrief(&input).contains("umanjena je zbog stanja prostora za"));
}

#[test]
fn season_mentor_gets_renewals_by_part_and_season_depreciation() {
    let g = restaurant_july_game("all");
    let input = g.season_debrief_input(me()).unwrap();
    let f = season_debrief_facts(&input);
    let d = f.decisions.as_ref().unwrap();
    assert!(
        d.renovation.contains("Restoran 40.000 € u srpnju"),
        "{}",
        d.renovation
    );
    let fin = f.finances.as_ref().unwrap();
    let invested = fin.investments.as_deref().unwrap();
    assert!(invested.starts_with("ukupno "), "{invested}");
    let dep = fin.depreciation.as_deref().unwrap();
    let season = input.me.hotel.finances.as_ref().unwrap();
    assert!(
        season.depreciation_cents > 0,
        "srpanjska obnova amortizira se u kolovozu i rujnu"
    );
    assert!(
        dep.starts_with(&format!("{} €", (season.depreciation_cents + 50) / 100)),
        "{dep}"
    );
    assert!(fin.gop_after_investment.is_some());
    let json = serde_json::to_string(&f).unwrap();
    for key in [
        "ulaganja_u_obnovu",
        "amortizacija_sezone",
        "dobit_iz_poslovanja_nakon_ulaganja",
    ] {
        assert!(json.contains(key), "{key}");
    }
}
