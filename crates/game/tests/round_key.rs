use game::RoundKey;
use sim_hotel::Month;

#[test]
fn month_keys_print_and_parse_like_old_month_codes() {
    for m in Month::SEASON {
        let k = RoundKey::from(m);
        assert_eq!(k.code(), m.code());
        assert_eq!(RoundKey::parse(m.code()), Some(k));
        assert_eq!(
            serde_json::to_string(&k).unwrap(),
            format!("\"{}\"", m.code())
        );
    }
}

#[test]
fn week_keys_print_and_parse() {
    assert_eq!(RoundKey::Week(7).code(), "t07");
    assert_eq!(RoundKey::parse("t01"), Some(RoundKey::Week(1)));
    assert_eq!(RoundKey::parse("t17"), Some(RoundKey::Week(17)));
    assert_eq!(
        serde_json::from_str::<RoundKey>("\"t03\"").unwrap(),
        RoundKey::Week(3)
    );
}

#[test]
fn bad_codes_are_rejected() {
    for bad in ["t00", "t18", "t7", "T07", "Jun", "jun ", "", "t1x", "week"] {
        assert_eq!(RoundKey::parse(bad), None, "{bad:?}");
        assert!(serde_json::from_str::<RoundKey>(&format!("\"{bad}\"")).is_err());
    }
}

#[test]
fn keys_order_by_season() {
    assert!(RoundKey::from(Month::Jun) < RoundKey::from(Month::Jul));
    assert!(RoundKey::from(Month::Aug) < RoundKey::from(Month::Sep));
    assert!(RoundKey::Week(2) < RoundKey::Week(10));
}
