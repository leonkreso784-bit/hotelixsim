### Task 8: Mentor u tjednoj igri

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 6 · **Paralelno s:** Task 7, Task 9 · **Model:** sonnet

**Cilj (🟨 P3):** u tjednoj igri AI mentor se zove samo u krugu u kojem završava kalendarski mjesec (t05, t09, t14,
t17); u ostalim krugovima mentor je rezervni tekst bez AI-ja (isti oblik u četiri dijela, „tri uzroka"). Činjenice
mentora govore o tjednu riječima („u tjednu od 29. lipnja do 5. srpnja") i uspoređuju s prethodnim krugom.

**Files:**
- Modify: `crates/game/src/{mentor,ai_input}.rs`, `crates/app/src/jobs.rs`
- Create: `crates/game/tests/mentor_weekly.rs`, dodaj test u `crates/app/tests/rounds.rs`
- Ne dirati: `prompts/*.md`, `crates/ai/**`, mjesečne tekstove mentora (v1 igre moraju dati isti tekst — postojeći
  testovi `crates/game/tests/mentor.rs` ostaju nepromijenjeni i zeleni).

**Interfaces:**
- Consumes: `Game::round_info`, `Period::{first_date, last_date, month_ending_in}`.
- Produces:

```rust
// crates/game/src/ai_input.rs — DebriefInput dobiva (serde default):
pub round_kind: String,           // "month" | "week"
pub first_date: (u8, MonthCode), pub last_date: (u8, MonthCode),
// crates/game/src/mentor.rs
/// Treba li za ovaj krug zvati AI mentora (🟨 P3): v1 i mjesečna v2 uvijek; tjedna samo kad u krugu završava mjesec.
pub fn wants_ai_mentor(g: &Game, r: RoundKey) -> Result<bool, GameError>;
```

Mjesto u rečenici: mjesec → postojeći `month_locative` („u lipnju"); tjedan → „u tjednu od {d}. {genitiv mjeseca} do
{d}. {genitiv mjeseca}" (genitivi: lipnja, srpnja, kolovoza, rujna); „sljedeći put" za tjedan je „sljedeći tjedan".

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/mentor_weekly.rs`:

```rust
mod common;

use game::*;

#[test]
fn week_locative_names_both_dates() {
    let g = common::v2_weekly_game_resolved_through("t05");
    let h = common::hotel_ids()[0];
    let input = g.debrief_input(h, RoundKey::Week(5)).unwrap();
    let facts = debrief_facts(&input);
    assert_eq!(facts.month, "u tjednu od 29. lipnja do 5. srpnja");
    assert_eq!(facts.next_month.as_deref(), Some("sljedeći tjedan"));
}

#[test]
fn first_week_compares_with_start_of_game() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let input = g.debrief_input(common::hotel_ids()[0], RoundKey::Week(1)).unwrap();
    assert!(input.prev_round.is_none());
}

#[test]
fn ai_mentor_only_when_a_month_ends() {
    let g = common::v2_weekly_game_resolved_through("t09");
    let wants: Vec<u8> = (1..=9).filter(|&n| wants_ai_mentor(&g, RoundKey::Week(n)).unwrap()).collect();
    assert_eq!(wants, [5, 9]);
    let (v1, _) = common::open_june_game();
    assert!(wants_ai_mentor(&v1, sim_hotel::Month::Jun.into()).unwrap());
}
```

U `crates/game/tests/common/mod.rs` dodaj `pub fn v2_weekly_game_resolved_through(code: &str) -> Game` (stvara v2
tjednu igru kao `model_v2.rs`, zatvara i obračunava krugove do uključivo `code`). Polje `facts.month` je postojeće ime u
`DebriefFacts` (`&'static str` postaje `String` ako treba — tada prilagodi postojeće testove samo tipom, ne tekstom).

`crates/app/tests/rounds.rs` — novi test `weekly_debrief_calls_ai_only_at_month_end`: memorijske ovisnosti s `FakeAi`
koji broji pozive mentora (ako brojač ne postoji, dodaj `AtomicUsize` u `FakeAi` iza postojećeg sučelja), v2 tjedna
igra, zatvori i obradi poslove za t01…t05 → mentor zapisan za svih 5 krugova, AI pozvan točno 1 put (t05), ostali
`source: fallback`.

- [ ] **Step 2:** `cargo test -p game --test mentor_weekly; cargo test -p app --test rounds` → FAIL.
- [ ] **Step 3: Implementiraj.** U `run_debrief` (`crates/app/src/jobs.rs`): ako `!wants_ai_mentor(&g, r)` → odmah
  `fallback_debrief(&input)` sa `AiSource::Fallback`, bez poziva `deps.ai.debrief`.
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno (postojeći mentor testovi nepromijenjeni).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/8: mentor u tjednoj igri — AI samo na kraju mjeseca".

---

