mod common;

use game::*;

#[test]
fn week_locative_names_both_dates() {
    let g = common::v2_weekly_game_resolved_through("t05");
    let h = common::hotel_ids()[0];
    let input = g.debrief_input(h, RoundKey::Week(5)).unwrap();
    let facts = debrief_facts(&input);
    assert_eq!(facts.month, "u tjednu od 29. lipnja do 5. srpnja");
    assert_eq!(facts.next_month.as_deref(), Some("sljedeći tjedan"));
}

#[test]
fn first_week_compares_with_start_of_game() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let input = g
        .debrief_input(common::hotel_ids()[0], RoundKey::Week(1))
        .unwrap();
    assert!(input.prev_round.is_none());
}

#[test]
fn ai_mentor_only_when_a_month_ends() {
    let g = common::v2_weekly_game_resolved_through("t09");
    let wants: Vec<u8> = (1..=9)
        .filter(|&n| wants_ai_mentor(&g, RoundKey::Week(n)).unwrap())
        .collect();
    assert_eq!(wants, [5, 9]);
    let (v1, _) = common::open_june_game();
    assert!(wants_ai_mentor(&v1, sim_hotel::Month::Jun.into()).unwrap());
}

#[test]
fn weekly_summary_is_three_causes_without_ai_note() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let input = g
        .debrief_input(common::hotel_ids()[0], RoundKey::Week(2))
        .unwrap();
    let text = weekly_summary(&input);
    assert!(
        text.starts_with(
            "U tjednu od 8. lipnja do 14. lipnja je napredak tvog hotela od početka igre "
        ),
        "{text}"
    );
    assert!(text.contains("\nTri uzroka\n"), "{text}");
    assert!(
        text.lines().filter(|l| l.starts_with("Indeks ")).count() <= 3,
        "{text}"
    );
    assert!(
        text.ends_with("Puni pregled s mentorom stiže na kraju mjeseca."),
        "{text}"
    );
    for long in [
        "Što je išlo dobro",
        "Što nije valjalo",
        "Zašto\n",
        "Što probati",
        "AI objašnjenje",
    ] {
        assert!(!text.contains(long), "{long:?} u {text}");
    }
}
