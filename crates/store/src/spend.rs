//! Knjiga troška AI-ja u Postgresu (pilot, dio 3): `ai_spend`, zbrojevi po UTC danu i mjesecu.

use app::BoxFut;
use app::spend::{Day, LedgerError, SpendEntry, SpendLedger, month_range_unix};

use crate::Store;

fn ledger_err(e: sqlx::Error) -> LedgerError {
    LedgerError(e.to_string())
}

impl Store {
    async fn sum_between(&self, from: i64, to: i64) -> Result<u64, LedgerError> {
        let sum = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(cost_nano_usd), 0)::BIGINT AS "sum!" FROM ai_spend WHERE at_unix >= $1 AND at_unix < $2"#,
            from,
            to
        )
        .fetch_one(&self.pool)
        .await
        .map_err(ledger_err)?;
        u64::try_from(sum).map_err(|e| LedgerError(e.to_string()))
    }
}

impl SpendLedger for Store {
    fn record(&self, entry: SpendEntry) -> BoxFut<'_, Result<(), LedgerError>> {
        Box::pin(async move {
            let cost = i64::try_from(entry.cost_nano_usd).unwrap_or(i64::MAX);
            sqlx::query!(
                "INSERT INTO ai_spend (at_unix, model, input_tokens, output_tokens, cache_write, cache_read, cost_nano_usd)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                entry.at_unix,
                entry.model,
                i64::from(entry.tokens.input),
                i64::from(entry.tokens.output),
                i64::from(entry.tokens.cache_write),
                i64::from(entry.tokens.cache_read),
                cost
            )
            .execute(&self.pool)
            .await
            .map_err(ledger_err)?;
            Ok(())
        })
    }

    fn spent_on_day(&self, day: Day) -> BoxFut<'_, Result<u64, LedgerError>> {
        Box::pin(async move { self.sum_between(day.start_unix(), day.end_unix()).await })
    }

    fn spent_in_month(&self, year: i32, month: u8) -> BoxFut<'_, Result<u64, LedgerError>> {
        Box::pin(async move {
            let (from, to) = month_range_unix(year, month);
            self.sum_between(from, to).await
        })
    }
}
