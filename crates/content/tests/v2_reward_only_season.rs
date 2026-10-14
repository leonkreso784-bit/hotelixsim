//! S-4 Minor 1 (put robota): u sezoni s predlošcima primjerak predloška s oznakom „samo nagrada" (P-POHVALA) ne smije
//! sniziti pokazatelje ni kad je odgovor loš. Brisanje `if no_penalty { reward_only(o) }` u `season.rs` mora pasti.
//!
//! Svi ponuđeni odgovori primjerka P-POHVALA zamijenjeni su istom mehanikom zadržavanja ljudi (bodovi
//! `(q − 3) × k`, k > 0) i istom kvalitetom X za sva tri odgovora, pa izbor robota ne smeta. Sezona s X = 1 mora biti
//! ista kao sezona s X = 3 (neutralna); sezona s X = 5 mora se razlikovati (nagrada nije ugašena).

use content::{Content, Snapshot, repo_dir};
use sim_hotel::{Mechanic, OfferedAnswer, SituationTemplate};
use sim_hotel_v2::situations::DrawnSituation;
use sim_hotel_v2::{
    CatalogError, PeriodKind, Profile, RefTable, ResolveError, SeasonResult, SeasonSituations,
    Strategy, TemplateDraw, run_robot_season, season_periods,
};

const PRAISE: &str = "P-POHVALA";

fn content() -> Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

fn retention() -> Mechanic {
    Mechanic::StaffRetention {
        raise_one_pct: 0,
        bonus_cents: 0,
    }
}

/// Sezona hotela C (Brižni) u kojoj svi odgovori pohvale imaju kvalitetu `q`.
fn season(c: &Content, seed: u64, kind: PeriodKind, q: u8) -> SeasonResult {
    let cat = c.v2.as_ref().unwrap();
    let tpl = c.situation_templates.as_ref().unwrap();
    let set = tpl.to_core();
    let category = cat.base.profile(Profile::C).unwrap().category;
    let refs = RefTable::compute(cat, category, &season_periods(kind)).unwrap();
    let instance = |d: &DrawnSituation| -> Result<SituationTemplate, ResolveError> {
        let mut t = tpl
            .instance(d, &cat.base, category)
            .map_err(|e| ResolveError::Catalog(CatalogError::OutOfRange(format!("{e:?}"))))?;
        if d.template == PRAISE {
            t.answers = (0..3)
                .map(|i| OfferedAnswer {
                    label: format!("odgovor {i}"),
                    mechanics: vec![retention()],
                    quality: q,
                })
                .collect();
            t.unanswered = vec![retention()];
        }
        Ok(t)
    };
    let draw = TemplateDraw {
        anchors: &c.situations,
        templates: &set,
        seed,
        slot: 1,
        instance: &instance,
    };
    run_robot_season(
        Profile::C,
        Strategy::Caring,
        seed,
        kind,
        cat,
        SeasonSituations::Drawn(&draw),
        &refs,
    )
    .unwrap()
}

fn has_praise(r: &SeasonResult) -> bool {
    r.drawn.iter().flatten().any(|d| d.template == PRAISE)
}

#[test]
fn reward_only_template_never_lowers_indicators_in_a_robot_season() {
    let c = content();
    assert!(
        c.situation_templates
            .as_ref()
            .unwrap()
            .no_penalty(&format!("{PRAISE}.t01")),
        "pretpostavka: P-POHVALA nosi oznaku „samo nagrada“"
    );
    let mut checked = 0;
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for seed in 1..=40_u64 {
            let neutral = season(&c, seed, kind, 3);
            if !has_praise(&neutral) {
                continue;
            }
            checked += 1;
            let poor = season(&c, seed, kind, 1);
            let poorer = season(&c, seed, kind, 2);
            assert_eq!(poor, neutral, "{kind:?} sjeme {seed}: q1 mora biti kao q3");
            assert_eq!(
                poorer, neutral,
                "{kind:?} sjeme {seed}: q2 mora biti kao q3"
            );
            let best = season(&c, seed, kind, 5);
            assert_ne!(
                best, neutral,
                "{kind:?} sjeme {seed}: nagrada q5 se mora vidjeti"
            );
            if checked >= 3 {
                break;
            }
        }
    }
    assert!(
        checked >= 2,
        "pohvala nije izvučena ni u jednoj od sezona ({checked})"
    );
}
