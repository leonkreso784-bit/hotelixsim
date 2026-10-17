//! Izgled lika primjerka (ADR-042 t. 11, kartica IZGLED): rod, dobna skupina i podrijetlo.
//! Izvor je popis imena snimka igre (`content.situation_templates.names`) i ključne riječi opisa uloge;
//! jezgra se ne dira. Ime koje nije u točno jednom popisu nema izgled (`None`, stari izgled u pregledniku).
//! Uniforma (IZGLED-c) dolazi iz KLJUČA uloge lika u predlošku (`Template.character.role`), ne iz opisa.
use content::Content;
use game::{Game, HotelId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookGender {
    F,
    M,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgeGroup {
    Young,
    Adult,
    Senior,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Regional,
    Foreign,
}

/// Odora službe vanjskog lika (kartica IZGLED-c §4): mijenja samo odjeću, kapu i ovratnik, nikad lice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uniform {
    Police,
    Fire,
    Ems,
    Doctor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterLook {
    pub gender: LookGender,
    pub age_group: Option<AgeGroup>,
    pub origin: Origin,
    /// `None`: lik bez odore (nepoznata uloga, sidra, v1).
    pub uniform: Option<Uniform>,
}

/// Rječnik uloga banke (BANKA_LJUDI §1.5); isti ključ u predlošku. Samo točan ključ, bez podniza i velikih slova.
pub const UNIFORM_ROLES: [(&str, Uniform); 4] = [
    ("policajac", Uniform::Police),
    ("vatrogasac", Uniform::Fire),
    ("djelatnik_hitne", Uniform::Ems),
    ("lijecnik", Uniform::Doctor),
];

/// Uniforma za ključ uloge (točna usporedba).
pub fn uniform_for_role(key: &str) -> Option<Uniform> {
    UNIFORM_ROLES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, u)| *u)
}

/// Uniforma lika predloška `template` iz snimka sadržaja: ključ `character.role` → `uniform_for_role`. `None`: v1,
/// nepoznat predložak, predložak bez lika, uloga bez uniforme.
pub fn character_uniform(content: &Content, template: &str) -> Option<Uniform> {
    let t = content
        .situation_templates
        .as_ref()?
        .templates
        .iter()
        .find(|t| t.code == template)?;
    uniform_for_role(&t.character.as_ref()?.role)
}

/// Ključ uloge lika primjerka hotela (kao `situation_card`): primjerak tog hotela, njegov predložak, `character.role`.
/// `None` za sidra S1–S4, v1 i nepoznatu šifru.
pub fn character_role_key<'a>(g: &'a Game, hotel: HotelId, code: &str) -> Option<&'a str> {
    let i = g
        .hotel(hotel)
        .ok()?
        .drawn
        .values()
        .flatten()
        .find(|i| i.situation.code == code)?;
    g.content
        .situation_templates
        .as_ref()?
        .templates
        .iter()
        .find(|t| t.code == i.drawn.template)?
        .character
        .as_ref()
        .map(|c| c.role.as_str())
}

/// `character_look` s uniformom iz `character_role_key`; ime izvan popisa imena → `None` (i bez uniforme).
pub fn look_of_character(
    g: &Game,
    hotel: HotelId,
    code: &str,
    c: &content::Character,
) -> Option<CharacterLook> {
    let look = character_look(&g.content, &c.name, &c.role)?;
    Some(CharacterLook {
        uniform: character_role_key(g, hotel, code).and_then(uniform_for_role),
        ..look
    })
}

/// Dobna skupina iz ključnih riječi opisa uloge (cijele riječi, mala slova); starija ima prednost pred mlađom.
pub fn age_hint(role: &str) -> Option<AgeGroup> {
    let lower = role.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .collect();
    let any = |f: &dyn Fn(&str) -> bool| words.iter().any(|w| f(w));
    if any(&|w| {
        w.starts_with("umirovljen")
            || matches!(
                w,
                "baka" | "djed" | "stariji" | "starija" | "starica" | "starac"
            )
    }) {
        Some(AgeGroup::Senior)
    } else if any(&|w| {
        w.starts_with("mladić") || w.starts_with("student") || matches!(w, "mlada" | "mladi")
    }) {
        Some(AgeGroup::Young)
    } else {
        None
    }
}

/// Izgled lika po imenu: ime mora biti u točno jednom popisu (izvor x rod). Gost je `Foreign` i bez dobi
/// (web bira iz imena), osoblje i partner `Regional` i odrasli; ključna riječ u opisu uloge nadjačava dob.
pub fn character_look(content: &Content, name: &str, role: &str) -> Option<CharacterLook> {
    let names = &content.situation_templates.as_ref()?.names;
    let mut hits = Vec::new();
    for (guest, list) in [
        (true, &names.guest),
        (false, &names.staff),
        (false, &names.partner),
    ] {
        for (gender, v) in [(LookGender::F, &list.f), (LookGender::M, &list.m)] {
            hits.extend(
                v.iter()
                    .filter(|n| n.as_str() == name)
                    .map(|_| (guest, gender)),
            );
        }
    }
    let [(guest, gender)] = hits.as_slice() else {
        return None;
    };
    let fallback = if *guest { None } else { Some(AgeGroup::Adult) };
    Some(CharacterLook {
        gender: *gender,
        age_group: age_hint(role).or(fallback),
        origin: if *guest {
            Origin::Foreign
        } else {
            Origin::Regional
        },
        uniform: None,
    })
}
