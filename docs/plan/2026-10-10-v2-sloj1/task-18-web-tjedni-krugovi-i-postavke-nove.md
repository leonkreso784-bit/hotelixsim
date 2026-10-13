### Task 18: Web — tjedni krugovi i postavke nove igre

**Veličina:** M, 4–5 h · **Ovisi o:** Task 7 · **Paralelno s:** Task 9–17 · **Model:** sonnet

**Cilj:** v1 igra izgleda točno kao danas; v2 igra radi po tjednima (oznaka kruga, „Zatvori tjedan", ekran
završenog kruga, ljestvica, tijek, pokazatelji po krugu); na početnom ekranu sklopivi dio „Za profesora" stvara v2 igru.

**Files:**
- Modify: `apps/web/src/{api/types,api/queries,format,game,router,App}.ts(x)`, `apps/web/src/ui/Layout.tsx`,
  `apps/web/src/screens/{HotelScreen,CloseMonth,MonthScreen,PlanScreen,FeedScreen,IndicatorScreen,KpiScreen,StartScreen}.tsx`,
  `apps/web/src/screens/month/*.tsx`, `apps/web/src/test/fixtures.ts`, `packages/i18n/src/hr.ts`, odgovarajući `*.test.ts(x)`
- Ne dirati: poslužitelj, `openapi.json`, `packages/api-client` (samo čita tipove iz Taska 7), `src/avatar/**`.

**Interfaces:**
- Consumes: `GameBody.{model, round_kind, rounds}`, `RoundBody.round`, `MonthScoreBody.round`, `round` u tijelima, `NewGameBody`.
- Produces:

```ts
// api/types.ts
export type RoundInfoBody = S["RoundInfoBody"]; export type ModelCode = S["ModelCode"];
export type RoundKindCode = S["RoundKindCode"]; export type NewGameBody = S["NewGameBody"];
// format.ts
/** Mjesec: „lipanj"; tjedan: „29. 6. – 5. 7.". */
export function roundDates(r: RoundInfoBody): string;
/** Mjesec: „Lipanj"; tjedan: „Tjedan 5 (29. 6. – 5. 7.)". */
export function roundTitle(r: RoundInfoBody): string;
// game.ts — ključ je kod kruga (u v1 jednak kodu mjeseca)
export function roundInfo(game: GameBody, code: string): RoundInfoBody | null;
export function roundPosition(game: GameBody): { round: RoundInfoBody; index: number; total: number } | null;
export function isV2(game: GameBody): boolean;
// indicesAt / indicesBefore / progressAt / leaderboard / nextRound primaju kod kruga i čitaju `MonthScoreBody.round`
// router.tsx
//   { name: "round"; code: string }  ← "/krug/:code" i stari "/mjesec/:code" (kod: jun|jul|aug|sep|t01…t17)
// App.tsx: StartScreen šalje NewGameBody samo kad je odabran novi model
```

Tekstovi (`hr.ts`, primjeri ključeva): `round.week` „Tjedan {n}", `chip.week` „TJEDAN {n} · {i}/{total}",
`close.month` „Zatvori mjesec", `close.week` „Zatvori tjedan", `round.done.week` „Tjedan {n} završen", `start.pro`
„Za profesora: postavke nove igre", `start.model.v1` „Demo (dva mjeseca, bez odjela)", `start.model.v2` „Novi model:
odjeli i račun dobiti", `start.kind.week` „Po tjednima (preporučeno)", `start.kind.month` „Po mjesecima",
`start.kind.help` „Tjedni krug ima manje odluka, a mjesečna lekcija i mentor dolaze na kraju svakog mjeseca."

- [ ] **Step 1: Testovi (padaju)** — u postojećim datotekama testova:

```ts
// format.test.ts
it("oznaka tjedna i mjeseca", () => {
  const t05 = { code: "t05", kind: "week", index: 4, days: 7, first_day: 29, first_month: "jun", last_day: 5, last_month: "jul", month_ends: true } as const;
  expect(roundDates(t05)).toBe("29. 6. – 5. 7.");
  expect(roundTitle(t05)).toBe("Tjedan 5 (29. 6. – 5. 7.)");
  const jun = { code: "jun", kind: "month", index: 0, days: 30, first_day: 1, first_month: "jun", last_day: 30, last_month: "jun", month_ends: true } as const;
  expect(roundTitle(jun)).toBe("Lipanj");
});
// router.test.tsx
it("krug po kodu i stari put mjeseca", () => {
  expect(matchRoute("/krug/t05")).toEqual({ name: "round", code: "t05" });
  expect(matchRoute("/mjesec/jun")).toEqual({ name: "round", code: "jun" });
  expect(matchRoute("/krug/t18")).toEqual({ name: "notFound" });
});
// HotelScreen.test.tsx
it("tjedna igra: čip TJEDAN 1 · 1/17 i gumb Zatvori tjedan", async () => {
  renderApp(standardApi().on("GET", "/v1/games/:game", weeklyGame()), "/");
  expect(await screen.findByText("TJEDAN 1 · 1/17")).toBeTruthy();
  expect(await screen.findByRole("button", { name: "Zatvori tjedan" })).toBeTruthy();
});
it("v1 igra izgleda kao prije: LIPANJ · 1/2 i Zatvori mjesec", async () => {
  renderApp(standardApi(), "/");
  expect(await screen.findByText("LIPANJ · 1/2")).toBeTruthy();
  expect(await screen.findByRole("button", { name: "Zatvori mjesec" })).toBeTruthy();
});
// MonthScreen.test.tsx
it("završen tjedan na /krug/t01", async () => {
  renderApp(resolvedWeekApi(), "/krug/t01");
  expect(await screen.findByRole("heading", { level: 1, name: "Tjedan 1 završen (1. 6. – 7. 6.)" })).toBeTruthy();
});
// StartScreen (App.test.tsx)
it("Započni igru bez postavki šalje prazno tijelo", async () => {
  const api = startApi();
  renderApp(api, "/");
  fireEvent.click(await screen.findByRole("button", { name: "Započni igru" }));
  await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  expect(api.posts("/v1/demo")[0]?.body).toBeUndefined();
});
it("novi model po tjednima šalje jednom tijelo s modelom", async () => {
  const api = startApi();
  renderApp(api, "/");
  fireEvent.click(await screen.findByText("Za profesora: postavke nove igre"));
  fireEvent.click(screen.getByRole("radio", { name: "Novi model: odjeli i račun dobiti" }));
  expect((screen.getByRole("radio", { name: "Po tjednima (preporučeno)" }) as HTMLInputElement).checked).toBe(true);
  const start = screen.getByRole("button", { name: "Započni igru" });
  fireEvent.click(start);
  fireEvent.click(start);
  await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  expect(api.posts("/v1/demo")[0]?.body).toEqual({ model: "v2", round_kind: "week" });
});
```

`fixtures.ts` dobiva `weeklyGame()` (17 krugova, `model: "v2"`, `round_kind: "week"`, `current.round` = t01),
`resolvedWeekApi()` i dopunu svih postojećih tvornica poljima `round`/`rounds`/`model` (v1 vrijednosti).

- [ ] **Step 2:** `corepack pnpm --filter @hotelixsim/web test` → FAIL.
- [ ] **Step 3: Implementiraj.** Datumi u `roundDates` iz polja tijela (ekran ne računa kalendar). Čip na `Layout`
  koristi `roundPosition`. Sve što je bilo po `month` čita `round` (za v1 je isto).
- [ ] **Step 4:** `corepack pnpm lint; corepack pnpm typecheck; corepack pnpm test; corepack pnpm --filter @hotelixsim/web build`
  (svaka zasebno, EXIT=0).
- [ ] **Step 5: Commit** „Sloj 1/18: web — tjedni krugovi i postavke nove igre za profesora".

---

