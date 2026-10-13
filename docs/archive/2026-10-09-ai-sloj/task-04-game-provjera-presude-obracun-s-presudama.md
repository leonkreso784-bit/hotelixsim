### Task 4: `game` — provjera presude, obračun s presudama, čekanje i neutralna presuda, ulazi za AI

**Files:**
- Create: `crates/game/src/verdict.rs`, `crates/game/src/ai_input.rs`
- Modify: `crates/game/src/resolve.rs`, `crates/game/src/event.rs` (`SituationResult.free_text`), `crates/game/src/lib.rs`
- Modify: `crates/app/src/jobs.rs` (samo prilagodba novom potpisu `resolve_job`, da se prevodi; prava logika je Task 6)
- Test: `crates/game/tests/verdict.rs` (novo), `crates/game/tests/resolve.rs`

**Interfaces:**
- Consumes: događaji i `talk.rs` (Task 3).
- Produces:

```rust
// verdict.rs
/// Oblik koji AI mora vratiti (JSON Schema u `ai` je zrcalo ovoga); ne vjeruje mu se dok ne prođe `validate_verdict`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictDraft { pub criteria: Vec<CriterionVerdict>, pub nearest_answer: u8, pub rationale: String }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerdictError {
    NotFreeText, CriteriaCount(usize), CriterionNumber(u8), DuplicateCriterion(u8),
    EmptyReason(u8), NearestAnswer(u8), EmptyRationale,
}

/// Ocjena je broj ispunjenih kriterija; 0 ili 1 → 1 (DEMO §7).
pub fn quality_from(criteria: &[CriterionVerdict]) -> u8;

impl Game {
    pub fn validate_verdict(&self, code: &str, draft: VerdictDraft) -> Result<ValidVerdict, VerdictError>;
    /// Upis presude: `VerdictRecorded` dok odgovor čeka (runda `Open`/`Closing`), inače `VerdictLate`.
    pub fn record_verdict(&self, hotel: HotelId, code: &str, v: ValidVerdict, meta: AiMeta) -> Result<GameEvent, GameError>;
    /// Neutralna presuda po pravilu (ARHITEKTURA §8.3 korak 5): mehanike `unanswered`, kvaliteta 3, na pregled.
    pub fn neutral_verdict(&self, hotel: HotelId, code: &str, reason: &str) -> Result<GameEvent, GameError>;
    pub fn pending_verdicts(&self, m: Month) -> Vec<(HotelId, String)>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidVerdict { pub criteria: Vec<CriterionVerdict>, pub nearest_answer: u8, pub quality: u8, pub rationale: String }

// resolve.rs
pub enum ResolveStep {
    Done(Vec<(Actor, GameEvent)>),
    /// Runda je zatvorena, ali presude za slobodne odgovore još nisu stigle i rok nije prošao.
    WaitForVerdicts(usize),
}
impl Game { pub fn resolve_job(&self, m: Month, deadline_passed: bool) -> Result<ResolveStep, GameError>; }

// ai_input.rs (sve Serialize, Clone, Debug, PartialEq, Eq)
pub struct OfferedForJudge { pub index: u8, pub label: String, pub criteria: Vec<u8> }
pub struct AssessInput {
    pub situation: String, pub title: String, pub rubric: Vec<String>,
    pub character: Option<String>, pub offered: Vec<OfferedForJudge>,
    pub transcript: Vec<ChatLine>, pub response: String,
}
pub struct CharacterInput { pub situation: String, pub title: String, pub character: content::Character /* bez Serialize: polja preslikaj u vlastitu strukturu `CharacterCard { name, role, brief, opening }` */, pub transcript: Vec<ChatLine> }
pub struct DebriefSituation { pub title: String, pub quality: u8, pub free_text: bool, pub rationale: Option<String>, pub neutral: bool }
pub struct DebriefInput {
    pub month: MonthCode, pub hotel_name: String, pub situations: Vec<DebriefSituation>,
    pub indices_before: [u16; 5], pub indices_after: [u16; 5], pub progress_hundredths: i32,
    pub lines: Vec<crate::dto::LineDto>,
}
impl Game {
    pub fn assess_input(&self, hotel: HotelId, code: &str) -> Result<AssessInput, GameError>;
    pub fn character_input(&self, hotel: HotelId, code: &str) -> Result<CharacterInput, GameError>;
    pub fn debrief_input(&self, hotel: HotelId, m: Month) -> Result<DebriefInput, GameError>;
}
/// Mentor bez AI-ja: deterministički tekst iz brojki (ARHITEKTURA §8.5 „bez AI-ja").
pub fn fallback_debrief(input: &DebriefInput) -> String;
```

  `CharacterInput.character` neka bude `CharacterCard { name, role, brief, opening }` (Serialize) — **ne** `content::Character`.
  `LineDto` mora imati `Serialize` (već ima, provjeri). `indices_before` = indeksi na početku mjeseca (početni ili iz
  prošlog `RoundResolved`), `indices_after` = iz `RoundResolved` tog mjeseca.

- [ ] **Step 1: Napiši testove koji padaju** — `crates/game/tests/verdict.rs`:

```rust
fn crit(met: [bool; 5]) -> Vec<CriterionVerdict> {
    met.iter().zip(1u8..).map(|(&m, n)| CriterionVerdict { n, met: m, reason: format!("razlog {n}") }).collect()
}

#[test]
fn quality_is_computed_not_trusted() {
    assert_eq!(quality_from(&crit([false; 5])), 1);
    assert_eq!(quality_from(&crit([true, false, false, false, false])), 1);
    assert_eq!(quality_from(&crit([true, true, false, true, false])), 3);
    assert_eq!(quality_from(&crit([true; 5])), 5);
}

#[test]
fn verdict_shape_is_checked() {
    let (g, _) = common::open_june_game();
    let ok = VerdictDraft { criteria: crit([true; 5]), nearest_answer: 0, rationale: "dobro".into() };
    assert_eq!(g.validate_verdict("S1", ok.clone()).unwrap().quality, 5);
    let mut four = ok.clone(); four.criteria.pop();
    assert_eq!(g.validate_verdict("S1", four), Err(VerdictError::CriteriaCount(4)));
    let mut dup = ok.clone(); dup.criteria[1].n = 1;
    assert_eq!(g.validate_verdict("S1", dup), Err(VerdictError::DuplicateCriterion(1)));
    let mut six = ok.clone(); six.criteria[4].n = 6;
    assert_eq!(g.validate_verdict("S1", six), Err(VerdictError::CriterionNumber(6)));
    let mut far = ok.clone(); far.nearest_answer = 4; // S1 ima 4 odgovora: 0..=3
    assert_eq!(g.validate_verdict("S1", far), Err(VerdictError::NearestAnswer(4)));
    let mut blank = ok.clone(); blank.criteria[2].reason = " ".into();
    assert_eq!(g.validate_verdict("S1", blank), Err(VerdictError::EmptyReason(3)));
    let mut norat = ok.clone(); norat.rationale = String::new();
    assert_eq!(g.validate_verdict("S1", norat), Err(VerdictError::EmptyRationale));
    assert_eq!(g.validate_verdict("S3", ok), Err(VerdictError::NotFreeText));
}
```

  i u `crates/game/tests/resolve.rs`:

```rust
#[test]
fn verdict_drives_resolution_with_nearest_answer_mechanics() {
    // Slobodan odgovor s presudom (q=3, najbliži odgovor 1) daje ISTI obračun hotela kao ponuđeni odgovor 1 (q=3).
    let free = common::june_resolved_with_free_verdict(3, 1); // RoundResolved
    let offered = common::june_resolved_with_offered(1);
    let a = common::my_resolution(&free);
    let b = common::my_resolution(&offered);
    assert_eq!(a.report, b.report);
    assert_eq!(a.indices, b.indices);
    let s1 = a.situations.iter().find(|s| s.code == "S1").unwrap();
    assert!(s1.free_text);
    assert_eq!(s1.quality, 3);
}

#[test]
fn closing_with_pending_verdict_waits_then_goes_neutral() {
    let g = common::june_closed_with_pending_response(); // slobodan odgovor bez presude, RoundClosed
    assert_eq!(g.pending_verdicts(Month::Jun).len(), 1);
    assert!(matches!(g.resolve_job(Month::Jun, false).unwrap(), ResolveStep::WaitForVerdicts(1)));
    let ResolveStep::Done(evs) = g.resolve_job(Month::Jun, true).unwrap() else { panic!() };
    let GameEvent::VerdictRecorded(v) = &evs[0].1 else { panic!("prvo neutralna presuda") };
    assert_eq!((v.quality, v.nearest_answer, v.source, v.review), (3, None, AiSource::Fallback, true));
    assert!(matches!(evs[1].1, GameEvent::RoundResolved(_)));
}

#[test]
fn verdict_after_neutral_is_late_and_changes_nothing() {
    let mut g = common::june_closed_with_pending_response();
    let ResolveStep::Done(evs) = g.resolve_job(Month::Jun, true).unwrap() else { panic!() };
    for (a, e) in evs { common::apply_ev(&mut g, a, e); }
    let before = g.resolved[&Month::Jun].clone();
    let h = common::my_hotel(&g);
    let v = g.validate_verdict("S1", common::draft_all_met(0)).unwrap();
    let ev = g.record_verdict(h, "S1", v, common::meta()).unwrap();
    assert!(matches!(ev, GameEvent::VerdictLate(_)));
    common::apply_ev(&mut g, Actor::System, ev);
    assert_eq!(g.resolved[&Month::Jun], before);
}

#[test]
fn replay_with_verdicts_is_byte_identical() {
    // Ponovljivost (DEMO §11.5) i uz presude: dnevnik pročitan kroz JSON (kao iz baze) obračuna lipanj bajt-isto.
    let g = common::june_resolved_with_free_verdict_game(4, 0);
    let reread: Vec<Recorded> = g.history.iter().map(|r| {
        let text = serde_json::to_string(&r.event.to_payload()).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();
        Recorded { seq: r.seq, actor: r.actor.clone(), event: GameEvent::from_record(r.event.kind(), 1, payload).unwrap() }
    }).collect();
    let pos = reread.iter().position(|r| matches!(r.event, GameEvent::RoundResolved(_))).unwrap();
    let before = Game::fold(&reread[..pos]).unwrap();
    let again = before.resolve_round(Month::Jun).unwrap();
    let stored = Game::fold(&reread).unwrap().resolved[&Month::Jun].clone();
    assert_eq!(serde_json::to_vec(&again).unwrap(), serde_json::to_vec(&stored).unwrap());
}

#[test]
fn fallback_debrief_mentions_progress_and_situations() {
    let g = common::june_resolved_game();
    let input = g.debrief_input(common::my_hotel(&g), Month::Jun).unwrap();
    let text = fallback_debrief(&input);
    assert!(text.contains("lipanj") || text.contains("lipnju"));
    assert!(!text.is_empty() && text.chars().count() < 1200);
}
```

  Pomoćnike (`june_resolved_with_free_verdict` → `RoundResolved`, `june_resolved_with_free_verdict_game` → `Game`,
  `june_resolved_with_offered`, `june_closed_with_pending_response`, `draft_all_met(nearest)`, `meta`, `my_hotel`,
  `my_resolution`, `june_resolved_game`) dodaj u `tests/common/mod.rs`; svaki gradi igru isključivo naredbama `game`-a
  (`submit_response`, `validate_verdict`, `record_verdict`, `choose_answer`, `close_round`, `resolve_job`) i `apply_ev`.
  `june_resolved_with_free_verdict(q, i)` gradi nacrt presude s točno `q` ispunjenih kriterija (za `q = 1` jedan) i
  najbližim odgovorom `i`.

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p game; echo EXIT=$?`.

- [ ] **Step 3: Implementiraj `verdict.rs`** — `validate_verdict`: situacija ima `free_text` (inače `NotFreeText`);
  točno 5 kriterija; svaki `n` u 1..=5 i jednom; `reason.trim()` neprazan; `nearest_answer < answers.len()`;
  `rationale.trim()` neprazan; vrati `ValidVerdict` s kriterijima poredanim po `n` i `quality = quality_from(..)`.
  `record_verdict`: odgovor postoji (`UnknownSituation` ako ne); ako nema presude i runda je `Open`/`Closing` →
  `VerdictRecorded { source: Ai, review: false, meta: Some(meta), nearest_answer: Some(i) }`, inače `VerdictLate(…)`.
  `neutral_verdict(hotel, code, reason)`: `criteria: vec![]`, `nearest_answer: None`, `quality: 3`, `rationale: reason`,
  `source: Fallback`, `meta: None`, `review: true`. Tekstovi razloga (konstante u `verdict.rs`):
  `NEUTRAL_DEADLINE = "AI presuda nije stigla do kraja zatvaranja mjeseca; neutralna presuda po pravilu, na pregled profesoru."`
  i `NEUTRAL_FAILED = "AI presuda nije uspjela ni nakon ponovljenih pokušaja; neutralna presuda po pravilu, na pregled profesoru."`

- [ ] **Step 4: Obračun s presudama** — u `resolve_round` za svaku situaciju redom: ponuđeni odgovor (kao danas) →
  slobodan odgovor s presudom (mehanike `answers[i]` za `Some(i)`, `t.unanswered` za `None`; kvaliteta iz presude;
  `SituationResult { answer: None, quality, free_text: true }`) → slobodan odgovor **bez** presude = `corrupt("presuda
  nedostaje")` → neodgovoreno (kao danas). `SituationResult` dobiva
  `#[serde(default, skip_serializing_if = "is_false")] pub free_text: bool`. Mehanike neutralne presude s kvalitetom 3
  daju učinak 0 (`(q − 3) = 0`), a trošak kompenzacije iz `unanswered` (0 €) — zapiši to kao doc-komentar.

- [ ] **Step 5: `resolve_job(m, deadline_passed)`** —

```rust
pub fn resolve_job(&self, m: Month, deadline_passed: bool) -> Result<ResolveStep, GameError> {
    // provjera core_version iz Taska 1 ostaje prva
    match self.status(m)? {
        RoundStatus::Resolved => Ok(ResolveStep::Done(Vec::new())),
        RoundStatus::Closing => {
            let pending = self.pending_verdicts(m);
            if !pending.is_empty() && !deadline_passed {
                return Ok(ResolveStep::WaitForVerdicts(pending.len()));
            }
            let mut next = self.clone();
            let mut out = Vec::new();
            for (hotel, code) in pending {
                let ev = next.neutral_verdict(hotel, &code, NEUTRAL_DEADLINE)?;
                next.apply(Recorded { seq: next.last_seq + 1, actor: Actor::System, event: ev.clone() })?;
                out.push((Actor::System, ev));
            }
            let rr = next.resolve_round(m)?;
            next.apply(Recorded { seq: next.last_seq + 1, actor: Actor::System, event: GameEvent::RoundResolved(rr.clone()) })?;
            out.push((Actor::System, GameEvent::RoundResolved(rr)));
            if let Some(n) = next.current_month() {
                out.extend(next.robot_moves(n)?);
            }
            Ok(ResolveStep::Done(out))
        }
        RoundStatus::Open | RoundStatus::Scheduled => Err(corrupt("posao obračuna za rundu koja nije zatvorena")),
    }
}
```

  `pending_verdicts(m)`: po hotelima redom slota, po situacijama redom iz sadržaja — odgovor u mjesecu `m` bez presude.
  U `crates/app/src/jobs.rs` privremeno: `match g.resolve_job(m, false)? { ResolveStep::Done(evs) => evs, ResolveStep::WaitForVerdicts(_) => return Err(AppError::internal("presude još čekaju")) }`
  (Task 6 to zamjenjuje odgodom).

- [ ] **Step 6: `ai_input.rs`** — `assess_input`: rubrika, ime lika, ponuđeni s kriterijima (iz `Content.ai`), prijepis
  razgovora i tekst odgovora (`UnknownSituation` ako odgovora nema). `character_input`: prijepis **bez** uvodne rečenice
  (uvod ide u `CharacterCard.opening`). `debrief_input`: iz `RoundResolved` tog mjeseca (`RoundNotOpen` ako nije obračunat).
  `fallback_debrief` — bez AI-ja, iz brojki, primjer oblika:

```text
U lipnju se tvoj napredak promijenio za +1,25 boda (ukupno +1,25).
Najviše se pomaknuo indeks Zadovoljstvo gostiju: 50,00 → 53,40.
Situacije: „Klima u sobi obitelji s djecom" — ocjena 4/5; „Agencija nudi grupu…" — ocjena 1/5 (bez odgovora).
Mentor danas govori iz brojki; AI objašnjenje nije bilo dostupno.
```

  Brojke formatiraj cjelobrojno (stotinke → `"{},{:02}"`, predznak), imena indeksa iz postojećih kodova (`IndexCode`)
  preslikaj u hrvatske nazive u `ai_input.rs` (G ugled/ocjena, Z zadovoljstvo, O osoblje, R …, F financije — uzmi točne
  nazive iz DEMO §4). Mjesec u lokativu: lipnju, srpnju, kolovozu, rujnu.

- [ ] **Step 7: Pokreni sve provjere** — sve `EXIT=0`.

- [ ] **Step 8: Commit** — „AI sloj 4/11: provjera presude, obračun s presudama, neutralna i zakašnjela presuda".

---

