import { act, cleanup, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { navigate } from "../router";
import { json, problem } from "../test/fakeApi";
import {
  explanation,
  game,
  ix,
  myHotel,
  myPublic,
  resolvedWeekApi,
  robot,
  roundInfo,
  standardApi,
  v2Api,
  weeklyGame,
  weeklyHotel,
} from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const jun = (p: number) => ({
  month: "jun" as const,
  round: "jun",
  indices: ix(5600, 5200, 5000, 5400, 4800),
  progress_hundredths: p,
});

const resolvedGame = game({
  current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 },
  hotels: [
    myPublic([jun(800)]),
    robot("h2", 2, "Štediša", "stedisa", [jun(400)]),
    robot("h3", 3, "Brižni", "brizni", [jun(600)]),
    robot("h4", 4, "Lovac na popunjenost", "lovac", [jun(300)]),
  ],
});

const feed: FeedItemBody[] = [
  { kind: "round_closed", seq: 1, month: "jun", round: "jun", by: "guest" },
  {
    kind: "change",
    seq: 2,
    month: "jun",
    round: "jun",
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
    round: "jun",
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
    round: "jun",
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
    round: "jun",
    indices: ix(5600, 5200, 5000, 5400, 4800),
    progress_hundredths: 800,
  },
  { kind: "mentor", seq: 6, month: "jun", round: "jun", text: "Usluga je bolja. Prati troškove." },
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
    expect(within(hero).getByText("+8 bodova")).toBeTruthy();
    // Pet traka s vrijednostima na kraju lipnja (ne početnih 50) i promjenom u lipnju.
    // Upiti unutar sekcije indeksa: „Zašto se indeksi pomaknuli“ se učitava zasebno i pokazuje iste promjene,
    // pa bi upit po cijelom ekranu ovisio o tome stigne li ona prije provjere (CI 10.10.).
    const bars = screen.getByText("Indeksi na kraju mjeseca").closest("section") as HTMLElement;
    expect(screen.getByRole("img", { name: "Zadovoljstvo gostiju: 56 od 100" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "Recenzije: 52 od 100" })).toBeTruthy();
    expect(screen.getByRole("img", { name: "Financije: 48 od 100" })).toBeTruthy();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    expect(screen.getByText("u ovom mjesecu")).toBeTruthy();
    expect(within(bars).getByText("▲ +6")).toBeTruthy();
    expect(within(bars).getByText("▼ −2")).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
    // Sirovi retci iz dnevnika (kod situacije, „osnova“) više se ne pokazuju.
    expect(screen.queryByText("Zašto (svaka promjena s uzrokom)")).toBeNull();
    expect(screen.queryByText("situacija S1")).toBeNull();
    expect(await screen.findByRole("heading", { name: "Zašto se indeksi pomaknuli" })).toBeTruthy();
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

  it("na mobitelu redom: rezultat, indeksi, mentor, ljestvica, objašnjenje; boje kartica kroz ton (I1, I2)", async () => {
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });
    renderApp(api, "/mjesec/jun");
    await screen.findByText("Usluga je bolja. Prati troškove.");
    const section = (text: string) => screen.getByText(text).closest("section") as HTMLElement;
    const cards = {
      result: section("Napredak od početka"),
      indices: section("Indeksi na kraju mjeseca"),
      mentor: section("Mentor"),
      board: section("Ljestvica napretka"),
      why: section("Zašto se indeksi pomaknuli"),
    };
    // `order` na omotaču s `contents` ne djeluje: smije biti samo na samim karticama.
    for (const el of document.querySelectorAll(".contents")) {
      expect(el.className.split(/\s+/).some((c) => /^order-/.test(c))).toBe(false);
    }
    // Redoslijed na mobitelu = (order bez prefiksa, pa položaj u dokumentu), kako ga slaže flex.
    const mobileOrder = (el: HTMLElement) => {
      const c = el.className.split(/\s+/).find((x) => /^order-\d+$/.test(x));
      return c ? Number(c.slice(6)) : 0;
    };
    const all = Object.entries(cards);
    const sorted = [...all].sort(([, a], [, b]) => {
      const d = mobileOrder(a) - mobileOrder(b);
      if (d !== 0) return d;
      return a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
    });
    expect(sorted.map(([k]) => k)).toEqual(["result", "indices", "mentor", "board", "why"]);
    expect(cards.result.className).toContain("bg-accent-soft");
    expect(cards.result.className).not.toContain("bg-surface");
    expect(cards.mentor.className).toContain("bg-mint");
    expect(cards.mentor.className).not.toContain("bg-surface");
  });

  it("dok se obračunava: čeka i ispituje, pa pokaže rezultat (otvoreno izravno, kao nakon osvježavanja)", async () => {
    let calls = 0;
    const closing = game({
      current: { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 },
    });
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
    const closing = game({
      current: { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 },
    });
    const api = standardApi().on("GET", "/v1/games/:game", closing);
    renderApp(api, "/mjesec/jun");
    expect(
      await screen.findByText(
        "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi profesoru.",
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

  it("mentorov tekst s prijelomima redaka čuva retke", async () => {
    const text = "Usluga je bolja.\nPrati troškove.\nCijene drži.\nSretno.";
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", {
        items: [...feed.slice(0, 5), { kind: "mentor", seq: 6, month: "jun", round: "jun", text }],
      });
    renderApp(api, "/mjesec/jun");
    const el = await screen.findByText((_, node) => node?.tagName === "P" && node.textContent === text);
    expect(el.className).toContain("whitespace-pre-line");
  });

  it("spor obračun: mentor koji stigne tek nakon obračuna se vidi, sat ne kreće dok traje „closing“", async () => {
    let gameCalls = 0;
    let resolved = false;
    const closing = game({
      current: { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 },
    });
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
      json(
        200,
        game({ current: { month: current, round: roundInfo(current), status: "closing", unanswered: 0 } }),
      ),
    );
    renderApp(api, "/mjesec/jun");
    const slow =
      "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi profesoru.";
    expect(await screen.findByText(slow, {}, { timeout: 2000 })).toBeTruthy();
    current = "jul";
    act(() => navigate("/mjesec/jul"));
    expect(await screen.findByText("Obračunavam srpanj…")).toBeTruthy();
    expect(screen.queryByText(slow)).toBeNull();
  });

  it("zadnji mjesec: nema „Nastavi“, ima „Natrag na hotel“", async () => {
    const jul = {
      month: "jul" as const,
      round: "jul",
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
        items: [{ kind: "mentor", seq: 9, month: "jul", round: "jul", text: "Dobra sezona." }],
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

describe("Mjesec završen: zašto se indeksi pomaknuli", () => {
  const settledApi = () =>
    standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });

  async function whyCard() {
    const h = await screen.findByRole("heading", { name: "Zašto se indeksi pomaknuli" });
    return h.closest("section") as HTMLElement;
  }

  it("po indeksu G R Z O F: naziv, prije → poslije, promjena; najveća promjena otvorena", async () => {
    const api = settledApi();
    renderApp(api, "/mjesec/jun");
    const card = await whyCard();
    const details = await within(card).findAllByRole("group");
    expect(details).toHaveLength(5);
    const summaries = details.map((d) => d.querySelector("summary")?.textContent ?? "");
    expect(summaries[0]).toContain("Zadovoljstvo gostiju");
    expect(summaries[0]).toContain("50 → 56");
    expect(summaries[0]).toContain("▲ +6");
    expect(summaries.slice(1).map((s) => s.split(/\d/)[0])).toEqual([
      "Recenzije",
      "Zaposlenici",
      "Operativa",
      "Financije",
    ]);
    expect(summaries[4]).toContain("50 → 48");
    expect(summaries[4]).toContain("▼ −2");
    expect(details.map((d) => (d as HTMLDetailsElement).open)).toEqual([true, false, false, false, false]);
    // Ikona uz naziv kao na trakama, skrivena čitaču.
    expect(details[0]?.querySelector("summary svg[aria-hidden='true']")).not.toBeNull();
    expect(api.calls.some((c) => c.path === "/v1/hotels/h1/months/jun/explanation")).toBe(true);
  });

  it("otvoreno: varijabla, rečenica značenja sitno, retci u boji po smjeru", async () => {
    renderApp(settledApi(), "/mjesec/jun");
    const card = await whyCard();
    const z = (await within(card).findAllByRole("group"))[2] as HTMLDetailsElement;
    z.open = true;
    expect(within(z).getByText("Moral")).toBeTruthy();
    expect(within(z).getByText("Raspoloženje i volja tima, na ljestvici do 100 bodova.").className).toContain(
      "text-muted",
    );
    expect(within(z).getByText("−10 bodova").className).toContain("text-down");
    expect(within(z).getByText("−1 bod").className).toContain("text-muted");
    expect(within(z).getByText("otišle 2 osobe").className).toContain("text-muted");
    const g = (await within(card).findAllByRole("group"))[0] as HTMLElement;
    expect(within(g).getByText("+5 bodova").className).toContain("text-up");
    expect(within(g).getByText("situacija „Klima u sobi obitelji s djecom“")).toBeTruthy();
    // Trošak je razina, ne dobitak: bez zelene boje.
    const f = (await within(card).findAllByRole("group"))[4] as HTMLElement;
    expect(within(f).getByText("117.040 €").className).toContain("text-muted");
  });

  it("otvoren je indeks s najvećom apsolutnom promjenom, i kad je to pad", async () => {
    const base = explanation();
    const indices = base.indices.map((i) => (i.code === "F" ? { ...i, after: 43, change: -7 } : i));
    renderApp(
      settledApi().on("GET", "/v1/hotels/:hotel/months/:month/explanation", explanation({ indices })),
      "/mjesec/jun",
    );
    const details = await within(await whyCard()).findAllByRole("group");
    expect(details.map((d) => (d as HTMLDetailsElement).open)).toEqual([false, false, false, false, true]);
  });

  it("dok se učitava: „Učitavam…“; greška: hrvatska rečenica", async () => {
    renderApp(
      settledApi().on(
        "GET",
        "/v1/hotels/:hotel/months/:month/explanation",
        () => new Promise<Response>(() => {}),
      ),
      "/mjesec/jun",
    );
    const card = await whyCard();
    expect(within(card).getByRole("status").textContent).toBe("Učitavam…");
    cleanup();
    renderApp(
      settledApi().on("GET", "/v1/hotels/:hotel/months/:month/explanation", () =>
        problem(503, "storage_unavailable"),
      ),
      "/mjesec/jun",
    );
    const alert = await within(await whyCard()).findByRole("alert");
    expect(alert.textContent).not.toMatch(/undefined|storage_unavailable/);
    expect(alert.textContent?.length).toBeGreaterThan(10);
  });

  it("veliki broj napretka u boji po predznaku; mentorov lik poravnat gore", async () => {
    renderApp(settledApi(), "/mjesec/jun");
    const hero = (await screen.findByText("Napredak od početka")).parentElement as HTMLElement;
    expect(within(hero).getByText("+8 bodova").className).toContain("text-up");
    const mentor = (await screen.findByText("Mentor")).closest("section") as HTMLElement;
    expect(mentor.className).toContain("items-start");
    expect(mentor.className).not.toContain("items-center");
  });

  it("ljestvica: −0,3 boda se zaokruži na 0 i nije crveno; brojevi imaju jedinicu", async () => {
    const g = game({
      current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 },
      hotels: [myPublic([jun(800)]), robot("h2", 2, "Štediša", "stedisa", [jun(-30)])],
    });
    renderApp(
      standardApi()
        .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
        .on("GET", "/v1/games/:game", g),
      "/mjesec/jun",
    );
    const board = await screen.findByRole("list", { name: "Ljestvica napretka" });
    const rows = within(board).getAllByRole("listitem");
    const mine = within(rows[0] as HTMLElement).getByText("+8 bodova");
    expect(mine.className).toContain("text-up");
    expect(mine.className).toContain("whitespace-nowrap");
    const zero = within(rows[1] as HTMLElement).getByText("0 bodova");
    expect(zero.className).not.toContain("text-down");
    expect(zero.className).not.toContain("text-up");
  });

  it.each([
    [-300, "−3 boda", "text-down"],
    [20, "0 bodova", "text-muted"],
  ])("napredak %i: %s u boji %s", async (p, shown, tone) => {
    const g = game({
      current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 },
      hotels: [myPublic([jun(p)]), robot("h2", 2, "Štediša", "stedisa", [jun(400)])],
    });
    renderApp(
      standardApi()
        .on("GET", "/v1/me/hotel", myHotel({ months: [jun(p)], planMonth: "jul" }))
        .on("GET", "/v1/games/:game", g),
      "/mjesec/jun",
    );
    const hero = (await screen.findByText("Napredak od početka")).parentElement as HTMLElement;
    expect(within(hero).getByText(shown).className).toContain(tone);
  });
});

describe("krug tjedna", () => {
  it("završen tjedan na /krug/t01", async () => {
    renderApp(resolvedWeekApi(), "/krug/t01");
    expect(
      await screen.findByRole("heading", { level: 1, name: "Tjedan 1 završen (1. 6. – 7. 6.)" }),
    ).toBeTruthy();
  });
});

describe("krug tjedna: mentor, ljestvica, nastavak", () => {
  it("tjedan usred mjeseca: ljestvica i nastavak, mentor se javlja tek na kraju mjeseca", async () => {
    const api = resolvedWeekApi();
    renderApp(api, "/krug/t01");
    expect(await screen.findByText("Mentor se javlja na kraju mjeseca, uz mjesečnu lekciju.")).toBeTruthy();
    expect(screen.queryByText("Mentor piše…")).toBeNull();
    expect(screen.getByText("Indeksi na kraju tjedna")).toBeTruthy();
    const board = screen.getByRole("list", { name: "Ljestvica napretka" });
    expect(within(board).getAllByRole("listitem")).toHaveLength(4);
    expect(screen.getByRole("link", { name: "Nastavi u tjedan 2 (8. 6. – 14. 6.)" })).toBeTruthy();
    // Mentor se ne čeka, pa dnevnik nema ponavljanih dohvata.
    expect(api.calls.filter((c) => c.path.endsWith("/feed")).length).toBeLessThanOrEqual(1);
  });

  it("tjedan koji zatvara mjesec (t05): čeka se mentor i prikazuje njegov tekst", async () => {
    const scored = [
      {
        month: "jun" as const,
        round: "t05",
        indices: ix(5600, 5200, 5000, 5400, 4800),
        progress_hundredths: 800,
      },
    ];
    const api = resolvedWeekApi()
      .on("GET", "/v1/me/hotel", weeklyHotel({ week: 6, months: scored }))
      .on("GET", "/v1/games/:game", weeklyGame({ week: 6, hotels: [myPublic(scored)] }))
      .on("GET", "/v1/hotels/:hotel/feed", {
        items: [{ kind: "mentor", seq: 1, month: "jun", round: "t05", text: "Lipanj je gotov." }],
      });
    renderApp(api, "/krug/t05");
    expect(
      await screen.findByRole("heading", { level: 1, name: "Tjedan 5 završen (29. 6. – 5. 7.)" }),
    ).toBeTruthy();
    expect(await screen.findByText("Lipanj je gotov.")).toBeTruthy();
  });

  it("tjedan koji još nije zatvoren: rečenica o tjednu", async () => {
    renderApp(resolvedWeekApi(), "/krug/t03");
    expect(await screen.findByText("Ovaj tjedan još nije zatvoren.")).toBeTruthy();
  });

  it("tjedan u obračunu: naslov s brojem tjedna", async () => {
    const api = v2Api().on("GET", "/v1/games/:game", weeklyGame({ status: "closing" }));
    renderApp(api, "/krug/t01");
    expect(await screen.findByText("Obračunavam tjedan 1…")).toBeTruthy();
  });
});
