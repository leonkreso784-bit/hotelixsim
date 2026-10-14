### Task A2: Načela igre kao provjere sadržaja pri učitavanju (s popisom duga koji smije samo padati)

**Veličina:** M, 6–8 h · **Ovisi o:** A1 (grana `sloj1-l111` spojena u `sloj1`; kod čitan na b1bf446) · **Paralelno s:** A3
(druge datoteke; najviše 2 agenta koji prevode Rust) · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Sadržaj s diska (repo ili `HOTELIXSIM_CONTENT`, npr. profesorova mapa) koji krši načelo igre ne prolazi: server
ne krene i kaže koji predložak krši koje načelo. Današnja kršenja (dug za Fazu B) su popis u kodu: server kreće, dug je vidljiv
i ne raste (novi prekršaj pada; popravljeni predložak ostao na popisu pada). Snimak stare igre se načelima nikad ne provjerava.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #5 (+ 02 #1, 2, 8, 9, 10, 11). Načela su samo testovi (`crates/content/tests/
v2_templates.rs:130-148, 154-167`), učitavanje gleda oblik (`templates_check.rs:18-189`), server prima sadržaj mimo `cargo test`
(`crates/server/src/main.rs:35-48`). Najbolji odgovor najdulji u 54/55, kvar bez uvjeta stanja u 11/15 (brojanje nad JSON-om).

**Files:**
- Create: `crates/content/src/{templates_principles,templates_principles_debt}.rs`, `crates/content/tests/v2_principles.rs`
- Modify: `crates/content/src/lib.rs` (moduli i `pub use` uz r. 14-21; `Snapshot::read_v2_checked` uz r. 606-612),
  `crates/content/src/templates.rs` (polje `external_cause` u `Template`, iza `no_penalty`, r. 88-91),
  `crates/content/src/templates_check.rs` (jedan uvjet u `check_template`, iza r. 134),
  `crates/server/src/main.rs:47` (`read_v2` → `read_v2_checked`),
  `crates/content/tests/v2_templates.rs` (iz `repo_templates_cover_kinds_and_departments` makni petlju r. 130-148;
  obriši `repo_names_have_twenty_per_list` r. 152-167 — oboje seli u načela),
  `content/hotel/v2/situation_templates.json` (samo `"external_cause": true` u P-TOPLINSKI-VAL, P-OLUJA, P-NESTANAK-STRUJE,
  P-DOBAVLJAC, P-TERMINAL — Leonova odluka ✅ ADR-042 t. 7; nijedan drugi bajt).
- Ne dirati: `build()` i `Snapshot::from_value` (načela NE ulaze u put snimka iz dnevnika), `crates/sim-hotel*`, `crates/game`,
  `crates/balance`, `content/hotel/{catalog,profiles,situations}.json`, `crates/content/tests/fixtures/**`, `docs/`.
  Zajednička pravila: `.superpowers/sdd/2026-10-10-v2-sloj1/global-constraints.md` i `implementer-common.md`.

**Interfaces:**
- Consumes: `Content { v2, situation_templates, .. }`, `SituationTemplates::{to_core, instance}` (`templates_instance.rs:252`),
  `sim_hotel::{effects, Effects, SituationOutcome, Month, Category}`, `sim_hotel_v2::situations::{reward_only, DrawnSituation,
  NumberParam, Scope}`, `sim_hotel_v2::calendar::month_period`, `TemplateAnswer::quality`.
- Produces:
```rust
// crates/content/src/templates_principles.rs (NOVO); lib.rs: pub use templates_principles::{Principle, Violation,
//   principle_violations, check_principles_with}; pub use templates_principles_debt::KNOWN_VIOLATIONS;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Principle { // prva četiri iz testa (ocjena 5; ocjena 3 ili 4; gap_rounds >= 4; >= 20 imena), ostalo NOVO (02 #1, 2, 9, 10, 11)
    BestAnswer, MiddleAnswer, RepeatGap, NamesPerList,
    LabelLengths, BreakdownCondition, DeptPlaceholder, NamesUnique, SilenceNotBetter,
}
impl Principle { pub fn name(self) -> &'static str } // hrvatski: npr. RepeatGap → "razmak ponavljanja barem 4 kruga"
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation { pub principle: Principle, pub code: String, pub detail: String }
// code = šifra predloška; za imena "names.<izvor>.<rod>" (NamesPerList) ili "names" (NamesUnique)
pub const LABEL_RATIO_PCT: usize = 150;   // 🟨 k = 1,5
pub const MIN_GAP_ROUNDS: u8 = 4;
pub const MIN_NAMES_PER_LIST: usize = 20;
pub(crate) fn is_breakdown(t: &Template) -> bool; // kind == Kind::Breakdown ili EquipmentRepair u odgovoru/unanswered
pub fn principle_violations(c: &Content) -> Result<Vec<Violation>, ContentError>; // bez predložaka → Ok(vec![])
pub fn check_principles_with(c: &Content, known: &[(Principle, &[&str])]) -> Result<(), ContentError>;
// crates/content/src/templates_principles_debt.rs (NOVO)
pub const KNOWN_VIOLATIONS: &[(Principle, &[&str])];
// crates/content/src/lib.rs
impl Snapshot { pub fn read_v2_checked(dir: &Path) -> Result<Snapshot, ContentError> } // read_v2 + build + načela
// crates/content/src/templates.rs, struct Template (JSON ključ "external_cause"):
#[serde(default, skip_serializing_if = "crate::is_false")] pub external_cause: bool,
```

**Pravila novih načela (stara četiri točno kao danas u testu):** **LabelLengths:** duljina = `label.chars().count()` sirove
oznake; krši kad `max * 100 > min * LABEL_RATIO_PCT` među svim odgovorima, bez obzira koji je najdulji (Rizik).
**BreakdownCondition:** za `is_breakdown(t)` traži `t.when.dept_condition_pts` s `below: Some(_)` ili `t.external_cause`;
`chance.boost` nije uvjet (kvar bi i dalje dolazio u dobro održan odjel). **DeptPlaceholder:** u tekstovima iz `check_texts`
(naslov, opis, rubrika, oznake, tri polja lika) svaki `{odjel}` je ili točno „({odjel})", ili mu prethodna riječ (`trim_end`,
zadnja po razmaku) malim slovima počinje s `odjel` („u {odjel}" ispisalo bi „u Restoran"). **NamesUnique:** isto ime u dva
od šest popisa; `detail` imenuje oba. **SilenceNotBetter:** za svaku kategoriju (`base.categories`) i mjesec (`when.months`,
prazno = `Month::SEASON`) složi primjerak (korak 2), `effects` šutnje (`SituationOutcome::unanswered`) i svakog odgovora
najmanje kvalitete (`from_answer`), uz `no_penalty` oboje kroz `reward_only`. Šutnja dominira kad su `groups` jednaki,
`satisfaction_pts, morale_pts, rating, equipment_pts` ≥, `cost_cents, extra_wage_cents` ≤ i `silent != worst`.

**Testovi čuvara** — `crates/content/tests/v2_principles.rs`. Danas crveni svi zajedno iz istog razloga: datoteka se ne
prevodi (`error[E0432]: unresolved imports content::Principle, content::KNOWN_VIOLATIONS…` i `E0599 no function
read_v2_checked`). Test 10 bi s današnjim `build()` prošao — čuva da načela ne uđu u put starih igara.
```rust
use content::{Content, ContentError, KNOWN_VIOLATIONS, Principle::{self, *}, Snapshot, check_principles_with,
              principle_violations, repo_dir};
use serde_json::{Value, json};
fn v2_value() -> Value { Snapshot::read_v2(&repo_dir()).unwrap().to_value() }
fn built(v: &Value) -> Content { Snapshot::from_value(v).unwrap().build().unwrap() }
fn tpl<'a>(v: &'a mut Value, code: &str) -> &'a mut Value {
    let i = v["v2"]["situation_templates"]["templates"].as_array().unwrap().iter().position(|t| t["code"] == code).unwrap();
    &mut v["v2"]["situation_templates"]["templates"][i]
}
/// Krši li `code` načelo `p` nakon izmjene.
fn hit(change: impl FnOnce(&mut Value), p: Principle, code: &str) -> bool {
    let mut v = v2_value();
    change(&mut v);
    principle_violations(&built(&v)).unwrap().iter().any(|x| x.principle == p && x.code == code)
}
fn repo_codes(p: Principle) -> Vec<String> { principle_violations(&built(&v2_value())).unwrap().into_iter()
    .filter(|x| x.principle == p).map(|x| x.code).collect() }
#[test] fn repo_content_loads_checked_with_known_debt() { Snapshot::read_v2_checked(&repo_dir()).unwrap(); } // 1
#[test] // 2: dug samo za tri nova načela; stropovi = stanje 14.10.
fn known_debt_names_existing_templates_and_never_grows() {
    let v = v2_value();
    let codes: Vec<&str> = v["v2"]["situation_templates"]["templates"].as_array().unwrap().iter()
        .map(|t| t["code"].as_str().unwrap()).collect();
    let ceiling = [(LabelLengths, 55), (BreakdownCondition, 6), (SilenceNotBetter, 16)];
    for (p, list) in KNOWN_VIOLATIONS {
        let max = ceiling.iter().find(|(q, _)| q == p).map_or(0, |(_, n)| *n);
        assert!(list.len() <= max, "{p:?}: dug {} > {max} — popis smije samo padati", list.len());
        let mut s = list.to_vec(); s.sort(); s.dedup();
        assert_eq!(s.len(), list.len(), "{p:?}: šifra dvaput");
        for c in *list { assert!(codes.contains(c), "{p:?}: {c} nije u sadržaju — makni ga s popisa"); }
    }
    for p in [BestAnswer, MiddleAnswer, RepeatGap, NamesPerList, DeptPlaceholder, NamesUnique] {
        assert!(repo_codes(p).is_empty(), "{p:?}: {:?}", repo_codes(p));
    }
}
#[test] // 3: preseljeno iz v2_templates.rs
fn old_principles_are_load_checks() {
    assert!(!hit(|_| {}, RepeatGap, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["answers"][0]["criteria"] = json!([1, 2, 3, 4]), BestAnswer, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["answers"][1]["criteria"] = json!([1, 2]), MiddleAnswer, "P-KLIMA"));
    assert!(hit(|v| tpl(v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3), RepeatGap, "P-KLIMA"));
    assert!(hit(|v| { v["v2"]["names"]["partner"]["m"].as_array_mut().unwrap().pop(); }, NamesPerList, "names.partner.m"));
}
#[test] // 4: k = 1,5, bez obzira koji je odgovor najdulji
fn label_lengths_stay_within_one_and_a_half() {
    let set = |a: usize, b: usize, c: usize| move |v: &mut Value| {
        for (i, n) in [a, b, c].into_iter().enumerate() { tpl(v, "P-KLIMA")["answers"][i]["label"] = json!("x".repeat(n)); }
    };
    assert!(!hit(set(150, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(hit(set(151, 120, 100), LabelLengths, "P-KLIMA"));
    assert!(hit(set(100, 151, 100), LabelLengths, "P-KLIMA"), "srednji najdulji također odaje");
}
#[test] // 5
fn breakdown_needs_bad_state_or_external_cause() {
    let no_cond = |v: &mut Value| { tpl(v, "P-BOJLER")["when"].as_object_mut().unwrap().remove("dept_condition_pts"); };
    assert!(!hit(|_| {}, BreakdownCondition, "P-BOJLER"));
    assert!(hit(no_cond, BreakdownCondition, "P-BOJLER"), "pojačanje nije uvjet");
    assert!(hit(|v| tpl(v, "P-BOJLER")["when"]["dept_condition_pts"] = json!({"at_least": 1000}), BreakdownCondition, "P-BOJLER"));
    assert!(!hit(|v| { no_cond(v); tpl(v, "P-BOJLER")["external_cause"] = json!(true); }, BreakdownCondition, "P-BOJLER"));
    assert!(!hit(|_| {}, BreakdownCondition, "P-OLUJA"), "ADR-042 t. 7: vanjski događaj");
    assert!(hit(|_| {}, BreakdownCondition, "P-BRAVE"), "dug do Faze B");
    let mut v = v2_value(); tpl(&mut v, "P-NOVINAR")["external_cause"] = json!(true);
    let r = Snapshot::from_value(&v).unwrap().build();
    assert!(matches!(&r, Err(ContentError::Invalid(m)) if m.contains("external_cause")), "{r:?}");
}
#[test] fn dept_placeholder_only_after_odjel_or_in_brackets() { // 6
    let title = |s: &'static str| move |v: &mut Value| tpl(v, "P-DOTRAJALO")["title"] = json!(s);
    assert!(!hit(|_| {}, DeptPlaceholder, "P-DOTRAJALO"), "naslov s „({{odjel}})” je dopušten");
    assert!(!hit(title("Gost fotografira {stvar} u odjelu {odjel}"), DeptPlaceholder, "P-DOTRAJALO"));
    assert!(hit(title("Gost fotografira {stvar} u {odjel}"), DeptPlaceholder, "P-DOTRAJALO"));
    assert!(hit(title("{odjel}: gost fotografira {stvar}"), DeptPlaceholder, "P-DOTRAJALO"));
}
#[test] fn names_are_unique_across_lists() { assert!(hit(|v| { let n = v["v2"]["names"]["guest"]["f"][0].clone(); // 7
    v["v2"]["names"]["staff"]["f"][0] = n; }, NamesUnique, "names")); }
#[test] // 8: P-DOTRAJALO danas: najlošiji s hitnim popravkom je skuplji, ali šutnja gubi opremu → nema dominacije
fn silence_is_not_better_than_the_worst_answer() {
    assert!(!hit(|_| {}, SilenceNotBetter, "P-DOTRAJALO"));
    assert!(hit(|v| tpl(v, "P-DOTRAJALO")["unanswered"] = json!([{"type": "ServiceRecovery", "compensation": 0}]),
                SilenceNotBetter, "P-DOTRAJALO"), "isti bodovi, bez troška popravka");
    assert!(hit(|_| {}, SilenceNotBetter, "P-NOVINAR"), "loš odgovor plaća boravak, šutnja ništa (dug)");
}
#[test] // 9: dug smije samo padati
fn debt_list_only_shrinks() {
    let (c, mut v) = (built(&v2_value()), v2_value());
    tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    let e = check_principles_with(&built(&v), KNOWN_VIOLATIONS).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("razmak")), "{e:?}");
    let mut known = KNOWN_VIOLATIONS.to_vec();
    known.push((RepeatGap, &["P-KLIMA"][..]));
    let e = check_principles_with(&c, &known).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA") && m.contains("makni")), "{e:?}");
    let mut other_dir = KNOWN_VIOLATIONS.to_vec();
    other_dir.push((LabelLengths, &["P-NEPOSTOJI"][..]));
    check_principles_with(&c, &other_dir).expect("šifra koje nema (druga mapa sadržaja) ne ruši učitavanje");
    assert!(check_principles_with(&c, &[]).is_err(), "bez popisa duga današnji sadržaj pada");
}
#[test] fn snapshot_from_the_log_is_never_checked_for_principles() { // 10: stara igra se uvijek čita
    let mut v = v2_value(); tpl(&mut v, "P-KLIMA")["repeat"]["gap_rounds"] = json!(3);
    Snapshot::from_value(&v).unwrap().build().expect("snimak iz dnevnika gradi se bez načela");
}
#[test] // 11: put kojim server čita HOTELIXSIM_CONTENT
fn content_dir_with_new_violation_is_refused_at_load() {
    let (src, dir) = (repo_dir(), std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("a2-sadrzaj"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("v2")).unwrap();
    for f in ["catalog.json", "profiles.json", "situations.json", "v2/periods.json", "v2/departments.json",
              "v2/profiles.json", "v2/economy.json", "v2/names.json"] { std::fs::copy(src.join(f), dir.join(f)).unwrap(); }
    let text = std::fs::read_to_string(src.join("v2/situation_templates.json")).unwrap();
    let mut t: Value = serde_json::from_str(&text).unwrap();
    let i = t["templates"].as_array().unwrap().iter().position(|x| x["code"] == "P-KLIMA").unwrap();
    t["templates"][i]["repeat"]["gap_rounds"] = json!(3);
    std::fs::write(dir.join("v2/situation_templates.json"), serde_json::to_string(&t).unwrap()).unwrap();
    let e = Snapshot::read_v2_checked(&dir).unwrap_err();
    assert!(matches!(&e, ContentError::Invalid(m) if m.contains("P-KLIMA")), "{e:?}");
}
```

**Koraci graditelja:**
1. `templates.rs`: polje `external_cause` (doc: „Kvar ili popravak čiji uzrok nije stanje prostora hotela — vanjski
   događaj, dobavljač, oprema partnera (ADR-042 t. 7); izuzet od načela ‚kvar samo kad je stanje loše'").
2. `templates_principles.rs`: tipovi, konstante, `is_breakdown`, `principle_violations` (imena, pa predlošci redom datoteke;
   rezultat sortiran). Primjerak za šutnju (privatna `fn probe`) kao `july_instance` (`v2_templates.rs:709-748`): jednom
   `to_core()`, pa `DrawnSituation { code: format!("{code}.{}", m.code()), template: code.into(), period: month_period(m),
   day: 1, occurrence: 1, slot: 1, kind: spec.kind, weight: spec.weight, scope: Scope::Hotel, major: None, roles:
   BTreeMap::new(), department: spec.department.members(v2).and_then(|d| d.first().cloned()), numbers: najmanja vrijednost
   (Range(a, _) → a, Pick(v) → min) }` i `instance(&d, &v2.base, category)`; greška `effects` → `Invalid("predložak {code}: …")`.
3. `check_principles_with`: (a) prekršaj kojeg nema u `known` → redak `"načelo '{name}': {code}: {detail} — novo kršenje;
   popravi sadržaj (popis duga smije samo padati)"`; (b) šifra iz `known` koja **postoji** u sadržaju bez prekršaja tog načela →
   `"načelo '{name}': {code} više ne krši — makni ga s popisa KNOWN_VIOLATIONS (crates/content/src/templates_principles_debt.rs)"`;
   šifre koje nema preskoči. Retci sortirani: `ContentError::Invalid(format!("načela igre:\n{}", retci.join("\n")))`.
4. `templates_check.rs` iza r. 134: `if t.external_cause && !crate::templates_principles::is_breakdown(t) { return
   invalid(format!("{ctx}: external_cause ide samo uz kvar ili popravak")); }`
5. JSON: `"external_cause": true` u pet predložaka iz Files (iza `"no_penalty"` ako ga ima, inače kao zadnji ključ).
6. `templates_principles_debt.rs` (doc: što je dug, da smije samo padati, da ga briše Faza B uz Leonov pregled):
   - `LabelLengths` — svih 55 šifri iz `situation_templates.json`, redom datoteke (P-KLIMA … P-TERMINAL); najmanji omjer
     danas je 2,52 (P-INSPEKCIJA), pa nijedan predložak ne prolazi.
   - `BreakdownCondition` — P-KLIMA P-INSPEKCIJA P-DIZALO P-PRAONICA P-BRAVE P-CURENJE-VODE.
   - `SilenceNotBetter` — P-KLIMA P-POVISICA P-BOJLER P-ROOFTOP-BUKA P-SPA-TECAJ P-LOBBY-SUKOB P-LOBBY-KAVA P-NOVINAR
     P-VAL-RECENZIJA P-KRIZA-OSOBLJA P-ALERGIJA P-OVERBOOKING P-BOLOVANJE P-PRAONICA P-PARKING-STETA P-UCJENA-RECENZIJOM.
   Prva dva popisa su strukturna: razlika od `principle_violations` = greška u kodu, popravi kod. Treći je iz Python
   zrcala `effects`: ako kod daje podskup, upiši stvarni popis i navedi razliku u izvještaju; ako daje **više** → stani, NEEDS_CONTEXT.
7. `lib.rs`: `read_v2_checked` = `let s = Snapshot::read_v2(dir)?; check_principles_with(&s.build()?, KNOWN_VIOLATIONS)?;
   Ok(s)`; doc: „Samo za sadržaj s diska; snimak iz dnevnika (`from_value`) se ne provjerava — stara igra se uvijek čita."
8. `server/src/main.rs:47` → `read(content::Snapshot::read_v2_checked)?`. 9. `v2_templates.rs`: makni preseljeno (Files).

**Provjere (iz korijena stabla, svaka zasebno, `; echo EXIT=$?`):**
- Prije koraka 1 u scratchpad: `cargo run -q -p balance --release -- --seeds 100 --check > b1-prije.txt`, isto s
  `--model v2 --round week --seeds 1000 --check` → `b2w-prije.txt` i `--round month` → `b2m-prije.txt` (zapiši EXIT).
- `cargo test -p content --test v2_principles` → `test result: ok. 11 passed`; zatim `cargo fmt --all -- --check` ·
  `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `node scripts/check-core-deps.mjs` ·
  `corepack pnpm check:kod` → EXIT=0. Ista tri balansa poslije: `diff` prazan, isti EXIT (oznaka se u obračunu ne čita).
- `HOTELIXSIM_CONTENT=target/tmp/a2-sadrzaj timeout 60 cargo run -q -p server -- serve; echo EXIT=$?` (mapu ostavlja test 11)
  → EXIT=1, poruka sadrži „načela igre" i „P-KLIMA"; pada prije baze i porta. Server s repo sadržajem ne pokreći (slušao bi).

**Rizik i odluke:**
- 🟨 **k = 1,5 i „svi odgovori unutar k"**, ne „ili najbolji nije najdulji" (nalog dopušta oba): Leon je rekao „duljine se
  ujednačuju" (ADR-042 t. 10), a „ili" propušta P-POVISICA (75 i 78 prema 25 znakova). Cijena: dug 55/55, ne 54/55.
- 🟨 **Šutnja s troškom:** usporedba uključuje trošak, pa je u dugu 16 predložaka gdje loš odgovor plaća, a šutnja ne (npr.
  P-NOVINAR; test `v2_templates.rs:750` ga drži jednakim samo u bodovima). „Samo bodovi" = makni `cost_cents`/`extra_wage_cents`, dug 0.
- 🟨 **P-INSPEKCIJA** (kriza s popravkom u restoranu) nije u Leonovoj podjeli iz ADR-042 t. 7: ostaje u dugu.
- **Uvjet „čijeg prostora" (Faza B, 02 #3):** P-BRAVE i P-CURENJE-VODE su `maintenance` (ne troši se), P-DIZALO `hotel`;
  kad Faza B doda stanje drugog odjela ili zgrade, načelo ga mora priznati kao uvjet, inače te stavke ne izlaze iz duga.
- **Redoslijed odgovora (najbolji uvijek prvi, B1) NIJE načelo sadržaja** nego prikaza (ADR-042 t. 10, miješanje po hotelu
  u igri): ne ide u provjeru, inače bi autor „ručno miješao", a stari zapisi odgovora značili bi drugo.
- **Provjeravatelj:** `build()`/`from_value` ne zovu načela (test 10); poruka imenuje predložak i načelo; `external_cause`
  u snimku samo kad je `true` (stari snimci isti JSON i otisak); balans bajt-isti. 🟨 u izvještaj taska („Za ADR").

**Leonov dio** (kroz anketu prije Faze B; A2 ne čeka): (1) koliko smiju razlikovati duljine odgovora: 1,5× (preporuka) /
1,3× / 2×; (2) smije li neodgovorena situacija biti jeftinija od lošeg odgovora uz iste bodove: ne, šutnja nikad nije bolja
(preporuka) / smije, gledaju se samo bodovi; (3) inspekcija u restoranu: kvar koji dolazi kad je stanje loše / vanjski događaj.

## Review Focus kandidati
1. **Sidra S1–S4** (v1 `situations.json`, zamrznut) idu u svaku v2 igru, najbolji odgovor prvi i najdulji; načela ih ne gledaju.
2. **Prepuštanje voditelju** (kvaliteta 2, mehanike `unanswered`) često je bolje od najlošijeg odgovora; načelo gleda samo
   šutnju (q 1), pa student može naučiti „prepusti umjesto da biraš".
3. **Put sadržaja mimo starta servera** (budući profesorov uređivač ili API, `balance` nad tuđom mapom) ne zove `read_v2_checked`.
