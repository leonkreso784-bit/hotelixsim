import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import { departmentDetailBody, departmentsBody, standardApi, v2Api } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("Odjeli", () => {
  it("popis: samo odjeli hotela, brojke s imenom i jedinicom", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    const spa = (await screen.findByRole("link", { name: /Mali wellness/ })).closest(
      "article",
    ) as HTMLElement;
    expect(within(spa).getByText("Stanje prostora")).toBeTruthy();
    expect(within(spa).getByText("65/100")).toBeTruthy();
    expect(within(spa).getByText("Zadovoljstvo gostiju")).toBeTruthy();
    expect(within(spa).getByText("Ljudi: 2 stalna, 1 sezonski")).toBeTruthy();
    expect(screen.queryByText("Rooftop bar")).toBeNull();
    expect(screen.getByRole("heading", { name: "Službe bez prihoda" })).toBeTruthy();
  });

  it("detalj odjela izravno na putanji: prostor, gosti, novac, ljudi", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    for (const h of ["Prostor", "Gosti", "Novac", "Ljudi"])
      expect(screen.getByRole("heading", { level: 2, name: h })).toBeTruthy();
    expect(screen.getByText("67.200 €")).toBeTruthy();
    expect(screen.getByText("trošenje −3")).toBeTruthy();
    expect(screen.getByText("preopterećenje −1")).toBeTruthy();
    expect(screen.getByText("održavanje +3")).toBeTruthy();
    // Poslužiteljeve promjene već nose predznak: prije + promjene = poslije.
    const r = departmentDetailBody("restaurant")?.department.last;
    expect(r && r.condition_before_pts + r.wear_pts + r.load_wear_pts + r.maintenance_pts).toBe(
      r?.condition_after_pts,
    );
    expect(screen.getByRole("heading", { level: 2, name: "Po tjednima" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 3, name: "Restoran" })).toBeTruthy();
    expect(screen.getByText("udio gostiju koji dolaze 45 %")).toBeTruthy();
  });

  it("računalo: popis i detalj u dva stupca", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const grid = (await screen.findByRole("heading", { level: 1, name: "Restoran" })).closest(
      "[data-layout='list-detail']",
    );
    expect(grid?.className).toMatch(/lg:grid/);
    // Stupac detalja se smije suziti i tablica se pomiče u svojoj regiji, nikad ne gura stranicu.
    expect(
      screen.getByRole("heading", { level: 1, name: "Restoran" }).closest("[data-layout] > div")?.className,
    ).toMatch(/min-w-0/);
    const region = screen.getByRole("region", { name: "Rezultat odjela po krugovima" });
    expect(region.className).toMatch(/overflow-x-auto/);
    expect(region.getAttribute("tabindex")).toBe("0");
  });

  it("v1 igra: rečenica umjesto odjela i nema stavke izbornika", async () => {
    renderApp(
      standardApi().on("GET", "/v1/hotels/:hotel/departments", () => problem(409, "model_v1")),
      "/odjeli",
    );
    expect(await screen.findByText("Ova igra je na starom modelu i nema ovaj prikaz.")).toBeTruthy();
    expect(await screen.findByText(/LIPANJ/)).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });

  it("nepoznat odjel: rečenica, ne prazno", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => problem(404, "unknown_department")),
      "/odjeli/casino",
    );
    expect(await screen.findByText("Taj odjel ne postoji u ovom hotelu.")).toBeTruthy();
    expect(screen.getByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.getAllByRole("link", { name: "Odjeli" }).length).toBeGreaterThan(1);
  });

  it("v2 igra: izbornik ima Odjeli, zatvoren odjel nosi oznaku, kartica je jedna poveznica", async () => {
    renderApp(v2Api(), "/odjeli");
    const parking = (await screen.findByRole("link", { name: "Parking" })).closest("article") as HTMLElement;
    expect(within(parking).getByText("Zatvoren")).toBeTruthy();
    expect(within(parking).getAllByRole("link")).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Odjeli" }).getAttribute("href")).toBe("/odjeli");
    expect(screen.getByRole("link", { name: /Uprava/ }).textContent).toContain("59.050 €");
  });

  it("služba bez prihoda: trošak po stavkama, bez kartica Prostor i Gosti", async () => {
    renderApp(v2Api(), "/odjeli/admin");
    expect(await screen.findByRole("heading", { level: 1, name: "Uprava" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Trošak po stavkama" })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Prostor" })).toBeNull();
    expect(screen.getByRole("heading", { level: 2, name: "Ljudi" })).toBeTruthy();
  });

  it("odjel bez obračunatog kruga: rečenica što će se pojaviti, ne nule", async () => {
    renderApp(v2Api(), "/odjeli/parking");
    expect(await screen.findByText("Rezultat se pojavi nakon prvog obračunatog tjedna.")).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Novac" })).toBeNull();
  });

  it("prije prvog obračuna: rečenica kad se pojavljuju rezultati", async () => {
    const empty = {
      round: null,
      departments: departmentsBody().departments.map((d) => ({ ...d, last: null, service_last: null })),
    };
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/departments", empty), "/odjeli");
    expect(
      await screen.findByText("Rezultati odjela pojave se nakon prvog obračunatog tjedna."),
    ).toBeTruthy();
  });

  it("detalj dok se učitava ima povratak i naslov prve razine", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => new Promise<Response>(() => {})),
      "/odjeli/restaurant",
    );
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    await waitFor(() => expect(screen.getAllByRole("link", { name: "Odjeli" }).length).toBeGreaterThan(1));
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("ljudi: odlasci odjela u zadnjem krugu, i kad nitko nije otišao", async () => {
    renderApp(v2Api(), "/odjeli/rooms");
    expect(await screen.findByRole("heading", { level: 1, name: "Smještaj" })).toBeTruthy();
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("Otišlo u krugu")).toBeTruthy();
    expect(within(people).getByText("2 osobe")).toBeTruthy();
    expect(
      within(people).getByText("Tko ode, zamijeni se novim zaposlenim; trošak zamjene je u Upravi."),
    ).toBeTruthy();
    cleanup();
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    const quiet = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(quiet).getByText("Otišlo u krugu")).toBeTruthy();
    expect(within(quiet).getByText("nitko")).toBeTruthy();
    expect(within(quiet).queryByText(/trošak zamjene/)).toBeNull();
  });

  it("služba: odlasci iz zadnjeg troška službe", async () => {
    renderApp(v2Api(), "/odjeli/admin");
    expect(await screen.findByRole("heading", { level: 1, name: "Uprava" })).toBeTruthy();
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("1 osoba")).toBeTruthy();
  });
});
