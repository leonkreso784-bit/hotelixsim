//! Pilot, dio 3 (graditelj, 🟨): greška knjige troška = fail-closed (`Disabled` + upozorenje u logu) i upozorenje
//! u logu kad se dnevni ili mjesečni strop dosegne (jednom po danu, odnosno mjesecu).

use std::io::Write;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use app::memory::FakeAi;
use app::spend::{CappedAi, Caps, Clock, Day, LedgerError, SpendEntry, SpendLedger};
use app::{AiError, AiPort, BoxFut};
use game::{CharacterCard, CharacterInput, ChatLine, Speaker};

const OCT_19_NOON: i64 = 1_792_411_200;
const OCT_20_NOON: i64 = OCT_19_NOON + 86_400;

struct Clk(AtomicI64);
impl Clock for Clk {
    fn now_unix(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Knjiga s fiksnim zbrojevima ili s greškom.
struct Fixed {
    day: Result<u64, ()>,
    month: Result<u64, ()>,
    reads: AtomicU32,
}

impl Fixed {
    fn new(day: Result<u64, ()>, month: Result<u64, ()>) -> Arc<Fixed> {
        Arc::new(Fixed {
            day,
            month,
            reads: AtomicU32::new(0),
        })
    }
}

impl SpendLedger for Fixed {
    fn record(&self, _entry: SpendEntry) -> BoxFut<'_, Result<(), LedgerError>> {
        Box::pin(async { Ok(()) })
    }
    fn spent_on_day(&self, _day: Day) -> BoxFut<'_, Result<u64, LedgerError>> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let r = self.day.map_err(|()| LedgerError("baza pala".into()));
        Box::pin(async move { r })
    }
    fn spent_in_month(&self, _y: i32, _m: u8) -> BoxFut<'_, Result<u64, LedgerError>> {
        let r = self.month.map_err(|()| LedgerError("baza pala".into()));
        Box::pin(async move { r })
    }
}

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Buf {
    type Writer = Buf;
    fn make_writer(&'a self) -> Buf {
        self.clone()
    }
}

fn capture() -> (Buf, tracing::subscriber::DefaultGuard) {
    let buf = Buf::default();
    let sub = tracing_subscriber::fmt()
        .with_writer(buf.clone())
        .with_ansi(false)
        .finish();
    (buf, tracing::subscriber::set_default(sub))
}

fn log_of(buf: &Buf) -> String {
    String::from_utf8(buf.0.lock().unwrap().clone()).unwrap()
}

fn input() -> CharacterInput {
    CharacterInput {
        situation: "S1".into(),
        title: "Gost".into(),
        character: CharacterCard {
            name: "Horvat".into(),
            role: "gost".into(),
            brief: "Ljuta.".into(),
            opening: "Dobar dan.".into(),
        },
        transcript: vec![ChatLine {
            from: Speaker::Student,
            text: "Dobar dan.".into(),
        }],
        message_no: 1,
        message_max: 6,
    }
}

fn capped(ledger: Arc<Fixed>, at: i64, fake: Arc<FakeAi>) -> CappedAi {
    capped_with_clock(ledger, Arc::new(Clk(AtomicI64::new(at))), fake)
}

fn capped_with_clock(ledger: Arc<Fixed>, clock: Arc<Clk>, fake: Arc<FakeAi>) -> CappedAi {
    CappedAi::new(fake, ledger, clock, Caps::from_usd(Some(5.0), Some(30.0)))
}

#[tokio::test]
async fn unreadable_ledger_fails_closed_with_a_warning() {
    for (day, month) in [(Err(()), Ok(0)), (Ok(0), Err(()))] {
        let (buf, _g) = capture();
        let fake = Arc::new(FakeAi::good());
        let ai = capped(Fixed::new(day, month), OCT_19_NOON, fake.clone());
        let r = ai.character(input()).await;
        assert_eq!(r.err(), Some(AiError::Disabled), "{day:?} {month:?}");
        assert_eq!(fake.calls(), (0, 0, 0), "unutarnji AI se ne zove");
        let log = log_of(&buf);
        assert!(log.contains("WARN") && log.contains("baza pala"), "{log}");
    }
}

#[tokio::test]
async fn reaching_a_cap_warns_once_per_day_and_once_per_month() {
    let (buf, _g) = capture();
    let fake = Arc::new(FakeAi::good());
    let day_full = Fixed::new(Ok(5_000_000_000), Ok(5_000_000_000));
    let ai = capped(day_full, OCT_19_NOON, fake.clone());
    for _ in 0..3 {
        assert_eq!(ai.character(input()).await.err(), Some(AiError::Disabled));
    }
    let log = log_of(&buf);
    assert_eq!(log.matches("dnevni strop").count(), 1, "{log}");

    let (buf, _g) = capture();
    let month_full = Fixed::new(Ok(0), Ok(30_000_000_000));
    let ai = capped(month_full, OCT_19_NOON, fake.clone());
    for _ in 0..3 {
        assert_eq!(ai.character(input()).await.err(), Some(AiError::Disabled));
    }
    let log = log_of(&buf);
    assert_eq!(log.matches("mjesečni strop").count(), 1, "{log}");
    assert_eq!(log.matches("dnevni strop").count(), 0, "{log}");
    assert_eq!(fake.calls(), (0, 0, 0));
}

#[tokio::test]
async fn warning_repeats_on_the_next_day() {
    let (buf, _g) = capture();
    let fake = Arc::new(FakeAi::good());
    let ledger = Fixed::new(Ok(6_000_000_000), Ok(6_000_000_000));
    let clock = Arc::new(Clk(AtomicI64::new(OCT_19_NOON)));
    let ai = capped_with_clock(ledger, clock.clone(), fake);
    let _ = ai.character(input()).await;
    let _ = ai.character(input()).await;
    clock.0.store(OCT_20_NOON, Ordering::SeqCst);
    let _ = ai.character(input()).await;
    assert_eq!(log_of(&buf).matches("dnevni strop").count(), 2);
}
