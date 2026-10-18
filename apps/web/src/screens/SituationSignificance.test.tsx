// L1-13 dio 2: razina značaja situacije (ključna / važna / svakodnevna) vidi se kao TEKST uz naslov, na popisu i na
// ekranu situacije (oznaka, ne samo boja: UI_PRAVILA). Sidra (`significance: null`) nemaju oznaku.
import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fastPolling, renderApp } from "../test/renderApp";
import { instance, v2SituationsApi } from "../test/situationsV2";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const LEVELS = ["Ključna", "Važna", "Svakodnevna"] as const;

/** Tri primjerka po jedne razine i sidro bez razine; naslovi ne sadrže riječi razina. */
function levelItems() {
  return [
    instance("P-KLIMA.t01", "rooms", "Klima ne radi u tri sobe", { weight: 4, significance: "kljucna" }),
    instance("P-HRANA.t01", "restaurant", "Gost se žali na hladnu večeru", {
      weight: 3,
      significance: "vazna",
    }),
    instance("P-BAR.t01", "lobby_bar", "Konobar kasni treći dan zaredom", {
      weight: 2,
      significance: "svakodnevna",
    }),
    instance("S2", null, "Šef kuhinje traži povišicu ili odlazi", {
      kind: null,
      weight: null,
      significance: null,
      mandatory: false,
    }),
  ];
}

function cards(): HTMLElement[] {
  const list = document.querySelector("main ul") as HTMLElement;
  return within(list).getAllByRole("link");
}

describe("popis situacija: oznaka razine kao tekst", () => {
  it("svaka razina ima svoj tekst na svojoj kartici, sidro nijedan", async () => {
    renderApp(v2SituationsApi(levelItems()), "/situacije");
    expect(await screen.findByRole("heading", { level: 1, name: "Situacije tjedna" })).toBeTruthy();
    await waitFor(() => expect(cards()).toHaveLength(4));
    const [key, important, everyday, anchor] = cards() as [
      HTMLElement,
      HTMLElement,
      HTMLElement,
      HTMLElement,
    ];
    for (const [card, level] of [
      [key, "Ključna"],
      [important, "Važna"],
      [everyday, "Svakodnevna"],
    ] as const) {
      const text = card.textContent ?? "";
      expect(text).toContain(level);
      // Točno jedna razina na kartici (ne i druge dvije).
      expect(LEVELS.filter((l) => text.includes(l))).toEqual([level]);
    }
    const anchorText = anchor.textContent ?? "";
    expect(LEVELS.some((l) => anchorText.includes(l))).toBe(false);
  });

  it("oznaka je vidljiv tekst (ne samo aria-label ili boja) i stoji prije statusa", async () => {
    renderApp(v2SituationsApi(levelItems()), "/situacije");
    await waitFor(() => expect(cards()).toHaveLength(4));
    const card = cards()[0] as HTMLElement;
    const badge = within(card).getByText("Ključna");
    expect(badge.textContent).toBe("Ključna");
    const status = within(card).getByText("Bez odgovora");
    expect(badge.compareDocumentPosition(status) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("stara igra ili v1 (significance: null svugdje): nijedna oznaka razine", async () => {
    const items = levelItems().map((i) => ({ ...i, significance: null }));
    renderApp(v2SituationsApi(items), "/situacije");
    await waitFor(() => expect(cards()).toHaveLength(4));
    for (const level of LEVELS) expect(screen.queryByText(level)).toBeNull();
  });
});

describe("ekran situacije: oznaka razine uz naslov", () => {
  it.each([
    ["P-KLIMA.t01", "Klima ne radi u tri sobe", "Ključna"],
    ["P-HRANA.t01", "Gost se žali na hladnu večeru", "Važna"],
    ["P-BAR.t01", "Konobar kasni treći dan zaredom", "Svakodnevna"],
  ])("%s pokazuje %s uz naslov", async (code, title, level) => {
    renderApp(v2SituationsApi(levelItems()), `/situacije/${code}`);
    const heading = await screen.findByRole("heading", { level: 1, name: title });
    const badge = await screen.findByText(level);
    // Oznaka je nakon naslova i prije ponuđenih odgovora.
    expect(heading.compareDocumentPosition(badge) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const answers = screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" });
    expect(
      badge.compareDocumentPosition(answers[0] as HTMLElement) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    for (const other of LEVELS.filter((l) => l !== level)) expect(screen.queryByText(other)).toBeNull();
  });

  it("sidro bez razine: naslov bez oznake", async () => {
    renderApp(v2SituationsApi(levelItems()), "/situacije/S2");
    expect(
      await screen.findByRole("heading", { level: 1, name: "Šef kuhinje traži povišicu ili odlazi" }),
    ).toBeTruthy();
    for (const level of LEVELS) expect(screen.queryByText(level)).toBeNull();
  });
});
