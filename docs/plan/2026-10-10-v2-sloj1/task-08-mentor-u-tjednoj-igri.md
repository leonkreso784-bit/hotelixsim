### Task 8: Mentor u tjednoj igri

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 6, **Task 7** (test u `app` stvara v2 tjednu igru kroz
`NewGameSettings` i `Deps.snapshot_v2`) · **Paralelno s:** Task 10, Task 13, Task 18 (val 4) · **Model:** sonnet

**Cilj (🟨 P3, Leon ADR-028 t. 5 „mentor tjedno kratko bez AI-ja"):** u tjednoj igri AI mentor se zove samo u krugu u
kojem završava kalendarski mjesec (t05, t09, t14, t17) — bez ključa tada dolazi puni rezervni mentor u četiri dijela,
kao danas. **Ostali tjedni dobivaju samo kratki sažetak „tri uzroka" bez AI-ja** (ne puni rezervni mentor). Činjenice
mentora govore o tjednu riječima („u tjednu od 29. lipnja do 5. srpnja") i uspoređuju s prethodnim krugom.

**Files:**
- Modify: `crates/game/src/{mentor,ai_input,lib}.rs` (`lib.rs`: izvoz `wants_ai_mentor` i `weekly_summary` uz
  postojeće `debrief_facts, fallback_debrief`, `lib.rs:30`), `crates/app/src/jobs.rs`,
  `crates/game/tests/common/mod.rs` (novi pomoćnik `v2_weekly_game_resolved_through`)
- Create: `crates/game/tests/mentor_weekly.rs`; dodaj test u `crates/app/tests/rounds.rs`
- Mehaničke izmjene literala (Global Constraints; očekivanja se ne mijenjaju) — tri nova polja `DebriefInput` s
  v1 vrijednostima srpnja (`round_kind: "month".into(), first_date: (1, MonthCode::Jul), last_date: (31,
  MonthCode::Jul)`) u `crates/game/tests/mentor.rs:49` (`sample()`) i `crates/ai/tests/request.rs:133`
  (`sample_debrief()`); ništa drugo u `crates/ai/**`.
- Ne dirati: `prompts/*.md`, `crates/ai/src/**`, `crates/app/src/memory.rs` (`FakeAi` već broji pozive:
  `FakeAi::calls()` vraća `(presuda, lik, mentor)`, `memory.rs:94-100`), mjesečne tekstove mentora (v1 igre moraju
  dati isti tekst — očekivanja postojećih testova u `crates/game/tests/mentor.rs` se ne mijenjaju).

**Interfaces:**
- Consumes: `Game::round_info`, `Period::{first_date, last_date, month_ending_in}`, Task 5 `round_locative`, Task 7
  `NewGameSettings`, `start_demo(deps, token, settings)`.
- Produces:

```rust
// crates/game/src/ai_input.rs — DebriefInput dobiva (serde default):
pub round_kind: String,           // "month" | "week"
pub first_date: (u8, MonthCode), pub last_date: (u8, MonthCode),
// crates/game/src/mentor.rs
/// Treba li za ovaj krug zvati AI mentora (🟨 P3): v1 i mjesečna v2 uvijek; tjedna samo kad u krugu završava mjesec.
pub fn wants_ai_mentor(g: &Game, r: RoundKey) -> Result<bool, GameError>;
/// Kratki tjedni sažetak bez AI-ja („tri uzroka", Leon ADR-028 t. 5) za tjedan u kojem ne završava mjesec.
pub fn weekly_summary(input: &DebriefInput) -> String;
```

Mjesto u rečenici: mjesec → `round_locative` iz Taska 5 („u lipnju", tekst isti kao danas); tjedan → „u tjednu od
{d}. {genitiv mjeseca} do {d}. {genitiv mjeseca}" (genitivi: lipnja, srpnja, kolovoza, rujna); „sljedeći put" za
tjedan je „sljedeći tjedan". `DebriefFacts.month` i `DebriefFacts.next_month` postaju `String` / `Option<String>`
(tekst tjedna se slaže iz datuma); postojeći testovi se prilagođavaju samo tipom (`.as_deref()`), ne tekstom.

`weekly_summary` (iz `debrief_facts(input)`, samo postojeće rečenice i pomoćnici iz `mentor.rs`):
1. `"{Mjesto velikim slovom} je napredak tvog hotela od početka igre {napredak}."` (ista prva rečenica kao
   `fallback_debrief`), prazan redak;
2. naslov `Tri uzroka`, pa najviše tri rečenice za tri indeksa s najvećom apsolutnom promjenom (`change ≠ 0`; jednaki →
   redoslijed `INDEX_ORDER`): pao → `why_fell(i)` („Indeks … pao je ponajviše zbog …"), porastao → `"Indeks {} porastao
   je s {} na {}."`; nijedan se nije pomaknuo → `"Ovaj tjedan se nijedan indeks nije pomaknuo."`;
3. zadnji redak `"Puni pregled s mentorom stiže na kraju mjeseca."`.
Bez dijelova „Što je išlo dobro / Što nije valjalo / Zašto / Što probati" i **bez** rečenice „AI objašnjenje danas nije
bilo dostupno…" (AI se ovdje namjerno ne zove).

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

#[test]
fn weekly_summary_is_three_causes_without_ai_note() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let input = g.debrief_input(common::hotel_ids()[0], RoundKey::Week(2)).unwrap();
    let text = weekly_summary(&input);
    assert!(text.starts_with("U tjednu od 8. lipnja do 14. lipnja je napredak tvog hotela od početka igre "), "{text}");
    assert!(text.contains("\nTri uzroka\n"), "{text}");
    assert!(text.lines().filter(|l| l.starts_with("Indeks ")).count() <= 3, "{text}");
    assert!(text.ends_with("Puni pregled s mentorom stiže na kraju mjeseca."), "{text}");
    for long in ["Što je išlo dobro", "Što nije valjalo", "Zašto\n", "Što probati", "AI objašnjenje"] {
        assert!(!text.contains(long), "{long:?} u {text}");
    }
}
```

U `crates/game/tests/common/mod.rs` dodaj `pub fn v2_weekly_game_resolved_through(code: &str) -> Game` (stvara v2
tjednu igru kao `model_v2.rs` — `new_game_with(.., &Snapshot::read_v2(&repo_dir()).unwrap(), .., GameSettings { kind:
PeriodKind::Week })` — pa zatvara i obračunava krugove redom do uključivo `code`).

`crates/app/tests/rounds.rs` — novi test (nakon Taska 7 postoji sve što treba):

```rust
#[tokio::test]
async fn weekly_debrief_calls_ai_only_at_month_end() {
    let ai = Arc::new(app::memory::FakeAi::good());
    let (d, mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai.clone());
    let settings = NewGameSettings { model: content::Model::V2, kind: Some(sim_hotel_v2::PeriodKind::Week) };
    let s = start_demo(&d, None, settings).await.unwrap();
    let me = caller(&d, s.new_token.as_deref()).await.unwrap();
    for n in 1..=5 {
        assert!(close_round(&d, &me, s.game_id, game::RoundKey::Week(n)).await.unwrap().accepted, "t0{n}");
        while run_next_job(&d).await.unwrap() {}
    }
    let debriefs: Vec<_> = mem.events(s.game_id).into_iter().filter(|e| e.kind == "DebriefRecorded").collect();
    assert_eq!(debriefs.len(), 5);
    assert_eq!(ai.calls().2, 1); // samo t05 (završava lipanj)
    for e in &debriefs[..4] {
        assert_eq!(e.payload["source"], "fallback");
        assert!(e.payload["text"].as_str().unwrap().contains("Tri uzroka"));
        assert!(!e.payload["text"].as_str().unwrap().contains("Što je išlo dobro"));
    }
    assert_eq!(debriefs[4].payload["month"], "t05");
    assert_eq!(debriefs[4].payload["source"], "ai");
}
```

(`close_round` prima `RoundKey` od Taska 5; `CloseOutcome.accepted` postoji danas, `rounds.rs:83`.)

- [ ] **Step 2:** `cargo test -p game --test mentor_weekly; cargo test -p app --test rounds` → FAIL.
- [ ] **Step 3: Implementiraj.** U `run_debrief` (`crates/app/src/jobs.rs:311-341`), prije poziva AI-ja: ako
  `!wants_ai_mentor(&g, r)?` → odmah `(weekly_summary(&input), AiSource::Fallback, None)`, bez poziva
  `deps.ai.debrief`. Inače kao danas: AI, a kad AI ne radi (nema ključa, ispad) → `fallback_debrief(&input)` (puni,
  četiri dijela).
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno (očekivanja postojećih mentor testova nepromijenjena).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/8: mentor u tjednoj igri — tjedno tri uzroka, AI na kraju mjeseca".

---

