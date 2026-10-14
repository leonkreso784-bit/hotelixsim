// W-1: situacije v2 — kartice po važnosti s odjelom, obvezne, brojač riješenih i „Prepusti voditelju“ (P4, P5).
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { SituationBody } from "../api/types";
import { json, problem } from "../test/fakeApi";
import { standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";
import {
  afterDelegate,
  anchor,
  instance,
  LIFT,
  LIFT_DESCRIPTION,
  submitted,
  v2Items,
  v2Situations,
  v2SituationsApi,
} from "../test/situationsV2";
import { situationStatus } from "./SituationsScreen";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const DELEGATE_PATH = `/v1/hotels/h1/situations/${LIFT}/delegate`;
const CONFIRM_TEXT =
  "Odjel Održavanje: voditelj rješava situaciju sam. Ishod: ocjena 2 od 5, bez razgovora i bez AI ocjene.";

/** Kartice popisa (svaka je jedan Link), redom kako su na ekranu. */
function cards(): HTMLElement[] {
  const list = document.querySelector("main ul") as HTMLElement;
  return within(list).getAllByRole("link");
}

/** Ime odjela je na kartici kao tekst ili kao `aria-label` (graditelj bira oblik). */
function showsDept(card: HTMLElement, name: string): boolean {
  return (card.textContent ?? "").includes(name) || card.querySelector(`[aria-label="${name}"]`) !== null;
}

/** Izmijenjena jedna situacija iz `v2Items()`. */
function itemsWith(code: string, over: Partial<SituationBody>): SituationBody[] {
  return v2Items().map((i) => (i.code === code ? { ...i, ...over } : i));
}

describe("W-1 popis: po važnosti, odjel, obvezne, brojač", () => {
  it("kartice redom kako stižu s poslužitelja, s imenom i ikonom odjela; bez internih kodova", async () => {
    renderApp(v2SituationsApi(), "/situacije");
    expect(await screen.findByRole("heading", { level: 1, name: "Situacije tjedna" })).toBeTruthy();
    // Imena odjela stižu iz API-ja (`/departments`) — čekamo zadnje.
    await waitFor(() => expect(showsDept(cards()[4] as HTMLElement, "Održavanje")).toBe(true));
    const all = cards();
    expect(all.map((a) => a.getAttribute("href"))).toEqual(
      v2Items().map((i) => `/situacije/${encodeURIComponent(i.code)}`),
    );
    const [hotel, rooms, restaurant, bar, lift] = all as [
      HTMLElement,
      HTMLElement,
      HTMLElement,
      HTMLElement,
      HTMLElement,
    ];
    expect(showsDept(rooms, "Smještaj")).toBe(true);
    expect(showsDept(restaurant, "Restoran")).toBe(true);
    expect(showsDept(bar, "Lobby bar")).toBe(true);
    // Ikona odjela iz `departments/icons.tsx`, skrivena od čitača zaslona (ime nosi tekst).
    const icon = (card: HTMLElement, name: string) => card.querySelector(`svg.lucide-${name}`);
    expect(icon(rooms, "bed-double")?.getAttribute("aria-hidden")).toBe("true");
    expect(icon(restaurant, "utensils-crossed")).not.toBeNull();
    expect(icon(bar, "martini")).not.toBeNull();
    expect(icon(lift, "wrench")).not.toBeNull();
    // Situacija cijelog hotela: ikona hotela (osim strelice kartice).
    expect(hotel.querySelectorAll("svg.lucide:not(.lucide-chevron-right)").length).toBeGreaterThan(0);
    // UI_PRAVILA §3.4: nikad interni kod na ekranu.
    const text = (document.querySelector("main ul") as HTMLElement).textContent ?? "";
    expect(text).not.toMatch(/restaurant|lobby_bar|maintenance|\brooms\b|\.t01|P-LIFT/);
  });

  it("značka „Obvezna“ točno na obveznima; prepuštena ima stanje „Prepušteno voditelju“", async () => {
    renderApp(v2SituationsApi(), "/situacije");
    await screen.findByRole("heading", { level: 1, name: "Situacije tjedna" });
    const all = cards();
    const mandatory = all.map((c) => within(c).queryByText("Obvezna") !== null);
    expect(mandatory).toEqual(v2Items().map((i) => i.mandatory));
    expect(within(all[3] as HTMLElement).getByText("Prepušteno voditelju")).toBeTruthy();
    expect(screen.getAllByText("Prepušteno voditelju")).toHaveLength(1);
    expect(screen.getAllByText("Bez odgovora")).toHaveLength(2);
  });

  it("brojač „Riješene situacije: 3 od 5“: odgovoreno + predano + prepušteno", async () => {
    renderApp(v2SituationsApi(), "/situacije");
    expect(await screen.findByText("Riješene situacije: 3 od 5")).toBeTruthy();
  });

  it("brojač broji i odgovor koji AI još ocjenjuje (predano)", async () => {
    const items = [
      instance("P-KLIMA.t01", "rooms", "Klima ne radi u tri sobe", { mandatory: true }),
      instance("P-HRANA.t01", "restaurant", "Gost se žali na hladnu večeru", {
        mandatory: true,
        response: { ...submitted, status: "pending", quality: null, rationale: null },
      }),
    ];
    renderApp(v2SituationsApi(items), "/situacije");
    expect(await screen.findByText("Riješene situacije: 1 od 2")).toBeTruthy();
  });

  it("zatvoren tjedan: bez brojača, samo rečenica „samo za čitanje“", async () => {
    renderApp(v2SituationsApi(undefined, { open: false, unanswered: 0 }), "/situacije");
    expect(await screen.findByText("Tjedan je zatvoren — situacije su samo za čitanje.")).toBeTruthy();
    expect(screen.queryByText(/Riješen/)).toBeNull();
  });

  it("prazno stanje (izravno na putanji): jedna rečenica, bez brojača i praznog popisa", async () => {
    renderApp(v2SituationsApi([]), "/situacije");
    expect(await screen.findByText("Ovaj tjedan nema situacija.")).toBeTruthy();
    expect(screen.queryByText(/Riješen/)).toBeNull();
    expect(document.querySelector("main ul")).toBeNull();
  });

  it("situationStatus: prepuštena je „Prepušteno voditelju“, prigušeni ton", () => {
    expect(situationStatus(instance("P-BAR.t01", "lobby_bar", "Konobar kasni", { delegated: true }))).toEqual(
      {
        text: "Prepušteno voditelju",
        tone: "muted",
      },
    );
  });
});

describe("W-1 detalj: opis, obvezna, prepuštena", () => {
  it("opis ispod naslova; gumb „Prepusti voditelju“ ispod ponuđenih odgovora", async () => {
    renderApp(v2SituationsApi(), `/situacije/${LIFT}`);
    expect(await screen.findByRole("heading", { level: 1, name: "Dizalo stane između katova" })).toBeTruthy();
    expect(screen.getByText(LIFT_DESCRIPTION)).toBeTruthy();
    const delegate = await screen.findByRole("button", { name: "Prepusti voditelju" });
    const answers = screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" });
    const last = answers[answers.length - 1] as HTMLElement;
    expect(last.compareDocumentPosition(delegate) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.queryByText("Obvezna")).toBeNull();
  });

  it("opis jednak naslovu se ne ponavlja", async () => {
    const title = "Dizalo stane između katova";
    renderApp(v2SituationsApi(itemsWith(LIFT, { description: title })), `/situacije/${LIFT}`);
    await screen.findByRole("button", { name: "Prepusti voditelju" });
    expect(screen.getAllByText(title)).toHaveLength(1);
  });

  it("obvezna: značka „Obvezna“ i rečenica o prve tri, bez gumba", async () => {
    renderApp(v2SituationsApi(), "/situacije/P-KLIMA.t01");
    expect(await screen.findByRole("heading", { level: 1, name: "Klima ne radi u tri sobe" })).toBeTruthy();
    expect(await screen.findByText("Obvezna")).toBeTruthy();
    expect(screen.getByText(/Prve tri situacije/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
  });

  it("prepuštena: jedna rečenica s imenom odjela, ništa za uređivanje", async () => {
    renderApp(v2SituationsApi(), "/situacije/P-BAR.t01");
    expect(await screen.findByText("Prepušteno voditelju (odjel Lobby bar).")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull();
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
    expect(screen.queryByText("Tjedan je zatvoren bez odgovora.")).toBeNull();
  });

  const noButton: [string, SituationBody[], { open: boolean } | undefined][] = [
    ["odgovorena", itemsWith(LIFT, { chosen: 0 }), undefined],
    ["predana (slobodan odgovor)", itemsWith(LIFT, { free_text: true, response: submitted }), undefined],
    ["zatvoren krug", v2Items(), { open: false }],
    ["stara igra bez voditelja (leader: null)", itemsWith(LIFT, { leader: null }), undefined],
  ];
  it.each(noButton)("bez gumba „Prepusti voditelju“: %s", async (_name, items, over) => {
    renderApp(v2SituationsApi(items, over), `/situacije/${LIFT}`);
    expect(await screen.findByRole("heading", { level: 1, name: "Dizalo stane između katova" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
  });
});

describe("W-1 prepuštanje: potvrda, jedan zahtjev, 409", () => {
  it("potvrda: naslov dobiva fokus, rečenica što se dogodi; „Odustani“ vraća fokus bez zahtjeva", async () => {
    const api = v2SituationsApi();
    renderApp(api, `/situacije/${LIFT}`);
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    const focused = document.activeElement as HTMLElement;
    expect(focused.tagName).toMatch(/^H[2-4]$/);
    expect(await screen.findByText(CONFIRM_TEXT)).toBeTruthy();
    expect(api.posts(DELEGATE_PATH)).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: "Odustani" }));
    expect(screen.queryByText(CONFIRM_TEXT)).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Prepusti voditelju" }));
    expect(api.posts(DELEGATE_PATH)).toHaveLength(0);
  });

  it("dvostruki klik na potvrdu: točno jedan POST …/delegate; odgovor ide u cache, ne čeka novi dohvat", async () => {
    // GET popisa i dalje vraća staro stanje: rečenica se može pojaviti samo iz tijela odgovora na POST.
    const api = v2SituationsApi().on("POST", "/v1/hotels/:hotel/situations/:code/delegate", () =>
      json(200, afterDelegate(LIFT)),
    );
    renderApp(api, `/situacije/${LIFT}`);
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    const card = (document.activeElement as HTMLElement).parentElement as HTMLElement;
    const confirm = within(card).getByRole("button", { name: /Prepusti/ });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    expect(await screen.findByText("Prepušteno voditelju (odjel Održavanje).")).toBeTruthy();
    expect(api.posts(DELEGATE_PATH)).toHaveLength(1);
    expect(api.posts(DELEGATE_PATH)[0]?.body).toBeUndefined();
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
    expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull();
    // UI_PRAVILA §6.5: kliknuti gumb je nestao, fokus se ne gubi na <body>.
    await waitFor(() => expect(document.activeElement).not.toBe(document.body));
  });

  it("nakon prepuštanja popis kaže „Prepušteno voditelju“ i brojač raste", async () => {
    let delegated = false;
    const api = v2SituationsApi()
      .on("GET", "/v1/hotels/:hotel/situations", () =>
        json(200, delegated ? afterDelegate(LIFT) : v2Situations()),
      )
      .on("POST", "/v1/hotels/:hotel/situations/:code/delegate", () => {
        delegated = true;
        return json(200, afterDelegate(LIFT));
      });
    renderApp(api, `/situacije/${LIFT}`);
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    const card = (document.activeElement as HTMLElement).parentElement as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /Prepusti/ }));
    await screen.findByText("Prepušteno voditelju (odjel Održavanje).");
    fireEvent.click(screen.getByRole("link", { name: "Sve situacije" }));
    expect(await screen.findByText("Riješene situacije: 4 od 5")).toBeTruthy();
    const lift = cards()[4] as HTMLElement;
    expect(within(lift).getByText("Prepušteno voditelju")).toBeTruthy();
  });

  it("409 je hrvatska rečenica u upozorenju (role=alert)", async () => {
    const api = v2SituationsApi().on("POST", "/v1/hotels/:hotel/situations/:code/delegate", () =>
      problem(409, "already_answered"),
    );
    renderApp(api, `/situacije/${LIFT}`);
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    const card = (document.activeElement as HTMLElement).parentElement as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /Prepusti/ }));
    const msg = await screen.findByText("Na ovu situaciju je već odgovoreno.");
    expect(msg.closest("[role='alert']")).not.toBeNull();
    expect(api.posts(DELEGATE_PATH)).toHaveLength(1);
  });
});

// W-1 popravak 1 (sit-w1-p1-brief.md): obvezne samo uz voditelja, sidra bez „Cijeli hotel“, 409, jedna potvrda.
const SITS_PATH = "/v1/hotels/h1/situations";
const ALREADY = "Na ovu situaciju je već odgovoreno.";

/** Otvorene potvrde na ekranu (izbor odgovora i prepuštanje) — smije biti najviše jedna. */
function openConfirms(): number {
  return screen.queryAllByRole("heading", { name: /^(Izabrani odgovor|Prepuštanje voditelju)$/ }).length;
}

describe("W-1 popravak 1: „Obvezna“ samo uz voditelja (I1)", () => {
  it("v1 igra (poslužitelj šalje mandatory: true, leader: null): popis bez značke „Obvezna“", async () => {
    renderApp(standardApi(), "/situacije");
    await screen.findByRole("heading", { level: 1, name: "Situacije mjeseca" });
    expect(screen.getAllByText("Bez odgovora")).toHaveLength(2);
    expect(screen.queryByText("Obvezna")).toBeNull();
  });

  it("v1 igra: detalj otvorene situacije bez značke i bez rečenice o prve tri", async () => {
    renderApp(standardApi(), "/situacije/S3");
    expect(await screen.findByRole("heading", { level: 1, name: /Agencija nudi grupu/ })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" }).length).toBeGreaterThan(0);
    expect(screen.queryByText("Obvezna")).toBeNull();
    expect(screen.queryByText(/Prve tri situacije/)).toBeNull();
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
  });

  // v2 kad nijedan odjel nije otvoren: poslužitelj i 4.+ situaciju šalje kao obveznu, bez voditelja.
  const noLeader = () => itemsWith(LIFT, { mandatory: true, leader: null });

  it("v2 bez voditelja (mandatory: true, leader: null): popis bez značke; prave obvezne je zadrže", async () => {
    renderApp(v2SituationsApi(noLeader()), "/situacije");
    await screen.findByRole("heading", { level: 1, name: "Situacije tjedna" });
    expect(cards().map((c) => within(c).queryByText("Obvezna") !== null)).toEqual([
      true,
      true,
      true,
      false,
      false,
    ]);
  });

  it("v2 bez voditelja: detalj bez značke i bez rečenice o prve tri", async () => {
    renderApp(v2SituationsApi(noLeader()), `/situacije/${LIFT}`);
    expect(await screen.findByRole("heading", { level: 1, name: "Dizalo stane između katova" })).toBeTruthy();
    expect(screen.queryByText("Obvezna")).toBeNull();
    expect(screen.queryByText(/Prve tri situacije/)).toBeNull();
  });
});

describe("W-1 popravak 1: sidra nisu „Cijeli hotel“ (I2)", () => {
  it("sidro (department: null, kind: null) bez retka odjela; predložak razine hotela i dalje „Cijeli hotel“", async () => {
    renderApp(v2SituationsApi([anchor(), ...v2Items()]), "/situacije");
    await waitFor(() => expect(showsDept(cards()[5] as HTMLElement, "Održavanje")).toBe(true));
    const [s2, hotel] = cards() as [HTMLElement, HTMLElement];
    expect(s2.textContent).not.toContain("Cijeli hotel");
    // Voditelj sidra (odjel s najviše ljudi) nije odjel situacije: ni njegovo ime ni ikona.
    expect(showsDept(s2, "Smještaj")).toBe(false);
    expect(s2.querySelectorAll("svg.lucide:not(.lucide-chevron-right)")).toHaveLength(0);
    expect(hotel.textContent).toContain("Cijeli hotel");
    expect(hotel.querySelectorAll("svg.lucide:not(.lucide-chevron-right)").length).toBeGreaterThan(0);
  });
});

describe("W-1 popravak 1: 409 pri prepuštanju", () => {
  /** Otvori potvrdu na LIFT-u i potvrdi; vraća broj GET-ova situacija prije potvrde. */
  async function confirmDelegate(api: ReturnType<typeof v2SituationsApi>): Promise<number> {
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    const card = (document.activeElement as HTMLElement).parentElement as HTMLElement;
    const before = situationGets(api);
    fireEvent.click(within(card).getByRole("button", { name: /Prepusti/ }));
    return before;
  }
  const situationGets = (api: ReturnType<typeof v2SituationsApi>) =>
    api.calls.filter((c) => c.method === "GET" && c.path === SITS_PATH).length;

  const conflictApi = () =>
    v2SituationsApi().on("POST", "/v1/hotels/:hotel/situations/:code/delegate", () =>
      problem(409, "already_answered"),
    );

  it("situacije se ponovno dohvate (invalidacija upita situacija)", async () => {
    const api = conflictApi();
    renderApp(api, `/situacije/${LIFT}`);
    const before = await confirmDelegate(api);
    expect(await screen.findByText(ALREADY)).toBeTruthy();
    await waitFor(() => expect(situationGets(api)).toBeGreaterThan(before));
  });

  it("fokus je na rečenici greške (ne na <body> ni na naslovu potvrde)", async () => {
    const api = conflictApi();
    renderApp(api, `/situacije/${LIFT}`);
    await confirmDelegate(api);
    expect(await screen.findByText(ALREADY)).toBeTruthy();
    await waitFor(() => {
      const alert = screen.getByText(ALREADY).closest("[role='alert']") as HTMLElement;
      expect(alert.contains(document.activeElement)).toBe(true);
    });
  });

  it("odgovoreno drugdje: osvježen ekran više ne nudi ni prepuštanje ni odgovore", async () => {
    let answered = false;
    const api = v2SituationsApi()
      .on("GET", "/v1/hotels/:hotel/situations", () =>
        json(200, v2Situations(answered ? itemsWith(LIFT, { chosen: 0 }) : v2Items())),
      )
      .on("POST", "/v1/hotels/:hotel/situations/:code/delegate", () => {
        answered = true;
        return problem(409, "already_answered");
      });
    renderApp(api, `/situacije/${LIFT}`);
    await confirmDelegate(api);
    await waitFor(() => expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull());
    expect(screen.queryByRole("button", { name: /Prepusti/ })).toBeNull();
    expect(screen.getByText("Tvoj izbor")).toBeTruthy();
    await waitFor(() => expect(document.activeElement).not.toBe(document.body));
  });
});

describe("W-1 popravak 1: nikad dvije potvrde istodobno", () => {
  it("izbor odgovora, pa „Prepusti voditelju“: najviše jedna potvrda i jedan „Odustani“", async () => {
    renderApp(v2SituationsApi(), `/situacije/${LIFT}`);
    await screen.findByRole("button", { name: "Prepusti voditelju" });
    fireEvent.click(screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" })[0] as HTMLElement);
    expect(screen.getByRole("heading", { name: "Izabrani odgovor" })).toBeTruthy();
    // Graditelj bira: gumb druge potvrde se sakrije, ili njezino otvaranje zatvori prvu.
    const delegate = screen.queryByRole("button", { name: "Prepusti voditelju" });
    if (delegate) {
      fireEvent.click(delegate);
      expect(screen.getByRole("heading", { name: "Prepuštanje voditelju" })).toBeTruthy();
    }
    expect(openConfirms()).toBe(1);
    expect(screen.getAllByRole("button", { name: "Odustani" })).toHaveLength(1);
  });

  it("„Prepusti voditelju“, pa izbor odgovora: najviše jedna potvrda i jedan „Odustani“", async () => {
    renderApp(v2SituationsApi(), `/situacije/${LIFT}`);
    fireEvent.click(await screen.findByRole("button", { name: "Prepusti voditelju" }));
    expect(screen.getByRole("heading", { name: "Prepuštanje voditelju" })).toBeTruthy();
    const pick = screen.queryAllByRole("button", { name: "Izaberi ovaj odgovor" })[0] as
      | HTMLButtonElement
      | undefined;
    if (pick && !pick.disabled) {
      fireEvent.click(pick);
      expect(screen.getByRole("heading", { name: "Izabrani odgovor" })).toBeTruthy();
    }
    expect(openConfirms()).toBe(1);
    expect(screen.getAllByRole("button", { name: "Odustani" })).toHaveLength(1);
  });
});
