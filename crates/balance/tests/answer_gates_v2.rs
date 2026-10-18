//! B3b: brane modela za situacije na zadanom planu (putovi odgovora iz B3a). `--check` pada kad situacije postanu
//! nevažne, obrnute ili presudne; robot pritom ne igra nikakvu ulogu. Ravnalo težine (Leon 15.10., opcija 1) je
//! konstanta u kodu, izmjerena jednom; ovdje se samo čita preko `plan_ruler`.
//! PRAVILO-b: lanac ima četiri koraka (najbolji > klik najbolji > srednji > prepušteno > šutnja); težina se i dalje
//! mjeri na vlastitom odgovoru (`best_vs_silent`); L1-13 dio 1b dodaje branu „B3 klik": omjer Δ klika
//! i vlastitog najboljeg na situacijama s likom (`click_vs_mute` / `best_vs_mute`, M1 Leon 18.10.) najviše 75 %; ukupna
//! težina klika (`click_vs_silent`) samo se ispisuje.
//!
//! L1-13 dio 4, runda 2 (Leon 18.10., četvrta anketa, K4): pojas težine „situacije ~20 %" je PROSJEK tri hotela
//! (A, B, C) u 15–25 % (`WEIGHT_BAND_BP`; zbroj tri udjela 4 500–7 500 bp, bez zaokruživanja prosjeka), a svaki hotel
//! posebno 10–35 % (`WEIGHT_HOTEL_BAND_BP`), tjedno i mjesečno. Poruke: po hotelu `B3 težina: {p} situacije nose X
//! (cilj 10,0 %–35,0 %)`, za prosjek `B3 težina: prosjek A, B, C situacije nose X (cilj 15,0 %–25,0 %)`; prosjek se
//! računa samo kad su sve tri Δ > 0 (inače već pada „B3 odgovori").
//!
//! Izvještaj se uzima iz pravog `run_v2` (3 sjemena), pa se putovi prepišu "čistim" brojkama; tako test mjeri samo
//! brane, a ne igru. Svaki test gleda samo poruke koje počinju s "B3".
mod common;

use std::sync::OnceLock;

use balance::v2::{
    CLICK_WEIGHT_MAX_BP, OptionsV2, PathReport, ReportV2, WEIGHT_BAND_BP, WEIGHT_HOTEL_BAND_BP,
    failures_v2, plan_ruler, render_v2, robot_measures_v2, run_v2,
};
use common::content_v2;
use sim_hotel_v2::{PeriodKind, Profile};

/// Izvještaj pravog `run_v2` (3 sjemena), izračunan jednom po vrsti kruga (deterministički; testovi dobivaju kopiju).
fn report(kind: PeriodKind) -> ReportV2 {
    static WEEK: OnceLock<ReportV2> = OnceLock::new();
    static MONTH: OnceLock<ReportV2> = OnceLock::new();
    let cell = match kind {
        PeriodKind::Week => &WEEK,
        PeriodKind::Month => &MONTH,
    };
    cell.get_or_init(|| {
        run_v2(
            &content_v2(),
            &OptionsV2 {
                seeds: 3,
                kind,
                situations: None,
            },
        )
        .unwrap()
    })
    .clone()
}

fn paths_mut(r: &mut ReportV2) -> &mut PathReport {
    &mut r.situations.as_mut().expect("repo ima predloške").paths
}

/// Izvještaj čiji su putovi svugdje zdravi: trošak u istom krugu, dobit nakon horizonta i na kraju, svi koraci lanca
/// pozitivni, težina 20 % (Δ = ravnalo / 4; L1-13 dio 4, Leon A3 „20 %"). Dug je otplaćen u B1a (15.10., bonus +1),
/// a mehanizam duga je obrisan u L1-13 dio 4.
fn clean(kind: PeriodKind) -> ReportV2 {
    let mut r = report(kind);
    for row in &mut paths_mut(&mut r).rows {
        let ruler = plan_ruler(kind, row.profile);
        row.best_vs_silent = [-50, 100, ruler / 4];
        // Klik nosi 14,3 % uz 20 % vlastitog: omjer 71 % (cilj 70–75 %, brana najviše 75 %).
        row.click_vs_silent = ruler / 6;
        // Isto na situacijama s likom (Δ prema šutnji tamo): omjer 66,7 %, brana najviše 75 % (M1, Leon 18.10.).
        row.best_vs_mute = ruler / 4;
        row.click_vs_mute = ruler / 6;
        row.steps = [100; 4];
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
fn ruler_is_pinned_and_changes_only_by_leons_decision() {
    // Izmjereno 15.10.2026 (76271d1, 1000 sjemena). Ravnalo se mijenja SAMO Leonovom odlukom u kalibraciji L1-13.
    let pinned = [
        (PeriodKind::Week, Profile::A, 845),
        (PeriodKind::Week, Profile::B, 678),
        (PeriodKind::Week, Profile::C, 1_033),
        (PeriodKind::Month, Profile::A, 812),
        (PeriodKind::Month, Profile::B, 679),
        (PeriodKind::Month, Profile::C, 1_090),
    ];
    for (kind, profile, v) in pinned {
        assert_eq!(
            plan_ruler(kind, profile),
            v,
            "{kind:?} {profile:?}: ravnalo se mijenja samo Leonovom odlukom (L1-13)"
        );
    }
    assert_eq!(WEIGHT_BAND_BP, (1_500, 2_500));
    assert_eq!(WEIGHT_HOTEL_BAND_BP, (1_000, 3_500));
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
        row.best_vs_silent = [-500, 100, ruler / 4];
        row.steps = [100; 4];
    }
    assert_eq!(b3(&r), Vec::<String>::new());
}

/// Četiri koraka lanca, redom: najbolji − klik najbolji, klik najbolji − srednji, srednji − prepušteno,
/// prepušteno − šutnja.
const STEPS: [(&str, &str); 4] = [
    ("najbolji", "klik najbolji"),
    ("klik najbolji", "srednji"),
    ("srednji", "prepušteno"),
    ("prepušteno", "šutnja"),
];

#[test]
fn order_gate_fails() {
    for (i, (a, b)) in STEPS.into_iter().enumerate() {
        for v in [0, -1] {
            let mut r = clean(PeriodKind::Week);
            paths_mut(&mut r).rows[0].steps[i] = v;
            let f = b3(&r);
            assert!(
                has(
                    &f,
                    &[
                        "B3 redoslijed",
                        &format!("{a} prema {b}"),
                        "na kraju sezone"
                    ]
                ),
                "korak {i} = {v}: {f:?}"
            );
            assert_eq!(f.len(), 1, "korak {i} = {v}: pada samo taj korak: {f:?}");
        }
    }
}

#[test]
fn click_steps_are_gates() {
    // Doslovna poruka iz kartice (profil A je prvi redak, korak 0.00).
    let mut r = clean(PeriodKind::Week);
    let p = paths_mut(&mut r).rows[0].profile.code();
    paths_mut(&mut r).rows[0].steps[0] = 0;
    assert!(
        b3(&r).contains(&format!(
            "B3 redoslijed: {p} najbolji prema klik najbolji 0.00 na kraju sezone (cilj više od 0)"
        )),
        "{:?}",
        b3(&r)
    );
    let mut r = clean(PeriodKind::Month);
    paths_mut(&mut r).rows[1].steps[1] = -3;
    let p = paths_mut(&mut r).rows[1].profile.code();
    assert!(
        b3(&r).contains(&format!(
            "B3 redoslijed: {p} klik najbolji prema srednji -0.03 na kraju sezone (cilj više od 0)"
        )),
        "{:?}",
        b3(&r)
    );
}

#[test]
fn monthly_middle_minus_delegated_is_a_real_gate() {
    // Dug je otplaćen (B1a, 15.10.) i mehanizam duga obrisan (L1-13 dio 4): mjesečno srednji − prepušteno ≤ 0 pada
    // kao i tjedno, bez oprosta; −25 je bivši strop duga (`ORDER_DEBT_FLOOR`), doslovno.
    for v in [0, -5, -25] {
        let mut r = clean(PeriodKind::Month);
        paths_mut(&mut r).rows[0].steps[2] = v;
        let f = b3(&r);
        assert!(
            has(&f, &["B3 redoslijed", "srednji prema prepušteno"]),
            "korak {v}: {f:?}"
        );
    }
    for i in [0, 1, 3] {
        let mut r = clean(PeriodKind::Month);
        paths_mut(&mut r).rows[0].steps[i] = 0;
        assert!(has(&b3(&r), &["B3 redoslijed"]), "korak {i}");
    }
}

/// L1-13 dio 4 (test čuvara 2): mehanizma duga redoslijeda više nema: ni popisa (`ORDER_DEBT`), ni stropa
/// (`ORDER_DEBT_FLOOR`), ni oprosta, ni poruke „B3 dug". Korak lanca ≤ 0 uvijek pada, tjedno i mjesečno, na svakom
/// koraku i profilu, koliko god bio malen ili velik.
#[test]
fn debt_mechanism_is_gone() {
    for (file, src) in [
        ("v2_answers.rs", include_str!("../src/v2_answers.rs")),
        ("v2.rs", include_str!("../src/v2.rs")),
    ] {
        for word in ["ORDER_DEBT", "B3 dug", "forgiven"] {
            assert!(
                !src.contains(word),
                "crates/balance/src/{file}: '{word}' još postoji; mehanizam duga se briše u L1-13 dio 4"
            );
        }
    }
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for (i, (a, b)) in STEPS.into_iter().enumerate() {
            for v in [0, -1, -14, -25, -26, -10_000] {
                for row in 0..3 {
                    let mut r = clean(kind);
                    paths_mut(&mut r).rows[row].steps[i] = v;
                    let f = b3(&r);
                    assert!(
                        has(&f, &["B3 redoslijed", &format!("{a} prema {b}")]),
                        "{kind:?} red {row} korak {i} = {v}: {f:?}"
                    );
                    assert!(!has(&f, &["B3 dug"]), "{kind:?}: {f:?}");
                }
            }
        }
        // Svi koraci pozitivni: nema poruke ni o redoslijedu ni o dugu.
        assert!(!has(&b3(&clean(kind)), &["B3 redoslijed"]), "{kind:?}");
        assert!(!has(&b3(&clean(kind)), &["B3 dug"]), "{kind:?}");
    }
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
        // Nevažne: Δ = 1 stotinka (manje od 10 %).
        let mut r = clean(kind);
        paths_mut(&mut r).rows[1].best_vs_silent[2] = 1;
        let p = paths_mut(&mut r).rows[1].profile.code();
        assert!(has(&b3(&r), &["B3 težina", p]), "{kind:?}: {:?}", b3(&r));
        // Gornji rub prosjeka prolazi: Δ = ravnalo / 3 je 25 % na svakom hotelu (cjelobrojno ne više); korak iznad na
        // sva tri hotela pada samo na prosjeku (svaki hotel je i dalje u 10–35 %).
        let mut r = clean(kind);
        for row in &mut paths_mut(&mut r).rows {
            row.best_vs_silent[2] = plan_ruler(kind, row.profile) / 3;
        }
        assert_eq!(
            b3(&r),
            Vec::<String>::new(),
            "{kind:?}: Δ = ravnalo / 3 je 25 %"
        );
        for row in &mut paths_mut(&mut r).rows {
            row.best_vs_silent[2] += 1;
        }
        let f = b3(&r);
        assert_eq!(
            f.len(),
            1,
            "{kind:?}: korak iznad 25 % pada na prosjeku: {f:?}"
        );
        assert!(f[0].starts_with(AVERAGE), "{f:?}");
    }
}

const AVERAGE: &str = "B3 težina: prosjek A, B, C situacije nose ";
const AVERAGE_GOAL: &str = " (cilj 15,0 %–25,0 %)";
const HOTEL_GOAL: &str = " (cilj 10,0 %–35,0 %)";

/// Udio situacija (bp) kao `share_bp` u balansu: Δ · 10 000 / (Δ + ravnalo), dolje (Δ > 0).
fn share(delta: i32, ruler: i32) -> i32 {
    i32::try_from(i64::from(delta) * 10_000 / (i64::from(delta) + i64::from(ruler))).unwrap()
}

/// Najmanja Δ (stotinke) s udjelom barem `bp` uz ravnalo `ruler`.
fn delta_for(bp: i32, ruler: i32) -> i32 {
    (1..).find(|&d| share(d, ruler) >= bp).unwrap()
}

/// Ravnalo hotela `k` (redom A, B, C kao u izvještaju).
fn ruler_of(kind: PeriodKind, k: usize) -> i32 {
    plan_ruler(kind, Profile::ALL[k])
}

/// Izvještaj u kojem hotel `k` ima Δ `delta`, a druga dva udio `others` bp (najmanja Δ s tim udjelom); vraća i zbroj
/// udjela druga dva.
fn with_shares(kind: PeriodKind, k: usize, delta: i32, others: i32) -> (ReportV2, i32) {
    let mut r = clean(kind);
    let mut rest = 0;
    for (i, row) in paths_mut(&mut r).rows.iter_mut().enumerate() {
        assert_eq!(row.profile, Profile::ALL[i], "redovi su redom A, B, C");
        let ruler = plan_ruler(kind, row.profile);
        if i == k {
            row.best_vs_silent[2] = delta;
        } else {
            row.best_vs_silent[2] = delta_for(others, ruler);
            rest += share(row.best_vs_silent[2], ruler);
        }
    }
    (r, rest)
}

/// L1-13 dio 4 (runda 2, Leon 18.10. K4): pojas „situacije ~20 %" je prosjek tri hotela u 15–25 % (zbroj udjela
/// 4 500–7 500 bp); oba ruba prolaze, korak izvan pada samo porukom prosjeka, tjedno i mjesečno, za svaki hotel.
#[test]
fn weight_band_is_average_of_three_hotels_fifteen_to_twenty_five() {
    assert_eq!(WEIGHT_BAND_BP, (1_500, 2_500));
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        // Stvarni slučaj (dijagnoza 18.10., mjesečno): jedan hotel 31 %, druga dva 18 % i 16 % — prosjek 21,7 % prolazi.
        for k in 0..3 {
            let mut r = clean(kind);
            for (i, row) in paths_mut(&mut r).rows.iter_mut().enumerate() {
                let bp = [3_100, 1_800, 1_600][(i + 3 - k) % 3];
                row.best_vs_silent[2] = delta_for(bp, plan_ruler(kind, row.profile));
            }
            assert_eq!(
                b3(&r),
                Vec::<String>::new(),
                "{kind:?}: hotel {k} 31 %, prosjek ~22 %"
            );
        }
        for k in 0..3 {
            let ruler = ruler_of(kind, k);
            // Gornji rub: druga dva po 22 %, hotel k najveća Δ uz zbroj ≤ 7 500 (tada sam ~31 %, ispod 35 %).
            let (_, rest) = with_shares(kind, k, 1, 2_200);
            let top = (1..)
                .take_while(|&d| rest + share(d, ruler) <= 7_500)
                .last()
                .unwrap();
            assert!(
                share(top + 1, ruler) <= 3_500,
                "rub prosjeka ne smije dirati rub hotela"
            );
            let (r, _) = with_shares(kind, k, top, 2_200);
            assert_eq!(
                b3(&r),
                Vec::<String>::new(),
                "{kind:?} hotel {k}: prosjek 25 % prolazi"
            );
            let (r, _) = with_shares(kind, k, top + 1, 2_200);
            let f = b3(&r);
            assert_eq!(
                f.len(),
                1,
                "{kind:?} hotel {k}: prosjek iznad 25 % pada: {f:?}"
            );
            assert!(
                f[0].starts_with(AVERAGE) && f[0].ends_with(AVERAGE_GOAL),
                "{kind:?} hotel {k}: {f:?}"
            );
            // Donji rub: druga dva po 12 %, hotel k najmanja Δ uz zbroj ≥ 4 500 (tada sam ~21 %, iznad 10 %).
            let (_, rest) = with_shares(kind, k, 1, 1_200);
            let low = (1..).find(|&d| rest + share(d, ruler) >= 4_500).unwrap();
            assert!(
                share(low - 1, ruler) >= 1_000,
                "rub prosjeka ne smije dirati rub hotela"
            );
            let (r, _) = with_shares(kind, k, low, 1_200);
            assert_eq!(
                b3(&r),
                Vec::<String>::new(),
                "{kind:?} hotel {k}: prosjek 15 % prolazi"
            );
            let (r, _) = with_shares(kind, k, low - 1, 1_200);
            let f = b3(&r);
            assert_eq!(
                f.len(),
                1,
                "{kind:?} hotel {k}: prosjek ispod 15 % pada: {f:?}"
            );
            assert!(
                f[0].starts_with(AVERAGE) && f[0].ends_with(AVERAGE_GOAL),
                "{kind:?} hotel {k}: {f:?}"
            );
        }
    }
}

/// L1-13 dio 4 (runda 2, Leon 18.10. K4): svaki hotel posebno nosi 10–35 %; oba ruba prolaze, korak izvan pada porukom
/// tog hotela (prosjek je pritom u pojasu), tjedno i mjesečno.
#[test]
fn weight_band_each_hotel_ten_to_thirty_five() {
    assert_eq!(WEIGHT_HOTEL_BAND_BP, (1_000, 3_500));
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for k in 0..3 {
            let (ruler, p) = (ruler_of(kind, k), Profile::ALL[k].code());
            // Gornji rub 35 %: najveća Δ s udjelom ≤ 3 500; druga dva po 15 % (zbroj ~6 500, prosjek u pojasu).
            let top = delta_for(3_501, ruler) - 1;
            // Donji rub 10 %: najmanja Δ s udjelom ≥ 1 000; druga dva po 25 % (zbroj ~6 000).
            let low = delta_for(1_000, ruler);
            for (delta, others, ok) in [
                (top, 1_500, true),
                (top + 1, 1_500, false),
                (low, 2_500, true),
                (low - 1, 2_500, false),
            ] {
                let (r, rest) = with_shares(kind, k, delta, others);
                assert!(
                    (4_500..=7_500).contains(&(rest + share(delta, ruler))),
                    "{kind:?} {p} Δ {delta}: prosjek u pojasu"
                );
                let f = b3(&r);
                if ok {
                    assert_eq!(
                        f,
                        Vec::<String>::new(),
                        "{kind:?} {p} Δ {delta}: rub prolazi"
                    );
                } else {
                    assert_eq!(f.len(), 1, "{kind:?} {p} Δ {delta}: izvan ruba pada: {f:?}");
                    assert!(
                        f[0].starts_with(&format!("B3 težina: {p} situacije nose "))
                            && f[0].ends_with(HOTEL_GOAL),
                        "{kind:?} {p}: {f:?}"
                    );
                }
            }
        }
        // Stari pojas po hotelu 15–25 % više ne vrijedi: hotel na 12 % ili 30 % uz ostale na 20 % prolazi.
        for bp in [1_200, 3_000] {
            let (r, _) = with_shares(kind, 0, delta_for(bp, ruler_of(kind, 0)), 2_000);
            assert_eq!(
                b3(&r),
                Vec::<String>::new(),
                "{kind:?}: hotel A {bp} bp uz ostale 20 %"
            );
        }
        // Stari pojas 5–50 %: hotel na 6 % ili 45 % pada i kad je prosjek u pojasu.
        for (bp, others) in [(600, 2_400), (4_500, 1_400)] {
            let (r, _) = with_shares(kind, 0, delta_for(bp, ruler_of(kind, 0)), others);
            let p = Profile::A.code();
            assert!(
                has(&b3(&r), &["B3 težina", p, HOTEL_GOAL.trim()]),
                "{kind:?}: hotel A {bp} bp: {:?}",
                b3(&r)
            );
        }
    }
}

#[test]
fn negative_delta_never_passes_as_weight_band() {
    // |Δ| je u pojasu (share_bp uzima apsolutnu vrijednost), ali Δ < 0 mora pasti na "B3 odgovori".
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[0];
        row.best_vs_silent[2] = -(plan_ruler(kind, row.profile) / 4);
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
    paths_mut(&mut r).rows[0].steps = [0; 4];
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

/// Brana klika (L1-13 dio 1b, Leon 18.10., M1): na situacijama s likom Δ klika (prema šutnji tamo) najviše 75 % Δ
/// vlastitog najboljeg (`click_vs_mute` / `best_vs_mute`), cjelobrojno. Najveća Δ klika koja još prolazi uz dani najbolji.
fn edge_click(best: i32) -> i32 {
    let c = best * CLICK_WEIGHT_MAX_BP / 10_000;
    assert!(i64::from(c) * 10_000 <= i64::from(CLICK_WEIGHT_MAX_BP) * i64::from(best));
    assert!(i64::from(c + 1) * 10_000 > i64::from(CLICK_WEIGHT_MAX_BP) * i64::from(best));
    c
}

#[test]
fn click_weight_limit_is_three_quarters() {
    assert_eq!(CLICK_WEIGHT_MAX_BP, 7_500);
}

#[test]
fn click_weight_gate_fails_above_three_quarters() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        // Doslovna poruka iz kartice: klik jednako težak kao vlastiti najbolji = 100,0 % (profil A je prvi redak).
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[0];
        row.click_vs_mute = row.best_vs_mute;
        let p = row.profile.code();
        assert_eq!(
            b3(&r),
            vec![format!(
                "B3 klik: {p} klik najbolji nosi 100,0 % težine vlastitog najboljeg (cilj najviše 75,0 %)"
            )],
            "{kind:?}"
        );
        // 80 % težine: pada samo taj profil i samo brana klika.
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[1];
        row.best_vs_mute = 1_000;
        row.click_vs_mute = 800;
        let p = row.profile.code();
        let f = b3(&r);
        assert_eq!(f.len(), 1, "{kind:?}: {f:?}");
        assert!(
            has(
                &f,
                &[
                    "B3 klik",
                    p,
                    "težine vlastitog najboljeg",
                    "(cilj najviše 75,0 %)"
                ]
            ),
            "{kind:?}: {f:?}"
        );
    }
}

#[test]
fn click_weight_gate_passes_at_three_quarters() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        // Točno na rubu (težina klika = 75 % težine najboljeg, cjelobrojno) prolazi, korak iznad pada.
        let mut r = clean(kind);
        for row in &mut paths_mut(&mut r).rows {
            row.best_vs_mute = plan_ruler(kind, row.profile);
            row.click_vs_mute = edge_click(row.best_vs_mute);
        }
        assert_eq!(b3(&r), Vec::<String>::new(), "{kind:?}: rub prolazi");
        for row in &mut paths_mut(&mut r).rows {
            row.click_vs_mute += 1;
        }
        let f = b3(&r);
        assert_eq!(
            f.len(),
            3,
            "{kind:?}: korak iznad ruba pada na sva tri profila: {f:?}"
        );
        assert!(f.iter().all(|x| x.starts_with("B3 klik")), "{f:?}");
    }
}

#[test]
fn click_gate_needs_positive_best() {
    // Uz Δ najboljeg ≤ 0 pada samo „B3 odgovori"; omjer težina nema smisla pa brane klika nema.
    for best in [0, -100] {
        for kind in [PeriodKind::Week, PeriodKind::Month] {
            let mut r = clean(kind);
            let row = &mut paths_mut(&mut r).rows[0];
            row.best_vs_silent[2] = best;
            row.best_vs_mute = best;
            row.click_vs_mute = 9 * plan_ruler(kind, row.profile);
            let f = b3(&r);
            assert!(
                has(&f, &["B3 odgovori", "na kraju sezone"]),
                "{kind:?} {best}: {f:?}"
            );
            assert!(!has(&f, &["B3 klik"]), "{kind:?} {best}: {f:?}");
        }
    }
}

#[test]
fn click_gate_is_not_a_robot_measure() {
    let mut r = clean(PeriodKind::Week);
    for row in &mut paths_mut(&mut r).rows {
        row.click_vs_mute = row.best_vs_mute;
    }
    assert!(!b3(&r).is_empty());
    assert!(!robot_measures_v2(&r).iter().any(|x| x.contains("B3")));
}

#[test]
fn print_has_click_to_own_ratio_lines() {
    let mut r = report(PeriodKind::Week);
    // A: klik = najbolji (100,0 %); B: klik bez težine (0,0 %); C: klik = najbolji (100,0 %).
    for (row, click_equals_best) in paths_mut(&mut r).rows.iter_mut().zip([true, false, true]) {
        row.best_vs_silent[2] = plan_ruler(PeriodKind::Week, row.profile) / 3;
        row.click_vs_silent = if click_equals_best {
            row.best_vs_silent[2]
        } else {
            0
        };
    }
    let text = render_v2(&r);
    assert!(
        !text.contains("samo ispis, nije brana"),
        "klik je od sada brana: {text}"
    );
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| {
            *l == "  klik prema vlastitom, cijeli put (težina klika / težina najboljeg; samo ispis):"
        })
        .unwrap_or_else(|| panic!("nema retka omjera klika: {text}"));
    assert_eq!(
        &lines[at + 1..at + 4],
        ["    A 100,0 %", "    B 0,0 %", "    C 100,0 %"],
        "{text}"
    );
    // Brana klika ima svoj redak: omjer Δ na situacijama s likom (A 50,0 %, B 100,0 %, C 0,0 %).
    for (row, (best, click)) in
        paths_mut(&mut r)
            .rows
            .iter_mut()
            .zip([(200, 100), (200, 200), (200, 0)])
    {
        row.best_vs_mute = best;
        row.click_vs_mute = click;
    }
    let text = render_v2(&r);
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| {
            *l == "  klik prema vlastitom na situacijama s likom (Δ prema šutnji tamo; brana najviše 75,0 %):"
        })
        .unwrap_or_else(|| panic!("nema retka brane klika: {text}"));
    assert_eq!(
        &lines[at + 1..at + 4],
        ["    A 50,0 %", "    B 100,0 %", "    C 0,0 %"],
        "{text}"
    );
    // Redak težine klika (stari ispis) ostaje, bez „nije brana".
    assert!(text.contains("klik najbolji prema rasponu plana"), "{text}");
}

/// Težina klika ima svoju branu, a pojas težine i dalje nosi vlastiti najbolji (`best_vs_silent`): klik u pojasu
/// ne spašava vlastiti izvan pojasa, a klik izvan pojasa (ispod 15 % ili iznad 25 %) sam po sebi nije kršenje pojasa.
#[test]
fn weight_band_stays_on_own_words_best() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        // Klik do tri četvrtine vlastitog (clean: 14,3 % uz 20 %) i svi manji (čak i nula) ne diraju ni pojas ni brane.
        for click in [Some(-1), Some(0), Some(1), None] {
            let mut r = clean(kind);
            for row in &mut paths_mut(&mut r).rows {
                // `None` = kao u `clean`: ravnalo / 6 profila.
                row.click_vs_silent = click.unwrap_or(plan_ruler(kind, row.profile) / 6);
                row.click_vs_mute = row.click_vs_silent;
            }
            assert_eq!(b3(&r), Vec::<String>::new(), "{kind:?} klik {click:?}");
            assert!(
                !robot_measures_v2(&r).iter().any(|x| x.contains("B3")),
                "{kind:?}: klik nije mjera robota"
            );
        }
        // Težinu i dalje nosi `best_vs_silent[2]`: izvan pojasa pada, a klik u pojasu to ne spašava.
        let mut r = clean(kind);
        let row = &mut paths_mut(&mut r).rows[0];
        row.best_vs_silent[2] = 9 * plan_ruler(kind, row.profile);
        row.click_vs_silent = plan_ruler(kind, row.profile) / 4;
        let p = row.profile.code();
        assert!(has(&b3(&r), &["B3 težina", p]), "{kind:?}: {:?}", b3(&r));
        assert!(
            !has(&b3(&r), &["B3 klik"]),
            "{kind:?}: klik je lakši od vlastitog"
        );
    }
}
