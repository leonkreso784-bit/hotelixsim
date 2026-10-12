// Cijene odluka plana (Leon 12.10.: obnova je koštala 150.000 € a kvačica to nije govorila). Brojke dolaze iz granica
// plana (poslužitelj ih čita iz kataloga); web ih samo prikazuje.
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HotelBody } from "../../api/types";
import { standardApi, v2Api, v2PlanHotel } from "../../test/fixtures";
import { renderApp } from "../../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** Opis elementa (aria-describedby) kao jedan tekst. */
function description(el: HTMLElement): string {
  return (el.getAttribute("aria-describedby") ?? "")
    .split(" ")
    .map((id) => document.getElementById(id)?.textContent ?? "")
    .join(" ");
}

function withCash(cents: number): HotelBody {
  const h = v2PlanHotel();
  if (h.private) h.private.now.cash_cents = cents;
  return h;
}

describe("cijene odluka u planu", () => {
  it("v1: održavanje i obuka imaju cijenu uz svaki izbor, obuka i učinak", async () => {
    renderApp(standardApi(), "/plan");
    const regular = await screen.findByRole("radio", { name: "redovno" });
    expect(description(regular)).toContain("15.000 €");
    expect(description(screen.getByRole("radio", { name: "pojačano" }))).toContain("30.000 €");
    expect(description(screen.getByRole("radio", { name: "osnovna" }))).toContain("5.000 €");
    fireEvent.click(screen.getByRole("radio", { name: "puna" }));
    expect(
      screen.getByText("Puna obuka: 10.000 € ovaj mjesec; od idućeg mjeseca iskustvo osoblja +7, moral +2."),
    ).toBeTruthy();
    expect(screen.getByText("Trošak 1.500 € ovaj mjesec.")).toBeTruthy();
  });

  it("tjedna igra: iznosi za tjedan", async () => {
    renderApp(v2Api(), "/plan");
    expect(description(await screen.findByRole("radio", { name: "osnovna" }))).toContain("1.167 €");
    expect(screen.getByText("Trošak 350 € ovaj tjedan.")).toBeTruthy();
  });

  it("obnova: cijena i učinak uvijek; uključena pokazuje blagajnu nakon i upozorenje kad ode u minus", async () => {
    renderApp(v2Api(), "/plan");
    const box = await screen.findByLabelText("Obnova (jednom u sezoni)");
    expect(screen.getByText("Jednokratno 150.000 €; oprema +25, zadovoljstvo gostiju +3.")).toBeTruthy();
    expect(screen.queryByText(/nakon obnove/)).toBeNull();
    fireEvent.click(box);
    expect(screen.getByText("Blagajna sada 42.000 €, nakon obnove −108.000 €.")).toBeTruthy();
    const warning = screen
      .getByText("Pozor: nakon obnove blagajna bi bila u minusu.")
      .closest("[role='status']");
    expect(warning).toBeTruthy();
    expect(within(warning as HTMLElement).getByText(/Pozor/)).toBeTruthy();
    expect((warning as HTMLElement).querySelector("svg[aria-hidden='true']")).toBeTruthy();
    // Jedan klik, bez skočnog prozora: kvačica je uključena.
    expect((box as HTMLInputElement).checked).toBe(true);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("obnova: manje od jednog tjedna plaća u blagajni je upozorenje; dovoljno novca nema upozorenja", async () => {
    renderApp(v2Api().on("GET", "/v1/me/hotel", withCash(17_000_000)), "/plan");
    fireEvent.click(await screen.findByLabelText("Obnova (jednom u sezoni)"));
    expect(screen.getByText("Blagajna sada 170.000 €, nakon obnove 20.000 €.")).toBeTruthy();
    expect(
      screen.getByText("Pozor: nakon obnove u blagajni ostaje manje od jednog tjedna plaća (30.000 €)."),
    ).toBeTruthy();
    cleanup();
    renderApp(v2Api().on("GET", "/v1/me/hotel", withCash(20_000_000)), "/plan");
    fireEvent.click(await screen.findByLabelText("Obnova (jednom u sezoni)"));
    expect(screen.getByText("Blagajna sada 200.000 €, nakon obnove 50.000 €.")).toBeTruthy();
    expect(screen.queryByText(/Pozor/)).toBeNull();
  });

  it("obuka: nulta promjena riječima, ne „moral 0“ (I3)", async () => {
    renderApp(v2Api(), "/plan");
    fireEvent.click(await screen.findByRole("radio", { name: "osnovna" }));
    expect(
      screen.getByText(
        "Osnovna obuka: 1.167 € ovaj tjedan; od idućeg tjedna iskustvo osoblja +1, moral bez promjene.",
      ),
    ).toBeTruthy();
    expect(screen.queryByText(/moral 0/)).toBeNull();
  });

  it("cijena sobe, marketing i plaće: uz svako polje što mijenja, plaće s iznosom (I3)", async () => {
    renderApp(v2Api(), "/plan");
    const price = await screen.findByLabelText("Cijena sobe po noći (€)");
    expect(description(price)).toContain(
      "Viša cijena donosi više novca po noći, ali manje gostiju rezervira",
    );
    expect(description(screen.getByLabelText("Marketing (€)"))).toContain(
      "Upisani iznos je trošak ovog tjedna. Marketing dovodi više gostiju",
    );
    const wage = description(screen.getByLabelText("Plaće (% uobičajene plaće)"));
    expect(wage).toContain("Pri sadašnjim plaćama hotel ovaj tjedan na plaće troši oko 30.000 €.");
    expect(wage).toContain("Viša plaća podiže moral osoblja");
  });

  it("odluke odjela: cijena sezonskog radnika i učinak cijena (I3)", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const rest = await screen.findByRole("group", { name: "Restoran" }, { timeout: 3000 });
    expect(
      within(rest).getByText(/^Jedan sezonski radnik u ovom odjelu stoji 490 € ovaj tjedan\. Više ljudi/),
    ).toBeTruthy();
    expect(within(rest).getByText(/^Više cijene donose više novca od svakog gosta/)).toBeTruthy();
  });

  it("održavanje odjela: tjedna cijena uz svaki izbor", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const rest = await screen.findByRole("group", { name: "Restoran" }, { timeout: 3000 });
    expect(description(within(rest).getByRole("radio", { name: "redovno" }))).toContain("933 €");
    expect(description(within(rest).getByRole("radio", { name: "pojačano" }))).toContain("1.867 €");
    expect(description(within(rest).getByRole("radio", { name: "redovno" }))).toContain("Restoran");
  });
});
