### Task 21: Web — Plan po odjelima

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 15, Task 18, **Task 19** (spojen: `v2Api()`, poruka
`problem.department_plan_invalid`) · **Paralelno s:** Task 20 (val 10) · **Spaja se nakon:** Task 20 · **Model:** sonnet

**Cilj:** u planu kruga (v2) dio „Odjeli": otvoren/zatvoren (ako se smije zatvoriti), sezonci, razina cijene, razina
održavanja; jedan zahtjev „Spremi".

**Files:**
- Create: `apps/web/src/screens/plan/DepartmentPlan.tsx`, `apps/web/src/screens/plan/DepartmentPlan.test.tsx`
- Modify: `apps/web/src/screens/PlanScreen.tsx`, `apps/web/src/test/fixtures.ts` (samo `GET /v1/me/hotel` u
  `v2Api()`: plan kruga t01 s `departments` i granice s `departments` i `seasonal_step_total: 5`),
  `packages/i18n/src/hr.ts` (tekstovi dijela „Odjeli")
- Ne dirati: poslužitelj, ostali ekrani, `router.tsx`, `App.tsx`, `ui/Layout.tsx`, `api/errors.ts`, `test/fakeApi.ts`
  (sve to dira Task 20 usporedno ili Task 19 prije).

**Interfaces:**
- Consumes: `PlanBody.departments`, `PlanLimitsBody.{departments, seasonal_step_total}`, postojeći `useSavePlan`.
- Produces: komponenta `DepartmentPlan({ plan, limits, onChange })` (kontrolirana; ne šalje sama).

Pravila na ekranu: ukupna promjena sezonaca prikazana kao „Ovaj krug se može promijeniti još za 3 sezonca" (iz
`seasonal_step_total` i već napravljenih promjena — računa iz plana i ograničenja koja je dao poslužitelj; ništa novo
se ne izvodi); stepper odjela onemogućen na granici; zatvaranje odjela postavlja sezonce na 0 i onemogućuje stepper;
razina cijene 90–120 % u koracima od 5 („Cijene u odjelu 110 %"); održavanje „odgođeno / redovno / pojačano". Greška 422
`department_plan_invalid` prikazuje se kao rečenica kroz `problemMessage(error)` (ključ
`problem.department_plan_invalid` dodao je Task 19).

- [ ] **Step 1: Testovi (padaju)** — `DepartmentPlan.test.tsx`:

```tsx
describe("Plan po odjelima", () => {
  it("v2: dio Odjeli s retkom za svaki odjel koji se planira", async () => {
    renderApp(v2Api(), "/plan");
    expect(await screen.findByRole("heading", { level: 2, name: "Odjeli" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Restoran" })).toBeTruthy();
    expect(screen.queryByRole("group", { name: "Uprava" })).toBeNull();
  });
  it("zatvaranje bara stavlja sezonce na 0 i šalje jedan plan", async () => {
    const api = v2Api();
    renderApp(api, "/plan");
    const bar = await screen.findByRole("group", { name: "Lobby bar" });
    fireEvent.click(within(bar).getByRole("checkbox", { name: "Otvoren" }));
    expect((within(bar).getByRole("button", { name: "Više sezonaca" }) as HTMLButtonElement).disabled).toBe(true);
    const save = screen.getByRole("button", { name: "Spremi" });
    fireEvent.click(save);
    fireEvent.click(save);
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const body = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as { departments: Record<string, { open: boolean; seasonal_target: number }> };
    expect(body.departments.lobby_bar).toMatchObject({ open: false, seasonal_target: 0 });
  });
  it("restoran se ne može zatvoriti", async () => {
    renderApp(v2Api(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    expect(within(rest).queryByRole("checkbox", { name: "Otvoren" })).toBeNull();
  });
  it("korak sezonaca potrošen: plus onemogućen i rečenica", async () => {
    renderApp(v2Api(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    const plus = within(rest).getByRole("button", { name: "Više sezonaca" });
    for (let i = 0; i < 5; i++) fireEvent.click(plus);
    expect(screen.getByText("Ovaj krug se sezonci više ne mogu mijenjati.")).toBeTruthy();
    expect((plus as HTMLButtonElement).disabled).toBe(true);
  });
  it("v1 igra nema dio Odjeli", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Odjeli" })).toBeNull();
  });
});
```

(`HOTEL_ID` = `"h1"` je konstanta iz `test/fixtures.ts` koju dodaje Task 18 — uvezi je uz `v2Api`, `standardApi`;
`seasonal_step_total` u fiksturi je 5, a `lobby_bar` u fiksturi ima 3 sezonca, pa zatvaranje troši 3 od 5.)

- [ ] **Step 2–4:** testovi padaju → implementacija → četiri web provjere EXIT=0.
- [ ] **Step 5: Commit** „Sloj 1/21: web — plan po odjelima".

---

