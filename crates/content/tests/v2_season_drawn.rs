//! J-3: sezona robota s predlošcima situacija — krug dobiva primjerke od generatora (uz S1–S4 kao sidra), svaki
//! primjerak postaje situacija s odgovorima (`SituationTemplates::instance`) i igra se u svom krugu; bez predložaka
//! sezona ostaje na starom popisu. Zamrznuti sadržaj s `52ce161`, da novi predlošci u repou ne pomiču test.

use std::cell::Cell;
use std::path::PathBuf;

use content::{Content, Snapshot};
use sim_hotel_v2::situations::DrawnSituation;
use sim_hotel_v2::{
    CatalogError, Cause, PeriodKind, Profile, RefTable, ResolveError, SeasonResult,
    SeasonSituations, Strategy, TemplateDraw, run_robot_season, run_season, season_periods,
};

const SEED: u64 = 7;

fn frozen() -> Content {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content-52ce161");
    Snapshot::read_v2(&dir).unwrap().build().unwrap()
}

/// Sezona hotela C (Brižni) s predlošcima; vraća i broj poziva `instance`.
fn drawn_season(c: &Content, kind: PeriodKind) -> (SeasonResult, usize) {
    let cat = c.v2.as_ref().unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    let set = tpl.to_core();
    let category = cat.base.profile(Profile::C).unwrap().category;
    let refs = RefTable::compute(cat, category, &season_periods(kind)).unwrap();
    let calls = Cell::new(0_usize);
    let instance = |d: &DrawnSituation| {
        calls.set(calls.get() + 1);
        tpl.instance(d, &cat.base, category)
            .map_err(|e| ResolveError::Catalog(CatalogError::OutOfRange(format!("{e:?}"))))
    };
    let draw = TemplateDraw {
        anchors: &c.situations,
        templates: &set,
        seed: SEED,
        slot: 1,
        instance: &instance,
    };
    let r = run_robot_season(
        Profile::C,
        Strategy::Caring,
        SEED,
        kind,
        cat,
        SeasonSituations::Drawn(&draw),
        &refs,
    )
    .unwrap();
    (r, calls.get())
}

#[test]
fn season_with_templates_plays_generator_instances() {
    let c = frozen();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let (r, calls) = drawn_season(&c, kind);
        assert_eq!(r.drawn.len(), r.periods.len(), "{kind:?}");
        let n: usize = r.drawn.iter().map(Vec::len).sum();
        assert!(n > 0, "{kind:?}: generator nije izvukao nijedan primjerak");
        // Svaki primjerak je pretvoren u situaciju s odgovorima točno jednom.
        assert_eq!(calls, n, "{kind:?}");
        // Primjerci se igraju u svom krugu: njihove šifre su u retcima objašnjenja tog kruga.
        let played = r
            .periods
            .iter()
            .zip(&r.drawn)
            .flat_map(|((rep, _), drawn)| drawn.iter().map(move |d| (rep, d)))
            .filter(|(rep, d)| {
                rep.summary
                    .lines
                    .iter()
                    .any(|l| l.cause == Cause::Situation(d.code.clone()))
            })
            .count();
        assert!(played > 0, "{kind:?}: nijedan primjerak nije u obračunu");
        // Ponovljivo: isto sjeme, ista sezona.
        assert_eq!(drawn_season(&c, kind).0, r, "{kind:?}");
    }
}

#[test]
fn season_without_templates_keeps_the_old_list() {
    let c = frozen();
    let cat = c.v2.as_ref().unwrap();
    let category = cat.base.profile(Profile::C).unwrap().category;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let refs = RefTable::compute(cat, category, &season_periods(kind)).unwrap();
        let fixed = run_season(
            Profile::C,
            Strategy::Caring,
            SEED,
            kind,
            cat,
            &c.situations,
            &refs,
        )
        .unwrap();
        assert!(fixed.drawn.is_empty(), "{kind:?}");
        let same = run_robot_season(
            Profile::C,
            Strategy::Caring,
            SEED,
            kind,
            cat,
            SeasonSituations::Fixed(&c.situations),
            &refs,
        )
        .unwrap();
        assert_eq!(same, fixed, "{kind:?}");
    }
}
