import { focusManager } from "@tanstack/react-query";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { draftKey, writeDraft } from "../drafts";
import { keepRoundTogether } from "../format";
import { json, problem } from "../test/fakeApi";
import {
  GAME_ID,
  game,
  HOTEL_ID,
  ix,
  myHotel,
  resolvedWeekApi,
  roundInfo,
  seasonApi,
  standardApi,
  v2Api,
  weekInfo,
  weeklyGame,
  weeklyHotel,
} from "../test/fixtures";
import { joined } from "../test/joined";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const junScore = {
  month: "jun" as const,
  round: "jun",
  indices: ix(5300, 5200, 5100, 5400, 4800),
  progress_hundredths: 800,
};

describe("Moj hotel", () => {
  it("prije prvog obračuna: indeksi bez promjena, stvarne brojke, ocjena do 10, broj situacija", async () => {
    renderApp(standardApi());
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    expect(screen.getByText("120 soba")).toBeTruthy();
    expect(screen.getByText("Ti vodiš ovaj hotel")).toBeTruthy();
    const head = screen.getByRole("heading", { name: "Moj hotel" }).parentElement
      ?.parentElement as HTMLElement;
    expect(head.querySelector("svg[aria-hidden='true']:not(.lucide)")).not.toBeNull();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    expect(screen.getAllByRole("img", { name: /od 100$/ })).toHaveLength(5);
    expect(screen.queryByText(/▲|▼/)).toBeNull();
    expect(screen.getByText("42.000 €")).toBeTruthy();
    expect(screen.getByText("58 osoba")).toBeTruthy();
    expect(screen.getByText("8,4/10")).toBeTruthy();
    const sit = screen.getByRole("link", { name: /Situacije mjeseca/ });
    expect(await within(sit).findByText("2")).toBeTruthy();
    expect(screen.getByRole("link", { name: /Plan mjeseca/ }).getAttribute("href")).toBe("/plan");
  });

  it("nakon lipnja: promjene od početka, popunjenost i prihod lipnja, poveznica na rezultat", async () => {
    const api = standardApi()
      .on(
        "GET",
        "/v1/me/hotel",
        myHotel({
          months: [junScore],
          planMonth: "jul",
          lastMonth: {
            month: "jun",
            round: roundInfo("jun"),
            sold: 2448,
            // 2 sobe izvan funkcije: popunjenost se dijeli sa SVIM sobama (6800 bp s poslužitelja), ne s 3 540.
            available: 3540,
            rooms_revenue_cents: 9_800_000,
            gop_cents: 2_000_000,
            goppar_cents: 55_500,
            mentor: "Usluga je bolja. Prati troškove.",
            occupancy_bp: 6800,
            adr_cents: 4003,
            revpar_cents: 2722,
          },
        }),
      )
      .on(
        "GET",
        "/v1/games/:game",
        game({ current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 } }),
      );
    renderApp(api);
    expect(await screen.findByText("SRPANJ · 2/2")).toBeTruthy();
    expect(await screen.findByText("▲ +3")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(screen.getByText("od početka")).toBeTruthy();
    expect(screen.getByText("68 %")).toBeTruthy();
    expect(screen.queryByText("69 %")).toBeNull();
    expect(screen.getByText("98.000 €")).toBeTruthy();
    expect(screen.getByRole("link", { name: /Rezultat: lipanj/ }).getAttribute("href")).toBe("/mjesec/jun");
  });

  it("svaki indeks i svaka kartica vode na detalj pokazatelja", async () => {
    renderApp(standardApi());
    await screen.findByRole("heading", { name: "Moj hotel" });
    const hrefs = [...document.querySelectorAll('a[href^="/pokazatelj"]')].map((a) => a.getAttribute("href"));
    expect(hrefs).toEqual([
      "/pokazatelj/G",
      "/pokazatelj/R",
      "/pokazatelj/Z",
      "/pokazatelj/O",
      "/pokazatelj/F",
      "/pokazatelj/F/sold",
      "/pokazatelj/F/revenue",
      "/pokazatelj/F/cash",
      "/pokazatelj/Z/staff",
      "/pokazatelj/R/rating",
      "/pokazatelji",
    ]);
    expect(screen.getByRole("link", { name: /Prihod od soba/ }).getAttribute("href")).toBe(
      "/pokazatelj/F/revenue",
    );
    expect(screen.getByRole("link", { name: /^Ocjena na portalima/ }).getAttribute("href")).toBe(
      "/pokazatelj/R/rating",
    );
    expect(screen.getByRole("link", { name: /^Zaposlenici/ }).getAttribute("href")).toBe("/pokazatelj/Z");
  });

  it("klik na karticu Zaposleni otvara Zaposlenike s istaknutom varijablom", async () => {
    renderApp(standardApi());
    fireEvent.click(await screen.findByRole("link", { name: /^Zaposleni(?!c)/ }));
    expect(await screen.findByRole("heading", { name: "Zaposlenici" })).toBeTruthy();
    expect(window.location.pathname).toBe("/pokazatelj/Z/staff");
  });

  it("kartice: iznos se ne lomi, a bilješka se prelama umjesto da se reže", async () => {
    renderApp(standardApi());
    expect((await screen.findByText("42.000 €")).className).toContain("whitespace-nowrap");
    const note = screen.getAllByText("nakon prvog mjeseca")[0] as HTMLElement;
    expect(note.className).not.toContain("truncate");
    expect(note.className).toContain("line-clamp-2");
  });

  it("mjesec se obračuna u pozadini: Moj hotel sam osvježi indekse bez ručnog osvježavanja", async () => {
    let gameCalls = 0;
    const closing = game({
      current: { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 },
    });
    const open = game({ current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 } });
    const api = standardApi()
      .on("GET", "/v1/games/:game", () => {
        gameCalls += 1;
        return json(200, gameCalls < 3 ? closing : open);
      })
      .on("GET", "/v1/me/hotel", () =>
        json(200, gameCalls < 3 ? myHotel() : myHotel({ months: [junScore], planMonth: "jul" })),
      );
    renderApp(api);
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    expect(screen.queryByText("od početka")).toBeNull();
    expect(await screen.findByText("▲ +3")).toBeTruthy();
    expect(screen.getByText("od početka")).toBeTruthy();
  });

  it("obračun završi između dva dohvata (nikad viđen „closing“): Moj hotel ipak dohvati nove indekse", async () => {
    let gameCalls = 0;
    let meCalls = 0;
    const api = standardApi()
      .on("GET", "/v1/games/:game", () => {
        gameCalls += 1;
        return json(
          200,
          gameCalls < 2
            ? game({ current: { month: "jun", round: roundInfo("jun"), status: "open", unanswered: 0 } })
            : game({ current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 } }),
        );
      })
      // Drugi dohvat „me“ ide istodobno s igrom i vidi stanje od prije obračuna; tek treći vidi novo.
      .on("GET", "/v1/me/hotel", () => {
        meCalls += 1;
        return json(200, meCalls < 3 ? myHotel() : myHotel({ months: [junScore], planMonth: "jul" }));
      });
    renderApp(api);
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    focusManager.setFocused(false);
    focusManager.setFocused(true);
    expect(await screen.findByText("▲ +3")).toBeTruthy();
    expect(meCalls).toBeGreaterThanOrEqual(3);
  });

  it("neuspjeli pozadinski dohvat igre ne zamjenjuje Moj hotel porukom o grešci", async () => {
    let gameCalls = 0;
    const api = standardApi().on("GET", "/v1/games/:game", () => {
      gameCalls += 1;
      return gameCalls < 2
        ? json(
            200,
            game({ current: { month: "jun", round: roundInfo("jun"), status: "open", unanswered: 2 } }),
          )
        : problem(503, "storage_unavailable");
    });
    renderApp(api);
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    focusManager.setFocused(false);
    focusManager.setFocused(true);
    await waitFor(() => expect(gameCalls).toBeGreaterThanOrEqual(2));
    // Pričekaj da upit uđe u stanje greške, pa provjeri da sadržaj ostaje.
    await new Promise((r) => requestAnimationFrame(() => r(null)));
    expect(screen.getByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("„Zatvori mjesec“ traži potvrdu s upozorenjem, šalje jednom i vodi na obračun", async () => {
    const api = standardApi().on("POST", "/v1/games/:game/rounds/:month/close", () =>
      json(202, { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 }),
    );
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    // Kliknuti gumb nestaje: fokus ide na naslov potvrde, ne na <body> (I6).
    const title = screen.getByRole("heading", { name: "Zatvaraš lipanj." });
    expect(document.activeElement).toBe(title);
    expect(
      await screen.findByText("Bez odgovora: 2. Takve situacije dobivaju najnižu ocjenu (1)."),
    ).toBeTruthy();
    const confirm = screen.getByRole("button", { name: "Zatvori lipanj" });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    await waitFor(() => expect(window.location.pathname).toBe("/mjesec/jun"));
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(1);
  });

  it("navigacija na obračun preživi odmontiranje potvrde (mjesec postao „closing“ prije odgovora na POST)", async () => {
    let closed = false;
    let release: () => void = () => {};
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const api = standardApi()
      .on("GET", "/v1/games/:game", () =>
        json(
          200,
          game({
            current: {
              month: "jun",
              round: roundInfo("jun"),
              status: closed ? "closing" : "open",
              unanswered: 2,
            },
          }),
        ),
      )
      .on("POST", "/v1/games/:game/rounds/:month/close", async () => {
        closed = true;
        await gate;
        return json(202, { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 });
      });
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    const confirm = await screen.findByRole("button", { name: "Zatvori lipanj" });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    // Ekran se osvježi dok POST još čeka: potvrda se odmontira, tek onda stiže odgovor.
    await waitFor(() => expect(closed).toBe(true));
    focusManager.setFocused(false);
    focusManager.setFocused(true);
    expect(await screen.findByText("Obračun mjeseca „lipanj“ je u tijeku.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori lipanj" })).toBeNull();
    release();
    await waitFor(() => expect(window.location.pathname).toBe("/mjesec/jun"));
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(1);
  });

  it("„Odustani“ zatvara potvrdu bez zahtjeva i vraća fokus na „Zatvori mjesec“", async () => {
    const api = standardApi();
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    fireEvent.click(screen.getByRole("button", { name: "Odustani" }));
    expect(screen.queryByText("Zatvaraš lipanj.")).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Zatvori mjesec" }));
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(0);
  });

  it("dok se mjesec obračunava: nema gumba, ima poveznica na obračun", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/games/:game",
      game({ current: { month: "jun", round: roundInfo("jun"), status: "closing", unanswered: 0 } }),
    );
    renderApp(api);
    expect(await screen.findByText("Obračun mjeseca „lipanj“ je u tijeku.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori mjesec" })).toBeNull();
    expect(screen.getByRole("link", { name: "Pogledaj obračun" }).getAttribute("href")).toBe("/mjesec/jun");
  });

  it("kraj sezone: nema zatvaranja ni plana, ima rezultat srpnja i „Nova igra“", async () => {
    const julScore = {
      month: "jul" as const,
      round: "jul",
      indices: ix(5600, 5200, 5000, 5400, 5000),
      progress_hundredths: 1100,
    };
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [junScore, julScore], planMonth: null }))
      .on("GET", "/v1/games/:game", game({ current: null }))
      .on("POST", "/v1/demo", () => json(201, { game_id: "g2", hotel_id: "h9" }));
    renderApp(api);
    expect(await screen.findByText("Sezona je završena.")).toBeTruthy();
    expect(screen.getByText("SEZONA ZAVRŠENA")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori mjesec" })).toBeNull();
    expect(screen.queryByRole("link", { name: /Plan mjeseca/ })).toBeNull();
    expect(screen.getByRole("link", { name: /Rezultat: srpanj/ })).toBeTruthy();
    // „Nova igra“ je poveznica na početni ekran s izborom; zahtjev ne šalje (v1 se ne pokreće tiho).
    // Stara (v1) igra nema ekran kraja sezone.
    expect(screen.queryByRole("link", { name: "Rezultati sezone" })).toBeNull();
    const link = screen.getByRole("link", { name: "Nova igra" });
    expect(link.getAttribute("href")).toBe("/nova");
    fireEvent.click(link);
    expect(await screen.findByRole("button", { name: "Započni igru" })).toBeTruthy();
    expect(api.posts("/v1/demo")).toHaveLength(0);
  });
});

describe("Moj hotel: krugovi", () => {
  it("tjedna igra: čip TJEDAN 1 · 1/17 i gumb Zatvori tjedan", async () => {
    renderApp(standardApi().on("GET", "/v1/games/:game", weeklyGame()), "/");
    expect(await screen.findByText("TJEDAN 1 · 1/17")).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Zatvori tjedan" })).toBeTruthy();
  });
  it("v1 igra izgleda kao prije: LIPANJ · 1/2 i Zatvori mjesec", async () => {
    renderApp(standardApi(), "/");
    expect(await screen.findByText("LIPANJ · 1/2")).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Zatvori mjesec" })).toBeTruthy();
  });
});

describe("Moj hotel: tjedna igra", () => {
  it("zatvaranje tjedna: potvrda s datumima, jedan zahtjev za t01, prelazak na /krug/t01", async () => {
    const api = v2Api().on("POST", "/v1/games/:game/rounds/:month/close", () =>
      json(202, { month: "jun", round: weekInfo(1), status: "closing", unanswered: 0 }),
    );
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori tjedan" }));
    expect(
      screen.getByRole("heading", { name: keepRoundTogether("Zatvaraš tjedan 1 (1. 6. – 7. 6.).") }),
    ).toBeTruthy();
    expect(
      screen.getByText("Tjedan se obračunava za sve hotele odjednom i nakon toga se više ne mijenja."),
    ).toBeTruthy();
    const confirm = screen.getByRole("button", {
      name: keepRoundTogether("Zatvori tjedan 1 (1. 6. – 7. 6.)"),
    });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    await waitFor(() => expect(window.location.pathname).toBe("/krug/t01"));
    expect(api.posts(`/v1/games/${GAME_ID}/rounds/t01/close`)).toHaveLength(1);
  });

  it("nespremljene izmjene: potvrda kaže da ulaze bez njih i vodi u plan (N-I2)", async () => {
    renderApp(v2Api());
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori tjedan" }));
    expect(screen.queryByText(/nespremljene izmjene/)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Odustani" }));
    writeDraft(draftKey("dept", HOTEL_ID, "t01", "restaurant"), { seasonal_target: 3 });
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori tjedan" }));
    expect(
      screen.getByText(
        "Plan ili odluke odjela imaju nespremljene izmjene: tjedan se obračunava bez njih. Spremi ih prije zatvaranja.",
      ),
    ).toBeTruthy();
    expect(screen.getByRole("link", { name: "Otvori plan" }).getAttribute("href")).toBe("/plan");
  });

  it("izbornik kaže tjedan; rezultat zadnjeg tjedna vodi na /krug/t01", async () => {
    renderApp(resolvedWeekApi());
    expect(await screen.findByText("TJEDAN 2 · 2/17")).toBeTruthy();
    expect(screen.getByText("Plan tjedna")).toBeTruthy();
    expect(screen.getByText("Situacije tjedna")).toBeTruthy();
    const link = screen.getByRole("link", {
      name: /Rezultat: tjedan\u00a01 \(1\.\u00a06\.\u00a0–\u2060\u00a07\.\u00a06\.\)/,
    });
    expect(link.getAttribute("href")).toBe("/krug/t01");
  });

  it("napomena uz popunjenost ne ponavlja riječ i ne lomi se usred datuma", async () => {
    const week1 = {
      month: "jun",
      round: weekInfo(1),
      sold: 100,
      available: 200,
      rooms_revenue_cents: 1_000_000,
      gop_cents: 100_000,
      goppar_cents: 500,
      mentor: "",
      occupancy_bp: 5000,
      adr_cents: 10_000,
      revpar_cents: 5000,
    } as const;
    renderApp(v2Api().on("GET", "/v1/me/hotel", weeklyHotel({ week: 2, lastMonth: week1 })));
    const notes = await screen.findAllByText(/^zadnji: tjedan/);
    expect(notes).toHaveLength(2);
    expect(notes[0]?.textContent).toBe(`zadnji: ${keepRoundTogether("tjedan 1 (1. 6. – 7. 6.)")}`);
    expect(screen.queryByText(/zadnji tjedan/)).toBeNull();
  });

  it("obračun tjedna u tijeku: rečenica s datumima i poveznica na krug", async () => {
    const api = v2Api().on("GET", "/v1/games/:game", weeklyGame({ status: "closing" }));
    renderApp(api);
    expect(await screen.findByText(joined("Obračun je u tijeku: tjedan 1 (1. 6. – 7. 6.)."))).toBeTruthy();
    expect(screen.getByRole("link", { name: "Pogledaj obračun" }).getAttribute("href")).toBe("/krug/t01");
  });
});

describe("Hotel nakon v2 sezone", () => {
  it("„Rezultati sezone“ vodi na kraj sezone, iznad „Nova igra“", async () => {
    renderApp(seasonApi());
    expect(await screen.findByText("Sezona je završena.")).toBeTruthy();
    const results = screen.getByRole("link", { name: "Rezultati sezone" });
    expect(results.getAttribute("href")).toBe("/kraj-sezone");
    const links = screen.getAllByRole("link").map((l) => l.textContent);
    expect(links.indexOf("Rezultati sezone")).toBeLessThan(links.indexOf("Nova igra"));
  });
});
