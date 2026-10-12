import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { clearDrafts, differs, readDraft, roundDrafts, useDraft, useStaleDrafts, writeDraft } from "./drafts";

afterEach(() => {
  vi.restoreAllMocks();
  clearDrafts();
});

describe("nacrti obrazaca", () => {
  it("izmjene se spremaju i čitaju po ključu; prazne izmjene brišu zapis", () => {
    writeDraft("plan:h1:t01", { price: "150" });
    expect(readDraft("plan:h1:t01")).toEqual({ price: "150" });
    expect(readDraft("plan:h1:t02")).toEqual({});
    writeDraft("plan:h1:t01", {});
    expect(readDraft("plan:h1:t01")).toEqual({});
  });

  it("neispravan zapis u spremištu se zanemaruje", () => {
    window.sessionStorage.setItem("hotelixsim:nacrt:plan:h1:t01", "{nije json");
    expect(readDraft("plan:h1:t01")).toEqual({});
    window.sessionStorage.setItem("hotelixsim:nacrt:plan:h1:t01", "[1,2]");
    expect(readDraft("plan:h1:t01")).toEqual({});
  });

  it("nacrt iz prošlog osvježavanja stranice čita se iz spremišta", () => {
    window.sessionStorage.setItem("hotelixsim:nacrt:plan:h1:t01", '{"price":"150"}');
    expect(readDraft("plan:h1:t01")).toEqual({ price: "150" });
  });

  it("blokirano spremište: izmjene žive u memoriji", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    writeDraft("dept:h1:t01:restaurant", { seasonal_target: 4 });
    expect(readDraft("dept:h1:t01:restaurant")).toEqual({ seasonal_target: 4 });
  });

  it("memorija je najnovija: kvota odbije novi upis ili brisanje, a čita se zadnje stanje (minor 1)", () => {
    writeDraft("plan:h1:t01", { price: "150" });
    const set = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });
    writeDraft("plan:h1:t01", { price: "160" });
    expect(readDraft("plan:h1:t01")).toEqual({ price: "160" });
    set.mockRestore();
    vi.spyOn(Storage.prototype, "removeItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    writeDraft("plan:h1:t01", null);
    expect(readDraft("plan:h1:t01")).toEqual({});
  });

  it("nacrti kruga: plan i odjeli s izmjenama", () => {
    writeDraft("plan:h1:t01", { price: "150" });
    writeDraft("dept:h1:t01:restaurant", { seasonal_target: 4 });
    writeDraft("dept:h1:t02:spa", { open: false });
    writeDraft("dept:h2:t01:spa", { open: false });
    expect(roundDrafts("h1", "t01")).toEqual({ plan: true, depts: ["restaurant"] });
    expect(roundDrafts("h1", "t03")).toEqual({ plan: false, depts: [] });
  });

  it("razlika prema planu gleda samo dirana polja", () => {
    expect(differs({ a: 1, b: 2 }, { a: 1 })).toBe(false);
    expect(differs({ a: 1, b: 2 }, { b: 3 })).toBe(true);
    expect(differs({ a: 1 }, {})).toBe(false);
  });
});

describe("useDraft", () => {
  const base = { price: "140", marketing: "0" };

  it("polje vraćeno na vrijednost iz plana nije izmjena (minor 2)", () => {
    const { result } = renderHook(() => useDraft("plan:h1:t01", base));
    act(() => result.current.update({ price: "150" }));
    expect(result.current.changes).toEqual({ price: "150" });
    act(() => result.current.update({ price: "140" }));
    expect(result.current.changes).toEqual({});
    expect(readDraft("plan:h1:t01")).toEqual({});
  });

  it("nakon spremanja ostaje ono što je upisano dok je zahtjev trajao (minor 8)", () => {
    const { result } = renderHook(() => useDraft("plan:h1:t01", base));
    act(() => result.current.update({ price: "150" }));
    const sent = result.current.changes;
    act(() => result.current.update({ marketing: "500" }));
    act(() => result.current.settle(sent));
    expect(result.current.changes).toEqual({ marketing: "500" });
  });

  it("isto polje promijenjeno tijekom spremanja ostaje izmjena", () => {
    const { result } = renderHook(() => useDraft("plan:h1:t01", base));
    act(() => result.current.update({ price: "150" }));
    const sent = result.current.changes;
    act(() => result.current.update({ price: "155" }));
    act(() => result.current.settle(sent));
    expect(result.current.changes).toEqual({ price: "155" });
  });
});

describe("useStaleDrafts (minor 3)", () => {
  it("nacrt prošlog kruga: rečenica i brisanje; nacrt ovog kruga i drugog hotela ostaju", () => {
    writeDraft("plan:h1:t01", { price: "150" });
    writeDraft("plan:h1:t02", { price: "160" });
    writeDraft("plan:h2:t01", { price: "170" });
    const { result } = renderHook(() => useStaleDrafts("plan", "h1", "t02"));
    expect(result.current).toBe(true);
    expect(readDraft("plan:h1:t01")).toEqual({});
    expect(readDraft("plan:h1:t02")).toEqual({ price: "160" });
    expect(readDraft("plan:h2:t01")).toEqual({ price: "170" });
  });

  it("bez starih nacrta ili bez otvorenog kruga: ništa", () => {
    writeDraft("plan:h1:t02", { price: "160" });
    expect(renderHook(() => useStaleDrafts("plan", "h1", "t02")).result.current).toBe(false);
    writeDraft("plan:h1:t01", { price: "150" });
    expect(renderHook(() => useStaleDrafts("plan", "h1", null)).result.current).toBe(false);
  });
});
