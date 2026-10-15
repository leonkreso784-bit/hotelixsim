<!-- Nastavak kartice [B1](task-08-b1-v2-mehanike-situacija.md) (B1a), odvojen zbog granice veličine. Sažetak i ovisnosti: u toj kartici i u planu. -->

## B1b: uvjet stanja „čijeg prostora" (`condition_of`)

**Veličina:** S–M, 3–4 h · **Ovisi o:** B1a (spojen) · **Slijedi ga:** B4a–c (B4 ga **troši**, pa B4 ide nakon B1b). B2a/B2b su već spojeni (red B2 → B3 → B1), pa nema sukoba u
`situations/draw.rs` · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Predložak smije reći čiji prostor gleda: `condition_of: "rooms"`. Tada `dept_condition_pts` (u `when` i pojačanjima)
čita stanje tog odjela, a popravak ide u taj odjel. Dopušteno je i na predlošku cijelog hotela (P-DIZALO). Načelo
`BreakdownCondition` priznaje uvjet samo na prostoru koji se troši. Brojke se ne mijenjaju: nijedan predložak još nema polje,
a B4 ga upisuje za P-BRAVE, P-CURENJE-VODE i P-DIZALO po ADR-042 t. 7.

**Files:**
- Modify:
  - `crates/content/src/templates.rs:92-95`: novo polje uz `external_cause`, `#[serde(default, skip_serializing_if = "Option::is_none")] pub condition_of: Option<String>`.
  - `crates/content/src/templates_check.rs:140` i `:154`: `check_conditions` dobiva `has_space = depts.is_some() || t.condition_of.is_some()`.
  - `crates/content/src/templates_check.rs:288-322`: za `dept_condition_pts` vrijedi `has_space`, za ostale `dept_*` i dalje `has_dept`.
  - `crates/content/src/templates_check.rs`, nova provjera u `check_template`: poznat odjel iz `v2.departments`, koji nije neraspoređen.
  - `crates/content/src/templates_core.rs:44-111`: `condition_of`.
  - `crates/content/src/templates_instance.rs:214-222`: `SituationTemplates::condition_of(&self, code) -> Option<DeptCode>`, obrazac kao `no_penalty`.
  - `crates/content/src/templates_principles.rs:164-173`: uvjet vrijedi samo ako se prostor troši. Prostor je
    `condition_of`, inače odjel predloška ili svi članovi skupine; nijedan ne smije biti neraspoređen, a `hotel` bez
    `condition_of` nema prostor.
  - `crates/sim-hotel-v2/src/situations/template.rs:24-31, 43-62`: `pub condition_of: Option<DeptCode>` i `TemplateSet::condition_of`.
  - `crates/sim-hotel-v2/src/situations/eligible.rs:75-134` i `draw.rs:189-217` (`candidates`), `:222-235` (`collective`: prostor `None`).
  - `crates/game/src/drawn.rs` (`Game::space`), `crates/sim-hotel-v2/src/season.rs:333`: prostor = `condition_of`, inače odjel primjerka.
  - Literal `crates/content/tests/situations_common/mod.rs:43-61`: `condition_of: None`.
- Create (testovi): dopuna `crates/content/tests/v2_situation_space.rs`, `crates/content/tests/v2_principles.rs` (test 13),
  `crates/game/tests/situation_space.rs` (drugi test).
- Ne dirati: `content/hotel/v2/situation_templates.json` (B4), `templates_principles_debt.rs` (dug se ne mijenja), jezgru obračuna.

**Interfaces:** Consumes: `RoundSituation`, `Game::space`, `situation_effect` (B1a). Produces:
```rust
// content::Template:  pub condition_of: Option<String>
// sim_hotel_v2::situations::SituationSpec:  pub condition_of: Option<DeptCode>
impl TemplateSet { pub fn condition_of(&self, code: &str) -> Option<DeptCode> }          // šifra predloška
impl content::SituationTemplates { pub fn condition_of(&self, code: &str) -> Option<DeptCode> } // predložak ili primjerak
// eligible.rs: holds(c, h, d, space: Option<i64>) — dept_condition_pts gleda `space`, ostali dept_* gledaju `d`
```

**Testovi čuvara:**
5. `v2_situation_space.rs` — `condition_of_reads_the_condition_of_that_space`. Obrazac je `boost_multiplies_weight_exactly`
   (`v2_situation_draw_edges.rs:417-460`).
   - Postav: `frozen()`, `general("P-LIFT", Kind::Breakdown, OFTEN)` s `condition_of = Some(DeptCode::rooms())` i
     `when.dept_condition_pts = below 4_000`. Samo on je u skupu, a sidra pune n − 1.
   - Smještaj 3_500 → za sjeme 0..50 `d.len() == 1`. Smještaj 6_000 → `0`.
   - Karakterizacija: isti predložak bez `condition_of` na 3_500 → `0` (cijeli hotel ne vidi stanje odjela).
   - Danas crven: nema polja.
6. `v2_situation_space.rs` — `condition_of_must_name_a_space_that_wears`, nad repo vrijednošću (`tpl` kao u `v2_principles.rs:13`).
   - P-DIZALO + `condition_of: "rooms"` + `when.dept_condition_pts: {below: 6000}` → `build()` prolazi.
   - Isto bez `condition_of` → `Err` s „uvjet odjela traži predložak s odjelom".
   - `condition_of: "maintenance"` i `"nepostoji"` → `Err` s „condition_of".
   - Repo snimak `to_value()` nema ključ `condition_of`.
7. `v2_principles.rs`, test 13 — `breakdown_condition_needs_a_space_that_wears`.
   - P-DIZALO kao u testu 6 → `!hit`.
   - P-DIZALO s `condition_of`, bez uvjeta → `hit`.
   - P-BRAVE + `when.dept_condition_pts: {below: 6000}` bez `condition_of` → `hit`. **Danas `!hit`, pa je crven na tvrdnji**:
     `maintenance` se ne troši.
   - P-BRAVE + `condition_of: "rooms"` + isti uvjet → `!hit`.
8. `crates/game/tests/situation_space.rs` — `condition_of_moves_the_repair_to_that_space`.
   - Snimak je repo vrijednost s `condition_of: "rooms"` na P-INSPEKCIJA (restoran, bez uvjeta stanja, odgovor 0 hitni).
   - `find` iz testa 4 traži primjerak P-INSPEKCIJA. Hitno prema šutnji: `rooms` A − B ≥ 1_000, a `restaurant` A == B.
   - 🟨 Ako P-INSPEKCIJA ne dođe ni u jednom sjemenu, čuvar širi raspon i to zapiše.

**Koraci:**
1. **Čuvar:** testovi 5–8. Commit „Popravak B1b (čuvar): …".
2. **Graditelj:**
   - `space_condition` = stanje odjela `condition_of` iz `view.state.departments` ako je polje zadano, inače `d.condition_pts`. Ide u `holds`/`weight`.
   - `Game::space` i `season.rs`: `condition_of(&template).or(department)`.
   - Commit „Popravak B1b: uvjet stanja čijeg prostora (condition_of)".

**Provjere:**
- Isti CI popis kao B1a. Kartične: `-p content --test v2_situation_space --test v2_principles --test v2_situation_draw_edges`,
  `-p game --test situation_space --test v2_sealed`.
- **Brojke se NE mijenjaju:** `cmp` tri balansa (v1 100; v2 week i month 1000) prema osnovici prije prve izmjene → bez razlike,
  isti EXIT. `v2_sealed` zelen **bez** novog pečata. `CORE_VERSION` se ne diže.

**Rizik i odluke:**
- 🟨 `condition_of` je samo odjel (danas samo `rooms`), ne „zgrada". Leon je za t. 7 rekao „stanje Smještaja", a gost Smještaja već osjeća zgradu.
- 🟨 Polje mijenja samo uvjet `dept_condition_pts` i cilj popravka. Ostali `dept_*` gledaju odjel situacije.
- Put `season.rs` s `condition_of` u B1b pokriva samo test 5 (izvlačenje). Paritet ga počinje vidjeti kad B4 upiše polje u
  sadržaj (Review Focus 3).

**Leonov dio:** — (ADR-042 t. 7 je odlučen ✅; pragovi za P-BRAVE, P-CURENJE-VODE i P-DIZALO su brojke za B4 i L1-13).

---

## Review Focus kandidati
1. **Igra s hitnim popravkom na prostoru blizu 100:** +10 se stisne na 10 000, pa student plati 250 € za manji učinak. Nijedan
   test ne gleda što ekran tada kaže.
2. **+10 nije pod pečatom:** zamrznuti snimak nema ključ, pa igra s bonusom 0. Buduća promjena puta bonusa prođe pečat, a čuvaju je
   samo test `urgent_repair_…` i balans.
3. **`condition_of` u sezoni robota** (B1b) nema test pariteta dok B4 ne upiše polje. Igra i balans bi se mogli razići baš na P-DIZALO.
4. **Isti kvar dvaput u mjesecu** (ADR-042 t. 12) i +10 nakon prvog popravka: drugi primjerak više ne bi smio proći uvjet. To je
   ponašanje B2 + B1a zajedno, a nijedna kartica ga ne testira.
