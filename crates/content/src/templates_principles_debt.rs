//! Dug prema načelima igre (`templates_principles.rs`): predlošci koji danas krše tri nova načela. Popis smije
//! samo padati — novi prekršaj ruši učitavanje, a predložak koji je popravljen mora s popisa (inače učitavanje
//! kaže „makni ga"). Brisanje duga je posao Faze B uz Leonov pregled (ADR-042 t. 7, t. 10).

use crate::templates_principles::Principle;

pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])] = &[
    // Predlošci čije oznake prepisuje B4c; najmanji omjer duljina odgovora danas je 2,52 (P-INSPEKCIJA).
    (
        Principle::LabelLengths,
        &[
            "P-NOVI-LJUDI",
            "P-VJENCANJE",
            "P-ROOFTOP-BUKA",
            "P-LEZALJKE",
            "P-BAZEN-PUMPA",
            "P-SPA-TERMIN",
            "P-SPA-TECAJ",
            "P-LOBBY-SUKOB",
            "P-LOBBY-KAVA",
            "P-INSPEKCIJA",
            "P-ALERGIJA",
            "P-POSEBNA-PREHRANA",
            "P-MANJAK-INVENTURA",
            "P-DOBAVLJAC",
            "P-SUSJEDI-BAZEN",
            "P-WELLNESS-PAKET",
            "P-PIJANI-GOST",
            "P-PROSIDBA",
        ],
    ),
    // Kvar bez uvjeta lošeg stanja odjela; P-BRAVE i P-CURENJE-VODE čekaju stanje drugog odjela (Faza B).
    (
        Principle::BreakdownCondition,
        &["P-INSPEKCIJA", "P-DIZALO", "P-BRAVE", "P-CURENJE-VODE"],
    ),
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
