### Task 13: Račun dobiti po USALI-ju i odstupanje od budžeta (čisto)

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 9 · **Paralelno s:** Task 10, 11, 12 · **Model:** sonnet

**Cilj:** čista funkcija koja od `PeriodAccounts` složi račun dobiti redoslijedom USALI-ja (11. izdanje, bez naknade za
upravljanje i informatike — 🟨 P10) i usporedi ostvareno s budžetom po retku.

**Files:**
- Create: `crates/sim-hotel-v2/src/usali.rs`, `crates/sim-hotel-v2/tests/usali.rs`,
  `crates/sim-hotel-v2/tests/common/usali.rs` (`pub fn june_accounts() -> PeriodAccounts`, ručno složen iz brojki
  niže; u podmapi `common/`, pa ga cargo ne prevodi kao zaseban test)
- Modify: `crates/sim-hotel-v2/src/lib.rs` (`pub mod usali;`)
- Ne dirati: sve ostalo (i `tests/common/mod.rs`).

**Interfaces (Produces):**

```rust
// crates/sim-hotel-v2/src/usali.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptAccount { pub code: DeptCode, pub class: UsaliClass, pub revenue_cents: i64, pub payroll_cents: i64, pub cost_of_sales_cents: i64, pub other_cents: i64, pub expenses_cents: i64, pub profit_cents: i64, pub margin_bp: Option<i64> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndistributedAccount { pub code: DeptCode, pub kind: UndistributedKind, pub payroll_cents: i64, pub other_cents: i64, pub total_cents: i64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsaliStatement {
    pub days: u32, pub room_nights: u32,
    /// Smještaj, hrana i piće, ostali operativni, ostali prihodi (najam, parking) — redom `UsaliClass::rank`, pa kod.
    pub departments: Vec<DeptAccount>,
    pub total_revenue_cents: i64, pub total_departmental_expenses_cents: i64, pub departmental_profit_cents: i64,
    pub undistributed: Vec<UndistributedAccount>, pub total_undistributed_cents: i64,
    pub gop_cents: i64, pub gop_margin_bp: Option<i64>, pub goppar_cents: Option<i64>,
    pub fixed_charges: Vec<(String, i64)>, pub total_fixed_charges_cents: i64,
    pub ebitda_cents: i64, pub ebitda_margin_bp: Option<i64>,
    pub interest_cents: i64, pub capex_cents: i64,
}
pub fn statement(a: &PeriodAccounts) -> Calc<UsaliStatement>;
/// Zbroj više krugova (npr. sezona do sada); marže i GOPPAR iz zbrojeva.
pub fn sum_statements(xs: &[UsaliStatement]) -> Calc<UsaliStatement>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section { Revenue, DepartmentalExpenses, DepartmentalProfit, Undistributed, Gop, FixedCharges, Ebitda }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind { Revenue, Expense, Result }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarianceLine { pub section: Section, pub code: String, pub kind: LineKind, pub is_total: bool, pub actual_cents: i64, pub budget_cents: i64, pub variance_cents: i64, pub favorable: bool }
/// Retci oba izvješća (unija kodova; kojeg nema → 0), redom sekcija; `variance = actual − budget`;
/// povoljno: prihod i rezultat ≥ 0, trošak ≤ 0. Svaka sekcija s više redaka ima i redak `is_total` (kod "total");
/// sekcija Gop ima jedan redak koda "gop", sekcija Ebitda jedan redak koda "ebitda".
pub fn budget_variance(actual: &UsaliStatement, budget: &UsaliStatement) -> Vec<VarianceLine>;
```

Pravila: rashodi odjela = plaće + `cost_of_sales` + `other_direct` (održavanje **nije** ovdje, nego u „Održavanju");
`margin_bp = div_round(profit·10 000, revenue)` ili `None` kad je prihod 0; `room_nights = rooms·days`;
`goppar = div_round(gop, room_nights)`; neraspoređeni: plaće = `payroll`, ostalo = `fixed + per_room + Σ assigned`;
`ebitda = gop − Σ naknade`; kamata i obnova su ispod EBITDA, samo za prikaz.

- [ ] **Step 1: Testovi (padaju)** — `tests/usali.rs` (ručno složeni `PeriodAccounts` s tri odjela: Smještaj prihod
  35 000 000, plaće 5 200 000, izravno 5 702 400; restoran iz Taska 9 (6 720 000 / 3 780 000 / 2 150 400 / 0);
  parking `Rentals` 699 900; neraspoređeni admin 5 905 000, sales 1 000 000 + marketing 500 000, maintenance
  1 500 000 + održavanje 2 300 000, energy 2 500 000 + 2 592·600; naknade insurance 800 000, lease 4 000 000,
  property_tax 700 000; 120 soba, 30 dana):

```rust
use sim_hotel_v2::usali::*;

// tests/common/usali.rs: pub fn june_accounts() -> PeriodAccounts (brojke iz opisa gore); podmapa se ne prevodi kao test.
#[path = "common/usali.rs"]
mod fixtures;

#[test]
fn statement_identities_hold() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    assert_eq!(s.total_revenue_cents, 35_000_000 + 6_720_000 + 699_900);
    let expenses: i64 = s.departments.iter().map(|d| d.expenses_cents).sum();
    assert_eq!(s.departmental_profit_cents, s.total_revenue_cents - expenses);
    assert_eq!(s.gop_cents, s.departmental_profit_cents - s.total_undistributed_cents);
    assert_eq!(s.total_fixed_charges_cents, 5_500_000);
    assert_eq!(s.ebitda_cents, s.gop_cents - 5_500_000);
    assert_eq!(s.room_nights, 3_600);
    assert_eq!(s.goppar_cents, Some((s.gop_cents * 2 + 3_600) / 7_200));
}

#[test]
fn rooms_come_first_and_rentals_have_no_departmental_expense() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let codes: Vec<&str> = s.departments.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["rooms", "restaurant", "parking"]);
    assert_eq!(s.departments[2].expenses_cents, 0);
    assert_eq!(s.departments[1].margin_bp, Some(1_175));
}

#[test]
fn maintenance_spend_sits_in_the_maintenance_service_not_in_departments() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let pom = s.undistributed.iter().find(|u| u.code.as_str() == "maintenance").unwrap();
    assert_eq!(pom.other_cents, 1_500_000 + 2_300_000);
}

#[test]
fn variance_signs_follow_line_kind() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut worse = fixtures::june_accounts();
    worse.departments.get_mut(&sim_hotel_v2::department::DeptCode::rooms()).unwrap().revenue_cents += 100_000;
    let b = statement(&worse).unwrap(); // budžet s većim prihodom soba
    let v = budget_variance(&a, &b);
    let rooms_rev = v.iter().find(|l| l.section == Section::Revenue && l.code == "rooms").unwrap();
    assert_eq!((rooms_rev.variance_cents, rooms_rev.favorable), (-100_000, false));
    let rooms_exp = v.iter().find(|l| l.section == Section::DepartmentalExpenses && l.code == "rooms").unwrap();
    assert_eq!((rooms_exp.variance_cents, rooms_exp.favorable), (0, true));
}

#[test]
fn variance_lists_departments_missing_from_the_budget_with_zero() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut no_parking = fixtures::june_accounts();
    no_parking.departments.remove(&sim_hotel_v2::department::DeptCode("parking".into()));
    let b = statement(&no_parking).unwrap();
    let line = budget_variance(&a, &b).into_iter().find(|l| l.section == Section::Revenue && l.code == "parking").unwrap();
    assert_eq!((line.actual_cents, line.budget_cents, line.favorable), (699_900, 0, true));
}

#[test]
fn four_weeks_sum_like_one_statement() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let sum = sum_statements(&[s.clone(), s.clone()]).unwrap();
    assert_eq!(sum.gop_cents, 2 * s.gop_cents);
    assert_eq!(sum.room_nights, 7_200);
    assert_eq!(sum.gop_margin_bp, s.gop_margin_bp);
}

#[test]
fn margins_are_none_without_revenue() {
    let mut a = fixtures::june_accounts();
    for d in a.departments.values_mut() {
        d.revenue_cents = 0;
    }
    assert_eq!(statement(&a).unwrap().gop_margin_bp, None);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2 --test usali` → FAIL.
- [ ] **Step 3: Implementiraj** `usali.rs` (sve kroz `engine::calc`).
- [ ] **Step 4:** `cargo test -p sim-hotel-v2; echo EXIT=$?`; `node scripts/probe-mutacija.mjs; echo EXIT=$?`.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/13: račun dobiti po USALI-ju i odstupanje od budžeta".

---

