import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FakeApi, json, problem } from "./test/fakeApi";
import { game, myHotel, startApi } from "./test/fixtures";
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

  it("početni ekran kaže ime igre „HotelixSim“ iznad naslova (I12)", async () => {
    const api = new FakeApi().on("GET", "/v1/me/hotel", () => problem(401, "guest_required"));
    renderApp(api, "/");
    const h1 = await screen.findByRole("heading", { level: 1 });
    // Riječ je složena od „Hotelix“ i plavog „Sim“; traži se najuži element s cijelim imenom.
    const name = screen.getByText(
      (_, el) =>
        el?.tagName === "SPAN" &&
        el.textContent === "HotelixSim" &&
        !Array.from(el.children).some((c) => c.textContent === "HotelixSim"),
    );
    expect(name.compareDocumentPosition(h1) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
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

describe("početni ekran: izbor tjedne ili mjesečne igre", () => {
  it("student odmah vidi izbor vrste igre, tjedna je zadana, a zahtjev još nije poslan", async () => {
    const api = startApi();
    renderApp(api, "/");
    const week = (await screen.findByRole("radio", { name: "Tjedna igra" })) as HTMLInputElement;
    const month = screen.getByRole("radio", { name: "Mjesečna igra" }) as HTMLInputElement;
    expect(week.checked).toBe(true);
    expect(month.checked).toBe(false);
    // Radio grupa ima legendu, a kratki opisi stoje uz svaki izbor.
    expect(screen.getByRole("group", { name: "Kako želiš igrati?" })).toBeTruthy();
    expect(week.getAttribute("aria-describedby")).toBeTruthy();
    expect(screen.getByText(/17 kraćih krugova/)).toBeTruthy();
    expect(screen.getByText(/4 kruga/)).toBeTruthy();
    expect(api.posts("/v1/demo")).toHaveLength(0);
  });

  it("zadano šalje novi model po tjednima, jedan zahtjev", async () => {
    const api = startApi();
    renderApp(api, "/");
    const start = await screen.findByRole("button", { name: "Započni igru" });
    fireEvent.click(start);
    fireEvent.click(start);
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
    expect(api.posts("/v1/demo")[0]?.body).toEqual({ model: "v2", round_kind: "week" });
  });

  it("mjesečna igra šalje novi model s round_kind month", async () => {
    const api = startApi();
    renderApp(api, "/");
    fireEvent.click(await screen.findByRole("radio", { name: "Mjesečna igra" }));
    fireEvent.click(screen.getByRole("button", { name: "Započni igru" }));
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
    expect(api.posts("/v1/demo")[0]?.body).toEqual({ model: "v2", round_kind: "month" });
  });

  it("stari model postoji samo pod „Za profesora“ i šalje prazno tijelo", async () => {
    const api = startApi();
    renderApp(api, "/");
    await screen.findByRole("radio", { name: "Tjedna igra" });
    // Stari model nije među glavnim izborima; nalazi se unutar sklopivog dijela.
    const pro = screen.getByText("Za profesora").closest("details") as HTMLElement;
    expect(pro).toBeTruthy();
    const legacy = screen.getByRole("checkbox", { name: "Stari model (usporedba)" });
    expect(pro.contains(legacy)).toBe(true);
    fireEvent.click(screen.getByText("Za profesora"));
    fireEvent.click(legacy);
    expect(screen.queryByRole("radio", { name: "Tjedna igra" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Započni igru" }));
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
    expect(api.posts("/v1/demo")[0]?.body).toBeUndefined();
  });
});

describe("Nova igra iz igre", () => {
  function seasonOverApi() {
    return startApi()
      .on("GET", "/v1/me/hotel", myHotel({ planMonth: null }))
      .on("GET", "/v1/games/:game", game({ current: null }));
  }

  it("„Nova igra“ ne šalje zahtjev, pokazuje izbor vrste igre, a „Natrag na igru“ vraća na igru bez zahtjeva", async () => {
    const api = seasonOverApi();
    renderApp(api, "/");
    fireEvent.click(await screen.findByRole("link", { name: "Nova igra" }));
    expect(await screen.findByRole("heading", { name: "Nova igra" })).toBeTruthy();
    expect(screen.getByRole("radio", { name: "Tjedna igra" })).toBeTruthy();
    expect(screen.getByRole("radio", { name: "Mjesečna igra" })).toBeTruthy();
    fireEvent.click(screen.getByRole("link", { name: "Natrag na igru" }));
    expect(await screen.findByText("Sezona je završena.")).toBeTruthy();
    expect(window.location.pathname).toBe("/");
    expect(api.posts("/v1/demo")).toHaveLength(0);
  });

  it("tjedna igra iz „Nova igra“ šalje novi model kao početni ekran i otvara novu igru", async () => {
    const api = seasonOverApi();
    renderApp(api, "/nova");
    await screen.findByRole("radio", { name: "Tjedna igra" });
    fireEvent.click(screen.getByRole("button", { name: "Započni igru" }));
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
    expect(api.posts("/v1/demo")[0]?.body).toEqual({ model: "v2", round_kind: "week" });
    await waitFor(() => expect(window.location.pathname).toBe("/"));
  });
});
