import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ExplanationBody } from "../api/types";
import { json, problem } from "../test/fakeApi";
import { explanation, game, ix, myHotel, standardApi, v2Api, weeklyHotel } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const jun = {
  month: "jun" as const,
  round: "jun",
  indices: ix(5300, 5200, 5100, 5400, 4800),
  progress_hundredths: 800,
};
const jul = {
  month: "jul" as const,
  round: "jul",
  indices: ix(5600, 5100, 5000, 5500, 4900),
  progress_hundredths: 900,
};

/** Dva obračunata mjeseca; objašnjenje za srpanj ima drugačiji redak za Zaposlenike. */
function twoMonths() {
  const julExplanation: ExplanationBody = explanation({
    month: "jul",
    round: "jul",
    indices: explanation().indices.map((i) =>
      i.code === "Z"
        ? {
            ...i,
            before: 51,
            after: 50,
            change: -1,
            drivers: [
              {
                var: "morale",
                name: "Moral",
                meaning: "Raspoloženje i volja tima, na ljestvici do 100 bodova.",
                lines: [{ cause: "gužva u srpnju", amount: "−1 bod", effect: "down" }],
              },
            ],
          }
        : i,
    ),
  });
  return standardApi()
    .on("GET", "/v1/me/hotel", myHotel({ months: [jun, jul], planMonth: null }))
    .on("GET", "/v1/games/:game", game({ current: null }))
    .on("GET", "/v1/hotels/:hotel/months/:month/explanation", (_req, p) =>
      json(200, p.month === "jul" ? julExplanation : explanation()),
    );
}

describe("Detalj pokazatelja", () => {
  it("naslov, što mjeri, ljestvica i povijest: početak pa svaki mjesec s promjenom prema prethodnom", async () => {
    renderApp(twoMonths(), "/pokazatelj/Z");
    expect(await screen.findByRole("heading", { name: "Zaposlenici" })).toBeTruthy();
    expect(screen.getByText("Koliko je tim zadovoljan i ostaje li u hotelu.")).toBeTruthy();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    const rows = within(screen.getByRole("list", { name: "Kroz mjesece" })).getAllByRole("listitem");
    expect(rows.map((r) => r.textContent)).toEqual([
      expect.stringMatching(/^Početak50$/),
      expect.stringMatching(/^Lipanj51▲ \+1$/),
      expect.stringMatching(/^Srpanj50▼ −1$/),
    ]);
    expect(within(rows[0] as HTMLElement).getByRole("img", { name: "Zaposlenici: 50 od 100" })).toBeTruthy();
  });

  it("na detalju pokazatelja aktivan je donji tab „Hotel“", async () => {
    renderApp(twoMonths(), "/pokazatelj/Z");
    await screen.findByRole("heading", { name: "Zaposlenici" });
    const nav = screen.getByRole("navigation", { name: "Glavni izbornik" });
    const active = nav.querySelector("[aria-current='page']");
    expect(active?.textContent).toBe("Hotel");
  });

  it("objašnjenje zadnjeg mjeseca iz poslužitelja, boje po učinku", async () => {
    const api = twoMonths();
    renderApp(api, "/pokazatelj/Z");
    expect(await screen.findByText("gužva u srpnju")).toBeTruthy();
    expect(screen.getByText("−1 bod").className).toContain("text-down");
    expect(screen.getByText(/što ga je pomaknulo: srpanj/)).toBeTruthy();
    expect(api.calls.some((c) => c.path === "/v1/hotels/h1/months/jul/explanation")).toBe(true);
    expect(api.calls.some((c) => c.path === "/v1/hotels/h1/months/jun/explanation")).toBe(false);
  });

  it("birač mjeseca mijenja objašnjenje", async () => {
    renderApp(twoMonths(), "/pokazatelj/Z");
    await screen.findByText("gužva u srpnju");
    const jun = screen.getByRole("button", { name: "Lipanj" });
    const jul = screen.getByRole("button", { name: "Srpanj" });
    expect(jul.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(jun);
    expect(await screen.findByText("plaće")).toBeTruthy();
    expect(screen.queryByText("gužva u srpnju")).toBeNull();
    expect(jun.getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByText("−10 bodova").className).toContain("text-down");
    expect(screen.getByText("−1 bod").className).toContain("text-muted");
  });

  it("prije prvog obračuna: vrijednost s početka i rečenica, bez dohvata objašnjenja", async () => {
    const api = standardApi();
    renderApp(api, "/pokazatelj/R");
    expect(await screen.findByRole("heading", { name: "Recenzije" })).toBeTruthy();
    const rows = within(screen.getByRole("list", { name: "Kroz mjesece" })).getAllByRole("listitem");
    expect(rows).toHaveLength(1);
    expect(rows[0]?.textContent).toBe("Početak50");
    expect(screen.getByText("Promjene ćeš vidjeti nakon prvog mjeseca.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Lipanj" })).toBeNull();
    expect(api.calls.some((c) => c.path.endsWith("/explanation"))).toBe(false);
  });

  it("istaknuta varijabla iz adrese je prva i označena", async () => {
    renderApp(
      twoMonths().on("GET", "/v1/hotels/:hotel/months/:month/explanation", explanation()),
      "/pokazatelj/Z/staff",
    );
    expect(await screen.findByText("odlasci zaposlenih (zamijenjeni novima)")).toBeTruthy();
    const names = screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent);
    expect(names).toEqual(["Zaposleni", "Moral"]);
    const marked = document.querySelectorAll("[aria-current='true']");
    expect(marked).toHaveLength(1);
    expect(within(marked[0] as HTMLElement).getByText("Zaposleni")).toBeTruthy();
  });

  it("bez istaknute varijable redoslijed je poslužiteljev i ništa nije označeno", async () => {
    renderApp(
      twoMonths().on("GET", "/v1/hotels/:hotel/months/:month/explanation", explanation()),
      "/pokazatelj/Z",
    );
    await screen.findByText("odlasci zaposlenih (zamijenjeni novima)");
    expect(screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent)).toEqual([
      "Moral",
      "Zaposleni",
    ]);
    expect(document.querySelector("[aria-current='true']")).toBeNull();
  });

  it("indeks bez pokretača kaže da ga mjesec nije pomaknuo", async () => {
    const empty = explanation().indices.map((i) => (i.code === "Z" ? { ...i, drivers: [] } : i));
    renderApp(
      twoMonths().on("GET", "/v1/hotels/:hotel/months/:month/explanation", explanation({ indices: empty })),
      "/pokazatelj/Z",
    );
    expect(await screen.findByText("Ovaj ga mjesec ništa nije pomaknulo.")).toBeTruthy();
  });

  it("natrag na Moj hotel", async () => {
    renderApp(twoMonths(), "/pokazatelj/G");
    const back = await screen.findByRole("link", { name: "Natrag na hotel" });
    expect(back.getAttribute("href")).toBe("/");
    fireEvent.click(back);
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
  });

  it("nepoznat kod ili varijabla: stranica ne postoji", async () => {
    renderApp(twoMonths(), "/pokazatelj/X");
    expect(await screen.findByText("Ova stranica ne postoji.")).toBeTruthy();
    cleanup();
    renderApp(twoMonths(), "/pokazatelj/Z/nema");
    expect(await screen.findByText("Ova stranica ne postoji.")).toBeTruthy();
  });

  it("greška objašnjenja je hrvatska rečenica, povijest ostaje", async () => {
    renderApp(
      twoMonths().on(
        "GET",
        "/v1/hotels/:hotel/months/:month/explanation",
        () =>
          new Response(JSON.stringify({ title: "x", status: 503, code: "storage_unavailable" }), {
            status: 503,
            headers: { "content-type": "application/problem+json" },
          }),
      ),
      "/pokazatelj/Z",
    );
    expect((await screen.findByRole("alert")).textContent).not.toMatch(/undefined|storage_unavailable/);
    expect(screen.getByRole("list", { name: "Kroz mjesece" })).toBeTruthy();
  });
});

describe("pokazatelj: tjedna igra", () => {
  it("povijest i izbor kruga su tjedni; objašnjenje se traži za izabrani kod", async () => {
    const wk = (round: string, g: number) => ({
      month: "jun" as const,
      round,
      indices: ix(g, 5000, 5000, 5000, 5000),
      progress_hundredths: 100,
    });
    const scored = [wk("t01", 5300), wk("t02", 5600)];
    const api = v2Api().on("GET", "/v1/me/hotel", weeklyHotel({ week: 3, months: scored }));
    renderApp(api, "/pokazatelj/G");
    expect(await screen.findByRole("heading", { name: "Kroz tjedne" })).toBeTruthy();
    expect(
      await screen.findByRole("heading", {
        name: "Od čega se sastoji i što ga je pomaknulo: tjedan 2 (8. 6. – 14. 6.)",
      }),
    ).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Tjedan 1 (1. 6. – 7. 6.)" }));
    expect(
      await screen.findByRole("heading", {
        name: "Od čega se sastoji i što ga je pomaknulo: tjedan 1 (1. 6. – 7. 6.)",
      }),
    ).toBeTruthy();
    expect(api.calls.some((c) => c.path.endsWith("/months/t01/explanation"))).toBe(true);
  });
});

describe("pokazatelj: birač tjedana i pad igre", () => {
  const wk = (round: string, g: number) => ({
    month: "jun" as const,
    round,
    indices: ix(g, 5000, 5000, 5000, 5000),
    progress_hundredths: 100,
  });
  const weekApi = () =>
    v2Api().on(
      "GET",
      "/v1/me/hotel",
      weeklyHotel({ week: 4, months: [wk("t01", 5300), wk("t02", 5600), wk("t03", 5700)] }),
    );

  it("mobitel: prethodni/sljedeći i izbornik svih krugova; računalo: sažeti čipovi s punim nazivom", async () => {
    const api = weekApi();
    renderApp(api, "/pokazatelj/G");
    const select = (await screen.findByRole("combobox", { name: "Tjedan obračuna" })) as HTMLSelectElement;
    expect(select.value).toBe("t03");
    expect(within(select).getAllByRole("option")).toHaveLength(3);
    fireEvent.click(screen.getByRole("button", { name: "Prethodni tjedan" }));
    expect(select.value).toBe("t02");
    expect(
      await screen.findByRole("heading", {
        name: "Od čega se sastoji i što ga je pomaknulo: tjedan 2 (8. 6. – 14. 6.)",
      }),
    ).toBeTruthy();
    fireEvent.change(select, { target: { value: "t01" } });
    expect((screen.getByRole("button", { name: "Prethodni tjedan" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    const chip = screen.getByRole("button", { name: "Tjedan 3 (15. 6. – 21. 6.)" });
    expect(chip.textContent).toBe("T3");
    expect(chip.getAttribute("title")).toBe("Tjedan 3 (15. 6. – 21. 6.)");
    fireEvent.click(chip);
    expect(select.value).toBe("t03");
  });

  it("igra se ne može učitati: ekran se ipak prikaže s imenima mjeseci", async () => {
    const api = standardApi()
      .on("GET", "/v1/games/:game", () => problem(500, "internal"))
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun] }));
    renderApp(api, "/pokazatelj/G");
    expect(await screen.findByRole("heading", { name: "Kroz mjesece" })).toBeTruthy();
  });
});
