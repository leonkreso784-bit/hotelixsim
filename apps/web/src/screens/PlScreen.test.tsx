import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import { HOTEL_ID, v2Api, v2PlApi } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("Račun dobiti", () => {
  it("sekcije redom USALI-ja i razlika riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ })).toBeTruthy();
    const titles = screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent);
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
    expect(await screen.findByText(/sezona do sada/i)).toBeTruthy();
  });
  it("prije prvog obračuna: rečenica, ne nule", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "no_resolved_round")),
      "/racun-dobiti",
    );
    expect(await screen.findByText("Račun dobiti postoji tek nakon prvog obračunatog kruga.")).toBeTruthy();
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
});
