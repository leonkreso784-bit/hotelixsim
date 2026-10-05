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
    Plan(PlanError),
    Corrupt(String),
}

pub(crate) fn corrupt(msg: impl Into<String>) -> GameError {
    GameError::Corrupt(msg.into())
}
