import { cleanup, configure, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { SeasonBody } from "../api/types";
import { json, problem } from "../test/fakeApi";
import { game, seasonApi, seasonBody, weeklyGame } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

const WAIT_MS = 15_000;
vi.setConfig({ testTimeout: 2 * WAIT_MS });

beforeEach(() => {
  fastPolling();
  configure({ asyncUtilTimeout: WAIT_MS });
});
afterEach(() => {
  configure({ asyncUtilTimeout: 1000 });
  cleanup();
  vi.unstubAllGlobals();
});

const SEASON_PATH = "/v1/games/g1/season";
const MENTOR = [
  "Kako je prošla sezona",
  "Napredak tvog hotela od početka sezone je +8 bodova.",
  "",
  "Što je išlo dobro",
  "Indeks Zaposlenici porastao je s 30 na 52.",
  "",
  "Što nije valjalo",
  "Nijedan indeks nije završio niže nego na početku.",
  "",
  "Što ponijeti u sljedeću igru",
  "Probaj osnovnu obuku.",
].join("\n");

function seasonCalls(api: ReturnType<typeof seasonApi>) {
  return api.calls.filter((c) => c.method === "GET" && c.path === SEASON_PATH).length;
}

async function open(body: SeasonBody = seasonBody({ mentor: MENTOR })) {
  const api = seasonApi(body);
  renderApp(api, "/kraj-sezone");
  expect(await screen.findByRole("heading", { level: 1, name: "Kraj sezone" })).toBeTruthy();
  return api;
}

/** Kartica hotela po naslovu („2. Tvoj hotel“). */
function card(title: string) {
  const h = screen.getByRole("heading", { level: 3, name: title });
  const section = h.closest("section");
  if (!section) throw new Error(`nema kartice ${title}`);
  return within(section);
}

function ranking() {
  return within(screen.getByRole("list", { name: "Konačni poredak" }));
}

describe("Kraj sezone: podaci", () => {
  it("izravno otvorena /kraj-sezone sama dohvati sezonu; tjedna igra", async () => {
    const api = await open();
    expect(seasonCalls(api)).toBe(1);
    expect(screen.getByText("17 tjedana je iza tebe. Ovako su prošla sva četiri hotela.")).toBeTruthy();
  });

  it("poredak redom mjesta, Tvoj hotel istaknut, pobjednik označen riječju", async () => {
    await open();
    const rows = ranking().getAllByRole("listitem");
    expect(rows.map((r) => r.textContent)).toEqual([
      "1.Brižni (robot)brine o ljudimaPobjednik+12 bodova",
      "2.Tvoj hotel+8 bodova",
      "3.Štediša (robot)štedi−3 boda",
      "4.Lovac na popunjenost (robot)lovi popunjenost−7 bodova",
    ]);
    expect(rows[1]?.getAttribute("aria-current")).toBe("true");
    expect(screen.getByText("Pobijedio je Brižni (robot) s napretkom od +12 bodova.")).toBeTruthy();
    expect(screen.getByText("Napredak od početka · 2. mjesto od 4")).toBeTruthy();
  });

  it("moj hotel pobijedio: rečenica bez roda", async () => {
    await open(seasonBody({ progress: [1500, -310, 1210, -700] }));
    expect(screen.getByText("Tvoj hotel je pobijedio: napredovao je najviše od svih.")).toBeTruthy();
    expect(ranking().getAllByRole("listitem")[0]?.textContent).toContain("Pobjednik");
  });

  it("jednak napredak dijeli mjesto", async () => {
    await open(seasonBody({ progress: [500, 500, 1200, -100] }));
    const ranks = ranking()
      .getAllByRole("listitem")
      .map((r) => r.textContent?.split(".")[0]);
    expect(ranks).toEqual(["1", "2", "2", "4"]);
    expect(screen.getByText("Napredak od početka · dijeliš 2. mjesto od 4")).toBeTruthy();
  });

  it("svi isti napredak: nitko nije posebno „Pobjednik“, jedna rečenica", async () => {
    await open(seasonBody({ progress: [300, 300, 300, 300] }));
    expect(screen.getByText("Svi hoteli su napredovali jednako i dijele prvo mjesto.")).toBeTruthy();
    expect(ranking().queryByText("Pobjednik")).toBeNull();
  });

  it("preciznost kao mentor: dva različita mjesta nisu oba „+8“", async () => {
    await open(seasonBody({ progress: [760, -310, 840, -700] }));
    const texts = ranking()
      .getAllByRole("listitem")
      .map((r) => r.textContent);
    expect(texts[0]).toMatch(/\+8,4 boda$/);
    expect(texts[1]).toMatch(/\+7,6 boda$/);
    expect(screen.getAllByText("+7,6 boda").length).toBeGreaterThan(0);
  });

  it("pet indeksa po hotelu s početkom, krajem i strelicom promjene", async () => {
    await open();
    const mine = card("2. Tvoj hotel");
    const bars = mine.getAllByRole("img", { name: /od 100$/ });
    expect(bars).toHaveLength(5);
    expect(mine.getByText("45 → 61")).toBeTruthy();
    expect(mine.getByText("▲ +16")).toBeTruthy();
    expect(mine.getByText("▼ −3")).toBeTruthy();
    // Samo moj hotel vodi na detalj pokazatelja.
    expect(mine.getAllByRole("link").some((l) => l.getAttribute("href") === "/pokazatelj/G")).toBe(true);
    expect(card("1. Brižni (robot)").queryAllByRole("link")).toHaveLength(0);
  });

  it("financije sezone u eurima i odluke u brojkama, isti retci za robota i za mene", async () => {
    await open();
    const mine = card("2. Tvoj hotel");
    expect(mine.getAllByText("412.300 €").length).toBeGreaterThan(0);
    expect(mine.getByText("80.000 € → 151.200 €")).toBeTruthy();
    expect(mine.getByText("Tvoje odluke u brojkama")).toBeTruthy();
    expect(mine.getByText("6 od 17 tjedana")).toBeTruthy();
    const robot = card("1. Brižni (robot)");
    expect(robot.getByText("Odluke u brojkama")).toBeTruthy();
    expect(robot.getByText("34 situacije · prosječna ocjena 3,6/5")).toBeTruthy();
  });

  it("rečenica strategije za svakog robota, napomena o robotima jednom", async () => {
    await open();
    expect(card("1. Brižni (robot)").getByText(/^Brižni je ulagao u ljude i goste/)).toBeTruthy();
    expect(card("3. Štediša (robot)").getByText(/^Štediša je cijelu sezonu čuvao novac/)).toBeTruthy();
    expect(
      card("4. Lovac na popunjenost (robot)").getByText(/^Lovac na popunjenost htio je puni hotel/),
    ).toBeTruthy();
    expect(card("2. Tvoj hotel").queryByText("Kako je igrao")).toBeNull();
    expect(screen.getAllByText("Roboti igraju po stalnom pravilu i ne čitaju situacije.")).toHaveLength(1);
  });

  it("bez financija i odluka (drugi student): kartica bez nula, s rečenicom", async () => {
    const body = seasonBody();
    body.hotels = body.hotels.map((h) =>
      h.name === "Štediša"
        ? { ...h, seat: { kind: "human", strategy: null }, finances: null, decisions: null }
        : h,
    );
    await open(body);
    const other = card("3. Štediša (drugi student)");
    expect(other.queryByText("Financije sezone")).toBeNull();
    expect(other.queryByText(/€/)).toBeNull();
    expect(other.getByText("Brojke i odluke drugog studenta vidi samo taj student.")).toBeTruthy();
  });

  it("krivulja: slika s opisom, os s mjesecima, nikad kod tjedna; tablica sa svim tjednima", async () => {
    await open();
    const img = screen.getByRole("img", { name: "Krivulja napretka" });
    const text = img.textContent ?? "";
    for (const m of ["lipanj", "srpanj", "kolovoz", "rujan"]) expect(text).toContain(m);
    expect(text).not.toMatch(/t\d\d/);
    fireEvent.click(screen.getByRole("button", { name: "Prikaži brojke" }));
    const table = screen.getByRole("table", { name: "Napredak po krugovima" });
    expect(within(table).getAllByRole("columnheader")).toHaveLength(19);
  });

  it("mjesečna igra: uvod o četiri mjeseca, os po mjesecima, odluke „od 4 mjeseca“", async () => {
    const api = seasonApi(seasonBody({ weekly: false, mentor: MENTOR })).on("GET", "/v1/games/:game", {
      ...game({ current: null }),
      model: "v2",
    });
    renderApp(api, "/kraj-sezone");
    expect(
      await screen.findByText("Četiri mjeseca su iza tebe. Ovako su prošla sva četiri hotela."),
    ).toBeTruthy();
    const text = screen.getByRole("img", { name: "Krivulja napretka" }).textContent ?? "";
    expect(text).toContain("lipanj");
    expect(text).not.toMatch(/\bjun\b|t\d\d/);
    expect(card("2. Tvoj hotel").getByText("4 od 4 mjeseca")).toBeTruthy();
    // Brižni u mjesečnoj igri bez rečenice o tjednom zapošljavanju.
    expect(card("1. Brižni (robot)").queryByText(/u tjednoj igri/)).toBeNull();
  });

  it("odluke u brojkama u tablici: ista odluka svih hotela u istom retku (od 1024 px)", async () => {
    await open();
    const table = within(screen.getByRole("table", { name: "Odluke u brojkama" }));
    const headers = table.getAllByRole("columnheader").map((h) => h.textContent);
    expect(headers).toEqual([
      "Odluka",
      "1. Brižni (robot)",
      "2. Tvoj hotel",
      "3. Štediša (robot)",
      "4. Lovac na popunjenost (robot)",
    ]);
    const wage = table.getByRole("rowheader", { name: "Plaće" }).closest("tr");
    expect(wage && within(wage).getAllByRole("cell")).toHaveLength(4);
    expect(table.getAllByRole("row")).toHaveLength(11);
  });

  it("pehar uz rečenicu pobjednika, ali ne kad su svi jednaki", async () => {
    await open();
    const line = screen.getByText("Pobijedio je Brižni (robot) s napretkom od +12 bodova.").closest("p");
    expect(line?.querySelector("svg")).not.toBeNull();
    cleanup();
    await open(seasonBody({ progress: [300, 300, 300, 300] }));
    const tied = screen.getByText("Svi hoteli su napredovali jednako i dijele prvo mjesto.").closest("p");
    expect(tied?.querySelector("svg")).toBeNull();
  });

  it("zaglavlje moje kartice ne ponavlja „Tvoj hotel“", async () => {
    await open();
    expect(card("2. Tvoj hotel").queryByText(/^Tvoj hotel ·/)).toBeNull();
    expect(card("1. Brižni (robot)").getByText(/^Robot ·/)).toBeTruthy();
  });

  it("radnje: „Nova igra“ vodi na izbor nove igre, „Natrag na hotel“ na hotel", async () => {
    await open();
    expect(screen.getByRole("link", { name: "Nova igra" }).getAttribute("href")).toBe("/nova");
    expect(screen.getByRole("link", { name: "Natrag na hotel" }).getAttribute("href")).toBe("/");
  });
});

describe("Kraj sezone: mentor", () => {
  it("stigao: tekst s četiri podnaslova", async () => {
    await open();
    for (const part of [
      "Kako je prošla sezona",
      "Što je išlo dobro",
      "Što nije valjalo",
      "Što ponijeti u sljedeću igru",
    ])
      expect(screen.getByRole("heading", { level: 3, name: part })).toBeTruthy();
    expect(screen.getByText("Probaj osnovnu obuku.")).toBeTruthy();
  });

  it("još piše: „Mentor piše…“ i ispitivanje dok ne stigne", async () => {
    let n = 0;
    const api = seasonApi().on("GET", "/v1/games/:game/season", () => {
      n += 1;
      return json(200, seasonBody({ mentor: n >= 3 ? MENTOR : null }));
    });
    renderApp(api, "/kraj-sezone");
    expect(await screen.findByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByRole("heading", { level: 3, name: "Što je išlo dobro" })).toBeTruthy();
    expect(n).toBe(3);
  });

  it("ne javi se: nakon isteka „Mentor se ovaj put nije javio.“ i ispitivanje staje", async () => {
    const api = await open(seasonBody({ mentor: null }));
    expect(screen.getByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByText("Mentor se ovaj put nije javio.")).toBeTruthy();
    // Najviše `maxFeedPolls` (5) ponovnih dohvata uz prvi, pa ništa više.
    await new Promise((r) => setTimeout(r, 400));
    const calls = seasonCalls(api);
    expect(calls).toBeLessThanOrEqual(6);
    await new Promise((r) => setTimeout(r, 200));
    expect(seasonCalls(api)).toBe(calls);
  });
  it("ispitivanje staje i kad dohvati padaju (broje se i neuspjesi)", async () => {
    let n = 0;
    const api = seasonApi().on("GET", "/v1/games/:game/season", () => {
      n += 1;
      return n === 1 ? json(200, seasonBody({ mentor: null })) : problem(500, "internal");
    });
    renderApp(api, "/kraj-sezone");
    expect(await screen.findByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByText("Mentor se ovaj put nije javio.")).toBeTruthy();
    await new Promise((r) => setTimeout(r, 400));
    const calls = seasonCalls(api);
    expect(calls).toBeLessThanOrEqual(6);
    await new Promise((r) => setTimeout(r, 200));
    expect(seasonCalls(api)).toBe(calls);
    // Podaci ostaju na ekranu i kad osvježavanje padne.
    expect(screen.getByRole("list", { name: "Konačni poredak" })).toBeTruthy();
  });
});

describe("Kraj sezone: druga stanja", () => {
  it("409 season_not_over: rečenica i povratak na hotel", async () => {
    renderApp(
      seasonApi().on("GET", "/v1/games/:game/season", () => problem(409, "season_not_over")),
      "/kraj-sezone",
    );
    expect(
      await screen.findByText("Sezona još traje. Rezultati se vide kad se obračuna zadnji tjedan."),
    ).toBeTruthy();
    expect(screen.getByRole("link", { name: "Natrag na hotel" }).getAttribute("href")).toBe("/");
  });

  it("sezona još traje (igra ima trenutni krug): rečenica bez upita sezone", async () => {
    const api = seasonApi().on("GET", "/v1/games/:game", weeklyGame({ week: 9 }));
    renderApp(api, "/kraj-sezone");
    expect(
      await screen.findByText("Sezona još traje. Rezultati se vide kad se obračuna zadnji tjedan."),
    ).toBeTruthy();
    expect(seasonCalls(api)).toBe(0);
  });

  it("409 model_v1 s poslužitelja: rečenica o starom modelu", async () => {
    renderApp(
      seasonApi().on("GET", "/v1/games/:game/season", () => problem(409, "model_v1")),
      "/kraj-sezone",
    );
    expect(await screen.findByText("Ova igra je na starom modelu i nema ekran kraja sezone.")).toBeTruthy();
  });

  it("stara (v1) igra: rečenica bez upita sezone", async () => {
    const api = seasonApi().on("GET", "/v1/games/:game", game({ current: null }));
    renderApp(api, "/kraj-sezone");
    expect(await screen.findByText("Ova igra je na starom modelu i nema ekran kraja sezone.")).toBeTruthy();
    expect(seasonCalls(api)).toBe(0);
  });

  it("greška mreže: rečenica i „Pokušaj ponovno“, koji ponovno dohvati", async () => {
    let fail = true;
    const api = seasonApi().on("GET", "/v1/games/:game/season", () =>
      fail ? problem(500, "internal") : json(200, seasonBody({ mentor: MENTOR })),
    );
    renderApp(api, "/kraj-sezone");
    expect(await screen.findByRole("alert")).toBeTruthy();
    fail = false;
    fireEvent.click(screen.getByRole("button", { name: "Pokušaj ponovno" }));
    expect(await screen.findByRole("list", { name: "Konačni poredak" })).toBeTruthy();
  });
});

// jsdom ne računa raspored, pa se čuvaju klase koje ga određuju; mjerenje na 1024/1280 ostaje UI čuvaru.
describe("Kraj sezone: raspored od 1024 px (ponovni pregled KS-3)", () => {
  function hotelsSection() {
    return screen.getByRole("region", { name: "Kako su igrali" });
  }

  it("kartice robota: dvije u redu od 1024 px, tri tek od 1280 px (N-2)", async () => {
    await open();
    const grid = card("1. Brižni (robot)").getByText("Kako je igrao").closest("section")?.parentElement;
    const classes = grid?.className.split(" ") ?? [];
    // `md:grid-cols-2` vrijedi i na 1024 px; tri stupca tek od `xl:` (1280 px).
    expect(classes).toContain("md:grid-cols-2");
    expect(classes).toContain("xl:grid-cols-3");
    expect(classes).not.toContain("lg:grid-cols-3");
  });

  it("tablica odluka: brojke s jedinicom u jednom retku, rečenice se smiju prelomiti (N-3)", async () => {
    await open();
    const table = within(screen.getByRole("table", { name: "Odluke u brojkama" }));
    const cells = (label: string) => {
      const row = table.getByRole("rowheader", { name: label }).closest("tr");
      if (!row) throw new Error(`nema retka ${label}`);
      return within(row).getAllByRole("cell");
    };
    for (const label of ["Prosječna cijena sobe", "Marketing u sezoni", "Obuka", "Bez odgovora"]) {
      for (const c of cells(label)) expect(c.className.split(" ")).toContain("whitespace-nowrap");
    }
    for (const label of ["Plaće", "Obnova", "Situacije"]) {
      for (const c of cells(label)) expect(c.className.split(" ")).not.toContain("whitespace-nowrap");
    }
  });

  it("svaka tablica od 1024 px je pozicionirana regija za pomicanje (C-1, N-4)", async () => {
    await open();
    const regions = screen
      .getAllByRole("region")
      .filter((r) => r.className.split(" ").includes("overflow-x-auto"));
    expect(regions.length).toBeGreaterThanOrEqual(3);
    for (const r of regions) expect(r.className.split(" ")).toContain("relative");
  });

  it("igra bez robota: „Kako su igrali“ od 1024 px se skriva, bez rečenice o robotima (N-7)", async () => {
    const body = seasonBody();
    body.hotels = body.hotels.map((h) => ({ ...h, seat: { kind: "human", strategy: null } }));
    await open(body);
    expect(hotelsSection().className.split(" ")).toContain("lg:hidden");
    expect(screen.queryByText("Roboti igraju po stalnom pravilu i ne čitaju situacije.")).toBeNull();
    // Ispod 1024 px kartice i dalje stoje.
    expect(card("2. Tvoj hotel").getByText("Financije sezone")).toBeTruthy();
  });

  it("s robotima sekcija ostaje i od 1024 px", async () => {
    await open();
    expect(hotelsSection().className.split(" ")).not.toContain("lg:hidden");
  });
});
