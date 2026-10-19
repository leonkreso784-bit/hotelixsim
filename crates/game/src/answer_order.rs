//! Redoslijed ponuđenih odgovora na ekranu (L1-13b dio 1, ADR-042 t. 10): izmiješan po hotelu, stalan za isti hotel i
//! situaciju. Događaji, jezgra, roboti i AI i dalje rade s IZVORNIM brojem (položaj u sadržaju); samo prikaz i klik
//! prevode prikazni položaj u izvorni.

use engine::{Rng, label};

use crate::error::GameError;
use crate::game::Game;
use crate::ids::HotelId;

/// `order[prikazni_položaj] = izvorni_broj`; potpuna permutacija `0..n` iz (sjeme igre, slot hotela, šifra primjerka).
///
/// NE MIJENJAJ oznaku `game:answer-order` ni algoritam bez dizanja verzije: student je prikazni položaj dobio iz
/// GET-a, a klik se prevodi istim pravilom. Promjena usred kruga zapisala bi drugi odgovor od kliknutoga.
/// Ne troši tokove igre (`Rng::derive`).
pub fn answer_order(seed: u64, slot: u8, code: &str, n: usize) -> Vec<u8> {
    let mut order: Vec<u8> = (0..n).map(|i| u8::try_from(i).unwrap_or(u8::MAX)).collect();
    let mut rng = Rng::derive(
        seed,
        &[label("game:answer-order"), u64::from(slot), label(code)],
    );
    for i in (1..n).rev() {
        let bound = u64::try_from(i + 1).unwrap_or(u64::MAX);
        let j = usize::try_from(rng.range(0, bound)).unwrap_or(0);
        order.swap(i, j);
    }
    order
}

impl Game {
    /// Redoslijed ponuđenih odgovora koji vidi student ovog hotela (v1 zamrznuti demo: identitet).
    pub fn display_order(&self, hotel: HotelId, code: &str) -> Result<Vec<u8>, GameError> {
        let h = self.hotel(hotel)?;
        let (_, t) = self.situation_round_for(hotel, code)?;
        let n = t.answers.len();
        Ok(self.model().display_order(self.seed, h.setup.slot, code, n))
    }

    /// Prikazni položaj -> izvorni broj; izvan raspona je `UnknownAnswer`.
    pub fn original_answer(&self, hotel: HotelId, code: &str, shown: u8) -> Result<u8, GameError> {
        self.display_order(hotel, code)?
            .get(usize::from(shown))
            .copied()
            .ok_or(GameError::UnknownAnswer)
    }

    /// Izvorni broj -> prikazni položaj (za `chosen` i dnevnik).
    pub fn shown_position(
        &self,
        hotel: HotelId,
        code: &str,
        original: u8,
    ) -> Result<u8, GameError> {
        let pos = self
            .display_order(hotel, code)?
            .iter()
            .position(|&o| o == original)
            .and_then(|p| u8::try_from(p).ok());
        pos.ok_or(GameError::UnknownAnswer)
    }
}
