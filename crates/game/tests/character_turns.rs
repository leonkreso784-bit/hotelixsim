mod common;
use game::*;

#[test]
fn character_input_counts_student_messages_up_to_the_limit() {
    let (mut g, h) = common::open_june_game();
    for n in 1..=6u8 {
        let ev = g.send_message(h, "S1", &format!("poruka {n}")).unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::guest()), ev);
        let ci = g.character_input(h, "S1").unwrap();
        assert_eq!(
            (ci.message_no, ci.message_max),
            (n, MAX_CHAT_MESSAGES),
            "poruka {n}"
        );
        let reply = ReplyBody {
            text: format!("odgovor {n}"),
            source: AiSource::Ai,
            meta: None,
        };
        let ev = g.record_reply(h, "S1", n, reply).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::System, ev);
    }
}
