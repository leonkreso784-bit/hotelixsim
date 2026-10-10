import { cleanup, fireEvent, screen, within } from "@testing-library/react";
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
  { kind: "plan_changed", seq: 2, month: "jun", plan: plan(), by: "guest" },
  {
    kind: "answer_chosen",
    seq: 3,
    month: "jun",
    situation: "S3",
    title: "Agencija",
    answer: 1,
    label: "Prihvatiti uz 15 % popusta i depozit",
    by: "guest",
  },
  { kind: "response", seq: 4, month: "jun", situation: "S1", title: "Klima" },
  { kind: "round_closed", seq: 5, month: "jun", by: "guest" },
  {
    kind: "verdict",
    seq: 6,
    month: "jun",
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
    indices: ix(5300, 5200, 5100, 5400, 4800),
    progress_hundredths: 800,
  },
  { kind: "mentor", seq: 10, month: "jun", text: "Usluga je bolja." },
];

describe("feedEntries", () => {
  it("najnovije prvo, uzastopne promjene istog mjeseca u jednoj stavci", () => {
    const e = feedEntries(items);
    expect(e[0]?.kind === "item" && e[0].item.kind).toBe("mentor");
    const changes = e.find((x) => x.kind === "changes");
    expect(changes?.kind === "changes" && changes.changes).toHaveLength(2);
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
    expect(screen.getByText("Dobio si hotel „Tvoj hotel“.")).toBeTruthy();
    const sum = screen.getByText("Obračun: 2 promjene s uzrokom");
    fireEvent.click(sum);
    const box = sum.closest("details") as HTMLElement;
    expect(within(box).getByText("−12 %")).toBeTruthy();
  });

  it("prazan tijek", async () => {
    renderApp(standardApi(), "/tijek");
    expect(await screen.findByText("Još se ništa nije dogodilo.")).toBeTruthy();
  });
});

describe("Tijek, množina promjena", () => {
  const change = (seq: number): FeedItemBody => ({
    kind: "change",
    seq,
    month: "jun",
    var: "sold",
    cause: "price",
    amount: -100,
    unit: "bp",
    situation: null,
  });
  it.each([
    [1, "Obračun: 1 promjena s uzrokom"],
    [5, "Obračun: 5 promjena s uzrokom"],
  ])("%i promjena", async (n, text) => {
    const list = Array.from({ length: n }, (_, k) => change(k + 1));
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/feed", { items: list }), "/tijek");
    expect(await screen.findByText(text)).toBeTruthy();
  });
});
