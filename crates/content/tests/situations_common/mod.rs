//! Zajedničko za testove izvlačenja situacija (J-2): zamrznuti sadržaj (`frozen`) i sadržaj iz repoa (`live`),
//! testni „opći" predlošci, krug i sezona jednog hotela uz nepromijenjeno početno stanje.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use content::{Snapshot, repo_dir};
use sim_hotel_v2::situations::{
    Anchor, Chance, Conditions, DeptTarget, Draw, DrawView, DrawnSituation, Gender, Kind, Repeat,
    RoleSource, RoleSpec, Scope, SituationSpec, TemplateSet, draw_situations,
};
use sim_hotel_v2::{Catalog, HotelState, IndexSet, Period, PeriodKind, Profile, season_periods};

/// Sadržaj iz `dir` (katalog v2 i predlošci u jezgrenom obliku).
fn read(dir: &Path) -> (Catalog, TemplateSet) {
    let c = Snapshot::read_v2(dir).unwrap().build().unwrap();
    let set = c
        .situation_templates
        .expect("v2 snimak ima predloške")
        .to_core();
    (c.v2.expect("v2 katalog"), set)
}

/// Zamrznuti sadržaj s `52ce161` (tri predloška S-1, katalog i imena toga trenutka;
/// `tests/fixtures/content-52ce161`, zapečaćen u `fixtures_sealed.rs`). Testovi pravila generatora i otisci
/// koriste njega, da novi predlošci u repou ne pomiču ni pravila ni otiske.
pub fn frozen() -> (Catalog, TemplateSet) {
    read(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content-52ce161"))
}

/// Sadržaj iz repoa (`content/hotel`): za testove pokrivenosti stvarnih predložaka.
pub fn live() -> (Catalog, TemplateSet) {
    read(&repo_dir())
}

pub fn indices(avg_hundredths: u16) -> IndexSet {
    IndexSet::from_hundredths([avg_hundredths; 5]).unwrap()
}

/// Opći predložak bez uvjeta (cijeli hotel).
pub fn general(code: &str, kind: Kind, repeat: Repeat) -> SituationSpec {
    SituationSpec {
        code: code.to_string(),
        kind,
        department: DeptTarget::Hotel,
        weight: 1,
        scope: Scope::Hotel,
        major: None,
        when: Conditions::default(),
        chance: Chance {
            base: 1,
            boost: vec![],
        },
        repeat,
        roles: BTreeMap::from([(
            "gost".to_string(),
            RoleSpec {
                source: RoleSource::Guest,
                gender: Gender::M,
                types: vec![],
            },
        )]),
        numbers: BTreeMap::new(),
    }
}

pub fn with_general(mut set: TemplateSet, n: usize, repeat: Repeat) -> TemplateSet {
    let kinds = [Kind::Request, Kind::Breakdown, Kind::Crisis, Kind::Staff];
    for i in 0..n {
        set.templates.push(general(
            &format!("P-OPCI-{i}"),
            kinds[i % kinds.len()],
            repeat,
        ));
    }
    set
}

pub struct Hotel<'a> {
    pub state: &'a HotelState,
    pub indices: IndexSet,
    pub slot: u8,
}

pub fn round(
    cat: &Catalog,
    set: &TemplateSet,
    h: &Hotel,
    p: &Period,
    history: &[DrawnSituation],
    seed: u64,
    anchors: &[Anchor],
) -> Vec<DrawnSituation> {
    let view = DrawView {
        state: h.state,
        previous: None,
        indices: &h.indices,
    };
    let draw = Draw {
        period: p,
        templates: set,
        history,
        seed,
        slot: h.slot,
        anchors,
    };
    draw_situations(&view, &draw, cat).unwrap()
}

/// Cijela sezona jednog hotela uz nepromijenjeno početno stanje; povijest raste krug po krug.
pub fn season(
    cat: &Catalog,
    set: &TemplateSet,
    h: &Hotel,
    kind: PeriodKind,
    seed: u64,
) -> Vec<Vec<DrawnSituation>> {
    let mut history: Vec<DrawnSituation> = Vec::new();
    let mut out = Vec::new();
    for p in season_periods(kind) {
        let drawn = round(cat, set, h, &p, &history, seed, &[]);
        history.extend(drawn.iter().cloned());
        out.push(drawn);
    }
    out
}

pub fn start(cat: &Catalog, profile: Profile) -> HotelState {
    HotelState::from_profile(cat, profile).unwrap()
}
