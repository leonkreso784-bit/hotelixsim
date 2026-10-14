//! G-2 popravak 1 (I1, I2): posao razgovora i pogled presude rade i za IZVUČENI PRIMJERAK situacije (v2),
//! čiji lik i rubrika žive u `ai_entry_for(hotel, šifra)`, ne u `content.ai[šifra]` (tamo ih za primjerak nema).
//!
//! PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
//! - v2 mjesečna igra (`NewGameSettings { model: V2, kind: Some(Month) }`, sjeme 42 iz `SeqIds`) u prvom krugu ima
//!   primjerak s likom (`SituationItem.free_text` i `character` postavljeni, šifra s točkom, npr. `P-ALERGIJA.jun`).
//! - Rezervni odgovor lika je popunjen `fallback_reply` primjerka: neprazan, bez `{`/`}`.
//! - Rubrika primjerka u pogledu: 5 kriterija s nepraznim tekstom bez `{`/`}`.

use std::sync::Arc;

use app::memory::FakeAi;
use app::{AiPort, Caller, Deps, GameId, HotelId, ResponseStatus, SituationItem};
use content::{Model, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

async fn start_v2(ai: Arc<dyn AiPort>) -> (Deps, Caller, HotelId, GameId) {
    let (deps, _mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai);
    let settings = app::NewGameSettings {
        model: Model::V2,
        kind: Some(PeriodKind::Month),
    };
    let d = app::start_demo(&deps, None, settings).await.unwrap();
    let caller = app::caller(&deps, d.new_token.as_deref()).await.unwrap();
    (deps, caller, d.hotel_id, d.game_id)
}

/// Primjerak (šifra s točkom) s likom u prvom krugu.
async fn instance_with_character(deps: &Deps, caller: &Caller, hotel: HotelId) -> SituationItem {
    let s = app::situations(deps, caller, hotel).await.unwrap();
    s.items
        .into_iter()
        .find(|i| i.code.contains('.') && i.free_text && i.character.is_some())
        .expect("prvi krug sjemena 42 ima primjerak s likom")
}

async fn run_all_jobs(deps: &Deps) {
    for _ in 0..60 {
        if let Ok(false) = app::run_next_job(deps).await {
            return;
        }
    }
    panic!("red se nije ispraznio u 60 koraka");
}

fn assert_clean(what: &str, s: &str) {
    assert!(!s.trim().is_empty(), "{what}: prazno");
    assert!(
        !s.contains('{') && !s.contains('}'),
        "{what}: neispunjeno mjesto u: {s}"
    );
}

/// I1: bez AI-ja (odbijanje, bez ponavljanja) posao razgovora za primjerak upiše rezervni odgovor lika i otključa razgovor.
#[tokio::test]
async fn refused_ai_chat_on_an_instance_writes_fallback_reply_and_unlocks() {
    let (deps, caller, hotel, _) = start_v2(Arc::new(FakeAi::refusing())).await;
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
    run_all_jobs(&deps).await;
    let v = app::conversation(&deps, &caller, hotel, &item.code)
        .await
        .unwrap();
    assert!(
        !v.awaiting_reply,
        "{}: razgovor ostao zaključan (posao pao bez rezervnog odgovora)",
        item.code
    );
    assert_eq!(v.lines.len(), 2, "{}", item.code);
    assert_clean("rezervni odgovor lika", &v.lines[1].text);
    // student smije pisati dalje
    app::send_message(&deps, &caller, hotel, &item.code, "Još nešto.".into())
        .await
        .unwrap();
}

/// I1 (ispad AI-ja): isti ishod nakon zadnjeg pokušaja.
#[tokio::test]
async fn failing_ai_chat_on_an_instance_falls_back_on_last_attempt() {
    let (deps, caller, hotel, _) = start_v2(Arc::new(FakeAi::failing())).await;
    let item = instance_with_character(&deps, &caller, hotel).await;
    app::send_message(&deps, &caller, hotel, &item.code, "Halo?".into())
        .await
        .unwrap();
    run_all_jobs(&deps).await;
    let v = app::conversation(&deps, &caller, hotel, &item.code)
        .await
        .unwrap();
    assert!(!v.awaiting_reply, "{}: razgovor ostao zaključan", item.code);
    assert_clean("rezervni odgovor lika", &v.lines.last().unwrap().text);
}

/// I2: presuda na primjerak u pogledu ima 5 kriterija s tekstom rubrike (ne prazne nizove).
#[tokio::test]
async fn judged_instance_shows_five_criteria_with_rubric_text() {
    let (deps, caller, hotel, _) = start_v2(Arc::new(FakeAi::good())).await;
    let item = instance_with_character(&deps, &caller, hotel).await;
    app::submit_response(
        &deps,
        &caller,
        hotel,
        &item.code,
        "Ispričat ću se i ponuditi rješenje.".into(),
    )
    .await
    .unwrap();
    run_all_jobs(&deps).await;
    let s = app::situations(&deps, &caller, hotel).await.unwrap();
    let r = s
        .items
        .iter()
        .find(|i| i.code == item.code)
        .unwrap()
        .response
        .clone()
        .unwrap();
    assert_eq!(r.status, ResponseStatus::Judged);
    assert_eq!(r.criteria.len(), 5, "{}", item.code);
    for c in &r.criteria {
        assert_clean(&format!("{} kriterij {}", item.code, c.n), &c.text);
    }
}
