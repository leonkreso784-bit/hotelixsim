### Task 19: Web — Odjeli i detalj odjela

**Veličina:** M, 3–4 h · **Ovisi o:** Task 15, Task 18 · **Paralelno s:** ništa iz weba (val 9; Taskovi 20 i 21
počinju tek kad je ovaj spojen, jer troše njegov `v2Api()`, stavku izbornika i `problem.*` poruke) · **Model:** sonnet

**Cilj (Leon: „stisne wellness → stanje prostora, koliko je zaradio, koliko su gosti bili zadovoljni"):** odjeljak
„Odjeli" s popisom odjela koje hotel ima i ekranom odjela.

**Files:**
- Create: `apps/web/src/screens/DepartmentsScreen.tsx`, `apps/web/src/screens/departments/{DepartmentCard,DepartmentDetail,ServiceCard}.tsx`,
  `apps/web/src/screens/DepartmentsScreen.test.tsx`
- Modify: `apps/web/src/{router,App}.tsx`, `apps/web/src/ui/Layout.tsx` (stavka „Odjeli", ikona `Building2`, samo v2),
  `apps/web/src/api/{types,queries}.ts`, `apps/web/src/test/fixtures.ts`, `packages/i18n/src/hr.ts`,
  `apps/web/src/api/errors.ts` (samo ako treba: `problemMessage` već čita poruku iz ključa `problem.<code>` u
  `hr.ts`, `errors.ts:32-39`, pa je dovoljno dodati ključeve)
- Ne dirati: poslužitelj, ekrani drugih taskova.

**Interfaces:**
- Consumes: `GET /v1/hotels/{id}/departments`, `GET /v1/hotels/{id}/departments/{code}`, `isV2`, `roundTitle`.
- Produces: rute `{ name: "departments" }` (`/odjeli`) i `{ name: "department"; code: string }` (`/odjeli/:code`);
  kuke `useDepartments(hotelId)`, `useDepartment(hotelId, code)`.

Izgled: mobitel — popis kartica (ime, „Stanje prostora 72/100" s trakom, „Zadovoljstvo gostiju 74/100", „Rezultat
zadnjeg tjedna +7.896 €", „Ljudi: 8 stalnih, 10 sezonskih", oznaka „Zatvoren"), dolje „Službe bez prihoda" (Uprava,
Prodaja i marketing, Održavanje, Energija i voda: „Trošak zadnjeg kruga 59.050 €"); klik → puni ekran detalja s
poveznicom „← Odjeli". Od 1024 px: popis lijevo, detalj desno (`lg:grid lg:grid-cols-[22rem_1fr]`). Detalj ima
četiri kartice: **Prostor** (stanje prije → poslije, „trošenje −3", „preopterećenje −1", „održavanje +3", trošak
održavanja), **Gosti** (korištenja, „udio gostiju koji dolaze 45 %", zadovoljstvo prije → poslije, „vrijednost za novac
70/100", razina cijene), **Novac** (prihod, plaće, namirnice i piće / materijal, ostali izravni trošak, rezultat,
marža), **Ljudi** (stalni, sezonski, kapacitet, „opterećenje 1,9 gosta po zaposlenom na dan, norma 1,8"); ispod tablica
po krugovima (laptop) ili popis (mobitel). Bodovi 0–10 000 se prikazuju kao cijeli broj 0–100. v1 igra: ruta pokazuje
rečenicu iz 409 `model_v1`, a izbornik nema „Odjeli".

- [ ] **Step 1: Testovi (padaju)** — `DepartmentsScreen.test.tsx`:

```tsx
describe("Odjeli", () => {
  it("popis: samo odjeli hotela, brojke s imenom i jedinicom", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    const spa = (await screen.findByRole("link", { name: /Mali wellness/ })).closest("article") as HTMLElement;
    expect(within(spa).getByText("Stanje prostora")).toBeTruthy();
    expect(within(spa).getByText("65/100")).toBeTruthy();
    expect(within(spa).getByText("Zadovoljstvo gostiju")).toBeTruthy();
    expect(within(spa).getByText("Ljudi: 2 stalna, 1 sezonski")).toBeTruthy();
    expect(screen.queryByText("Rooftop bar")).toBeNull();
    expect(screen.getByRole("heading", { name: "Službe bez prihoda" })).toBeTruthy();
  });
  it("detalj odjela izravno na putanji: prostor, gosti, novac, ljudi", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    for (const h of ["Prostor", "Gosti", "Novac", "Ljudi"]) expect(screen.getByRole("heading", { level: 2, name: h })).toBeTruthy();
    expect(screen.getByText("67.200 €")).toBeTruthy();
    expect(screen.getByText("trošenje −3")).toBeTruthy();
    expect(screen.getByText("udio gostiju koji dolaze 45 %")).toBeTruthy();
  });
  it("računalo: popis i detalj u dva stupca", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const grid = (await screen.findByRole("heading", { level: 1, name: "Restoran" })).closest("[data-layout='list-detail']");
    expect(grid?.className).toMatch(/lg:grid/);
  });
  it("v1 igra: rečenica umjesto odjela i nema stavke izbornika", async () => {
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/departments", () => problem(409, "model_v1")), "/odjeli");
    expect(await screen.findByText("Ova igra je na starom modelu i nema ovaj prikaz.")).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });
  it("nepoznat odjel: rečenica, ne prazno", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => problem(404, "unknown_department")), "/odjeli/casino");
    expect(await screen.findByText("Taj odjel ne postoji u ovom hotelu.")).toBeTruthy();
  });
});
```

`fixtures.ts`: kostur `v2Api()` iz Taska 18 dobiva rute `GET /v1/hotels/:hotel/departments` → `departmentsBody()`
(brojke iz Taska 9: restoran 6 720 000 / 3 780 000 / 2 150 400, stanje 7 000 → 6 900, zadovoljstvo 7 000 → 6 949,
udio 4 500) i `GET /v1/hotels/:hotel/departments/:code` → `departmentDetailBody()`. U `hr.ts` dodaj **sve četiri**
poruke problema sloja 1 odjednom (da ih Taskovi 20 i 21 ne dodaju usporedno): `problem.model_v1` „Ova igra je na
starom modelu i nema ovaj prikaz.", `problem.unknown_department` „Taj odjel ne postoji u ovom hotelu.",
`problem.no_resolved_round` „Račun dobiti postoji tek nakon prvog obračunatog kruga.", `problem.department_plan_invalid`
„Postavke odjela nisu dopuštene. Provjeri sezonce, razinu cijene i koje se odjele smije zatvoriti." —
`problemMessage(error)` ih tada vraća sam.

- [ ] **Step 2:** `corepack pnpm --filter @hotelixsim/web test` → FAIL.
- [ ] **Step 3: Implementiraj** (tokeni, `t()`, `role="status"` za učitavanje, `Link` za kartice, širina trake jedina
  inline vrijednost).
- [ ] **Step 4:** četiri web provjere (EXIT=0) i pregled na 360 px (bez vodoravnog pomicanja) u testu ili pregledniku.
- [ ] **Step 5: Commit** „Sloj 1/19: web — Odjeli i ekran odjela".

---

