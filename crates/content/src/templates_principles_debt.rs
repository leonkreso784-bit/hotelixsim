//! Dug prema načelima igre (`templates_principles.rs`): predlošci koji danas krše tri nova načela. Popis smije
//! samo padati — novi prekršaj ruši učitavanje, a predložak koji je popravljen mora s popisa (inače učitavanje
//! kaže „makni ga"). Brisanje duga je posao Faze B uz Leonov pregled (ADR-042 t. 7, t. 10).

use crate::templates_principles::Principle;

pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])] = &[
    // Oznake svih predložaka su unutar 1,5× od B4c (B4a–B4c); dug duljina je otplaćen.
    (Principle::LabelLengths, &[]),
    // Svaki kvar ima uvjet lošeg stanja (P-BRAVE, P-CURENJE-VODE i P-DIZALO preko `condition_of: rooms`) ili je
    // vanjski događaj (P-INSPEKCIJA); dug je otplaćen u B4c.
    (Principle::BreakdownCondition, &[]),
    // Šutnja nije lošija od najlošijeg odgovora (uz trošak).
    (
        Principle::SilenceNotBetter,
        &[
            "P-KLIMA",
            "P-POVISICA",
            "P-BOJLER",
            "P-ROOFTOP-BUKA",
            "P-SPA-TECAJ",
            "P-LOBBY-SUKOB",
            "P-LOBBY-KAVA",
            "P-NOVINAR",
            "P-VAL-RECENZIJA",
            "P-KRIZA-OSOBLJA",
            "P-ALERGIJA",
            "P-OVERBOOKING",
            "P-BOLOVANJE",
            "P-PRAONICA",
            "P-PARKING-STETA",
            "P-UCJENA-RECENZIJOM",
        ],
    ),
];
