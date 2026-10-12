import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { keepRoundTogether } from "../format";
import { problem } from "../test/fakeApi";
import { game, ix, kpiMonth, myHotel, roundInfo, standardApi, v2Api } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const julKpi = kpiMonth({
  month: "jul",
  round: "jul",
  sold_nights: 3100,
  lost_nights: 0,
  occupancy_bp: 8333,
  sellable_occupancy_bp: 10_000,
  adr_cents: 15_010,
  break_even_occupancy_bp: 12_121,
});

function withKpi(months: ReturnType<typeof kpiMonth>[]) {
  return standardApi()
    .on(
      "GET",
      "/v1/games/:game",
      game({ current: { month: "aug", round: roundInfo("aug"), status: "open", unanswered: 0 } }),
    )
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
    expect(rows).toHaveLength(16);
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
    // Ukupni iznosi u eurima, svaki sa svojom rečenicom.
    const money = (name: RegExp) =>
      within(table).getByRole("rowheader", { name }).closest("tr") as HTMLElement;
    expect(within(money(/^Prihod od soba/)).getAllByText("324.238 €")).toHaveLength(2);
    expect(within(money(/^Ukupni prihod/)).getAllByText("421.509 €")).toHaveLength(2);
    expect(within(money(/^Trošak rada/)).getAllByText("113.000 €")).toHaveLength(2);
    expect(within(money(/Bruto operativna dobit/)).getAllByText("116.500 €")).toHaveLength(2);
    const direct = within(table).getByRole("rowheader", { name: /^Izravni trošak po prodanoj noći/ });
    expect(direct.textContent).toContain("ne uključuje plaće");
    expect(within(direct.closest("tr") as HTMLElement).getAllByText("22,00 €")).toHaveLength(2);
    expect(within(table).queryByText("CPOR")).toBeNull();
  });

  it("točka pokrića rečenicom, i kad je iznad punog hotela", async () => {
    renderApp(withKpi([kpiMonth(), julKpi]), "/pokazatelji");
    expect(
      await screen.findAllByText("GOP je nula pri popunjenosti od 52,4 %; tvoja popunjenost bila je 68 %."),
    ).not.toHaveLength(0);
    expect(
      screen.getAllByText(
        "GOP bi bio nula tek pri popunjenosti od 121,2 %, a sobe u funkciji dopuštaju najviše 100 %; tvoja popunjenost bila je 83,3 %.",
      ),
    ).not.toHaveLength(0);
  });

  it("točka pokrića ispod 100 %, ali iznad onoga što sobe u funkciji dopuštaju", async () => {
    renderApp(withKpi([kpiMonth({ break_even_occupancy_bp: 9900 })]), "/pokazatelji");
    expect(
      await screen.findAllByText(
        "GOP bi bio nula tek pri popunjenosti od 99 %, a sobe u funkciji dopuštaju najviše 98,3 %; tvoja popunjenost bila je 68 %.",
      ),
    ).not.toHaveLength(0);
  });

  it("GOP pozitivan i bez prodaje: posebna rečenica", async () => {
    renderApp(withKpi([kpiMonth({ break_even_occupancy_bp: 0 })]), "/pokazatelji");
    expect(await screen.findAllByText("GOP je pozitivan i bez prodaje.")).not.toHaveLength(0);
  });

  it("prodaja koja ne pokriva izravni trošak: točka pokrića se ne može izračunati", async () => {
    renderApp(withKpi([kpiMonth({ break_even_occupancy_bp: null })]), "/pokazatelji");
    expect(
      await screen.findAllByText(
        "Točka pokrića se ne može izračunati: prodaja nije pokrila ni izravni trošak soba.",
      ),
    ).not.toHaveLength(0);
  });

  it("mobitel: kartica po mjesecu s istim pokazateljima i rečenicama", async () => {
    renderApp(withKpi([kpiMonth(), julKpi]), "/pokazatelji");
    const jun = await screen.findByRole("heading", { level: 2, name: "Lipanj" });
    const card = jun.closest("section") as HTMLElement;
    expect(card.parentElement?.className).toContain("lg:hidden");
    const items = within(card).getAllByRole("term");
    expect(items).toHaveLength(15);
    expect(within(card).getByText("RevPAR")).toBeTruthy();
    expect(within(card).getByText("90,07 €")).toBeTruthy();
    expect(
      within(card).getByText(
        "Prihod od soba po svakoj sobi hotela: prihod od soba ÷ (sve sobe × dani), isto što i ADR × popunjenost.",
      ),
    ).toBeTruthy();
    expect(
      within(card).getByText("GOP je nula pri popunjenosti od 52,4 %; tvoja popunjenost bila je 68 %."),
    ).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Srpanj" })).toBeTruthy();
  });

  it("mjesec bez prodaje: crtica umjesto ADR-a i rečenica da prodaje nije bilo", async () => {
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
    expect(screen.getAllByText("U mjesecu nije bilo prodaje.")).not.toHaveLength(0);
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
        months: [
          { month: "jun", round: "jun", indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 },
        ],
        planMonth: "jul",
        lastMonth: {
          month: "jun",
          round: roundInfo("jun"),
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
    expect(link.textContent).toContain("ADR\u00a0132,45\u00a0€ · RevPAR\u00a090,07\u00a0€");
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

describe("Pokazatelji: tjedna igra", () => {
  it("stupci su tjedni s datumima, uvod i prazno stanje govore o tjednu", async () => {
    const api = v2Api().on("GET", "/v1/hotels/:hotel/kpi", {
      months: [kpiMonth({ round: "t01", days: 7 }), kpiMonth({ round: "t02", days: 7 })],
    });
    renderApp(api, "/pokazatelji");
    const table = await screen.findByRole("table", { name: "Pokazatelji po tjednima" });
    expect(
      within(table)
        .getAllByRole("columnheader")
        .map((c) => c.textContent),
    ).toEqual([
      "Pokazatelj",
      keepRoundTogether("Tjedan 1 (1. 6. – 7. 6.)"),
      keepRoundTogether("Tjedan 2 (8. 6. – 14. 6.)"),
    ]);
    expect(screen.getByText(/dani u tjednu/)).toBeTruthy();
  });

  it("prije prvog tjedna: rečenica o tjednu", async () => {
    renderApp(v2Api(), "/pokazatelji");
    expect(await screen.findByText("Pokazatelji se računaju nakon prvog tjedna.")).toBeTruthy();
  });
});

describe("Pokazatelji: 17 tjedana i pad igre", () => {
  const weeks = Array.from({ length: 17 }, (_v, i) =>
    kpiMonth({ round: `t${String(i + 1).padStart(2, "0")}`, days: 7 }),
  );

  it("tablica: pomična regija dostupna tipkovnicom, prvi stupac ostaje vidljiv, objašnjenja sklopljena", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/kpi", { months: weeks }), "/pokazatelji");
    const region = await screen.findByRole("region", { name: "Pokazatelji po tjednima" });
    expect(region.getAttribute("tabindex")).toBe("0");
    expect(region.className).toContain("overflow-x-auto");
    const first = within(region).getAllByRole("rowheader")[0] as HTMLElement;
    expect(first.className).toContain("sticky");
    expect(first.className).toContain("bg-surface");
    expect(region.querySelectorAll("tbody details").length).toBe(15);
  });

  it("naslovi stupaca se ne lome usred datuma, a „Što znači?“ je meta od 44 px", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/kpi", { months: weeks.slice(0, 2) }), "/pokazatelji");
    const region = await screen.findByRole("region", { name: "Pokazatelji po tjednima" });
    for (const th of region.querySelectorAll("thead th[scope=col]:not(:first-child)"))
      expect(th.className).toContain("whitespace-nowrap");
    for (const sm of region.querySelectorAll("tbody summary")) expect(sm.className).toContain("min-h-11");
  });

  it("tjedna mjerila govore o tjednu", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/kpi", { months: weeks.slice(0, 1) }), "/pokazatelji");
    expect(
      (await screen.findAllByText("Plaće, zapošljavanje novih ljudi i obuka u tjednu.")).length,
    ).toBeGreaterThan(0);
  });

  it("igra se ne može učitati: pokazatelji se ipak vide, s imenima mjeseci", async () => {
    const api = standardApi()
      .on("GET", "/v1/games/:game", () => problem(500, "internal"))
      .on("GET", "/v1/hotels/:hotel/kpi", { months: [kpiMonth()] });
    renderApp(api, "/pokazatelji");
    const table = await screen.findByRole("table", { name: "Pokazatelji po mjesecima" });
    expect(within(table).getByRole("columnheader", { name: "Lipanj" })).toBeTruthy();
  });
});
