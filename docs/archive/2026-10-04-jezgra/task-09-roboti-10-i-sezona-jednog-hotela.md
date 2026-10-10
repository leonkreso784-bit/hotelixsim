### Task 9: Roboti (§10) i sezona jednog hotela

**Files:**
- Create: `crates/sim-hotel/src/robots.rs`, `crates/sim-hotel/src/season.rs`, `crates/sim-hotel/tests/robots.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strategy { Saver, Caring, OccupancyHunter, Random }
impl Strategy {
    pub const ALL: [Strategy; 4];
    pub fn code(self) -> &'static str;   // "stedisa", "brizni", "lovac", "nasumicni"
    pub fn plan(self, s: &HotelState, m: Month, cat: &Catalog, rng: &mut engine::Rng) -> Result<MonthPlan, ResolveError>;
    pub fn choose<'a>(self, answers: &'a [OfferedAnswer], cat: &Catalog, category: Category, rng: &mut engine::Rng) -> Result<&'a OfferedAnswer, ResolveError>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonResult { pub start: IndexSet, pub months: Vec<(MonthReport, IndexSet)>, pub progress_pts: i32 }
pub fn run_season(profile: Profile, strategy: Strategy, seed: u64, cat: &Catalog, situations: &[SituationTemplate], refs: &RefTable)
    -> Result<SeasonResult, ResolveError>;
pub fn run_season_with(profile: Profile, seed: u64, cat: &Catalog, situations: &[SituationTemplate], refs: &RefTable,
    planner: &mut dyn FnMut(&HotelState, Month) -> Result<MonthPlan, ResolveError>,
    chooser: &mut dyn FnMut(&SituationTemplate) -> Result<usize, ResolveError>) -> Result<SeasonResult, ResolveError>;
```
`run_season` = `run_season_with` sa strategijom. Kocka (bez `as`): `pos = u64::try_from(m.position())`; plan mjeseca
`Rng::derive(seed, &[pos, label("robot:plan")])`; izbor odgovora `Rng::derive(seed, &[pos, label(&template.code)])`.
`progress_pts` = `progress_pts(start, indeksi zadnjeg mjeseca)`.

- [ ] **Step 1: Pravila robota** (DEMO §10; cijena se zaokružuje na cijeli euro: `div_round(x, 100)·100`):

| | Štediša (`Saver`) | Brižni (`Caring`) | Lovac (`OccupancyHunter`) | Nasumični (`Random`) |
|---|---|---|---|---|
| cijena | ref | ref·105 % | ref·85 % | jednoliko, cijeli euro, 60–150 % ref |
| marketing | 0 | 500 000 | 2 500 000 | 0–4 000 000 u koracima 100 000 |
| sezonci | plan − 5 | `ceil(rooms·base_bp / 18000) − permanent` | današnji broj | jednoliko u dopuštenom rasponu |
| plaća | 95 | 103 | 100 | 90–115 |
| obuka | ne | Basic | ne | jedna od tri |
| održavanje | Deferred | Regular | Regular | jedno od tri |
| obnova · odgovaranje | ne · ne | ne · ne | ne · ne | ne · nasumično |
| situacije | najmanji `direct_cost` (izjednačeno: niži q, pa raniji) | najviši q (izjednačeno: raniji) | grupa: prihvat s najvećim popustom; inače srednji po q (stabilno sortiraj po q, uzmi indeks `len/2`) | jednoliko |

Sezonci svake strategije se na kraju ograniče na dopušteni raspon (0–40 i ±korak od današnjeg), da plan uvijek prođe `validate`.

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/robots.rs`:

```rust
mod common;
use common::*;
use engine::Rng;
use sim_hotel::*;

#[test]
fn robot_plans_in_june_from_profile_c() {
    let cat = test_catalog();
    let s = state_c();
    let mut rng = Rng::from_seed(1);
    let p = |st: Strategy, rng: &mut Rng| st.plan(&s, Month::Jun, &cat, rng).unwrap();
    let saver = p(Strategy::Saver, &mut rng);
    assert_eq!((saver.price_cents, saver.seasonal_target, saver.wage_pct, saver.maintenance), (13_500, 15, 95, Maintenance::Deferred));
    let caring = p(Strategy::Caring, &mut rng);
    assert_eq!((caring.price_cents, caring.marketing_cents, caring.seasonal_target, caring.wage_pct, caring.training), (14_200, 500_000, 18, 103, Training::Basic));
    let hunter = p(Strategy::OccupancyHunter, &mut rng);
    assert_eq!((hunter.price_cents, hunter.marketing_cents, hunter.seasonal_target), (11_500, 2_500_000, 20));
    let aug = Strategy::Caring.plan(&s, Month::Aug, &cat, &mut rng).unwrap();
    assert_eq!(aug.seasonal_target, 26); // ceil(120 · 8 300 / 18 000) = 56 − 30
}

#[test]
fn random_robot_plans_are_valid() {
    let cat = test_catalog();
    let s = state_c();
    for seed in 0..200 {
        let mut rng = Rng::derive(seed, &[]);
        for m in Month::SEASON {
            let p = Strategy::Random.plan(&s, m, &cat, &mut rng).unwrap();
            assert_eq!(p.validate(&s, m, &cat), Ok(()), "sjeme {seed}, {m:?}: {p:?}");
        }
    }
}

fn s1() -> Vec<OfferedAnswer> {
    let a = |q: u8, comp: i64, urgent: bool| OfferedAnswer { label: format!("q{q}"), quality: q,
        mechanics: vec![Mechanic::ServiceRecovery { compensation_cents: comp }, Mechanic::EquipmentRepair { urgent }] };
    vec![a(5, 12_000, true), a(3, 0, true), a(1, 94_500, false), a(1, 0, false)]
}

#[test]
fn robots_choose_answers_by_their_rule() {
    let cat = test_catalog();
    let mut rng = Rng::from_seed(3);
    let answers = s1();
    let pick = |st: Strategy, rng: &mut Rng| st.choose(&answers, &cat, Category::Four, rng).unwrap().label.clone();
    assert_eq!(pick(Strategy::Saver, &mut rng), "q1");            // 8 000 centi, najjeftiniji
    assert!(std::ptr::eq(Strategy::Saver.choose(&answers, &cat, Category::Four, &mut rng).unwrap(), &answers[3]));
    assert_eq!(pick(Strategy::Caring, &mut rng), "q5");
    assert_eq!(pick(Strategy::OccupancyHunter, &mut rng), "q3"); // srednji po q: [q1, q1, q3, q5][2]
}

#[test]
fn hunter_accepts_group_on_agency_terms() {
    let cat = test_catalog();
    let g = |q: u8, accept: bool, disc: u16, m: Month| OfferedAnswer { label: format!("{q}-{disc}"), quality: q,
        mechanics: vec![Mechanic::GroupBooking { accept, rooms: 25, nights: 7, discount_bp: disc, month: m }] };
    let answers = vec![g(5, true, 2_000, Month::Sep), g(3, true, 1_500, Month::Aug), g(1, true, 2_500, Month::Aug), g(2, false, 2_500, Month::Aug)];
    let mut rng = Rng::from_seed(0);
    assert_eq!(Strategy::OccupancyHunter.choose(&answers, &cat, Category::Four, &mut rng).unwrap().label, "1-2500");
}

#[test]
fn season_is_reproducible() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let sits = vec![SituationTemplate { code: "S1".into(), month: Month::Jun, title: "Klima".into(), answers: s1() }];
    for st in Strategy::ALL {
        let a = run_season(Profile::A, st, 99, &cat, &sits, &refs).unwrap();
        let b = run_season(Profile::A, st, 99, &cat, &sits, &refs).unwrap();
        assert_eq!(a, b, "{st:?}");
        assert_eq!(a.months.len(), 4);
    }
}

#[test]
fn passive_profile_c_season_scores_zero() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let r = run_season_with(Profile::C, 0, &cat, &[], &refs,
        &mut |s, m| Ok(MonthPlan::default_for(s, m, &cat)?), &mut |_| Ok(0)).unwrap();
    assert_eq!(r.start, IndexSet::from_hundredths([5_000; 5]).unwrap());
    assert_eq!(r.progress_pts, 0);
}
```

- [ ] **Step 3–5:** pokreni (pada) → implementiraj → `cargo test -p sim-hotel; echo EXIT=$?` i clippy → 0.
- [ ] **Step 6: Commit** — „Jezgra 9/12: roboti (§10) i sezona hotela".

---

