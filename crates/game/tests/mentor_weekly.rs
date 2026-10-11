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

/// ZP-10: u v2 su „fiksni troškovi" dva različita broja — stalni dio zajedničkih službi (trošak iznad GOP-a) i
/// fiksni troškovi zgrade (najam, osiguranje, porez; ispod GOP-a, iz blagajne). Redak blagajne nosi ime iz Računa
/// dobiti, pa isto ime ne stoji uz dva broja.
#[test]
fn building_fixed_costs_have_their_own_name() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let input = g
        .debrief_input(common::hotel_ids()[0], RoundKey::Week(1))
        .unwrap();
    let facts = debrief_facts(&input);
    let causes = |var: dto::VarCode| -> Vec<String> {
        facts
            .indices
            .iter()
            .flat_map(|i| i.drivers.iter())
            .filter(|d| d.source == DriverSource::Var(var))
            .flat_map(|d| d.changes.iter().map(|c| c.cause.clone()))
            .collect()
    };
    let cash = causes(dto::VarCode::Cash);
    let costs = causes(dto::VarCode::Costs);
    assert!(
        cash.iter()
            .any(|c| c == "fiksni troškovi zgrade (najam, osiguranje, porez)"),
        "{cash:?}"
    );
    assert!(costs.iter().any(|c| c == "fiksni troškovi"), "{costs:?}");
    assert!(
        cash.iter().all(|c| !costs.contains(c)),
        "{cash:?} / {costs:?}"
    );
}

/// Tekst o odlascima u v2 krugu nosi odjele iz računa kruga; zbroj raščlambe je hotelski broj odlazaka.
#[test]
fn real_weekly_departures_name_departments_and_add_up() {
    let g = common::v2_weekly_game_resolved_through("t17");
    let mut with_breakdown = 0;
    for h in common::hotel_ids() {
        for n in 1..=17 {
            let input = g.debrief_input(h, RoundKey::Week(n)).unwrap();
            let hotel_total: i64 = input
                .lines
                .iter()
                .filter(|l| {
                    l.var == game::dto::VarCode::Staff && l.cause == game::dto::CauseDto::Turnover
                })
                .map(|l| l.amount)
                .sum();
            let by_dept: i64 = input
                .department_departures
                .iter()
                .map(|d| i64::from(d.count))
                .sum();
            assert_eq!(by_dept, hotel_total, "tjedan {n}");
            if hotel_total == 0 {
                assert!(input.department_departures.is_empty());
                continue;
            }
            with_breakdown += 1;
            let text = serde_json::to_string(&debrief_facts(&input)).unwrap();
            let first = &input.department_departures[0].name;
            assert!(
                text.contains(&format!("{first} ")) || text.contains(&format!("({first})")),
                "tjedan {n}: {text}"
            );
        }
    }
    assert!(
        with_breakdown > 0,
        "sezona bez ijednog odlaska ne provjerava ništa"
    );
}
