### Task 5: Ključ kruga u igri (v1, ponašanje isto)

**Veličina:** L, 5–7 h (širok, mehanički) · **Ovisi o:** Task 1 · **Paralelno s:** Task 2, 3, 4 · **Model:** sonnet

**Cilj:** `game`, `app` i `api` više ne koriste `Month` kao ključ kruga nego `RoundKey` (`Month(m)` ili `Week(n)`),
a JSON svih v1 događaja i poslova ostaje bajt-isti. Tjedni još ne postoje u igri (to je Task 6) — ovo je čisto
preimenovanje tipa ključa.

**Files:**
- Create: `crates/game/src/round.rs`, `crates/game/tests/round_key.rs`
- Modify: `crates/game/src/{lib,game,event,command,resolve,view,feed,ai_input,talk,verdict,mentor,kpi}.rs`,
  `crates/game/tests/**` (mehanički), `crates/app/src/{views,commands,jobs,talk}.rs`, `crates/app/tests/**`,
  `crates/api/src/{schema,games,hotels,explanation,kpi}.rs`, `crates/api/tests/**`
- Ne dirati: `crates/sim-hotel/**`, `crates/content/**`, `crates/game/tests/fixtures/**`, `crates/game/src/dto.rs`
  (`MonthCode` ostaje kao **kalendarski** mjesec u `MonthReportDto.month` i `GroupBlockDto.month`), `openapi.json`
  (ne smije se promijeniti — provjera `corepack pnpm check:api`).

**Interfaces (Produces):**

```rust
// crates/game/src/round.rs
/// Ključ kruga (MODEL_V2 §4.4): `Month(m)` → "jun"…; `Week(n)`, n = 1..=17 → "t01"…"t17".
/// Serde je kod (niz), isti kao stari `MonthCode` za mjesece — v1 događaji su bajt-isti.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoundKey { Month(Month), Week(u8) }
pub const MAX_WEEK: u8 = 17;
impl RoundKey {
    pub fn code(self) -> String;
    pub fn parse(s: &str) -> Option<RoundKey>;  // "jun".."sep", "t01".."t17"; ništa drugo
    pub fn month(self) -> Option<Month>;
}
impl From<Month> for RoundKey;
impl Serialize for RoundKey; impl<'de> Deserialize<'de> for RoundKey;
```

Promjene tipova (sve s `RoundKey`, imena JSON polja ostaju `month`/`months`):
`GameCreated.months: Vec<RoundKey>`; `PlanSet/AnswerChosen/RoundClosed/RoundResolved/ChatMessageSent/CharacterReplied/
ResponseSubmitted/VerdictRecorded/DebriefRecorded.month: RoundKey`; `Game.months` → `Game.rounds: Vec<RoundKey>`;
`closed/resolved: BTreeMap<RoundKey, …>`; `HotelRecord.{plans, debriefs}` po `RoundKey`, `answers/chats/responses` po
`(RoundKey, String)`; `current_month()` → `current_round()`; metode koje su primale `Month` (`status`, `state_at`,
`effective_plan`, `set_plan`, `close_round`, `resolve_round`, `resolve_job`, `robot_moves`, `situations_in`,
`unanswered_count`, `record_debrief`, `debrief_input`, `pending_verdicts`) primaju `impl Into<RoundKey>` (testovi s
`Month::Jun` se tako ne mijenjaju). `situations_in(r)`: za `RoundKey::Month(m)` situacije mjeseca `m`, za tjedan
prazno (Task 6). Pogledi: `MonthScore.round`, `LastMonth.round`, `PlanState.round`, `RoundView.round`,
`SituationsView.round`, `FeedItem::*.round`, `KpiMonth.round`, `DebriefInput.{round, next_round, prev_round}`.
Poslovi (`crates/app/src/jobs.rs`) `ResolvePayload.month`, `DebriefPayload.month`: `RoundKey` (JSON isti); ključevi
poslova `resolve:<game>:<code>` i `debrief:<game>:<code>:<hotel>` isti tekst.
API: `parse_month` → `parse_round(s) -> Result<RoundKey, Problem>` (404 `unknown_month` za neispravan kod);
`month` polja u tijelima ostaju `MonthCode` i pune se iz `round.month()` (za `Week` → 500 `internal`, dok Task 7
ne doda kalendar; u v1 igrama se nikad ne događa).

- [ ] **Step 1: Testovi ključa (padaju)** — `crates/game/tests/round_key.rs`:

```rust
use game::RoundKey;
use sim_hotel::Month;

#[test]
fn month_keys_print_and_parse_like_old_month_codes() {
    for m in Month::SEASON {
        let k = RoundKey::from(m);
        assert_eq!(k.code(), m.code());
        assert_eq!(RoundKey::parse(m.code()), Some(k));
        assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{}\"", m.code()));
    }
}

#[test]
fn week_keys_print_and_parse() {
    assert_eq!(RoundKey::Week(7).code(), "t07");
    assert_eq!(RoundKey::parse("t01"), Some(RoundKey::Week(1)));
    assert_eq!(RoundKey::parse("t17"), Some(RoundKey::Week(17)));
    assert_eq!(serde_json::from_str::<RoundKey>("\"t03\"").unwrap(), RoundKey::Week(3));
}

#[test]
fn bad_codes_are_rejected() {
    for bad in ["t00", "t18", "t7", "T07", "Jun", "jun ", "", "t1x", "week"] {
        assert_eq!(RoundKey::parse(bad), None, "{bad:?}");
        assert!(serde_json::from_str::<RoundKey>(&format!("\"{bad}\"")).is_err());
    }
}

#[test]
fn keys_order_by_season() {
    assert!(RoundKey::from(Month::Jun) < RoundKey::from(Month::Jul));
    assert!(RoundKey::from(Month::Aug) < RoundKey::from(Month::Sep));
    assert!(RoundKey::Week(2) < RoundKey::Week(10));
}
```

- [ ] **Step 2:** `cargo test -p game --test round_key` → FAIL.
- [ ] **Step 3: `round.rs`** (deserializacija: `String::deserialize` pa `parse`, inače `de::Error::custom("nepoznat
  krug …")`), izvoz `pub use round::{MAX_WEEK, RoundKey};` u `lib.rs`.
- [ ] **Step 4: Zamjena tipa** u `game` (popis gore), pa `app`, pa `api`. Prevoditelj vodi: nakon svake datoteke
  `cargo check --workspace --all-targets`. Pozor: `BTreeMap` po `RoundKey` je u sezonskom redoslijedu (izvedeni `Ord`),
  kao i po `Month` — iteracije po mapi ostaju istog redoslijeda.
- [ ] **Step 5:** `cargo test --workspace; echo EXIT=$?` → sve zeleno, posebno `game --test v1_frozen` (6 passed).
  `corepack pnpm check:api; echo EXIT=$?` → EXIT=0 (`openapi.json` nepromijenjen).
- [ ] **Step 6: Sve provjere i commit** „Sloj 1/5: ključ kruga (RoundKey) umjesto mjeseca; v1 događaji bajt-isti".

**Prihvaćanje:** `git diff --stat -- openapi.json packages/api-client crates/game/tests/fixtures crates/sim-hotel` prazno;
`grep -rn "MonthCode" crates/game/src` pogađa samo `dto.rs` (kalendarski mjesec).

---

