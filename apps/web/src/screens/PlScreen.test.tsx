import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import { HOTEL_ID, plBody, v2Api, v2PlApi } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("Račun dobiti", () => {
  it("sekcije redom USALI-ja i razlika riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ })).toBeTruthy();
    const sections = document.querySelector('[data-layout="pl-sections"]') as HTMLElement;
    const titles = within(sections)
      .getAllByRole("heading", { level: 2 })
      .map((h) => h.textContent);
    expect(titles).toEqual([
      "Prihodi odjela",
      "Troškovi odjela",
      "Rezultat odjela",
      "Zajednički troškovi hotela",
      "Bruto operativna dobit (GOP)",
      "Fiksni troškovi zgrade",
      "EBITDA",
    ]);
    expect(screen.getAllByText("+1.200 € bolje od budžeta").length).toBeGreaterThan(0);
    expect(screen.getAllByText("−800 € lošije od budžeta").length).toBeGreaterThan(0);
  });
  it("računalo: tablica sa četiri stupca", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    const table = await screen.findByRole("table", { name: "Račun dobiti po USALI-ju" });
    expect(
      within(table)
        .getAllByRole("columnheader")
        .map((c) => c.textContent),
    ).toEqual(["Stavka", "Ostvareno", "Budžet", "Razlika"]);
    expect(table.closest("section")?.className).toMatch(/hidden .*lg:block/);
  });
  it("naslov tjednog kruga: broj tjedna i datumi su nelomljivi", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    const h1 = await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ });
    const text = h1.textContent ?? "";
    expect(text).toContain("Tjedan 1 (");
    expect(text).toMatch(/\([^ )]+\)/); // u zagradi nema običnih razmaka
    expect(text).toContain("–⁠ ");
  });
  it("sezona do sada šalje jedan novi upit sa scope=season", async () => {
    const api = v2PlApi();
    renderApp(api, "/racun-dobiti");
    const btn = await screen.findByRole("button", { name: "Sezona do sada" });
    const seasonCalls = () =>
      api.calls.filter(
        (c) =>
          c.method === "GET" &&
          c.path === `/v1/hotels/${HOTEL_ID}/pl` &&
          new URLSearchParams(c.search).get("scope") === "season",
      );
    expect(seasonCalls()).toHaveLength(0); // prvi upit ekrana je za krug, ne za sezonu
    fireEvent.click(btn);
    await waitFor(() => expect(seasonCalls()).toHaveLength(1));
    expect(await screen.findByText(/Zbroj od početka sezone/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sezona do sada" }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("heading", { level: 1 }).textContent).toMatch(/sezona do tjedna 1/);
  });
  it("prije prvog obračuna: rečenica, ne nule", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "no_resolved_round")),
      "/racun-dobiti",
    );
    // Tjedna igra: rečenica govori o tjednu, bez riječi „mjesec“ ni općenitog „kruga“.
    expect(await screen.findByText("Račun dobiti postoji tek nakon prvog obračunatog tjedna.")).toBeTruthy();
    expect(screen.queryByText("0 €")).toBeNull();
  });
  it("GOP i EBITDA imaju objašnjenje riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByText(/dobit prije kamata, poreza i amortizacije/)).toBeTruthy();
  });
  it("krug iz adrese ide u upit kao round, bez scope=season", async () => {
    const api = v2PlApi();
    renderApp(api, "/racun-dobiti/t01");
    await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ });
    const pl = api.calls.filter((c) => c.path === `/v1/hotels/${HOTEL_ID}/pl`);
    expect(pl).toHaveLength(1);
    const q = new URLSearchParams(pl[0]?.search);
    expect(q.get("round")).toBe("t01");
    expect(q.get("scope")).toBe("round");
  });
  it("razlika se računa iz prikazanih eura: tri broja se uvijek zbrajaju", async () => {
    const body = plBody();
    const line = body.sections[0]?.lines[0];
    if (!line) throw new Error("fixture");
    Object.assign(line, {
      actual_cents: 2_374_549,
      budget_cents: 2_663_501,
      variance_cents: -288_952,
      favorable: false,
    });
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", body), "/racun-dobiti");
    expect((await screen.findAllByText("−2.890 € lošije od budžeta")).length).toBeGreaterThan(0);
  });
  it("razlika ispod 1 € je „jednako budžetu“, bez strelice i „0 €“", async () => {
    const body = plBody();
    const line = body.sections[0]?.lines[0];
    if (!line) throw new Error("fixture");
    Object.assign(line, {
      actual_cents: 5_000_040,
      budget_cents: 5_000_000,
      variance_cents: 40,
      favorable: true,
    });
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", body), "/racun-dobiti");
    await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ });
    expect(screen.queryByText(/^0 € /)).toBeNull();
    expect(screen.getAllByText("jednako budžetu").length).toBeGreaterThan(0);
  });
  it("tablica: iznosi se ne lome, naziv stavke se smije lomiti", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    const table = await screen.findByRole("table", { name: "Račun dobiti po USALI-ju" });
    const row = within(table).getByRole("row", { name: /Smještaj/ });
    const [name, ...cells] = Array.from(row.children);
    expect(name?.className).not.toContain("whitespace-nowrap");
    for (const c of cells) {
      expect(c.className).toContain("whitespace-nowrap");
      expect(c.className).toContain("align-top");
    }
  });
  it("nepostojeći tjedan: rečenica o tjednu, ne o mjesecu", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "round_not_open")),
      "/racun-dobiti/t09",
    );
    expect(await screen.findByText("Tjedan nije otvoren — sad se ne može mijenjati.")).toBeTruthy();
  });
  it("bez podataka nema uvodnih rečenica", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "no_resolved_round")),
      "/racun-dobiti",
    );
    await screen.findByText("Račun dobiti postoji tek nakon prvog obračunatog tjedna.");
    expect(screen.queryByText(/Ovako hotelski računovođe/)).toBeNull();
  });
  it("pokazatelji imaju naslov razine 2 i postotke s jednom decimalom", async () => {
    const body = plBody();
    body.gop_margin_bp = { actual: 1300, budget: 1590 };
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", body), "/racun-dobiti");
    expect(await screen.findByRole("heading", { level: 2, name: "Pokazatelji računa" })).toBeTruthy();
    expect(screen.getByText("13,0 %")).toBeTruthy();
    expect(screen.getByText("15,9 %")).toBeTruthy();
  });
});
