//! G-1 popravak 1, I2 (sezona balansa): kolektivna situacija (`scope: svi`) je u sezoni robota ISTA kod svakog hotela
//! (isto sjeme i utor, različiti profili A, B, C = različita stanja i povijest) ili je nema nijedan, i nijedna sezona
//! ne nudi više grupa od `groups_per_season` (sidra uključena). Isti kod izvlačenja kao u igri (`draw_for_round`).
//!
//! Sintetski sadržaj: iz stvarnog snimka izbačene sve kolektivne osim P-REGATA (nudi grupu za srpanj), REGATA dolazi
//! u lipnju, srpnju i kolovozu, često, s dolaskom grupe u rujnu, a granica grupa po mjesecu je uklonjena — tek tada hotelske ponude grupe mogu napuniti
//! granicu sezone jednom hotelu prije kolektivne, a drugom ne. (U stvarnom sadržaju S3 nudi grupu za rujan, a REGATA
//! je samo u lipnju, pa se kršenje ne dosiže.)
//!
//! PRETPOSTAVKA O SUČELJU (jedino mjesto: `season`): sezona robota ide kroz `run_robot_season` s
//! `SeasonSituations::Drawn(&TemplateDraw)`; `SeasonResult.drawn[i]` su primjerci kruga i (kolektivna ima `slot == 0`).

use content::{Content, Snapshot, repo_dir};
use serde_json::{Value, json};
use sim_hotel::SituationTemplate;
use sim_hotel_v2::situations::DrawnSituation;
use sim_hotel_v2::{
    CatalogError, PeriodKind, Profile, RefTable, ResolveError, SeasonResult, SeasonSituations,
    Strategy, TemplateDraw, group_month, run_robot_season, season_periods, situations_for,
};

const SEEDS: u64 = 30;

/// Stvarni snimak s jednom, čestom kolektivnom (REGATA u svakom mjesecu) bez granice grupa po mjesecu.
fn content() -> Content {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let mut v: Value = serde_json::to_value(&snap).unwrap();
    let tf = v["v2"]["situation_templates"].as_object_mut().unwrap();
    tf.remove("groups_per_month");
    let list = tf["templates"].as_array_mut().unwrap();
    list.retain(|t| t["scope"] != "svi" || t["code"] == "P-REGATA");
    for t in list.iter_mut().filter(|t| t["code"] == "P-REGATA") {
        let o = t.as_object_mut().unwrap();
        o.insert("when".to_owned(), json!({"months": ["jun", "jul", "aug"]}));
        o.insert(
            "repeat".to_owned(),
            json!({"max_per_season": 12, "gap_rounds": 4}),
        );
        o.insert("chance".to_owned(), json!({"base": 5}));
        arrive_in_september(t);
    }
    serde_json::from_value::<Snapshot>(v)
        .unwrap()
        .build()
        .unwrap()
}

/// Dolazak grupe REGATE u rujan (kao S3 i hotelske grupe): validacija traži mjesece pojave prije dolaska.
fn arrive_in_september(v: &mut Value) {
    match v {
        Value::Object(m) => {
            if m.get("type").is_some_and(|t| t == "GroupBooking") {
                m.insert("month".to_owned(), json!("sep"));
            }
            m.values_mut().for_each(arrive_in_september);
        }
        Value::Array(a) => a.iter_mut().for_each(arrive_in_september),
        _ => {}
    }
}

fn season(c: &Content, profile: Profile, seed: u64, kind: PeriodKind) -> SeasonResult {
    let cat = c.v2.as_ref().unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    let set = tpl.to_core();
    let category = cat.base.profile(profile).unwrap().category;
    let refs = RefTable::compute(cat, category, &season_periods(kind)).unwrap();
    let instance = |d: &DrawnSituation| -> Result<SituationTemplate, ResolveError> {
        tpl.instance(d, &cat.base, category)
            .map_err(|e| ResolveError::Catalog(CatalogError::OutOfRange(format!("{e:?}"))))
    };
    let draw = TemplateDraw {
        anchors: &c.situations,
        templates: &set,
        seed,
        slot: 1,
        instance: &instance,
    };
    run_robot_season(
        profile,
        Strategy::Caring,
        seed,
        kind,
        cat,
        SeasonSituations::Drawn(&draw),
        &refs,
    )
    .unwrap()
}

fn collectives(r: &SeasonResult) -> Vec<Vec<DrawnSituation>> {
    r.drawn
        .iter()
        .map(|round| round.iter().filter(|s| s.slot == 0).cloned().collect())
        .collect()
}

/// Ponuđene grupe sezone: sidra (S1–S4) i primjerci čiji predložak nudi grupu.
fn offered_groups(c: &Content, r: &SeasonResult, kind: PeriodKind) -> usize {
    let tpl = c.situation_templates.as_ref().unwrap().to_core();
    let from_anchors = season_periods(kind)
        .iter()
        .flat_map(|p| situations_for(p, &c.situations))
        .filter(|t| group_month(t).is_some())
        .count();
    let from_drawn = r
        .drawn
        .iter()
        .flatten()
        .filter(|d| {
            tpl.templates
                .iter()
                .any(|t| t.code == d.template && t.group_month.is_some())
        })
        .count();
    from_anchors + from_drawn
}

#[test]
fn collective_is_the_same_in_every_hotels_season() {
    let c = content();
    let mut seen_collective = false;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in 1..=SEEDS {
            let a = collectives(&season(&c, Profile::A, seed, kind));
            let b = collectives(&season(&c, Profile::B, seed, kind));
            let cc = collectives(&season(&c, Profile::C, seed, kind));
            seen_collective |= a.iter().any(|r| !r.is_empty());
            assert_eq!(
                a, b,
                "{kind:?} sjeme {seed}: kolektivne hotela A i B nisu iste"
            );
            assert_eq!(
                a, cc,
                "{kind:?} sjeme {seed}: kolektivne hotela A i C nisu iste"
            );
        }
    }
    assert!(seen_collective, "kontrola: kolektivna nikad izvučena");
}

#[test]
fn no_season_offers_more_groups_than_the_season_limit() {
    let c = content();
    let limit = usize::from(
        c.situation_templates
            .as_ref()
            .unwrap()
            .groups_per_season
            .expect("granica grupa u sadržaju"),
    );
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in [Profile::A, Profile::B, Profile::C] {
            for seed in 1..=SEEDS {
                let n = offered_groups(&c, &season(&c, profile, seed, kind), kind);
                assert!(
                    n <= limit,
                    "{kind:?} {profile:?} sjeme {seed}: {n} ponuđenih grupa, granica {limit}"
                );
            }
        }
    }
}
