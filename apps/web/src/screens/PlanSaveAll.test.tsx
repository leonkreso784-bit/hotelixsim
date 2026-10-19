// Jedan gumb „Spremi plan“ (L1-13b dio 2, ADR-042 t. 4): sprema plan hotela i odluke SVIH odjela jednim PUT-om.
// Gumb stoji na traci koja je vidljiva i na Planu i na stranici odjela (🟨 pretpostavka voditelja); bez izmjena je
// onemogućen kroz aria-disabled (fokus tipkovnice ostaje, kao M-6); zatvaranje kruga i dalje upozori na nespremljeno.
import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HotelBody, PlanBody } from "../api/types";
import { draftKey, readDraft } from "../drafts";
import { navigate } from "../router";
import { json, problem } from "../test/fakeApi";
import { HOTEL_ID, v2Api, v2PlanHotel } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  sessionStorage.clear();
});

const PUT = `/v1/hotels/${HOTEL_ID}/plans/t01`;
const SAVED = /^Plan za .+ je spremljen\.$/;
const PRICE = "Cijena sobe po noći (€)";
const SAVE = { name: "Spremi plan" };

/** Poslužitelj koji pamti plan: PUT ga sprema, a moj hotel ga nakon toga vraća. */
function storingApi(seasonalMax?: number) {
  let stored: PlanBody | null = null;
  const hotel = (): HotelBody => {
    const h = v2PlanHotel();
    if (h.private?.plan) {
      if (stored) h.private.plan.plan = stored;
      if (seasonalMax !== undefined) h.private.plan.limits.seasonal_max = seasonalMax;
    }
    return h;
  };
  return v2Api()
    .on("GET", "/v1/me/hotel", () => json(200, hotel()))
    .on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => {
      stored = body as PlanBody;
      return json(200, stored);
    });
}

async function group(name: string) {
  return await screen.findByRole("group", { name });
}

/** Lobby bar +1 radnik, Restoran +2 radnika, cijena sobe 150 €: sve kao nacrti, bez spremanja. */
async function editEverywhere() {
  const hire = { name: "Zaposli sezonskog radnika" };
  act(() => navigate("/odjeli/lobby_bar"));
  fireEvent.click(within(await group("Lobby bar")).getByRole("button", hire));
  act(() => navigate("/odjeli/restaurant"));
  const rest = await group("Restoran");
  fireEvent.click(within(rest).getByRole("button", hire));
  fireEvent.click(within(rest).getByRole("button", hire));
  act(() => navigate("/plan"));
  fireEvent.change(await screen.findByLabelText(PRICE), { target: { value: "150" } });
}

describe("jedan gumb Spremi plan", () => {
  it("izmjene u dva odjela i hotelu: jedan klik, TOČNO JEDAN PUT sa svim izmjenama, nacrti očišćeni", async () => {
    const api = storingApi();
    renderApp(api, "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    fireEvent.click(screen.getByRole("button", SAVE));
    expect(await screen.findByText(SAVED)).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(1);
    const body = api.posts(PUT)[0]?.body as PlanBody;
    expect(body.price_cents).toBe(15_000);
    expect(body.departments?.lobby_bar).toMatchObject({ seasonal_target: 4, open: true });
    expect(body.departments?.restaurant).toMatchObject({ seasonal_target: 4, open: true });
    // Ostali odjeli netaknuti; ukupni sezonski radnici = zbroj odjela (23 + 1 + 2).
    expect(body.departments?.rooms).toMatchObject({ seasonal_target: 10 });
    expect(body.departments?.admin).toMatchObject({ seasonal_target: 2 });
    expect(body.seasonal_target).toBe(26);
    // Nema nespremljenih nacrta: ni plana ni odjela, ni napomene na ekranu.
    expect(readDraft(draftKey("plan", HOTEL_ID, "t01"))).toEqual({});
    expect(readDraft(draftKey("dept", HOTEL_ID, "t01", "lobby_bar"))).toEqual({});
    expect(readDraft(draftKey("dept", HOTEL_ID, "t01", "restaurant"))).toEqual({});
    await waitFor(() => expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull());
    act(() => navigate("/odjeli/lobby_bar"));
    expect(within(await group("Lobby bar")).getByText("4 sezonska radnika")).toBeTruthy();
    expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull();
  });

  it("dvostruki klik na skupnom spremanju šalje jedan zahtjev", async () => {
    const api = storingApi();
    renderApp(api, "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    const button = screen.getByRole("button", SAVE);
    fireEvent.click(button);
    fireEvent.click(button);
    expect(await screen.findByText(SAVED)).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(1);
    const body = api.posts(PUT)[0]?.body as PlanBody;
    expect(body.departments?.lobby_bar?.seasonal_target).toBe(4);
    expect(body.departments?.restaurant?.seasonal_target).toBe(4);
  });

  it("gumb je vidljiv i na Planu i na stranici odjela; stranica odjela nema zasebno spremanje odjela", async () => {
    renderApp(storingApi(), "/plan");
    await screen.findByLabelText(PRICE);
    expect(screen.getAllByRole("button", SAVE)).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "Spremi odluke odjela" })).toBeNull();
    for (const code of ["restaurant", "lobby_bar"]) {
      act(() => navigate(`/odjeli/${code}`));
      await screen.findByRole("heading", { level: 2, name: "Odluke za ovaj tjedan" });
      expect(screen.getAllByRole("button", SAVE)).toHaveLength(1);
      expect(screen.queryByRole("button", { name: "Spremi odluke odjela" })).toBeNull();
    }
  });

  it("spremanje s ekrana odjela sprema i izmjene hotela s Plana (nacrt cijene)", async () => {
    const api = storingApi();
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText(PRICE), { target: { value: "150" } });
    act(() => navigate("/odjeli/lobby_bar"));
    const bar = await group("Lobby bar");
    fireEvent.click(within(bar).getByRole("button", { name: "Zaposli sezonskog radnika" }));
    fireEvent.click(screen.getByRole("button", SAVE));
    expect(await screen.findByText(SAVED)).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(1);
    const body = api.posts(PUT)[0]?.body as PlanBody;
    expect(body.price_cents).toBe(15_000);
    expect(body.departments?.lobby_bar?.seasonal_target).toBe(4);
    expect(readDraft(draftKey("plan", HOTEL_ID, "t01"))).toEqual({});
  });

  it("bez izmjena je gumb onemogućen (aria-disabled, fokus ostaje) i klik ne šalje ništa; izmjena ga omogući", async () => {
    const api = storingApi();
    renderApp(api, "/plan");
    const price = (await screen.findByLabelText(PRICE)) as HTMLInputElement;
    const original = price.value;
    const button = screen.getByRole("button", SAVE);
    expect(button.getAttribute("aria-disabled")).toBe("true");
    expect(button.hasAttribute("disabled")).toBe(false);
    fireEvent.click(button);
    expect(api.calls.filter((c) => c.method === "PUT")).toHaveLength(0);
    fireEvent.change(price, { target: { value: "150" } });
    expect(screen.getByRole("button", SAVE).getAttribute("aria-disabled")).not.toBe("true");
    // Vraćena ista vrijednost: opet nema izmjena, i to vrijedi na stranici odjela.
    fireEvent.change(price, { target: { value: original } });
    act(() => navigate("/odjeli/lobby_bar"));
    await group("Lobby bar");
    expect(screen.getByRole("button", SAVE).getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(screen.getByRole("button", SAVE));
    expect(api.calls.filter((c) => c.method === "PUT")).toHaveLength(0);
  });

  it("neuspjeh spremanja: nacrti ostaju, poruka o grešci, napomena o nespremljenom ostaje", async () => {
    const api = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () =>
      problem(422, "department_plan_invalid"),
    );
    renderApp(api, "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    fireEvent.click(screen.getByRole("button", SAVE));
    expect((await screen.findByRole("alert")).textContent).toContain("Postavke odjela nisu dopuštene");
    expect(api.posts(PUT)).toHaveLength(1);
    // Zahtjev je bio objedinjen (nije samo hotel), pa nijedan nacrt ne smije nestati.
    const sent = api.posts(PUT)[0]?.body as PlanBody;
    expect(sent.price_cents).toBe(15_000);
    expect(sent.departments?.lobby_bar?.seasonal_target).toBe(4);
    expect(sent.departments?.restaurant?.seasonal_target).toBe(4);
    expect(readDraft(draftKey("plan", HOTEL_ID, "t01"))).toMatchObject({ price: "150" });
    expect(readDraft(draftKey("dept", HOTEL_ID, "t01", "lobby_bar"))).toMatchObject({ seasonal_target: 4 });
    expect(readDraft(draftKey("dept", HOTEL_ID, "t01", "restaurant"))).toMatchObject({ seasonal_target: 4 });
    expect(screen.getByText("Izmjene još nisu spremljene.")).toBeTruthy();
    expect((screen.getByLabelText(PRICE) as HTMLInputElement).value).toBe("150");
    expect(screen.getByRole("button", SAVE).getAttribute("aria-disabled")).not.toBe("true");
    expect(screen.queryByText(SAVED)).toBeNull();
  });

  it("zbroj odjela iznad najvećeg broja hotela (svaki odjel zasebno stane): poruka, bez PUT-a, nacrti ostaju", async () => {
    const api = storingApi(25);
    renderApp(api, "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    fireEvent.click(screen.getByRole("button", SAVE));
    expect(await screen.findByText(/previše sezonskih radnika/)).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(0);
    expect(readDraft(draftKey("dept", HOTEL_ID, "t01", "restaurant"))).toMatchObject({ seasonal_target: 4 });
  });
});

describe("zatvaranje kruga i nespremljene izmjene", () => {
  const closeButton = { name: "Zatvori tjedan" };
  const warning =
    "Plan ili odluke odjela imaju nespremljene izmjene: tjedan se obračunava bez njih. Spremi ih prije zatvaranja.";

  it("s nespremljenim izmjenama (odjeli + hotel) potvrda i dalje upozorava i vodi u plan", async () => {
    renderApp(storingApi(), "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    act(() => navigate("/"));
    fireEvent.click(await screen.findByRole("button", closeButton));
    expect(screen.getByText(warning)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Otvori plan" }).getAttribute("href")).toBe("/plan");
  });

  it("nakon „Spremi plan“ upozorenja više nema", async () => {
    renderApp(storingApi(), "/plan");
    await screen.findByLabelText(PRICE);
    await editEverywhere();
    fireEvent.click(screen.getByRole("button", SAVE));
    expect(await screen.findByText(SAVED)).toBeTruthy();
    act(() => navigate("/"));
    fireEvent.click(await screen.findByRole("button", closeButton));
    expect(screen.queryByText(warning)).toBeNull();
    expect(screen.queryByRole("link", { name: "Otvori plan" })).toBeNull();
  });
});
