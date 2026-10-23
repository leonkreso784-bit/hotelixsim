# 2026-10-14-popravak-temelja-kartice — dio 3

Arhiva (zamrznuto). Indeks: [2026-10-14-popravak-temelja-kartice.md](../2026-10-14-popravak-temelja-kartice.md).

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

Drugi dio kartice (B1b): [task-08b-b1b-uvjet-condition-of.md](#b1b-uvjet-stanja-čijeg-prostora-condition_of).

<!-- Nastavak kartice [B1](#task-b1-vlastite-v2-mehanike-situacija-s-ciljem-odjela--podijeljeno-na-b1a-učinak-na-prostor-odjela-i-b1b-uvjet-čijeg-prostora-condition_of) (B1a), odvojen zbog granice veličine. Sažetak i ovisnosti: u toj kartici i u planu. -->

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

Cigla B2 je podijeljena na dva dijela, jer ne stane u jedan dan (grana živi najviše ~1 dan). **B2a** mijenja pravilo
izvlačenja, dnevnik i pečat, uz jedno dizanje `CORE_VERSION`. **B2b** daje balansu broj situacija kao ulaz i proširuje
paritet. B2b ne mijenja brojke. Redoslijed: B2a, pa B2b.

### Task B2a: Broj situacija u postavkama igre, mjesečno 4/5/6, isti predložak najviše jednom u krugu

**Veličina:** M–L, 7–8 h · **Ovisi o:** A1, A3 · **Prvi u fazi B** (redom B2a → B2b → B3a → B3b → B1a → B1b → B4a–c, voditelj 15.10.). Ne ide paralelno s B1a, jer oba dižu
`CORE_VERSION` (B2a 2.2.0 → 2.3.0, B1a 2.3.0 → 2.4.0) i pišu `v2-golden.json`; B4 dolazi tek nakon B1b.
· **Model:** opus · **Prevodi Rust:** da

**Cilj:**
- Broj situacija po krugu više nije konstanta u kodu. Postavka igre zapisana je u `GameCreated`.
- Zadane vrijednosti: tjedno 3/4/5, mjesečno 4/5/6, po očekivanoj popunjenosti (ispod 75 % / 75–89 % / od 90 %).
  Dopušteno je 3–8 situacija, i broj ne smije padati s popunjenošću.
- Mjesečni krug se izvlači kao **jedan** krug, a ne kao niz virtualnih tjedana. Isti predložak dolazi najviše jednom u
  krugu, a iz kruga u krug smije se ponoviti.
- Najmanji razmak ponavljanja (4 tjedna) i „prilika barem svaka 3 tjedna" sele iz koda u datoteku predložaka.

**Zašto (nalaz):**
- [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §4 #13 i §6 (B2); [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 12–14.
- [BUGS.md](../../records/BUGS.md) B3 (`P-PRAONICA.jul` i `.jul.2`): uzrok je `count.rs:64-94` (`virtual_weeks`) i
  razmak u danima (`eligible.rs:136-155`).
- Izvještaji revizora: 02 #6–7, 03 #10 (`v2_situations.rs:266-272`), 01 #12.
- `faza-a-review.md` I3: pečat i paritet znaju samo vrstu kruga. Trijaža: provjera da `s.drawn` nije prazan → B2.

**Kako se danas zapisuju događaji (provjereno):**
- Postoji jedna verzija oblika za sve događaje, `SCHEMA_VER = 1` (`event.rs:13`). Kod čitanja (`event.rs:371-381`) nema
  „podizanja" starih događaja.
- Polja se dodaju samo uz `#[serde(default, skip_serializing_if = …)]` (uzor `GameCreated.rounds`, `event.rs:28-30`).
  Strukture imaju `deny_unknown_fields`.
- Zato B2a **ne** diže `SCHEMA_VER`. Novo polje je `Option`, a stari dnevnik bez njega čita se sa zadanim brojem za
  vrstu kruga.
- v1 igre polje nikad ne pišu. Da se i dalje čitaju, dokazuju `v1_frozen.rs` i `fixtures_sealed.rs`.
- Stare v2 igre ionako dobiju 409 zbog nove verzije jezgre (`model.rs:139-148`, Review Focus 1 plana).

**Files:**
- Create: `crates/game/tests/situation_settings.rs`.
- Modify (jezgra):
  - `crates/sim-hotel-v2/src/situations/count.rs:29-94`: brišu se `weekly_count`, `period_count`, `VirtualWeek` i
    `virtual_weeks`. Dolaze `SituationCounts` i `situation_count_with`. `situation_count` (`:51-54`) ostaje kao omotač
    sa zadanim brojem.
  - `situations/mod.rs:17-19` (`pub use`), `:44-46` (opis polja `day`: „početak kruga").
  - `situations/template.rs:14-22`: `TemplateSet` dobiva tri polja.
  - `situations/eligible.rs:136-155`: konstanta `MIN_GAP_WEEKS` odlazi, a `repeat_allows` dobiva `min_gap_weeks: u8`.
  - `situations/draw.rs`:
    - `:13` (uvoz), `:26-27` (konstanta `OPPORTUNITY_EVERY` odlazi);
    - `:189-218` (`candidates`: predložak koji je već u `out` ne ulazi);
    - `:286-292` (`needs_opportunity` dobiva broj tjedana);
    - `:313` i `:385` (broj iz `round_count`);
    - `:443-545`: petlja po virtualnim tjednima postaje jedan blok s `day = p.start_day`, `target = n`,
      `anchors = d.anchors` i jednom granicom odjela `dept_limit(n)`.
  - `crates/sim-hotel-v2/Cargo.toml:3` → `2.3.0` (2.2.0 → 2.3.0; sljedeće dizanje je B1a na 2.4.0), uz redak verzije u `Cargo.lock`.
- Modify (sadržaj):
  - `crates/content/src/templates.rs:12-21`: `SituationTemplates` dobiva dva polja.
  - `templates.rs:23-37`: `TemplateFile` dobiva dva `Option<u8>` polja s `serde(default, skip_serializing_if)`.
  - `crates/content/src/v2.rs:118-124`: prepisuje ta dva polja.
  - `templates_check.rs:26-29`: vrijednost 0 je greška, kao kod grupa.
  - `templates_core.rs:15-31`: zadano je 4 i 3, a `counts: None`.
  - `content/hotel/v2/situation_templates.json:3-4`: dodaju se `"min_gap_weeks": 4` i `"opportunity_every_weeks": 3`.
- Modify (igra):
  - `crates/game/src/event.rs:15-31`: `GameCreated.situation_counts` i nova struktura `SituationCountsDto`.
  - `command.rs:27-31`: `GameSettings.situations`.
  - `command.rs:41-59` i `:62-119`: provjera, pa zapis za v2. v1 polje ne piše.
  - `game.rs:88-107`: polje `situation_counts`.
  - `game.rs:126-193`: čitanje u `created()`; neispravna vrijednost daje `Corrupt`.
  - `drawn.rs:286`: `let mut set = tpl.to_core(); set.counts = Some(self.situation_counts);`.
- Modify (balans): `crates/balance/src/v2_situations.rs:266-272`: `count_band` vraća `(low, high)` iz
  `SituationCounts::default_for(kind)`, što daje tjedno 3–5 i mjesečno 4–6.
- Ne dirati:
  - API, `openapi.json`, web (profesorov ekran dolazi u kolosijeku PLATFORMA);
  - `MANDATORY_SITUATIONS`, ocjenu 2 za prepuštanje i pragove 75/90 % (vidi Rizik);
  - `situation_effect_bp` (L1-13);
  - zamrznute datoteke `game/tests/fixtures_v2/v2-snapshot.json` i `content/tests/fixtures/content-52ce161/`;
  - tekstove poruka brana balansa;
  - `docs/`.

**Interfaces:**
- Consumes: `expected_occupancy_bp` (`count.rs:17`), `Game::rounds_info`, `Model::check_version`, `GameError::InvalidSettings`
  (`error.rs:34-35`), `common::{hotel_ids, GUEST, play_through, apply_ev}`.
- Produces:
```rust
// sim_hotel_v2::situations (count.rs) — jezgra bez serde
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SituationCounts { low: u8, mid: u8, high: u8 }   // polja privatna: invarijanta samo kroz `new`
impl SituationCounts {
    pub const MIN: u8 = 3;                   // = obvezne (MANDATORY_SITUATIONS)
    pub const MAX: u8 = 8;
    pub const WEEK: SituationCounts;         // 3/4/5
    pub const MONTH: SituationCounts;        // 4/5/6
    pub fn new(low: u8, mid: u8, high: u8) -> Option<SituationCounts>; // MIN ≤ low ≤ mid ≤ high ≤ MAX
    pub fn default_for(kind: PeriodKind) -> SituationCounts;
    pub fn levels(self) -> (u8, u8, u8);
    pub fn for_occupancy(self, occupancy_bp: u16) -> u8;              // <7_500 low, <9_000 mid, inače high
}
pub fn situation_count_with(s: &HotelState, p: &Period, cat: &Catalog, c: SituationCounts) -> Result<u8, ResolveError>;
// TemplateSet: pub min_gap_weeks: u8, pub opportunity_every_weeks: u8,
//              pub counts: Option<SituationCounts>  // postavka igre; None = default_for(p.kind); sadržaj ga ne puni
// content::templates: pub const DEFAULT_MIN_GAP_WEEKS: u8 = 4; pub const DEFAULT_OPPORTUNITY_EVERY_WEEKS: u8 = 3;
// game::event
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct SituationCountsDto { pub low: u8, pub mid: u8, pub high: u8 }
// GameCreated: #[serde(default, skip_serializing_if = "Option::is_none")] pub situation_counts: Option<SituationCountsDto>,
// game::command
pub struct GameSettings { pub kind: PeriodKind, pub situations: Option<SituationCounts> } // None = zadano
// game::Game
pub fn situation_counts(&self) -> SituationCounts;
```

**Testovi čuvara** (čuvar sam uskladi i doslovne zapise `GameSettings { … }` (13 mjesta, `grep -rn "GameSettings {"
crates`) i `TemplateSet { … }` (`v2_situation_capacity.rs:226`, `v2_situation_draw_edges.rs:110`) novim poljima; sve
je to CRVENO jer se ne prevodi):
1. `crates/game/tests/situation_settings.rs` (nova; pomoćna `created(kind, situations)` zove `new_game_with` sa sjemenom
   42, `common::GUEST`, `common::hotel_ids()`):
   - `new_v2_game_writes_default_counts_for_its_round_kind`: tjedno `Some(SituationCountsDto { low: 3, mid: 4, high: 5 })`,
     mjesečno 4/5/6, `Some(new(5, 7, 8))` daje 5/7/8. Za v1 igru (`Snapshot::read`) vrijednost je `None`.
   - `counts_outside_three_to_eight_or_not_rising_are_refused`: `new` vraća `None` za (2,4,5), (3,4,9), (3,5,4) i
     (6,5,6), a `Some` za (3,3,3) i (8,8,8). Provjerava i `usize::from(SituationCounts::MIN) == MANDATORY_SITUATIONS`.
   - `log_without_counts_still_reads_with_defaults` (stari dnevnik):
```rust
let mut log: Vec<Recorded> = created(PeriodKind::Week, None).unwrap().into_iter().zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event }).collect();
let mut v = log[0].event.to_payload();
v.as_object_mut().unwrap().remove("situation_counts").expect("nova igra piše broj");
log[0].event = GameEvent::from_record("GameCreated", SCHEMA_VER, v).unwrap();
assert_eq!(Game::fold(&log).unwrap().situation_counts(), SituationCounts::WEEK);
```
   - `corrupt_counts_in_log_are_refused`: `GameCreated` s 3/9/9 → `Game::fold` daje `Err(GameError::Corrupt(_))`.
     `new_game_with` sa `situations` izvan granica nije moguć, jer `new` vraća `None`.
   - `monthly_game_with_three_has_three_in_every_round`: mjesečna igra s `new(3, 3, 3)` → `play_through(g, "sep")` →
     za svaki krug `situations_of_hotel(me, r)?.len() == 3`.
2. `crates/content/tests/v2_situation_count.rs`:
   - `monthly_count_is_proportional_to_days` (`:62-82`) postaje `monthly_count_is_four_five_six`:
     `MONTH.for_occupancy` za 7_000 / 8_000 / 9_500 daje 4/5/6;
   - po profilu preko `situation_count`: A `[4, 5, 6, 5]`, B i C `[4, 5, 6, 4]`. To je ista razina popunjenosti kao
     danas: 13→4, 17/18→5, 22→6;
   - tjedni testovi prelaze na `SituationCounts::WEEK.for_occupancy`, a očekivanja ostaju ista.
3. `crates/content/tests/v2_situation_draw_month.rs`:
   - brišu se `virtual_weeks_split_the_month_count` i test ponavljanja „od 28. dana", s očekivanjem `[5,5,5,5,2]` i
     `repeated > 0`;
   - novi `monthly_round_draws_a_template_at_most_once`: 3 profila × 300 sjemena, `month_set()`. U svakom krugu
     `drawn.len() == situation_count(...)`, nijedan predložak dvaput, svaki primjerak ima `day == p.start_day`,
     `occurrence == 1` i šifru `"{predložak}.{krug}"`;
   - isti test broji predloške koji dođu dva mjeseca zaredom i traži da ih bude `> 0` (ADR-042 t. 12: iz kruga u krug
     ponavljanje je dopušteno).
4. `crates/content/tests/v2_templates.rs`, novi `gap_and_opportunity_live_in_the_template_file`:
   - sadržaj iz repoa daje `to_core()` s `(min_gap_weeks, opportunity_every_weeks, counts) == (4, 3, None)`;
   - snimak bez ta dva ključa (`v["v2"]["situation_templates"]`, `remove`) daje 4 i 3;
   - `min_gap_weeks: 0` → `build()` je `Err`.
5. `crates/game/tests/v2_sealed.rs` (I3):
   - `Case` dobiva `situations: Option<(u8, u8, u8)>`, a `CASES: [Case; 8]`. Dodaju se `t7-broj-578` (tjedno, sjeme 7,
     odgovara, 5/7/8) i `m42-broj-333` (mjesečno, 42, šutnja, 3/3/3).
   - `play` zove novi `common::v2_game_with(snap, seed, GameSettings)`, a `v2_game_from` ostaje kao omotač.
   - `digest` iz `GameCreated` uzima i `"situation_counts": c.situation_counts`.
   - Test 3 traži slučaj sa svojim brojem za obje vrste kruga. U svakom slučaju svaki krug mora imati izvučenu situaciju
     (šifra s točkom) i najviše `high` situacija.
   - Novi test 5, `digest_reacts_to_situation_counts`: isti slučaj s 3/3/3 i sa zadanim brojem daje različit otisak.
   - Test 1 je CRVEN dok se ne zapečati nova verzija (drugi popis slučajeva).
6. `crates/api/tests/student_sees.rs:226`: makni `#[ignore]` s `monthly_round_never_repeats_a_template`. Danas je
   CRVEN s porukom „jul: predložak P-PRAONICA dvaput".
7. `crates/game/tests/situations_delegate.rs:14-17, 46-56`: lipanj ima 4 situacije, a ne 13, pa `>= 5` postaje `>= 4`
   (opis u komentaru se ispravlja). Ovo je karakterizacija starog broja.

Testovi koji moraju ostati zeleni i nepromijenjeni (karakterizacija):
- `v2_situation_draw_snapshot.rs`: tjedni otisci od 52ce161. To je dokaz da se tjedno izvlačenje nije promijenilo.
- `v1_frozen.rs`, `fixtures_sealed.rs`, test 2 i test 4 pečata;
- tjedni scenariji A6 i t05;
- `game_parity_v2.rs` (zadani broj na obje strane).

**Koraci:**
1. **Čuvar:** testovi 1–7 u jednom commitu, „Popravak B2a (čuvar): …".
   - U izvještaj ide izlaz `cargo test -p game --test situation_settings` (ne prevodi se).
   - Za test 6 ide izlaz `cargo test -p api --test student_sees -- monthly_round_never` (asercija).
2. **Graditelj, jezgra:**
   - `SituationCounts` i `round_count(d, occ) = d.templates.counts.unwrap_or(default_for(p.kind)).for_occupancy(occ)`.
   - Jedan blok u `draw_situations_with`, filtar „već u krugu" u `candidates`, i `repeat_allows` i `needs_opportunity`
     s brojem iz `TemplateSet`.
   - Za tjedni krug redoslijed izvlačenja i potrošnja kocke ostaju isti, jer je virtualni tjedan i dosad bio jedan blok.
3. **Sadržaj:** dva polja, provjera nule, `to_core` i JSON.
4. **Igra:** `SituationCountsDto` ↔ `SituationCounts`.
   - `new_game_with`: `situations.unwrap_or(default_for(kind))`, pa `Some(dto)` samo za v2.
   - `created()`: `None` → `default_for(prvi krug.period.kind)`. Neispravan zapis → `corrupt("broj situacija u postavkama")`.
   - `drawn.rs:286`: postavlja `set.counts`.
5. **Balans:** `count_band` iz `default_for` (komentar `:266` → „tjedno 3–5, mjesečno 4–6").
6. **Pečat:** digni `CORE_VERSION`, pa `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest`.
   Stari ključevi u `v2-golden.json` ostaju.
7. **Bilješke:** u `.superpowers/sdd/2026-10-14-popravak-temelja/za-dokumentaciju.md` upiši:
   - BUGS B3 → „Stanje: popravljeno B2a";
   - ADR-042 t. 12–14 provedeno;
   - MODEL_V2 i plan situacija: „mjesečno 13–22" → „4/5/6";
   - 🟨 pretpostavke iz Rizika;
   - BACKLOG: pragovi 75/90 %, ocjena 2 i „3 obvezne" još u kodu (REVIZIJA #13 ostatak; C5 za web tekst).
8. Commit „Popravak B2a: broj situacija u postavkama igre, mjesečno 4/5/6, predložak jednom u krugu".

**Provjere** (svaka zasebno, `; echo EXIT=$?`, izlaz u datoteku, cijeli `ci.yml`):
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings`
  · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` (samo kad nitko drugi ne prevodi u stablu)
  · `git diff --exit-code`.
- `cargo sqlx migrate run --source crates/store/migrations` i `SQLX_OFFLINE=false cargo sqlx prepare --workspace --check`
  (baza `hotelixsim_agenti`) · `cargo test --workspace`. Cijeli `student_sees` je zelen, bez `#[ignore]` za B3.
- Balansi: prije prve izmjene spremi osnovicu u scratchpad.
  - v1 `--seeds 100 --check` i `--seeds 1000`: `cmp` je bajt-isti.
  - v2 `--round week --seeds 1000 --check`: `diff` smije dirati samo retke odjeljka
    „== Tjedno prema mjesečnom ==" i retke „sloj 1 tjedno ≈ mjesečno" u „Mjere robota"; sve ostalo je bajt-isto.
  - v2 `--round month --seeds 1000 --check`: **brojke se mijenjaju**, pa se diže `CORE_VERSION` i obnavlja pečat A3.
    Očekuje se „najmanje 4 najviše 6" i ~19–20 po sezoni, uz EXIT=0. Ako padne brana modela (J-3 raznolikost ili
    dominantna proba), stani i vrati BLOCKED s dijagnozom. Brojke se ne štimaju, to radi L1-13.
- `corepack pnpm check:docs` · `test:docs` · `check:kod` (`draw.rs` se smanjuje) · `check:api` (bez promjene)
  · `lint` · `typecheck` · `test` · `corepack pnpm --filter @hotelixsim/web build`.
- `git diff --stat` ne smije sadržavati `openapi.json`, `apps/`, `packages/` ni zamrznute datoteke (`v2-snapshot.json`,
  `content-52ce161/`). `cargo deny` radi samo CI.

**Rizik i odluke (🟨):**
- **Mjesečni krug je jedan blok**, a ne tjedni. To je ADR-042 t. 13 doslovno („ne da se računa kao da su tjedni prošli").
  - Svi primjerci imaju dan početka kruga. Razmak od 4 tjedna zato između dva mjeseca uvijek prolazi (30 ≥ 28), pa
    ponavljanje iz kruga u krug ostaje dopušteno (t. 12).
  - Predložak s razmakom 5 ili 6 preskače jedan mjesec.
- **Prilika:** u mjesečnoj igri od srpnja vrijedi „barem jedna prilika u krugu", jer je prošli krug uvijek više od 14
  dana prije. Tako je bilo i dosad (svaki mjesec je imao 4–5 virtualnih tjedana).
- **Postavka je zapisana izričito za svaku novu v2 igru** (`Some`). Kasnija promjena zadanih brojeva u kodu ne mijenja
  staru igru. To je temelj za C1 (stara igra se čita i nakon promjene jezgre). `None` znači samo „stari dnevnik".
- **Broj ne smije padati s popunjenošću** (`low ≤ mid ≤ high`): više gostiju znači više posla (izvedeno iz t. 13, nije
  pitano). Pragovi 75/90 % ostaju u kodu, jer ih nijedna odluka ne seli.
- **Profesor mijenja broj između krugova** (t. 14): to nije u B2a. `Game::situation_counts()` vrijedi za cijelu igru.
  Događaj „postavke od kruga k" dolazi s ekranom profesora (PLATFORMA), a tada metoda dobiva krug.
- **Preračun učinka jedne situacije prema broju** (t. 14, „profesorov izbor ne mijenja pravednost") nije u B2:
  - mijenja obračun i ide uz kalibraciju (L1-13);
  - dok ga nema, svoj broj može postaviti samo test, jer API ga ne nudi.
  - Posljedica: mjesečni udio situacija u napretku (danas Brižni 19,3 %) **pada** s B2a. To je mjera robota i ne ruši
    `--check`. Scenarij A6 „mjesečno odgovori > šutnja" čuva smjer.
- **Seme u scenarijima:** lipanj sjemena 42 sada ima 4 situacije (S1, S3, kolektivna i jedna izvučena). Ako 4. nema
  voditelja ili lipanj nema lika sa slobodnim odgovorom, padaju `situations_delegate`, A6 „prepuštanje" ili A6
  „razgovor". Unaprijed dopušteno: **čuvar** u dodatnom commitu bira prvi krug (jun→sep) koji ima takvu situaciju.
  Tvrdnja testa se ne mijenja. Graditelj testove ne dira.
- **Pečat i slobodni odgovor (I3, drugi dio):** slučaj `respond` + `record_verdict` i slučaj `set_plan` čovjeka
  **ne ulaze u B2**, nego u **B1a** (voditelj 15.10.). Razlozi:
  - B1 mijenja baš taj put (mehanike najbližeg odgovora, `resolve.rs:52-79`) i ionako diže verziju, pa ga tamo
    pregledava onaj tko ga mijenja, bez dodatnog dizanja verzije (2.4.0);
  - B2 taj put ne dira, pa slučaj ovdje ne bi ništa čuvao između B2 i B1 (B3 je samo balans, a B4 mijenja sadržaj
    koji zamrznuti snimak pečata ne vidi);
  - B2a je već ~1 dan.
  - B2a priprema teren: `Case` ima postavke, a `play` zove `v2_game_with`. B1a dodaje samo nove vrste `Human`.

**Leonov dio:** — (odlučeno, ADR-042 t. 12–14). Voditelj mu **prije igranja** kaže dvije stvari:
- nakon spajanja (verzija 2.3.0) njegove probne v2 igre daju 409 i treba nova igra;
- mjesečna igra sada ima 4–6 situacija, a pojedina situacija do L1-13 teži manje.

Drugi dio kartice (B2b): [task-09b-b2b-balans-prima-broj-situacija.md](#task-b2b-balans-prima-broj-situacija-kao-ulaz-paritet-s-postavkama).

<!-- Nastavak kartice [B2](#task-b2a-broj-situacija-u-postavkama-igre-mjesečno-456-isti-predložak-najviše-jednom-u-krugu) (B2a), odvojen zbog granice veličine. -->

### Task B2b: Balans prima broj situacija kao ulaz, paritet s postavkama

**Veličina:** S, 2–3 h · **Ovisi o:** B2a · **Slijedi ga:** B3a. Nikad paralelno s B3a, jer obje diraju
`crates/balance/src/v2.rs`. · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** `balance --model v2 --round month --situations 3,4,8` vrti sezone s tim brojem, a brana „J-3 broj situacija"
mjeri prema njemu. Igra i balans uz isti broj daju istu sezonu (paritet A4 za barem dvije vrijednosti po vrsti kruga).
Paritet pada ako nijedna sezona nema izvučenu situaciju. Popis robota u paritetu više nije ručna kopija.

**Zašto:** `faza-a-review.md` I3 (paritet: `GameSettings { kind }`, `robot_season_v2(..., kind)`) i trijaža A4
(`GAME_ROBOTS` ručna kopija, `s.drawn` neprovjeren). Izvještaj revizora 03 #10 i prijedlog 3 (`--situations`).

**Files:**
- Modify (balans):
  - `crates/balance/src/v2.rs:27-30`: `OptionsV2.situations: Option<SituationCounts>`.
  - `v2.rs:55-65`: `SituationReport.counts: SituationCounts`.
  - `v2.rs:455-473`: `robot_season_v2(..., kind, situations: Option<SituationCounts>)`.
  - `v2.rs:475+`: `run_v2` daje postavku samo vrsti `opts.kind`; druga vrsta u „tjedno prema mjesečnom" ostaje na
    zadanom broju.
  - `v2.rs:713-737`: `count_band(sr.counts)`.
  - `v2_situations.rs:41-46`: `Situations::new(content, situations)` postavlja `set.counts`.
  - `v2_situations.rs:266-272`: `count_band(c: SituationCounts) -> (u64, u64)`.
  - `crates/balance/src/main.rs:6-36`: zastavica `--situations L,M,H`.
  - `main.rs:59-77`: ispis „Broj situacija (postavka): L/M/H" **samo** kad je zastavica zadana.
  - `main.rs:95-100`: uporaba.
  - Novo u `v2.rs`: `pub fn parse_situations(s: &str) -> Option<SituationCounts>`.
- Modify (igra): `crates/game/src/command.rs:17-21`: `pub const ROBOTS: [(Strategy, &str); 3]` (Saver, Caring,
  OccupancyHunter). Petlja `:86` piše `s.code()`. Zapis u `GameCreated` ostaje bajt-isti.
- Modify (testovi, čuvar):
  - `crates/balance/tests/game_parity_v2.rs`;
  - literali `OptionsV2 {` u `balance_v2.rs`, `robots_v2.rs` i `robot_measures_v2.rs` dobivaju `situations: None`.
- Create: `crates/balance/tests/situations_input.rs`.
- Ne dirati: jezgru, sadržaj, pečat (B2b ne mijenja brojke), tekstove poruka brana i `docs/`.

**Interfaces:** Consumes `SituationCounts` iz B2a i `GameSettings.situations`. Produces:
`OptionsV2 { seeds, kind, situations }`, `robot_season_v2(content, profile, strategy, seed, kind, situations)`,
`parse_situations`, `game::command::ROBOTS`.

**Testovi čuvara:**
1. `game_parity_v2.rs`:
   - `GAME_ROBOTS` = `game::command::ROBOTS.map(|(s, _)| s)`;
   - `robot_game(seed, kind, profile, strategy, situations)` i `GameSettings { kind, situations }`;
   - petlja ide po `[None, Some(X)]` po vrsti kruga: tjedno `X = new(5, 7, 8)`, mjesečno `new(3, 3, 3)`. Svoj broj se
     vrti samo sa sjemenom 42 (36 + 18 kombinacija);
   - uz svaku kombinaciju idu dvije provjere:
```rust
assert!(s.drawn.iter().any(|d| !d.is_empty()), "{at}: balans bez izvučenih situacija");
assert!(gt.iter().any(|(codes, _, _)| codes.iter().any(|c| c.contains('.'))), "{at}: igra samo sa sidrima");
```
2. `situations_input.rs` (nova):
   - `parse_situations_accepts_only_valid_counts`: `"4,5,6"` daje `Some`; `"2,4,5"`, `"3,5,4"`, `"4,5"` i `"a,b,c"`
     daju `None`.
   - `count_band_follows_the_setting`: `run_v2(&content, &OptionsV2 { seeds: 3, kind: Month, situations: new(3,3,3) })`
     daje `sr.counts.levels() == (3,3,3)` i `sr.draws.max <= 3`. Poruka J-3 u `failures_v2` za prazan ili kriv
     pojas nosi „dopušteno 3–3".
   - Svi testovi su CRVENI jer se ne prevode (nema polja ni funkcija).

**Koraci:**
1. Čuvar commita testove („Popravak B2b (čuvar): …").
2. Graditelj gradi po Files.
3. U bilješkama za dokumentaciju zapiše uporabu `--situations` (README „Lokalni razvoj", CLAUDE.md naredba balansa).
4. Commit „Popravak B2b: balans prima broj situacija, paritet s postavkama igre".

**Provjere:** cijeli popis iz B2a (fmt, clippy, core-deps, probe-mutacija, `git diff --exit-code`, sqlx prepare, test
workspace, sedam pnpm provjera i web build). Uz to:
- svi balansi (v1 100 i 1000, v2 week i month 1000) **bez zastavice** su `cmp` bajt-isti s izlazom nakon B2a. B2b ne
  mijenja brojke, pa nema dizanja verzije;
- ručno: `cargo run -q -p balance --release -- --model v2 --round month --seeds 100 --situations 3,3,3` → redak postavke
  i „najviše 3". `--situations 2,4,5` → EXIT=2 i uporaba. Oba izlaza idu u izvještaj.

**Rizik i odluke (🟨):** zastavica vrijedi samo za vrstu iz `--round`, a usporedba „tjedno prema mjesečnom" vrti drugu
vrstu sa zadanim brojem. `ROBOTS` u `game` postaje javan, jer je to jedini izvor popisa. **Leonov dio:** —.

---

## Review Focus kandidati
1. **Postavka 3 u mjesecu sa sidrima:** lipanj ima S1, S3 i kolektivnu, pa uz broj 3 student u lipnju nema nijednu
   izvučenu situaciju. Ima samo sidra i kolektivnu, a nijedan test ne pita je li to još smislena vježba.
2. **Profesor promijeni broj usred semestra:** B2 to ne podržava, jer je postavka samo u `GameCreated`. Prvi ekran
   profesora mora dodati događaj i `situation_counts(r)` po krugu, inače promjena tiho vrijedi i za prošle krugove.
3. **Udio situacija u mjesečnoj igri** pada od B2a do L1-13. Leon koji igra mjesečno između tih cigli vidi da situacije
   „malo znače". To nije kvar, ali ga treba reći prije igranja.
4. **Stari dnevnik s `.jul.2`:** izvučene situacije se čitaju iz `SituationsDrawn`, a ne izvlače ponovno, pa ih C1 smije
   čitati bez starog pravila. To vrijedi samo dok nitko ne uvede ponovno izvlačenje pri čitanju.
