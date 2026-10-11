import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../../test/fakeApi";
import { HOTEL_ID, standardApi, v2Api } from "../../test/fixtures";
import { renderApp } from "../../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** v2 igra u kojoj PUT plana vraća poslano tijelo. */
const echoApi = () =>
  v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));

describe("Plan po odjelima", () => {
  it("v2: dio Odjeli s retkom za svaki odjel koji se planira", async () => {
    renderApp(echoApi(), "/plan");
    expect(await screen.findByRole("heading", { level: 2, name: "Odjeli" })).toBeTruthy();
    expect(await screen.findByRole("group", { name: "Restoran" })).toBeTruthy();
    expect(screen.queryByRole("group", { name: "Uprava" })).toBeNull();
  });

  it("službe nemaju grupu u planu", async () => {
    renderApp(echoApi(), "/plan");
    await screen.findByRole("group", { name: "Restoran" });
    for (const name of ["Uprava", "Prodaja i marketing", "Održavanje", "Energija i voda"])
      expect(screen.queryByRole("group", { name }), name).toBeNull();
    expect(
      screen.getAllByRole("group").filter((g) => g.tagName === "FIELDSET" && g.querySelector("legend")?.id),
    ).toHaveLength(4);
  });

  it("svaki gumb, potvrda i izbor u grupi imaju opis s imenom odjela", async () => {
    renderApp(echoApi(), "/plan");
    const bar = await screen.findByRole("group", { name: "Lobby bar" });
    const controls = [
      ...within(bar).getAllByRole("button"),
      ...within(bar).getAllByRole("checkbox"),
      ...within(bar).getAllByRole("radio"),
    ];
    expect(controls.length).toBeGreaterThanOrEqual(8);
    for (const c of controls) {
      const id = c.getAttribute("aria-describedby");
      expect(id, c.outerHTML).toBeTruthy();
      expect(document.getElementById(id ?? "")?.textContent).toBe("Lobby bar");
    }
  });

  it("zatvoren odjel: sve osim „Otvoren“ je onemogućeno; ponovno otvaranje vraća sezonce i oslobađa korak", async () => {
    renderApp(echoApi(), "/plan");
    const bar = await screen.findByRole("group", { name: "Lobby bar" });
    const open = within(bar).getByRole("checkbox", { name: "Otvoren" });
    fireEvent.click(open);
    expect(screen.getByText("Ovaj tjedan se može promijeniti još za 2 sezonca")).toBeTruthy();
    for (const b of within(bar).getAllByRole("button")) expect((b as HTMLButtonElement).disabled).toBe(true);
    for (const r of within(bar).getAllByRole("radio")) expect((r as HTMLInputElement).disabled).toBe(true);
    fireEvent.click(open);
    expect(screen.getByText("Ovaj tjedan se može promijeniti još za 5 sezonaca")).toBeTruthy();
    expect(within(bar).getByText("3 sezonca")).toBeTruthy();
  });

  it("tjedni tekst ne govori o krugu; „Spremi plan“ čeka odjele", async () => {
    renderApp(
      echoApi().on("GET", "/v1/hotels/:hotel/departments", () => new Promise<Response>(() => {})),
      "/plan",
    );
    expect(await screen.findByRole("heading", { level: 2, name: "Odjeli" })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Spremi plan" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("zatvaranje bara stavlja sezonce na 0 i šalje jedan plan", async () => {
    const api = echoApi();
    renderApp(api, "/plan");
    const bar = await screen.findByRole("group", { name: "Lobby bar" });
    fireEvent.click(within(bar).getByRole("checkbox", { name: "Otvoren" }));
    expect((within(bar).getByRole("button", { name: "Više sezonaca" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    const save = screen.getByRole("button", { name: "Spremi plan" });
    fireEvent.click(save);
    fireEvent.click(save);
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const body = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as {
      seasonal_target: number;
      maintenance: string;
      departments: Record<string, { open: boolean; seasonal_target: number }>;
    };
    expect(body.departments.lobby_bar).toMatchObject({ open: false, seasonal_target: 0 });
    // v1 polja se izvode iz odjela: Σ sezonaca (10 + 2 + 0 + 0 + službe 4 × 2 = 20) i održavanje Smještaja.
    expect(body).toMatchObject({ seasonal_target: 20, maintenance: "regular" });
  });

  it("restoran se ne može zatvoriti", async () => {
    renderApp(echoApi(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    expect(within(rest).queryByRole("checkbox", { name: "Otvoren" })).toBeNull();
  });

  it("korak sezonaca potrošen: plus onemogućen i rečenica", async () => {
    renderApp(echoApi(), "/plan");
    expect(await screen.findByText("Ovaj tjedan se može promijeniti još za 5 sezonaca")).toBeTruthy();
    const rest = await screen.findByRole("group", { name: "Restoran" });
    const plus = within(rest).getByRole("button", { name: "Više sezonaca" });
    for (let i = 0; i < 5; i++) fireEvent.click(plus);
    expect(screen.getByText("Ovaj tjedan sezonce više ne možeš mijenjati.")).toBeTruthy();
    expect((plus as HTMLButtonElement).disabled).toBe(true);
  });

  it("zatvaranje koje ne stane u preostali korak je onemogućeno", async () => {
    renderApp(echoApi(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    const plus = within(rest).getByRole("button", { name: "Više sezonaca" });
    for (let i = 0; i < 4; i++) fireEvent.click(plus);
    const bar = screen.getByRole("group", { name: "Lobby bar" });
    expect((within(bar).getByRole("checkbox", { name: "Otvoren" }) as HTMLInputElement).disabled).toBe(true);
  });

  it("razina cijene u koracima od 5 do granice; održavanje se šalje po odjelu", async () => {
    const api = echoApi();
    renderApp(api, "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    expect(within(rest).getByText("Cijene u odjelu")).toBeTruthy();
    expect(within(rest).getByText("100 %")).toBeTruthy();
    const up = within(rest).getByRole("button", { name: "Više cijene" });
    for (let i = 0; i < 6; i++) fireEvent.click(up);
    expect(within(rest).getByText("120 %")).toBeTruthy();
    expect((up as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(within(rest).getByRole("radio", { name: "pojačano" }));
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const body = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as {
      departments: Record<string, { price_level_pct: number; maintenance: string }>;
    };
    expect(body.departments.restaurant).toMatchObject({ price_level_pct: 120, maintenance: "intensive" });
  });

  it("422 department_plan_invalid je rečenica", async () => {
    renderApp(
      v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () => problem(422, "department_plan_invalid")),
      "/plan",
    );
    await screen.findByRole("group", { name: "Restoran" });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    expect((await screen.findByRole("alert")).textContent).toContain("Postavke odjela nisu dopuštene");
  });

  it("v1 igra nema dio Odjeli", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Odjeli" })).toBeNull();
  });
});
