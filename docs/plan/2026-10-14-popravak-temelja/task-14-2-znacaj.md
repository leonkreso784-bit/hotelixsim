Dio 2 cigle [L1-13](task-14-l1-13-kalibracija.md) (puna kartica 18.10., retci provjereni na f81a622). Leonove odluke, CORE_VERSION,
zajednička sučelja i redoslijed računa u `situation_effect` (koraci 1–5), cijeli popis provjera, rizici 🟨 i Leonov dio su u pregledu.

### Task L1-13 dio 2 · tri razine značaja (B2, Leon 18.10. t. 2) — `l113-2-znacaj`

**Cilj:** razina iz `weight`: 4 **ključna**, 3 **važna**, 1–2 **svakodnevna** (danas 9 / 17 / 29 predložaka). Bodovi situacije × faktor
razine (`economy.significance_bp`, u repou 10 000 sva tri; dio 4 kreće od 15 000 / 10 000 / 6 000 [PROC]). Troškovi, grupe i kvar
se ne množe. Student vidi razinu kao **tekst** uz naslov (oznaka, ne samo boja; [UI_PRAVILA](../../product/UI_PRAVILA.md)).
Sidra S1–S4 nemaju `weight`: računaju se kao važna, oznake nema 🟨. `major` (`templates.rs:83-85`) ostaje uvjet izvlačenja.

**Files:**
- Modify (jezgra): `crates/sim-hotel-v2/src/situations/mod.rs` (`Significance`, polje), `crates/sim-hotel-v2/src/catalog.rs:33-50`,
  `:392-397`, `:686`; `crates/sim-hotel-v2/src/resolve/situations.rs:37-51`, `:122-128` (korak 2); `crates/sim-hotel-v2/src/season.rs:407-428`
  (`item.drawn.map(|d| Significance::from_weight(d.weight))`, sidra `Important`).
- Modify (sadržaj): `crates/content/src/v2_dept.rs:230-241`, `:490-501`, `:540-552` (`SignificanceBpDto`, `deny_unknown_fields`).
- Modify (igra): `crates/game/src/drawn.rs:399-428` (uz `space` novi `Game::significance(h, r, code)` iz `i.drawn.weight`),
  `crates/game/src/resolve.rs:37-46`.
- Modify (pogled): `crates/app/src/views.rs:48-75` (uz `weight` :63 `significance: Option<Significance>`; `None` za sidra i v1), `:363`;
  `crates/api/src/schema_situations.rs:13-40` (uz `weight` :36 `significance: Option<SignificanceBody>`, `rename_all = "snake_case"`,
  vrijednosti `kljucna` / `vazna` / `svakodnevna`), `:279`; `openapi.json` i `packages/api-client/src` (`corepack pnpm gen:api`).
- Modify (web): `apps/web/src/screens/SituationsScreen.tsx:74-77` (`<Badge>` razine prije statusa), `apps/web/src/screens/SituationScreen.tsx:86-96`
  (`<Badge>` uz naslov); `packages/i18n/src/hr.ts:227` (`situation.significance.kljucna` „Ključna", `.vazna` „Važna", `.svakodnevna`
  „Svakodnevna"); `apps/web/src/test/fixtures.ts` (novo polje).
- Ne dirati: `content/hotel/v2/economy.json`, `situation_templates.json` (razina se izvodi, nema novog polja), mentor (🟨 kasnije).

**Testovi čuvara** (RED: `Significance`, `significance_bp`, polja pogleda ne postoje):
1. `significance_from_weight` (4 → Key, 3 → Important, 2 i 1 → Everyday) i `code()`.
2. `significance_scales_points_not_costs` (Key 15 000: bodovi × 1,5 uz jedno zaokruživanje, trošak isti); `neutral_significance_is_identity`.
3. (content) `levels_cover_all_templates_9_17_29`; `missing_significance_reads_neutral`; `significance_out_of_range_is_rejected`.
4. (app) `situations_view_shows_significance` (primjerak `Some`, sidro `None`, v1 `None`); (api) polje u `openapi.json`.
5. (vitest, `SituationsScreen.test.tsx` i `SituationScreen` test) oznaka „Ključna" vidi se kao tekst; sidro bez oznake.
Ostaju zeleni: `v2_sealed`, balans bajt-isti (faktori 10 000).

**Koraci graditelja:** jezgra → sadržaj → igra i sezona → pogled, API (`gen:api`), web → probni balans s 15 000 / 10 000 / 6 000
(`--content` kopija; težina i B3 u izvještaj) → sve provjere.

**Rizik:** `weight` sada nosi tri značenja (poredak, prednost u izvlačenju, faktor bodova). Ključnih ima malo, ali se češće izvlače, pa
prosječni faktor ne treba biti 1; mjeri dio 4. Ako Leon želi razdvojiti, dolazi polje `significance` u predlošku (~2 h, 55 predložaka).
