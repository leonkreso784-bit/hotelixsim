//! Dug prema načelima igre (`templates_principles.rs`): predlošci koji danas krše jedno od tri nova načela. Popis smije
//! samo padati — novi prekršaj ruši učitavanje, a predložak koji je popravljen mora s popisa (inače učitavanje
//! kaže „makni ga"). Nakon Faze B (B4a–B4c) duljina oznaka i uvjet kvara nemaju duga (0 i 0); ostaje samo
//! „šutnja nije bolja" (16 predložaka), koji se otplaćuje u L1-13 uz kalibraciju mehanike (ADR-042 t. 7, t. 10).

use crate::templates_principles::Principle;

pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])] = &[
    // Oznake svih predložaka su unutar 1,5× od B4c (B4a–B4c); dug duljina je otplaćen.
    (Principle::LabelLengths, &[]),
    // Svaki kvar ima uvjet lošeg stanja (P-BRAVE, P-CURENJE-VODE i P-DIZALO preko `condition_of: rooms`) ili je
    // vanjski događaj (P-INSPEKCIJA); dug je otplaćen u B4c.
    (Principle::BreakdownCondition, &[]),
    // Šutnja nije lošija od najlošijeg odgovora (uz trošak); 16 predložaka, vlasnik L1-13 (mehanika, ne oznake).
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
