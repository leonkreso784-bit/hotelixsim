use game::debrief_has_parts;

#[test]
fn round_mentor_parts_are_checked() {
    let ok =
        "Što je išlo dobro\nA.\n\nŠto nije valjalo\nB.\n\nZašto\nC.\n\nŠto probati u kolovozu\nD.";
    assert!(debrief_has_parts(ok));
    assert!(debrief_has_parts(
        &ok.replace("u kolovozu", "sljedeći tjedan")
    ));
    assert!(!debrief_has_parts("Mentor: dobar mjesec."));
    assert!(!debrief_has_parts("  \n"));
    assert!(
        !debrief_has_parts(&ok.replace("Zašto\n", "")),
        "bez dijela Zašto"
    );
    assert!(!debrief_has_parts(&ok.replace(
        "Što probati u kolovozu",
        "Što probati ne znam jer je bilo puno toga"
    )));
}
