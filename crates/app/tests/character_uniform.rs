//! IZGLED-c (kartica `task-13c-izgled-uniforme.md`, §7 testovi 1–6; Leonove odluke 18.10.): uniforma lika iz KLJUČA uloge
//! predloška (`Template.character.role`), ne iz opisa. Pogled samo čita snimak sadržaja igre; jezgra se ne dira.
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - `app::{Uniform, UNIFORM_ROLES, uniform_for_role, character_uniform, character_role_key, look_of_character}`
//!   (modul `app::look`), `CharacterLook.uniform: Option<Uniform>`, `Uniform::{Police, Fire, Ems, Doctor}`;
//! - `character_uniform(&Content, template_code: &str) -> Option<Uniform>`;
//! - `character_role_key(&Game, HotelId, code) -> Option<&str>`;
//! - `look_of_character(&Game, HotelId, code, &content::Character) -> Option<CharacterLook>`.

use std::sync::Arc;

use app::memory::FakeAi;
use app::{
    Caller, CharacterLook, Deps, GameId, HotelId, UNIFORM_ROLES, Uniform, character_look,
    character_role_key, character_uniform, look_of_character, uniform_for_role,
};
use content::templates::RoleSource;
use content::{Content, Model, Snapshot, repo_dir};
use game::{Actor, Game, GameEvent, Recorded};
use sim_hotel_v2::PeriodKind;

fn v2_content() -> Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

fn v1_content() -> Content {
    Snapshot::read(&repo_dir()).unwrap().build().unwrap()
}

/// 1. Rječnik uloga: točan ključ, bez podniza i velikih slova.
#[test]
fn uniform_for_role_knows_four_bank_roles() {
    assert_eq!(uniform_for_role("policajac"), Some(Uniform::Police));
    assert_eq!(uniform_for_role("vatrogasac"), Some(Uniform::Fire));
    assert_eq!(uniform_for_role("djelatnik_hitne"), Some(Uniform::Ems));
    assert_eq!(uniform_for_role("lijecnik"), Some(Uniform::Doctor));
    for key in [
        "inspektor",
        "inspektor_sanitarni",
        "gost",
        "",
        "Policajac",
        "policajac ",
        "policajka",
        "djelatnik",
        "hitna",
    ] {
        assert_eq!(uniform_for_role(key), None, "{key:?}");
    }
    assert_eq!(UNIFORM_ROLES.len(), 4);
    for (key, u) in UNIFORM_ROLES {
        assert_eq!(uniform_for_role(key), Some(u), "{key}");
    }
}

/// 2. Uniforma se čita iz ključa uloge lika u predlošku snimka (klon sadržaja s preimenovanim ključem).
#[test]
fn character_uniform_reads_template_role_key() {
    let original = v2_content();
    let templates = &original.situation_templates.as_ref().unwrap().templates;
    let t = templates
        .iter()
        .find(|t| {
            t.character.as_ref().is_some_and(|c| {
                t.roles
                    .get(&c.role)
                    .is_some_and(|r| r.from == RoleSource::Partner)
            })
        })
        .expect("predložak s likom partnera");
    let code = t.code.clone();
    let old_key = t.character.as_ref().unwrap().role.clone();
    assert_eq!(character_uniform(&original, &code), None);

    let rename = |c: &mut Content, from: &str, to: &str| {
        let st = c.situation_templates.as_mut().unwrap();
        let t = st.templates.iter_mut().find(|t| t.code == code).unwrap();
        let role = t.roles.remove(from).unwrap();
        t.roles.insert(to.to_owned(), role);
        t.character.as_mut().unwrap().role = to.to_owned();
    };
    let mut changed = original.clone();
    rename(&mut changed, &old_key, "policajac");
    assert_eq!(character_uniform(&changed, &code), Some(Uniform::Police));
    // Ključ sa sličnim imenom nije uniforma.
    rename(&mut changed, "policajac", "policajac_u_civilu");
    assert_eq!(character_uniform(&changed, &code), None);

    assert_eq!(character_uniform(&original, "P-NE-POSTOJI"), None);
    assert_eq!(character_uniform(&original, ""), None);
    assert!(v1_content().situation_templates.is_none());
    assert_eq!(character_uniform(&v1_content(), &code), None);
}

/// 3. Stanje prije cigle sadržaja C10: nijedan predložak danas nema ulogu s uniformom (ta cigla ovaj test mijenja
/// svjesno, u svom RED commitu).
#[test]
fn no_template_has_a_uniform_today() {
    let c = v2_content();
    let templates = &c.situation_templates.as_ref().unwrap().templates;
    assert!(!templates.is_empty());
    for t in templates {
        assert_eq!(character_uniform(&c, &t.code), None, "{}", t.code);
    }
}

/// 6. Čuvar budućeg sadržaja: uloga koja opisom (naziv ili vrsta) govori o službi mora imati ključ iz rječnika i biti
/// partner — inače lik policajca ostaje tiho bez uniforme. Fraze, ne korijeni ("hitno odgovoriti" nije služba).
#[test]
fn service_roles_use_bank_keys() {
    let c = v2_content();
    let keys: Vec<&str> = UNIFORM_ROLES.iter().map(|(k, _)| *k).collect();
    let phrases = [
        "polic",
        "vatrogas",
        "hitna pomoć",
        "hitne medicin",
        "liječni",
    ];
    let mut found = 0;
    for t in &c.situation_templates.as_ref().unwrap().templates {
        for (key, role) in &t.roles {
            let text = role
                .title
                .iter()
                .chain(role.types.iter())
                .map(|s| s.to_lowercase())
                .collect::<Vec<_>>()
                .join(" | ");
            if phrases.iter().any(|p| text.contains(p)) {
                found += 1;
                assert!(
                    keys.contains(&key.as_str()),
                    "{}: uloga službe `{key}` ({text}) nema ključ iz UNIFORM_ROLES",
                    t.code
                );
                assert_eq!(role.from, RoleSource::Partner, "{}: {key}", t.code);
            }
        }
    }
    // Danas nema takvih uloga; kad ih cigla sadržaja doda, ovaj test ih počinje čuvati (i ova brojka se mijenja svjesno).
    assert_eq!(found, 0, "očekivano 0 uloga službe prije C10");
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

/// Igra složena iz dnevnika (kao `app` interno), da se vidi što pogled vidi.
async fn folded(deps: &Deps, id: GameId) -> Game {
    let records: Vec<Recorded> = deps
        .log
        .load(id)
        .await
        .unwrap()
        .iter()
        .map(|r| Recorded {
            seq: r.seq,
            actor: Actor::parse(&r.actor).unwrap(),
            event: GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone()).unwrap(),
        })
        .collect();
    Game::fold(&records).unwrap()
}

/// 4. Ključ uloge lika primjerka hotela = `template.character.role` (kao `situation_card`); sidra i nepoznata šifra nemaju.
#[tokio::test]
async fn character_role_key_matches_situation_card() {
    let (deps, _caller, hotel, id) = start_v2().await;
    let g = folded(&deps, id).await;
    let templates = &g.content.situation_templates.as_ref().unwrap().templates;
    let mut with_character = 0;
    let mut anchors = 0;
    for r in &g.rounds {
        let record = g.hotel(hotel).unwrap();
        for t in g.situations_of_hotel(hotel, *r).unwrap() {
            let key = character_role_key(&g, hotel, &t.code);
            let inst = record
                .drawn
                .get(r)
                .and_then(|l| l.iter().find(|i| i.situation.code == t.code));
            if let Some(i) = inst {
                let want = templates
                    .iter()
                    .find(|p| p.code == i.drawn.template)
                    .and_then(|p| p.character.as_ref())
                    .map(|c| c.role.as_str());
                assert_eq!(key, want, "{}", t.code);
                with_character += usize::from(want.is_some());
            } else {
                anchors += 1;
                assert_eq!(key, None, "sidro {}", t.code);
            }
        }
    }
    assert!(with_character > 0, "sjeme 42: primjerak s likom");
    assert!(anchors > 0, "sjeme 42: sidro S1/S2");
    assert_eq!(character_role_key(&g, hotel, "S1"), None);
    assert_eq!(character_role_key(&g, hotel, "NE.POSTOJI"), None);
}

/// 5. Pogledi nose polje `uniform` (danas `None` za sve); rod, dob i podrijetlo kao prije; `look_of_character`
/// bez uniforme daje isto što i `character_look`.
#[tokio::test]
async fn views_carry_uniform_field() {
    let (deps, caller, hotel, id) = start_v2().await;
    let g = folded(&deps, id).await;
    let items = app::situations(&deps, &caller, hotel).await.unwrap().items;
    let mut seen = 0;
    for i in &items {
        let Some(ch) = &i.character else { continue };
        let Some(look) = ch.look else { continue };
        seen += 1;
        assert_eq!(look.uniform, None, "{}", i.code);
        let base = character_look(&g.content, &ch.name, &ch.role).expect("izgled iz imena");
        assert_eq!(
            CharacterLook {
                uniform: None,
                ..look
            },
            base,
            "{}: rod, dob i podrijetlo kao prije",
            i.code
        );
        let c = g
            .ai_entry_for(hotel, &i.code)
            .and_then(|a| a.character.as_ref())
            .expect("lik primjerka");
        assert_eq!(
            look_of_character(&g, hotel, &i.code, c),
            Some(look),
            "{}",
            i.code
        );
        if i.free_text {
            let v = app::conversation(&deps, &caller, hotel, &i.code)
                .await
                .unwrap();
            assert_eq!(v.character_look, Some(look), "{}", i.code);
        }
    }
    assert!(seen > 0, "prvi krug sjemena 42 ima primjerak s likom");
    // Ime izvan popisa imena: bez izgleda i bez uniforme.
    let mut stranger = items
        .iter()
        .find_map(|i| {
            g.ai_entry_for(hotel, &i.code)
                .and_then(|a| a.character.clone())
        })
        .expect("barem jedan lik");
    "Nepoznata Osoba".clone_into(&mut stranger.name);
    assert_eq!(look_of_character(&g, hotel, "S1", &stranger), None);
}
