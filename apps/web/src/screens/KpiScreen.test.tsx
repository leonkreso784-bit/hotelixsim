import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import { game, ix, kpiMonth, myHotel, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const julKpi = kpiMonth({
  month: "jul",
  sold_nights: 3100,
  lost_nights: 0,
  occupancy_bp: 8333,
  adr_cents: 15_010,
  break_even_occupancy_bp: 12_121,
});

function withKpi(months: ReturnType<typeof kpiMonth>[]) {
  return standardApi()
    .on("GET", "/v1/games/:game", game({ current: { month: "aug", status: "open", unanswered: 0 } }))
    .on("GET", "/v1/hotels/:hotel/kpi", { months });
}

describe("Pokazatelji", () => {
  it("računalo: tablica pokazatelji × mjeseci, uz svaki pokazatelj rečenica, brojke s jedinicom", async () => {
    renderApp(withKpi([kpiMonth(), julKpi]), "/pokazatelji");
    expect(await screen.findByRole("heading", { level: 1, name: "Pokazatelji" })).toBeTruthy();
    const table = await screen.findByRole("table", { name: "Pokazatelji po mjesecima" });
    const cols = within(table)
      .getAllByRole("columnheader")
      .map((c) => c.textContent);
    expect(cols).toEqual(["Pokazatelj", "Lipanj", "Srpanj"]);
    expect(table.closest("section")?.className).toMatch(/ hidden .*lg:block/);
    const rows = within(table).getAllByRole("row");
    expect(rows).toHaveLength(13);
    const adr = within(table).getByRole("rowheader", { name: /^ADR/ });
    expect(adr.textContent).toContain("Prosječna cijena prodane sobe: prihod od soba ÷ prodane noći.");
    const adrRow = adr.closest("tr") as HTMLElement;
    expect(within(adrRow).getByText("132,45 €")).toBeTruthy();
    expect(within(adrRow).getByText("150,10 €")).toBeTruthy();
    const occ = within(table)
      .getByRole("rowheader", { name: /^Popunjenost/ })
      .closest("tr") as HTMLElement;
    expect(within(occ).getByText("68 %")).toBeTruthy();
    expect(within(occ).getByText("83,3 %")).toBeTruthy();
    const lost = within(table)
      .getByRole("rowheader", { name: /^Izgubljene noći/ })
      .closest("tr") as HTMLElement;
    expect(within(lost).getByText("60 noći")).toBeTruthy();
    const be = within(table)
      .getByRole("rowheader", { name: /^Točka pokrića/ })
      .closest("tr") as HTMLElement;
    expect(within(be).getByText("52,4 %")).toBeTruthy();
  });

  it("točka pokrića rečenicom, i kad je iznad punog hotela", async () => {
    renderApp(withKpi([kpiMonth(), julKpi]), "/pokazatelji");
    expect(
      await screen.findAllByText("GOP je nula pri popunjenosti od 52,4 %; ti si bio na 68 %."),
    ).not.toHaveLength(0);
    expect(
      screen.getAllByText(
        "GOP bi bio nula tek pri popunjenosti od 121,2 %, više od punog hotela; ti si bio na 83,3 %.",
      ),
    ).not.toHaveLength(0);
  });

  it("mobitel: kartica po mjesecu s istim pokazateljima i rečenicama", async () => {
    renderApp(withKpi([kpiMonth(), julKpi]), "/pokazatelji");
    const jun = await screen.findByRole("heading", { level: 2, name: "Lipanj" });
    const card = jun.closest("section") as HTMLElement;
    expect(card.parentElement?.className).toContain("lg:hidden");
    const items = within(card).getAllByRole("term");
    expect(items).toHaveLength(12);
    expect(within(card).getByText("RevPAR")).toBeTruthy();
    expect(within(card).getByText("90,07 €")).toBeTruthy();
    expect(
      within(card).getByText(
        "Prihod od soba po svakoj sobi hotela: prihod od soba ÷ (sve sobe × dani), isto što i ADR × popunjenost.",
      ),
    ).toBeTruthy();
    expect(within(card).getByText("GOP je nula pri popunjenosti od 52,4 %; ti si bio na 68 %.")).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Srpanj" })).toBeTruthy();
  });

  it("mjesec bez prodaje: crtica umjesto ADR-a, točka pokrića se ne može izračunati", async () => {
    const empty = kpiMonth({
      sold_nights: 0,
      occupancy_bp: 0,
      adr_cents: null,
      gop_margin_bp: null,
      labor_share_bp: null,
      cpor_cents: null,
      break_even_occupancy_bp: null,
    });
    renderApp(withKpi([empty]), "/pokazatelji");
    const table = await screen.findByRole("table");
    const adrRow = within(table).getByRole("rowheader", { name: /^ADR/ }).closest("tr") as HTMLElement;
    expect(within(adrRow).getByText("—")).toBeTruthy();
    expect(
      screen.getAllByText(
        "Točka pokrića se ne može izračunati: prodaja nije pokrila ni izravni trošak soba.",
      ),
    ).not.toHaveLength(0);
  });

  it("prije prvog obračuna: prazno stanje, bez tablice", async () => {
    renderApp(standardApi(), "/pokazatelji");
    expect(await screen.findByText("Pokazatelji se računaju nakon prvog mjeseca.")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
  });

  it("greška poslužitelja je hrvatska rečenica", async () => {
    renderApp(
      standardApi().on("GET", "/v1/hotels/:hotel/kpi", () => problem(503, "storage_unavailable")),
      "/pokazatelji",
    );
    expect((await screen.findByRole("alert")).textContent).toBe(
      "Baza trenutno nije dostupna. Pokušaj ponovno.",
    );
  });

  it("Moj hotel: redak Pokazatelji s ADR i RevPAR zadnjeg mjeseca vodi na ekran", async () => {
    const api = withKpi([kpiMonth()]).on(
      "GET",
      "/v1/me/hotel",
      myHotel({
        months: [{ month: "jun", indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 }],
        planMonth: "jul",
        lastMonth: {
          month: "jun",
          sold: 2448,
          available: 3540,
          rooms_revenue_cents: 32_423_760,
          gop_cents: 11_650_000,
          goppar_cents: 3236,
          mentor: null,
          occupancy_bp: 6800,
          adr_cents: 13_245,
          revpar_cents: 9007,
        },
      }),
    );
    renderApp(api);
    const link = await screen.findByRole("link", { name: /^Pokazatelji/ });
    expect(link.getAttribute("href")).toBe("/pokazatelji");
    expect(link.textContent).toContain("ADR 132,45 € · RevPAR 90,07 €");
    fireEvent.click(link);
    expect(await screen.findByRole("heading", { level: 1, name: "Pokazatelji" })).toBeTruthy();
    expect(window.location.pathname).toBe("/pokazatelji");
    expect(screen.getByRole("link", { name: "Hotel" }).getAttribute("aria-current")).toBe("page");
  });

  it("Moj hotel prije prvog obračuna: redak Pokazatelji bez brojki", async () => {
    renderApp(standardApi());
    const link = await screen.findByRole("link", { name: /^Pokazatelji/ });
    expect(link.textContent).not.toContain("ADR");
  });
});
