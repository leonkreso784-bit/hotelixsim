### Task B1: Vlastite v2 mehanike situacija s ciljem odjela — podijeljeno na **B1a** (učinak na prostor odjela) i **B1b** (uvjet „čijeg prostora", `condition_of`)

**Zašto podjela:** revizori procjenjuju 8–18 h (01 #5: 8–12 h; 02 t. 3 + t. 4: 14–22 h), a grana živi najviše ~1 dan. B1a mijenja
brojke (diže `CORE_VERSION` 2.3.0 → 2.4.0), B1b ih ne mijenja (balans bajt-isti), jer nijedan predložak još nema `condition_of` (to upisuje B4).
Redoslijed (voditelj, 15.10.): **B2a → B2b → B3a → B3b → B1a → B1b → B4a–c**. B1a preuzima i slučajeve pečata I3.

**Zašto (nalaz):** [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §4 #2 i #5. Izvještaj 01 #5: `SituationOutcome`
nema odjel, hitan popravak je samo trošak (`crates/sim-hotel/src/mechanic.rs:160-167`), a sva „oprema" ide u Smještaj
(`crates/sim-hotel-v2/src/resolve/mod.rs:349` → `resolve/guests.rs:105-112`). Izvještaj 02 t. 3–4. [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md)
t. 1 (hitan popravak +10 stanju **tog** odjela, samo v2) i t. 7 (curenje, brave, klima, praonica, dizalo → stanje Smještaja).
Završni pregled faze A (`.superpowers/sdd/2026-10-14-popravak-temelja/faza-a-review.md`):
- **I1:** B1a se ne spaja bez B3.
- **I2:** načelo šutnje računa v1 bez faktora kruga (`crates/content/src/templates_principles.rs:224-305`). B1a ga seli na v2. `BreakdownCondition` ne vidi `condition_of` → B1b.
- Trijaža: `silence_dominates` zove `to_core()` po predlošku → B1a.
- „3. Međudjelovanja": novo polje sadržaja mora imati `serde(default)`.

---

## B1a: popravak mijenja stanje prostora odjela situacije

**Veličina:** M–L, 7–8 h · **Ovisi o:** A2, A3 (spojeni), **B2a** (verzija 2.3.0, `Case` s postavkama, `common::v2_game_with`), **B3b spojen u `model-v2/sloj1`** (I1, vidi „Oslonac na B3") · **Nije paralelno
s:** B2a (oba dižu verziju i pišu `v2-golden.json`; B2a ide prije) · **Model:** opus · **Prevodi Rust:** da

**Cilj:** U v2 obračunu situacija djeluje na prostor svog odjela. Hitan popravak (`EquipmentRepair { urgent: true }`) diže stanje
tog odjela za `economy.repair_urgent_condition_pts` (sadržaj: 1000 = +10 bodova), bez faktora kruga. Privremeni popravak ostaje
kakav je danas (v1 −100 puta faktor kruga), ali ide u isti odjel. Bez odjela (sidra S1–S4, cijeli hotel) i za neraspoređeni odjel
učinak ide u Smještaj, kao danas. Načelo „šutnja nije bolja" računa istu v2 funkciju kao obračun, za obje vrste kruga. v1 se ne mijenja.

**Files:**
- Create: `crates/sim-hotel-v2/tests/situation_space.rs`, `crates/content/tests/v2_situation_space.rs`, `crates/game/tests/situation_space.rs`
- Modify (jezgra):
  - `crates/sim-hotel-v2/src/situations/mod.rs:69-81`: iza `reward_only` dodaj `RoundSituation`.
  - `crates/sim-hotel-v2/src/resolve/situations.rs:1-79`: `pub fn situation_effect`. `SituationSum.equipment` postaje mapa
    po odjelu. `shown` nosi i odjel.
  - `crates/sim-hotel-v2/src/resolve/mod.rs:200-206`: potpis `&[RoundSituation]`.
  - `crates/sim-hotel-v2/src/resolve/mod.rs:213, 265-280, 349`: Smještaj iz mape, ostali odjeli kroz `ops_in`.
  - `crates/sim-hotel-v2/src/resolve/mod.rs:492-531`: objašnjenje.
  - `crates/sim-hotel-v2/src/resolve/departments.rs:21-37, 54-70`: polje `situation_pts` u `OperatedIn`.
  - `crates/sim-hotel-v2/src/department.rs:193-213` (`DeptInput.situation_pts`), `:437-443` (u zbroj `condition_base`).
  - `crates/sim-hotel-v2/src/explain.rs:40-49, 172-180`: polje `situation_others`, oduzima se od retka „održavanje".
  - `crates/sim-hotel-v2/src/catalog.rs:33-46` (polje), `:384-389` (provjera 0..=10 000), `:668-678` (degenerirani = 0).
  - `crates/sim-hotel-v2/src/lib.rs:27-50` (izvoz), `crates/sim-hotel-v2/src/season.rs:321-348` (prostor primjerka),
    `crates/sim-hotel-v2/Cargo.toml:3` (`version` 2.3.0 → 2.4.0; 2.3.0 je B2a).
- Modify (igra):
  - `crates/game/src/resolve.rs:26-115`: `hotel_outcomes` vraća `Vec<RoundSituation>`, a `:126` i `:149-151` ga troše.
  - `crates/game/src/drawn.rs:398-411`: iza `no_penalty` dodaj `space`.
  - `crates/game/src/model.rs:398-430` (`resolve`; v1 grana uzima `.outcome`), `:755-769` (`period_report`).
- Modify (sadržaj):
  - `crates/content/src/v2_dept.rs:216-236` (`EconomyDto`: `Option<i32>`, `serde(default, skip_serializing_if)`), `:460-497`, `:500-546`.
  - `content/hotel/v2/economy.json`: `"repair_urgent_condition_pts": 1000`.
  - `crates/content/src/templates_principles.rs:9-11, 72-101, 224-305` (v2 funkcija, `to_core()` jednom).
  - `crates/content/src/templates_principles_debt.rs:81-102`: samo brisanje šifara koje više ne krše.
- Modify (postojeći testovi, samo mehanička prilagodba potpisa; to nisu testovi čuvara):
  - `DeptInput` literali dobivaju `situation_pts: 0`: `crates/sim-hotel-v2/tests/department.rs:75,166`, `leases.rs:82`, `parking_price.rs:78,158`.
  - Pozivi `resolve_period` s ishodima dobivaju `RoundSituation::from`: `period.rs:519,533,544`, `situation_effect.rs:49`.
  - **Ručne kopije obračuna** u `crates/game/tests/situations_delegate.rs:265-296` i `situations_replay.rs:110-136` moraju
    nositi prostor kao igra: `space` iz `h.drawn[&r]` primjerka, `i.drawn.department`.
  - Pečat: `crates/game/tests/fixtures_v2/v2-golden.json` dobiva samo novi ključ, preko generatora.
- Ne dirati:
  - `crates/sim-hotel/**`: v1 i `effects` ostaju bajt-isti, B1a ih samo poziva.
  - `crates/game/tests/fixtures_v2/v2-snapshot.json` (test 2).
  - `DrawnItem`/`event.rs` (dnevnik isti), `openapi.json`, `apps/web`, `docs/`.
  - Balans i paritet (`crates/balance/**`): vlasnik je B2/B3.

**Interfaces:**
- Consumes: `sim_hotel::effects` (v1, nepromijenjen), `Catalog::dept(&DeptCode) -> Result<&DeptSpec, CatalogError>`,
  `UsaliClass::is_undistributed()` (`department.rs:73`), `PeriodCoefficients::situation_effect_bp`, `DrawnSituation.department`.
- Produces:
```rust
// sim_hotel_v2::situations (izvoz i iz korijena)
/// Ishod situacije u obračunu v2 (ADR-042 t. 1): v1 ishod i odjel čiji prostor popravak mijenja;
/// `None` (sidra, cijeli hotel) i neraspoređeni odjel = Smještaj; nepoznat odjel = greška kataloga.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundSituation { pub outcome: SituationOutcome, pub space: Option<DeptCode> }
impl From<SituationOutcome> for RoundSituation { /* space: None */ }
// sim_hotel_v2 (resolve/situations.rs, izvoz iz korijena)
pub struct SituationEffect { pub effects: Effects, pub space: DeptCode }
/// Učinak jedne situacije u krugu `p`, isti račun kao obračun (bez granice zbroja kruga). Bodovi imaju faktor kruga, troškovi su
/// puni. `effects.equipment_pts` = promjena stanja prostora `space` = privremeni popravak s faktorom + broj hitnih popravaka ×
/// `economy.repair_urgent_condition_pts` bez faktora.
pub fn situation_effect(s: &RoundSituation, p: &Period, cat: &Catalog, category: Category)
    -> Result<SituationEffect, ResolveError>;
pub fn resolve_period(state: &HotelState, p: &Period, plan: &PeriodPlan, situations: &[RoundSituation], cat: &Catalog)
    -> Result<PeriodReport, ResolveError>;
// EconomyV2: pub repair_urgent_condition_pts: i32   (degenerirani katalog 0 → ekvivalencija s v1 ostaje)
// DeptInput: pub situation_pts: i32                 (promjena stanja iz situacija kruga; 0 = bez)
// game (crate): fn Game::space(&self, h: &HotelRecord, r: RoundKey, code: &str) -> Option<DeptCode>
```

**Testovi čuvara:**
1. `crates/sim-hotel-v2/tests/situation_space.rs` (danas ne prevodi: nema `RoundSituation`, `repair_urgent_condition_pts`):
```rust
mod common;
use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::RoundSituation;
use sim_hotel_v2::*;
fn code(s: &str) -> DeptCode { DeptCode(s.into()) }
fn cat(bonus: i32, effect_bp: u16) -> Catalog {
    let mut c = common::catalog_with_departments();
    c.economy.repair_urgent_condition_pts = bonus;
    for k in PeriodKind::ALL { c.periods.get_mut(&k).unwrap().situation_effect_bp = effect_bp; }
    c
}
fn repair(id: &str, urgent: bool, space: Option<&str>) -> RoundSituation {
    RoundSituation { outcome: SituationOutcome { code: id.into(),
        mechanics: vec![Mechanic::EquipmentRepair { urgent }], quality: 3 }, space: space.map(code) }
}
fn run(c: &Catalog, p: &Period, s: &[RoundSituation]) -> Result<PeriodReport, ResolveError> {
    let st = HotelState::from_profile(c, Profile::C).unwrap();
    resolve_period(&st, p, &common::hold_plan(&st, p, c), s, c)
}
/// Stanje odjela nakon kruga sa situacijama minus bez njih.
fn delta(c: &Catalog, p: &Period, s: &[RoundSituation], dept: &str) -> i32 {
    let (a, b) = (run(c, p, s).unwrap(), run(c, p, &[]).unwrap());
    assert!(b.after.departments[&code(dept)].condition_pts < 9_000, "bez stiskanja na 100");
    a.after.departments[&code(dept)].condition_pts - b.after.departments[&code(dept)].condition_pts
}
#[test]
fn urgent_repair_raises_the_situation_department_without_round_factor() {
    let c = cat(1_000, 350);
    for p in [month_period(Month::Jun), period_by_code(PeriodKind::Week, "t02").unwrap()] {
        let s = [repair("P-X.a", true, Some("restaurant"))];
        assert_eq!(delta(&c, &p, &s, "restaurant"), 1_000);
        assert_eq!(delta(&c, &p, &s, "rooms"), 0, "Smještaj ne dobiva popravak restorana");
    }
}
#[test]
fn temporary_repair_moves_to_the_situation_department() {
    let c = cat(1_000, 10_000);
    let s = [repair("P-X.a", false, Some("restaurant"))];
    let p = month_period(Month::Jun);
    assert_eq!((delta(&c, &p, &s, "restaurant"), delta(&c, &p, &s, "rooms")), (-100, 0));
}
#[test]
fn no_department_and_undistributed_go_to_rooms_unknown_is_an_error() {
    let (c, p) = (cat(1_000, 350), month_period(Month::Jun));
    for space in [None, Some("maintenance")] {
        let s = [repair("P-X.a", true, space)];
        assert_eq!((delta(&c, &p, &s, "rooms"), delta(&c, &p, &s, "restaurant")), (1_000, 0), "{space:?}");
    }
    assert!(matches!(run(&c, &p, &[repair("P-X.a", true, Some("nepostoji"))]), Err(ResolveError::Catalog(_))));
}
#[test]
fn lines_explain_department_repairs() {
    let (c, p) = (cat(1_000, 10_000), month_period(Month::Jun));
    let r = run(&c, &p, &[repair("P-A.jun", true, Some("restaurant")), repair("P-B.jun", false, None)]).unwrap();
    common::assert_lines_explain(&r.summary);
    let line = |id: &str| -> i64 { r.summary.lines.iter()
        .filter(|l| l.var == Var::Equipment && l.cause == Cause::Situation(id.into())).map(|l| l.amount).sum() };
    assert!(line("P-A.jun") > 0, "popravak restorana vidi se u sažetku opreme (udio ostalih odjela)");
    assert_eq!(line("P-B.jun"), -100);
}
#[test]
fn situation_effect_is_what_resolve_uses_and_degenerate_has_no_bonus() {
    let c = cat(1_000, 350);
    let cat_c = HotelState::from_profile(&c, Profile::C).unwrap().category;
    let e = situation_effect(&repair("P-X.a", true, Some("restaurant")), &month_period(Month::Jun), &c, cat_c).unwrap();
    assert_eq!((e.effects.equipment_pts, e.effects.cost_cents, e.space), (1_000, 25_000, code("restaurant")));
    assert_eq!(degenerate(&common::test_catalog()).economy.repair_urgent_condition_pts, 0);
}
```
2. `crates/content/tests/v2_situation_space.rs` — `repair_bonus_round_trips_and_old_snapshot_reads_zero`: obrazac iz
   `v2_obnova.rs:278-298`. Repo snimak gradi 1000. Bez ključa gradi 0, a `fingerprint_value(&back.to_value()) ==
   fingerprint_value(&old)`. Vrijednost 20_000 → `validate()` je `Err`. Danas crven: polje ne postoji.
3. `crates/content/tests/v2_principles.rs`, novi test 12 (proba ugriza iz I2). Mijenja se **samo v2** (faktor tjednog kruga,
   bonus), a načelo mora pasti:
```rust
#[test] // 12 (B1a, I2): načelo šutnje računa v2 situacije, obje vrste kruga
fn silence_principle_uses_v2_situation_mechanics() {
    let week_off = |v: &mut Value| v["v2"]["periods"]["week"]["situation_effect_bp"] = json!(1);
    let no_bonus = |v: &mut Value| v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(0);
    assert!(!hit(|_| {}, SilenceNotBetter, "P-DOTRAJALO"));
    assert!(!hit(week_off, SilenceNotBetter, "P-DOTRAJALO"), "+10 odvaja hitni odgovor od šutnje");
    assert!(!hit(no_bonus, SilenceNotBetter, "P-DOTRAJALO"), "šutnja i dalje gubi opremu (privremeni popravak)");
    assert!(hit(|v| { week_off(v); no_bonus(v) }, SilenceNotBetter, "P-DOTRAJALO"),
        "tjedni krug bez bonusa: bodovi 0 = 0, šutnja jeftinija (80 € prema 250 €)");
}
```
   Danas crven: nepoznato polje `repair_urgent_condition_pts` (`deny_unknown_fields`). Zadnja tvrdnja je crvena i sa starom
   mjerom, jer v1 `effects` ne vidi `periods`.
4. `crates/game/tests/situation_space.rs` — `drawn_department_gets_the_urgent_repair`. Danas prevodi, a crven je na tvrdnji.
   - Pomoćna funkcija `find(snap, want)`: tjedne igre `common::v2_game_from(snap, seed, PeriodKind::Week)` za sjeme 1..=40.
     Igrač šuti, a krugovi se zatvaraju kao `common::play_through`.
   - Traži prvi krug u kojem moj hotel (`common::hotel_ids()[0]`) u `hotels[..].drawn[&r]` ima `DrawnInstance` sa
     `drawn.department = Some(d)`. Uvjeti: `d` nije `rooms` ni neraspoređen, `latest_state` daje stanje `d` ≤ 8_500, a neki
     odgovor sadrži `Mechanic::EquipmentRepair { urgent: true }`.
   - Klon A bira taj odgovor (`choose_answer`), klon B šuti. Oba zatvara i obračunava.
   - `CoreState::V2` iz `latest_state(me)`: stanje `d` je A − B ≥ 1_000, stanje `rooms` je A == B.
   - Nijedno sjeme 1..=40 → `panic!` s porukom. Čuvar tada širi raspon i to zapiše.
- **Karakterizacijski (moraju ostati zeleni bez izmjene):** `situation_effect.rs` (faktor i granica kruga), `equivalence.rs` (v1 =
  v2 na degeneriranom katalogu), `game_parity_v2.rs` (igra = balans; hvata ako igra i `season.rs` različito biraju prostor),
  `crates/api/tests/student_sees.rs` (A6), `v1_frozen.rs`, testovi 1–11 u `v2_principles.rs`.

**Koraci:**
1. **Čuvar:** testovi 1–4. Commit „Popravak B1a (čuvar): …". RED zapisuje u `b1a-tests.md`.
2. **Graditelj:**
   - **Osnovica u scratchpad prije izmjene:** tri balansa (`--seeds 100 --check`; `--model v2 --round week|month --seeds 1000
     --check`), ispis mjere B3 i `principle_violations` (popis šifara po načelu).
   - **Jezgra:** `situation_effect` = v1 `effects` (kao danas, `reward_only` ostaje kod pozivatelja).
     - Bodovi idu kroz `scaled` (`situations.rs:74-79`), uključujući v1 `equipment_pts`.
     - Na to se doda `n_hitnih × repair_urgent_condition_pts`.
     - `space` = `s.space`, osim kad je `None` ili `cat.dept(..)?.class.is_undistributed()` → `DeptCode::rooms()`.
     - `situation_effects` zbraja `situation_effect` po stavci, pa `cap_round` kao danas.
     - `equipment` je `BTreeMap<DeptCode, i128>`. Smještaj ide u `GuestIn.sit_equipment`, ostali kroz `DeptInput.situation_pts` u `condition_base`.
3. **Objašnjenje:** redak situacije za Smještaj = njezini bodovi (kao danas). Za drugi odjel: `after.summary().equipment_pts` −
   isto sa stanjem tog odjela umanjenim za bodove te situacije. Zbroj tih iznosa ide u `situation_others` i oduzima se od retka
   „održavanje". Tako `assert_lines_explain` ostaje točan po konstrukciji.
4. **Igra:**
   - `Game::space` = `i.drawn.department` primjerka iz `h.drawn[&r]`; šifra koja nije primjerak (sidra) → `None`.
   - `hotel_outcomes` puni `RoundSituation`.
   - `season.rs`: `instances` nosi `s.department.clone()`; sidra → `None`.
5. **Sadržaj i načelo:**
   - `EconomyDto` dobiva polje (bez ključa = 0), `economy.json` dobiva 1000.
   - `principle_violations` zove `to_core()` jednom i predaje skup.
   - `silence_dominates` za svaku kategoriju i mjesec računa obje vrste kruga: `month_period(m)` i prvi `season_periods(Week)`
     s `situation_month() == m`. Zove `sim_hotel_v2::situation_effect` s `space` = prvi član odjela predloška. Usporedba polja
     ostaje ista. U opis ide i vrsta kruga: `šutnja ({kat}, {mjesec}, {week|month}) …`.
   - **Dug:** šifra koja više ne krši briše se s popisa. **Ako se pojavi novo kršenje** (mjera je sad točnija), stani i vrati
     BLOCKED s popisom; strop iz testa 2 ne diraš sam.
6. **Verzija i pečat (A3):** prvo dokaži da `cargo test -p game --test v2_sealed` pada na testu 1 („obračun v2 se promijenio").
   Zatim `version` u `crates/sim-hotel-v2/Cargo.toml` 2.3.0 → 2.4.0 (`Cargo.lock` ide u commit), pa
   `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest; echo EXIT=$?` → 0.
   `v2-golden.json` dobiva ključ `2.4.0`, stari ključevi ostaju, `v2-snapshot.json` je bajt-isti.
   **Slučajevi pečata I3 (voditelj 15.10.: pripadaju B1a):** u isti pečat dodaj slobodni odgovor s presudom i plan čovjeka, preko `Case` s postavkama i `common::v2_game_with` koje je pripremio B2a. B1a ionako mijenja put odgovor → učinak i obnavlja pečat, pa to ide u istu verziju 2.4.0.
7. Bilješke u `.superpowers/sdd/2026-10-14-popravak-temelja/za-dokumentaciju.md`: ADR 🟨 (odluke ispod) i MODEL_V2 §situacije.
   Commit „Popravak B1a: popravak u situaciji mijenja stanje prostora odjela (ADR-042 t. 1)".

**Provjere** (svaka zasebno, `; echo EXIT=$?`, izlaz u datoteku):
- **Kartične:** `cargo test -p sim-hotel-v2 --test situation_space` (5 passed), `-p content --test v2_situation_space --test v2_principles`,
  `-p game --test situation_space --test v2_sealed --test situations_delegate --test situations_replay`,
  `-p balance --test game_parity_v2`.
- **CI popis:** `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` ·
  `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` · `git diff --exit-code` ·
  `cargo sqlx migrate run --source crates/store/migrations` + `SQLX_OFFLINE=false cargo sqlx prepare --workspace --check`
  (baza `hotelixsim_agenti`) · `cargo test --workspace` · tri balansa · `corepack pnpm check:docs`, `test:docs`, `check:kod`,
  `check:api`, `lint`, `typecheck`, `test`, `corepack pnpm --filter @hotelixsim/web build`. `cargo deny` je samo u CI-ju.
- **v1 bajt-isti:** `cmp` izlaza `--seeds 100 --check` i `--seeds 1000` s osnovicom, a `v1_frozen.rs` zelen.
- **v2: brojke se mijenjaju → diže se `CORE_VERSION` na 2.4.0 i obnavlja pečat A3** (korak 6). Izlazi week/month 1000 ostaju `--check`
  EXIT 0, a `diff` prema osnovici ide u izvještaj. Tekstovi poruka brana ostaju doslovno isti.
- `git diff --stat` ne sadrži `crates/sim-hotel/`, `v2-snapshot.json`, `openapi.json`, `apps/web`.

**Oslonac na B3 (I1) — B1a se NE spaja bez ovoga:**
- B3b (mjera „udio situacija" kao **brana modela** u `failures_v2`, na referentnom planu, s Leonovom mjerom „najbolji odgovor prema
  šutnji kroz N krugova, po vrsti kruga") je spojen u `sloj1`. Po redoslijedu B1a dolazi nakon B3b; **ne spaja se prije B3b**.
- Ako je brana B3 prolazila prije B1a, mora prolaziti i poslije. Ako padne, to je nalaz za dijagnozu, ne razlog da se brana olabavi.
- Ako je B3 brana čekala kalibraciju (L1-13) već prije B1a, B1a ne smije smanjiti razliku „najbolji − šutnja" ni u jednoj vrsti
  kruga. Brojke prije i poslije idu u izvještaj.
- Ako B3b kasni, B1a čeka. Privremena brana „Brižni Δ > 0" iz I1 se ne gradi: B3 ne ovisi o B1, pa nema razloga za zakrpu.

**Rizik i odluke (🟨):**
- 🟨 **Hitni +10 bez faktora kruga, isto u tjednoj i mjesečnoj igri.** Popravak je fizički. S faktorom bi to bilo +0,35 boda (ADR-042 t. 1 kaže „oko +10").
- 🟨 **Privremeni popravak ostaje kakav je danas** (v1 −100 puta faktor, praktično 0). Mijenja mu se samo odjel. ADR t. 1: „slab/privremeni ne".
- 🟨 **Bez odjela i neraspoređeni odjel → Smještaj**, kao danas. Neraspoređeni odjel se ne troši (P-BRAVE i P-CURENJE-VODE su
  `maintenance`, a po ADR t. 7 pripadaju Smještaju).
- **Bonus ovisi o mehanici, ne o ocjeni:** hitni popravak u odgovoru ocjene 1 također diže stanje. ✅ Leon 15.10.: „popravak je popravak" (opcija a).
- 🟨 **Stari snimak bez ključa ima bonus 0** (presedan `groups`, `deferred_wear_bp`). Zato zamrznuti snimak pečata igra preusmjeravanje,
  ali ne +10. +10 čuvaju test 1 i balans (Review Focus 2).
- I3 (pečat sa slobodnim odgovorom i planom čovjeka) **ulazi u B1a** (voditelj 15.10.), uz dizanje na 2.4.0 (korak 6). B2a samo priprema `Case` s postavkama i `common::v2_game_with`.
- m3 (J-3 broji izvlačenja na planu robota) ostaje B4-u: ondje ulaze novi uvjeti stanja.
- Ekran odjela ne dobiva redak „+10 iz situacije". Stanje nakon kruga ga već sadrži. Prikaz ide u BACKLOG (C5).
- Leonove v2 igre u tijeku nakon spajanja daju 409 dok ne dođe C1. Voditelj mu to kaže prije igranja.

**Leonov dio: ✅ odgovoreno 15.10. — (a) da, popravak je popravak, i uz ocjenu 1. Pitanje je bilo:** „Učenik u kvaru izabere hitni popravak, ali loše postupi s gostom (ocjena 1).
Popravlja li se tada prostor (+10)?"
(a) **Da: popravak je popravak, a ocjena mjeri postupak s gostom** (preporuka: tako je u svih 15 predložaka s popravkom,
pa sadržaj ne treba mijenjati).
(b) Samo uz ocjenu 3 ili više: tada slab odgovor s hitnim popravkom troši 250 €, a ne popravlja.

Drugi dio kartice (B1b): [task-08b-b1b-uvjet-condition-of.md](task-08b-b1b-uvjet-condition-of.md).
