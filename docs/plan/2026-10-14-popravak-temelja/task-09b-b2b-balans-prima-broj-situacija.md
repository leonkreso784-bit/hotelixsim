<!-- Nastavak kartice [B2](task-09-b2-broj-situacija-u-postavkama.md) (B2a), odvojen zbog granice veličine. -->

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
