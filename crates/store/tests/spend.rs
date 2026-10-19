//! Pilot, dio 3: trošak AI-ja preživi restart. `impl app::spend::SpendLedger for Store` (nova tablica troška, migracija
//! sljedećeg slobodnog broja; zbrojevi po UTC danu i UTC kalendarskom mjesecu).

use app::spend::{Day, SpendEntry, SpendLedger, TokenUsage};
use sqlx::PgPool;
use store::Store;

const OCT_19_NOON: i64 = 1_792_411_200;
const OCT_19_LAST: i64 = 1_792_454_399; // 23:59:59
const OCT_20_START: i64 = 1_792_454_400;
const OCT_31_LAST: i64 = 1_793_491_199;
const NOV_1_START: i64 = 1_793_491_200;

fn entry(at_unix: i64, cost: u64) -> SpendEntry {
    SpendEntry {
        at_unix,
        model: "claude-sonnet-5-5".into(),
        tokens: TokenUsage {
            input: 900,
            output: 210,
            cache_write: 4000,
            cache_read: 12000,
        },
        cost_nano_usd: cost,
    }
}

fn day(year: i32, month: u8, day: u8) -> Day {
    Day { year, month, day }
}

async fn fill(s: &Store) {
    for (at, cost) in [
        (OCT_19_NOON, 1_000),
        (OCT_19_LAST, 2_000),
        (OCT_20_START, 4_000),
        (OCT_31_LAST, 8_000),
        (NOV_1_START, 16_000),
    ] {
        s.record(entry(at, cost)).await.unwrap();
    }
}

#[sqlx::test]
async fn empty_ledger_sums_to_zero(pool: PgPool) {
    let s = Store::from_pool(pool);
    assert_eq!(s.spent_on_day(day(2026, 10, 19)).await.unwrap(), 0);
    assert_eq!(s.spent_in_month(2026, 10).await.unwrap(), 0);
}

#[sqlx::test]
async fn sums_are_per_utc_day_and_per_utc_month(pool: PgPool) {
    let s = Store::from_pool(pool);
    fill(&s).await;
    assert_eq!(s.spent_on_day(day(2026, 10, 19)).await.unwrap(), 3_000);
    assert_eq!(s.spent_on_day(day(2026, 10, 20)).await.unwrap(), 4_000);
    assert_eq!(s.spent_on_day(day(2026, 10, 31)).await.unwrap(), 8_000);
    assert_eq!(s.spent_on_day(day(2026, 11, 1)).await.unwrap(), 16_000);
    assert_eq!(s.spent_on_day(day(2026, 10, 18)).await.unwrap(), 0);
    assert_eq!(s.spent_in_month(2026, 10).await.unwrap(), 15_000);
    assert_eq!(s.spent_in_month(2026, 11).await.unwrap(), 16_000);
    assert_eq!(s.spent_in_month(2026, 9).await.unwrap(), 0);
    assert_eq!(
        s.spent_in_month(2025, 10).await.unwrap(),
        0,
        "godina se razlikuje"
    );
}

#[sqlx::test]
async fn spend_survives_a_restart(pool: PgPool) {
    let first = Store::from_pool(pool.clone());
    fill(&first).await;
    drop(first);
    // "restart": novi Store nad istom bazom vidi isti trošak
    let second = Store::from_pool(pool);
    assert_eq!(second.spent_on_day(day(2026, 10, 19)).await.unwrap(), 3_000);
    assert_eq!(second.spent_in_month(2026, 10).await.unwrap(), 15_000);
    second.record(entry(OCT_20_START, 500)).await.unwrap();
    assert_eq!(second.spent_in_month(2026, 10).await.unwrap(), 15_500);
}

#[sqlx::test]
async fn large_costs_do_not_overflow(pool: PgPool) {
    // 30 $ = 3 * 10^10 nano; zbrojevi moraju izdržati red veličine puno veći od stropa
    let s = Store::from_pool(pool);
    for _ in 0..3 {
        s.record(entry(OCT_19_NOON, 30_000_000_000)).await.unwrap();
    }
    assert_eq!(s.spent_in_month(2026, 10).await.unwrap(), 90_000_000_000);
}
