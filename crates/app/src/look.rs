//! Izgled lika primjerka (ADR-042 t. 11, kartica IZGLED): rod, dobna skupina i podrijetlo.
//! Izvor je popis imena snimka igre (`content.situation_templates.names`) i ključne riječi opisa uloge;
//! jezgra se ne dira. Ime koje nije u točno jednom popisu nema izgled (`None`, stari izgled u pregledniku).
use content::Content;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterLook {
    pub gender: LookGender,
    pub age_group: Option<AgeGroup>,
    pub origin: Origin,
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
    })
}
