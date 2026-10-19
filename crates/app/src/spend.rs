//! Strop troška AI-ja (pilot, dio 3; ADR-062 t. 9): cijene kao podatak, trošak u nano-dolarima, knjiga troška,
//! brojač (`Metered`) i vrata (`CappedAi`) pred `AiPort`-om. Dan i mjesec su UTC.
//! Strop dosegnut ili knjiga neispravna = `AiError::Disabled` (rezerve, igra radi), nikad pad.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::{AiError, AiOut, AiPort, BoxFut};

/// 1 $ u nano-dolarima.
pub const NANO_PER_USD: u64 = 1_000_000_000;

/// Tokeni jednog odgovora Anthropica, svi brojači koje naplaćuje.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
    pub cache_write: u32,
    pub cache_read: u32,
}

/// Potrošnja jednog odgovora s imenom modela (iz tijela odgovora, inače iz zahtjeva).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiUsage {
    pub model: String,
    pub tokens: TokenUsage,
}

/// Cijena u nano-$ po tokenu (1 $ po milijunu tokena = 1000).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Price {
    pub input: u64,
    pub cache_write: u64,
    pub cache_read: u64,
    pub output: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ModelPrice {
    pub model_prefix: &'static str,
    pub price: Price,
    /// Cjenik za upite dulje od `long.0` tokena (ulaz + upis + čitanje), ako ga model ima.
    pub long: Option<(u32, Price)>,
}

const fn price(input: u64, cache_write: u64, cache_read: u64, output: u64) -> Price {
    Price {
        input,
        cache_write,
        cache_read,
        output,
    }
}

const fn model(model_prefix: &'static str, price: Price) -> ModelPrice {
    ModelPrice {
        model_prefix,
        price,
        long: None,
    }
}

/// Cijene iz https://platform.claude.com/docs/en/about-claude/pricing (pročitano 19.10.2026., sve trenutne modele
/// osim ukinutih: Opus 4/4.1, Sonnet 4, Haiku 3.5); upis predmemorije = 5 min. Model se traži po NAJDULJEM prefiksu
/// (`claude-sonnet-5` ne guta `claude-sonnet-5-5`; sufiks s datumom je dopušten). Redoslijed redaka nije bitan.
/// Za model izvan tablice vrijedi po komponentama najskuplji redak (`dearest`).
/// Prag dugog Haiku cjenika: prompt = ulaz + upis + čitanje > 100 000 (izvor: "counts all of its input tokens,
/// including cache reads and cache writes").
pub const PRICES: &[ModelPrice] = &[
    model("claude-fable-5-1", price(10_000, 12_500, 250, 50_000)),
    model("claude-fable-5", price(10_000, 12_500, 1_000, 50_000)),
    model("claude-mythos-5-1", price(10_000, 12_500, 250, 50_000)),
    model("claude-mythos-5", price(10_000, 12_500, 1_000, 50_000)),
    model("claude-opus-5-5", price(4_000, 5_000, 200, 20_000)),
    model("claude-opus-5", price(5_000, 6_250, 500, 25_000)),
    model("claude-opus-4-8", price(5_000, 6_250, 500, 25_000)),
    model("claude-opus-4-7", price(5_000, 6_250, 500, 25_000)),
    model("claude-opus-4-6", price(5_000, 6_250, 500, 25_000)),
    model("claude-opus-4-5", price(5_000, 6_250, 500, 25_000)),
    model("claude-sonnet-5-5", price(2_000, 2_500, 100, 10_000)),
    model("claude-sonnet-5", price(2_000, 2_500, 200, 10_000)),
    model("claude-sonnet-4-6", price(3_000, 3_750, 300, 15_000)),
    model("claude-sonnet-4-5", price(3_000, 3_750, 300, 15_000)),
    ModelPrice {
        model_prefix: "claude-haiku-5-5",
        price: price(100, 125, 10, 500),
        long: Some((100_000, price(500, 625, 50, 2_500))),
    },
    model("claude-haiku-4-5", price(1_000, 1_250, 100, 5_000)),
];

/// Po komponentama najskuplja cijena iz CIJELE tablice: za nepoznati model nikad ne podcijenimo.
fn dearest() -> Price {
    let mut all = PRICES
        .iter()
        .flat_map(|p| std::iter::once(p.price).chain(p.long.map(|(_, l)| l)));
    let first = all.next().expect("tablica cijena nije prazna");
    all.fold(first, |a, b| Price {
        input: a.input.max(b.input),
        cache_write: a.cache_write.max(b.cache_write),
        cache_read: a.cache_read.max(b.cache_read),
        output: a.output.max(b.output),
    })
}

fn find_price(model: &str) -> Option<&'static ModelPrice> {
    PRICES
        .iter()
        .filter(|p| model.starts_with(p.model_prefix))
        .max_by_key(|p| p.model_prefix.len())
}

/// Je li model u tablici cijena (poslužitelj u produkciji odbija start s nepoznatim modelom).
pub fn is_priced(model: &str) -> bool {
    find_price(model).is_some()
}

fn price_of(model: &str, t: &TokenUsage) -> Price {
    match find_price(model) {
        Some(p) => {
            let prompt = u64::from(t.input) + u64::from(t.cache_write) + u64::from(t.cache_read);
            match p.long {
                Some((over, long)) if prompt > u64::from(over) => long,
                _ => p.price,
            }
        }
        None => dearest(),
    }
}

/// Trošak odgovora u nano-$ (cijeli brojevi, bez zaokruživanja).
pub fn cost_nano_usd(model: &str, t: &TokenUsage) -> u64 {
    let p = price_of(model, t);
    u64::from(t.input) * p.input
        + u64::from(t.cache_write) * p.cache_write
        + u64::from(t.cache_read) * p.cache_read
        + u64::from(t.output) * p.output
}

/// Kalendarski dan po UTC-u.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Day {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

/// Dani od 1970-01-01 za građanski datum (Hinnant, `days_from_civil`).
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

impl Day {
    pub fn from_unix(secs: i64) -> Day {
        let z = secs.div_euclid(86_400) + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        Day {
            year: i32::try_from(year).unwrap_or(i32::MAX),
            month: u8::try_from(month).unwrap_or(1),
            day: u8::try_from(day).unwrap_or(1),
        }
    }

    /// Početak dana (00:00:00 UTC) kao unix sekunde.
    pub fn start_unix(&self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 86_400
    }

    /// Početak sljedećeg UTC dana.
    pub fn end_unix(&self) -> i64 {
        self.start_unix() + 86_400
    }
}

/// `[početak, kraj)` UTC kalendarskog mjeseca u unix sekundama; nevaljan mjesec daje prazan raspon.
pub fn month_range_unix(year: i32, month: u8) -> (i64, i64) {
    if !(1..=12).contains(&month) {
        return (0, 0);
    }
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    (
        days_from_civil(year, month, 1) * 86_400,
        days_from_civil(ny, nm, 1) * 86_400,
    )
}

pub trait Clock: Send + Sync {
    fn now_unix(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendEntry {
    pub at_unix: i64,
    pub model: String,
    pub tokens: TokenUsage,
    pub cost_nano_usd: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("knjiga troška: {0}")]
pub struct LedgerError(pub String);

/// Knjiga troška AI-ja: upis jednog odgovora i zbrojevi po UTC danu i UTC mjesecu (nano-$).
pub trait SpendLedger: Send + Sync {
    fn record(&self, entry: SpendEntry) -> BoxFut<'_, Result<(), LedgerError>>;
    fn spent_on_day(&self, day: Day) -> BoxFut<'_, Result<u64, LedgerError>>;
    fn spent_in_month(&self, year: i32, month: u8) -> BoxFut<'_, Result<u64, LedgerError>>;
}

/// Prima potrošnju svakog odgovora Anthropica (i odrezanog) čim stigne, prije tumačenja odgovora.
pub trait UsageSink: Send + Sync {
    fn record(&self, usage: ApiUsage) -> BoxFut<'_, ()>;
}

/// Brojač: cijena iz tablice, vrijeme iz sata, upis u knjigu. Neuspjeh upisa se samo zapiše u log.
pub struct Metered {
    ledger: Arc<dyn SpendLedger>,
    clock: Arc<dyn Clock>,
}

impl Metered {
    pub fn new(ledger: Arc<dyn SpendLedger>, clock: Arc<dyn Clock>) -> Metered {
        Metered { ledger, clock }
    }
}

impl UsageSink for Metered {
    fn record(&self, usage: ApiUsage) -> BoxFut<'_, ()> {
        Box::pin(async move {
            let entry = SpendEntry {
                at_unix: self.clock.now_unix(),
                cost_nano_usd: cost_nano_usd(&usage.model, &usage.tokens),
                model: usage.model,
                tokens: usage.tokens,
            };
            if let Err(e) = self.ledger.record(entry).await {
                tracing::warn!(error = %e, "trošak AI-ja nije upisan u knjigu");
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Caps {
    pub day_nano_usd: Option<u64>,
    pub month_nano_usd: Option<u64>,
}

impl Caps {
    pub fn from_usd(day: Option<f64>, month: Option<f64>) -> Caps {
        // `as` zasićuje: negativno i NaN postaju 0, preveliko u64::MAX
        let nano = |usd: f64| (usd * NANO_PER_USD as f64).round() as u64;
        Caps {
            day_nano_usd: day.map(nano),
            month_nano_usd: month.map(nano),
        }
    }

    pub fn any(&self) -> bool {
        self.day_nano_usd.is_some() || self.month_nano_usd.is_some()
    }
}

/// Vrata pred `AiPort`-om: potrošnja današnjeg UTC dana >= dnevnog ili tekućeg UTC mjeseca >= mjesečnog stropa
/// (ili knjiga koja se ne može pročitati: fail-closed 🟨) = `Disabled`, unutarnji port se ne poziva.
pub struct CappedAi {
    inner: Arc<dyn AiPort>,
    ledger: Arc<dyn SpendLedger>,
    clock: Arc<dyn Clock>,
    caps: Caps,
    warned_day: AtomicI64,
    warned_month: AtomicI64,
    warned_ledger: AtomicI64,
}

/// Najviše jedno upozorenje o nečitljivoj knjizi u toliko sekundi (inače poplava pri svakoj poruci).
const LEDGER_WARN_EVERY: i64 = 60;

impl CappedAi {
    pub fn new(
        inner: Arc<dyn AiPort>,
        ledger: Arc<dyn SpendLedger>,
        clock: Arc<dyn Clock>,
        caps: Caps,
    ) -> CappedAi {
        CappedAi {
            inner,
            ledger,
            clock,
            caps,
            warned_day: AtomicI64::new(i64::MIN),
            warned_month: AtomicI64::new(i64::MIN),
            warned_ledger: AtomicI64::new(i64::MIN),
        }
    }

    fn ledger_down(&self, e: &LedgerError, now: i64) -> AiError {
        let last = self.warned_ledger.load(Ordering::SeqCst);
        if last == i64::MIN || now.saturating_sub(last) >= LEDGER_WARN_EVERY {
            self.warned_ledger.store(now, Ordering::SeqCst);
            tracing::warn!(error = %e, "knjiga troška AI-ja nije čitljiva: AI je isključen dok se ne popravi (strop se ne može provjeriti)");
        }
        AiError::Disabled
    }

    async fn gate(&self) -> Result<(), AiError> {
        let now = self.clock.now_unix();
        let today = Day::from_unix(now);
        if let Some(cap) = self.caps.day_nano_usd {
            let spent = self
                .ledger
                .spent_on_day(today)
                .await
                .map_err(|e| self.ledger_down(&e, now))?;
            if spent >= cap {
                let key = today.start_unix();
                if self.warned_day.swap(key, Ordering::SeqCst) != key {
                    tracing::warn!(
                        potroseno_usd = usd(spent),
                        strop_usd = usd(cap),
                        "dnevni strop troška AI-ja dosegnut: AI je do ponoći (UTC) isključen, igra radi na rezervama"
                    );
                }
                return Err(AiError::Disabled);
            }
        }
        if let Some(cap) = self.caps.month_nano_usd {
            let spent = self
                .ledger
                .spent_in_month(today.year, today.month)
                .await
                .map_err(|e| self.ledger_down(&e, now))?;
            if spent >= cap {
                let key = i64::from(today.year) * 12 + i64::from(today.month);
                if self.warned_month.swap(key, Ordering::SeqCst) != key {
                    tracing::warn!(
                        potroseno_usd = usd(spent),
                        strop_usd = usd(cap),
                        "mjesečni strop troška AI-ja dosegnut: AI je do kraja mjeseca (UTC) isključen, igra radi na rezervama"
                    );
                }
                return Err(AiError::Disabled);
            }
        }
        Ok(())
    }
}

fn usd(nano: u64) -> f64 {
    nano as f64 / NANO_PER_USD as f64
}

impl AiPort for CappedAi {
    fn assess(
        &self,
        input: game::AssessInput,
    ) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, AiError>> {
        Box::pin(async move {
            self.gate().await?;
            self.inner.assess(input).await
        })
    }
    fn character(&self, input: game::CharacterInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            self.gate().await?;
            self.inner.character(input).await
        })
    }
    fn debrief(&self, input: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            self.gate().await?;
            self.inner.debrief(input).await
        })
    }
    fn season_debrief(
        &self,
        input: game::SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(async move {
            self.gate().await?;
            self.inner.season_debrief(input).await
        })
    }
}
