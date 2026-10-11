import { cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { keepRoundTogether } from "../format";
import { json } from "../test/fakeApi";
import { ix, plan, standardApi, v2Api, weeklyGame } from "../test/fixtures";
import { joined } from "../test/joined";
import { renderApp } from "../test/renderApp";
import { feedEntries } from "./FeedScreen";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const items: FeedItemBody[] = [
  { kind: "hotel_received", seq: 1, name: "Tvoj hotel", profile: "C" },
  { kind: "plan_changed", seq: 2, month: "jun", round: "jun", plan: plan(), by: "guest" },
  {
    kind: "answer_chosen",
    seq: 3,
    month: "jun",
    round: "jun",
    situation: "S3",
    title: "Agencija",
    answer: 1,
    label: "Prihvatiti uz 15 % popusta i depozit",
    by: "guest",
  },
  { kind: "response", seq: 4, month: "jun", round: "jun", situation: "S1", title: "Klima" },
  { kind: "round_closed", seq: 5, month: "jun", round: "jun", by: "guest" },
  {
    kind: "verdict",
    seq: 6,
    month: "jun",
    round: "jun",
    situation: "S1",
    title: "Klima",
    quality: 4,
    neutral: false,
    late: false,
  },
  {
    kind: "change",
    seq: 7,
    month: "jun",
    round: "jun",
    var: "sold",
    cause: "price",
    amount: -1200,
    unit: "bp",
    situation: null,
  },
  {
    kind: "change",
    seq: 8,
    month: "jun",
    round: "jun",
    var: "cash",
    cause: "interest",
    amount: 10_000,
    unit: "cents",
    situation: null,
  },
  {
    kind: "month_resolved",
    seq: 9,
    month: "jun",
    round: "jun",
    indices: ix(5300, 5200, 5100, 5400, 4800),
    progress_hundredths: 800,
  },
  { kind: "mentor", seq: 10, month: "jun", round: "jun", text: "Usluga je bolja." },
];

describe("feedEntries", () => {
  it("najnovije prvo, uzastopne promjene istog mjeseca u jednoj stavci", () => {
    const e = feedEntries(items);
    expect(e[0]?.kind === "item" && e[0].item.kind).toBe("mentor");
    expect(e.filter((x) => x.kind === "changes")).toHaveLength(1);
    expect(e).toHaveLength(9);
    expect(e.at(-1)?.month).toBeNull();
  });
});

describe("Tijek", () => {
  it("rečenice po vrsti, mjeseci kao naslovi, promjene se otvaraju", async () => {
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/feed", { items }), "/tijek");
    expect(await screen.findByRole("heading", { name: "Tijek" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Lipanj" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Početak" })).toBeTruthy();
    expect(screen.getByText("Mentor: Usluga je bolja.")).toBeTruthy();
    const mentorRow = screen.getByText("Mentor: Usluga je bolja.").closest("li") as HTMLElement;
    expect(mentorRow.querySelector("svg[aria-hidden='true']")).not.toBeNull();
    expect(screen.getByText("Lipanj je obračunat: napredak +8 bodova.")).toBeTruthy();
    expect(screen.getByText("Klima: AI ocjena 4/5.")).toBeTruthy();
    expect(screen.getByText("Agencija: izabrano „Prihvatiti uz 15 % popusta i depozit“.")).toBeTruthy();
    expect(screen.getByText("Igra počinje: vodiš hotel „Tvoj hotel“.")).toBeTruthy();
    // Promjene obračuna su jedan redak s poveznicom na „Mjesec završen“, ne sirovi retci.
    expect(screen.queryByText(/Obračun: \d+ promjen/)).toBeNull();
    expect(screen.queryByText("−12 %")).toBeNull();
    const why = screen.getByRole("link", { name: "Pogledaj zašto su se indeksi pomaknuli" });
    expect(why.getAttribute("href")).toBe("/mjesec/jun");
    expect(why.className).toContain("min-h-11");
    expect(document.querySelector("details")).toBeNull();
    fireEvent.click(why);
    expect(window.location.pathname).toBe("/mjesec/jun");
  });

  it("prazan tijek", async () => {
    renderApp(standardApi(), "/tijek");
    expect(await screen.findByText("Još se ništa nije dogodilo.")).toBeTruthy();
  });
});

describe("Tijek, ključevi", () => {
  // Jedan događaj daje više stavki s istim `seq` (sve promjene obračuna, neodgovorene situacije zatvaranja).
  const sameSeq: FeedItemBody[] = [
    { kind: "round_closed", seq: 5, month: "jun", round: "jun", by: "guest" },
    { kind: "situation_ignored", seq: 5, month: "jun", round: "jun", situation: "S2", title: "Šef kuhinje" },
    { kind: "situation_ignored", seq: 5, month: "jun", round: "jun", situation: "S3", title: "Agencija" },
    {
      kind: "change",
      seq: 21,
      month: "jun",
      round: "jun",
      var: "sold",
      cause: "price",
      amount: -100,
      unit: "bp",
      situation: null,
    },
    {
      kind: "change",
      seq: 21,
      month: "jun",
      round: "jun",
      var: "cash",
      cause: "interest",
      amount: -100,
      unit: "cents",
      situation: null,
    },
  ];

  it("stavke istog `seq` dobivaju različite ključeve", () => {
    const keys = feedEntries(sameSeq).map((e) => e.key);
    expect(new Set(keys).size).toBe(keys.length);
  });

  it("bez upozorenja Reacta o istim ključevima", async () => {
    const err = vi.spyOn(console, "error").mockImplementation(() => {});
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/feed", { items: sameSeq }), "/tijek");
    expect(await screen.findByText("Agencija: bez odgovora (ocjena 1/5).")).toBeTruthy();
    const dup = err.mock.calls.filter((c) => c.some((a) => String(a).includes("same key")));
    err.mockRestore();
    expect(dup).toEqual([]);
    expect(screen.getAllByRole("link", { name: "Pogledaj zašto su se indeksi pomaknuli" })).toHaveLength(1);
  });
});

describe("Tijek: tjedna igra", () => {
  const weekItems: FeedItemBody[] = [
    { kind: "round_closed", seq: 1, month: "jun", round: "t01", by: "guest" },
    {
      kind: "change",
      seq: 2,
      month: "jun",
      round: "t01",
      var: "sold",
      cause: "price",
      amount: -1200,
      unit: "bp",
      situation: null,
    },
    {
      kind: "month_resolved",
      seq: 3,
      month: "jun",
      round: "t01",
      indices: ix(5300, 5200, 5100, 5400, 4800),
      progress_hundredths: 800,
    },
    { kind: "plan_changed", seq: 4, month: "jun", round: "t02", plan: plan(), by: "guest" },
  ];

  it("naslovi su tjedni s datumima, rečenice govore o tjednu, poveznica vodi na /krug/t01", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/feed", { items: weekItems }), "/tijek");
    expect(
      await screen.findByRole("heading", { name: keepRoundTogether("Tjedan 1 (1. 6. – 7. 6.)") }),
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: keepRoundTogether("Tjedan 2 (8. 6. – 14. 6.)") }),
    ).toBeTruthy();
    expect(screen.getByText(joined("Tjedan 1 (1. 6. – 7. 6.) je zatvoren."))).toBeTruthy();
    expect(
      screen.getByText(joined("Tjedan 1 (1. 6. – 7. 6.) je obračunat: napredak +8 bodova.")),
    ).toBeTruthy();
    expect(screen.getByText(joined("Plan za tjedan 2 (8. 6. – 14. 6.) je promijenjen."))).toBeTruthy();
    const why = screen.getByRole("link", { name: "Pogledaj zašto su se indeksi pomaknuli" });
    expect(why.getAttribute("href")).toBe("/krug/t01");
  });

  it("dok igra ne stigne, naslovi su neutralni „Tjedan N“, nikad mjesec", async () => {
    let release: () => void = () => {};
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const api = v2Api()
      .on("GET", "/v1/games/:game", async () => {
        await gate;
        return json(200, weeklyGame());
      })
      .on("GET", "/v1/hotels/:hotel/feed", { items: weekItems });
    renderApp(api, "/tijek");
    expect(await screen.findByRole("heading", { name: keepRoundTogether("Tjedan 1") })).toBeTruthy();
    expect(screen.getByRole("heading", { name: keepRoundTogether("Tjedan 2") })).toBeTruthy();
    expect(screen.queryByText(/lipanj/i)).toBeNull();
    release();
    expect(
      await screen.findByRole("heading", { name: keepRoundTogether("Tjedan 1 (1. 6. – 7. 6.)") }),
    ).toBeTruthy();
  });
});
