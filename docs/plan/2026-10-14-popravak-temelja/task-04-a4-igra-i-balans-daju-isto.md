### Task A4: Jedna istina o sezoni robota — igra i balans daju isto

**Veličina:** S, 2–3 h (+ do 3 h ako test nađe razilaženje) · **Ovisi o:** A1 (situacije spojene u `sloj1`; isti
paket `crates/balance`) · **Paralelno s:** kartice A2 koje ne diraju `crates/balance` ni
`crates/sim-hotel-v2/src/season.rs` · **Model:** sonnet (opus ako test pokaže razilaženje) · **Prevodi Rust:** da

**Cilj:** Postoji test koji pada čim se sezona robota u balansu i robot-hotel u igri raziđu: isto sjeme, isti profil,
ista strategija → iste situacije u svakom krugu, isto stanje hotela na početku svakog kruga, isti pokazatelji i isti
napredak. Leon i agenti nakon toga znaju da brojke balansa govore o petlji koju student stvarno igra, a ne o kopiji.

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #8; 01 nalaz 6; 03 nalaz 13. Dvije petlje sezone:
igra `crates/game/src/command.rs:121-176` (`robot_moves`, poziva ga `open_round` u `drawn.rs:327-344`) → `resolve_job`
(`resolve.rs:168-215`), a balans `crates/sim-hotel-v2/src/season.rs:125-150` (`run_robot_season`) →
`run_season_from` (`:303-361`) preko `crates/balance/src/v2_situations.rs:96-107` (`Situations::robot`). Oznake
kocke se razlikuju (`game:robot-plan`/`game:robot-answer` prema `robot:plan`/kod situacije), a testa jednakosti nema.

**Odluka o obliku (preporuka: varijanta a, test jednakosti):**
- **(a) Test jednakosti — preporučeno, ~2–3 h, rizik nizak.** Većinu posla obje petlje već dijele: `robot_plan`,
  `Strategy::choose`, `draw_for_round_with` / `collective_for_round` („isti poziv koristi sezona robota i igra",
  `season.rs:232`), `resolve_period`, `period_indices`, `progress_pts`. Razlikuje se samo spojni kod (~30 redaka
  na svakoj strani), a njega test čuva.
- **(b) Jedna funkcija — ne sad.** Igra je dnevnik događaja: plan i odgovori upisuju se pri otvaranju kruga, a obračun
  tek nakon zatvaranja i presuda. Petlja `for p in periods` ne može voditi igru. Zajednički „korak kruga" značio bi
  preuređenje `resolve_round`/`robot_moves`, koje dijele v1 i v2, pa bi bio ugrožen v1 bajt po bajt (`v1_frozen`). To
  je ~6–10 h (izvještaj 01, cijena 4) bez dobiti koju test ne daje.
- **Oznake kocke se NE izjednačuju.** Kocku troši samo strategija `Random` (`sim-hotel-v2/src/robots.rs:190-238`,
  `sim-hotel/src/robots.rs:239-242`), a igra nema nasumičnog robota (`command.rs:17-21`: Štediša, Brižni, Lovac).
  Za tri robota iz igre oznake ne mijenjaju ništa, pa test vrijedi i bez njih. Promjena oznaka u igri krši
  global-constraints („postojeći tokovi se ne diraju"), a u balansu ruši jednakost mjesečni v2 = v1 (`season.rs:4-5`,
  imenovana iznimka). Zato nema promjene `CORE_VERSION`, starih v2 igara ni v1.
- **Granica testa:** uspoređuje se igra s **jednim** robot-hotelom u utoru balansa (`HOTEL_SLOT` = 1). U pravoj igri
  kolektivnu situaciju izvlači skup od 4 hotela prema stanju hotela u utoru 1, a to je čovjek (`drawn.rs:296-318`,
  `situations/draw.rs:299-336`). Ta razlika je svjesna i ide u Review Focus.

**Files:**
- Modify: `crates/balance/src/v2.rs` (nova `robot_season_v2` iza `kind_medians` `:421-449`, ponovni izvoz
  `HOTEL_SLOT`); `crates/balance/src/v2_situations.rs:17` (`pub(crate) const HOTEL_SLOT` → `pub const`, vrijednost
  ista); `crates/balance/Cargo.toml` (`[dev-dependencies]` `game`, `uuid`); `Cargo.lock` (samo ovisnosti `balance`).
- Create: `crates/balance/tests/game_parity_v2.rs`.
- Modify samo ako test nađe razilaženje (korak 5): `crates/sim-hotel-v2/src/season.rs:303-361` (`run_season_from`)
  ili `crates/balance/src/v2_situations.rs:69-105`.
- Ne dirati: `crates/game/**` (igra je istina, vidi korak 5), `crates/sim-hotel/**`, sadržaj, oznake kocke,
  `CORE_VERSION`, `failures_v2` i `robot_measures_v2` iz A1.

**Interfaces:**
- Consumes (postoje): `game::command::{new_game_with, GameSettings}`; `game::{Game, GameEvent, Recorded, Actor,
  Controller, CoreState, ResolveStep, GameId, GuestId, HotelId}`; `game::dto::indices_dto`; `Game::{fold, open_round,
  close_round, resolve_job, apply, state_at, current_round}` i polja `rounds`, `resolved`, `hotels`;
  `HotelResolution.{situations, indices, progress_hundredths}`; `sim_hotel_v2::{situations_for, SeasonResult,
  PeriodKind, Profile, Strategy}`; `sim_hotel::progress_pts`.
- Produces:
```rust
// crates/balance/src/v2.rs
pub use crate::v2_situations::HOTEL_SLOT; // utor robot-hotela u balansu (1)
/// Sezona jednog robota točno kako je vrti balans v2: situacije iz predložaka kad ih sadržaj ima (`Situations::robot`),
/// utor `HOTEL_SLOT`, referentna tablica kategorije profila za `kind`. Ulaz testa jednakosti s igrom (A4).
pub fn robot_season_v2(
    content: &content::Content,
    profile: Profile,
    strategy: Strategy,
    seed: u64,
    kind: PeriodKind,
) -> Result<SeasonResult, ResolveError>;
```

**Testovi čuvara** (`crates/balance/tests/game_parity_v2.rs`, nova datoteka):
```rust
//! A4 (revizija temelja #8): balans mjeri istu petlju kao igra. Igra s jednim robot-hotelom u utoru balansa (bez
//! čovjeka i AI-ja) i sezona robota iz balansa (`robot_season_v2`) za isto sjeme daju iste situacije, isto stanje na
//! početku svakog kruga, iste pokazatelje i isti napredak. Nasumični robot nije u igri i troši drugu kocku.
use balance::v2::{HOTEL_SLOT, robot_season_v2};
use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::dto::indices_dto;
use game::{Actor, Controller, CoreState, Game, GameEvent, GameId, GuestId, HotelId, Recorded, ResolveStep};
use sim_hotel_v2::{PeriodKind, Profile, SeasonResult, SituationTemplate, Strategy, situations_for};
use uuid::Uuid;

const SEEDS: [u64; 2] = [3, 42];
const GAME_ROBOTS: [Strategy; 3] = [Strategy::Saver, Strategy::Caring, Strategy::OccupancyHunter];
/// Po krugu: šifre situacija (sortirane), pet pokazatelja nakon kruga, napredak od početka (stotinke).
type Trace = Vec<(Vec<String>, [u16; 5], i32)>;

fn apply(g: &mut Game, actor: Actor, event: GameEvent) {
    let seq = g.last_seq + 1;
    g.apply(Recorded { seq, actor, event }).unwrap();
}

/// v2 igra s jednim robot-hotelom (utor `HOTEL_SLOT`), odigrana do kraja sezone.
fn robot_game(seed: u64, kind: PeriodKind, profile: Profile, strategy: Strategy) -> Game {
    let ids = [1_u128, 2, 3, 4].map(|n| HotelId(Uuid::from_u128(n)));
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let events = new_game_with(GameId(Uuid::from_u128(1)), seed, &snap, GuestId(Uuid::from_u128(99)), ids,
        GameSettings { kind }).unwrap();
    let Some((actor, GameEvent::GameCreated(mut created))) = events.into_iter().next() else {
        panic!("prvi događaj nije GameCreated")
    };
    let mut hotel = created.hotels.remove(0);
    hotel.slot = HOTEL_SLOT;
    hotel.profile = profile.code().into();
    hotel.controller = Controller::Robot { strategy: strategy.code().into() };
    created.hotels = vec![hotel];
    let first = Recorded { seq: 1, actor, event: GameEvent::GameCreated(created) };
    let mut g = Game::fold(std::slice::from_ref(&first)).unwrap();
    let r = g.rounds[0];
    for (a, e) in g.open_round(r).unwrap() {
        apply(&mut g, a, e);
    }
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        apply(&mut g, Actor::System, ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else { panic!("robot ne čeka presude") };
        for (a, e) in evs {
            apply(&mut g, a, e);
        }
    }
    g
}

fn game_trace(g: &Game) -> Trace {
    let id = g.hotels[0].setup.hotel_id;
    g.rounds.iter().map(|r| {
        let h = g.resolved[r].hotels.iter().find(|h| h.hotel_id == id).unwrap();
        let mut codes: Vec<String> = h.situations.iter().map(|s| s.code.clone()).collect();
        codes.sort();
        (codes, h.indices, h.progress_hundredths)
    }).collect()
}

fn season_trace(s: &SeasonResult, anchors: &[SituationTemplate]) -> Trace {
    s.periods.iter().enumerate().map(|(k, (rep, idx))| {
        let mut codes: Vec<String> = situations_for(&rep.period, anchors).iter().map(|t| t.code.clone())
            .chain(s.drawn.get(k).into_iter().flatten().map(|d| d.code.clone())).collect();
        codes.sort();
        (codes, indices_dto(idx), sim_hotel::progress_pts(&s.start, idx).unwrap())
    }).collect()
}

#[test]
fn robot_hotel_in_game_matches_balance_season() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for profile in Profile::ALL {
            for strategy in GAME_ROBOTS {
                for seed in SEEDS {
                    let at = format!("{kind:?} {} {} sjeme {seed}", profile.code(), strategy.code());
                    let g = robot_game(seed, kind, profile, strategy);
                    let s = robot_season_v2(&content, profile, strategy, seed, kind).unwrap();
                    assert_eq!(g.rounds.len(), s.periods.len(), "{at}");
                    for (r, (rep, _)) in g.rounds.iter().zip(&s.periods) {
                        let CoreState::V2(st) = g.state_at(g.hotels[0].setup.hotel_id, *r).unwrap() else {
                            panic!("{at}: igra nije v2")
                        };
                        assert_eq!(*st, rep.before, "{at}: stanje na početku kruga {r:?}");
                    }
                    let gt = game_trace(&g);
                    assert_eq!(gt, season_trace(&s, &content.situations), "{at}");
                    assert_eq!(gt.last().unwrap().2, s.progress_pts, "{at}: napredak sezone");
                }
            }
        }
    }
}

/// Usporedba nije prazna: drugo sjeme u balansu daje drugi trag.
#[test]
fn parity_check_sees_a_different_seed() {
    let content = Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap();
    let g = robot_game(42, PeriodKind::Week, Profile::B, Strategy::Caring);
    let other = robot_season_v2(&content, Profile::B, Strategy::Caring, 43, PeriodKind::Week).unwrap();
    assert_ne!(game_trace(&g), season_trace(&other, &content.situations));
}
```
(`cargo fmt` preoblikuje retke, a sadržaj se ne mijenja.)

**Zašto je danas crven:** prevođenje pada na `unresolved import balance::v2::robot_season_v2`, na `HOTEL_SLOT`
(`pub(crate)`) i na `use of undeclared crate game` (nije dev-ovisnost). Ponašanje se ne može predvidjeti bez
pokretanja. Nakon koraka 2–3 test može biti zelen odmah: tada je čuvar budućeg razilaženja, a drugi test dokazuje da
usporedba nije prazna. Može biti i crven na jednom od poznatih šavova:
- (1) poredak situacija u krugu: igra ih slaže po težini silazno (`drawn.rs:238-242`), a sezona redom izvlačenja
  (`season.rs:333-347`). Poredak utječe na `cap_round` i redoslijed grupa (`resolve/situations.rs:53-56, 83-111`);
- (2) plan kroz `PlanDto` i `plan_for_resolve` (`model.rs:330-350`); (3) stanje kroz `DetailDto`
  (`Model::state_after`, `model.rs:436`); (4) primjerak kroz `SituationsDrawn` (`drawn_of`, `drawn.rs:117`) i
  `instance_with` prema `instance`.

**Koraci graditelja:**
1. Tek kad je A1 spojen (`git log model-v2/sloj1 --oneline -1` = commit A1): `git worktree add -b popravak/a4-sezona
   ../hotelixsim-a4 model-v2/sloj1`. Čuvar piše test i zapisuje RED (prevođenje).
2. `crates/balance/Cargo.toml`: novi odjeljak `[dev-dependencies]` s retcima `game.workspace = true` i
   `uuid.workspace = true`.
3. `v2_situations.rs:17`: `pub const HOTEL_SLOT: u8 = 1;`. U `v2.rs` ide `pub use crate::v2_situations::HOTEL_SLOT;`
   i nova funkcija:
```rust
pub fn robot_season_v2(content: &content::Content, profile: Profile, strategy: Strategy, seed: u64, kind: PeriodKind)
    -> Result<SeasonResult, ResolveError> {
    let cat = content.v2.as_ref()
        .ok_or_else(|| ResolveError::Catalog(CatalogError::Missing("v2".to_string())))?;
    let refs = category_refs(cat, kind)?;
    Situations::new(content).robot((profile, strategy, seed), kind, cat, refs_for(cat, &refs, profile)?)
}
```
4. `cargo test -p balance --test game_parity_v2; echo EXIT=$?`. Ako je zeleno, slijede provjere i commit.
5. **Ako je crveno:** poruka imenuje vrstu kruga, profil, strategiju, sjeme i krug. Graditelj nalazi koji je od šavova
   (1)–(4) uzrok i zapisuje ga u izvještaj. **Pravilo: igra je istina**, jer nju student igra i jer izmjena igre mijenja
   igre u tijeku i traži novu `CORE_VERSION`.
   - Popravlja se strana balansa (`run_season_from` ili `Situations`). Za šav (1): u `run_season_from`
     (`season.rs:333-347`) primjerci kruga slažu se kao u igri, stabilno po `weight` silazno, nakon sidara. Igra
     `run_season_from` ne zove, pa nema nove `CORE_VERSION`; pokreće se `node scripts/probe-mutacija.mjs` (dira
     jezgru) i `cargo test -p sim-hotel-v2 --test equivalence` (stari popis nije dirnut, mora ostati zelen).
   - Ako je uzrok u igri (npr. `DetailDto` ili `PlanDto` gubi polje, šav 2 ili 3), to je greška koju student osjeti:
     **stani, BLOCKED**, s dijagnozom. Kontrolor odlučuje uz A2 (zapečaćeni otisak v2 i `CORE_VERSION`).
6. Commit „Popravak A4: test jednakosti igre i balansa za sezonu robota" s izričitim putanjama, uključujući
   `Cargo.lock`.

**Provjere (točne naredbe i očekivani izlaz):** u `/c/Users/leonk/Documents/hotelixsim-a4`, svaka zasebno pa
`; echo EXIT=$?`.
- `cargo test -p balance --test game_parity_v2` → `test result: ok. 2 passed`. Ako traje dulje od ~2 min u debug
  načinu, `SEEDS` se smanjuje na `[42]`, uz bilješku u izvještaju.
- Opće provjere: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` (baza `hotelixsim_agenti`), `node scripts/check-core-deps.mjs` (balans nije jezgra, a `game`
  ostaje čist sloj) i `corepack pnpm check:kod` → sve EXIT=0.
- Balans prema izlazu prije izmjene: `cargo run -q -p balance --release -- --seeds 100 --check` → EXIT=0 i bajt-isto.
  `--model v2 --round week|month --seeds 1000 --check` → EXIT=0. Bez koraka 5 izlaz v2 je bajt-isti (`cmp`); s
  korakom 5 smiju se promijeniti samo brojke sezona sa situacijama (medijani, mjere robota, F na kraju), a ekonomija
  referentnog hotela, marže, početak i probe ostaju iste. Razlika se zapisuje u izvještaj.
- `git diff model-v2/sloj1 --stat -- crates/game crates/sim-hotel content` → prazno.

**Rizik i odluke:**
- 🟨 **Ugovor se pomiče s „kako robot igra" na „ista petlja".** Test kaže da igra i balans računaju isto za
  determinirane robote. Ništa ne kaže o nasumičnom robotu (samo balans) ni o čovjeku (prepuštanje, slobodni odgovor,
  neodgovoreno; 03 nalaz 3). Ako Leon jednom doda nasumičnog robota u igru, oznake kocke postaju važne, a test ga mora
  dodati, s istim oznakama na obje strane, kroz novu `CORE_VERSION`.
- Provjeravatelj neka gleda: test uspoređuje i stanje na početku kruga (hvata novac, ljude i odjele), ne samo
  napredak; `HOTEL_SLOT` je stvarno utor koji balans koristi; korak 5, ako se dogodio, nije dirao `crates/game`.

**Leonov dio:** ništa. Ako korak 5 nađe grešku u igri (šav 2 ili 3), kontrolor Leonu objašnjava što student vidi
drukčije.

## Review Focus kandidati
1. **Kolektivna situacija u pravoj igri** izvlači se prema stanju hotela u utoru 1 (čovjek) i mora proći granice sva
   četiri hotela (`situations/draw.rs:303-329`). Robot u utorima 2–4 zato vidi drukčiji niz situacija nego u balansu
   (utor 1, sam). Test pokriva samo igru s jednim hotelom. Za razred (ADR-037, N hotela) treba balans s N hotela
   (03, cijena 6).
2. **Čovjekov put nitko ne mjeri:** prepuštanje voditelju (q = 2), slobodni odgovor s neutralnom presudom (3) i šutnja
   (1) postoje samo u igri (`game/src/resolve.rs:26-110`). Student koji prepusti sve dopuštene situacije igra sezonu
   koju balans nikad nije vrtio.
3. **Ako nasumični robot uđe u `ROBOTS`** (`command.rs:17-21`), njegova igra i njegov balans tiho se razilaze (druge
   oznake kocke), a ovaj test to neće vidjeti, jer ga ne uključuje.
