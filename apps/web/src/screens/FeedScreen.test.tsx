import { cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { ix, plan, standardApi } from "../test/fixtures";
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
