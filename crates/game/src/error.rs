use sim_hotel::PlanError;

/// Greške pravila igre. `Corrupt` znači da dnevnik ili snimak krše pravilo koje je provjereno pri upisu — to je
/// greška sustava (500), nikad korisnika.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    RoundNotOpen,
    AlreadyAnswered,
    /// Situacija nema razgovor ni slobodni odgovor (`free_text` je isključen).
    NotFreeText,
    /// Svih šest studentovih poruka je iskorišteno.
    ChatLimit,
    /// Lik još nije odgovorio na zadnju poruku.
    AwaitingReply,
    EmptyText,
    TextTooLong,
    Plan(PlanError),
    Corrupt(String),
    /// Igra je stvorena drugom verzijom jezgre; obračun bi dao druge brojke (BACKLOG M3).
    CoreMismatch(String),
    /// v2 igra stvorena drugom verzijom jezgre v2: ne čita se (🟨 P11). v1 igra ovo nikad ne dobije.
    Outdated {
        created: String,
        current: String,
    },
    /// Metoda ili prikaz postoji samo za drugi model igre.
    WrongModel,
    /// Postavke nove igre nisu dopuštene (npr. v1 po tjednima).
    InvalidSettings,
    /// Greška plana v2 (v1 ostaje `Plan`).
    PlanV2(sim_hotel_v2::PlanError),
}

pub(crate) fn corrupt(msg: impl Into<String>) -> GameError {
    GameError::Corrupt(msg.into())
}
