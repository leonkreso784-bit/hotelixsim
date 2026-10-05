use app::{BoxFut, GameId, GuestId, Guests, HotelId, StoreError};

use crate::Store;
use crate::log::store_err;

impl Guests for Store {
    fn create_guest(&self, id: GuestId, token_hash: String) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query!(
                "INSERT INTO guests (id, token_hash) VALUES ($1, $2)",
                id.0,
                token_hash
            )
            .execute(&self.pool)
            .await
            .map_err(store_err)?;
            Ok(())
        })
    }

    fn guest_by_token_hash(
        &self,
        token_hash: String,
    ) -> BoxFut<'_, Result<Option<GuestId>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query!("SELECT id FROM guests WHERE token_hash = $1", token_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(store_err)?;
            Ok(row.map(|r| GuestId(r.id)))
        })
    }

    fn game_of_hotel(&self, hotel: HotelId) -> BoxFut<'_, Result<Option<GameId>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query!("SELECT game_id FROM hotels WHERE id = $1", hotel.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(store_err)?;
            Ok(row.map(|r| GameId(r.game_id)))
        })
    }

    fn latest_hotel_of(
        &self,
        guest: GuestId,
    ) -> BoxFut<'_, Result<Option<(GameId, HotelId)>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query!(
                "SELECT h.game_id, h.id FROM hotels h JOIN games g ON g.id = h.game_id \
                 WHERE h.guest_id = $1 ORDER BY g.seq DESC LIMIT 1",
                guest.0
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(store_err)?;
            Ok(row.map(|r| (GameId(r.game_id), HotelId(r.id))))
        })
    }
}
