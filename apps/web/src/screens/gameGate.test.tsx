// Tjedni tekstovi na ekranima koji ovise o igri (m2, m5, m7): dok igra ne stigne ne smije se pokazati mjesečni tekst ni
// sirovi kod kruga, greške u tjednoj igri govore o tjednu, a greška popisa odjela ne krije detalj odjela.
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../test/fakeApi";
import { s1, s3, situations, v2Api, v2PlApi, weeklyGame } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** Igra stiže tek kad se pozove `release`; do tada GET igre visi. */
function gatedGame<T extends ReturnType<typeof v2Api>>(api: T) {
  let release: () => void = () => {};
  const gate = new Promise<void>((r) => {
    release = r;
  });
  api.on("GET", "/v1/games/:game", async () => {
    await gate;
    return json(200, weeklyGame());
  });
  return { api, release };
}

describe("dok igra ne stigne (m7)", () => {
  it("Račun dobiti: samo učitavanje, bez sirovog koda kruga", async () => {
    const { api, release } = gatedGame(v2PlApi());
    renderApp(api, "/racun-dobiti/t05");
    expect(await screen.findByText("Učitavam…")).toBeTruthy();
    expect(screen.queryByText(/t05/)).toBeNull();
    release();
    expect(await screen.findByRole("heading", { level: 1, name: /Račun dobiti — Tjedan/ })).toBeTruthy();
  });

  it("Plan: samo učitavanje, naslov tek s igrom", async () => {
    const { api, release } = gatedGame(v2Api());
    renderApp(api, "/plan");
    expect(await screen.findByText("Učitavam…")).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Plan mjeseca" })).toBeNull();
    release();
    expect(await screen.findByRole("heading", { level: 1, name: "Plan tjedna" })).toBeTruthy();
  });

  it("Situacija: samo učitavanje, tekstovi tek s igrom", async () => {
    const { api, release } = gatedGame(v2Api());
    renderApp(api, "/situacije/S9");
    expect(await screen.findByText("Učitavam…")).toBeTruthy();
    release();
    expect(await screen.findByText("Ta situacija nije u ovom tjednu.")).toBeTruthy();
  });

  it("Odjeli: popis i detalj čekaju igru, bez mjesečnih tekstova", async () => {
    const { api, release } = gatedGame(v2Api());
    renderApp(api, "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjel" })).toBeTruthy();
    expect(screen.queryByText("Po mjesecima")).toBeNull();
    release();
    expect(await screen.findByRole("heading", { level: 2, name: "Po tjednima" })).toBeTruthy();
  });
});

describe("tjedna igra: greške govore o tjednu (m2)", () => {
  it("izbor odgovora u zatvorenom tjednu: „Tjedan nije otvoren“", async () => {
    const api = v2Api()
      .on("GET", "/v1/hotels/:hotel/situations", situations([s3(), s1()]))
      .on("POST", "/v1/hotels/:hotel/situations/:code/answer", () => problem(409, "round_not_open"));
    renderApp(api, "/situacije/S3");
    fireEvent.click(
      (await screen.findAllByRole("button", { name: "Izaberi ovaj odgovor" }))[0] as HTMLElement,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Potvrdi izbor" }));
    expect(await screen.findByText("Tjedan nije otvoren — sad se ne može mijenjati.")).toBeTruthy();
  });
});

describe("popis odjela s greškom (m5)", () => {
  it("greška je samo u stupcu popisa, detalj odjela se i dalje vidi", async () => {
    const api = v2Api().on("GET", "/v1/hotels/:hotel/departments", () => problem(500, "internal"));
    renderApp(api, "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Pokušaj ponovno" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Novac" })).toBeTruthy();
  });
});

describe("igra se ne učita: ekran ima izlaz", () => {
  const failingGame = () => v2Api().on("GET", "/v1/games/:game", () => problem(500, "internal"));

  it("Račun dobiti: rečenica i „Pokušaj ponovno“ koji ponovno traži igru", async () => {
    const api = v2PlApi().on("GET", "/v1/games/:game", () => problem(500, "internal"));
    renderApp(api, "/racun-dobiti");
    const retry = await screen.findByRole("button", { name: "Pokušaj ponovno" });
    expect(screen.getByText("Greška na poslužitelju. Pokušaj ponovno.")).toBeTruthy();
    const gameCalls = () => api.calls.filter((c) => c.path === "/v1/games/g1").length;
    const before = gameCalls();
    fireEvent.click(retry);
    await waitFor(() => expect(gameCalls()).toBeGreaterThan(before));
  });

  it("Plan, Situacija i Odjeli nude „Pokušaj ponovno“", async () => {
    for (const path of ["/plan", "/situacije/S1", "/odjeli", "/odjeli/restaurant"]) {
      renderApp(failingGame(), path);
      expect(await screen.findByRole("button", { name: "Pokušaj ponovno" }), path).toBeTruthy();
      cleanup();
    }
  });
});
