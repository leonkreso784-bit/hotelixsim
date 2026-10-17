// IZGLED (ADR-042 t. 11): ekrani crtaju lika po rodu iz API-ja (`look`, `character_look`), ne samo po imenu.
import { cleanup, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CharacterLookBody } from "../api/types";
import { lookFromSeed } from "../avatar/look";
import { conversation, s1, situations, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** Ime čiji bi izgled iz imena imao bradu, pa je jasno da je prisutnost brade posljedica roda, ne slučaja. */
const BEARDED = (() => {
  for (let i = 0; i < 1000; i += 1) {
    const name = `Lik ${i}`;
    if (lookFromSeed(name).facialHair !== "none") return name;
  }
  throw new Error("nema imena s bradom");
})();

const FEMALE: CharacterLookBody = { gender: "f", age_group: null, origin: "foreign" };

const facial = () => document.querySelectorAll('[data-part^="facial-"]').length;

describe("izgled lika iz podataka API-ja", () => {
  it("popis situacija: bez `look` lik ima bradu iz imena, a gošća iz API-ja je nema", async () => {
    const withLook = (look: CharacterLookBody | null) =>
      standardApi().on(
        "GET",
        "/v1/hotels/:hotel/situations",
        situations([s1({ character: { name: BEARDED, role: "gošća hotela", look } })]),
      );
    renderApp(withLook(null), "/situacije");
    await screen.findByText("Klima u sobi obitelji s djecom");
    expect(facial()).toBeGreaterThan(0);
    cleanup();
    renderApp(withLook(FEMALE), "/situacije");
    await screen.findByText("Klima u sobi obitelji s djecom");
    expect(facial()).toBe(0);
  });

  it("razgovor: `character_look` gošće crta lice bez brade", async () => {
    const withLook = (character_look: CharacterLookBody | null) =>
      standardApi().on(
        "GET",
        "/v1/hotels/:hotel/situations/:code/conversation",
        conversation({ character_name: BEARDED, character_look }),
      );
    renderApp(withLook(null), "/situacije/S1");
    await screen.findAllByText(BEARDED);
    expect(facial()).toBeGreaterThan(0);
    cleanup();
    renderApp(withLook(FEMALE), "/situacije/S1");
    await screen.findAllByText(BEARDED);
    expect(facial()).toBe(0);
  });
});
