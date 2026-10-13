### Task 14: Igra — odjeli, račun dobiti, budžet i pokazatelji za v2

**Veličina:** M, 3–4 h · **Ovisi o:** Task 7, Task 12, Task 13 · **Paralelno s:** Task 16 · **Model:** sonnet

**Cilj:** `game` sprema račune odjela u obračunu i nudi poglede: odjeli hotela, jedan odjel s poviješću, račun dobiti
kruga ili sezone uz budžet (🟨 P8), pokazatelji sloja 0 s danima razdoblja.

**Files:**
- Create: `crates/game/src/departments.rs`, `crates/game/tests/departments.rs`
- Modify: `crates/game/src/{lib,model,dto_v2,kpi,error,game}.rs`
- Ne dirati: jezgra, `crates/api/**`, `crates/app/**` (osim mapiranja nove greške u `crates/app/src/error.rs`).

**Interfaces:**
- Consumes: Task 12 (`PeriodReport.accounts`), Task 13 (`statement`, `sum_statements`, `budget_variance`).
- Produces:

```rust
// crates/game/src/dto_v2.rs
pub struct DetailDto { pub after: HotelStateV2Dto, #[serde(default, skip_serializing_if = "Option::is_none")] pub accounts: Option<PeriodAccountsDto> }
// PeriodAccountsDto, DeptOutcomeDto, UndistributedOutcomeDto — serde-zrcala polje po polje

// crates/game/src/departments.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentView {
    pub code: String, pub name: String, pub class: UsaliClass, pub open: bool, pub can_close: bool,
    pub permanent: u16, pub seasonal: u16, pub condition_pts: i32,
    /// `None` za odjele bez gostiju (neraspoređeni, najam, parking).
    pub guest_satisfaction_pts: Option<i32>,
    pub last: Option<(RoundKey, DeptOutcome)>,
    pub undistributed_last: Option<(RoundKey, UndistributedOutcome)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentsView { pub round: Option<RoundKey>, pub departments: Vec<DepartmentView> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentDetail { pub department: DepartmentView, pub history: Vec<(RoundKey, DeptOutcome)> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlScope { Round, Season }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlView {
    pub round: RoundKey, pub scope: PlScope,
    pub actual: UsaliStatement, pub budget: UsaliStatement, pub lines: Vec<VarianceLine>,
    /// Ime za svaki kod retka (odjeli iz profila, naknade iz kataloga).
    pub names: BTreeMap<String, String>,
}
impl Game {
    pub fn departments(&self, h: HotelId) -> Result<DepartmentsView, GameError>;           // v1 → WrongModel
    pub fn department(&self, h: HotelId, code: &str) -> Result<DepartmentDetail, GameError>; // UnknownDepartment
    /// `round = None` → zadnji obračunati krug; nijedan obračunat → `NothingResolved`.
    pub fn profit_and_loss(&self, h: HotelId, round: Option<RoundKey>, scope: PlScope) -> Result<PlView, GameError>;
}
// crates/game/src/error.rs — dodano: UnknownDepartment, NothingResolved
```

Budžet (🟨 P8): `Model::budget_accounts(content, profile, rounds)` (u `model.rs`) vrti `run_season_with` od početnog
stanja profila, sa `PeriodPlan::default_for` u svakom krugu i bez situacija, po istom kalendaru igre; računa se pri
čitanju (ne sprema se). Za `PlScope::Season` zbrajaju se krugovi od prvog do traženog (`sum_statements`) i za
ostvareno i za budžet. Pokazatelji (`kpi.rs`, sloj 0): `days` iz `round_info`, izravni trošak po noći za v2 iz odjela
Smještaj (`direct_per_use_cents`), kroz `Model::rooms_direct_cost`.

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/departments.rs` (koristi `common::v2_weekly_game_resolved_through`
  iz Taska 8):

```rust
mod common;

use game::*;

#[test]
fn departments_of_profile_after_first_week() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let h = common::hotel_ids()[0];
    let v = g.departments(h).unwrap();
    assert_eq!(v.round, Some(RoundKey::Week(1)));
    assert!(v.departments.iter().any(|d| d.code == "rooms" && d.last.is_some()));
    let admin = v.departments.iter().find(|d| d.code == "admin").unwrap();
    assert_eq!(admin.guest_satisfaction_pts, None);
    assert!(admin.undistributed_last.is_some());
}

#[test]
fn department_history_has_one_entry_per_resolved_round() {
    let g = common::v2_weekly_game_resolved_through("t03");
    let d = g.department(common::hotel_ids()[0], "rooms").unwrap();
    assert_eq!(d.history.iter().map(|(r, _)| r.code()).collect::<Vec<_>>(), ["t01", "t02", "t03"]);
    assert!(matches!(g.department(common::hotel_ids()[0], "casino"), Err(GameError::UnknownDepartment)));
}

#[test]
fn pl_lines_align_actual_and_budget() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let pl = g.profit_and_loss(common::hotel_ids()[0], None, PlScope::Round).unwrap();
    assert_eq!(pl.round, RoundKey::Week(2));
    assert_eq!(pl.actual.ebitda_cents, pl.actual.gop_cents - pl.actual.total_fixed_charges_cents);
    for l in &pl.lines {
        assert_eq!(l.variance_cents, l.actual_cents - l.budget_cents);
        assert!(pl.names.contains_key(&l.code) || l.is_total || ["gop", "ebitda"].contains(&l.code.as_str()), "{}", l.code);
    }
    let gop = pl.lines.iter().find(|l| l.code == "gop").unwrap();
    assert_eq!(gop.actual_cents, g.resolved[&RoundKey::Week(2)].hotels[0].report.money.gop_cents);
}

#[test]
fn season_scope_sums_rounds() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let h = common::hotel_ids()[0];
    let one = g.profit_and_loss(h, Some(RoundKey::Week(1)), PlScope::Round).unwrap();
    let two = g.profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Round).unwrap();
    let season = g.profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Season).unwrap();
    assert_eq!(season.actual.gop_cents, one.actual.gop_cents + two.actual.gop_cents);
}

#[test]
fn v1_game_has_no_departments_and_no_pl() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.departments(h).unwrap_err(), GameError::WrongModel);
    assert_eq!(g.profit_and_loss(h, None, PlScope::Round).unwrap_err(), GameError::WrongModel);
}

#[test]
fn pl_before_first_resolution_is_nothing_resolved() {
    let g = common::v2_weekly_game_resolved_through("none");
    assert_eq!(g.profit_and_loss(common::hotel_ids()[0], None, PlScope::Round).unwrap_err(), GameError::NothingResolved);
}

#[test]
fn weekly_kpi_uses_seven_days() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let k = g.hotel_kpi(common::hotel_ids()[0]).unwrap();
    assert_eq!(k[0].kpi.room_nights, 120 * 7);
}
```

(`v2_weekly_game_resolved_through("none")` vraća igru bez ijednog obračuna — dodaj tu granu u pomoćnik.)

- [ ] **Step 2:** `cargo test -p game --test departments` → FAIL.
- [ ] **Step 3: Implementiraj.** Grananje (v1 → `WrongModel`, budžet, izravni trošak) samo u `model.rs`.
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/14: igra — odjeli, račun dobiti s budžetom, pokazatelji po razdoblju".

---

