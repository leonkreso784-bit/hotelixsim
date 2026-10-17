Dio 1 cigle [L1-13](task-14-l1-13-kalibracija.md) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 1 · kvar i popravak (A1) — `l113-1-kvar`

**Cilj:** kvar prostora spušta stanje prostora za `fault_condition_pts` u krugu u kojem se pojavio (i uz šutnju); hitni popravak
uz ocjenu ≥ 4 diže `fault_fixed_condition_pts`. U repou oba 0 i `repair_urgent_condition_pts` ostaje 100 do dijela 4.

**„Kvar prostora" 🟨:** predložak vrste `kvar` **ili** s `EquipmentRepair` u nekom odgovoru, a bez `external_cause`. Danas točno 9:
P-KLIMA, P-DOTRAJALO (pritužbe s hitnim popravkom), P-BOJLER, P-BAZEN-PUMPA, P-LOBBY-KAVA, P-DIZALO, P-PRAONICA, P-BRAVE,
P-CURENJE-VODE. Nisu: P-DOBAVLJAC, P-TERMINAL, P-INSPEKCIJA, P-TOPLINSKI-VAL, P-OLUJA, P-NESTANAK-STRUJE (vanjski uzrok).
Prostor je `condition_of`, inače odjel primjerka, inače Smještaj (postojeće pravilo `situation_effect` `:52-55`). Sidra S1–S4 nisu kvar.

**Files** (f81a622):
- Create: `crates/sim-hotel-v2/tests/situation_fault.rs`, `crates/content/tests/v2_situation_fault.rs`, `crates/game/tests/situation_fault.rs`.
- Modify:
  - `crates/sim-hotel-v2/src/catalog.rs:33-50` (dva polja uz `repair_urgent_condition_pts` :46-48), `:392-397` (ista provjera
    raspona za oba), `:686` (testni literal: `0`).
  - `crates/content/src/v2_dept.rs:238-240` (DTO `Option<i32>`, `skip_serializing_if`), `:499-500` (natrag u DTO samo kad ≠ 0),
    `:550` (`unwrap_or(0)`).
  - `crates/sim-hotel-v2/src/situations/template.rs:59-80` (`SituationSpec.space_fault: bool`), `:31-45` (`TemplateSet::space_fault`
    uz `no_penalty` :34 i `condition_of` :41).
  - `crates/content/src/templates_core.rs:47-` (`template()`: `space_fault` iz vrste, odgovora i `external_cause`);
    `crates/content/src/templates_instance.rs:215-234` (`SituationTemplates::space_fault(code)`, isti obrazac šifre kao `no_penalty`).
  - `crates/sim-hotel-v2/src/situations/mod.rs:80-96` (`fault`), `crates/sim-hotel-v2/src/resolve/situations.rs:15-57` (koraci 3 i 5).
  - `crates/sim-hotel-v2/src/season.rs:356-375` (uz `no_penalty` i `condition_of` u n-torci primjerka i `space_fault`; graditelj smije
    n-torku pretvoriti u mali `struct`), `:407-428` (`fault` u `RoundSituation`).
  - `crates/game/src/drawn.rs:20-32` (`DrawnInstance.space_fault`), `:231` (punjenje), uz `:399-412` novi `Game::fault(h, r, code)`;
    `crates/game/src/resolve.rs:37-46` (`fault` u `push`).
  - Ostali literali `RoundSituation {`: `crates/content/src/templates_principles.rs:324`, `crates/game/tests/situations_delegate.rs:315`,
    `crates/game/tests/situations_replay.rs:134`, `crates/sim-hotel-v2/tests/situation_space.rs:26` (`fault: false`).
- Ne dirati: `content/hotel/v2/economy.json` (ključevi tek u dijelu 4), `sim-hotel` (v1), `fixtures_v2/*`, `prompts/`, `docs/`.

**Testovi čuvara** (RED: polja i funkcije ne postoje, E0560/E0599; katalog s ključevima 100/200 i `repair_urgent_condition_pts` 0
gradi se u testu iz repoa, obrazac `crates/content/tests/v2_principles.rs:287`):
1. `fault_lowers_space_condition_by_fault_pts` — `fault: true`, šutnja bez mehanika opreme → oprema prostora točno −100.
2. `urgent_repair_with_good_answer_nets_plus_one` — hitni popravak uz q 5 i uz q 4 → +100 (−100 + 200).
3. `urgent_repair_with_weak_answer_keeps_minus_one` — uz q 3 i q 1 → −100.
4. `fault_rule_is_off_without_keys` — ključevi 0: `SituationEffect` jednak današnjem za svih 9 predložaka i oba kruga (karakterizacija).
5. `repair_without_fault_gets_no_fix_bonus` — `fault: false`, hitni popravak q 5 → samo stari ključ.
6. `fault_effect_has_no_round_factor` — isti −100 / +200 tjedno i mjesečno.
7. `fault_keys_out_of_range_are_rejected` — −1 i 10 001 → `CatalogError::OutOfRange("v2.economy.fault_…")`.
8. `fault_outcomes_table` — ispisuje (`--nocapture`) Leonovu tablicu za P-BRAVE i P-KLIMA: šutnja / popravak uz 3 / uz 5 → −1 / −1 / +1.
9. (content) `space_fault_templates_are_the_nine` — popis gore, i šest vanjskih `false`; `no_template_is_fault_and_reward_only`;
   `repo_fault_keys_are_off_until_part_four` (repo: ključeva nema, `repair_urgent_condition_pts` 100; dio 4 ga mijenja);
   `missing_fault_keys_read_as_zero`.
10. (game) `game_applies_fault_from_drawn_template` — v2 igra nad snimkom s ključevima (`Snapshot::read_v2(&repo_dir())` + izmjena
    JSON-a, obrazac `v2_sealed.rs:428-438`, pa `common::v2_game_with`): prvi krug s kvarom prostora, šutnja → stanje prostora 100 bp
    niže nego u istoj igri bez ključeva (isto sjeme; uspoređuje se samo taj krug).
Ostaju zeleni bez izmjene: `v2_sealed` (dokaz neutralnosti), `answer_paths_in_game_match_balance` (`answer_paths_v2.rs:260`, isto
pravilo u igri i balansu), `v1_frozen`, `templates_principles` (šutnja i odgovor dobivaju isti −100).

**Koraci graditelja:** (1) `EconomyV2` + DTO + provjera; (2) `space_fault` u specifikaciji, pretvorbi i oba skupa; (3) `RoundSituation.fault`
i svi literali; (4) `situation_effect` koraci 3 i 5; (5) igra (`DrawnInstance`, `Game::fault`, `resolve.rs`) i sezona; (6) `cargo test
--workspace`; balans tjedno i mjesečno `cmp` s osnovicom (**bajt-isto**); (7) probni balans: kopija `content/hotel` u scratchpad s
`repair_urgent_condition_pts` 0, `fault_condition_pts` 100, `fault_fixed_condition_pts` 200, `--content <kopija>` tjedno i mjesečno, B3 blok u izvještaj.

**Rizik:** probni balans pokaže smjer: zapušten hotel (više kvarova) gubi više nego danas (🟨 17.10. „−1 može spustiti lošeg studenta").
Ako probni balans ruši pojas, to je ulaz za dio 4, ne razlog za izmjenu pravila u dijelu 1.
