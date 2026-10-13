### Task 2: Sadržaj — rubrike, kriteriji ponuđenih odgovora, likovi i slobodan tekst

**Files:**
- Modify: `crates/content/src/lib.rs`
- Modify: `content/hotel/situations.json` (S1, S2)
- Test: `crates/content/tests/load.rs`, `crates/game/tests/events.rs`

**Interfaces:**
- Consumes: `Snapshot::to_value/from_value/fingerprint` (Task 1).
- Produces (javno u `content`):

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    /// Tko je lik, jednom rečenicom za studenta (npr. „gošća u sobi 214, s dvoje djece").
    pub role: String,
    /// Upute za AI kako lik razmišlja i što skriva (studentu se ne prikazuje).
    pub brief: String,
    /// Prva rečenica lika, napisana (bez AI-ja).
    pub opening: String,
    /// Odgovor lika kad AI ne radi (rezerva, ARHITEKTURA §8.4).
    pub fallback_reply: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SituationAi {
    /// Pet kriterija rubrike (DEMO §7) ili prazno.
    pub rubric: Vec<String>,
    pub character: Option<Character>,
    /// Smije li student odgovoriti svojim riječima (Leonova odluka 1: S1 i S2).
    pub free_text: bool,
    /// Za svaki ponuđeni odgovor (istim redom) brojevi ispunjenih kriterija 1..=5.
    pub criteria: Vec<Vec<u8>>,
}

pub struct Content {
    pub catalog: Catalog,
    pub situations: Vec<SituationTemplate>,
    /// Po šifri situacije; svaka situacija ima unos (prazan ako nema AI-ja).
    pub ai: BTreeMap<String, SituationAi>,
}
```

- [ ] **Step 1: Napiši testove koji padaju** — u `crates/content/tests/load.rs`:

```rust
#[test]
fn s1_and_s2_are_free_text_with_rubric_and_character() {
    let c = content::load(&content::repo_dir()).unwrap();
    for code in ["S1", "S2"] {
        let ai = &c.ai[code];
        assert!(ai.free_text, "{code}");
        assert_eq!(ai.rubric.len(), 5, "{code}");
        assert!(ai.character.is_some(), "{code}");
    }
    assert!(!c.ai["S3"].free_text);
}

#[test]
fn offered_answer_quality_matches_criteria_count() {
    // Ponuđeni odgovori i slobodan tekst su na istoj ljestvici (ARHITEKTURA §10.3): q = max(1, broj kriterija).
    let c = content::load(&content::repo_dir()).unwrap();
    for t in &c.situations {
        let ai = &c.ai[&t.code];
        if ai.rubric.is_empty() { continue; }
        for (a, crit) in t.answers.iter().zip(&ai.criteria) {
            let expected = u8::try_from(crit.len()).unwrap().max(1);
            assert_eq!(a.quality, expected, "{} / {}", t.code, a.label);
        }
    }
}
```

  i testove odbijanja (napiši JSON u privremenu mapu kao postojeći testovi u `load.rs` — koristi njihov pomoćnik s
  `Drop` čuvarom): (a) `free_text: true` bez `character` → `ContentError::Invalid`; (b) rubrika s 4 kriterija →
  `Invalid`; (c) kriterij 6 → `Invalid`; (d) kriterij dvaput u istom odgovoru → `Invalid`; (e) kvaliteta 5 uz kriterije
  `[1,2]` kad rubrika postoji → `Invalid`; (f) broj `criteria` nizova ≠ 0 i ≠ broj odgovora → `Invalid`.

  U `crates/game/tests/events.rs`:

```rust
#[test]
fn old_snapshot_without_ai_fields_still_folds() {
    // Snimak iz koraka 3 nema rubric/character/free_text/criteria; mora se čitati i dati stari otisak.
    let snap = content::Snapshot::read(&content::repo_dir()).unwrap();
    let mut v = snap.to_value();
    for s in v["situations"].as_array_mut().unwrap() {
        let o = s.as_object_mut().unwrap();
        o.remove("rubric"); o.remove("character"); o.remove("free_text");
        for a in o["answers"].as_array_mut().unwrap() { a.as_object_mut().unwrap().remove("criteria"); }
    }
    let old = content::Snapshot::from_value(&v).unwrap();
    let mut events = common::new_game_events(&old);
    let GameEvent::GameCreated(c) = &mut events[0].event else { panic!() };
    c.ruleset_id = old.legacy_fingerprint();
    let g = Game::fold(&events).unwrap();
    assert!(!g.content.ai["S1"].free_text);
}
```

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p content -p game; echo EXIT=$?`.

- [ ] **Step 3: Implementiraj DTO-e** — u `lib.rs`:

```rust
fn is_false(b: &bool) -> bool { !*b }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterDto { name: String, role: String, brief: String, opening: String, fallback_reply: String }
```

  `AnswerDto` dobiva `#[serde(default, skip_serializing_if = "Vec::is_empty")] criteria: Vec<u8>`.
  `SituationDto` dobiva `#[serde(default, skip_serializing_if = "Vec::is_empty")] rubric: Vec<String>`,
  `#[serde(default, skip_serializing_if = "Option::is_none")] character: Option<CharacterDto>`,
  `#[serde(default, skip_serializing_if = "is_false")] free_text: bool`.
  `skip_serializing_if` je **obavezan**: stari snimak mora dati bajt-isti tipizirani zapis (stari otisak).

- [ ] **Step 4: Implementiraj provjeru i gradnju** — `build_situations` vraća i `BTreeMap<String, SituationAi>`.
  Pravila (svaka poruka imenuje situaciju, kao postojeće):
  - `rubric` je prazan ili ima točno 5 nepraznih kriterija;
  - `free_text` traži rubriku i lika; lik traži neprazna sva polja;
  - svaki `criteria` broj je u 1..=5, bez ponavljanja (spremi poredano);
  - kad rubrika postoji: `quality == max(1, criteria.len())` za svaki odgovor (prazni kriteriji → kvaliteta 1);
  - kad rubrika ne postoji: `criteria` mora biti prazan.
  `SituationAi.criteria` ima po jedan niz za svaki odgovor (prazni nizovi kad rubrike nema). `Content` dobiva `ai`.

- [ ] **Step 5: Upiši sadržaj u `content/hotel/situations.json`** — S1 i S2 dobivaju (S3 se ne dira):

  S1 (kriteriji i redoslijed odgovora iz DEMO §7):
```json
"free_text": true,
"rubric": [
  "Brza isprika i preuzimanje odgovornosti, bez prebacivanja krivnje (vrućina, tehničar)",
  "Trenutno rješenje: druga soba ili prijenosni uređaj, odmah",
  "Razmjerna kompenzacija (večera, popust na noćenje), ne povrat cijelog boravka",
  "Uzrok se popravlja i prijavljuje održavanju",
  "Provjera kasnije (follow-up)"
],
"character": {
  "name": "Petra Horvat",
  "role": "gošća u sobi 214, s dvoje male djece",
  "brief": "Umorna si i ljuta: klima ne radi od jutra, vani je 34 °C, djeca ne mogu spavati, a ovo je tvoj treći poziv recepciji. Najviše ti treba hladna soba za djecu ODMAH; novac ti je sporedan. Smiriš se ako se netko iskreno ispriča i odmah ponudi rješenje. Ako ti kažu da je kriva vrućina ili da tehničar dolazi 'kad stigne', postaješ ljuća i spominješ lošu recenziju. Ne nudiš sama rješenja; čekaš da ih ponudi hotel.",
  "opening": "Treći put zovem! Klima u sobi 214 ne radi od jutra, djeca su mokra od znoja. Što ćete poduzeti?",
  "fallback_reply": "Gošća je previše ljuta da bi nastavila razgovor. Odluči što ćeš poduzeti."
}
```
  i `criteria` po odgovorima: `[1,2,3,4,5]`, `[1,2,4]`, `[1]`, `[4]`.

  S2:
```json
"free_text": true,
"rubric": [
  "Saslušati ga i otkriti pravi razlog (plaća, opterećenje, priznanje)",
  "Odluka u okviru budžeta, s obrazloženjem",
  "Pravednost prema ostatku tima: bez presedana koji ruši moral drugih",
  "Nenovčani elementi: raspored, odgovornost, priznanje, razvoj",
  "Plan B: zamjenik ili nasljednik iz tima"
],
"character": {
  "name": "Marko Babić",
  "role": "šef kuhinje, u hotelu sedam sezona",
  "brief": "Konkurentski hotel nudi ti 20 % veću plaću. Novac je važan, ali pravi razlog je da radiš 12 sati dnevno bez slobodnog dana i da te nitko nije pitao za mišljenje o jelovniku tri sezone. Ne kažeš to odmah: prvo tražiš povišicu. Otkriješ pravi razlog tek ako te netko pita kako ti je i sasluša. Ne prihvaćaš prijetnje ni 'nitko nije nezamjenjiv'. Rado bi obučio zamjenika ako dobiš više odgovornosti.",
  "opening": "Šefe, imam ponudu iz hotela preko puta, 20 % više. Ili mi dignete plaću ili odlazim na kraju mjeseca.",
  "fallback_reply": "Marko je otišao u kuhinju; razgovor se nastavlja drugi put. Odluči što ćeš učiniti."
}
```
  i `criteria`: `[1,2,3,4,5]`, `[2,5]`, `[1]`, `[]` (zadnji odgovor nema `criteria` polje ili ga ima prazno).
  Napomena u ADR-019: imena i ponašanje likova su nacrt za Leonovu provjeru.

- [ ] **Step 6: Pokreni sve provjere** — sve `EXIT=0`. `balance --check` mora proći nepromijenjeno (kvalitete i mehanike
  se ne mijenjaju; mijenja se samo otisak).

- [ ] **Step 7: Commit** — „AI sloj 2/11: rubrike, kriteriji, likovi i slobodan tekst u sadržaju".

---

