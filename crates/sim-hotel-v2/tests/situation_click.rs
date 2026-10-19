//! L1-13 dio 1b (Leon 18.10. t. 1): klik na ponuđeni odgovor (ili prepisan odgovor) gdje se smije pisati nosi samo
//! `economy.click_gain_bp` od svake DOBITI situacije (zadovoljstvo, moral, ocjena, stanje prostora, noćenja grupe);
//! TROŠKOVI, povišice i svi negativni dijelovi ostaju puni. Redoslijed (kartica, koraci 3–5): bodovi × faktor kruga,
//! hitni popravak i bonus kvara, tek onda klik na pozitivne dijelove, pa kvar (−`fault_condition_pts` × faktor kruga i
//! razine od Leonove odluke 18.10. K1, bez klika). Ovdje je faktor kruga 10 000, pa su kvar i bonus −100 / +200. Zaokruživanje je na najbliži cijeli broj (kao `scaled`); uz 10 000 klik je identitet (stari snimak).
//! Klik je `RoundSituation.click`.

mod common;

use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::RoundSituation;
use sim_hotel_v2::*;

fn cat(click_bp: u16, effect_bp: u16) -> Catalog {
    let mut c = common::catalog_with_departments();
    c.economy.click_gain_bp = click_bp;
    // Stari ključ hitnog popravka daje pozitivno stanje prostora koje klik mora umanjiti.
    c.economy.repair_urgent_condition_pts = 1_000;
    for k in PeriodKind::ALL {
        c.periods.get_mut(&k).unwrap().situation_effect_bp = effect_bp;
    }
    c
}

fn periods() -> [Period; 2] {
    [
        month_period(Month::Jun),
        period_by_code(PeriodKind::Week, "t02").unwrap(),
    ]
}

fn category(c: &Catalog) -> Category {
    HotelState::from_profile(c, Profile::C).unwrap().category
}

fn sit(click: bool, fault: bool, mechanics: Vec<Mechanic>, q: u8) -> RoundSituation {
    RoundSituation {
        outcome: SituationOutcome {
            code: "P-X.a".into(),
            mechanics,
            quality: q,
        },
        space: None,
        fault,
        click,
        significance: sim_hotel_v2::situations::Significance::Important,
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

/// Dobitna situacija: +400 zadovoljstva, +600 morala, +1000 stanja prostora (stari ključ), uz troškove i povišicu.
fn gainful(click: bool) -> RoundSituation {
    sit(click, false, vec![recovery(), retention(), urgent()], 5)
}

fn gains(e: &SituationEffect) -> (i32, i32, i32, i32) {
    let f = &e.effects;
    (f.satisfaction_pts, f.morale_pts, f.rating, f.equipment_pts)
}

#[test] // 1
fn click_scales_positive_effects_only() {
    for (bp, want) in [
        (7_500_u16, (300, 450, 0, 750)),
        (5_000, (200, 300, 0, 500)),
        (0, (0, 0, 0, 0)),
    ] {
        let c = cat(bp, 10_000);
        for p in periods() {
            let full = effect(&c, &p, &gainful(false));
            let click = effect(&c, &p, &gainful(true));
            assert_eq!(
                gains(&full),
                (400, 600, 0, 1_000),
                "{bp} {p:?}: bez klika puna dobit"
            );
            assert_eq!(gains(&click), want, "{bp} {p:?}: dobit klika");
            // Trošak (kompenzacija, popravak, bonus) i povišica plaće ostaju puni.
            assert!(full.effects.cost_cents > 0 && full.effects.extra_wage_cents > 0);
            assert_eq!(click.effects.cost_cents, full.effects.cost_cents, "{bp}");
            assert_eq!(
                click.effects.extra_wage_cents, full.effects.extra_wage_cents,
                "{bp}"
            );
            assert_eq!(click.space, full.space, "{bp}: odjel isti");
        }
    }
}

#[test] // 1 (zaokruživanje na najbliži cijeli broj: 133,32 / 199,98 / 333,3)
fn click_rounds_to_nearest() {
    let c = cat(3_333, 10_000);
    let e = effect(&c, &month_period(Month::Jun), &gainful(true));
    assert_eq!(gains(&e), (133, 200, 0, 333));
}

#[test] // 1 (faktor kruga se računa prije klika; hitni popravak je fizički, bez faktora kruga)
fn click_applies_after_the_round_factor() {
    let c = cat(7_500, 5_000);
    for p in periods() {
        let full = effect(&c, &p, &gainful(false));
        let click = effect(&c, &p, &gainful(true));
        assert_eq!(gains(&full), (200, 300, 0, 1_000), "{p:?}");
        assert_eq!(gains(&click), (150, 225, 0, 750), "{p:?}: 400 → 200 → 150");
    }
}

#[test] // 2
fn click_full_gain_is_identity() {
    let c = cat(10_000, 350);
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
                    let (off, on) = (
                        sit(false, fault, mechanics.clone(), q),
                        sit(true, fault, mechanics.clone(), q),
                    );
                    assert_eq!(
                        effect(&c, &p, &on),
                        effect(&c, &p, &off),
                        "{p:?} q {q} {mechanics:?} kvar {fault}: 10 000 = bez klika"
                    );
                }
            }
        }
    }
}

#[test] // 3
fn click_scales_group_room_nights() {
    let p = month_period(Month::Jun);
    let c = cat(7_500, 10_000);
    let full = effect(&c, &p, &sit(false, false, vec![group(10, 4)], 4));
    let click = effect(&c, &p, &sit(true, false, vec![group(10, 4)], 4));
    assert_eq!(full.effects.groups.len(), 1);
    assert_eq!(full.effects.groups[0].room_nights, 40);
    assert_eq!(click.effects.groups.len(), 1);
    assert_eq!(click.effects.groups[0].room_nights, 30, "40 × 75 %");
    // Cijena i mjesec grupe ostaju isti: umanjuje se samo broj noćenja.
    assert_eq!(click.effects.groups[0].month, full.effects.groups[0].month);
    assert_eq!(
        click.effects.groups[0].price_cents,
        full.effects.groups[0].price_cents
    );
    // Najmanje 1 noćenje: prihvaćena grupa se ne gubi potpuno, ni uz 0 %.
    for bp in [1_000_u16, 0] {
        let c = cat(bp, 10_000);
        let e = effect(&c, &p, &sit(true, false, vec![group(1, 1)], 4));
        assert_eq!(e.effects.groups.len(), 1, "{bp}");
        assert_eq!(e.effects.groups[0].room_nights, 1, "{bp}");
    }
    // Odbijena grupa i dalje nema bloka.
    let declined = Mechanic::GroupBooking {
        accept: false,
        rooms: 10,
        nights: 4,
        discount_bp: 0,
        month: Month::Aug,
    };
    let e = effect(&c, &p, &sit(true, false, vec![declined], 4));
    assert!(e.effects.groups.is_empty());
}

fn fault_cat(click_bp: u16, old_urgent: i32) -> Catalog {
    let mut c = cat(click_bp, 10_000);
    c.economy.fault_condition_pts = 100;
    c.economy.fault_fixed_condition_pts = 200;
    c.economy.repair_urgent_condition_pts = old_urgent;
    c
}

#[test] // 4 (bonus popravka kvara je dobit: umanjuje se, kvar −100 ostaje pun)
fn click_scales_fault_fix_bonus() {
    for p in periods() {
        for (bp, old, want) in [
            (7_500_u16, 0, 50),  // −100 + 200 × 0,75
            (5_000, 0, 0),       // −100 + 100
            (0, 0, -100),        // −100 + 0
            (10_000, 0, 100),    // −100 + 200
            (7_500, 1_000, 800), // −100 + (1000 + 200) × 0,75
        ] {
            let c = fault_cat(bp, old);
            let s = sit(true, true, vec![urgent()], 5);
            assert_eq!(
                effect(&c, &p, &s).effects.equipment_pts,
                want,
                "{p:?} klik {bp} stari ključ {old}"
            );
        }
        // L1 (Leon 18.10.): ocjena 3 već dobiva bonus (klik ga umanjuje); slab odgovor (ocjena 2) ne dobiva bonus
        // pa ostaje samo puni kvar, klik ili ne.
        let c = fault_cat(7_500, 0);
        for (click, want) in [(false, 100), (true, 50)] {
            let s = sit(click, true, vec![urgent()], 3);
            assert_eq!(
                effect(&c, &p, &s).effects.equipment_pts,
                want,
                "q 3 klik {click}"
            );
        }
        for click in [false, true] {
            let s = sit(click, true, vec![urgent()], 2);
            assert_eq!(effect(&c, &p, &s).effects.equipment_pts, -100, "{click}");
        }
    }
}

#[test] // 5
fn click_never_scales_losses() {
    for bp in [7_500_u16, 5_000, 0] {
        let c = fault_cat(bp, 0);
        for p in periods() {
            let cases = [
                // šutnja uz kvar: −100 (kvar, fizički)
                sit(true, true, vec![], 1),
                // loš odgovor: negativno zadovoljstvo i moral, privremeni popravak skida stanje, kvar skida stanje
                sit(true, true, vec![recovery(), retention(), temporary()], 1),
                // bez kvara, samo gubici
                sit(true, false, vec![recovery(), retention(), temporary()], 2),
                // neutralna ocjena 3: bez dobiti
                sit(true, false, vec![recovery(), retention()], 3),
            ];
            for s in cases {
                let mut off = s.clone();
                off.click = false;
                assert_eq!(
                    effect(&c, &p, &s),
                    effect(&c, &p, &off),
                    "{bp} {p:?} {:?}: gubici i trošak su isti uz klik",
                    s.outcome
                );
            }
        }
    }
    // Brojke gubitaka (uz faktor kruga 100 %): −400 zadovoljstva, −600 morala, −100 −100 stanja prostora.
    let c = fault_cat(7_500, 0);
    let e = effect(
        &c,
        &month_period(Month::Jun),
        &sit(true, true, vec![recovery(), retention(), temporary()], 1),
    );
    assert_eq!(gains(&e), (-400, -600, 0, -200));
    assert_eq!(e.space, DeptCode::rooms());
}

#[test] // 6
fn click_gain_out_of_range_is_rejected() {
    for ok in [0_u16, 7_500, 10_000] {
        cat(ok, 10_000).validate().unwrap();
    }
    match cat(10_001, 10_000).validate() {
        Err(CatalogError::OutOfRange(k)) => assert_eq!(k, "v2.economy.click_gain_bp"),
        other => panic!("{other:?}"),
    }
}

#[test] // 6 (bez ključa klik je identitet, i u degeneriranom katalogu)
fn degenerate_catalog_has_full_click_gain() {
    let d = degenerate(&common::test_catalog());
    assert_eq!(d.economy.click_gain_bp, 10_000);
}
