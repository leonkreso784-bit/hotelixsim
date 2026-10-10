import { focusManager } from "@tanstack/react-query";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { json } from "../test/fakeApi";
import { game, ix, myHotel, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const junScore = {
  month: "jun" as const,
  indices: ix(5300, 5200, 5100, 5400, 4800),
  progress_hundredths: 800,
};

describe("Moj hotel", () => {
  it("prije prvog obračuna: indeksi bez promjena, stvarne brojke, ocjena do 10, broj situacija", async () => {
    renderApp(standardApi());
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    expect(screen.getByText("120 soba")).toBeTruthy();
    expect(screen.getByText("Ti, upravitelj")).toBeTruthy();
    const head = screen.getByRole("heading", { name: "Moj hotel" }).parentElement
      ?.parentElement as HTMLElement;
    expect(head.querySelector("svg[aria-hidden='true']:not(.lucide)")).not.toBeNull();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    expect(screen.getAllByRole("img", { name: /od 100$/ })).toHaveLength(5);
    expect(screen.queryByText(/▲|▼/)).toBeNull();
    expect(screen.getByText("42.000 €")).toBeTruthy();
    expect(screen.getByText("58")).toBeTruthy();
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
            sold: 2448,
            available: 3600,
            rooms_revenue_cents: 9_800_000,
            gop_cents: 2_000_000,
            goppar_cents: 55_500,
            mentor: "Usluga je bolja. Prati troškove.",
          },
        }),
      )
      .on("GET", "/v1/games/:game", game({ current: { month: "jul", status: "open", unanswered: 1 } }));
    renderApp(api);
    expect(await screen.findByText("SRPANJ · 2/2")).toBeTruthy();
    expect(await screen.findByText("▲ +3")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(screen.getByText("od početka")).toBeTruthy();
    expect(screen.getByText("68 %")).toBeTruthy();
    expect(screen.getByText("98.000 €")).toBeTruthy();
    expect(screen.getByRole("link", { name: /Rezultat: lipanj/ }).getAttribute("href")).toBe("/mjesec/jun");
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
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const open = game({ current: { month: "jul", status: "open", unanswered: 1 } });
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

  it("„Zatvori mjesec“ traži potvrdu s upozorenjem, šalje jednom i vodi na obračun", async () => {
    const api = standardApi().on("POST", "/v1/games/:game/rounds/:month/close", () =>
      json(202, { month: "jun", status: "closing", unanswered: 0 }),
    );
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    expect(screen.getByText("Zatvaraš lipanj.")).toBeTruthy();
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
        json(200, game({ current: { month: "jun", status: closed ? "closing" : "open", unanswered: 2 } })),
      )
      .on("POST", "/v1/games/:game/rounds/:month/close", async () => {
        closed = true;
        await gate;
        return json(202, { month: "jun", status: "closing", unanswered: 0 });
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

  it("„Odustani“ zatvara potvrdu bez zahtjeva", async () => {
    const api = standardApi();
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    fireEvent.click(screen.getByRole("button", { name: "Odustani" }));
    expect(screen.queryByText("Zatvaraš lipanj.")).toBeNull();
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(0);
  });

  it("dok se mjesec obračunava: nema gumba, ima poveznica na obračun", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/games/:game",
      game({ current: { month: "jun", status: "closing", unanswered: 0 } }),
    );
    renderApp(api);
    expect(await screen.findByText("Obračun mjeseca „lipanj“ je u tijeku.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori mjesec" })).toBeNull();
    expect(screen.getByRole("link", { name: "Pogledaj obračun" }).getAttribute("href")).toBe("/mjesec/jun");
  });

  it("kraj sezone: nema zatvaranja ni plana, ima rezultat srpnja i „Nova igra“", async () => {
    const julScore = {
      month: "jul" as const,
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
    fireEvent.click(screen.getByRole("button", { name: "Nova igra" }));
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  });
});
