// Razina cijene pri obračunu (O-P popravak 1): plan poslan prije nego su odjeli bez prihoda po gostu dobili granice
// 100–100 (Uprava 110 % sa starog weba; parking sa starim snimkom, gdje je još stalni najam) obračunava se bez greške —
// razina se pri obračunu stisne u granice kruga, a zapisani plan je onaj stvarno obračunati. Obračunati krugovi se ne
// računaju ponovno.
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use serde_json::json;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

fn fold(snap: &Snapshot) -> Game {
    let events: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind: PeriodKind::Week,
            situations: None,
            answer_rule: None,
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    Game::fold(&events).unwrap()
}

/// Snimak kao prije O-P: parking sa stalnim prihodom 1,50 € po noćenju (bez razine cijene).
fn old_snapshot() -> Snapshot {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["departments"]["parking"]["revenue"] =
        json!({ "type": "fixed", "monthly_cents": 0, "per_guest_night_cents": 150 });
    Snapshot::from_value(&v).unwrap()
}

/// Upiše plan prvog kruga s razinom 110 % za zadane odjele mimo provjere predaje (kao događaj starog dnevnika),
/// zatvori i obračuna krug; vraća igru i događaj obračuna.
fn resolve_with_levels(snap: &Snapshot, codes: &[&str]) -> (Game, RoundKey, RoundResolved) {
    let mut g = fold(snap);
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let mut dto = g.effective_plan(h, r).unwrap().to_dto();
    for k in codes {
        dto.departments.get_mut(*k).unwrap().price_level_pct = 110;
    }
    let ev = GameEvent::PlanSet(PlanSet {
        hotel_id: h,
        month: r,
        plan: dto,
    });
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(r).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("čeka")
    };
    let mut resolved = None;
    for (a, e) in evs {
        if let GameEvent::RoundResolved(x) = &e {
            resolved = Some(x.clone());
        }
        common::apply_ev(&mut g, a, e);
    }
    (g, r, resolved.unwrap())
}

fn level(res: &RoundResolved, code: &str) -> u16 {
    res.hotels[0].plan.departments[code].price_level_pct
}

#[test]
fn stored_admin_price_level_is_clamped_when_resolving() {
    let (_, _, res) = resolve_with_levels(
        &Snapshot::read_v2(&repo_dir()).unwrap(),
        &["admin", "restaurant", "parking"],
    );
    assert_eq!(level(&res, "admin"), 100);
    // odjeli s prihodom po gostu zadržavaju razinu
    assert_eq!(level(&res, "restaurant"), 110);
    assert_eq!(level(&res, "parking"), 110);
}

#[test]
fn stored_parking_price_level_from_old_snapshot_is_clamped_when_resolving() {
    let (_, _, res) = resolve_with_levels(&old_snapshot(), &["parking", "admin"]);
    assert_eq!((level(&res, "parking"), level(&res, "admin")), (100, 100));
}

/// Obračunati krug se ne računa ponovno: dnevnik s obračunom čiji zapisani plan nosi staru razinu 110 % za Upravu
/// (obračunat prije O-P) se čita, a stanje idućeg kruga je zapisano stanje nakon kruga.
#[test]
fn resolved_round_with_old_level_is_read_not_recomputed() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let (g, r, mut res) = resolve_with_levels(&snap, &["admin"]);
    let h = common::hotel_ids()[0];
    let r2 = g.current_round().unwrap();
    let after = g.state_at(h, r2).unwrap();

    res.hotels[0]
        .plan
        .departments
        .get_mut("admin")
        .unwrap()
        .price_level_pct = 110;
    let mut old = fold(&snap);
    let mut dto = old.effective_plan(h, r).unwrap().to_dto();
    dto.departments.get_mut("admin").unwrap().price_level_pct = 110;
    let ev = GameEvent::PlanSet(PlanSet {
        hotel_id: h,
        month: r,
        plan: dto,
    });
    common::apply_ev(&mut old, Actor::Guest(common::GUEST), ev);
    let ev = old.close_round(r).unwrap().unwrap();
    common::apply_ev(&mut old, Actor::Guest(common::GUEST), ev);
    common::apply_ev(&mut old, Actor::System, GameEvent::RoundResolved(res));
    assert_eq!(old.state_at(h, r2).unwrap(), after);
}
