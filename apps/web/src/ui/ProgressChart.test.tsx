import { t } from "@hotelixsim/i18n";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { formatPointsPrecise, formatWholeSigned } from "../format";
import { axisTicks, curveColumns, curveSeries } from "../screens/season/rows";
import { seasonBody } from "../test/fixtures";
import { ProgressChart } from "./ProgressChart";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** Okvir zadane širine: ResizeObserver javi širinu odmah pri promatranju. */
function stubWidth(width: number) {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      cb: ResizeObserverCallback;
      constructor(cb: ResizeObserverCallback) {
        this.cb = cb;
      }
      observe() {
        this.cb([{ contentRect: { width } } as ResizeObserverEntry], this as unknown as ResizeObserver);
      }
      disconnect() {}
      unobserve() {}
    },
  );
}

function renderChart(weekly = true) {
  const body = seasonBody({ weekly });
  return render(
    <ProgressChart
      lines={curveSeries(body)}
      ticks={axisTicks(body)}
      columns={curveColumns(body)}
      formatValue={(v) => formatPointsPrecise(v, 0)}
      formatAxis={(n) => t("season.curve.axis", { n: formatWholeSigned(n) })}
      text={{
        title: t("season.curve"),
        desc: (label, value) => t("season.curve.desc", { hotel: label, progress: value }),
        zero: t("season.curve.zero"),
        show: t("season.curve.show"),
        hide: t("season.curve.hide"),
        table: t("season.curve.table"),
        rowHeader: t("season.curve.hotel"),
      }}
    />,
  );
}

describe("ProgressChart", () => {
  it("jedna crta po hotelu, slika s naslovom i opisom koji kaže gdje je tko završio", () => {
    const { container } = renderChart();
    const img = screen.getByRole("img", { name: "Krivulja napretka" });
    expect(img.tagName.toLowerCase()).toBe("svg");
    expect(container.querySelectorAll("[data-line]")).toHaveLength(4);
    const desc = img.querySelector("desc")?.textContent ?? "";
    expect(desc).toContain("Tvoj hotel završio je na +8 bodova.");
    expect(desc).toContain("Brižni (robot) završio je na +12 bodova.");
  });

  it("crte se razlikuju uzorkom i debljinom, ne samo bojom", () => {
    const { container } = renderChart();
    const lines = [...container.querySelectorAll("[data-line]")];
    const looks = lines.map((l) => `${l.getAttribute("stroke-dasharray")}|${l.getAttribute("stroke-width")}`);
    expect(new Set(looks).size).toBe(4);
    const robots = lines.filter((l) => l.getAttribute("data-style") !== "0");
    expect(new Set(robots.map((l) => l.getAttribute("stroke-dasharray"))).size).toBe(3);
    // Zadnja točka svake crte ima svoj znak (krug, kvadrat, trokut, romb).
    expect(
      new Set([...container.querySelectorAll("[data-marker]")].map((m) => m.tagName)).size,
    ).toBeGreaterThan(1);
  });

  it("legenda: uzorak crte, ime i konačni broj za svaki hotel", () => {
    renderChart();
    const legend = screen.getByRole("list", { name: "Krivulja napretka" });
    const items = within(legend).getAllByRole("listitem");
    // Redom poretka, kao ljestvica (UI KS-3 M-9).
    expect(items.map((i) => i.textContent)).toEqual([
      "Brižni (robot)+12 bodova",
      "Tvoj hotel+8 bodova",
      "Štediša (robot)−3 boda",
      "Lovac na popunjenost (robot)−7 bodova",
    ]);
  });

  it("os: mjeseci, nikad kod tjedna; „0 = početak“ uz crtu nule", () => {
    const { container } = renderChart();
    const svgText = container.querySelector("svg")?.textContent ?? "";
    for (const m of ["lipanj", "srpanj", "kolovoz", "rujan"]) expect(svgText).toContain(m);
    expect(svgText).not.toMatch(/t\d\d/);
    expect(screen.getByText(/Crta 0 = početak/)).toBeTruthy();
  });

  it("široka krivulja (686 px, zaslon 1024 px): os bez „Početak“, prvi mjesec na sredini odsječka (UI KS-3 I-A)", () => {
    // Na 686 px „Početak“ (poravnat na x osi Y) i „lipanj“ (sredina prvog mjeseca) sudarali su se u „Početaklipanj“.
    // Početak već kaže crta 0, pa ga os ne piše; svaka oznaka osi je ime mjeseca na sredini svog odsječka.
    stubWidth(686);
    for (const weekly of [true, false]) {
      const { container } = renderChart(weekly);
      const axis = [...container.querySelectorAll("svg text[data-axis-x]")];
      expect(axis.map((n) => n.textContent)).toEqual(["lipanj", "srpanj", "kolovoz", "rujan"]);
      expect(axis.every((n) => n.getAttribute("text-anchor") === "middle")).toBe(true);
      expect(container.querySelector("svg")?.textContent).not.toContain("Početak");
      cleanup();
    }
  });

  it("„Prikaži brojke“ otvara tablicu sa svim krugovima; prva točka je 0", () => {
    renderChart();
    const btn = screen.getByRole("button", { name: "Prikaži brojke" });
    expect(btn.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(btn);
    expect(btn.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("button", { name: "Sakrij brojke" })).toBe(btn);
    const table = screen.getByRole("table", { name: "Napredak po krugovima" });
    // Hotel + početak + 17 tjedana
    expect(within(table).getAllByRole("columnheader")).toHaveLength(19);
    const rows = within(table).getAllByRole("row").slice(1);
    expect(rows).toHaveLength(4);
    for (const r of rows) expect(within(r).getAllByRole("cell")[0]?.textContent).toBe("0 bodova");
    expect(within(table).queryByText(/^t\d\d$/)).toBeNull();
    expect(within(table).getByText("T5")).toBeTruthy();
  });

  it("regija tablice je pozicionirana, da skrivena puna imena krugova ne šire stranicu (C-1)", () => {
    // jsdom ne računa raspored (scrollWidth je uvijek 0), pa se čuva uzrok: `sr-only` je `position: absolute` i
    // bez pozicioniranog pretka u regiji za pomicanje računa se u širinu stranice. Mjerenje ostaje UI čuvaru (§9.3).
    renderChart();
    fireEvent.click(screen.getByRole("button", { name: "Prikaži brojke" }));
    const region = screen.getByRole("region", { name: "Napredak po krugovima" });
    expect(region.className.split(" ")).toContain("relative");
    expect(region.className.split(" ")).toContain("overflow-x-auto");
    // Rizik postoji: u regiji su skriveni (`sr-only`) dijelovi zaglavlja, koje `relative` drži unutar nje.
    expect(region.querySelectorAll(".sr-only").length).toBeGreaterThan(0);
  });

  it("uska krivulja (mobitel): os bez „Poč.“ i bez praznog teksta, mjeseci skraćeno (UI KS-3 M-3, m-3)", () => {
    stubWidth(360);
    const { container } = renderChart();
    const labels = [...container.querySelectorAll("svg text")].map((n) => n.textContent ?? "");
    expect(labels).toContain("lip.");
    expect(labels).not.toContain("Poč.");
    expect(labels).not.toContain("Početak");
    expect(labels).not.toContain("");
    const axis = [...container.querySelectorAll("svg text[data-axis-x]")].map((n) => n.textContent);
    expect(axis).toEqual(["lip.", "srp.", "kol.", "ruj."]);
  });

  it("mjesečna igra: četiri kruga i početak", () => {
    renderChart(false);
    fireEvent.click(screen.getByRole("button", { name: "Prikaži brojke" }));
    const table = screen.getByRole("table", { name: "Napredak po krugovima" });
    expect(within(table).getAllByRole("columnheader")).toHaveLength(6);
  });
});
