### Task 20: Web — Račun dobiti i budžet

**Veličina:** M, 3–4 h · **Ovisi o:** Task 15, Task 18 · **Paralelno s:** Task 19, 21 (spaja se nakon 19) · **Model:** sonnet

**Cilj:** ekran „Račun dobiti" po USALI-ju s budžetom i razlikom, za krug ili sezonu do sada.

**Files:**
- Create: `apps/web/src/screens/PlScreen.tsx`, `apps/web/src/screens/pl/{PlTable,PlCards}.tsx`, `apps/web/src/screens/PlScreen.test.tsx`
- Modify: `apps/web/src/{router,App}.tsx`, `apps/web/src/ui/Layout.tsx` (stavka „Račun dobiti", ikona `Receipt`, samo v2),
  `apps/web/src/api/{types,queries}.ts`, `apps/web/src/test/fixtures.ts`, `packages/i18n/src/hr.ts`
- Ne dirati: ekrani Taskova 19 i 21, poslužitelj.

**Interfaces:**
- Consumes: `GET /v1/hotels/{id}/pl?round=&scope=`.
- Produces: rute `{ name: "pl"; code: string | null }` (`/racun-dobiti` i `/racun-dobiti/:code`); kuka
  `usePl(hotelId, code, scope)`; prekidač „Ovaj krug" / „Sezona do sada" (stanje ekrana, novi upit), izbornik krugova
  iz `PlBody.rounds` (oznaka `roundTitle`).

Izgled: zaglavlje „Račun dobiti — Tjedan 2 (8. 6. – 14. 6.)"; objašnjenje jednom rečenicom: „Ovako hotelski računovođe
slažu rezultat (USALI): prvo odjeli, zatim zajednički troškovi hotela, pa GOP, pa fiksni troškovi zgrade."; „Budžet =
što bi tvoj hotel ostvario uz zadane odluke i bez situacija." Laptop (≥ 1024 px): tablica „Stavka | Ostvareno | Budžet |
Razlika" po sekcijama (naslovi sekcija iz `t()`: „Prihodi odjela", „Troškovi odjela", „Rezultat odjela", „Zajednički
troškovi hotela", „Bruto operativna dobit (GOP)", „Fiksni troškovi zgrade", „EBITDA"); mobitel: kartica po sekciji,
redak = ime, ostvareno, ispod sitno „budžet 12.300 €" i razlika riječima „+1.200 € bolje od budžeta" / „−800 € lošije
od budžeta" (boja tokenom uspjeha ili upozorenja uz riječi, ne samo boja). Ispod: GOP marža, GOPPAR, EBITDA marža,
ostvareno i budžet.

- [ ] **Step 1: Testovi (padaju)** — `PlScreen.test.tsx`:

```tsx
describe("Račun dobiti", () => {
  it("sekcije redom USALI-ja i razlika riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ })).toBeTruthy();
    const titles = screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent);
    expect(titles).toEqual(["Prihodi odjela", "Troškovi odjela", "Rezultat odjela", "Zajednički troškovi hotela",
      "Bruto operativna dobit (GOP)", "Fiksni troškovi zgrade", "EBITDA"]);
    expect(screen.getAllByText("+1.200 € bolje od budžeta").length).toBeGreaterThan(0);
    expect(screen.getAllByText("−800 € lošije od budžeta").length).toBeGreaterThan(0);
  });
  it("računalo: tablica sa četiri stupca", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    const table = await screen.findByRole("table", { name: "Račun dobiti po USALI-ju" });
    expect(within(table).getAllByRole("columnheader").map((c) => c.textContent)).toEqual(["Stavka", "Ostvareno", "Budžet", "Razlika"]);
    expect(table.closest("section")?.className).toMatch(/hidden .*lg:block/);
  });
  it("sezona do sada šalje jedan novi upit sa scope=season", async () => {
    const api = v2PlApi();
    renderApp(api, "/racun-dobiti");
    const btn = await screen.findByRole("button", { name: "Sezona do sada" });
    fireEvent.click(btn);
    await waitFor(() => expect(api.calls.some((c) => c.path.endsWith("/pl") && c.method === "GET")).toBe(true));
    expect(await screen.findByText(/sezona do sada/i)).toBeTruthy();
  });
  it("prije prvog obračuna: rečenica, ne nule", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "no_resolved_round")), "/racun-dobiti");
    expect(await screen.findByText("Račun dobiti postoji tek nakon prvog obračunatog kruga.")).toBeTruthy();
    expect(screen.queryByText("0 €")).toBeNull();
  });
  it("GOP i EBITDA imaju objašnjenje riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByText(/dobit prije kamata, poreza i amortizacije/)).toBeTruthy();
  });
});
```

(`FakeApi` uspoređuje samo putanju bez upita; parametar `scope` provjeri kroz `new URL(req.url).searchParams` u
rukovatelju rute u `v2PlApi()` i vrati drugo tijelo za `season`.)

- [ ] **Step 2–4:** testovi padaju → implementacija → četiri web provjere EXIT=0.
- [ ] **Step 5: Commit** „Sloj 1/20: web — Račun dobiti po USALI-ju s budžetom".

---

