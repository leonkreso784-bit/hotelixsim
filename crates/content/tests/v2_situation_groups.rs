//! J-3: granice ponuda grupe u generatoru — najviše `group_limit` u sezoni hotela i najviše `groups_per_month` s
//! dolaskom u istom mjesecu (M4); sidra (S1–S4) koja nude grupu broje se u obje granice. Zamrznuti predlošci uz
//! testne „opće" predloške koji nude grupu (`situations_common`).

mod situations_common;

use std::collections::BTreeMap;

use sim_hotel_v2::situations::draw_situations;
use sim_hotel_v2::situations::{Draw, DrawView, DrawnSituation, Kind, Repeat, TemplateSet};
use sim_hotel_v2::{Catalog, Month, PeriodKind, Profile, season_periods};
use situations_common::{Hotel, frozen, general, indices, start};

const KINDS: [PeriodKind; 2] = [PeriodKind::Week, PeriodKind::Month];
const SEEDS: u64 = 200;

/// Zamrznuti predlošci i šest općih koji nude grupu (dolasci srpanj, kolovoz, rujan, po dva).
fn with_groups(limit: Option<u8>, per_month: Option<u8>) -> (Catalog, TemplateSet) {
    let (cat, mut set) = frozen();
    let months = [Month::Jul, Month::Aug, Month::Sep];
    for i in 0..6 {
        let mut t = general(
            &format!("P-GRUPA-{i}"),
            Kind::Request,
            Repeat {
                max_per_season: 3,
                gap_rounds: 2,
            },
        );
        t.group_month = Some(months[i % months.len()]);
        t.weight = 3;
        set.templates.push(t);
    }
    set.group_limit = limit;
    set.groups_per_month = per_month;
    (cat, set)
}

/// Sezona jednog hotela uz nepromijenjeno početno stanje; `anchor_groups` su mjeseci dolaska grupa sidara.
fn season(
    cat: &Catalog,
    set: &TemplateSet,
    kind: PeriodKind,
    seed: u64,
    anchor_groups: &[Month],
) -> Vec<DrawnSituation> {
    let state = start(cat, Profile::C);
    let h = Hotel {
        state: &state,
        indices: indices(5_000),
        slot: 1,
    };
    let mut history: Vec<DrawnSituation> = Vec::new();
    for p in season_periods(kind) {
        let view = DrawView {
            state: h.state,
            previous: None,
            indices: &h.indices,
        };
        let draw = Draw {
            period: &p,
            templates: set,
            history: &history,
            seed,
            slot: h.slot,
            anchors: &[],
            anchor_groups,
        };
        let drawn = draw_situations(&view, &draw, cat).unwrap();
        history.extend(drawn);
    }
    history
}

/// Mjeseci dolaska grupa koje sezona nudi (primjerci predložaka s grupom).
fn group_months(set: &TemplateSet, drawn: &[DrawnSituation]) -> Vec<Month> {
    drawn
        .iter()
        .filter_map(|d| {
            set.templates
                .iter()
                .find(|t| t.code == d.template)
                .and_then(|t| t.group_month)
        })
        .collect()
}

fn per_month(months: &[Month]) -> BTreeMap<Month, usize> {
    let mut m = BTreeMap::new();
    for x in months {
        *m.entry(*x).or_insert(0) += 1;
    }
    m
}

#[test]
fn without_limits_seasons_offer_many_groups() {
    // Bez granica generator nudi više grupa i više u istom mjesecu: granice u ostalim testovima stvarno rade.
    let (cat, set) = with_groups(None, None);
    for kind in KINDS {
        let (mut most, mut most_in_month) = (0, 0);
        for seed in 0..SEEDS {
            let g = group_months(&set, &season(&cat, &set, kind, seed, &[]));
            most = most.max(g.len());
            most_in_month = most_in_month.max(per_month(&g).values().copied().max().unwrap_or(0));
        }
        assert!(most > 3, "{kind:?}: najviše {most}");
        assert!(
            most_in_month > 1,
            "{kind:?}: najviše {most_in_month} u mjesecu"
        );
    }
}

#[test]
fn season_limit_caps_group_offers() {
    let (cat, set) = with_groups(Some(3), None);
    for kind in KINDS {
        let mut reached = false;
        for seed in 0..SEEDS {
            let n = group_months(&set, &season(&cat, &set, kind, seed, &[])).len();
            assert!(n <= 3, "{kind:?} sjeme {seed}: {n} ponuda grupe");
            reached |= n == 3;
        }
        assert!(reached, "{kind:?}: granica 3 nikad dosegnuta");
    }
}

#[test]
fn at_most_one_group_per_arrival_month() {
    let (cat, set) = with_groups(Some(3), Some(1));
    for kind in KINDS {
        let mut three = false;
        for seed in 0..SEEDS {
            let g = group_months(&set, &season(&cat, &set, kind, seed, &[]));
            for (m, n) in per_month(&g) {
                assert!(
                    n <= 1,
                    "{kind:?} sjeme {seed}: {n} grupe s dolaskom u {m:?}"
                );
            }
            three |= g.len() == 3;
        }
        // Tri različita mjeseca dolaska: sezona i uz M4 može imati tri grupe.
        assert!(three, "{kind:?}: nijedna sezona s tri grupe");
    }
}

#[test]
fn anchor_groups_count_toward_both_limits() {
    // Sidro s grupom u kolovozu: uz granicu 1 po sezoni generator ne nudi nijednu grupu.
    let (cat, set) = with_groups(Some(1), None);
    for kind in KINDS {
        for seed in 0..SEEDS {
            let g = group_months(&set, &season(&cat, &set, kind, seed, &[Month::Aug]));
            assert!(g.is_empty(), "{kind:?} sjeme {seed}: {g:?}");
        }
    }
    // Uz jednu po mjesecu kolovoz je zauzet, ostali mjeseci nisu.
    let (cat, set) = with_groups(None, Some(1));
    for kind in KINDS {
        let mut other = false;
        for seed in 0..SEEDS {
            let g = group_months(&set, &season(&cat, &set, kind, seed, &[Month::Aug]));
            assert!(!g.contains(&Month::Aug), "{kind:?} sjeme {seed}: {g:?}");
            other |= !g.is_empty();
        }
        assert!(other, "{kind:?}: nijedna grupa u drugim mjesecima");
    }
}
