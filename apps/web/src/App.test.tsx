import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FakeApi, json, problem } from "./test/fakeApi";
import { game, myHotel } from "./test/fixtures";
import { renderApp } from "./test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("gost", () => {
  it("bez igre vidi početak, a „Započni igru“ stvori igru jednom i otvori hotel", async () => {
    let started = false;
    let release: () => void = () => {};
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const api = new FakeApi()
      .on("GET", "/v1/me/hotel", () => (started ? json(200, myHotel()) : problem(401, "guest_required")))
      .on("GET", "/v1/games/:game", game())
      .on("POST", "/v1/demo", async () => {
        await gate;
        started = true;
        return json(201, { game_id: "g1", hotel_id: "h1" });
      });
    renderApp(api, "/demo");
    const button = await screen.findByRole("button", { name: "Započni igru" });
    fireEvent.click(button);
    fireEvent.click(button);
    await waitFor(() => expect((button as HTMLButtonElement).disabled).toBe(true));
    release();
    expect(await screen.findByRole("heading", { name: /Moj hotel/ })).toBeTruthy();
    expect(api.posts("/v1/demo")).toHaveLength(1);
    expect(window.location.pathname).toBe("/");
  });

  it("greška pri stvaranju igre (503) je hrvatska rečenica", async () => {
    const api = new FakeApi()
      .on("GET", "/v1/me/hotel", () => problem(404, "not_in_game"))
      .on("POST", "/v1/demo", () => problem(503, "storage_unavailable"));
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Započni igru" }));
    expect(await screen.findByText("Baza trenutno nije dostupna. Pokušaj ponovno.")).toBeTruthy();
  });

  it("poslužitelj nedostupan → poruka i „Pokušaj ponovno“", async () => {
    const api = new FakeApi();
    api.fetch = async () => {
      throw new TypeError("Failed to fetch");
    };
    renderApp(api);
    expect(
      await screen.findByText("Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu."),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Pokušaj ponovno" })).toBeTruthy();
  });
});

describe("okvir", () => {
  it("zaglavlje pokazuje oznaku mjeseca, a donja traka tri kartice", async () => {
    const api = new FakeApi().on("GET", "/v1/me/hotel", myHotel()).on("GET", "/v1/games/:game", game());
    renderApp(api);
    expect(await screen.findByText("LIPANJ · 1/2")).toBeTruthy();
    const nav = screen.getByRole("navigation", { name: "Glavni izbornik" });
    expect(nav.textContent).toContain("Hotel");
    expect(nav.textContent).toContain("Situacije");
    expect(nav.textContent).toContain("Tijek");
  });

  it("nepoznata putanja daje „Ova stranica ne postoji.“", async () => {
    const api = new FakeApi().on("GET", "/v1/me/hotel", myHotel()).on("GET", "/v1/games/:game", game());
    renderApp(api, "/nesto");
    expect(await screen.findByText("Ova stranica ne postoji.")).toBeTruthy();
  });
});
