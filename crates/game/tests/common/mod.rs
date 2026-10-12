//! Zajednički pomoćnici testova (dijele ih `round.rs` i `resolve.rs`).
#![allow(dead_code)]

use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::event::AiSource;
use game::*;
use uuid::Uuid;

pub fn hotel_ids() -> [HotelId; 4] {
    [11, 12, 13, 14].map(|n| HotelId(Uuid::from_u128(n)))
}
pub const GUEST: GuestId = GuestId(Uuid::from_u128(99));

/// Nova igra iz zadanog snimka kao zapisani događaji (redni brojevi od 1).
pub fn new_game_events(snap: &Snapshot) -> Vec<Recorded> {
    new_game(GameId(Uuid::from_u128(1)), 42, snap, GUEST, hotel_ids())
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

/// Nova igra kao zapisani događaji (redni brojevi od 1).
pub fn created(seed: u64) -> Vec<Recorded> {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    new_game(GameId(Uuid::from_u128(1)), seed, &snap, GUEST, hotel_ids())
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

/// Igra sa zatvorenim lipnjem (`RoundClosed`), obračun još čeka.
pub fn closed_june_game() -> Game {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let ev = g
        .close_round(sim_hotel::Month::Jun)
        .unwrap()
        .expect("lipanj se zatvara");
    push(&mut log, Actor::Guest(GUEST), ev);
    Game::fold(&log).unwrap()
}

pub fn push(log: &mut Vec<Recorded>, actor: Actor, event: GameEvent) {
    let seq = log.last().map_or(1, |r| r.seq + 1);
    log.push(Recorded { seq, actor, event });
}

pub fn guest() -> GuestId {
    GUEST
}

/// Igra s otvorenim lipnjem i hotelom igrača (prvi hotel).
pub fn open_june_game() -> (Game, HotelId) {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let g = Game::fold(&new_game_events(&snap)).unwrap();
    (g, hotel_ids()[0])
}

/// Primijeni događaj na igru sa sljedećim rednim brojem.
pub fn apply_ev(g: &mut Game, actor: Actor, event: GameEvent) {
    let seq = g.last_seq + 1;
    g.apply(Recorded { seq, actor, event }).unwrap();
}

pub fn my_hotel(_g: &Game) -> HotelId {
    hotel_ids()[0]
}

pub fn meta() -> AiMeta {
    AiMeta {
        model: "test-model".into(),
        prompt: "test-prompt".into(),
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Nacrt presude s točno `met` ispunjenih kriterija (prvih `met`) i najbližim odgovorom `nearest`.
pub fn draft_with(met: usize, nearest: u8) -> VerdictDraft {
    VerdictDraft {
        criteria: (1u8..=5)
            .map(|n| CriterionVerdict {
                n,
                met: usize::from(n) <= met,
                reason: format!("razlog {n}"),
            })
            .collect(),
        nearest_answer: nearest,
        rationale: "obrazloženje".into(),
    }
}

pub fn draft_all_met(nearest: u8) -> VerdictDraft {
    draft_with(5, nearest)
}

const FREE_TEXT: &str = "Ispričat ću se i ponuditi zamjenu sobe.";

/// Lipanj zatvoren, S1 je slobodan odgovor bez presude.
pub fn june_closed_with_pending_response() -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.submit_response(h, "S1", FREE_TEXT).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    g
}

fn run_resolve(g: &mut Game) {
    let ResolveStep::Done(evs) = g.resolve_job(sim_hotel::Month::Jun, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        apply_ev(g, a, e);
    }
}

/// Lipanj obračunat sa slobodnim odgovorom na S1 i presudom (`q` kriterija, najbliži odgovor `i`).
pub fn june_resolved_with_free_verdict_game(q: usize, i: u8) -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.submit_response(h, "S1", FREE_TEXT).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let v = g.validate_verdict("S1", draft_with(q, i)).unwrap();
    let ev = g.record_verdict(h, "S1", v, meta()).unwrap();
    apply_ev(&mut g, Actor::System, ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    run_resolve(&mut g);
    // Mentor za lipanj (rezervni tekst, kao kad AI ne radi).
    let ev = g
        .record_debrief(
            h,
            sim_hotel::Month::Jun,
            "Mentor: dobar početak.".to_owned(),
            AiSource::Fallback,
            None,
        )
        .unwrap()
        .unwrap();
    apply_ev(&mut g, Actor::System, ev);
    g
}

pub fn june_resolved_with_free_verdict(q: usize, i: u8) -> RoundResolved {
    june_resolved_with_free_verdict_game(q, i).resolved[&RoundKey::Month(sim_hotel::Month::Jun)]
        .clone()
}

fn june_resolved_offered_game(i: u8) -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.choose_answer(h, "S1", i).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    run_resolve(&mut g);
    g
}

pub fn june_resolved_with_offered(i: u8) -> RoundResolved {
    june_resolved_offered_game(i).resolved[&RoundKey::Month(sim_hotel::Month::Jun)].clone()
}

pub fn june_resolved_game() -> Game {
    june_resolved_offered_game(0)
}

/// Obračun hotela igrača (prvi hotel).
pub fn my_resolution(rr: &RoundResolved) -> &HotelResolution {
    &rr.hotels[0]
}

fn resolve_month_now(g: &mut Game, m: sim_hotel::Month) {
    let ev = g.close_round(m).unwrap().unwrap();
    apply_ev(g, Actor::Guest(GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(m, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        apply_ev(g, a, e);
    }
}

/// Stvaran loš srpanj (Leonov primjer): lipanj dobro, u srpnju niže plaće, odgođeno održavanje i
/// šefu kuhinje (S2) samo bonus bez razgovora (ocjena 2).
pub fn july_resolved_bad_chef_game() -> (Game, HotelId) {
    use sim_hotel::{Maintenance, Month};
    let (mut g, h) = open_june_game();
    let ev = g.choose_answer(h, "S1", 0).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.choose_answer(h, "S3", 1).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    resolve_month_now(&mut g, Month::Jun);
    let CorePlan::V1(mut plan) = g.effective_plan(h, Month::Jul).unwrap() else {
        panic!("v1 plan")
    };
    plan.wage_pct = 95;
    plan.maintenance = Maintenance::Deferred;
    let ev = g.set_plan(h, Month::Jul, plan).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.choose_answer(h, "S2", 1).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    resolve_month_now(&mut g, Month::Jul);
    (g, h)
}

/// V2 tjedna igra s obračunatim krugovima redom do uključivo `code` (npr. "t05"); `"none"` = nijedan obračun.
pub fn v2_weekly_game_resolved_through(code: &str) -> Game {
    v2_game_resolved_through(sim_hotel_v2::PeriodKind::Week, code)
}

/// V2 mjesečna igra s obračunatim krugovima redom do uključivo `code` (npr. "sep"); `"none"` = nijedan obračun.
pub fn v2_monthly_game_resolved_through(code: &str) -> Game {
    v2_game_resolved_through(sim_hotel_v2::PeriodKind::Month, code)
}

fn v2_game_resolved_through(kind: sim_hotel_v2::PeriodKind, code: &str) -> Game {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    play_through(v2_game_from(&snap, 42, kind), code)
}

/// Nova v2 igra iz zadanog snimka i sjemena (bez obračuna).
pub fn v2_game_from(snap: &Snapshot, seed: u64, kind: sim_hotel_v2::PeriodKind) -> Game {
    use game::command::{GameSettings, new_game_with};
    let log: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        seed,
        snap,
        GUEST,
        hotel_ids(),
        GameSettings { kind },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    Game::fold(&log).unwrap()
}

/// Snimak v2 sadržaja s obnovom (brojke O1, O2, O4, O5, O9 iz obnova-v2-spec §0): krivulja za svaki odjel s gostima
/// i parking, zgrada, pričuva 4 %, indeks F iz EBIT-a i početno stanje zgrade po profilu. Lokal i neraspoređeni su
/// bez obnove.
pub fn renewal_snapshot() -> Snapshot {
    use serde_json::json;
    let renewal = |half: i64, max: i64, step: i64, life: u16| {
        json!({ "gain_pts": 5000, "half_cents": half * 100, "max_monthly_cents": max * 100,
                "step_cents": step * 100, "life_months": life })
    };
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    for (code, spec) in [
        ("rooms", renewal(150_000, 150_000, 5_000, 84)),
        ("restaurant", renewal(55_000, 50_000, 5_000, 84)),
        ("lobby_bar", renewal(25_000, 50_000, 5_000, 84)),
        ("rooftop_bar", renewal(25_000, 50_000, 5_000, 84)),
        ("pool_bar", renewal(20_000, 50_000, 5_000, 84)),
        ("spa", renewal(40_000, 50_000, 5_000, 120)),
        ("parking", renewal(60_000, 50_000, 5_000, 180)),
    ] {
        v["v2"]["departments"][code]["renewal"] = spec;
    }
    v["v2"]["economy"]["building"] = json!({
        "name": "Zgrada",
        "wear_pts_per_month": 50,
        "satisfaction_share_bp": 2000,
        "renewal": renewal(150_000, 200_000, 10_000, 480),
    });
    v["v2"]["economy"]["ffe_reserve_bp"] = json!(400);
    v["v2"]["economy"]["finance_index"] = json!("ebit");
    for (p, pts) in [("A", 8000), ("B", 4500), ("C", 6500)] {
        v["v2"]["profiles"][p]["building_pts"] = json!(pts);
    }
    Snapshot::from_value(&v).unwrap()
}

/// Snimak v2 sadržaja bez obnove v2 (sadržaj je ima od O-2): odjeli bez krivulje obnove, zgrada bez učinka, bez
/// pričuve, indeksa F iz EBIT-a i stanja zgrade profila — za staru kvačicu obnove i igru bez izbora obnove.
pub fn no_renewal_snapshot() -> Snapshot {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    for d in v["v2"]["departments"].as_object_mut().unwrap().values_mut() {
        d.as_object_mut().unwrap().remove("renewal");
    }
    for p in v["v2"]["profiles"].as_object_mut().unwrap().values_mut() {
        p.as_object_mut().unwrap().remove("building_pts");
    }
    let e = v["v2"]["economy"].as_object_mut().unwrap();
    for key in ["building", "ffe_reserve_bp", "finance_index"] {
        e.remove(key);
    }
    Snapshot::from_value(&v).unwrap()
}

/// Zatvori i obračunaj krugove redom do uključivo `code`; `"none"` = nijedan, `"all"` = cijela sezona.
pub fn play_through(mut g: Game, code: &str) -> Game {
    if code == "none" {
        return g;
    }
    let last = RoundKey::parse(code);
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        apply_ev(&mut g, Actor::Guest(GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("obračun je trebao biti gotov");
        };
        for (a, e) in evs {
            apply_ev(&mut g, a, e);
        }
        if Some(r) == last {
            break;
        }
    }
    g
}
