Dio 1b cigle [L1-13](task-14-l1-13-kalibracija.md) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 1b · klik nosi ~75 % (Leon 18.10. t. 1) — `l113-1b-klik`

**Cilj:** klik na ponuđeni odgovor gdje se smije pisati (`click_capped`) i prepisan odgovor (`copied`) dobivaju `click_gain_bp` od svake
dobiti (korak 4); vrijednost u `economy.json` izmjerena tako da je težina klika ≤ 75 % težine vlastitog najboljeg za A, B i C,
tjedno i mjesečno (cilj 70–75 %), a lanac `klik − srednji > 0` i dalje vrijedi. Nova brana „B3 klik".

**Cilj mjerenja uz današnju težinu vlastitog** (izračunato iz `nakon-pravilo-c`; Δ = najbolji/klik − šutnja, stotinke):

| | tjedno A | B | C | mjesečno A | B | C |
|---|---|---|---|---|---|---|
| Δ klika danas | 540 | 557 | 618 | 259 | 321 | 258 |
| Δ klika za omjer 75 % | ≤ 383 | ≤ 393 | ≤ 462 | ≤ 194 | ≤ 231 | ≤ 196 |
| pad prema „klik − srednji" danas | 157 < 274 | 164 < 224 | 156 < 283 | 65 < 193 | 90 < 217 | 62 < 161 |

Pad od ~25–30 % Δ klika stane u razmak do srednjeg na svim profilima, pa lanac ostaje. `click_gain_bp` 🟨 [PROC] vjerojatno 3 000–6 000.

**Files:**
- Modify (jezgra): `crates/sim-hotel-v2/src/situations/mod.rs:80-96` (`click`), `crates/sim-hotel-v2/src/resolve/situations.rs` (korak 4,
  i za `fault_fixed_condition_pts` i grupe), `crates/sim-hotel-v2/src/catalog.rs:33-50`, `:392-397` (`click_gain_bp`), `:686`;
  `crates/content/src/v2_dept.rs:238-240`, `:499-500`, `:550` (`unwrap_or(10_000)`; u DTO samo kad ≠ 10 000).
- Modify (sezona): `crates/sim-hotel-v2/src/season.rs:310-312` — `Responder` vraća `Vec<Answered>`, novi
  `pub struct Answered { pub outcome: SituationOutcome, pub click: bool }` s `From<SituationOutcome>` (`click: false`);
  `:324-334` (`run_season_from`: `.into()`), `:403-428` (`click` u `RoundSituation`); izvoz u `crates/sim-hotel-v2/src/lib.rs:41-43`.
- Modify (igra): `crates/game/src/resolve.rs:47-70` (`click: self.click_capped(..)`), `:71-109` (`click: v.copied.is_some() &&
  self.click_capped(..)`), ostale grane `false`; `crates/game/src/own_words.rs:22` (`CAPPED_FACT`: „…ograničena na 4/5 i dobiva manji
  učinak…"); `packages/i18n/src/hr.ts:256` (`answers.clickCap`: „Klikom dobivaš najviše 4 od 5 i manji učinak odluke. Za punih 5 i
  puni učinak odgovori gostu svojim riječima.").
- Modify (balans): `crates/balance/src/v2_answers.rs:86-137` (`respond` → `Vec<Answered>`, `ClickBest ∧ writable` → `click: true`),
  `:149-150`, uz `:268-269` nova `pub const CLICK_WEIGHT_MAX_BP: i32 = 7_500;`, `:306-357` (brana), `:393-405` (naslov bez „samo
  ispis, nije brana" + redak omjera); `crates/balance/src/v2.rs:28-29` (izvoz).
- Modify (sadržaj): `content/hotel/v2/economy.json` (`"click_gain_bp": <izmjereno>`).
- Modify (testovi): `crates/balance/tests/answer_gates_v2.rs:42-51` (`clean`: `click_vs_silent = ruler / 5` → 16,7 % uz 25 %, omjer 66,7 %),
  `:309-332` (`weight_band_stays_on_own_words_best` prepisati: pojas i dalje na vlastitom, klik ima svoju branu).
- Ne dirati: put `Best`, roboti (`run_season_from` uvijek `click: false`), `Classic`, AI upute (`prompts/`), `docs/`.

**Brana (format poruke čuva čuvar):** za svaki profil s `best_vs_silent[2] > 0`: `share_bp(click) × 10 000 > CLICK_WEIGHT_MAX_BP ×
share_bp(best)` → `"B3 klik: {p} klik najbolji nosi {x} težine vlastitog najboljeg (cilj najviše 75,0 %)"`. Redak ispisa:
`"  klik prema vlastitom (težina klika / težina najboljeg; brana najviše 75,0 %):"` pa `"    A 71,2 %"`.

**Testovi čuvara** (RED: `Answered`, `click`, `click_gain_bp`, `CLICK_WEIGHT_MAX_BP` ne postoje):
1. (jezgra, `crates/sim-hotel-v2/tests/situation_click.rs` nova) `click_scales_positive_effects_only` (zadovoljstvo i oprema × faktor,
   trošak pun); `click_full_gain_is_identity` (10 000 → isti `SituationEffect`); `click_scales_group_room_nights` (najmanje 1);
   `click_scales_fault_fix_bonus` (s ključevima dijela 1: −100 + 200 × faktor); `click_never_scales_losses` (šutnja, negativni bodovi).
2. (igra, `crates/game/tests/answer_rule.rs`) `click_loses_part_of_gain` (klik najboljeg s likom: napredak niži nego nad snimkom s
   `click_gain_bp` 10 000, isto sjeme); `copied_answer_loses_gain_like_click`; `own_words_best_keeps_full_gain`;
   `classic_click_keeps_full_gain`; `robot_click_keeps_full_gain`.
3. (balans) `click_weight_gate_fails_above_three_quarters` (klik 80 % težine → točna poruka); `click_weight_gate_passes_at_three_quarters`
   (rub prolazi); `click_gate_needs_positive_best` (uz Δ najboljeg ≤ 0 pada samo „B3 odgovori"); `mirrors_match_the_game` ostaje;
   `answer_paths_in_game_match_balance` (`answer_paths_v2.rs:260`) mora ostati zelen (igra i balans istom zastavicom).
4. `crates/app/tests/answer_rule.rs` / vitest `OfferedAnswers.test.tsx`: regex `/Klikom dobivaš najviše 4 od 5/` ostaje; novi dio `/manji učinak/`.
Ostaje zelen bez izmjene: `v2_sealed` (snimak bez ključa = 10 000).

**Koraci graditelja:** (1) jezgra i sadržaj s ključem 10 000 (balans bajt-isti — kontrolna točka); (2) igra i balans; (3) mjerenje
`click_gain_bp` u koracima od 1 000 (tjedno i mjesečno 1000 sjemena; tablica u izvještaj), izbor najvećeg koji drži omjer ≤ 75 % na
svih 6 stupaca; (4) branu i ispis; (5) sve provjere. **Diff balansa** smije dirati samo stupac „klik najbolji", korake 0 i 1, težinu
klika i novi redak omjera; sve ostalo (uključivo težinu vlastitog) bajt-isto.

**Rizik:** nakon dijela 4 težina vlastitog pada na ~20 %, a omjer težina tada se približava omjeru Δ (niži od današnjeg), pa isti
faktor daje omjer ispod 70 %; dio 4 ga diže natrag u 70–75 %. Ako ni `click_gain_bp` 0 ne spusti omjer ispod 75 % (dobit nije glavni
dio Δ), čuvar to javlja voditelju prije koda — tada se umanjuje i trošak klika, što je nova odluka za Leona.
