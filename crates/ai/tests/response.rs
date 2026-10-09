const OK: &str = r#"{"model":"claude-opus-5-5","content":[{"type":"thinking","thinking":""},{"type":"text","text":"{\"criteria\":[{\"n\":1,\"met\":true,\"reason\":\"isprika\"},{\"n\":2,\"met\":true,\"reason\":\"soba\"},{\"n\":3,\"met\":false,\"reason\":\"nema\"},{\"n\":4,\"met\":true,\"reason\":\"tehničar\"},{\"n\":5,\"met\":false,\"reason\":\"nema\"}],\"nearest_answer\":1,\"rationale\":\"Dobra isprika.\"}"}],"stop_reason":"end_turn","usage":{"input_tokens":900,"output_tokens":210}}"#;

#[test]
fn ok_reply_parses_into_verdict_draft() {
    let r = ai::response::parse_reply(200, OK).unwrap();
    assert_eq!((r.input_tokens, r.output_tokens), (900, 210));
    let d = ai::response::parse_verdict(&r).unwrap();
    assert_eq!(d.nearest_answer, 1);
    assert_eq!(d.criteria.iter().filter(|c| c.met).count(), 3);
}

#[test]
fn refusal_max_tokens_http_errors_and_garbage_are_errors() {
    use app::AiError::*;
    let refusal = r#"{"model":"m","content":[],"stop_reason":"refusal","stop_details":{"type":"refusal","category":null,"explanation":"x"},"usage":{"input_tokens":1,"output_tokens":0}}"#;
    assert!(matches!(
        ai::response::parse_reply(200, refusal),
        Err(Refused(_))
    ));
    let cut = r#"{"model":"m","content":[{"type":"text","text":"{\"crit"}],"stop_reason":"max_tokens","usage":{"input_tokens":1,"output_tokens":9}}"#;
    assert!(matches!(
        ai::response::parse_reply(200, cut),
        Err(BadOutput(_))
    ));
    assert!(matches!(
        ai::response::parse_reply(529, "{}"),
        Err(Unavailable(_))
    ));
    assert!(matches!(
        ai::response::parse_reply(401, "{}"),
        Err(Unavailable(_))
    ));
    let prose = r#"{"model":"m","content":[{"type":"text","text":"Mislim da je dobro."}],"stop_reason":"end_turn","usage":{"input_tokens":1,"output_tokens":5}}"#;
    let r = ai::response::parse_reply(200, prose).unwrap();
    assert!(matches!(ai::response::parse_verdict(&r), Err(BadOutput(_))));
}
