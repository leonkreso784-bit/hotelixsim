mod common;

use content::{Snapshot, repo_dir};
use game::*;
use sim_hotel::Month;

/// v2 u degeneriranom katalogu (mjesečno, kao v1) daje iste obračune kao v1 za istu igru (MODEL_V2 §2 t. 6).
#[test]
fn v2_monthly_degenerate_game_matches_v1_game() {
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    let v2 = v1.degenerate_v2();
    let play = |snap: &Snapshot| {
        // Test ne govori o pravilu ocjene: v2 igra igra po starom pravilu (`Classic`), kao v1.
        let mut events = common::new_game_events(snap);
        if let GameEvent::GameCreated(c) = &mut events[0].event
            && c.answer_rule.is_some()
        {
            c.answer_rule = Some(event::AnswerRule::Classic);
        }
        let mut g = Game::fold(&events).unwrap();
        let h = common::hotel_ids()[0];
        let me = Actor::Guest(common::GUEST);
        let ev = g.choose_answer(h, "S1", 0).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        let ev = g.choose_answer(h, "S3", 1).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        for m in [Month::Jun, Month::Jul] {
            let ev = g.close_round(m).unwrap().unwrap();
            common::apply_ev(&mut g, me.clone(), ev);
            let ResolveStep::Done(evs) = g.resolve_job(m, false).unwrap() else {
                panic!()
            };
            for (a, e) in evs {
                common::apply_ev(&mut g, a, e);
            }
        }
        g
    };
    let (a, b) = (play(&v1), play(&v2));
    assert_eq!(b.model(), Model::V2);
    for m in [Month::Jun, Month::Jul] {
        let (ra, rb) = (&a.resolved[&m.into()], &b.resolved[&m.into()]);
        for (ha, hb) in ra.hotels.iter().zip(&rb.hotels) {
            // v2 plan nosi i postavke odjela (Task 11); v1 dio plana mora biti isti.
            assert!(ha.plan.departments.is_empty() && !hb.plan.departments.is_empty());
            let v1_part = dto::PlanDto {
                departments: std::collections::BTreeMap::new(),
                building_renewal_cents: None,
                ..hb.plan.clone()
            };
            assert_eq!(ha.plan, v1_part, "{m:?}");
            // Degenerirani katalog: sve sezonce nosi Smještaj.
            for (code, d) in &hb.plan.departments {
                let expected = if code == "rooms" {
                    ha.plan.seasonal_target
                } else {
                    0
                };
                assert_eq!(d.seasonal_target, expected, "{m:?} {code}");
            }
            assert!(hb.plan.departments.contains_key("rooms"));
            assert_eq!(ha.report, hb.report, "{m:?}");
            assert_eq!(ha.indices, hb.indices, "{m:?}");
            assert_eq!(ha.progress_hundredths, hb.progress_hundredths, "{m:?}");
        }
    }
}
