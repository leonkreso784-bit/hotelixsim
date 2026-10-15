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

struct Student {
    app: Router,
    deps: app::Deps,
    c: String,
    game: String,
    hotel: String,
}

impl Student {
    async fn start(ai: FakeAi, body: Option<Value>) -> Student {
        let (deps, _) = deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), Arc::new(ai));
        let app = router(AppState { deps: deps.clone() });
        let r = call(&app, "POST", "/v1/demo", None, body).await;
        assert_eq!(r.status, 201, "{:?}", r.body);
        let c = cookie_value(r.set_cookie.as_deref().unwrap());
        let (game, hotel) = (
            r.body["game_id"].as_str().unwrap().into(),
            r.body["hotel_id"].as_str().unwrap().into(),
        );
        Student {
            app,
            deps,
            c,
            game,
            hotel,
        }
    }
    async fn v2(kind: &str) -> Student {
        Self::start(
            FakeAi::good(),
            Some(json!({ "model": "v2", "round_kind": kind })),
        )
        .await
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
    async fn situations(&self) -> Value {
        self.get(&format!("/v1/hotels/{}/situations", self.hotel))
            .await
    }
    async fn feed(&self) -> Vec<Value> {
        self.get(&format!("/v1/hotels/{}/feed", self.hotel)).await["items"]
            .as_array()
            .unwrap()
            .clone()
    }
    async fn close(&self, round: &str) {
        let r = self
            .post(
                &format!("/v1/games/{}/rounds/{round}/close", self.game),
                None,
            )
            .await;
        assert_eq!(r.status, 202, "{round}: {:?}", r.body);
        run_worker(&self.deps).await;
    }
    async fn progress(&self, round: &str) -> i64 {
        let g = self.get(&format!("/v1/games/{}", self.game)).await;
        let me = g["hotels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["hotel_id"] == self.hotel.as_str())
            .unwrap();
        let m = me["months"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["round"] == round)
            .expect("krug obračunat");
        m["progress_hundredths"].as_i64().unwrap()
    }
}

fn items(s: &Value) -> Vec<Value> {
    s["items"].as_array().unwrap().clone()
}

async fn round_effect(kind: &str, round: &str, next: &str, compare_next: bool) {
    let (a, b) = (Student::v2(kind).await, Student::v2(kind).await); // a odgovara na sve, b ni na jednu
    let s = a.situations().await;
    let list = items(&s);
    assert_eq!(
        (s["round"].as_str(), s["open"].as_bool()),
        (Some(round), Some(true))
    );
    assert!(!list.is_empty());
    assert_eq!(s["unanswered"], u64::try_from(list.len()).unwrap());
    assert_eq!(
        b.situations().await["items"],
        s["items"],
        "isto sjeme, iste situacije"
    );
    let mut chosen = Vec::new();
    for it in &list {
        let code = it["code"].as_str().unwrap();
        let r = a
            .post(&a.sit(code, "answer"), Some(json!({ "answer": 0 })))
            .await;
        assert_eq!(r.status, 200, "{code}: {:?}", r.body);
        let shown = items(&r.body)
            .into_iter()
            .find(|i| i["code"] == code)
            .unwrap();
        assert_eq!(
            shown["chosen"], 0,
            "{code}: ekran pokazuje izabrani odgovor"
        );
        chosen.push((
            code.to_owned(),
            it["title"].as_str().unwrap().to_owned(),
            it["answers"][0].clone(),
        ));
    }
    assert_eq!(a.situations().await["unanswered"], 0);
    a.close(round).await;
    b.close(round).await;
    // Idući krug: nove, otvorene, ništa odgovoreno.
    let n = a.situations().await;
    let next_items = items(&n);
    assert_eq!(
        (n["round"].as_str(), n["open"].as_bool()),
        (Some(next), Some(true))
    );
    assert!(
        !next_items.is_empty()
            && next_items
                .iter()
                .all(|i| i["chosen"].is_null() && i["delegated"] == false)
    );
    assert_eq!(n["unanswered"], u64::try_from(next_items.len()).unwrap());
    // Dnevnik: svaki odgovor s oznakom koju je student kliknuo; b ima sve „bez odgovora".
    let fa = a.feed().await;
    for (code, _, label) in &chosen {
        assert!(
            fa.iter().any(|f| f["kind"] == "answer_chosen"
                && f["round"] == round
                && f["situation"] == code.as_str()
                && f["label"] == *label),
            "{code}: nema u dnevniku"
        );
    }
    assert!(
        !fa.iter()
            .any(|f| f["kind"] == "situation_ignored" && f["round"] == round)
    );
    let ignored = b
        .feed()
        .await
        .into_iter()
        .filter(|f| f["kind"] == "situation_ignored" && f["round"] == round)
        .count();
    assert_eq!(ignored, list.len());
    // Objašnjenje kruga spominje situaciju istim naslovom kao popis.
    let ex = a
        .get(&format!(
            "/v1/hotels/{}/months/{round}/explanation",
            a.hotel
        ))
        .await;
    let causes: Vec<String> = ex["indices"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|i| i["drivers"].as_array().unwrap().iter())
        .flat_map(|d| d["lines"].as_array().unwrap().iter())
        .map(|l| l["cause"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        chosen
            .iter()
            .any(|(_, t, _)| causes.iter().any(|c| c.contains(&format!("„{t}“")))),
        "{causes:?}"
    );
    // Napredak: odgovoreni krug vrijedi više od šutnje. Tjedni krug: trošak odgovora pada u Financije odmah, a
    // korist (ugled, zadovoljstvo) stiže od idućeg tjedna (Leonova odluka 15.10., a6-dijagnoza.md "trošak odmah,
    // ugled kasnije"), pa se uspoređuje napredak na kraju IDUĆEG kruga (u njemu nitko ne odgovara). Mjesečni: isti krug.
    let round = if compare_next {
        a.close(next).await;
        b.close(next).await;
        next
    } else {
        round
    };
    let (pa, pb) = (a.progress(round).await, b.progress(round).await);
    assert!(
        pa > pb,
        "{round}: napredak s odgovorima {pa} nije veći od napretka bez odgovora {pb}"
    );
}

#[tokio::test]
async fn weekly_round_answers_show_in_report_and_next_round() {
    round_effect("week", "t01", "t02", true).await;
}

#[tokio::test]
async fn monthly_round_answers_show_in_report_and_next_round() {
    round_effect("month", "jun", "jul", false).await;
}

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
            assert!(
                seen.insert(template.clone()),
                "{round}: predložak {template} dvaput ({code})"
            );
        }
        s.close(round).await;
    }
}

#[tokio::test]
async fn conversation_with_a_character_ends_after_six_messages() {
    let s = Student::v2("month").await;
    let list = items(&s.situations().await);
    let it = list
        .iter()
        .find(|i| {
            i["code"].as_str().unwrap().contains('.')
                && !i["character"].is_null()
                && i["free_text"] == true
        })
        .expect("jun sjemena 42 ima primjerak s likom");
    let code = it["code"].as_str().unwrap();
    let conv = s.sit(code, "conversation");
    let msgs = s.sit(code, "messages");
    let c = s.get(&conv).await;
    assert_eq!(c["remaining"], 6);
    assert_eq!(
        (c["awaiting_reply"].as_bool(), c["open"].as_bool()),
        (Some(false), Some(true))
    );
    assert!(c["lines"].as_array().unwrap().is_empty());
    assert_eq!(c["character_name"], it["character"]["name"]);
    assert!(!c["opening"].as_str().unwrap().is_empty());
    for n in 1..=6u64 {
        let r = s
            .post(&msgs, Some(json!({ "text": format!("Poruka {n}.") })))
            .await;
        assert_eq!(r.status, 202, "{n}: {:?}", r.body);
        assert_eq!(r.body["remaining"], 6 - n, "{n}: {:?}", r.body);
        assert_eq!(r.body["awaiting_reply"], true, "{n}: {:?}", r.body);
        run_worker(&s.deps).await;
        let c = s.get(&conv).await;
        let lines = c["lines"].as_array().unwrap();
        assert_eq!(lines.len(), usize::try_from(2 * n).unwrap(), "{n}");
        let last = lines.last().unwrap();
        assert_eq!(last["from"], "character");
        assert!(!last["text"].as_str().unwrap().is_empty());
        assert_eq!(c["awaiting_reply"], false, "{n}");
    }
    assert_problem(
        &s.post(&msgs, Some(json!({ "text": "Sedma." }))).await,
        409,
        "chat_limit",
    );
    let c = s.get(&conv).await;
    assert_eq!(
        (c["remaining"].as_u64(), c["open"].as_bool()),
        (Some(0), Some(true))
    );
    assert_eq!(c["lines"].as_array().unwrap().len(), 12);
    // Razgovor je zaključen, odgovor svojim riječima i dalje ide.
    let r = s
        .post(
            &s.sit(code, "response"),
            Some(json!({ "text": "Ispričavam se i odmah šaljem tehničara." })),
        )
        .await;
    assert_eq!(r.status, 202, "{:?}", r.body);
    let after = items(&s.situations().await)
        .into_iter()
        .find(|i| i["code"] == code)
        .unwrap();
    assert_eq!(after["response"]["status"], "pending", "{after:?}");
}

#[tokio::test]
async fn v1_game_has_no_leader_so_nothing_reads_as_mandatory() {
    let s = Student::start(FakeAi::good(), None).await;
    let list = items(&s.situations().await);
    assert!(!list.is_empty());
    for it in &list {
        let code = it["code"].as_str().unwrap();
        for k in ["leader", "department", "kind", "weight"] {
            assert!(
                it[k].is_null(),
                "{code}: {k} u v1 igri mora biti null, a je {:?}",
                it[k]
            );
        }
        let r = s.post(&s.sit(code, "delegate"), None).await;
        assert_problem(&r, 409, "situation_mandatory");
    }
}

#[tokio::test]
#[ignore = "REVIZIJA #15 (W-1): „Obvezna” danas tumači web; API za v1 šalje mandatory: true — seli u API, Faza C"]
async fn v1_items_are_not_mandatory_on_the_wire() {
    let s = Student::start(FakeAi::good(), None).await;
    let list = items(&s.situations().await);
    assert!(!list.is_empty());
    for it in &list {
        assert_eq!(it["mandatory"], false, "{}", it["code"]);
    }
}

#[tokio::test]
async fn delegated_situation_shows_grade_two_in_the_report() {
    let s = Student::start(
        FakeAi::refusing(),
        Some(json!({ "model": "v2", "round_kind": "month" })),
    )
    .await;
    let list = items(&s.situations().await);
    let it = list
        .iter()
        .find(|i| i["mandatory"] == false && !i["leader"].is_null())
        .expect("jun ima situaciju koju se smije prepustiti voditelju");
    let (code, title) = (it["code"].as_str().unwrap(), it["title"].as_str().unwrap());
    let r = s.post(&s.sit(code, "delegate"), None).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let shown = items(&r.body)
        .into_iter()
        .find(|i| i["code"] == code)
        .unwrap();
    assert_eq!(shown["delegated"], true);
    assert!(shown["chosen"].is_null());
    s.close("jun").await;
    let me = s.get("/v1/me/hotel").await;
    let mentor = me["private"]["last_month"]["mentor"].as_str().unwrap();
    let line = format!("Situacija „{title}“ prepuštena je voditelju (2/5).");
    assert!(mentor.contains(&line), "{line} nije u mentoru: {mentor}");
    let ignored = s
        .feed()
        .await
        .into_iter()
        .any(|f| f["kind"] == "situation_ignored" && f["situation"] == code);
    assert!(!ignored, "{code}: prepuštena situacija nije bez odgovora");
}
