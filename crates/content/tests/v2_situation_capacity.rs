//! Kapacitet sadržaja (S-3 popravak 1, Leon): predlošci iz repoa (`live`) kroz pravi generator
//! (`draw_situations`) s pravim profilima A/B/C. Svaki krug mora dobiti puni broj situacija (broj kruga − sidra
//! S1–S4), u tjednoj i u mjesečnoj igri, a kolovoz mjesečne igre mora imati barem 22 različita dopuštena
//! predloška za svaki profil (obične + najviše jedna kolektivna; veće ne, one dolaze samo u krajnostima).
//!
//! „Tipično stanje": sezonu igra robot (sve četiri strategije redom po sjemenu), bez situacija; krug vidi stanje
//! na svom početku, pokazatelje i obračun prošlog kruga — popunjenost i opterećenje prošlog kruga, kako ih
//! generator stvarno računa (`DrawView::previous`).

mod situations_common;

use std::collections::BTreeMap;

use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{
    Draw, DrawView, DrawnSituation, Repeat, Scope, TemplateSet, draw_situations, situation_count,
};
use sim_hotel_v2::{
    Catalog, HotelState, IndexSet, Month, Period, PeriodKind, PeriodReport, Profile, RefTable,
    Strategy, run_season, season_periods,
};
use situations_common::{Hotel, anchors_for, indices, live, round, start};

const KINDS: [PeriodKind; 2] = [PeriodKind::Week, PeriodKind::Month];
const PROFILES: [Profile; 3] = [Profile::A, Profile::B, Profile::C];
const STRATEGIES: [Strategy; 4] = [
    Strategy::Saver,
    Strategy::Caring,
    Strategy::OccupancyHunter,
    Strategy::Random,
];
const SEEDS: u64 = 100;
/// Zatvoreni odjeli od kolovoza: najviše ovoliko krugova (od 6.300) s jednom situacijom manje (izmjereno 1).
const CLOSED_SHORT_MAX: usize = 2;
/// Leon: kolovoz mjesečne igre traži barem ovoliko različitih dopuštenih predložaka po profilu.
const AUGUST_DISTINCT_MIN: usize = 22;

/// Početak jednog kruga odigrane sezone: stanje, obračun prošlog kruga i pokazatelji.
struct RoundStart {
    period: Period,
    state: HotelState,
    previous: Option<PeriodReport>,
    indices: IndexSet,
}

/// Sezona koju robot odigra bez situacija; za svaki krug ono što generator vidi na njegovu početku.
fn played(cat: &Catalog, profile: Profile, kind: PeriodKind, seed: u64) -> Vec<RoundStart> {
    let category = cat.base.profile(profile).unwrap().category;
    let refs = RefTable::compute(cat, category, &season_periods(kind)).unwrap();
    let strategy = STRATEGIES[usize::try_from(seed).unwrap() % STRATEGIES.len()];
    let r = run_season(profile, strategy, seed, kind, cat, &[], &refs).unwrap();
    let mut out = Vec::new();
    let mut previous: Option<&(PeriodReport, IndexSet)> = None;
    for item in &r.periods {
        out.push(RoundStart {
            period: item.0.period,
            state: item.0.before.clone(),
            previous: previous.map(|p| p.0.clone()),
            indices: previous.map_or_else(|| r.start.clone(), |p| p.1.clone()),
        });
        previous = Some(item);
    }
    out
}

fn draw(
    cat: &Catalog,
    set: &TemplateSet,
    r: &RoundStart,
    history: &[DrawnSituation],
    seed: u64,
    anchors: &[sim_hotel_v2::situations::Anchor],
) -> Vec<DrawnSituation> {
    let view = DrawView {
        state: &r.state,
        previous: r.previous.as_ref(),
        indices: &r.indices,
    };
    let d = Draw {
        period: &r.period,
        templates: set,
        history,
        seed,
        slot: 1,
        anchors,
        anchor_groups: &[],
    };
    draw_situations(&view, &d, cat).unwrap()
}

/// Nedostaje po (profil, igra, mjesec): (traženo, izvučeno) zbrojeno preko svih sjemena.
type Totals = BTreeMap<(String, String, Month), (u32, u32)>;

fn kind_code(kind: PeriodKind) -> &'static str {
    match kind {
        PeriodKind::Week => "tjedno",
        PeriodKind::Month => "mjesečno",
    }
}

/// Svaki krug tipične sezone dobiva puni broj situacija: tjedna i mjesečna igra, A/B/C, 100 sjemena.
#[test]
fn typical_season_fills_every_round() {
    let (cat, set) = live();
    let mut totals = Totals::new();
    let mut short: Vec<String> = Vec::new();
    for kind in KINDS {
        for profile in PROFILES {
            for seed in 0..SEEDS {
                let mut history: Vec<DrawnSituation> = Vec::new();
                for r in played(&cat, profile, kind, seed) {
                    let anchors = anchors_for(kind, r.period.index);
                    let drawn = draw(&cat, &set, &r, &history, seed, &anchors);
                    let n = usize::from(situation_count(&r.state, &r.period, &cat).unwrap());
                    let want = n - anchors.len();
                    let e = totals
                        .entry((
                            format!("{profile:?}"),
                            kind_code(kind).to_string(),
                            r.period.start_month(),
                        ))
                        .or_default();
                    e.0 += u32::try_from(want).unwrap();
                    e.1 += u32::try_from(drawn.len()).unwrap();
                    if drawn.len() != want {
                        short.push(format!(
                            "{} {profile:?} sjeme {seed}: {} od {want}",
                            r.period.code(),
                            drawn.len()
                        ));
                    }
                    history.extend(drawn);
                }
            }
        }
    }
    println!("profil | igra | mjesec | traženo | izvučeno (zbroj {SEEDS} sjemena)");
    for ((profile, kind, month), (want, got)) in &totals {
        println!("{profile} | {kind} | {month:?} | {want} | {got}");
    }
    assert!(
        short.is_empty(),
        "{} krugova bez punog broja, npr. {:?}",
        short.len(),
        &short[..short.len().min(10)]
    );
}

/// Zatvoreni odjeli (pregled S-3 popravak 1, m7): od kolovoza su zatvoreni svi odjeli koji se smiju zatvoriti
/// (lobby bar, rooftop bar, bar uz bazen, wellness) u svakom krugu koji počinje u kolovozu ili rujnu. Generator
/// ih tada ne nudi (`DrawView::state`), a krugovi su puni; tjedno i mjesečno, A/B/C, 100 sjemena.
///
/// Izmjereno (S-4): jedan od 6.300 krugova (mjesečni kolovoz, A, sjeme 69) dobije 20 od 21 situacije. Predložaka
/// ima dovoljno (17 dopuštenih ostaje neizvučeno), ali su gosti tada samo u Smještaju i restoranu, a granica po
/// odjelu je namjerno stroga (`draw.rs`: „radije manje situacija nego tri iz istog odjela u tjednu"). Zato ovdje
/// najviše jedna situacija manje i najviše `CLOSED_SHORT_MAX` takvih krugova.
#[test]
fn closed_departments_from_august_keep_rounds_nearly_full() {
    let (cat, set) = live();
    let closable: Vec<DeptCode> = cat
        .departments
        .iter()
        .filter(|(_, s)| s.can_close)
        .map(|(k, _)| k.clone())
        .collect();
    assert_eq!(closable.len(), 4, "{closable:?}");
    let mut short: Vec<String> = Vec::new();
    for kind in KINDS {
        for profile in PROFILES {
            for seed in 0..SEEDS {
                let mut history: Vec<DrawnSituation> = Vec::new();
                for mut r in played(&cat, profile, kind, seed) {
                    if r.period.start_month() >= Month::Aug {
                        for code in &closable {
                            if let Some(d) = r.state.departments.get_mut(code) {
                                d.open = false;
                            }
                        }
                    }
                    let anchors = anchors_for(kind, r.period.index);
                    let drawn = draw(&cat, &set, &r, &history, seed, &anchors);
                    assert!(
                        drawn.iter().all(|d| r.period.start_month() < Month::Aug
                            || d.department.as_ref().is_none_or(|c| !closable.contains(c))),
                        "{}: situacija na zatvorenom odjelu",
                        r.period.code()
                    );
                    let n = usize::from(situation_count(&r.state, &r.period, &cat).unwrap());
                    let want = n - anchors.len();
                    assert!(
                        drawn.len() + 1 >= want,
                        "{} {} {profile:?} sjeme {seed}: {} od {want}",
                        kind_code(kind),
                        r.period.code(),
                        drawn.len()
                    );
                    if drawn.len() != want {
                        short.push(format!(
                            "{} {} {profile:?} sjeme {seed}: {} od {want}",
                            kind_code(kind),
                            r.period.code(),
                            drawn.len()
                        ));
                    }
                    history.extend(drawn);
                }
            }
        }
    }
    println!("zatvoreni odjeli od kolovoza, krugovi bez punog broja: {short:?}");
    assert!(short.len() <= CLOSED_SHORT_MAX, "{short:?}");
}

/// Različiti predlošci koje generator u kolovozu mjesečne igre stvarno dopušta (svaki predložak sam u skupu, bez
/// povijesti): obične + najviše jedna kolektivna; veće se ne broje.
fn august_distinct(cat: &Catalog, set: &TemplateSet, r: &RoundStart) -> usize {
    let free = Repeat {
        max_per_season: 17,
        gap_rounds: 1,
    };
    let mut ordinary = 0;
    let mut collective = 0;
    for t in set.templates.iter().filter(|t| t.major.is_none()) {
        let mut one = t.clone();
        one.repeat = free;
        let alone = TemplateSet {
            major: set.major,
            templates: vec![one],
            names: set.names.clone(),
            group_limit: None,
            groups_per_month: None,
            min_gap_weeks: set.min_gap_weeks,
            opportunity_every_weeks: set.opportunity_every_weeks,
            counts: set.counts,
        };
        if draw(cat, &alone, r, &[], 0, &[])
            .iter()
            .any(|d| d.template == t.code)
        {
            match t.scope {
                Scope::Hotel => ordinary += 1,
                Scope::All => collective += 1,
            }
        }
    }
    ordinary + collective.min(1)
}

#[test]
fn august_has_enough_distinct_templates_for_every_profile() {
    let (cat, set) = live();
    let mut low: BTreeMap<String, usize> = BTreeMap::new();
    for profile in PROFILES {
        for seed in 0..SEEDS {
            let rounds = played(&cat, profile, PeriodKind::Month, seed);
            let aug = rounds
                .iter()
                .find(|r| r.period.start_month() == Month::Aug)
                .unwrap();
            let n = august_distinct(&cat, &set, aug);
            let e = low.entry(format!("{profile:?}")).or_insert(usize::MAX);
            *e = (*e).min(n);
        }
    }
    println!("najmanje različitih dopuštenih u kolovozu (mjesečna igra, {SEEDS} sjemena): {low:?}");
    assert!(low.values().all(|&n| n >= AUGUST_DISTINCT_MIN), "{low:?}");
}

/// Pokrivenost iz početnog stanja (pregled J-2, I2): A/B/C, tjedna i mjesečna igra, nepromijenjeno početno stanje
/// i prosječni pokazatelji — svaki krug dobiva točno `broj − sidra`.
#[test]
fn repo_templates_fill_every_round() {
    let (cat, set) = live();
    let mut short: Vec<String> = Vec::new();
    for kind in KINDS {
        for profile in PROFILES {
            let state = start(&cat, profile);
            let h = Hotel {
                state: &state,
                indices: indices(5_000),
                slot: 1,
            };
            for seed in 0..SEEDS {
                let mut history: Vec<DrawnSituation> = Vec::new();
                for p in season_periods(kind) {
                    let anchors = anchors_for(kind, p.index);
                    let drawn = round(&cat, &set, &h, &p, &history, seed, &anchors);
                    let n = usize::from(situation_count(&state, &p, &cat).unwrap());
                    if drawn.len() != n - anchors.len() {
                        short.push(format!(
                            "{} {profile:?} sjeme {seed}: {} od {}",
                            p.code(),
                            drawn.len(),
                            n - anchors.len()
                        ));
                    }
                    history.extend(drawn);
                }
            }
        }
    }
    assert!(
        short.is_empty(),
        "{} krugova bez punog broja: {:?}",
        short.len(),
        &short[..short.len().min(20)]
    );
}
