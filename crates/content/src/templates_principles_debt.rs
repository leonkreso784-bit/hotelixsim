//! Dug prema načelima igre (`templates_principles.rs`): predlošci koji danas krše jedno od tri nova načela. Popis smije
//! samo padati — novi prekršaj ruši učitavanje, a predložak koji je popravljen mora s popisa (inače učitavanje
//! kaže „makni ga"). Nakon Faze B (B4a–B4c) duljina oznaka i uvjet kvara nemaju duga (0 i 0), a „šutnja nije bolja"
//! (16 predložaka) otplaćen je u L1-13 dijelu 4 (ADR-042 t. 7, t. 10): sva tri popisa su prazna.

use crate::templates_principles::Principle;

pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])] = &[
    // Oznake svih predložaka su unutar 1,5× od B4c (B4a–B4c); dug duljina je otplaćen.
    (Principle::LabelLengths, &[]),
    // Svaki kvar ima uvjet lošeg stanja (P-BRAVE, P-CURENJE-VODE i P-DIZALO preko `condition_of: rooms`) ili je
    // vanjski događaj (P-INSPEKCIJA); dug je otplaćen u B4c.
    (Principle::BreakdownCondition, &[]),
    // Šutnja nije lošija od najlošijeg odgovora: dug (16 predložaka) otplaćen u L1-13 dijelu 4 (A8: gora šutnja,
    // `unanswered` gubi više bodova od najlošijeg odgovora).
    (Principle::SilenceNotBetter, &[]),
];
