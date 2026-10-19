// Čuvar L1-13b dio 3: „opis Smještaja" (Leon 19.10., ADR-042 t. 7) — kratak tekst na stranici odjela Smještaj (v2)
// koji kaže da stanje zgrade i soba određuje kvarove prostora. Tekst je jedan i18n ključ bez parametara.

import { isMessageKey, type MessageKey, t } from "@hotelixsim/i18n";
import { cleanup, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { v2Api } from "../../test/fixtures";
import { fastPolling, renderApp } from "../../test/renderApp";
import { ROOMS_ABOUT_KEY } from "./roomsAboutKey";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function aboutText(): string {
  return isMessageKey(ROOMS_ABOUT_KEY) ? t(ROOMS_ABOUT_KEY as MessageKey) : "";
}

describe("Opis Smještaja (L1-13b-3)", () => {
  it("i18n ključ postoji i nije prazan", () => {
    expect(isMessageKey(ROOMS_ABOUT_KEY)).toBe(true);
    expect(aboutText().trim().length).toBeGreaterThan(20);
  });

  it("stranica Smještaja (v2) prikazuje opis", async () => {
    renderApp(v2Api(), "/odjeli/rooms");
    expect(await screen.findByRole("heading", { level: 1, name: "Smještaj" })).toBeTruthy();
    const text = aboutText();
    expect(text).not.toBe("");
    expect(screen.getByText(text)).toBeTruthy();
  });

  it("opis kaže da stanje zgrade i soba određuje kvarove i da ga održavanje podiže", () => {
    const text = aboutText();
    expect(text).toMatch(/zgrad/i);
    expect(text).toMatch(/sob/i);
    expect(text).toMatch(/stanj/i);
    expect(text).toMatch(/kvar/i);
    expect(text).toMatch(/održavanj/i);
  });

  // Smještaj je jedini odjel s ovim opisom; službe (Održavanje) i najam (Parking) uključeni.
  it.each([
    ["restaurant", "Restoran"],
    ["lobby_bar", "Lobby bar"],
    ["spa", "Mali wellness"],
    ["parking", "Parking"],
    ["maintenance", "Održavanje"],
    ["admin", "Uprava"],
  ])("odjel %s ne prikazuje opis Smještaja", async (code, name) => {
    renderApp(v2Api(), `/odjeli/${code}`);
    expect(await screen.findByRole("heading", { level: 1, name })).toBeTruthy();
    expect(isMessageKey(ROOMS_ABOUT_KEY)).toBe(true);
    expect(screen.queryByText(aboutText())).toBeNull();
  });

  it("popis odjela (/odjeli) ne prikazuje opis", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    expect(isMessageKey(ROOMS_ABOUT_KEY)).toBe(true);
    expect(screen.queryByText(aboutText())).toBeNull();
  });
});
