//! „Samo nagrada" (S-4, Leon 12.10.): pohvala (P-POHVALA) je čista prilika — ni ponuđeni, ni slobodan, ni
//! neodgovoren ishod ne snižava moral ni drugi pokazatelj. Pravi put: primjerak predloška iz repoa kao situacija
//! mjesečne v2 igre, slobodan odgovor s presudom, `resolve_job`, obračun jezgre v2.

mod common;

use std::collections::BTreeMap;

use common::{GUEST, apply_ev, draft_with, hotel_ids, meta};
use content::templates::Template;
use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use serde_json::{Value, json};
use sim_hotel::{Mechanic, Month, SituationTemplate};
use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::situations::{DeptTarget, DrawnSituation, NumberParam, Scope};
use sim_hotel_v2::{PeriodKind, Profile};
use uuid::Uuid;

const CODE: &str = "P-POHVALA.jun";

/// Primjerak P-POHVALA za lipanj (hotel kategorije profila C), složen kao u igri (`SituationTemplates::instance`).
fn praise_instance(snap: &Snapshot) -> (SituationTemplate, Template) {
    let c = snap.build().unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    let cat = c.v2.as_ref().unwrap();
    let set = tpl.to_core();
    let spec = set
        .templates
        .iter()
        .find(|t| t.code == "P-POHVALA")
        .unwrap();
    let department = match spec.department.members(cat) {
        Some(m) => m.first().cloned(),
        None => None,
    };
    assert!(!matches!(spec.department, DeptTarget::Hotel));
    let numbers = spec
        .numbers
        .iter()
        .map(|(k, n)| {
            let v = match n {
                NumberParam::Range(a, _) => *a,
                NumberParam::Pick(v) => v[0],
            };
            (k.clone(), v)
        })
        .collect();
    let d = DrawnSituation {
        code: CODE.to_string(),
        template: "P-POHVALA".to_string(),
        period: month_period(Month::Jun),
        day: 0,
        occurrence: 1,
        slot: 1,
        kind: spec.kind,
        weight: spec.weight,
        scope: Scope::Hotel,
        major: None,
        department,
        roles: BTreeMap::new(),
        numbers,
    };
    let category = cat.base.profile(Profile::C).unwrap().category;
    let source = tpl
        .templates
        .iter()
        .find(|t| t.code == "P-POHVALA")
        .unwrap()
        .clone();
    (tpl.instance(&d, &cat.base, category).unwrap(), source)
}

fn mechanic_json(m: &Mechanic) -> Value {
    match m {
        Mechanic::ServiceRecovery { compensation_cents } => {
            json!({"type": "ServiceRecovery", "compensation_cents": compensation_cents})
        }
        Mechanic::EquipmentRepair { urgent } => {
            json!({"type": "EquipmentRepair", "urgent": urgent})
        }
        Mechanic::StaffRetention {
            raise_one_pct,
            bonus_cents,
        } => {
            json!({"type": "StaffRetention", "raise_one_pct": raise_one_pct, "bonus_cents": bonus_cents})
        }
        Mechanic::GroupBooking {
            accept,
            rooms,
            nights,
            discount_bp,
            month,
        } => json!({"type": "GroupBooking", "accept": accept, "rooms": rooms, "nights": nights,
                    "discount_bp": discount_bp, "month": month.code()}),
        Mechanic::CrisisCommunication { backup_cents } => {
            json!({"type": "CrisisCommunication", "backup_cents": backup_cents})
        }
    }
}

/// Snimak v2 kojem je lipanjska situacija primjerak P-POHVALA (slobodan odgovor dopušten).
fn snapshot_with_praise() -> Snapshot {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let (t, source) = praise_instance(&snap);
    let mut v = snap.to_value();
    let answers: Vec<Value> = t
        .answers
        .iter()
        .zip(&source.answers)
        .map(|(a, src)| {
            json!({"label": a.label, "quality": a.quality, "criteria": src.criteria,
                   "mechanics": a.mechanics.iter().map(mechanic_json).collect::<Vec<_>>()})
        })
        .collect();
    v["situations"] = json!([{
        "code": CODE,
        "month": "jun",
        "title": t.title,
        "unanswered": t.unanswered.iter().map(mechanic_json).collect::<Vec<_>>(),
        "answers": answers,
        "rubric": source.rubric,
        "character": {
            "name": "Ana Horvat",
            "role": "konobarica",
            "brief": "Ti si Ana, konobarica koju je gost pohvalio.",
            "opening": "Čula sam za pismo.",
            "fallback_reply": "Ana se vratila na posao.",
        },
        "free_text": true,
    }]);
    Snapshot::from_value(&v).unwrap()
}

enum Choice {
    None,
    Free { met: usize, nearest: u8 },
}

/// Mjesečna v2 igra: lipanj s pohvalom zatvoren i obračunat; obračun hotela igrača.
fn june(choice: Choice) -> HotelResolution {
    let snap = snapshot_with_praise();
    let log: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        GUEST,
        hotel_ids(),
        GameSettings {
            kind: PeriodKind::Month,
            situations: None,
            answer_rule: None,
        },
    )
    .unwrap()
    .into_iter()
    // Staro pravilo (G-1): jedina situacija je ona iz snimka; izvlačenje iz predložaka i odgovori robota na
    // primjerke izbačeni, da se ispituje samo put obračuna ponuđenog situacijskog zapisa.
    .filter(|(_, e)| e.kind() != "SituationsDrawn")
    .filter(|(_, e)| !matches!(e, GameEvent::AnswerChosen(a) if a.situation != CODE))
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    let mut g = Game::fold(&log).unwrap();
    let h = hotel_ids()[0];
    let r = g.current_round().unwrap();
    assert_eq!(g.situations_in(r).len(), 1);
    if let Choice::Free { met, nearest } = choice {
        let ev = g
            .submit_response(h, CODE, "Zahvalit ću joj osobno.")
            .unwrap();
        apply_ev(&mut g, Actor::Guest(GUEST), ev);
        let v = g.validate_verdict(CODE, draft_with(met, nearest)).unwrap();
        let ev = g.record_verdict(h, CODE, v, meta()).unwrap();
        apply_ev(&mut g, Actor::System, ev);
    }
    let ev = g.close_round(r).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        apply_ev(&mut g, a, e);
    }
    g.resolved[&r]
        .hotels
        .iter()
        .find(|x| x.hotel_id == h)
        .unwrap()
        .clone()
}

/// Slobodan odgovor kvalitete 1 uz najbliži odgovor „zahvala bez nagrade" (zadržavanje ljudi): bez oznake bi dao
/// 300 × (1 − 3) = −6 morala. Sa „samo nagrada" obračun je isti kao kad pohvalu nitko ne odgovori (bez štete).
#[test]
fn free_answer_of_quality_one_to_praise_does_not_lower_morale() {
    let silent = june(Choice::None);
    let poor = june(Choice::Free { met: 1, nearest: 1 });
    let s = poor.situations.iter().find(|s| s.code == CODE).unwrap();
    assert!(s.free_text);
    assert_eq!(s.quality, 1, "kvaliteta presude ostaje stvarna");
    assert!(
        poor.report.after.morale_pts >= silent.report.after.morale_pts,
        "moral {} < {}",
        poor.report.after.morale_pts,
        silent.report.after.morale_pts
    );
    assert_eq!(poor.report, silent.report);
    assert_eq!(poor.indices, silent.indices);
}

/// Oznaka ne gasi nagradu: slobodan odgovor kvalitete 5 uz najbolji odgovor diže moral.
#[test]
fn free_answer_of_quality_five_to_praise_raises_morale() {
    let silent = june(Choice::None);
    let best = june(Choice::Free { met: 5, nearest: 0 });
    assert!(
        best.report.after.morale_pts > silent.report.after.morale_pts,
        "moral {} <= {}",
        best.report.after.morale_pts,
        silent.report.after.morale_pts
    );
}

// ---------------------------------------------------------------------------------------------------------------
// S-4 Minor 1/2: zaštita „samo nagrada" na SVA TRI puta obračuna u igri (ponuđen, slobodan, neodgovoren).
// Sintetska situacija sa šifrom `P-POHVALA.jun` (oznaka ide po predlošku) kojoj SVAKI ishod, i neodgovoren, nosi
// mehaniku zadržavanja ljudi: bez zaštite q1 daje 300 × (1 − 3) = −600 bodova morala. Kontrola s drugom šifrom
// pokazuje da bi bez oznake isti odgovor snizio moral (test nije prazan).

#[derive(Clone, Copy)]
enum Pick {
    Silent,
    Offered(u8),
    Free { met: usize, nearest: u8 },
}

fn retention() -> Value {
    json!({"type": "StaffRetention", "raise_one_pct": 0, "bonus_cents": 0})
}

/// Snimak v2 s jednom sintetskom lipanjskom situacijom: odgovori 0/1/2 imaju kvalitetu 5/3/1 i istu mehaniku.
fn synthetic_snapshot(code: &str) -> Snapshot {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut v = snap.to_value();
    let answer = |label: &str, criteria: Vec<u8>| {
        json!({"label": label, "quality": criteria.len(), "criteria": criteria,
               "mechanics": [retention()]})
    };
    v["situations"] = json!([{
        "code": code,
        "month": "jun",
        "title": "Sintetska pohvala",
        "unanswered": [retention()],
        "answers": [
            answer("Odlično", vec![1, 2, 3, 4, 5]),
            answer("Prosječno", vec![1, 2, 3]),
            answer("Loše", vec![5]),
        ],
        "rubric": ["Jedan.", "Dva.", "Tri.", "Četiri.", "Pet."],
        "character": {
            "name": "Ana Horvat",
            "role": "konobarica",
            "brief": "Ti si Ana.",
            "opening": "Dobar dan.",
            "fallback_reply": "Ana se vratila na posao.",
        },
        "free_text": true,
    }]);
    Snapshot::from_value(&v).unwrap()
}

fn june_synthetic(code: &str, pick: Pick) -> HotelResolution {
    let snap = synthetic_snapshot(code);
    let log: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        GUEST,
        hotel_ids(),
        GameSettings {
            kind: PeriodKind::Month,
            situations: None,
            answer_rule: None,
        },
    )
    .unwrap()
    .into_iter()
    // Jedina situacija lipnja je sintetska: izvlačenje i odgovori robota na primjerke se izbacuju (staro pravilo).
    .filter(|(_, e)| e.kind() != "SituationsDrawn")
    .filter(|(_, e)| !matches!(e, GameEvent::AnswerChosen(a) if a.situation != code))
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    let mut g = Game::fold(&log).unwrap();
    let h = hotel_ids()[0];
    let r = g.current_round().unwrap();
    match pick {
        Pick::Silent => {}
        Pick::Offered(i) => {
            let ev = g.choose_answer(h, code, i).unwrap();
            apply_ev(&mut g, Actor::Guest(GUEST), ev);
        }
        Pick::Free { met, nearest } => {
            let ev = g.submit_response(h, code, "Zahvalit ću joj.").unwrap();
            apply_ev(&mut g, Actor::Guest(GUEST), ev);
            let v = g.validate_verdict(code, draft_with(met, nearest)).unwrap();
            let ev = g.record_verdict(h, code, v, meta()).unwrap();
            apply_ev(&mut g, Actor::System, ev);
        }
    }
    let ev = g.close_round(r).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        apply_ev(&mut g, a, e);
    }
    g.resolved[&r]
        .hotels
        .iter()
        .find(|x| x.hotel_id == h)
        .unwrap()
        .clone()
}

/// Pretpostavka na kojoj počiva `reward_only`: bodovi mehanika su `(q − 3) × k` s `k ≥ 0` (v1 i v2 katalog).
#[test]
fn per_quality_points_of_mechanics_are_not_negative() {
    let c = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let v2 = c.v2.as_ref().unwrap();
    for m in [&c.catalog.mechanics, &v2.base.mechanics] {
        assert!(m.recovery_pts_per_q >= 0, "oporavak");
        assert!(m.retention_pts_per_q >= 0, "zadržavanje");
        assert!(m.crisis_pts_per_q >= 0, "kriza");
    }
}

/// Bez oznake (druga šifra) isti odgovor kvalitete 1 snižava moral: kontrola da test hvata kvar.
#[test]
fn control_without_the_mark_a_poor_answer_lowers_morale() {
    let code = "P-KLIMA.jun";
    let neutral = june_synthetic(code, Pick::Offered(1));
    let poor = june_synthetic(code, Pick::Offered(2));
    let silent = june_synthetic(code, Pick::Silent);
    assert!(poor.report.after.morale_pts < neutral.report.after.morale_pts);
    assert!(silent.report.after.morale_pts < neutral.report.after.morale_pts);
}

/// Sva tri puta (ponuđen, slobodan, neodgovoren) uz oznaku daju isti obračun kao neutralna kvaliteta 3.
#[test]
fn reward_only_protects_offered_free_and_unanswered_paths() {
    let neutral = june_synthetic(CODE, Pick::Offered(1));
    let best = june_synthetic(CODE, Pick::Offered(0));
    assert!(
        best.report.after.morale_pts > neutral.report.after.morale_pts,
        "oznaka ne smije gasiti nagradu"
    );
    for (name, pick) in [
        ("ponuđen q1", Pick::Offered(2)),
        ("neodgovoren", Pick::Silent),
        ("slobodan q1", Pick::Free { met: 1, nearest: 2 }),
        ("slobodan q2", Pick::Free { met: 2, nearest: 2 }),
    ] {
        let r = june_synthetic(CODE, pick);
        assert_eq!(
            r.report, neutral.report,
            "{name}: obračun mora biti kao neutralan"
        );
        assert_eq!(r.indices, neutral.indices, "{name}");
    }
}
