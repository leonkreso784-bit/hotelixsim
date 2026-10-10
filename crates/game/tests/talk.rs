mod common;

use game::feed::FeedItem;
use game::talk::{MAX_TEXT_CHARS, ReplyBody};
use game::*;
use sim_hotel::Month;

#[test]
fn chat_alternates_and_stops_at_six() {
    let (mut g, h) = common::open_june_game();
    for n in 1..=6u8 {
        let ev = g.send_message(h, "S1", &format!("poruka {n}")).unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
        assert_eq!(
            g.send_message(h, "S1", "još"),
            Err(GameError::AwaitingReply)
        );
        let reply = ReplyBody {
            text: format!("odgovor {n}"),
            source: AiSource::Ai,
            meta: None,
        };
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
    let r = || ReplyBody {
        text: "x".into(),
        source: AiSource::Ai,
        meta: None,
    };
    let ev = g.record_reply(h, "S1", 1, r()).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    assert_eq!(g.record_reply(h, "S1", 1, r()).unwrap(), None);
}

#[test]
fn free_text_only_where_content_allows() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.send_message(h, "S3", "bok"), Err(GameError::NotFreeText));
    assert_eq!(
        g.submit_response(h, "S3", "prihvaćam"),
        Err(GameError::NotFreeText)
    );
}

#[test]
fn text_bounds() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.submit_response(h, "S1", "   "), Err(GameError::EmptyText));
    let long = "a".repeat(MAX_TEXT_CHARS + 1);
    assert_eq!(
        g.submit_response(h, "S1", &long),
        Err(GameError::TextTooLong)
    );
    let ok = "č".repeat(MAX_TEXT_CHARS); // broje se znakovi, ne bajtovi
    assert!(g.submit_response(h, "S1", &ok).is_ok());
}

#[test]
fn free_response_and_offered_answer_exclude_each_other() {
    let (mut g, h) = common::open_june_game();
    let ev = g
        .submit_response(h, "S1", "Ispričavam se, odmah vas premještam.")
        .unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
    assert_eq!(
        g.submit_response(h, "S1", "opet"),
        Err(GameError::AlreadyAnswered)
    );
    assert_eq!(g.choose_answer(h, "S1", 0), Err(GameError::AlreadyAnswered));
    assert_eq!(
        g.send_message(h, "S1", "još nešto"),
        Err(GameError::AlreadyAnswered)
    );
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
    let r = ReplyBody {
        text: "kasno".into(),
        source: AiSource::Ai,
        meta: None,
    };
    assert_eq!(g.record_reply(h, "S1", 1, r).unwrap(), None);
}

#[test]
fn new_events_roundtrip_through_payload() {
    let h = HotelId(uuid::Uuid::from_u128(7));
    let meta = AiMeta {
        model: "claude-opus-5-5".into(),
        prompt: "presuda.v1".into(),
        input_tokens: 900,
        output_tokens: 200,
    };
    let verdict = VerdictRecorded {
        hotel_id: h,
        month: RoundKey::Month(Month::Jun),
        situation: "S1".into(),
        criteria: vec![CriterionVerdict {
            n: 1,
            met: true,
            reason: "isprika".into(),
        }],
        nearest_answer: Some(1),
        quality: 1,
        rationale: "obrazloženje".into(),
        source: AiSource::Ai,
        meta: Some(meta.clone()),
        review: false,
    };
    let all = vec![
        GameEvent::ChatMessageSent(ChatMessageSent {
            hotel_id: h,
            month: RoundKey::Month(Month::Jun),
            situation: "S1".into(),
            text: "halo".into(),
        }),
        GameEvent::CharacterReplied(CharacterReplied {
            hotel_id: h,
            month: RoundKey::Month(Month::Jun),
            situation: "S1".into(),
            reply_to: 1,
            text: "Da?".into(),
            source: AiSource::Fallback,
            meta: None,
        }),
        GameEvent::ResponseSubmitted(ResponseSubmitted {
            hotel_id: h,
            month: RoundKey::Month(Month::Jun),
            situation: "S1".into(),
            text: "odgovor".into(),
        }),
        GameEvent::VerdictRecorded(verdict.clone()),
        GameEvent::VerdictLate(verdict),
        GameEvent::DebriefRecorded(DebriefRecorded {
            hotel_id: h,
            month: RoundKey::Month(Month::Jun),
            text: "mentor".into(),
            source: AiSource::Ai,
            meta: Some(meta),
        }),
    ];
    for e in all {
        assert_eq!(
            GameEvent::from_record(e.kind(), 1, e.to_payload()).unwrap(),
            e,
            "{}",
            e.kind()
        );
    }
}

#[test]
fn feed_shows_response_verdict_and_mentor_only_to_owner_stream() {
    let g = common::june_resolved_with_free_verdict_game(4, 0); // + DebriefRecorded za igrača
    let h = common::my_hotel(&g);
    let feed = g.feed(h).unwrap();
    assert!(
        feed.iter()
            .any(|i| matches!(i, FeedItem::Response { situation, .. } if situation == "S1"))
    );
    assert!(feed.iter().any(|i| matches!(
        i,
        FeedItem::Verdict {
            quality: 4,
            neutral: false,
            late: false,
            ..
        }
    )));
    assert!(feed.iter().any(|i| matches!(i, FeedItem::Mentor { .. })));
    // Slobodan odgovor nije "zanemarena situacija".
    assert!(
        !feed.iter().any(
            |i| matches!(i, FeedItem::SituationIgnored { situation, .. } if situation == "S1")
        )
    );
}

#[test]
fn private_view_carries_mentor_of_last_month() {
    let g = common::june_resolved_with_free_verdict_game(4, 0);
    let p = g.private_hotel(common::my_hotel(&g)).unwrap();
    assert!(p.last_month.unwrap().mentor.is_some());
}
