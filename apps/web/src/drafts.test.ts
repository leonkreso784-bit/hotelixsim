import { afterEach, describe, expect, it, vi } from "vitest";
import { clearDrafts, differs, readDraft, writeDraft } from "./drafts";

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

  it("razlika prema planu gleda samo dirana polja", () => {
    expect(differs({ a: 1, b: 2 }, { a: 1 })).toBe(false);
    expect(differs({ a: 1, b: 2 }, { b: 3 })).toBe(true);
    expect(differs({ a: 1 }, {})).toBe(false);
  });
});
