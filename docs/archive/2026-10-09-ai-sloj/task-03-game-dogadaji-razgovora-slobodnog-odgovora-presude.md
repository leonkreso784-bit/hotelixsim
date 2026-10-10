### Task 3: `game` — događaji razgovora, slobodnog odgovora, presude i mentora; naredbe i pravila

**Files:**
- Modify: `crates/game/src/event.rs`, `crates/game/src/game.rs`, `crates/game/src/error.rs`, `crates/game/src/lib.rs`,
  `crates/game/src/command.rs` (`choose_answer` odbija ako postoji slobodan odgovor), `crates/game/src/feed.rs` (samo da
  se prevodi: nove varijante se u feedu zasad preskaču; prave stavke su Task 5)
- Create: `crates/game/src/talk.rs`
- Modify: `crates/app/src/error.rs` (mapiranje novih grešaka)
- Test: `crates/game/tests/talk.rs` (novo)

**Interfaces:**
- Consumes: `Content.ai` (Task 2).
- Produces (`game::event`, sve `#[serde(deny_unknown_fields)]`, `Serialize/Deserialize`, `Clone, Debug, PartialEq, Eq`):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiSource { Ai, Fallback }

pub struct AiMeta { pub model: String, pub prompt: String, pub input_tokens: u32, pub output_tokens: u32 }

pub struct CriterionVerdict { pub n: u8, pub met: bool, pub reason: String }

pub struct ChatMessageSent { pub hotel_id: HotelId, pub month: MonthCode, pub situation: String, pub text: String }

pub struct CharacterReplied {
    pub hotel_id: HotelId, pub month: MonthCode, pub situation: String,
    /// Redni broj studentove poruke na koju lik odgovara (1..=6).
    pub reply_to: u8,
    pub text: String,
    pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
}

pub struct ResponseSubmitted { pub hotel_id: HotelId, pub month: MonthCode, pub situation: String, pub text: String }

pub struct VerdictRecorded {
    pub hotel_id: HotelId, pub month: MonthCode, pub situation: String,
    /// Pet ocjena kriterija (prazno kod neutralne presude).
    pub criteria: Vec<CriterionVerdict>,
    /// Ponuđeni odgovor čije se mehanike primjenjuju; `None` = neutralna presuda (mehanike `unanswered`).
    pub nearest_answer: Option<u8>,
    pub quality: u8,
    pub rationale: String,
    pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
    /// Na pregled profesoru (neutralna presuda ili AI ispad, ARHITEKTURA §8.3 korak 5).
    pub review: bool,
}

pub struct DebriefRecorded {
    pub hotel_id: HotelId, pub month: MonthCode, pub text: String, pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
}

pub enum GameEvent {
    // … postojećih pet …
    ChatMessageSent(ChatMessageSent),
    CharacterReplied(CharacterReplied),
    ResponseSubmitted(ResponseSubmitted),
    VerdictRecorded(VerdictRecorded),
    /// Presuda koja je stigla nakon neutralne presude ili obračuna: zapisana, ne mijenja obračun (ARHITEKTURA §9.4).
    VerdictLate(VerdictRecorded),
    DebriefRecorded(DebriefRecorded),
}
```

  `kind()` imena: `"ChatMessageSent"`, `"CharacterReplied"`, `"ResponseSubmitted"`, `"VerdictRecorded"`, `"VerdictLate"`,
  `"DebriefRecorded"`; `to_payload`/`from_record` prošireni. Projekcija po hotelu (`game::HotelRecord`):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker { Student, Character }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChatLine { pub from: Speaker, pub text: String }   // Speaker: Serialize, rename_all = "snake_case"

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeResponse { pub text: String, pub verdict: Option<VerdictRecorded>, pub late: Vec<VerdictRecorded> }

pub struct HotelRecord {
    // … postojeće …
    pub chats: BTreeMap<(Month, String), Vec<ChatLine>>,
    pub responses: BTreeMap<(Month, String), FreeResponse>,
    pub debriefs: BTreeMap<Month, DebriefRecorded>,
}
```

  Naredbe (`talk.rs`, sve čiste, vraćaju događaj za upis):

```rust
pub const MAX_CHAT_MESSAGES: u8 = 6;
pub const MAX_TEXT_CHARS: usize = 2000;

pub struct ReplyBody { pub text: String, pub source: AiSource, pub meta: Option<AiMeta> }

impl Game {
    pub fn send_message(&self, hotel: HotelId, code: &str, text: &str) -> Result<GameEvent, GameError>;
    pub fn submit_response(&self, hotel: HotelId, code: &str, text: &str) -> Result<GameEvent, GameError>;
    /// `None` = nema što upisati (runda nije otvorena ili je na tu poruku već odgovoreno).
    pub fn record_reply(&self, hotel: HotelId, code: &str, reply_to: u8, reply: ReplyBody) -> Result<Option<GameEvent>, GameError>;
    /// `None` = mentor za taj mjesec već postoji.
    pub fn record_debrief(&self, hotel: HotelId, m: Month, text: String, source: AiSource, meta: Option<AiMeta>) -> Result<Option<GameEvent>, GameError>;
    pub fn chat(&self, hotel: HotelId, code: &str) -> Result<Vec<ChatLine>, GameError>;
    pub fn situation_month(&self, code: &str) -> Result<Month, GameError>;
}
```

  Nove greške: `GameError::NotFreeText`, `ChatLimit`, `AwaitingReply`, `EmptyText`, `TextTooLong`. U `app/src/error.rs`
  dodaj `ErrorCode` varijante `NotFreeText` (422 `not_free_text`), `ChatLimit` (409 `chat_limit`), `AwaitingReply`
  (409 `awaiting_reply`), `BadText` (422 `bad_text`, za obje tekstualne greške) i proširi `ErrorCode::ALL`, `as_str`,
  statuse i `ProblemCode` istim obrascem kao postojeće (`AlreadyAnswered` je uzor za 409).

- [ ] **Step 1: Napiši testove koji padaju** — `crates/game/tests/talk.rs` (pomoćnik `common::open_june_game()` vraća
  `Game` s otvorenim lipnjem i hotelom igrača `h`; `apply_ev(&mut g, actor, ev)` dodaje zapis sa sljedećim rednim brojem —
  dodaj ih u `tests/common/mod.rs` ako ne postoje):

```rust
#[test]
fn chat_alternates_and_stops_at_six() {
    let (mut g, h) = common::open_june_game();
    for n in 1..=6u8 {
        let ev = g.send_message(h, "S1", &format!("poruka {n}")).unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
        assert_eq!(g.send_message(h, "S1", "još"), Err(GameError::AwaitingReply));
        let reply = ReplyBody { text: format!("odgovor {n}"), source: AiSource::Ai, meta: None };
        let ev = g.record_reply(h, "S1", n, reply).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::System, ev);
    }
    assert_eq!(g.send_message(h, "S1", "sedma"), Err(GameError::ChatLimit));
    assert_eq!(g.chat(h, "S1").unwrap().len(), 12);
}

#[test]
fn second_reply_to_same_message_is_ignored() {
    let (mut g, h) = common::open_june_game();
    let ev = g.send_message(h, "S1", "dobar dan").unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
    let r = || ReplyBody { text: "x".into(), source: AiSource::Ai, meta: None };
    let ev = g.record_reply(h, "S1", 1, r()).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    assert_eq!(g.record_reply(h, "S1", 1, r()).unwrap(), None);
}

#[test]
fn free_text_only_where_content_allows() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.send_message(h, "S3", "bok"), Err(GameError::NotFreeText));
    assert_eq!(g.submit_response(h, "S3", "prihvaćam"), Err(GameError::NotFreeText));
}

#[test]
fn text_bounds() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.submit_response(h, "S1", "   "), Err(GameError::EmptyText));
    let long = "a".repeat(MAX_TEXT_CHARS + 1);
    assert_eq!(g.submit_response(h, "S1", &long), Err(GameError::TextTooLong));
    let ok = "č".repeat(MAX_TEXT_CHARS); // broje se znakovi, ne bajtovi
    assert!(g.submit_response(h, "S1", &ok).is_ok());
}

#[test]
fn free_response_and_offered_answer_exclude_each_other() {
    let (mut g, h) = common::open_june_game();
    let ev = g.submit_response(h, "S1", "Ispričavam se, odmah vas premještam.").unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
    assert_eq!(g.submit_response(h, "S1", "opet"), Err(GameError::AlreadyAnswered));
    assert_eq!(g.choose_answer(h, "S1", 0), Err(GameError::AlreadyAnswered));
    assert_eq!(g.send_message(h, "S1", "još nešto"), Err(GameError::AlreadyAnswered));
    assert_eq!(g.unanswered_count(h, Month::Jun).unwrap(), 1); // ostaje samo S3
}

#[test]
fn closed_round_rejects_talk_and_drops_late_reply() {
    let (mut g, h) = common::open_june_game();
    let ev = g.send_message(h, "S1", "halo").unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
    let ev = g.close_round(Month::Jun).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
    assert_eq!(g.send_message(h, "S1", "još"), Err(GameError::RoundNotOpen));
    let r = ReplyBody { text: "kasno".into(), source: AiSource::Ai, meta: None };
    assert_eq!(g.record_reply(h, "S1", 1, r).unwrap(), None);
}

#[test]
fn new_events_roundtrip_through_payload() {
    let h = HotelId(uuid::Uuid::from_u128(7));
    let meta = AiMeta { model: "claude-opus-5-5".into(), prompt: "presuda.v1".into(), input_tokens: 900, output_tokens: 200 };
    let verdict = VerdictRecorded {
        hotel_id: h, month: MonthCode::Jun, situation: "S1".into(),
        criteria: vec![CriterionVerdict { n: 1, met: true, reason: "isprika".into() }],
        nearest_answer: Some(1), quality: 1, rationale: "obrazloženje".into(),
        source: AiSource::Ai, meta: Some(meta.clone()), review: false,
    };
    let all = vec![
        GameEvent::ChatMessageSent(ChatMessageSent { hotel_id: h, month: MonthCode::Jun, situation: "S1".into(), text: "halo".into() }),
        GameEvent::CharacterReplied(CharacterReplied {
            hotel_id: h, month: MonthCode::Jun, situation: "S1".into(), reply_to: 1,
            text: "Da?".into(), source: AiSource::Fallback, meta: None,
        }),
        GameEvent::ResponseSubmitted(ResponseSubmitted { hotel_id: h, month: MonthCode::Jun, situation: "S1".into(), text: "odgovor".into() }),
        GameEvent::VerdictRecorded(verdict.clone()),
        GameEvent::VerdictLate(verdict),
        GameEvent::DebriefRecorded(DebriefRecorded {
            hotel_id: h, month: MonthCode::Jun, text: "mentor".into(), source: AiSource::Ai, meta: Some(meta),
        }),
    ];
    for e in all {
        assert_eq!(GameEvent::from_record(e.kind(), 1, e.to_payload()).unwrap(), e, "{}", e.kind());
    }
}
```

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p game --test talk; echo EXIT=$?`.

- [ ] **Step 3: Implementiraj događaje** u `event.rs` (gore u Interfaces) i `Speaker`/`ChatLine`/`FreeResponse` u `game.rs`.

- [ ] **Step 4: Implementiraj fold** (`Game::apply`) — pravila (prekršaj = `corrupt(...)`, kao postojeće):
  - `ChatMessageSent`: runda `Open`; situacija u tom mjesecu ima `free_text`; nije odgovorena ni slobodno ni ponuđeno;
    broj studentovih redaka < 6; zadnji redak nije studentov → dodaj `ChatLine { from: Student, .. }`.
  - `CharacterReplied`: runda `Open`; `reply_to` = broj studentovih redaka i zadnji redak je studentov → dodaj redak lika.
  - `ResponseSubmitted`: runda `Open`; `free_text`; nema `AnswerChosen` ni odgovora → `responses.insert`.
  - `VerdictRecorded`: runda `Open` ili `Closing`; odgovor postoji i nema presudu → `verdict = Some`.
  - `VerdictLate`: odgovor postoji i (ima presudu ili je runda `Resolved`) → `late.push`.
  - `DebriefRecorded`: runda `Resolved`; mentor za hotel i mjesec ne postoji → `debriefs.insert`.
  - `AnswerChosen`: dodatno odbij ako postoji slobodan odgovor na tu situaciju.
  `unanswered_count` broji situacije bez ponuđenog **i** bez slobodnog odgovora.

- [ ] **Step 5: Implementiraj naredbe** u `talk.rs`. Redoslijed provjera u `send_message`/`submit_response`:
  hotel postoji → situacija postoji u mjesecima igre (`UnknownSituation`) → runda otvorena (`RoundNotOpen`) →
  `free_text` (`NotFreeText`) → već odgovoreno ponuđeno ili slobodno (`AlreadyAnswered`) → tekst: `trim()` prazan
  (`EmptyText`), `chars().count() > MAX_TEXT_CHARS` (`TextTooLong`) → (samo poruka) zadnji redak studentov
  (`AwaitingReply`), 6 studentovih poruka (`ChatLimit`). Tekst se sprema **trimani**.
  `record_reply`: runda nije `Open` → `None`; `reply_to` ≠ broj studentovih poruka ili zadnji redak nije studentov → `None`.
  `record_debrief`: runda `m` nije `Resolved` → `Err(RoundNotOpen)`; mentor postoji → `None`.
  `choose_answer`: prije provjere `AlreadyAnswered` dodaj provjeru slobodnog odgovora.

- [ ] **Step 6: Feed** — u `feed.rs` nove događaje zasad preskoči (`_ => continue` samo za šest novih varijanti, izričito
  nabrojanih), da se prevodi; prave stavke dolaze u Tasku 5.

- [ ] **Step 7: Pokreni sve provjere** — sve `EXIT=0`.

- [ ] **Step 8: Commit** — „AI sloj 3/11: događaji i pravila razgovora, slobodnog odgovora, presude i mentora".

---

