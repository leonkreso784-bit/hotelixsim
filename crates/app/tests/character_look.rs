//! IZGLED-a (ADR-042 t. 11): lik primjerka dobiva rod, dobnu skupinu i podrijetlo iz popisa imena snimka igre
//! (`content.situation_templates.names`) i ključnih riječi opisa uloge. Kartica `task-13-izgled-lika.md`.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `app::{LookGender, AgeGroup, Origin, CharacterLook, character_look, age_hint}` (modul `app::look`);
//! - `CharacterCardView.look` i `ConversationView.character_look` su `Option<CharacterLook>`;
//! - v2 mjesečna igra sjemena 42 u prvom krugu ima primjerak s likom (kao `instance_ai.rs`).

use std::collections::BTreeMap;
use std::sync::Arc;

use app::memory::FakeAi;
use app::{
    AgeGroup, Caller, CharacterLook, Deps, GameId, HotelId, LookGender, Origin, SituationItem,
    age_hint, character_look,
};
use content::templates::Names;
use content::{Content, Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

fn v2_content() -> Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

fn v1_content() -> Content {
    Snapshot::read(&repo_dir()).unwrap().build().unwrap()
}

fn names(c: &Content) -> &Names {
    &c.situation_templates
        .as_ref()
        .expect("v2 snimak nosi predloške")
        .names
}

/// Svako ime s popisom (izvor, rod).
fn all_names(n: &Names) -> Vec<(&'static str, &'static str, &String)> {
    let mut out = Vec::new();
    for (src, list) in [
        ("guest", &n.guest),
        ("staff", &n.staff),
        ("partner", &n.partner),
    ] {
        for (g, v) in [("f", &list.f), ("m", &list.m)] {
            out.extend(v.iter().map(|x| (src, g, x)));
        }
    }
    out
}

/// 1. Čuva pretpostavku šava: ime je u točno jednom popisu (izvor x rod).
#[test]
fn names_are_unique_across_lists() {
    let c = v2_content();
    let mut seen: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (src, g, name) in all_names(names(&c)) {
        seen.entry(name.as_str())
            .or_default()
            .push(format!("{src}/{g}"));
    }
    assert!(!seen.is_empty(), "popis imena je prazan");
    let dup: Vec<_> = seen.iter().filter(|(_, v)| v.len() != 1).collect();
    assert!(dup.is_empty(), "ime u više popisa: {dup:?}");
}

/// 2. Rod i izvor iz popisa imena; osoblje i partneri su odrasli.
#[test]
fn look_from_name_lists() {
    let c = v2_content();
    let look = |n: &str| character_look(&c, n, "gošća hotela");
    assert_eq!(
        look("Anna Müller"),
        Some(CharacterLook {
            gender: LookGender::F,
            age_group: None,
            origin: Origin::Foreign
        })
    );
    assert_eq!(
        look("Ivan Kovač"),
        Some(CharacterLook {
            gender: LookGender::M,
            age_group: None,
            origin: Origin::Foreign
        })
    );
    assert_eq!(
        look("Ana Jurić"),
        Some(CharacterLook {
            gender: LookGender::F,
            age_group: Some(AgeGroup::Adult),
            origin: Origin::Regional
        })
    );
    assert_eq!(
        look("Damir Tomić"),
        Some(CharacterLook {
            gender: LookGender::M,
            age_group: Some(AgeGroup::Adult),
            origin: Origin::Regional
        })
    );
}

/// 3. Ime koje nije u popisu (sidro), prazno ime i v1 sadržaj (bez predložaka) nemaju izgled.
#[test]
fn unknown_name_has_no_look() {
    let c = v2_content();
    assert_eq!(character_look(&c, "Petra Horvat", "gošća hotela"), None);
    assert_eq!(character_look(&c, "", "gošća hotela"), None);
    assert!(v1_content().situation_templates.is_none());
    assert_eq!(
        character_look(&v1_content(), "Anna Müller", "gošća hotela"),
        None
    );
}

/// 4. Ključne riječi u opisu uloge; cijele riječi, ne podnizovi.
#[test]
fn age_hint_reads_role_words() {
    let cases = [
        ("umirovljenica na godišnjem odmoru", Some(AgeGroup::Senior)),
        (
            "baka koja je s unucima na ljetovanju",
            Some(AgeGroup::Senior),
        ),
        (
            "stariji gost koji teško hoda do restorana",
            Some(AgeGroup::Senior),
        ),
        ("mladić koji je došao s prijateljima", Some(AgeGroup::Young)),
        ("gost koji putuje s ostarjelom majkom", None),
        ("gošća koja putuje sama sa starim psom", None),
        ("gošća hotela", None),
    ];
    for (role, want) in cases {
        assert_eq!(age_hint(role), want, "{role}");
    }
}

async fn start_v2() -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(
        Snapshot::read(&repo_dir()).unwrap(),
        Arc::new(FakeAi::good()),
    );
    let settings = app::NewGameSettings {
        model: Model::V2,
        kind: Some(PeriodKind::Month),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    (deps, caller, d.hotel_id, d.game_id)
}

async fn items(deps: &Deps, caller: &Caller, hotel: HotelId) -> Vec<SituationItem> {
    app::situations(deps, caller, hotel).await.unwrap().items
}

/// Rod iz popisa imena, neovisno o `character_look`.
fn gender_in_lists(c: &Content, name: &str) -> Option<LookGender> {
    let hits: Vec<_> = all_names(names(c))
        .into_iter()
        .filter(|(_, _, n)| n.as_str() == name)
        .collect();
    match hits.as_slice() {
        [(_, "f", _)] => Some(LookGender::F),
        [(_, "m", _)] => Some(LookGender::M),
        _ => None,
    }
}

/// 5. Pogled situacija nosi izgled za likove primjeraka; sidra ga nemaju.
#[tokio::test]
async fn situations_view_carries_look() {
    let (deps, caller, hotel, _) = start_v2().await;
    let c = v2_content();
    let list = items(&deps, &caller, hotel).await;
    let mut with_char = 0;
    for i in &list {
        let Some(ch) = &i.character else { continue };
        if i.code.contains('.') {
            with_char += 1;
            let look = ch
                .look
                .unwrap_or_else(|| panic!("{}: lik bez izgleda", i.code));
            assert_eq!(
                Some(look.gender),
                gender_in_lists(&c, &ch.name),
                "{}",
                i.code
            );
        } else {
            assert_eq!(ch.look, None, "sidro {} nema izgled", i.code);
        }
    }
    assert!(with_char > 0, "prvi krug sjemena 42 ima primjerak s likom");
    assert!(
        list.iter().any(|i| i.code == "S1" || i.code == "S2"),
        "očekivano sidro S1/S2 u prvom krugu"
    );
}

/// 6. Razgovor daje isti izgled kao pogled situacija.
#[tokio::test]
async fn conversation_view_carries_look() {
    let (deps, caller, hotel, _) = start_v2().await;
    let inst = items(&deps, &caller, hotel)
        .await
        .into_iter()
        .find(|i| i.code.contains('.') && i.free_text && i.character.is_some())
        .expect("primjerak s likom");
    let want = inst.character.as_ref().unwrap().look;
    assert!(want.is_some(), "pogled situacija nosi izgled");
    let v = app::conversation(&deps, &caller, hotel, &inst.code)
        .await
        .unwrap();
    assert_eq!(v.character_look, want);
}
