import { act, cleanup, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { navigate } from "../router";
import { json } from "../test/fakeApi";
import { game, ix, myHotel, myPublic, robot, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const jun = (p: number) => ({
  month: "jun" as const,
  indices: ix(5600, 5200, 5000, 5400, 4800),
  progress_hundredths: p,
});

const resolvedGame = game({
  current: { month: "jul", status: "open", unanswered: 1 },
  hotels: [
    myPublic([jun(800)]),
    robot("h2", 2, "Štediša", "stedisa", [jun(400)]),
    robot("h3", 3, "Brižni", "brizni", [jun(600)]),
    robot("h4", 4, "Lovac na popunjenost", "lovac", [jun(300)]),
  ],
});

const feed: FeedItemBody[] = [
  { kind: "round_closed", seq: 1, month: "jun", by: "guest" },
  {
    kind: "change",
    seq: 2,
    month: "jun",
    var: "sold",
    cause: "price",
    amount: -1200,
    unit: "bp",
    situation: null,
  },
  {
    kind: "change",
    seq: 3,
    month: "jun",
    var: "satisfaction",
    cause: "situation",
    amount: 450,
    unit: "points",
    situation: "S1",
  },
  {
    kind: "change",
    seq: 4,
    month: "jun",
    var: "cash",
    cause: "fixed_costs",
    amount: -1_250_000,
    unit: "cents",
    situation: null,
  },
  {
    kind: "month_resolved",
    seq: 5,
    month: "jun",
    indices: ix(5600, 5200, 5000, 5400, 4800),
    progress_hundredths: 800,
  },
  { kind: "mentor", seq: 6, month: "jun", text: "Usluga je bolja. Prati troškove." },
];

describe("Mjesec završen", () => {
  it("obračunat lipanj: +8, promjene indeksa, uzroci, mentor, ljestvica, Nastavi u srpanj", async () => {
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByRole("heading", { name: "Lipanj završen" })).toBeTruthy();
    const hero = screen.getByText("Napredak od početka").parentElement as HTMLElement;
    expect(within(hero).getByText("+8")).toBeTruthy();
    // Pet traka s vrijednostima na kraju lipnja (ne početnih 50) i promjenom u lipnju.
    expect(screen.getByText("Indeksi na kraju mjeseca")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Zadovoljstvo gostiju: 56 od 100" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "Recenzije: 52 od 100" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "Financije: 48 od 100" })).toBeTruthy();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    expect(screen.getByText("u ovom mjesecu")).toBeTruthy();
    expect(screen.getByText("▲ +6")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
    const changes = screen.getByText("Zašto (svaka promjena s uzrokom)").closest("section") as HTMLElement;
    expect(within(changes).getByText("Prodane noći")).toBeTruthy();
    expect(within(changes).getByText("cijena")).toBeTruthy();
    expect(within(changes).getByText("−12 %")).toBeTruthy();
    expect(within(changes).getByText("situacija S1")).toBeTruthy();
    expect(within(changes).getByText("−12.500 €")).toBeTruthy();
    const board = screen.getByRole("list", { name: "Ljestvica napretka" });
    const rows = within(board).getAllByRole("listitem");
    expect(rows.map((r) => r.textContent)).toEqual([
      expect.stringContaining("Tvoj hotel"),
      expect.stringContaining("Brižni"),
      expect.stringContaining("Štediša"),
      expect.stringContaining("Lovac na popunjenost"),
    ]);
    expect(rows[0]?.getAttribute("aria-current")).toBe("true");
    expect(board.querySelectorAll("svg")).toHaveLength(4);
    expect(board.querySelectorAll('svg[aria-hidden="true"]')).toHaveLength(4);
    expect(rows[2]?.querySelector('[data-part="glasses-square"]')).not.toBeNull();
    expect(screen.getByRole("link", { name: "Nastavi u srpanj" }).getAttribute("href")).toBe("/");
  });

  it("dok se obračunava: čeka i ispituje, pa pokaže rezultat (otvoreno izravno, kao nakon osvježavanja)", async () => {
    let calls = 0;
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", () => {
        calls += 1;
        return json(200, calls < 3 ? closing : resolvedGame);
      })
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Obračunavam lipanj…")).toBeTruthy();
    expect(screen.queryByText("+0")).toBeNull();
    expect(await screen.findByRole("heading", { name: "Lipanj završen" })).toBeTruthy();
  });

  it("predugo obračunavanje: poruka da traje dulje", async () => {
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const api = standardApi().on("GET", "/v1/games/:game", closing);
    renderApp(api, "/mjesec/jun");
    expect(
      await screen.findByText(
        "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi Leonu.",
        {},
        { timeout: 2000 },
      ),
    ).toBeTruthy();
  });

  it("mentor još nije stigao: „Mentor piše…“, zatim tekst", async () => {
    let n = 0;
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", () => {
        n += 1;
        return json(200, { items: n < 3 ? feed.slice(0, 5) : feed });
      });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
  });

  it("spor obračun: mentor koji stigne tek nakon obračuna se vidi, sat ne kreće dok traje „closing“", async () => {
    let gameCalls = 0;
    let resolved = false;
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", () => {
        gameCalls += 1;
        if (gameCalls < 12) return json(200, closing);
        resolved = true;
        return json(200, resolvedGame);
      })
      // Mentor postoji tek kad je mjesec obračunat; „closing“ traje dulje od maxFeedPolls * feedMs (100 ms).
      .on("GET", "/v1/hotels/:hotel/feed", () => json(200, { items: resolved ? feed : feed.slice(0, 5) }));
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Obračunavam lipanj…")).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Lipanj završen" }, { timeout: 2000 })).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
    expect(screen.queryByText("Mentor se ovaj put nije javio.")).toBeNull();
  });

  it("mentor se uopće ne javi: nakon budžeta ispitivanja piše da se nije javio", async () => {
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed.slice(0, 5) });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByText("Mentor se ovaj put nije javio.", {}, { timeout: 2000 })).toBeTruthy();
  });

  it("prijelaz lipanj → srpanj: sat „traje dulje“ iz lipnja ne prelazi u srpanj", async () => {
    let current: "jun" | "jul" = "jun";
    const api = standardApi().on("GET", "/v1/games/:game", () =>
      json(200, game({ current: { month: current, status: "closing", unanswered: 0 } })),
    );
    renderApp(api, "/mjesec/jun");
    const slow = "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi Leonu.";
    expect(await screen.findByText(slow, {}, { timeout: 2000 })).toBeTruthy();
    current = "jul";
    act(() => navigate("/mjesec/jul"));
    expect(await screen.findByText("Obračunavam srpanj…")).toBeTruthy();
    expect(screen.queryByText(slow)).toBeNull();
  });

  it("zadnji mjesec: nema „Nastavi“, ima „Natrag na hotel“", async () => {
    const jul = {
      month: "jul" as const,
      indices: ix(5600, 5200, 5000, 5400, 5000),
      progress_hundredths: 1100,
    };
    const over = game({
      current: null,
      hotels: [
        myPublic([jun(800), jul]),
        robot("h2", 2, "Štediša", "stedisa", [jun(400), { ...jul, progress_hundredths: 500 }]),
      ],
    });
    const api = standardApi()
      .on("GET", "/v1/games/:game", over)
      .on("GET", "/v1/hotels/:hotel/feed", {
        items: [{ kind: "mentor", seq: 9, month: "jul", text: "Dobra sezona." }],
      });
    renderApp(api, "/mjesec/jul");
    expect(await screen.findByRole("heading", { name: "Srpanj završen" })).toBeTruthy();
    expect(screen.getByText("To je bio zadnji mjesec sezone.")).toBeTruthy();
    expect(screen.queryByRole("link", { name: /Nastavi u/ })).toBeNull();
    expect(screen.getByRole("link", { name: "Natrag na hotel" })).toBeTruthy();
  });

  it("mjesec koji još nije ni zatvoren", async () => {
    renderApp(standardApi(), "/mjesec/jul");
    expect(await screen.findByText("Ovaj mjesec još nije zatvoren.")).toBeTruthy();
  });
});
