//! L1-13 dio 2 (Leon 18.10. t. 2): tri razine značaja situacije iz `weight` primjerka (4 ključna, 3 važna, 1–2
//! svakodnevna). Bodovi situacije (zadovoljstvo, moral, ocjena, privremeni popravak) dobivaju još faktor razine
//! `economy.significance_bp.{key,important,everyday}`, u JEDNOM računu s faktorom kruga:
//! `bod × situation_effect_bp × significance_bp / 10^8`, jedno zaokruživanje. TROŠKOVI, povišice, noćenja grupe i
//! stari ključ hitnog popravka (fizički) se NE množe. Kvar prostora i bonus njegova popravka se od Leonove odluke 18.10.
//! (četvrta anketa, K1: „slabe kao ostali bodovi situacije") MNOŽE istim faktorom kruga i razine. Klik (dio 1b) djeluje
//! poslije. Bez ključa (sva tri 10 000) obračun je bajt-isti današnjem. Razina je `RoundSituation.significance`.

mod common;

use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{RoundSituation, Significance};
use sim_hotel_v2::*;

const LEVELS: [Significance; 3] = [
    Significance::Key,
    Significance::Important,
    Significance::Everyday,
];

/// Katalog s faktorima razina `(key, important, everyday)`, faktorom kruga i starim ključem hitnog popravka.
fn cat(levels: (u16, u16, u16), effect_bp: u16, old_urgent: i32) -> Catalog {
    let mut c = common::catalog_with_departments();
    c.economy.significance_bp.key = levels.0;
    c.economy.significance_bp.important = levels.1;
    c.economy.significance_bp.everyday = levels.2;
    c.economy.repair_urgent_condition_pts = old_urgent;
    for k in PeriodKind::ALL {
        c.periods.get_mut(&k).unwrap().situation_effect_bp = effect_bp;
    }
    c
}

/// Dio 4 počinje od 15 000 / 10 000 / 6 000 (kartica): ključna 1,5, važna 1,0, svakodnevna 0,6.
const PART4: (u16, u16, u16) = (15_000, 10_000, 6_000);

fn periods() -> [Period; 2] {
    [
        month_period(Month::Jun),
        period_by_code(PeriodKind::Week, "t02").unwrap(),
    ]
}

fn category(c: &Catalog) -> Category {
    HotelState::from_profile(c, Profile::C).unwrap().category
}

fn sit(
    level: Significance,
    fault: bool,
    click: bool,
    mechanics: Vec<Mechanic>,
    q: u8,
) -> RoundSituation {
    RoundSituation {
        outcome: SituationOutcome {
            code: "P-X.a".into(),
            mechanics,
            quality: q,
        },
        space: None,
        fault,
        click,
        significance: level,
    }
}

fn effect(c: &Catalog, p: &Period, s: &RoundSituation) -> SituationEffect {
    situation_effect(s, p, c, category(c)).unwrap()
}

fn recovery() -> Mechanic {
    Mechanic::ServiceRecovery {
        compensation_cents: 12_000,
    }
}

fn retention() -> Mechanic {
    Mechanic::StaffRetention {
        raise_one_pct: 5,
        bonus_cents: 50_000,
    }
}

fn urgent() -> Mechanic {
    Mechanic::EquipmentRepair { urgent: true }
}

fn temporary() -> Mechanic {
    Mechanic::EquipmentRepair { urgent: false }
}

fn group(rooms: u16, nights: u16) -> Mechanic {
    Mechanic::GroupBooking {
        accept: true,
        rooms,
        nights,
        discount_bp: 0,
        month: Month::Aug,
    }
}

fn gains(e: &SituationEffect) -> (i32, i32, i32, i32) {
    let f = &e.effects;
    (f.satisfaction_pts, f.morale_pts, f.rating, f.equipment_pts)
}

/// Najbliži cijeli broj `bod × bp / 10^k` (polovina od nule), isto što i `c::div_round` u jezgri.
fn round_div(pts: i64, num: i64, den: i64) -> i64 {
    let n = pts * num;
    let half = den / 2;
    if n >= 0 {
        (n + half) / den
    } else {
        (n - half) / den
    }
}

#[test] // 1
fn significance_from_weight() {
    assert_eq!(Significance::from_weight(4), Significance::Key);
    assert_eq!(Significance::from_weight(3), Significance::Important);
    assert_eq!(Significance::from_weight(2), Significance::Everyday);
    assert_eq!(Significance::from_weight(1), Significance::Everyday);
}

#[test] // 1
fn significance_codes_are_the_api_words() {
    assert_eq!(Significance::Key.code(), "kljucna");
    assert_eq!(Significance::Important.code(), "vazna");
    assert_eq!(Significance::Everyday.code(), "svakodnevna");
}

/// Dobitna situacija: +400 zadovoljstva, +600 morala (kvaliteta 5) i povišica plaće, trošak kompenzacije i bonusa.
fn gainful(level: Significance) -> RoundSituation {
    sit(level, false, false, vec![recovery(), retention()], 5)
}

#[test] // 2
fn significance_scales_points_not_costs() {
    // Sva tri faktora zadana: bodovi = osnovica × faktor razine (faktor kruga 100 %).
    for (levels, want) in [
        (PART4, [(600, 900), (400, 600), (240, 360)]),
        (
            (20_000, 10_000, 5_000),
            [(800, 1_200), (400, 600), (200, 300)],
        ),
        (
            (30_000, 1_000, 12_500),
            [(1_200, 1_800), (40, 60), (500, 750)],
        ),
    ] {
        let c = cat(levels, 10_000, 0);
        for p in periods() {
            let base = effect(&c, &p, &gainful(Significance::Important));
            for (level, (sat, morale)) in LEVELS.into_iter().zip(want) {
                let e = effect(&c, &p, &gainful(level));
                assert_eq!(
                    (e.effects.satisfaction_pts, e.effects.morale_pts),
                    (sat, morale),
                    "{levels:?} {level:?} {p:?}"
                );
                // Trošak odluke, povišica plaće i prostor odjela ostaju kakvi su (ne množe se razinom).
                assert!(e.effects.cost_cents > 0 && e.effects.extra_wage_cents > 0);
                assert_eq!(e.effects.cost_cents, base.effects.cost_cents, "{level:?}");
                assert_eq!(
                    e.effects.extra_wage_cents, base.effects.extra_wage_cents,
                    "{level:?}"
                );
                assert_eq!(e.space, base.space, "{level:?}");
            }
        }
    }
}

#[test] // 2 (ocjena i privremeni popravak su bodovi i množe se razinom; zaokruženo jednom na cijeli broj)
fn significance_scales_rating_and_temporary_repair() {
    let c = cat(PART4, 10_000, 0);
    let p = month_period(Month::Jun);
    let mechanics = vec![temporary(), recovery()];
    let base = effect(
        &c,
        &p,
        &sit(Significance::Important, false, false, mechanics.clone(), 5),
    );
    // Osnovica mora imati i stanje prostora (privremeni popravak), da razlika ima smisla.
    assert_ne!(
        base.effects.equipment_pts, 0,
        "privremeni popravak mijenja stanje"
    );
    for (level, bp) in [
        (Significance::Key, 15_000_i64),
        (Significance::Everyday, 6_000),
    ] {
        let e = effect(&c, &p, &sit(level, false, false, mechanics.clone(), 5));
        for (got, b, what) in [
            (
                e.effects.equipment_pts,
                base.effects.equipment_pts,
                "oprema",
            ),
            (e.effects.rating, base.effects.rating, "ocjena"),
            (
                e.effects.satisfaction_pts,
                base.effects.satisfaction_pts,
                "zadovoljstvo",
            ),
        ] {
            assert_eq!(
                i64::from(got),
                round_div(i64::from(b), bp, 10_000),
                "{level:?} {what}"
            );
        }
    }
}

#[test] // 2 (negativni bodovi su isto bodovi: loš odgovor na ključnu situaciju boli 1,5 puta)
fn significance_scales_negative_points_too() {
    let c = cat(PART4, 10_000, 0);
    let p = month_period(Month::Jun);
    let mech = vec![recovery(), retention()];
    let base = effect(
        &c,
        &p,
        &sit(Significance::Important, false, false, mech.clone(), 1),
    );
    assert!(base.effects.satisfaction_pts < 0 && base.effects.morale_pts < 0);
    let key = effect(
        &c,
        &p,
        &sit(Significance::Key, false, false, mech.clone(), 1),
    );
    let eve = effect(&c, &p, &sit(Significance::Everyday, false, false, mech, 1));
    for (e, bp) in [(&key, 15_000_i64), (&eve, 6_000)] {
        assert_eq!(
            i64::from(e.effects.satisfaction_pts),
            round_div(i64::from(base.effects.satisfaction_pts), bp, 10_000)
        );
        assert_eq!(
            i64::from(e.effects.morale_pts),
            round_div(i64::from(base.effects.morale_pts), bp, 10_000)
        );
    }
}

#[test] // 2 (hitni popravak je fizički, kao i bez razine: bez faktora kruga i bez faktora razine)
fn significance_does_not_scale_urgent_repair_groups_or_costs() {
    let c = cat(PART4, 5_000, 1_000);
    for p in periods() {
        let base = effect(
            &c,
            &p,
            &sit(Significance::Important, false, false, vec![urgent()], 5),
        );
        assert_eq!(
            base.effects.equipment_pts, 1_000,
            "{p:?}: stari ključ, bez faktora"
        );
        let g = effect(
            &c,
            &p,
            &sit(Significance::Important, false, false, vec![group(10, 4)], 4),
        );
        assert_eq!(g.effects.groups[0].room_nights, 40);
        for level in LEVELS {
            let e = effect(&c, &p, &sit(level, false, false, vec![urgent()], 5));
            assert_eq!(
                e.effects.equipment_pts, 1_000,
                "{p:?} {level:?}: hitni popravak"
            );
            let e = effect(&c, &p, &sit(level, false, false, vec![group(10, 4)], 4));
            assert_eq!(e.effects.groups.len(), 1);
            assert_eq!(
                e.effects.groups[0].room_nights, 40,
                "{p:?} {level:?}: noćenja grupe"
            );
            assert_eq!(e.effects.cost_cents, g.effects.cost_cents, "{level:?}");
        }
    }
}

#[test] // 2 (K1, Leon 18.10. četvrta anketa: kvar i bonus popravka slabe kao ostali bodovi — i faktorom razine)
fn significance_scales_fault_and_fix_bonus() {
    let mut c = cat(PART4, 5_000, 0);
    c.economy.fault_condition_pts = 100;
    c.economy.fault_fixed_condition_pts = 200;
    // Faktor kruga 0,5 × razina 1,5 / 1,0 / 0,6: kvar −75 / −50 / −30, bonus +150 / +100 / +60.
    let want = [
        (Significance::Key, -75, 150),
        (Significance::Important, -50, 100),
        (Significance::Everyday, -30, 60),
    ];
    for p in periods() {
        for (level, lose, gain) in want {
            let silent = sit(level, true, false, vec![], 1);
            assert_eq!(
                effect(&c, &p, &silent).effects.equipment_pts,
                lose,
                "{p:?} {level:?}: kvar bez odgovora"
            );
            for q in [5, 3] {
                let fixed = sit(level, true, false, vec![urgent()], q);
                assert_eq!(
                    effect(&c, &p, &fixed).effects.equipment_pts,
                    gain + lose,
                    "{p:?} {level:?} q {q}: bonus + kvar"
                );
            }
        }
    }
    // Jedno zaokruživanje (kao bodovi): 3 × 5 000 × 15 000 / 10^8 = 2,25 → 2; u dva koraka bilo bi 3.
    let mut c = cat(PART4, 5_000, 0);
    c.economy.fault_condition_pts = 3;
    for p in periods() {
        let silent = sit(Significance::Key, true, false, vec![], 1);
        assert_eq!(
            effect(&c, &p, &silent).effects.equipment_pts,
            -2,
            "{p:?}: jedno zaokruživanje"
        );
    }
}

#[test] // 2 (jedan račun: faktor kruga i razine množe se prije jednog zaokruživanja, ne u dva koraka)
fn significance_and_round_factor_round_once() {
    // 400 × 3 333 × 12 500 / 10^8 = 166,65 → 167; u dva koraka: 400 × 0,3333 = 133 pa × 1,25 = 166,25 → 166.
    let c = cat((12_500, 10_000, 10_000), 3_333, 0);
    for p in periods() {
        let e = effect(&c, &p, &gainful(Significance::Key));
        assert_eq!(
            e.effects.satisfaction_pts, 167,
            "{p:?}: jedno zaokruživanje"
        );
        // 600 × 3 333 × 12 500 / 10^8 = 249,975 → 250.
        assert_eq!(e.effects.morale_pts, 250, "{p:?}");
        // Važna razina uz faktor 1,0 je točno današnji račun: 400 × 0,3333 = 133,32 → 133.
        let imp = effect(&c, &p, &gainful(Significance::Important));
        assert_eq!(imp.effects.satisfaction_pts, 133, "{p:?}");
    }
}

#[test] // 2 (razina se množi prije klika; klik umanjuje već uvećanu dobit, a ne obrnuto)
fn click_applies_after_significance() {
    let mut c = cat(PART4, 10_000, 0);
    c.economy.click_gain_bp = 7_500;
    let p = month_period(Month::Jun);
    let click = |level| sit(level, false, true, vec![recovery(), retention()], 5);
    // Ključna: 400 × 1,5 = 600, pa klik × 0,75 = 450; svakodnevna: 400 × 0,6 = 240, pa 180.
    assert_eq!(
        effect(&c, &p, &click(Significance::Key))
            .effects
            .satisfaction_pts,
        450
    );
    assert_eq!(
        effect(&c, &p, &click(Significance::Key)).effects.morale_pts,
        675
    );
    assert_eq!(
        effect(&c, &p, &click(Significance::Everyday))
            .effects
            .satisfaction_pts,
        180
    );
    assert_eq!(
        effect(&c, &p, &click(Significance::Important))
            .effects
            .satisfaction_pts,
        300
    );
    // Noćenja grupe klik umanjuje, a razina ne.
    let g = effect(
        &c,
        &p,
        &sit(Significance::Key, false, true, vec![group(10, 4)], 4),
    );
    assert_eq!(g.effects.groups[0].room_nights, 30, "40 × 75 %");
}

#[test] // 3
fn neutral_significance_is_identity() {
    let c = cat((10_000, 10_000, 10_000), 350, 0);
    let mechanic_sets = [
        vec![],
        vec![recovery()],
        vec![recovery(), retention(), urgent()],
        vec![temporary()],
        vec![urgent(), temporary()],
        vec![group(10, 4)],
    ];
    for p in periods() {
        for q in 1..=5 {
            for mechanics in &mechanic_sets {
                for fault in [false, true] {
                    for click in [false, true] {
                        let base = effect(
                            &c,
                            &p,
                            &sit(Significance::Important, fault, click, mechanics.clone(), q),
                        );
                        for level in LEVELS {
                            assert_eq!(
                                effect(&c, &p, &sit(level, fault, click, mechanics.clone(), q)),
                                base,
                                "{p:?} q {q} {mechanics:?} kvar {fault} klik {click} {level:?}: 10 000 = bez razine"
                            );
                        }
                    }
                }
            }
        }
    }
    // Brojke današnjeg računa (400 / 600 uz faktor kruga 100 %), da identitet nije samo „svi isti".
    let c = cat((10_000, 10_000, 10_000), 10_000, 1_000);
    let e = effect(
        &c,
        &month_period(Month::Jun),
        &sit(
            Significance::Key,
            false,
            false,
            vec![recovery(), retention(), urgent()],
            5,
        ),
    );
    assert_eq!(gains(&e), (400, 600, 0, 1_000));
}

#[test] // 3 (važna razina uz faktor 10 000 je današnji račun i kad su ostale dvije razine izmijenjene)
fn important_level_is_the_unchanged_baseline() {
    let neutral = cat((10_000, 10_000, 10_000), 350, 0);
    let tuned = cat(PART4, 350, 0);
    for p in periods() {
        for q in 1..=5 {
            let s = sit(
                Significance::Important,
                false,
                false,
                vec![recovery(), retention(), temporary()],
                q,
            );
            assert_eq!(
                effect(&tuned, &p, &s),
                effect(&neutral, &p, &s),
                "{p:?} q {q}"
            );
        }
    }
}

#[test] // 4 (u potpunom krugu: razina mijenja ishod obračuna, ne samo učinak jedne situacije)
fn significance_changes_the_period_resolution() {
    let p = period_by_code(PeriodKind::Week, "t02").unwrap();
    let run = |levels: (u16, u16, u16), level| {
        let c = cat(levels, 10_000, 0);
        let st = HotelState::from_profile(&c, Profile::C).unwrap();
        let plan = common::hold_plan(&st, &p, &c);
        resolve_period(
            &st,
            &p,
            &plan,
            &[sit(level, false, false, vec![recovery(), retention()], 5)],
            &c,
        )
        .unwrap()
    };
    let key = run(PART4, Significance::Key);
    let important = run(PART4, Significance::Important);
    let everyday = run(PART4, Significance::Everyday);
    let neutral = run((10_000, 10_000, 10_000), Significance::Key);
    assert_eq!(important.after, neutral.after, "važna = današnji račun");
    assert_ne!(key.after, important.after, "ključna daje drugačije stanje");
    assert_ne!(
        everyday.after, important.after,
        "svakodnevna daje drugačije stanje"
    );
    assert!(key.after.satisfaction_pts > important.after.satisfaction_pts);
    assert!(everyday.after.satisfaction_pts < important.after.satisfaction_pts);
}

#[test] // 5
fn significance_out_of_range_is_rejected() {
    for ok in [(1_000, 1_000, 1_000), (30_000, 30_000, 30_000), PART4] {
        cat(ok, 10_000, 0).validate().unwrap();
    }
    for bad in [999_u16, 0, 30_001, 65_535] {
        for (field, levels) in [
            ("key", (bad, 10_000, 10_000)),
            ("important", (10_000, bad, 10_000)),
            ("everyday", (10_000, 10_000, bad)),
        ] {
            match cat(levels, 10_000, 0).validate() {
                Err(CatalogError::OutOfRange(k)) => assert!(
                    k.contains("significance_bp") && k.contains(field),
                    "{field} = {bad}: ključ u poruci: {k}"
                ),
                other => panic!("{field} = {bad}: {other:?}"),
            }
        }
    }
}

#[test] // 5 (bez ključa svaka razina je identitet, i u degeneriranom katalogu)
fn degenerate_catalog_has_neutral_significance() {
    let d = degenerate(&common::test_catalog());
    assert_eq!(
        (
            d.economy.significance_bp.key,
            d.economy.significance_bp.important,
            d.economy.significance_bp.everyday
        ),
        (10_000, 10_000, 10_000)
    );
}

#[test] // 5 (razina se čuva uz odjel: oznaka razine ne mijenja kamo ide učinak)
fn significance_keeps_the_department_of_the_effect() {
    let c = cat(PART4, 10_000, 1_000);
    let p = month_period(Month::Jun);
    for level in LEVELS {
        let mut s = sit(level, false, false, vec![urgent()], 5);
        s.space = Some(DeptCode("restaurant".into()));
        assert_eq!(
            effect(&c, &p, &s).space,
            DeptCode("restaurant".into()),
            "{level:?}"
        );
    }
}
