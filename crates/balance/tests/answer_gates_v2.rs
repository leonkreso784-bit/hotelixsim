//! B3b: brane modela za situacije na zadanom planu (putovi odgovora iz B3a). `--check` pada kad situacije postanu
//! nevažne, obrnute ili presudne; robot pritom ne igra nikakvu ulogu. Ravnalo težine (Leon 15.10., opcija 1) je
//! konstanta u kodu, izmjerena jednom; ovdje se samo čita preko `plan_ruler`.
//!
//! Izvještaj se uzima iz pravog `run_v2` (3 sjemena), pa se putovi prepišu "čistim" brojkama; tako test mjeri samo
//! brane, a ne igru. Svaki test gleda samo poruke koje počinju s "B3".
mod common;

use balance::v2::{
    ORDER_DEBT, ORDER_DEBT_FLOOR, OptionsV2, PathReport, ReportV2, WEIGHT_BAND_BP, failures_v2,
    plan_ruler, render_v2, robot_measures_v2, run_v2,
};
use common::content_v2;
use sim_hotel_v2::PeriodKind;

fn report(kind: PeriodKind) -> ReportV2 {
    run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind,
            situations: None,
        },
    )
    .unwrap()
}

fn paths_mut(r: &mut ReportV2) -> &mut PathReport {
    &mut r.situations.as_mut().expect("repo ima predloške").paths
}

/// Izvještaj čiji su putovi svugdje zdravi: trošak u istom krugu, dobit nakon horizonta i na kraju, svi koraci lanca
/// pozitivni, težina oko 25 % (Δ = ravnalo / 3). U mjesečnom krugu je korak „srednji − prepušteno" 0 jer je to
/// zapisani dug (ORDER_DEBT); tjedno je pozitivan.
fn clean(kind: PeriodKind) -> ReportV2 {
    let mut r = report(kind);
    for row in &mut paths_mut(&mut r).rows {
        let ruler = plan_ruler(kind, row.profile);
        row.best_vs_silent = [-50, 100, ruler / 3];
        row.steps = if kind == PeriodKind::Month {
            [100, 0, 100]
        } else {
            [100, 100, 100]
        };
    }
    r
}

fn b3(r: &ReportV2) -> Vec<String> {
    failures_v2(r)
        .into_iter()
        .filter(|x| x.starts_with("B3"))
        .collect()
}

fn has(f: &[String], parts: &[&str]) -> bool {
    f.iter().any(|x| parts.iter().all(|p| x.contains(p)))
}

#[test]
fn ruler_is_a_plausible_constant_per_kind_and_profile() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for row in &report(kind).situations.unwrap().paths.rows {
            let ruler = plan_ruler(kind, row.profile);
            // Anketa 15.10.: razlika dobar/slab plan je oko 6,8-10,9 boda (680-1090 stotinki).
            assert!((300..=3_000).contains(&ruler), "{kind:?} {ruler}");
        }
    }
    assert_eq!(WEIGHT_BAND_BP, (500, 5_000));
}

#[test]
fn clean_report_passes_all_b3_gates() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        assert_eq!(b3(&clean(kind)), Vec::<String>::new(), "{kind:?}");
    }
}

#[test]
fn answer_gates_fail_check() {
    let mut r = clean(PeriodKind::Week);
    paths_mut(&mut r).rows[0].best_vs_silent[2] = -100;
    let code = paths_mut(&mut r).rows[0].profile.code();
    let f = b3(&r);
    assert!(
        has(
            &f,
            &["B3 odgovori", "na kraju sezone", code, "cilj više od 0"]
        ),
        "{f:?}"
    );
    // Robot ne smije biti u igri: mjere robota nemaju B3 poruka.
    assert!(!robot_measures_v2(&r).iter().any(|x| x.contains("B3")));
}

#[test]
fn zero_is_not_better_than_silence() {
    let mut r = clean(PeriodKind::Week);
    paths_mut(&mut r).rows[1].best_vs_silent[2] = 0;
    assert!(has(&b3(&r), &["B3 odgovori", "na kraju sezone"]));
}

#[test]
fn late_payoff_gate_uses_horizon() {
    for (kind, h) in [
        (PeriodKind::Week, "nakon 4. kruga"),
        (PeriodKind::Month, "nakon 2. kruga"),
    ] {
        let mut r = clean(kind);
        paths_mut(&mut r).rows[0].best_vs_silent[1] = 0;
        let f = b3(&r);
        assert!(has(&f, &["B3 odgovori", h]), "{kind:?}: {f:?}");
        assert!(!has(&f, &["na kraju sezone"]), "{kind:?}: {f:?}");
    }
}

#[test]
fn same_round_cost_is_not_a_gate() {
    let mut r = clean(PeriodKind::Week);
    for row in &mut paths_mut(&mut r).rows {
        let ruler = plan_ruler(PeriodKind::Week, row.profile);
        row.best_vs_silent = [-500, 100, ruler / 3];
        row.steps = [100; 3];
    }
    assert_eq!(b3(&r), Vec::<String>::new());
}

#[test]
fn order_gate_fails() {
    let mut r = clean(PeriodKind::Week);
    paths_mut(&mut r).rows[0].steps[1] = 0;
    let f = b3(&r);
    assert!(
        has(
            &f,
            &[
                "B3 redoslijed",
                "srednji prema prepušteno",
                "na kraju sezone"
            ]
        ),
        "{f:?}"
    );
    for (i, names) in [(0, ["najbolji", "srednji"]), (2, ["prepušteno", "šutnja"])] {
        let mut r = clean(PeriodKind::Week);
        paths_mut(&mut r).rows[0].steps[i] = -1;
        let f = b3(&r);
        assert!(
            has(&f, &["B3 redoslijed", names[0], "prema", names[1]]),
            "{f:?}"
        );
    }
}

#[test]
fn monthly_debt_forgives_only_middle_minus_delegated() {
    // Dug: srednji − prepušteno u mjesečnom krugu smije biti 0 (clean mjesečni već to ima, nema B3 poruke).
    assert!(!has(&b3(&clean(PeriodKind::Month)), &["B3 redoslijed"]));
    // Ostali koraci mjesečno nemaju dug.
    for i in [0, 2] {
        let mut r = clean(PeriodKind::Month);
        paths_mut(&mut r).rows[0].steps[i] = 0;
        let f = b3(&r);
        assert!(has(&f, &["B3 redoslijed"]), "korak {i}: {f:?}");
    }
    // Dug je današnji negativan korak (do strop-a): -5 je oprošteno, ispod stropa pada.
    assert!(
        ORDER_DEBT_FLOOR > -100 && ORDER_DEBT_FLOOR < 0,
        "{ORDER_DEBT_FLOOR}"
    );
    let mut r = clean(PeriodKind::Month);
    paths_mut(&mut r).rows[0].steps[1] = -5;
    assert!(!has(&b3(&r), &["B3 redoslijed"]), "{:?}", b3(&r));
    paths_mut(&mut r).rows[0].steps[1] = ORDER_DEBT_FLOOR;
    assert!(!has(&b3(&r), &["B3 redoslijed"]), "{:?}", b3(&r));
    paths_mut(&mut r).rows[0].steps[1] = -30;
    let f = b3(&r);
    assert!(
        has(&f, &["B3 redoslijed", "srednji prema prepušteno"]),
        "{f:?}"
    );
}

#[test]
fn paid_off_debt_must_leave_the_list() {
    // Mjesečno srednji − prepušteno > 0 na svim profilima: dug je otplaćen i mora se maknuti.
    let mut r = clean(PeriodKind::Month);
    for row in &mut paths_mut(&mut r).rows {
        row.steps = [100, 100, 100];
    }
    let f = b3(&r);
    assert!(has(&f, &["B3 dug:", "L1-13"]), "{f:?}");
    // Dok ijedan profil još duguje, dug stoji.
    paths_mut(&mut r).rows[2].steps[1] = 0;
    assert!(!has(&b3(&r), &["B3 dug:"]), "{:?}", b3(&r));
    // Tjedno nema duga, pa ni poruke.
    assert!(!has(&b3(&clean(PeriodKind::Week)), &["B3 dug:"]));
}

#[test]
fn order_debt_list_has_one_entry_and_only_shrinks() {
    assert!(
        ORDER_DEBT.len() <= 1,
        "dug smije samo padati: {ORDER_DEBT:?}"
    );
    assert!(
        ORDER_DEBT.iter().all(|d| *d == (PeriodKind::Month, 1)),
        "jedini dopušteni dug: (mjesečni krug, korak srednji − prepušteno): {ORDER_DEBT:?}"
    );
    assert_eq!(
        ORDER_DEBT.len(),
        1,
        "dok dug stoji, popis ima točno jednu stavku"
    );
}

#[test]
fn weight_gate_fails() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        // Presudne: 9000 bp, Δ = 9 × ravnalo (share_bp = |Δ| / (|Δ| + ravnalo)).
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[0];
        row.best_vs_silent[2] = 9 * plan_ruler(kind, row.profile);
        let p = row.profile.code();
        let f = b3(&r);
        assert!(
            has(&f, &["B3 težina", p, "situacije nose", "cilj"]),
            "{kind:?}: {f:?}"
        );
        assert!(!has(&f, &["B3 odgovori"]), "{kind:?}: {f:?}");
        // Nevažne: Δ = 1 stotinka (manje od 5 %).
        let mut r = clean(kind);
        paths_mut(&mut r).rows[1].best_vs_silent[2] = 1;
        let p = paths_mut(&mut r).rows[1].profile.code();
        assert!(has(&b3(&r), &["B3 težina", p]), "{kind:?}: {:?}", b3(&r));
        // Gornji rub pojasa prolazi: Δ = ravnalo je točno 50 %.
        let mut r = clean(kind);
        for row in &mut paths_mut(&mut r).rows {
            row.best_vs_silent[2] = plan_ruler(kind, row.profile);
        }
        assert_eq!(
            b3(&r),
            Vec::<String>::new(),
            "{kind:?}: Δ = ravnalo je 50 %"
        );
    }
}

#[test]
fn negative_delta_never_passes_as_weight_band() {
    // |Δ| je u pojasu (share_bp uzima apsolutnu vrijednost), ali Δ < 0 mora pasti na "B3 odgovori".
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[0];
        row.best_vs_silent[2] = -(plan_ruler(kind, row.profile) / 3);
        let f = b3(&r);
        assert!(!f.is_empty(), "{kind:?}: obrnuta težina ne smije proći");
        assert!(
            has(&f, &["B3 odgovori", "na kraju sezone"]),
            "{kind:?}: {f:?}"
        );
        assert!(
            !has(&f, &["B3 težina"]),
            "{kind:?}: Δ ≤ 0 nije za branu težine: {f:?}"
        );
    }
}

#[test]
fn b3_messages_are_not_robot_measures() {
    let mut r = clean(PeriodKind::Week);
    paths_mut(&mut r).rows[0].best_vs_silent = [0, 0, -1];
    paths_mut(&mut r).rows[0].steps = [0, 0, 0];
    assert!(!robot_measures_v2(&r).iter().any(|x| x.contains("B3")));
    let text = render_v2(&r);
    let robots = text.split("== Mjere robota").nth(1).unwrap_or("");
    assert!(!robots.contains("B3 "), "B3 poruke nisu mjere robota");
}

#[test]
fn print_keeps_only_the_ruler_weight_line() {
    let text = render_v2(&report(PeriodKind::Week));
    assert!(text.contains("prema rasponu plana"), "{text}");
    assert!(
        !text.contains("prema zadanom planu bez situacija"),
        "stari redak težine odlazi (anketa 15.10., opcija 1)"
    );
}
