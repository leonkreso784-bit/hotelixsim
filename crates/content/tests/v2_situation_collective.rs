//! G-1 popravak 1, I2 (generator): kolektivna situacija (`scope: svi`) je JEDNA po krugu za cijelu igru, ista kod
//! svakog hotela, i ne smije probiti granicu nijednog hotela — ni granicu grupa (`group_limit`) ni broj situacija
//! kruga (`situation_count`). Kolektivni predložak koji nudi grupu ulazi samo ako svaki hotel ima mjesta za grupu;
//! kolektivna se izvlači samo ako svaki hotel ima slobodno mjesto.
//!
//! Zamrznuti sadržaj + sintetski predlošci (`situations_common`): jedan kolektivni koji nudi grupu i četiri hotelska
//! koja nude grupu (njihove šifre trebaju samo da ih povijest može brojati). Stvarni sadržaj ne dosiže kršenje
//! (S3 nudi grupu za rujan, REGATA je samo u lipnju kad je povijest prazna), pa test mora biti sintetski.
//!
//! PRETPOSTAVKA O SUČELJU (jedino mjesto: `draw_hotel` i `effective`): danas generator izvlači po hotelu
//! (`draw_situations`, kolektivna je u izlazu kao primjerak s `slot == 0`), a igra (`game::drawn::draw_round`) uzme
//! kolektivnu prvog hotela koji je dobije i dade je svima. `effective` modelira to pravilo (G-1). Kad graditelj uvede
//! korak „kolektivna za krug" (ulazi svih hotela), uskladi samo ova dva pomoćnika; tvrdnje ostaju iste.

mod situations_common;

use std::borrow::Borrow;
use std::collections::BTreeMap;

use sim_hotel_v2::situations::{
    Anchor, Draw, DrawView, DrawnSituation, Kind, Repeat, Scope, TemplateSet, draw_collective,
    draw_situations_with, situation_count,
};
use sim_hotel_v2::{Catalog, Month, Period, PeriodKind, Profile, season_periods};
use situations_common::{anchor, frozen, general, indices, start};

const SEEDS: u64 = 40;

/// Zamrznuti sadržaj, jedna kolektivna s grupom (dolazak srpanj) i četiri hotelska s grupom (dolazak rujan).
/// Granica sezone 3, bez granice po mjesecu.
fn content() -> (Catalog, TemplateSet) {
    let (cat, mut set) = frozen();
    set.templates.retain(|t| t.scope != Scope::All);
    let mut c = general(
        "P-KOL-GRUPA",
        Kind::Request,
        Repeat {
            max_per_season: 6,
            gap_rounds: 4,
        },
    );
    c.scope = Scope::All;
    c.weight = 4;
    c.group_month = Some(Month::Jul);
    set.templates.push(c);
    for i in 0..4 {
        let mut g = general(
            &format!("P-GRUPA-{i}"),
            Kind::Request,
            Repeat {
                max_per_season: 3,
                gap_rounds: 2,
            },
        );
        g.group_month = Some(Month::Sep);
        set.templates.push(g);
    }
    set.group_limit = Some(3);
    set.groups_per_month = None;
    (cat, set)
}

/// Primjerak iz prijašnjeg kruga sezone hotela `slot` (povijest generatora).
fn old(template: &str, p: &Period, slot: u8) -> DrawnSituation {
    DrawnSituation {
        code: format!("{template}.{}", p.index),
        template: template.to_owned(),
        period: *p,
        day: p.start_day,
        occurrence: 1,
        slot,
        kind: Kind::Request,
        weight: 1,
        scope: Scope::Hotel,
        major: None,
        department: None,
        roles: BTreeMap::new(),
        numbers: BTreeMap::new(),
    }
}

/// Hotel u krugu: utor, povijest, sidra kruga i mjeseci grupa koje nude sidra (kao S3).
struct Hotel {
    slot: u8,
    history: Vec<DrawnSituation>,
    anchors: Vec<Anchor>,
    anchor_groups: Vec<Month>,
}

/// Izvlačenje kruga za SVE hotele (jedino mjesto pretpostavke o sučelju generatora): kolektivna se izvlači jednom
/// za sve (`draw_collective`, ulaz su svi hoteli), pa svaki hotel svoje (`draw_situations_with`); izlaz hotela
/// sadrži kolektivnu kao primjerak utora 0.
fn draw_all<H: Borrow<Hotel>>(
    cat: &Catalog,
    set: &TemplateSet,
    p: &Period,
    hotels: &[H],
    seed: u64,
) -> Vec<Vec<DrawnSituation>> {
    let state = start(cat, Profile::C);
    let idx = indices(5_000);
    let view = DrawView {
        state: &state,
        previous: None,
        indices: &idx,
    };
    let draws: Vec<Draw> = hotels
        .iter()
        .map(|h| {
            let h = h.borrow();
            Draw {
                period: p,
                templates: set,
                history: &h.history,
                seed,
                slot: h.slot,
                anchors: &h.anchors,
                anchor_groups: &h.anchor_groups,
            }
        })
        .collect();
    let all: Vec<(DrawView, Draw)> = draws.iter().map(|d| (view, *d)).collect();
    let shared = draw_collective(&all, cat).unwrap();
    draws
        .iter()
        .map(|d| draw_situations_with(&view, d, cat, shared.as_ref()).unwrap())
        .collect()
}

fn collective_of(found: &[DrawnSituation]) -> Vec<DrawnSituation> {
    found.iter().filter(|s| s.slot == 0).cloned().collect()
}

/// Što hotel stvarno dobiva u igri: vlastiti primjerci i kolektivna prvog hotela koji je dobije (G-1,
/// `draw_round`), jedna za sve.
fn effective(found: &[Vec<DrawnSituation>]) -> (Vec<DrawnSituation>, Vec<Vec<DrawnSituation>>) {
    let shared = found
        .iter()
        .map(|f| collective_of(f))
        .find(|c| !c.is_empty())
        .unwrap_or_default();
    let own = found
        .iter()
        .map(|f| f.iter().filter(|s| s.slot != 0).cloned().collect())
        .collect();
    (shared, own)
}

fn group_months(set: &TemplateSet, list: &[DrawnSituation]) -> Vec<Month> {
    list.iter()
        .filter_map(|d| {
            set.templates
                .iter()
                .find(|t| t.code == d.template)
                .and_then(|t| t.group_month)
        })
        .collect()
}

fn aug(kind: PeriodKind) -> Period {
    // Prvi krug kolovoza: kolektivna je dopuštena samo u prvom krugu mjeseca.
    season_periods(kind)
        .into_iter()
        .find(|p| p.start_month() == Month::Aug)
        .unwrap()
}

fn hotels_with_history(p_old: &[Period]) -> Vec<Hotel> {
    let groups = |slot: u8, n: usize| -> Vec<DrawnSituation> {
        (0..n)
            .map(|i| old(&format!("P-GRUPA-{i}"), &p_old[i], slot))
            .collect()
    };
    vec![
        // Utor 1: nema vlastitih grupa, mjesta ima.
        Hotel {
            slot: 1,
            history: vec![],
            anchors: vec![],
            anchor_groups: vec![Month::Sep],
        },
        // Utor 2: sidro S3 + dva vlastita primjerka s grupom = granica 3, bez mjesta za grupu.
        Hotel {
            slot: 2,
            history: groups(2, 2),
            anchors: vec![],
            anchor_groups: vec![Month::Sep],
        },
        // Utor 3: sidro + jedan primjerak, ima mjesta.
        Hotel {
            slot: 3,
            history: groups(3, 1),
            anchors: vec![],
            anchor_groups: vec![Month::Sep],
        },
    ]
}

/// Kolektivna je kod svakog hotela ista (ili ju nema nijedan), i kad jedan hotel nema mjesta za grupu.
#[test]
fn collective_is_the_same_at_every_hotel_when_one_hotel_has_no_group_room() {
    let (cat, set) = content();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let p = aug(kind);
        let olds: Vec<Period> = season_periods(kind).into_iter().take(3).collect();
        let hotels = hotels_with_history(&olds);
        for seed in 0..SEEDS {
            let found = draw_all(&cat, &set, &p, &hotels, seed);
            let first = collective_of(&found[0]);
            for (h, f) in hotels.iter().zip(&found) {
                assert_eq!(
                    collective_of(f),
                    first,
                    "{kind:?} sjeme {seed}: kolektivna hotela utora {} nije ista kao hotela utora 1 \
                     (hotel bez mjesta za grupu ne smije promijeniti kolektivnu)",
                    h.slot
                );
            }
        }
    }
}

/// Nijedan hotel nema više situacija od broja kruga ni više ponuđenih grupa od granice sezone, računajući
/// kolektivnu koju dobiva; sidra uključena.
#[test]
fn no_hotel_goes_over_the_round_count_or_the_group_limit() {
    let (cat, set) = content();
    let state = start(&cat, Profile::C);
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let p = aug(kind);
        let n = usize::from(situation_count(&state, &p, &cat).unwrap());
        let olds: Vec<Period> = season_periods(kind).into_iter().take(3).collect();
        let hotels = hotels_with_history(&olds);
        for seed in 0..SEEDS {
            let found = draw_all(&cat, &set, &p, &hotels, seed);
            let (shared, own) = effective(&found);
            for (h, mine) in hotels.iter().zip(&own) {
                let total = h.anchors.len() + mine.len() + shared.len();
                assert!(
                    total <= n,
                    "{kind:?} sjeme {seed}: hotel utora {} ima {total} situacija, krug ima {n}",
                    h.slot
                );
                let groups = h.anchor_groups.len()
                    + group_months(&set, &h.history).len()
                    + group_months(&set, mine).len()
                    + group_months(&set, &shared).len();
                assert!(
                    groups <= 3,
                    "{kind:?} sjeme {seed}: hotel utora {} ima {groups} ponuđenih grupa, granica je 3",
                    h.slot
                );
            }
        }
    }
}

/// Hotel bez slobodnog mjesta (sidra popunila cijeli krug) znači da nema kolektivne ni za koga: inače bi taj hotel
/// dobio n + 1 situaciju.
#[test]
fn collective_is_not_drawn_when_a_hotel_has_no_free_slot() {
    let (cat, set) = content();
    let state = start(&cat, Profile::C);
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let p = aug(kind);
        let n = usize::from(situation_count(&state, &p, &cat).unwrap());
        let hotels = [
            Hotel {
                slot: 1,
                history: vec![],
                anchors: vec![],
                anchor_groups: vec![],
            },
            // Sidra zauzimaju cijeli krug: `slots == 0`.
            Hotel {
                slot: 2,
                history: vec![],
                anchors: (0..n).map(|_| anchor(Kind::Complaint)).collect(),
                anchor_groups: vec![],
            },
        ];
        for seed in 0..SEEDS {
            let found = draw_all(&cat, &set, &p, &hotels, seed);
            let first = collective_of(&found[0]);
            assert_eq!(
                collective_of(&found[1]),
                first,
                "{kind:?} sjeme {seed}: kolektivna mora biti ista kod hotela s mjestom i bez mjesta"
            );
            let (shared, own) = effective(&found);
            for (h, mine) in hotels.iter().zip(&own) {
                let total = h.anchors.len() + mine.len() + shared.len();
                assert!(
                    total <= n,
                    "{kind:?} sjeme {seed}: hotel utora {} ima {total} situacija, krug ima {n}",
                    h.slot
                );
            }
        }
    }
}

/// Kontrola: kad svi hoteli imaju mjesta, kolektivna se izvlači (test nije prazan) i ista je kod svih; dobiva je i
/// onaj bez vlastitih grupa i onaj s jednom.
#[test]
fn control_collective_is_drawn_when_every_hotel_has_room() {
    let (cat, set) = content();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let p = aug(kind);
        let olds: Vec<Period> = season_periods(kind).into_iter().take(3).collect();
        let all = hotels_with_history(&olds);
        let hotels = [&all[0], &all[2]];
        let mut drawn = 0;
        for seed in 0..SEEDS {
            let found = draw_all(&cat, &set, &p, &hotels, seed);
            let first = collective_of(&found[0]);
            assert_eq!(collective_of(&found[1]), first, "{kind:?} sjeme {seed}");
            drawn += usize::from(!first.is_empty());
        }
        assert!(drawn > 0, "{kind:?}: kolektivna nikad izvučena");
    }
}
