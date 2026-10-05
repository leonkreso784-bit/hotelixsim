use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HotelId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuestId(pub Uuid);

impl GameId {
    /// Jedan tok po igri: ukupan redoslijed svih događaja igre (ADR-015 t. 4).
    pub fn stream(self) -> String {
        format!("game:{}", self.0)
    }
}
