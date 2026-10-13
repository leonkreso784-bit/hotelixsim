### Task 6: Mehanike situacija (§6.2) i S1–S4

**Files:**
- Modify: `crates/sim-hotel/src/mechanic.rs` (puna logika), `crates/sim-hotel/src/resolve.rs` (situacije u fazama)
- Create: `crates/sim-hotel/tests/mechanics.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mechanic {
    ServiceRecovery { compensation_cents: i64 },
    EquipmentRepair { urgent: bool },
    StaffRetention { raise_one_pct: u16, bonus_cents: i64 },
    GroupBooking { accept: bool, rooms: u16, nights: u16, discount_bp: u16, month: Month },
    CrisisCommunication { backup_cents: i64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationOutcome { pub code: String, pub mechanics: Vec<Mechanic>, pub quality: u8 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedAnswer { pub label: String, pub mechanics: Vec<Mechanic>, pub quality: u8 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationTemplate { pub code: String, pub month: Month, pub title: String, pub answers: Vec<OfferedAnswer> }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub satisfaction_pts: i32, pub morale_pts: i32, pub rating: i32, pub equipment_pts: i32,
    pub cost_cents: i64, pub extra_wage_cents: i64, pub groups: Vec<GroupBlock>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanicError { BadQuality(u8), GroupInPast, BadGroup, Calc(engine::calc::CalcError) }
pub fn effects(outcome: &SituationOutcome, month: Month, cat: &Catalog, category: Category) -> Result<Effects, MechanicError>;
impl Mechanic { pub fn direct_cost_cents(&self, cat: &Catalog, category: Category) -> engine::calc::Calc<i64>; }
impl SituationOutcome { pub fn from_answer(code: &str, a: &OfferedAnswer) -> SituationOutcome; }
```

- [ ] **Step 1: Pravila** (DEMO §6.2, ADR-013 t. 6–7; `q` = kvaliteta 1–5, inače `BadQuality`):

| mehanika | učinak |
|---|---|
| `ServiceRecovery` | zadovoljstvo += `recovery_pts_per_q·(q − 3)`; trošak += naknada |
| `EquipmentRepair` | hitno: trošak += `repair_urgent_cents`; privremeno: trošak += `repair_temporary_cents`, oprema += `repair_temporary_equipment_pts` |
| `StaffRetention` | cilj morala += `retention_pts_per_q·(q − 3)`; `extra_wage += div_round(wage_cents·raise_one_pct, 100)` (trajno, od ovog mjeseca); trošak += bonus |
| `GroupBooking` | ako `accept`: `rooms·nights > 0` (inače `BadGroup`), `month ≥ tekući` (inače `GroupInPast`); popust = `max(0, discount − (q ≥ 4 ? cut : 0))`; cijena po sobi-noći = `div_round(price_ref(ciljni mjesec)·(10000 − popust), 10000)`; dodaj `GroupBlock` |
| `CrisisCommunication` | zadovoljstvo += `crisis_pts_per_q·(q − 3)`; trošak += rezervni plan |

Granice po situaciji (zbroj mehanika jedne situacije): zadovoljstvo i moral ±`*_bound_pts`, ocjena ±`rating_bound`.
`direct_cost_cents`: naknada / popravak / `bonus + div_round(wage·raise, 100)` / 0 / rezervni plan.
U `resolve_month`: `effects` svake situacije se zbroje; grupe i `extra_wage` ulaze PRIJE tržišta (korak 2), morale u
cilj morala, oprema/zadovoljstvo/ocjena u svoje formule (već predviđeno u Tasku 5), trošak u `costs`; svaka situacija
dobiva retke `Cause::Situation(code)` za varijable na koje djeluje. Briše se grana `NotYetSupported`.

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/mechanics.rs`:

```rust
mod common;
use common::*;
use sim_hotel::*;

fn out(code: &str, q: u8, mechanics: Vec<Mechanic>) -> SituationOutcome { SituationOutcome { code: code.into(), mechanics, quality: q } }

#[test]
fn june_with_s1_s2_s3_like_outcomes() {
    let cat = test_catalog();
    let s = state_c();
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let sits = [
        out("S1", 5, vec![Mechanic::ServiceRecovery { compensation_cents: 12_000 }]),
        out("S2", 2, vec![Mechanic::StaffRetention { raise_one_pct: 0, bonus_cents: 0 }]),
        out("S3", 3, vec![Mechanic::GroupBooking { accept: true, rooms: 25, nights: 7, discount_bp: 2_500, month: Month::Jun }]),
    ];
    let r = resolve_month(&s, Month::Jun, &plan, &sits, &cat).unwrap();
    assert_eq!((r.market.sold, r.market.group_sold, r.market.individual_sold), (2_767, 175, 2_592));
    assert_eq!((r.money.rooms_revenue_cents, r.money.other_revenue_cents, r.money.costs_cents), (36_763_875, 11_029_163, 30_899_400));
    assert_eq!((r.money.gop_cents, r.money.goppar_cents), (16_893_638, 4_693));
    assert_eq!((r.people.load_hundredths, r.people.load_penalty_pts, r.people.morale_target_pts, r.people.turnover_bp), (184, 400, 5_000, 398));
    assert_eq!((r.after.satisfaction_pts, r.after.morale_pts, r.after.rating), (7_221, 5_350, 842));
    assert!(r.lines.iter().any(|l| l.cause == Cause::Situation("S1".into()) && l.var == Var::Satisfaction));
}

#[test]
fn group_booked_in_june_sells_in_august_with_good_quality_discount_cut() {
    let cat = test_catalog();
    let s = state_c();
    let sits = [
        out("S3", 4, vec![Mechanic::GroupBooking { accept: true, rooms: 25, nights: 7, discount_bp: 2_500, month: Month::Aug }]),
        out("S2", 5, vec![Mechanic::StaffRetention { raise_one_pct: 8, bonus_cents: 0 }]),
    ];
    let june = resolve_month(&s, Month::Jun, &MonthPlan::default_for(&s, Month::Jun, &cat).unwrap(), &sits, &cat).unwrap();
    assert_eq!(june.after.groups, vec![GroupBlock { month: Month::Aug, room_nights: 175, price_cents: 14_000 }]);
    assert_eq!((june.after.extra_wage_cents, june.after.morale_pts), (17_600, 5_680));
    let july = resolve_month(&june.after, Month::Jul, &MonthPlan::default_for(&june.after, Month::Jul, &cat).unwrap(), &[], &cat).unwrap();
    let aug = resolve_month(&july.after, Month::Aug, &MonthPlan::default_for(&july.after, Month::Aug, &cat).unwrap(), &[], &cat).unwrap();
    assert_eq!((aug.market.group_room_nights, aug.market.sold, aug.market.individual_sold, aug.market.f_reputation_bp), (175, 3_258, 3_083, 9_984));
    assert_eq!((aug.money.rooms_revenue_cents, aug.money.labor_cents, aug.money.gop_cents), (56_402_500, 11_317_600, 41_338_050));
    assert!(aug.after.groups.is_empty());
}

#[test]
fn situation_bounds_cap_effects() {
    let cat = test_catalog();
    let e = effects(&out("X", 5, vec![
        Mechanic::ServiceRecovery { compensation_cents: 0 },
        Mechanic::CrisisCommunication { backup_cents: 0 },
    ]), Month::Jun, &cat, Category::Four).unwrap();
    assert_eq!(e.satisfaction_pts, 800); // 400 + 600 → granica 800
}

#[test]
fn bad_quality_rejected() {
    let cat = test_catalog();
    for q in [0_u8, 6] {
        let r = effects(&out("X", q, vec![Mechanic::ServiceRecovery { compensation_cents: 0 }]), Month::Jun, &cat, Category::Four);
        assert_eq!(r, Err(MechanicError::BadQuality(q)));
    }
}

#[test]
fn group_in_past_rejected() {
    let cat = test_catalog();
    let r = effects(&out("S3", 3, vec![Mechanic::GroupBooking { accept: true, rooms: 25, nights: 7, discount_bp: 2_500, month: Month::Jun }]), Month::Jul, &cat, Category::Four);
    assert_eq!(r, Err(MechanicError::GroupInPast));
}

#[test]
fn rejected_group_has_no_effect() {
    let cat = test_catalog();
    let e = effects(&out("S3", 2, vec![Mechanic::GroupBooking { accept: false, rooms: 25, nights: 7, discount_bp: 2_500, month: Month::Aug }]), Month::Jun, &cat, Category::Four).unwrap();
    assert_eq!(e, Effects::default());
}

#[test]
fn temporary_repair_is_cheaper_but_wears_equipment() {
    let cat = test_catalog();
    let e = effects(&out("S1", 3, vec![Mechanic::EquipmentRepair { urgent: false }]), Month::Jun, &cat, Category::Four).unwrap();
    assert_eq!((e.cost_cents, e.equipment_pts), (8_000, -100));
    let u = effects(&out("S1", 3, vec![Mechanic::EquipmentRepair { urgent: true }]), Month::Jun, &cat, Category::Four).unwrap();
    assert_eq!((u.cost_cents, u.equipment_pts), (25_000, 0));
}

#[test]
fn direct_cost_for_robots() {
    let cat = test_catalog();
    let c = |m: Mechanic| m.direct_cost_cents(&cat, Category::Four).unwrap();
    assert_eq!(c(Mechanic::StaffRetention { raise_one_pct: 8, bonus_cents: 200_000 }), 217_600);
    assert_eq!(c(Mechanic::GroupBooking { accept: true, rooms: 25, nights: 7, discount_bp: 2_500, month: Month::Aug }), 0);
}
```

- [ ] **Step 3: Pokreni** → pada. **Step 4: Implementiraj.** **Step 5: Pokreni** `cargo test -p sim-hotel; echo EXIT=$?` → 0; clippy → 0.
- [ ] **Step 6: Commit** — „Jezgra 6/12: mehanike situacija (§6.2) u obračunu mjeseca".

---

