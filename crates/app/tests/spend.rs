//! Pilot, dio 3: strop troška AI-ja (ADR-062 t. 9: 5 $/dan i 30 $/mj). Izračun troška iz tokena, vrata
//! (`CappedAi`) pred unutarnjim `AiPort`-om, UTC dan i mjesec, posao koji se zbog stropa ne ponavlja.
//!
//! PRETPOSTAVKE O SUČELJU (modul `app::spend`; jedino mjesto, graditelj ih smije uskladiti uz obrazloženje):
//! - `TokenUsage { input, output, cache_write, cache_read }` (sva polja `u32`, `Default`, `Clone`, `Debug`, `PartialEq`, `Eq`).
//! - `ApiUsage { model: String, tokens: TokenUsage }` (ono što `ai` javlja za SVAKI odgovor Anthropica).
//! - Cijene su PODATAK: `PRICES: &[ModelPrice]` (`ModelPrice { model_prefix: &str, price: Price, .. }`), a
//!   `cost_nano_usd(model, &TokenUsage) -> u64` računa u nano-dolarima (1 $ = `NANO_PER_USD` = 10^9), cijeli brojevi bez zaokruživanja.
//! - `Day { year: i32, month: u8, day: u8 }` (UTC) i `Day::from_unix(secs: i64) -> Day`.
//! - `trait Clock: Send + Sync { fn now_unix(&self) -> i64 }`.
//! - `SpendEntry { at_unix: i64, model: String, tokens: TokenUsage, cost_nano_usd: u64 }`; `LedgerError(pub String)`.
//! - `trait SpendLedger: Send + Sync`: `record(&self, SpendEntry) -> BoxFut<'_, Result<(), LedgerError>>`,
//!   `spent_on_day(&self, Day) -> BoxFut<'_, Result<u64, LedgerError>>`, `spent_in_month(&self, year: i32, month: u8) -> ...`.
//! - `trait UsageSink: Send + Sync { fn record(&self, ApiUsage) -> BoxFut<'_, ()> }`;
//!   `Metered::new(Arc<dyn SpendLedger>, Arc<dyn Clock>)` je `UsageSink` (cijena iz tablice, vrijeme iz sata).
//! - `Caps { day_nano_usd: Option<u64>, month_nano_usd: Option<u64> }`, `Caps::from_usd(Option<f64>, Option<f64>) -> Caps`.
//! - `CappedAi::new(inner: Arc<dyn AiPort>, ledger: Arc<dyn SpendLedger>, clock: Arc<dyn Clock>, caps: Caps)` je `AiPort`:
//!   prije svakog poziva, ako je potrošnja današnjeg UTC dana >= dnevnog stropa ili tekućeg UTC mjeseca >= mjesečnog,
//!   vrati `AiError::Disabled` bez poziva unutarnjeg porta (strop dosegnut = zatvoreno; `None` = bez ograničenja).
//!
//! CIJENE (izvor: https://platform.claude.com/docs/en/about-claude/pricing, pročitano 19.10.2026., USD po milijunu tokena;
//! 1 $/MTok = 1000 nano-$ po tokenu). Predmemorija: zapis 5 min (zahtjevi koriste `ephemeral`), čitanje.
//!   claude-sonnet-5-5: ulaz 2, upis 2,50, čitanje 0,10, izlaz 10.
//!   claude-haiku-5-5 (upit do 100 000 tokena): 0,10 / 0,125 / 0,01 / 0,50; (upit preko 100 000): 0,50 / 0,625 / 0,05 / 2,50.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use app::memory::FakeAi;
use app::spend::{
    ApiUsage, CappedAi, Caps, Clock, Day, LedgerError, Metered, NANO_PER_USD, PRICES, SpendEntry,
    SpendLedger, TokenUsage, UsageSink, cost_nano_usd,
};
use app::{AiError, AiOut, AiPort, BoxFut, Caller, Deps, GameId, HotelId, SituationItem};
use content::{Model, Snapshot, repo_dir};
use game::{CharacterCard, CharacterInput, ChatLine, Speaker};
use sim_hotel_v2::PeriodKind;

const SONNET: &str = "claude-sonnet-5-5";
const HAIKU: &str = "claude-haiku-5-5";
const USD: u64 = NANO_PER_USD;

// UTC trenuci
const OCT_19_NOON: i64 = 1_792_411_200;
const OCT_19_LAST: i64 = 1_792_454_399; // 23:59:59
const OCT_20_START: i64 = 1_792_454_400;
const OCT_31_LAST: i64 = 1_793_491_199;
const NOV_1_START: i64 = 1_793_491_200;
const DEC_31_LAST: i64 = 1_798_761_599;
const JAN_1_2027: i64 = 1_798_761_600;
const FEB_29_2028_LAST: i64 = 1_835_481_599;
const MAR_1_2028: i64 = 1_835_481_600;

fn usage(input: u32, output: u32, cache_write: u32, cache_read: u32) -> TokenUsage {
    TokenUsage {
        input,
        output,
        cache_write,
        cache_read,
    }
}

// ---------- 1. izračun troška ----------

#[test]
fn sonnet_cost_is_tokens_times_published_prices_in_nano_usd() {
    // 2 $/MTok = 2000 nano po tokenu; izlaz 10 $ = 10 000; upis 2,50 $ = 2500; čitanje 0,10 $ = 100.
    assert_eq!(cost_nano_usd(SONNET, &usage(1_000, 0, 0, 0)), 2_000_000);
    assert_eq!(cost_nano_usd(SONNET, &usage(0, 500, 0, 0)), 5_000_000);
    assert_eq!(cost_nano_usd(SONNET, &usage(0, 0, 10_000, 0)), 25_000_000);
    assert_eq!(cost_nano_usd(SONNET, &usage(0, 0, 0, 20_000)), 2_000_000);
    assert_eq!(
        cost_nano_usd(SONNET, &usage(1_000, 500, 10_000, 20_000)),
        34_000_000
    );
    assert_eq!(cost_nano_usd(SONNET, &TokenUsage::default()), 0);
    // milijun izlaznih tokena = točno 10 $
    assert_eq!(cost_nano_usd(SONNET, &usage(0, 1_000_000, 0, 0)), 10 * USD);
}

#[test]
fn haiku_cost_uses_its_own_cheaper_prices() {
    // 0,10 / 0,125 / 0,01 / 0,50 $ po MTok = 100 / 125 / 10 / 500 nano po tokenu (upit do 100 000 tokena)
    assert_eq!(
        cost_nano_usd(HAIKU, &usage(1_000, 500, 10_000, 20_000)),
        100_000 + 250_000 + 1_250_000 + 200_000
    );
    assert!(
        cost_nano_usd(HAIKU, &usage(1_000, 500, 10_000, 20_000))
            < cost_nano_usd(SONNET, &usage(1_000, 500, 10_000, 20_000))
    );
}

#[test]
fn haiku_prompts_over_100k_tokens_use_the_long_context_prices() {
    // do 100 000 (uključivo) kratka cijena: 100 nano po ulaznom tokenu
    assert_eq!(
        cost_nano_usd(HAIKU, &usage(100_000, 0, 0, 0)),
        100_000 * 100
    );
    // preko 100 000: 0,50 $ ulaz (500 nano), 2,50 $ izlaz (2500 nano)
    assert_eq!(
        cost_nano_usd(HAIKU, &usage(100_001, 0, 0, 0)),
        100_001 * 500
    );
    assert_eq!(
        cost_nano_usd(HAIKU, &usage(150_000, 1_000, 0, 0)),
        150_000 * 500 + 1_000 * 2_500
    );
    // veličinu upita čini zbroj ulaza i predmemorije (sigurnija strana): 60 000 + 50 000 čitanja = 110 000
    // -> dugi cjenik: ulaz 500, čitanje 0,05 $ = 50 nano
    assert_eq!(
        cost_nano_usd(HAIKU, &usage(60_000, 0, 0, 50_000)),
        60_000 * 500 + 50_000 * 50
    );
}

#[test]
fn unknown_model_is_priced_at_the_most_expensive_known_price() {
    let u = usage(1_000, 500, 10_000, 20_000);
    let sonnet = cost_nano_usd(SONNET, &u);
    for unknown in ["claude-opus-5-5", "", "nesto-drugo", "gpt-x"] {
        assert_eq!(cost_nano_usd(unknown, &u), sonnet, "{unknown:?}");
    }
    // i po komponentama: nepoznat model nikad nije jeftiniji od ijednog poznatog u tablici
    assert!(
        PRICES.len() >= 2,
        "tablica cijena je podatak: Sonnet i Haiku"
    );
    for (label, only) in [
        ("ulaz", usage(1_000_000, 0, 0, 0)),
        ("izlaz", usage(0, 1_000_000, 0, 0)),
        ("upis", usage(0, 0, 1_000_000, 0)),
        ("čitanje", usage(0, 0, 0, 1_000_000)),
    ] {
        let unknown = cost_nano_usd("model-kojeg-nema", &only);
        for p in PRICES {
            assert!(
                unknown >= cost_nano_usd(p.model_prefix, &only),
                "{label}: nepoznat model jeftiniji od {}",
                p.model_prefix
            );
        }
    }
}

// ---------- UTC dan ----------

#[test]
fn day_is_the_utc_calendar_day() {
    let d = |year, month, day| Day { year, month, day };
    assert_eq!(Day::from_unix(0), d(1970, 1, 1));
    assert_eq!(Day::from_unix(OCT_19_NOON), d(2026, 10, 19));
    assert_eq!(Day::from_unix(OCT_19_LAST), d(2026, 10, 19));
    assert_eq!(Day::from_unix(OCT_20_START), d(2026, 10, 20));
    assert_eq!(Day::from_unix(OCT_31_LAST), d(2026, 10, 31));
    assert_eq!(Day::from_unix(NOV_1_START), d(2026, 11, 1));
    assert_eq!(Day::from_unix(DEC_31_LAST), d(2026, 12, 31));
    assert_eq!(Day::from_unix(JAN_1_2027), d(2027, 1, 1));
    assert_eq!(Day::from_unix(FEB_29_2028_LAST), d(2028, 2, 29));
    assert_eq!(Day::from_unix(MAR_1_2028), d(2028, 3, 1));
}

#[test]
fn caps_from_usd_converts_dollars_to_exact_nano() {
    let c = Caps::from_usd(Some(5.0), Some(30.0));
    assert_eq!(c.day_nano_usd, Some(5 * USD));
    assert_eq!(c.month_nano_usd, Some(30 * USD));
    let c = Caps::from_usd(Some(0.01), Some(0.5));
    assert_eq!(c.day_nano_usd, Some(10_000_000));
    assert_eq!(c.month_nano_usd, Some(500_000_000));
    let c = Caps::from_usd(None, None);
    assert_eq!((c.day_nano_usd, c.month_nano_usd), (None, None));
}

// ---------- pomagala ----------

struct TestClock(AtomicI64);

impl TestClock {
    fn at(secs: i64) -> Arc<TestClock> {
        Arc::new(TestClock(AtomicI64::new(secs)))
    }
    fn set(&self, secs: i64) {
        self.0.store(secs, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now_unix(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Knjiga troška u memoriji: zbrojevi po UTC danu i mjesecu iz zapisa.
#[derive(Default)]
struct TestLedger(Mutex<Vec<SpendEntry>>);

impl TestLedger {
    fn entries(&self) -> Vec<SpendEntry> {
        self.0.lock().unwrap().clone()
    }
}

impl SpendLedger for TestLedger {
    fn record(&self, entry: SpendEntry) -> BoxFut<'_, Result<(), LedgerError>> {
        self.0.lock().unwrap().push(entry);
        Box::pin(async { Ok(()) })
    }
    fn spent_on_day(&self, day: Day) -> BoxFut<'_, Result<u64, LedgerError>> {
        let sum = self
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|e| Day::from_unix(e.at_unix) == day)
            .map(|e| e.cost_nano_usd)
            .sum();
        Box::pin(async move { Ok(sum) })
    }
    fn spent_in_month(&self, year: i32, month: u8) -> BoxFut<'_, Result<u64, LedgerError>> {
        let sum = self
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|e| {
                let d = Day::from_unix(e.at_unix);
                (d.year, d.month) == (year, month)
            })
            .map(|e| e.cost_nano_usd)
            .sum();
        Box::pin(async move { Ok(sum) })
    }
}

/// Unutarnji AI koji se ponaša kao `ai::Anthropic` s brojačem: svaki poziv potroši 100 000 izlaznih tokena Sonneta
/// (točno 1 $) i javi ih brojaču, pa vrati uspjeh.
struct OneDollarAi {
    calls: AtomicU32,
    sink: Arc<dyn UsageSink>,
}

impl OneDollarAi {
    async fn spend_and_answer(&self) -> Result<AiOut<String>, AiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.sink
            .record(ApiUsage {
                model: SONNET.into(),
                tokens: usage(0, 100_000, 0, 0),
            })
            .await;
        Ok(AiOut {
            value: "Hvala.".into(),
            meta: game::AiMeta {
                model: SONNET.into(),
                prompt: "lik.v3".into(),
                input_tokens: 0,
                output_tokens: 100_000,
            },
        })
    }
}

impl AiPort for OneDollarAi {
    fn assess(
        &self,
        _input: game::AssessInput,
    ) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, AiError>> {
        Box::pin(async {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(AiError::Unavailable("nije potrebno".into()))
        })
    }
    fn character(&self, _input: CharacterInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(self.spend_and_answer())
    }
    fn debrief(&self, _input: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(self.spend_and_answer())
    }
    fn season_debrief(
        &self,
        _input: game::SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        Box::pin(self.spend_and_answer())
    }
}

struct Rig {
    capped: CappedAi,
    inner: Arc<OneDollarAi>,
    ledger: Arc<TestLedger>,
    clock: Arc<TestClock>,
}

fn rig(day_usd: Option<f64>, month_usd: Option<f64>, start: i64) -> Rig {
    let ledger = Arc::new(TestLedger::default());
    let clock = TestClock::at(start);
    let sink: Arc<dyn UsageSink> = Arc::new(Metered::new(ledger.clone(), clock.clone()));
    let inner = Arc::new(OneDollarAi {
        calls: AtomicU32::new(0),
        sink,
    });
    let capped = CappedAi::new(
        inner.clone(),
        ledger.clone(),
        clock.clone(),
        Caps::from_usd(day_usd, month_usd),
    );
    Rig {
        capped,
        inner,
        ledger,
        clock,
    }
}

fn sample_character() -> CharacterInput {
    CharacterInput {
        situation: "S1".into(),
        title: "Gost se žali na buku".into(),
        character: CharacterCard {
            name: "Gospođa Horvat".into(),
            role: "gost".into(),
            brief: "Ljuta zbog buke.".into(),
            opening: "Cijelu noć nisam oka sklopila!".into(),
        },
        transcript: vec![ChatLine {
            from: Speaker::Student,
            text: "Dobar dan.".into(),
        }],
        message_no: 1,
        message_max: 6,
    }
}

impl Rig {
    /// Jedan poziv uloge lika; `true` ako je prošao do unutarnjeg porta.
    async fn call(&self) -> bool {
        match self.capped.character(sample_character()).await {
            Ok(_) => true,
            Err(AiError::Disabled) => false,
            Err(e) => panic!("neočekivana greška: {e}"),
        }
    }
    fn inner_calls(&self) -> u32 {
        self.inner.calls.load(Ordering::SeqCst)
    }
}

// ---------- brojač (Metered) ----------

#[tokio::test]
async fn metered_records_cost_from_the_price_table_stamped_with_the_clock() {
    let ledger = Arc::new(TestLedger::default());
    let clock = TestClock::at(OCT_19_NOON);
    let m = Metered::new(ledger.clone(), clock.clone());
    let tokens = usage(1_000, 500, 10_000, 20_000);
    m.record(ApiUsage {
        model: HAIKU.into(),
        tokens: tokens.clone(),
    })
    .await;
    clock.set(NOV_1_START);
    m.record(ApiUsage {
        model: "claude-opus-5-5".into(),
        tokens: tokens.clone(),
    })
    .await;
    let e = ledger.entries();
    assert_eq!(e.len(), 2);
    assert_eq!(e[0].at_unix, OCT_19_NOON);
    assert_eq!(e[0].model, HAIKU);
    assert_eq!(e[0].tokens, tokens);
    assert_eq!(e[0].cost_nano_usd, 1_800_000);
    assert_eq!(e[1].at_unix, NOV_1_START);
    assert_eq!(
        e[1].cost_nano_usd, 34_000_000,
        "nepoznat model = najskuplji"
    );
}

// ---------- 2. vrata (CappedAi) ----------

#[tokio::test]
async fn below_the_cap_calls_pass_through_to_the_inner_port() {
    let r = rig(Some(5.0), Some(30.0), OCT_19_NOON);
    for n in 1..=5 {
        assert!(r.call().await, "poziv {n} je ispod stropa");
    }
    assert_eq!(r.inner_calls(), 5);
    assert_eq!(r.ledger.entries().len(), 5);
}

#[tokio::test]
async fn reaching_the_daily_cap_returns_disabled_without_calling_inner() {
    let r = rig(Some(5.0), Some(30.0), OCT_19_NOON);
    for _ in 0..5 {
        assert!(r.call().await);
    }
    // potrošeno točno 5 $ = strop dosegnut
    assert!(!r.call().await, "dnevni strop dosegnut");
    assert!(!r.call().await);
    assert_eq!(r.inner_calls(), 5, "unutarnji port se više ne poziva");
    assert_eq!(r.ledger.entries().len(), 5, "odbijeni poziv ne troši");
}

#[tokio::test]
async fn all_gated_roles_return_disabled_over_the_cap() {
    let r = rig(Some(1.0), None, OCT_19_NOON);
    assert!(r.call().await);
    let a = r.capped.assess(sample_assess()).await;
    assert_eq!(a.err(), Some(AiError::Disabled));
    assert_eq!(r.inner_calls(), 1);
}

#[tokio::test]
async fn a_new_utc_day_reopens_the_daily_cap() {
    let r = rig(Some(5.0), Some(30.0), OCT_19_LAST);
    for _ in 0..5 {
        assert!(r.call().await);
    }
    assert!(!r.call().await, "23:59:59: isti UTC dan, zatvoreno");
    r.clock.set(OCT_20_START);
    assert!(
        r.call().await,
        "00:00:00 sljedećeg UTC dana: dnevni otvoren"
    );
    assert_eq!(r.inner_calls(), 6);
}

#[tokio::test]
async fn the_monthly_cap_closes_even_when_the_day_is_fresh() {
    let r = rig(Some(5.0), Some(8.0), OCT_19_NOON);
    for _ in 0..5 {
        assert!(r.call().await);
    }
    r.clock.set(OCT_20_START);
    for n in 1..=3 {
        assert!(r.call().await, "drugi dan, poziv {n} (mjesec 5..8 $)");
    }
    // dan: 3 $ od 5 $; mjesec: 8 $ od 8 $ -> mjesečni strop dosegnut
    assert!(!r.call().await, "mjesečni strop");
    assert_eq!(r.inner_calls(), 8);
    // novi UTC dan istog mjeseca NE otvara mjesečni
    r.clock.set(OCT_31_LAST);
    assert!(!r.call().await, "31.10. i dalje listopad");
    assert_eq!(r.inner_calls(), 8);
}

#[tokio::test]
async fn a_new_utc_month_reopens_the_monthly_cap() {
    let r = rig(Some(5.0), Some(8.0), OCT_19_NOON);
    for _ in 0..5 {
        assert!(r.call().await);
    }
    r.clock.set(OCT_20_START);
    for _ in 0..3 {
        assert!(r.call().await);
    }
    r.clock.set(OCT_31_LAST);
    assert!(!r.call().await);
    r.clock.set(NOV_1_START);
    assert!(r.call().await, "1.11. 00:00:00 UTC: novi mjesec i novi dan");
    assert_eq!(r.inner_calls(), 9);
}

#[tokio::test]
async fn without_caps_nothing_is_limited_and_a_single_cap_still_applies() {
    let free = rig(None, None, OCT_19_NOON);
    for _ in 0..12 {
        assert!(free.call().await);
    }
    assert_eq!(free.inner_calls(), 12);

    let only_month = rig(None, Some(3.0), OCT_19_NOON);
    for _ in 0..3 {
        assert!(only_month.call().await);
    }
    assert!(!only_month.call().await, "samo mjesečni, bez dnevnog");

    let only_day = rig(Some(2.0), None, OCT_19_NOON);
    for _ in 0..2 {
        assert!(only_day.call().await);
    }
    assert!(!only_day.call().await, "samo dnevni, bez mjesečnog");
}

fn sample_assess() -> game::AssessInput {
    game::AssessInput {
        situation: "S1".into(),
        title: "Gost se žali na buku".into(),
        rubric: (1..=5).map(|n| format!("kriterij {n}")).collect(),
        character: None,
        offered: vec![],
        transcript: vec![],
        response: "Ispričavam se.".into(),
    }
}

// ---------- 5. Disabled zbog stropa: rezerva i bez ponavljanja ----------

/// Broji pozive uloge lika prema "vanjskom" portu (pred vratima stropa).
struct Probe {
    inner: Arc<dyn AiPort>,
    character_calls: AtomicU32,
}

impl AiPort for Probe {
    fn assess(
        &self,
        input: game::AssessInput,
    ) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, AiError>> {
        self.inner.assess(input)
    }
    fn character(&self, input: CharacterInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        self.character_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.character(input)
    }
    fn debrief(&self, input: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        self.inner.debrief(input)
    }
    fn season_debrief(
        &self,
        input: game::SeasonDebriefInput,
    ) -> BoxFut<'_, Result<AiOut<String>, AiError>> {
        self.inner.season_debrief(input)
    }
}

async fn instance_with_character(deps: &Deps, caller: &Caller, hotel: HotelId) -> SituationItem {
    let s = app::situations(deps, caller, hotel).await.unwrap();
    s.items
        .into_iter()
        .find(|i| i.code.contains('.') && i.free_text && i.character.is_some())
        .expect("prvi krug sjemena 42 ima primjerak s likom")
}

#[tokio::test]
async fn chat_job_over_the_cap_writes_fallback_reply_and_is_not_retried() {
    // knjiga već pokazuje 6 $ potrošeno danas -> dnevni strop od 5 $ dosegnut
    let ledger = Arc::new(TestLedger::default());
    ledger
        .record(SpendEntry {
            at_unix: OCT_19_NOON,
            model: SONNET.into(),
            tokens: usage(0, 600_000, 0, 0),
            cost_nano_usd: 6 * USD,
        })
        .await
        .unwrap();
    let clock = TestClock::at(OCT_19_NOON);
    let fake = Arc::new(FakeAi::good());
    let capped = CappedAi::new(
        fake.clone(),
        ledger.clone(),
        clock,
        Caps::from_usd(Some(5.0), Some(30.0)),
    );
    let probe = Arc::new(Probe {
        inner: Arc::new(capped),
        character_calls: AtomicU32::new(0),
    });

    let (deps, _mem) =
        app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), probe.clone());
    let settings = app::NewGameSettings {
        model: Model::V2,
        kind: Some(PeriodKind::Month),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    let (hotel, _game): (HotelId, GameId) = (d.hotel_id, d.game_id);
    let item = instance_with_character(&deps, &caller, hotel).await;
    let v = app::send_message(
        &deps,
        &caller,
        hotel,
        &item.code,
        "Razumijem, odmah se javljam.".into(),
    )
    .await
    .unwrap();
    assert!(v.awaiting_reply);
    let mut drained = false;
    for _ in 0..60 {
        if let Ok(false) = app::run_next_job(&deps).await {
            drained = true;
            break;
        }
    }
    assert!(drained, "red se nije ispraznio: posao se ponavlja");

    let v = app::conversation(&deps, &caller, hotel, &item.code)
        .await
        .unwrap();
    assert!(!v.awaiting_reply, "razgovor ostao zaključan");
    assert_eq!(v.lines.len(), 2);
    let text = &v.lines[1].text;
    assert!(!text.trim().is_empty() && !text.contains('{'), "{text}");
    assert_eq!(
        probe.character_calls.load(Ordering::SeqCst),
        1,
        "Disabled zbog stropa: točno jedan pokušaj, bez ponavljanja"
    );
    assert_eq!(fake.calls(), (0, 0, 0), "unutarnji AI nije pozvan");
}
