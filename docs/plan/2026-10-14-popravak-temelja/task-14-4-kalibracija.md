Dio 4 cigle [L1-13](task-14-l1-13-kalibracija.md) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 4 · kalibracija (A2, A3, A4, A8) i 2.6.0 — `l113-4-kalibracija`

**Cilj:** uključiti dijelove 1–3 u repou, naći brojke s kojima vlastiti najbolji nosi **15–25 %** za A, B i C tjedno i mjesečno, klik
**70–75 %** težine vlastitog (brana ≤ 75 %), lanac pozitivan, `SilenceNotBetter` 0; zapečatiti 2.6.0.

**Files:**
- `content/hotel/v2/economy.json`: `repair_urgent_condition_pts` 100 → ključ se briše (0), `fault_condition_pts` 100, `fault_fixed_condition_pts`
  200, `significance_bp` (početak 15 000 / 10 000 / 6 000), `click_gain_bp` (ponovno izmjereno).
- `content/hotel/v2/situation_templates.json`: `guest_extra_slots: true`; `chance` i `when.dept_condition_pts` 9 kvarova prostora
  (inačica iz tablice A2); A4 trošak na slabijem odgovoru (popis daje čuvar testom; grubo: P-VJENCANJE, P-LEZALJKE, P-KONGRES,
  P-GOST-POZLILO, P-LJUBIMAC, P-BUCNA-SOBA, P-POHVALA, P-DIJETE-IZGUBLJENO, P-RACUN-GRESKA, P-SLOBODNI-DANI, P-PROSIDBA); A8 `unanswered`
  za 16 predložaka s popisa `crates/content/src/templates_principles_debt.rs:15-35`.
- `content/hotel/v2/periods.json:7` i `:15` (`situation_effect_bp` 350 mjesečno, 325 tjedno → izmjereno).
- `crates/content/src/templates_principles_debt.rs:1-35` (popis `SilenceNotBetter` prazan, opis).
- `crates/balance/src/v2_answers.rs:268-269` (`WEIGHT_BAND_BP` → `(1_500, 2_500)`), brisanje `:271-280` (`ORDER_DEBT_FLOOR`, `ORDER_DEBT`),
  `:336` (oprost) i `:345-356` (petlja „B3 dug"); `crates/balance/src/v2.rs:28-29` (izvoz).
- `crates/balance/src/v2_faults.rs` (nova): blok A2 (niže); ožičenje u `v2.rs` najviše ~20 redaka (granica 1000).
- Testovi: `crates/balance/tests/answer_gates_v2.rs:12`, `:18-21` (brisanje), `:39-51` (`clean`: najbolji `ruler / 4` = 20 %, klik
  `ruler / 6` = 14,3 %, omjer 71 %), `:82` (`(1_500, 2_500)`), `:198-215` (`ORDER_DEBT_FLOOR` → `-25` doslovno), brisanje `:217-234`,
  `:255-264` (rub: Δ = ravnalo / 3 = 25 % prolazi, + 1 pada; donji rub 15 %); `crates/content/tests/v2_situation_space.rs:26`, `:42`
  (100 → 0), `crates/content/tests/v2_principles.rs:287`; test dijela 1 `repo_fault_keys_are_off_until_part_four` → `repo_fault_keys_are_on`.
  „Grana `clean(Month)`" iz obrisa više ne postoji (`clean` je od B1a jedan oblik); ništa za brisanje.
- Pečat: `crates/game/tests/v2_sealed.rs:31-40` (`Case.l113: bool`), `:41-128` (tri nova slučaja: `t7-odgovara-l113`, `m7-klik-l113`
  uz `OwnWords`, `t42-sutnja-l113`), `:189-` (`play`: uz `l113` zamrznuti snimak dobiva **u testu zapisane** ključeve dijela 4 i
  `guest_extra_slots`), novi `digest_reacts_to_l113_keys` uz `:428-438`; `crates/sim-hotel-v2/Cargo.toml:3` `2.5.0` → `2.6.0` i
  `Cargo.lock`; `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest` (pečat 2.5.0 ostaje).
- Ne dirati: `plan_ruler` (`v2_answers.rs:287-296`, mijenja se samo Leonovom odlukom), `fixtures_v2/v2-snapshot.json`, `prompts/`.

**A2 — tablica mjerenja koju Leon vidi prije spajanja.** Novi blok balansa (zadani plan, put „najbolji", održavanje **svih** odjela
postavljeno kao u probi `ProbeV2::MaintenanceAll`, `crates/balance/src/v2.rs:252-276`):
```
== Kvarovi prostora (A2): zadani plan, najbolji odgovor; održavanje svih odjela pojačano / zadano / odgođeno ==
  kvarova po sezoni, medijan:            A 2 / 4 / 9   B … C …
  stanje Smještaja na kraju, medijan:    A 71,0 / 64,5 / 48,2 …
  napredak na kraju, medijan:            A … / … / …
```
Inačice (čestoća 9 kvarova prostora, ostalo isto) 🟨 [PROC]: **I0** današnja (osnova 2, ×3 ispod praga stanja); **I1** osnova 1, ×4;
**I2** osnova 1, ×6, prag +10 bodova; **I3** osnova 0 izvan `when` (kvar samo kad je prostor loš), ×8. Za svaku: kopija
`cp -r content/hotel <scratchpad>/l113-I<n>`, izmjena `chance`/`when` skriptom (`node`, zapisana u izvještaj), pa
`cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check --content <scratchpad>/l113-I<n>` i isto `--round month`.
Leonova tablica (`.superpowers/sdd/2026-10-14-popravak-temelja/l113-4-a2-tablica.md`, sažetak u izvještaju):

| inačica | čestoća (osnova, ×, prag) | kvarova/sezonu tjedno A·B·C: pojačano / odgođeno | omjer odgođeno : pojačano | stanje Smještaja na kraju (pojačano / odgođeno) | težina vlastitog tjedno A·B·C | mjesečno A·B·C | klik : vlastiti | `--check` |
|---|---|---|---|---|---|---|---|---|

Voditelj bira inačicu „pravedno": dobro održan hotel ima malo kvarova (cilj ≤ 1 na mjesec 🟨), zapušten barem trostruko, a sve brane
prolaze. Leon vidi tablicu prije spajanja; spaja se tek uz njegov OK.

**Testovi čuvara** (RED: brane i popisi stari):
1. `weight_band_is_fifteen_to_twenty_five` (`WEIGHT_BAND_BP == (1_500, 2_500)`, rubovi 15 % i 25 % prolaze, izvan padaju).
2. `debt_mechanism_is_gone` (nema „B3 dug" ni oprosta; korak ≤ 0 uvijek pada, i mjesečno).
3. `silence_never_better_has_no_debt` (`KNOWN_VIOLATIONS` za `SilenceNotBetter` prazan; učitavanje repoa prolazi).
4. `weaker_answer_costs_where_only_best_did` (A4: nijedan predložak nema trošak samo u najboljem odgovoru; točan popis kao karakterizacija).
5. `fault_report_prints_three_maintenance_levels` (blok A2) i `well_kept_hotel_has_fewer_faults` (pojačano ≤ zadano ≤ odgođeno, medijan, 3 sjemena).
6. `repo_fault_keys_are_on`, `repo_guest_extra_slots_on`, `repo_significance_factors_set`.
7. Pečat: `digest_reacts_to_l113_keys`; `digest_cases_cover_rounds_profiles_and_answer_paths` traži barem jedan slučaj s kvarom i klikom.

**Koraci graditelja:** (1) uključiti ključeve dijelova 1–3 s početnim brojkama, izmjeriti (tablica „prije kalibracije"); (2) A8 i A4
(sadržaj), izmjeriti; (3) A2 tablica i izbor inačice; (4) `situation_effect_bp` dok vlastiti nije 15–25 % na svih 6 stupaca (A3: broj
situacija se ne dira); (5) faktori razina (ključna mora nositi više od svakodnevne i nakon kalibracije); (6) `click_gain_bp` na 70–75 %;
(7) pojas, brisanje duga, testovi; (8) `CORE_VERSION` 2.6.0 i pečat; (9) sve provjere i tablica prije/poslije za Leona.

**Rizik:** `situation_effect_bp` množi samo bodove, ne novac i grupe; ako ni 0 ne spušta težinu na 25 %, slabi se novac odgovora (A3 kaže
„novac i bodovi odgovora") — javiti voditelju s brojkama prije izmjene iznosa naknada.
