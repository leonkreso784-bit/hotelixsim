### Task A6: Scenariji „što student vidi" na razini API-ja

**Veličina:** M, 4–5 h · **Ovisi o:** A0 (kopija) · **Paralelno s:** A5 (A6 piše samo novu datoteku; najviše 2 agenta
koji prevode Rust) · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Pet scenarija igra kao student kroz HTTP (isti pozivi kao ekran) i čita ono što ekran prikazuje: popis
situacija, razgovor, dnevnik, objašnjenje kruga, napredak, mentor. Danas su zeleni (osim dva izričito označena); svrha im
je da puknu kad buduća izmjena (Faza B: v2 mehanike, broj situacija, miješanje odgovora) pokvari ono što student vidi.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §6 A3 i izvještaj 06 nalaz 7: testovi provjeravaju „napravljeno po kartici",
a nema testa „student otvori krug → vidi X, odgovori Y → idući krug vidi Z"; B1–B5 je našao samo Leon igrajući
(`docs/records/BUGS.md:10-27`). API testovi danas provjeravaju pojedine rute (`crates/api/tests/situations_delegate.rs`,
`hotels.rs:162-179` samo za v1 `S1`), ne tijek kruga.

**Files:**
- Create: `crates/api/tests/student_sees.rs` (jedina trajna izmjena; cilj ≤ 450 redaka)
- Privremeno (probe ugriza, korak 3, vratiti alatom Edit, nikad `git checkout`): `crates/game/src/talk.rs:16`,
  `crates/game/src/resolve.rs:13`, `crates/app/src/views.rs:334`
- Ne dirati: sve ostalo; `crates/api/tests/common/mod.rs` se samo koristi (`call`, `cookie_value`, `run_worker`,
  `assert_problem`, `Resp`).

**Interfaces:**
- Consumes (HTTP, imena polja iz `crates/api/src/schema.rs`, `schema_situations.rs`, `explanation.rs`):
  `POST /v1/demo` (tijelo `{"model":"v2","round_kind":"week"|"month"}` ili bez tijela = v1) → 201, `game_id`, `hotel_id`,
  kolačić; `GET /v1/hotels/{h}/situations` → `SituationsBody { round, open, unanswered, items: [SituationBody { code,
  title, answers, chosen, free_text, character, response, department, kind, weight, mandatory, delegated, leader }] }`;
  `POST …/situations/{code}/answer` `{"answer":n}` → 200 `SituationsBody`; `POST …/delegate` → 200 / 409
  `situation_mandatory`; `GET|POST …/conversation|messages` → `ConversationBody { character_name, opening, lines: [{from,
  text}], remaining, awaiting_reply, open }`, sedma → 409 `chat_limit`; `POST …/response` → 202; `POST
  /v1/games/{g}/rounds/{r}/close` → 202; `GET /v1/games/{g}` → `hotels[].{hotel_id, months[].{round,
  progress_hundredths}}`; `GET /v1/hotels/{h}/feed` → `items[].kind` (`answer_chosen` s `round, situation, label`;
  `situation_ignored`); `GET /v1/hotels/{h}/months/{r}/explanation` → `indices[].drivers[].lines[].cause`;
  `GET /v1/me/hotel` → `private.last_month.mentor`.
  Rust: `app::memory::{FakeAi, deps_with_ai}` (`FakeAi::good()`, `FakeAi::refusing()`), `api::{AppState, router}`.
- Produces: samo testove (nijedan novi javni tip).

**Testovi čuvara** (`crates/api/tests/student_sees.rs`). Pomoćnik, točno:
```rust
//! Scenariji „što student vidi" (REVIZIJA_TEMELJA §6 A3): isti pozivi kao ekran, provjera onoga što ekran prikazuje.
//! Sjeme je uvijek 42 (`SeqIds`), pa dvije aplikacije daju istu igru: usporedba „s odgovorima" i „bez odgovora".
mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use api::{AppState, router};
use app::memory::{FakeAi, deps_with_ai};
use axum::Router;
use common::{Resp, assert_problem, call, cookie_value, run_worker};
use content::{Snapshot, repo_dir};
use serde_json::{Value, json};

struct Student { app: Router, deps: app::Deps, c: String, game: String, hotel: String }

impl Student {
    async fn start(ai: FakeAi, body: Option<Value>) -> Student {
        let (deps, _) = deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), Arc::new(ai));
        let app = router(AppState { deps: deps.clone() });
        let r = call(&app, "POST", "/v1/demo", None, body).await;
        assert_eq!(r.status, 201, "{:?}", r.body);
        let c = cookie_value(r.set_cookie.as_deref().unwrap());
        let (game, hotel) = (r.body["game_id"].as_str().unwrap().into(), r.body["hotel_id"].as_str().unwrap().into());
        Student { app, deps, c, game, hotel }
    }
    async fn v2(kind: &str) -> Student {
        Self::start(FakeAi::good(), Some(json!({ "model": "v2", "round_kind": kind }))).await
    }
    async fn get(&self, uri: &str) -> Value {
        let r = call(&self.app, "GET", uri, Some(&self.c), None).await;
        assert_eq!(r.status, 200, "{uri}: {:?}", r.body);
        r.body
    }
    async fn post(&self, uri: &str, body: Option<Value>) -> Resp {
        call(&self.app, "POST", uri, Some(&self.c), body).await
    }
    fn sit(&self, code: &str, tail: &str) -> String {
        format!("/v1/hotels/{}/situations/{code}/{tail}", self.hotel)
    }
    async fn situations(&self) -> Value { self.get(&format!("/v1/hotels/{}/situations", self.hotel)).await }
    async fn feed(&self) -> Vec<Value> {
        self.get(&format!("/v1/hotels/{}/feed", self.hotel)).await["items"].as_array().unwrap().clone()
    }
    async fn close(&self, round: &str) {
        let r = self.post(&format!("/v1/games/{}/rounds/{round}/close", self.game), None).await;
        assert_eq!(r.status, 202, "{round}: {:?}", r.body);
        run_worker(&self.deps).await;
    }
    async fn progress(&self, round: &str) -> i64 {
        let g = self.get(&format!("/v1/games/{}", self.game)).await;
        let me = g["hotels"].as_array().unwrap().iter().find(|h| h["hotel_id"] == self.hotel.as_str()).unwrap();
        let m = me["months"].as_array().unwrap().iter().find(|m| m["round"] == round).expect("krug obračunat");
        m["progress_hundredths"].as_i64().unwrap()
    }
}

fn items(s: &Value) -> Vec<Value> { s["items"].as_array().unwrap().clone() }
```
**(a) Cijeli krug: odgovor → zatvaranje → izvješće i idući krug.** `weekly_round_answers_show_in_report_and_next_round`
= `round_effect("week", "t01", "t02")`; `monthly_round_answers_show_in_report_and_next_round` = `round_effect("month",
"jun", "jul")`:
```rust
async fn round_effect(kind: &str, round: &str, next: &str) {
    let (a, b) = (Student::v2(kind).await, Student::v2(kind).await); // a odgovara na sve, b ni na jednu
    let s = a.situations().await;
    let list = items(&s);
    assert_eq!((s["round"].as_str(), s["open"].as_bool()), (Some(round), Some(true)));
    assert!(!list.is_empty());
    assert_eq!(s["unanswered"], u64::try_from(list.len()).unwrap());
    assert_eq!(b.situations().await["items"], s["items"], "isto sjeme, iste situacije");
    let mut chosen = Vec::new();
    for it in &list {
        let code = it["code"].as_str().unwrap();
        let r = a.post(&a.sit(code, "answer"), Some(json!({ "answer": 0 }))).await;
        assert_eq!(r.status, 200, "{code}: {:?}", r.body);
        let shown = items(&r.body).into_iter().find(|i| i["code"] == code).unwrap();
        assert_eq!(shown["chosen"], 0, "{code}: ekran pokazuje izabrani odgovor");
        chosen.push((code.to_owned(), it["title"].as_str().unwrap().to_owned(), it["answers"][0].clone()));
    }
    assert_eq!(a.situations().await["unanswered"], 0);
    a.close(round).await;
    b.close(round).await;
    // Idući krug: nove, otvorene, ništa odgovoreno.
    let n = a.situations().await;
    let next_items = items(&n);
    assert_eq!((n["round"].as_str(), n["open"].as_bool()), (Some(next), Some(true)));
    assert!(!next_items.is_empty() && next_items.iter().all(|i| i["chosen"].is_null() && i["delegated"] == false));
    assert_eq!(n["unanswered"], u64::try_from(next_items.len()).unwrap());
    // Dnevnik: svaki odgovor s oznakom koju je student kliknuo; b ima sve „bez odgovora".
    let fa = a.feed().await;
    for (code, _, label) in &chosen {
        assert!(fa.iter().any(|f| f["kind"] == "answer_chosen" && f["round"] == round
            && f["situation"] == code.as_str() && f["label"] == *label), "{code}: nema u dnevniku");
    }
    assert!(!fa.iter().any(|f| f["kind"] == "situation_ignored" && f["round"] == round));
    let ignored = b.feed().await.into_iter().filter(|f| f["kind"] == "situation_ignored" && f["round"] == round).count();
    assert_eq!(ignored, list.len());
    // Objašnjenje kruga spominje situaciju istim naslovom kao popis.
    let ex = a.get(&format!("/v1/hotels/{}/months/{round}/explanation", a.hotel)).await;
    let causes: Vec<String> = ex["indices"].as_array().unwrap().iter()
        .flat_map(|i| i["drivers"].as_array().unwrap().iter())
        .flat_map(|d| d["lines"].as_array().unwrap().iter())
        .map(|l| l["cause"].as_str().unwrap().to_owned()).collect();
    assert!(chosen.iter().any(|(_, t, _)| causes.iter().any(|c| c.contains(&format!("„{t}“")))), "{causes:?}");
    // Napredak: odgovoreni krug vrijedi više od šutnje.
    let (pa, pb) = (a.progress(round).await, b.progress(round).await);
    assert!(pa > pb, "{round}: napredak s odgovorima {pa} nije veći od napretka bez odgovora {pb}");
}
```
**(b) Ista situacija najviše jednom u mjesečnom krugu** — B3 **nije** popravljen na `sloj1-l111` (`occurrence`,
`crates/sim-hotel-v2/src/situations/draw.rs:181-184`, daje `….jul.2`), pa:
```rust
#[tokio::test]
#[ignore = "B3, ADR-042 t. 12, Faza B: isti predložak dvaput u mjesečnom krugu (P-PRAONICA.jul i .jul.2)"]
async fn monthly_round_never_repeats_a_template() {
    let s = Student::v2("month").await;
    for round in ["jun", "jul", "aug", "sep"] {
        let list = s.situations().await;
        assert_eq!(list["round"], round);
        let mut seen = BTreeSet::new();
        for it in items(&list) {
            let code = it["code"].as_str().unwrap();
            let template = code.split('.').next().unwrap().to_owned();
            assert!(seen.insert(template.clone()), "{round}: predložak {template} dvaput ({code})");
        }
        s.close(round).await;
    }
}
```
**(c) Razgovor završava nakon 6 studentovih poruka** — `conversation_with_a_character_ends_after_six_messages`:
`Student::v2("month")`; prvi `item` s `code` koji sadrži `'.'`, `character` nije `null` i `free_text == true`
(`expect("jun sjemena 42 ima primjerak s likom")`). `GET conversation` → `remaining == 6`, `awaiting_reply == false`,
`open == true`, `lines` prazno, `character_name == item.character.name`, `opening` neprazan. Za `n in 1..=6u64`:
`POST messages {"text": format!("Poruka {n}.")}` → 202, `remaining == 6 - n`, `awaiting_reply == true`; `run_worker`;
`GET conversation` → `lines.len() == 2 * n`, zadnji `from == "character"` i neprazan `text`, `awaiting_reply == false`.
Zatim `POST messages "Sedma."` → `assert_problem(…, 409, "chat_limit")`; `GET conversation` → `remaining == 0`,
`open == true`, 12 redaka. Razgovor je zaključen, odgovor svojim riječima i dalje ide: `POST response {"text":
"Ispričavam se i odmah šaljem tehničara."}` → 202, a stavka `code` u tijelu ima `response.status == "pending"`.

**(d) v1 igra nema voditelja, pa ništa ne čita kao „Obvezna" (W-1)** — `v1_game_has_no_leader_so_nothing_reads_as_mandatory`:
`Student::start(FakeAi::good(), None)`; `items` neprazan; za svaku stavku `leader`, `department`, `kind`, `weight` su
`null` (web prikazuje „Obvezna" samo uz `mandatory && leader !== null`, `apps/web/src/screens/situation/rules.ts:17-23`),
a `POST …/delegate` → `assert_problem(…, 409, "situation_mandatory")`. Drugi test, isto postavljanje, tvrdi istinu na žici:
```rust
#[tokio::test]
#[ignore = "REVIZIJA #15 (W-1): „Obvezna” danas tumači web; API za v1 šalje mandatory: true — seli u API, Faza C"]
async fn v1_items_are_not_mandatory_on_the_wire() { /* za svaku stavku: assert_eq!(it["mandatory"], false, "{code}") */ }
```
**(e) Prepuštanje voditelju daje ocjenu 2 i vidi se u izvješću** — `delegated_situation_shows_grade_two_in_the_report`:
`Student::start(FakeAi::refusing(), Some(json!({"model":"v2","round_kind":"month"})))` (odbijanje → mentor odmah iz brojki,
`crates/app/src/jobs.rs:122-124`); prva stavka s `mandatory == false` i `leader` nije `null` (`expect`); `POST …/delegate`
→ 200, ta stavka `delegated == true`, `chosen` `null`; `close("jun")`; `GET /v1/me/hotel` → `private.last_month.mentor`
sadrži `format!("Situacija „{title}“ prepuštena je voditelju (2/5).")` (`crates/game/src/mentor.rs:1258-1263`); dnevnik
nema `situation_ignored` za taj `code` (prepuštena nije „bez odgovora").

**Zašto danas nisu crveni:** (a), (c), (d), (e) opisuju današnje ponašanje (karakterizacijski testovi); ugriz se dokazuje
probom u koraku 3. (b) i `v1_items_are_not_mandatory_on_the_wire` su `#[ignore]` s razlogom: druga je danas sigurno crvena
(`left: Bool(true)`), (b) je crvena samo ako sjeme 42 izvuče ponavljanje — ishod se zapisuje.

**Koraci (čuvar; graditelj nema što graditi):**
1. Napiši datoteku (pomoćnik i (a), (b) doslovno; (c)–(e) točno po opisu, isti stil). `cargo fmt`.
2. `cargo test -p api --test student_sees; echo EXIT=$?` → `test result: ok. 5 passed; 0 failed; 2 ignored`.
   Ako (a) padne na `pa > pb` ili na naslovu u objašnjenju: **ne slabi tvrdnju** — to je nalaz o igri; stani, zapiši
   brojke i javi kontroloru (NEEDS_CONTEXT).
3. Probe ugriza, svaka zasebno (izmjena → test → vrati Editom → `git diff --stat` bez te datoteke):
   `talk.rs:16` `= 6` → `= 7` ⇒ (c) crven (`remaining` 6 ≠ 7); `resolve.rs:13` `= 2` → `= 3` ⇒ (e) crven (`(3/5)`);
   `views.rs:334` `chosen: record.answers.get(&key).copied()` → `chosen: None` ⇒ (a) crven („ekran pokazuje izabrani
   odgovor"). Zapiši tri crvena izlaza u izvještaj.
4. `cargo test -p api --test student_sees -- --ignored; echo EXIT=$?` → zapiši ishod oba (očekivano: W-1 crven; B3 crven ili
   zelen za sjeme 42).

**Provjere (točne naredbe i očekivani izlaz):** svaka zasebno, `; echo EXIT=$?`.
- `cargo test -p api --test student_sees` → `5 passed; 0 failed; 2 ignored`
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test -p api` → EXIT=0
- `corepack pnpm check:kod` → EXIT=0 (nova datoteka < 600 redaka, bez upozorenja)
- `git diff --stat` = samo `crates/api/tests/student_sees.rs` (probe vraćene); balans i jezgra nedirnuti, pa se ne vrte.
- Commit „Popravak A6: scenariji „što student vidi" na razini API-ja (tjedni i mjesečni krug, razgovor, v1, prepuštanje)".

**Rizik i odluke:**
- 🟨 (a) bira odgovor 0 jer je danas najbolji u svih 55 predložaka (B1). Kad Faza B (ADR-042 t. 10) izmiješa prikaz ili
  B4 prepiše oznake, ta kartica u istom commitu mijenja (a) da bira po izvornom broju najboljeg odgovora — test će tada
  pasti i to je namjera, ne kvar.
- 🟨 (e) prikuje „(2/5)" i rečenicu rezerve mentora; ADR-042 t. 13–14 i REVIZIJA #13 (ocjena prepuštanja u postavkama)
  mijenjaju je svjesno, uz izmjenu testa.
- Brojevi iz ADR-042 t. 13 (mjesečno 4/5/6) neće srušiti scenarije: nijedan ne broji situacije, (e) treba ≥ 4 stavke.
- (b) sa sjemenom 42 vidi samo jednu igru; pravi čuvar B3 u Fazi B je test jezgre preko mnogo sjemena (`content`/`sim-hotel-v2`).
- Dvije aplikacije dijele sjeme i identifikatore (`SeqIds`), pa su igre iste do prvog studentova poteza; provjeravatelj
  neka potvrdi da (a) ne ovisi o redoslijedu testova (svaki ima svoju memoriju).

**Leonov dio:** nema (ništa u igri se ne mijenja).

---

## Review Focus kandidati
1. **Tjedan preko granice mjeseca** (t05, 29. 6.–5. 7., mentor s AI-jem i obnova): nijedan scenarij ne prolazi t05, a
   tamo se miješaju tjedni sažetak, mjesečni mentor i plaćanje obnove.
2. **Slobodan odgovor i presuda** (presuda bira najbliži odgovor → trošak u obračunu): (c) staje na `pending`; nitko ne
   provjerava da ocjena iz presude stigne u izvješće i dnevnik (`verdict`) onako kako je student vidi na kartici situacije.
3. **Stara v2 igra nakon promjene jezgre** (409 `game_outdated` već pri čitanju, REVIZIJA #6): scenariji uvijek počinju
   novu igru, pa ne vide što student s igrom u tijeku dobije nakon popravka iz Faze B.
