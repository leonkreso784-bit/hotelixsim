import { cleanup, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import { standardApi, v2Api } from "../test/fixtures";
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
    expect(screen.getByText("udio gostiju koji dolaze 45 %")).toBeTruthy();
  });

  it("računalo: popis i detalj u dva stupca", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const grid = (await screen.findByRole("heading", { level: 1, name: "Restoran" })).closest(
      "[data-layout='list-detail']",
    );
    expect(grid?.className).toMatch(/lg:grid/);
  });

  it("v1 igra: rečenica umjesto odjela i nema stavke izbornika", async () => {
    renderApp(
      standardApi().on("GET", "/v1/hotels/:hotel/departments", () => problem(409, "model_v1")),
      "/odjeli",
    );
    expect(await screen.findByText("Ova igra je na starom modelu i nema ovaj prikaz.")).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });

  it("nepoznat odjel: rečenica, ne prazno", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => problem(404, "unknown_department")),
      "/odjeli/casino",
    );
    expect(await screen.findByText("Taj odjel ne postoji u ovom hotelu.")).toBeTruthy();
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

  it("mobitel: popis se krije dok je otvoren detalj, a detalj ima povratak; bez vodoravnog pomicanja", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const h1 = await screen.findByRole("heading", { level: 1, name: "Restoran" });
    const back = screen
      .getAllByRole("link", { name: "Odjeli" })
      .find((a) => /lg:hidden/.test(a.className)) as HTMLElement;
    expect(back.className).toMatch(/lg:hidden/);
    expect(back.className).toMatch(/min-h-11/);
    const list = screen.getByRole("heading", { level: 2, name: "Odjeli" }).closest("div[class*='hidden']");
    expect(list?.className).toMatch(/hidden lg:block/);
    expect(h1.closest("[data-layout='list-detail']")?.className).not.toMatch(/overflow-x/);
  });
});
