### Task 5: `game` — pogledi i feed (situacije s likom i odgovorom, mentor)

**Files:**
- Modify: `crates/game/src/view.rs`, `crates/game/src/feed.rs`
- Test: `crates/game/tests/talk.rs` (dodaci)

**Interfaces:**
- Produces:

```rust
// view.rs
pub struct LastMonth { /* postojeće */ pub mentor: Option<String> }

// feed.rs — nove varijante FeedItem (postojeći obrazac: mjesec, naslov situacije …)
FeedItem::Response { month: MonthCode, situation: String, title: String },
FeedItem::Verdict { month: MonthCode, situation: String, title: String, quality: u8, neutral: bool, late: bool },
FeedItem::Mentor { month: MonthCode, text: String },
```

  Razgovor (`ChatMessageSent`/`CharacterReplied`) **ne** ulazi u feed (dio je situacije). `SituationResult.free_text` u
  postojećoj stavci obračuna situacije prikazuje „tvoj odgovor (ocjena AI-ja)" umjesto „bez odgovora".

- [ ] **Step 1: Napiši testove koji padaju** —

```rust
#[test]
fn feed_shows_response_verdict_and_mentor_only_to_owner_stream() {
    let g = common::june_resolved_with_free_verdict_game(4, 0); // + DebriefRecorded za igrača
    let h = common::my_hotel(&g);
    let feed = g.feed(h).unwrap();
    assert!(feed.iter().any(|i| matches!(i, FeedItem::Response { situation, .. } if situation == "S1")));
    assert!(feed.iter().any(|i| matches!(i, FeedItem::Verdict { quality: 4, neutral: false, late: false, .. })));
    assert!(feed.iter().any(|i| matches!(i, FeedItem::Mentor { .. })));
}

#[test]
fn private_view_carries_mentor_of_last_month() {
    let g = common::june_resolved_with_free_verdict_game(4, 0);
    let p = g.private_hotel(common::my_hotel(&g)).unwrap();
    assert!(p.last_month.unwrap().mentor.is_some());
}
```

  `june_resolved_with_free_verdict_game` dodaj u `tests/common` (kao Task 4 pomoćnik, plus `record_debrief`).

- [ ] **Step 2: Pokreni i provjeri da pada.**
- [ ] **Step 3: Implementiraj** — u `feed.rs` zamijeni privremeno preskakanje iz Taska 3 stvarnim stavkama (samo za
  hotel čiji se feed traži, kao postojeće stavke); `CharacterReplied`/`ChatMessageSent` ostaju izričito preskočeni.
- [ ] **Step 4: Pokreni sve provjere** — sve `EXIT=0`.
- [ ] **Step 5: Commit** — „AI sloj 5/11: feed i pogledi za slobodan odgovor, presudu i mentora".

---

