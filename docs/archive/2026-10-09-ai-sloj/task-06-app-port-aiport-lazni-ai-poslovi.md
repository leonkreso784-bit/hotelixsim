### Task 6: `app` — port `AiPort`, lažni AI, poslovi razgovora, presude i mentora, čekanje obračuna

**Files:**
- Modify: `crates/app/src/ports.rs`, `crates/app/src/lib.rs` (`Deps.ai`), `crates/app/src/memory.rs`, `crates/app/src/jobs.rs`,
  `crates/app/src/commands.rs` (`Decision.enqueue: Vec<NewJob>`), `crates/app/src/views.rs`
- Create: `crates/app/src/talk.rs`
- Modify: `crates/store/src/log.rs` (samo petlja po `enqueue` umjesto `Option`, da se prevodi — SQL isti)
- Modify: `crates/store/src/jobs.rs` (samo prazni `defer` koji vraća grešku „Task 7", i `age_secs: 0` u `Job`, da se prevodi)
- Modify: `crates/server/src/main.rs`, `crates/api/tests/common/mod.rs` (Deps dobiva `ai`; privremeno `app::DisabledAi`)
- Test: `crates/app/tests/talk.rs` (novo), `crates/app/tests/rounds.rs`

**Interfaces:**
- Consumes: sve iz Taskova 3–5.
- Produces:

```rust
// ports.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiOut<T> { pub value: T, pub meta: game::AiMeta }

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("AI nije uključen (nema ključa)")] Disabled,
    #[error("AI nije dostupan: {0}")] Unavailable(String),
    #[error("AI je odbio zahtjev: {0}")] Refused(String),
    #[error("AI je vratio neispravan izlaz: {0}")] BadOutput(String),
}

pub trait AiPort: Send + Sync {
    fn assess(&self, input: game::AssessInput) -> BoxFut<'_, Result<AiOut<game::VerdictDraft>, AiError>>;
    fn character(&self, input: game::CharacterInput) -> BoxFut<'_, Result<AiOut<String>, AiError>>;
    fn debrief(&self, input: game::DebriefInput) -> BoxFut<'_, Result<AiOut<String>, AiError>>;
}

/// AI bez ključa: sve uloge odmah vraćaju `Disabled`, pa rade rezerve.
pub struct DisabledAi;

pub struct Job { /* postojeće */ pub age_secs: i64 }   // sekunde od upisa posla (računa baza)

pub trait JobQueue: Send + Sync {
    // postojeći claim, fail
    /// Vrati posao u red bez trošenja pokušaja, najranije za `secs` sekundi (čekanje presuda pri zatvaranju).
    fn defer(&self, claim: JobClaim, secs: i32) -> BoxFut<'_, Result<(), StoreError>>;
}

pub struct Decision { pub events: Vec<NewEvent>, pub enqueue: Vec<NewJob>, pub finish_job: Option<JobClaim> }

// jobs.rs
pub const LEASE_SECS: i32 = 180;               // AI poziv za presudu može trajati desetke sekundi
pub const VERDICT_DEADLINE_SECS: i64 = 120;    // od zatvaranja mjeseca do neutralne presude
pub const VERDICT_WAIT_SECS: i32 = 5;
pub const CHAT_KIND: &str = "character_reply";
pub const ASSESS_KIND: &str = "assess_response";
pub const DEBRIEF_KIND: &str = "debrief";
pub fn chat_key(g: GameId, h: HotelId, code: &str, n: u8) -> String;     // "chat:<g>:<h>:<code>:<n>"
pub fn assess_key(g: GameId, h: HotelId, code: &str) -> String;          // "assess:<g>:<h>:<code>"
pub fn debrief_key(g: GameId, m: Month, h: HotelId) -> String;           // "debrief:<g>:<mjesec>:<h>"

// talk.rs (app)
pub struct ConversationView {
    pub situation: String, pub character_name: String, pub role: String, pub opening: String,
    pub lines: Vec<game::ChatLine>, pub remaining: u8, pub awaiting_reply: bool, pub open: bool,
}
pub async fn send_message(deps: &Deps, caller: &Caller, hotel: HotelId, code: &str, text: String) -> Result<ConversationView, AppError>;
pub async fn conversation(deps: &Deps, caller: &Caller, hotel: HotelId, code: &str) -> Result<ConversationView, AppError>;
pub async fn submit_response(deps: &Deps, caller: &Caller, hotel: HotelId, code: &str, text: String) -> Result<SituationsView, AppError>;

// views.rs — SituationItem dobiva:
pub free_text: bool,
pub character: Option<CharacterCardView { name: String, role: String }>,
pub response: Option<ResponseView>,
pub struct ResponseView {
    pub text: String,
    pub status: ResponseStatus,           // Pending | Judged | Neutral
    pub quality: Option<u8>,
    pub criteria: Vec<CriterionView>,     // { n, text (iz rubrike), met, reason }
    pub rationale: Option<String>,
}
```

  `memory.rs`: `FakeAi` s načinima `FakeAi::good()` (presuda: kriteriji 1–4 ispunjeni, 5 nije → q4, najbliži 0;
  lik: `"Hvala, to mi pomaže."`; mentor: `"Mentor: dobar mjesec."`; meta `model: "fake", prompt: "fake.v1"`) i
  `FakeAi::failing()` (sve `Unavailable("lažni ispad")`), plus brojač poziva po ulozi (`calls() -> (u32, u32, u32)`).
  `Memory` dobiva `set_job_age(key: &str, secs: i64)`; `defer` vraća posao u `Queued` i smanjuje `attempts` za 1.
  `memory::deps(snapshot)` koristi `FakeAi::good()`; dodaj `memory::deps_with_ai(snapshot, Arc<dyn AiPort>)`.

- [ ] **Step 1: Napiši testove koji padaju** — `crates/app/tests/talk.rs`. Pomoćnici na vrhu datoteke:

```rust
use std::sync::Arc;

use app::memory::FakeAi;
use app::{Caller, Deps, ErrorCode, GameId, HotelId, ResponseStatus};
use game::RoundStatus;
use game::dto::MonthCode;
use sim_hotel::Month;

fn snapshot() -> content::Snapshot {
    content::Snapshot::read(&content::repo_dir()).unwrap()
}

async fn start(deps: &Deps) -> (Caller, HotelId, GameId) {
    let d = app::start_demo(deps, None).await.unwrap();
    let caller = app::caller(deps, d.new_token.as_deref()).await.unwrap();
    (caller, d.hotel_id, d.game_id)
}

/// Vrti radnika dok red nije prazan; greške poslova se ignoriraju (red ih sam ponavlja), najviše 60 koraka.
async fn run_all_jobs(deps: &Deps) {
    for _ in 0..60 {
        if let Ok(false) = app::run_next_job(deps).await {
            return;
        }
    }
    panic!("red se nije ispraznio u 60 koraka");
}

fn s1_response(s: &app::SituationsView) -> app::ResponseView {
    s.items.iter().find(|i| i.code == "S1").unwrap().response.clone().unwrap()
}
```

  Testovi:

```rust
#[tokio::test]
async fn chat_message_gets_character_reply_through_job() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, _) = start(&deps).await;
    let v = app::send_message(&deps, &caller, hotel, "S1", "Ispričavam se, odmah dolazim.".into()).await.unwrap();
    assert!(v.awaiting_reply);
    assert_eq!(v.remaining, 5);
    run_all_jobs(&deps).await;
    let v = app::conversation(&deps, &caller, hotel, "S1").await.unwrap();
    assert!(!v.awaiting_reply);
    assert_eq!(v.lines.len(), 2);
    assert_eq!(v.lines[1].text, "Hvala, to mi pomaže.");
}

#[tokio::test]
async fn free_response_is_judged_and_shown_with_criteria() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (caller, hotel, _) = start(&deps).await;
    let s = app::submit_response(&deps, &caller, hotel, "S1", "Ispričam se i premjestim obitelj.".into()).await.unwrap();
    assert_eq!(s1_response(&s).status, ResponseStatus::Pending);
    run_all_jobs(&deps).await;
    let r = s1_response(&app::situations(&deps, &caller, hotel).await.unwrap());
    assert_eq!((r.status, r.quality), (ResponseStatus::Judged, Some(4)));
    assert_eq!(r.criteria.len(), 5);
    assert!(r.criteria[0].text.starts_with("Brza isprika"));
}

#[tokio::test]
async fn failing_ai_falls_back_on_last_attempt() {
    let (deps, _mem) = app::memory::deps_with_ai(snapshot(), Arc::new(FakeAi::failing()));
    let (caller, hotel, _) = start(&deps).await;
    app::send_message(&deps, &caller, hotel, "S1", "Halo?".into()).await.unwrap();
    // razgovor ne blokira slobodan odgovor: student smije razgovarati pa odgovoriti
    app::submit_response(&deps, &caller, hotel, "S1", "Premjestit ću vas odmah.".into()).await.unwrap();
    // S2 je srpanj: šifra postoji (jedinstvena po igri, ADR-017), ali runda nije otvorena
    let e = app::submit_response(&deps, &caller, hotel, "S2", "Dobit ćeš povišicu.".into()).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::RoundNotOpen);
    run_all_jobs(&deps).await; // memorija nema sat: neuspio posao odmah se vraća; peti pokušaj piše rezervu
    let v = app::conversation(&deps, &caller, hotel, "S1").await.unwrap();
    assert_eq!(v.lines.last().unwrap().text, "Gošća je previše ljuta da bi nastavila razgovor. Odluči što ćeš poduzeti.");
    let r = s1_response(&app::situations(&deps, &caller, hotel).await.unwrap());
    assert_eq!((r.status, r.quality), (ResponseStatus::Neutral, Some(3)));
}

#[tokio::test]
async fn close_waits_for_verdict_then_neutral_and_late_verdict_is_kept() {
    let (deps, mem) = app::memory::deps(snapshot());
    let (caller, hotel, game) = start(&deps).await;
    app::submit_response(&deps, &caller, hotel, "S1", "Ispričam se i premjestim obitelj.".into()).await.unwrap();
    // posao procjene preuzmi ručno i drži ga „u radu" (kao spor AI)
    let assess = deps.jobs.claim(app::LEASE_SECS).await.unwrap().unwrap();
    assert_eq!(assess.kind, app::ASSESS_KIND);
    app::close_round(&deps, &caller, game, MonthCode::Jun).await.unwrap();
    // obračun vidi presudu na čekanju i odgađa se
    assert!(app::run_next_job(&deps).await.unwrap());
    let r = app::round_status(&deps, &caller, game, MonthCode::Jun).await.unwrap();
    assert_eq!(r.status, RoundStatus::Closing);
    // rok je prošao → neutralna presuda i obračun
    mem.set_job_age(&app::resolve_key(game, Month::Jun), app::VERDICT_DEADLINE_SECS + 1);
    assert!(app::run_next_job(&deps).await.unwrap());
    let r = app::round_status(&deps, &caller, game, MonthCode::Jun).await.unwrap();
    assert_eq!(r.status, RoundStatus::Resolved);
    let s = app::situations_for_month(&deps, &caller, hotel, MonthCode::Jun).await.unwrap();
    assert_eq!(s1_response(&s).status, ResponseStatus::Neutral);
    // spori AI napokon završi: presuda se zapiše kao zakašnjela, obračun se ne mijenja
    app::run_job_for_test(&deps, &assess).await.unwrap();
    let feed = app::feed(&deps, &caller, hotel).await.unwrap();
    assert!(feed.iter().any(|i| matches!(i, game::feed::FeedItem::Verdict { late: true, .. })));
}

#[tokio::test]
async fn resolve_enqueues_debrief_for_human_only_and_mentor_appears() {
    let ai = Arc::new(FakeAi::good());
    let (deps, _mem) = app::memory::deps_with_ai(snapshot(), ai.clone());
    let (caller, hotel, game) = start(&deps).await;
    app::close_round(&deps, &caller, game, MonthCode::Jun).await.unwrap();
    run_all_jobs(&deps).await;
    let h = app::hotel(&deps, &caller, hotel).await.unwrap();
    let last = h.private.unwrap().last_month.unwrap();
    assert_eq!(last.mentor.as_deref(), Some("Mentor: dobar mjesec."));
    assert_eq!(ai.calls().2, 1, "mentor samo za hotel s čovjekom");
}

#[tokio::test]
async fn other_guest_cannot_chat_or_respond_in_my_hotel() {
    let (deps, _mem) = app::memory::deps(snapshot());
    let (_, hotel, _) = start(&deps).await;
    let (other, _, _) = start(&deps).await; // drugi gost u drugoj igri
    let e = app::send_message(&deps, &other, hotel, "S1", "bok".into()).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
    let e = app::submit_response(&deps, &other, hotel, "S1", "bok".into()).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
    let e = app::conversation(&deps, &other, hotel, "S1").await.unwrap_err();
    assert_eq!(e.code, ErrorCode::NotInGame);
}
```

  Imena postojećih funkcija `app::hotel`, `app::feed`, `app::situations` provjeri u `crates/app/src/views.rs` i prilagodi
  poziv ako se zovu drukčije (ne mijenjaj im ime). Potrebne nove javne stavke: `app::situations_for_month` (situacije
  zadanog mjeseca — `situations` vraća samo trenutni mjesec, a nakon obračuna je trenutni srpanj), `app::run_job_for_test`
  (`#[doc(hidden)] pub async fn` koja zove privatni `run_job`), `app::resolve_key` (već postoji), `Memory::set_job_age`.
  Memorijski `claim` mora preskakati poslove `Running` (nema isteka zakupa bez sata) — provjeri da je tako i danas.

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p app; echo EXIT=$?`.

- [ ] **Step 3: Port i memorija** — kako je zadano u Interfaces. `Deps` dobiva `pub ai: Arc<dyn AiPort>`. `DisabledAi`
  implementira `AiPort` (sve `Err(AiError::Disabled)`).

- [ ] **Step 4: Slučajevi upotrebe (`talk.rs`)** — svaki je JEDAN `decide` (pravo pa pravilo, kao `commands::answer`):
  `send_message` upisuje `ChatMessageSent` (actor gost) i posao `CHAT_KIND` s payloadom
  `ChatPayload { hotel_id, situation, reply_to }`; `submit_response` upisuje `ResponseSubmitted` i posao `ASSESS_KIND`
  s payloadom `AssessPayload { hotel_id, situation }`. Payloadi su `#[serde(deny_unknown_fields)]` kao `ResolvePayload`.
  `conversation` čita (vlasnik), `remaining = 6 − broj studentovih poruka`, `open = runda situacije je Open`.

- [ ] **Step 5: Radnik (`jobs.rs`)** — `run_job` grana po `job.kind` (`RESOLVE_KIND`, `CHAT_KIND`, `ASSESS_KIND`,
  `DEBRIEF_KIND`; nepoznata vrsta → `AppError::internal`, kao danas). Pravilo za sve AI poslove: **AI se zove izvan
  `decide`**, a unutar `decide` se nad SVJEŽOM igrom ponovno provjerava smije li se rezultat upisati. U `game` dodaj
  `pub fn response_pending(&self, hotel: HotelId, code: &str) -> Result<bool, GameError>` (odgovor postoji i nema presudu).

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatPayload { pub hotel_id: HotelId, pub situation: String, pub reply_to: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessPayload { pub hotel_id: HotelId, pub situation: String }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebriefPayload { pub hotel_id: HotelId, pub month: MonthCode }

fn payload<T: serde::de::DeserializeOwned>(job: &Job) -> Result<T, AppError> {
    serde_json::from_value(job.payload.clone()).map_err(|e| AppError::internal(e.to_string()))
}

fn system_events(evs: &[GameEvent]) -> Vec<NewEvent> {
    evs.iter().map(|e| new_event(&Actor::System, e)).collect()
}

/// Zatvori posao bez događaja (nema više što raditi).
async fn finish(deps: &Deps, game: GameId, claim: JobClaim) -> Result<(), AppError> {
    deps.log
        .decide(game, Box::new(move |_raw: &[RawEvent]| Ok(Decision { finish_job: Some(claim), ..Decision::default() })))
        .await?;
    Ok(())
}

async fn run_resolve(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: ResolvePayload = payload(job)?;
    let m: Month = p.month.into();
    let claim = job.claim();
    let deadline_passed = job.age_secs >= VERDICT_DEADLINE_SECS;
    let g = load_game(deps, job.game_id).await?;
    if !deadline_passed && !g.pending_verdicts(m).is_empty() {
        deps.jobs.defer(claim, VERDICT_WAIT_SECS).await?;
        return Ok(());
    }
    let game_id = job.game_id;
    deps.log
        .decide(game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            let evs = match g.resolve_job(m, deadline_passed)? {
                ResolveStep::Done(evs) => evs,
                ResolveStep::WaitForVerdicts(n) => {
                    return Err(AppError::internal(format!("{n} presuda čeka unutar zaključavanja")));
                }
            };
            let resolved = evs.iter().any(|(_, e)| matches!(e, GameEvent::RoundResolved(_)));
            let mut enqueue = Vec::new();
            if resolved {
                for h in g.hotels.iter().filter(|h| matches!(h.setup.controller, Controller::Human { .. })) {
                    let hotel_id = h.setup.hotel_id;
                    enqueue.push(NewJob {
                        key: debrief_key(game_id, m, hotel_id),
                        kind: DEBRIEF_KIND.to_owned(),
                        game_id,
                        payload: serde_json::to_value(DebriefPayload { hotel_id, month: m.into() })
                            .map_err(|e| AppError::internal(e.to_string()))?,
                    });
                }
            }
            Ok(Decision {
                events: evs.iter().map(|(a, e)| new_event(a, e)).collect(),
                enqueue,
                finish_job: Some(claim),
            })
        }))
        .await?;
    Ok(())
}

async fn run_chat(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: ChatPayload = payload(job)?;
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if g.status(g.situation_month(&p.situation)?)? != RoundStatus::Open {
        return finish(deps, job.game_id, claim).await;
    }
    let input = g.character_input(p.hotel_id, &p.situation)?;
    let reply = match deps.ai.character(input).await {
        Ok(out) => ReplyBody { text: out.value, source: AiSource::Ai, meta: Some(out.meta) },
        Err(e) if job.attempts < MAX_ATTEMPTS => return Err(AppError::internal(e.to_string())),
        Err(_) => {
            let text = g.content.ai.get(&p.situation)
                .and_then(|a| a.character.as_ref())
                .map(|c| c.fallback_reply.clone())
                .ok_or_else(|| AppError::internal("situacija bez lika"))?;
            ReplyBody { text, source: AiSource::Fallback, meta: None }
        }
    };
    let (hotel, code, n) = (p.hotel_id, p.situation, p.reply_to);
    deps.log
        .decide(job.game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            let ev = g.record_reply(hotel, &code, n, reply)?;
            Ok(Decision { events: system_events(ev.as_slice()), finish_job: Some(claim), ..Decision::default() })
        }))
        .await?;
    Ok(())
}

async fn run_assess(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: AssessPayload = payload(job)?;
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if !g.response_pending(p.hotel_id, &p.situation)? {
        return finish(deps, job.game_id, claim).await; // presuda već postoji (npr. neutralna nakon roka)
    }
    let input = g.assess_input(p.hotel_id, &p.situation)?;
    let judged = match deps.ai.assess(input).await {
        Ok(out) => g
            .validate_verdict(&p.situation, out.value)
            .map(|v| (v, out.meta))
            .map_err(|e| format!("presuda nije prošla provjeru: {e:?}")),
        Err(e) => Err(e.to_string()),
    };
    let judged = match judged {
        Ok(ok) => Some(ok),
        Err(msg) if job.attempts < MAX_ATTEMPTS => return Err(AppError::internal(msg)),
        Err(_) => None, // zadnji pokušaj: neutralna presuda po pravilu
    };
    let (hotel, code) = (p.hotel_id, p.situation);
    deps.log
        .decide(job.game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            let ev = match judged {
                // VerdictRecorded dok odgovor čeka, VerdictLate ako je u međuvremenu došla neutralna presuda
                Some((v, meta)) => Some(g.record_verdict(hotel, &code, v, meta)?),
                None if g.response_pending(hotel, &code)? => Some(g.neutral_verdict(hotel, &code, NEUTRAL_FAILED)?),
                None => None,
            };
            Ok(Decision { events: system_events(ev.as_slice()), finish_job: Some(claim), ..Decision::default() })
        }))
        .await?;
    Ok(())
}

async fn run_debrief(deps: &Deps, job: &Job) -> Result<(), AppError> {
    let p: DebriefPayload = payload(job)?;
    let m: Month = p.month.into();
    let claim = job.claim();
    let g = load_game(deps, job.game_id).await?;
    if g.hotel(p.hotel_id)?.debriefs.contains_key(&m) {
        return finish(deps, job.game_id, claim).await;
    }
    let input = g.debrief_input(p.hotel_id, m)?;
    let (text, source, meta) = match deps.ai.debrief(input.clone()).await {
        Ok(out) => (out.value, AiSource::Ai, Some(out.meta)),
        Err(e) if job.attempts < MAX_ATTEMPTS => return Err(AppError::internal(e.to_string())),
        Err(_) => (game::fallback_debrief(&input), AiSource::Fallback, None),
    };
    let hotel = p.hotel_id;
    deps.log
        .decide(job.game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            let ev = g.record_debrief(hotel, m, text, source, meta)?;
            Ok(Decision { events: system_events(ev.as_slice()), finish_job: Some(claim), ..Decision::default() })
        }))
        .await?;
    Ok(())
}
```

  `job.attempts` je broj pokušaja **uključujući** ovaj (`claim` ga poveća prije vraćanja), pa je `attempts == MAX_ATTEMPTS`
  zadnji pokušaj. `ResolvePayload` već postoji; `Decision::default()` mora i dalje postojati (`enqueue: Vec::new()`).
  `run_next_job` ostaje isti.
  `run_next_job` ostaje isti; `AppError` iz `AiError` mapiraj u `internal` s porukom (ulazi u `last_error`).

- [ ] **Step 6: Pogledi** — `situations()` puni `free_text`, `character`, `response` (status `Pending` dok nema presude,
  `Judged` za `source: Ai`, `Neutral` za `source: Fallback`; `criteria` spaja rubriku s presudom po `n`). Kasna
  presuda ne mijenja `response` u situacijama (status ostaje `Neutral`, jer obračun ide po neutralnoj); vidi se samo u
  feedu kao `FeedItem::Verdict { late: true, .. }`, a ispravku obračuna radi profesor u koraku 6. Dodaj i
  `situations_for_month(deps, caller, hotel, month)` (isti pogled za zadani mjesec; `situations` ga zove s trenutnim).

- [ ] **Step 7: Prilagodi pozivatelje** — `close_round` (`enqueue: vec![…]`), `jobs.rs`, `store/src/log.rs` (petlja),
  `store/src/jobs.rs` (`age_secs: 0`, `defer` privremeno `Err(StoreError("defer: Task 7".into()))`), `server` i
  `api/tests/common` (`ai: Arc::new(app::DisabledAi)`). Stari testovi u `crates/app/tests/rounds.rs` moraju proći.

- [ ] **Step 8: Pokreni sve provjere** — sve `EXIT=0`.
- [ ] **Step 9: Commit** — „AI sloj 6/11: port AiPort, poslovi lika, presude i mentora, čekanje obračuna".

---

