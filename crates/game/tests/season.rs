//! Kraj sezone (kraj-sezone-spec §5–§6.1): poredak, krivulja, indeksi, financije i odluke svih hotela nakon
//! zadnjeg kruga v2 igre. Ništa se ne računa novo: zbrojevi i prosjeci polja iz dnevnika.

mod common;

use game::*;
use uuid::Uuid;

fn weekly_done() -> Game {
    common::v2_weekly_game_resolved_through("t17")
}

fn monthly_done() -> Game {
    common::v2_monthly_game_resolved_through("sep")
}

fn me() -> HotelId {
    common::hotel_ids()[0]
}

fn by_strategy<'a>(v: &'a SeasonView, code: &str) -> &'a SeasonHotel {
    v.hotels
        .iter()
        .find(|h| matches!(&h.seat, view::Seat::Robot { strategy } if strategy == code))
        .unwrap_or_else(|| panic!("robot {code}"))
}

/// Ista igra iz izmijenjenog dnevnika (fold ne preračunava obračun, pa se ishod može namjestiti).
fn refold(g: &Game, f: impl FnOnce(&mut Vec<Recorded>)) -> Game {
    let mut log = g.history.clone();
    f(&mut log);
    Game::fold(&log).unwrap()
}

fn set_final_progress(g: &Game, progress: [(usize, i32); 4]) -> Game {
    let last = *g.rounds.last().unwrap();
    refold(g, |log| {
        for r in log.iter_mut() {
            if let GameEvent::RoundResolved(rr) = &mut r.event
                && rr.month == last
            {
                for (i, p) in progress {
                    let id = common::hotel_ids()[i];
                    let h = rr.hotels.iter_mut().find(|h| h.hotel_id == id).unwrap();
                    h.progress_hundredths = p;
                }
            }
        }
    })
}

fn check_full_season(g: &Game) {
    let v = g.season(me()).unwrap();
    assert_eq!(v.rounds.len(), g.rounds.len());
    assert_eq!(v.hotels.len(), 4);
    assert_eq!(v.mentor, None);

    let ranks: Vec<u8> = v.hotels.iter().map(|h| h.rank).collect();
    let mut sorted = ranks.clone();
    sorted.sort_unstable();
    assert_eq!(ranks, sorted, "hoteli su redom poretka");
    assert_eq!(ranks[0], 1);
    for w in v.hotels.windows(2) {
        assert!(w[0].progress_hundredths >= w[1].progress_hundredths);
    }
    assert_eq!(v.hotels.iter().filter(|h| h.mine).count(), 1);
    for h in &v.hotels {
        assert_eq!(h.winner, h.rank == 1, "{}", h.name);
    }

    let last = *g.rounds.last().unwrap();
    for h in &v.hotels {
        let public = g.public_hotel(h.hotel_id).unwrap();
        assert_eq!(h.curve.len(), v.rounds.len());
        assert_eq!(h.start, public.start);
        for (c, m) in h.curve.iter().zip(&public.months) {
            assert_eq!(
                (c.round, c.progress_hundredths),
                (m.round, m.progress_hundredths)
            );
        }
        assert_eq!(
            h.progress_hundredths,
            public.months.last().unwrap().progress_hundredths
        );
        let res = g.resolved[&last]
            .hotels
            .iter()
            .find(|x| x.hotel_id == h.hotel_id)
            .unwrap();
        assert_eq!(h.end, res.indices);

        // Svi su ovdje robot ili ja, pa svi imaju brojke (pravilo §12 skriva samo tuđeg čovjeka).
        let f = h.finances.as_ref().expect("financije");
        let pl = g
            .profit_and_loss(h.hotel_id, None, PlScope::Season)
            .unwrap();
        assert_eq!(f.total_revenue_cents, pl.actual.total_revenue_cents);
        assert_eq!(f.gop_cents, pl.actual.gop_cents);
        assert_eq!(f.gop_margin_bp, pl.actual.gop_margin_bp);
        assert_eq!(
            f.cash_end_cents,
            g.latest_state(h.hotel_id).unwrap().summary().cash_cents
        );
        assert_eq!(
            f.cash_start_cents,
            g.hotel(h.hotel_id)
                .unwrap()
                .start_state
                .summary()
                .cash_cents
        );
        let d = h.decisions.as_ref().expect("odluke");
        // Situacije su po hotelu (primjerci iz predložaka razlikuju se među hotelima), pa se broje za taj hotel.
        let n: usize = g
            .rounds
            .iter()
            .map(|r| g.situations_of_hotel(h.hotel_id, *r).unwrap().len())
            .sum();
        assert_eq!(usize::from(d.situations), n);
        assert!(d.unanswered <= d.situations);
        assert!(
            d.avg_quality_hundredths
                .is_some_and(|q| (100..=500).contains(&q))
        );
    }

    // Tekst „Kako je igrao" (spec §4) drži se ovih brojki iz koda robota.
    let n = u8::try_from(v.rounds.len()).unwrap();
    let brizni = by_strategy(&v, "brizni").decisions.clone().unwrap();
    assert_eq!((brizni.wage_pct_min, brizni.wage_pct_max), (103, 103));
    assert_eq!(brizni.training_rounds, n);
    assert_eq!(brizni.maintenance_deferred_rounds, 0);
    let stedisa = by_strategy(&v, "stedisa").decisions.clone().unwrap();
    assert_eq!(stedisa.marketing_total_cents, 0);
    assert_eq!((stedisa.wage_pct_min, stedisa.wage_pct_max), (95, 95));
    assert_eq!(stedisa.training_rounds, 0);
    assert_eq!(stedisa.maintenance_deferred_rounds, n);
    assert_eq!(stedisa.renovation_round, None);
    assert_eq!(stedisa.reply_reviews_rounds, 0);
    let lovac = by_strategy(&v, "lovac").decisions.clone().unwrap();
    assert!(lovac.marketing_total_cents > brizni.marketing_total_cents);
    assert!(lovac.avg_price_cents < brizni.avg_price_cents);
}

#[test]
fn weekly_season_has_all_hotels_ranked_with_curves_and_money() {
    check_full_season(&weekly_done());
}

#[test]
fn monthly_season_has_all_hotels_ranked_with_curves_and_money() {
    check_full_season(&monthly_done());
}

#[test]
fn equal_progress_shares_rank_and_keeps_slot_order() {
    let g = set_final_progress(&monthly_done(), [(0, 900), (1, 500), (2, 500), (3, -100)]);
    let v = g.season(me()).unwrap();
    let got: Vec<(u8, u8, bool)> = v
        .hotels
        .iter()
        .map(|h| (h.slot, h.rank, h.winner))
        .collect();
    assert_eq!(
        got,
        [(1, 1, true), (2, 2, false), (3, 2, false), (4, 4, false)]
    );
}

#[test]
fn shared_first_place_makes_two_winners() {
    let g = set_final_progress(&monthly_done(), [(0, 700), (1, 100), (2, 50), (3, 700)]);
    let v = g.season(me()).unwrap();
    let got: Vec<(u8, u8, bool)> = v
        .hotels
        .iter()
        .map(|h| (h.slot, h.rank, h.winner))
        .collect();
    assert_eq!(
        got,
        [(1, 1, true), (4, 1, true), (2, 3, false), (3, 4, false)]
    );
}

#[test]
fn other_human_hotel_shows_only_indices() {
    let other = GuestId(Uuid::from_u128(77));
    let g = refold(&monthly_done(), |log| {
        let GameEvent::GameCreated(c) = &mut log[0].event else {
            panic!("prvi događaj")
        };
        let h = c
            .hotels
            .iter_mut()
            .find(|h| h.hotel_id == common::hotel_ids()[1])
            .unwrap();
        h.controller = Controller::Human { guest_id: other };
    });
    let v = g.season(me()).unwrap();
    let theirs = v
        .hotels
        .iter()
        .find(|h| h.hotel_id == common::hotel_ids()[1])
        .unwrap();
    assert!(!theirs.mine);
    assert_eq!(theirs.seat, view::Seat::Human);
    assert_eq!(theirs.finances, None);
    assert_eq!(theirs.decisions, None);
    assert_eq!(theirs.curve.len(), 4);
    let mine = v.hotels.iter().find(|h| h.mine).unwrap();
    assert!(mine.finances.is_some() && mine.decisions.is_some());
    // Isti hotel gledan iz drugog sjedala: tada je moj hotel tuđi.
    let v2 = g.season(common::hotel_ids()[1]).unwrap();
    let first = v2.hotels.iter().find(|h| h.hotel_id == me()).unwrap();
    assert_eq!(
        (first.finances.as_ref(), first.decisions.as_ref()),
        (None, None)
    );
}

#[test]
fn season_in_progress_is_not_over() {
    let g = common::v2_weekly_game_resolved_through("t16");
    assert_eq!(g.season(me()).unwrap_err(), GameError::SeasonNotOver);
    let g = common::v2_monthly_game_resolved_through("none");
    assert_eq!(g.season(me()).unwrap_err(), GameError::SeasonNotOver);
}

#[test]
fn v1_game_has_no_season_view() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.season(h).unwrap_err(), GameError::WrongModel);
}

#[test]
fn unknown_viewer_is_unknown_hotel() {
    let g = monthly_done();
    assert_eq!(
        g.season(HotelId(Uuid::from_u128(5))).unwrap_err(),
        GameError::UnknownHotel
    );
}
