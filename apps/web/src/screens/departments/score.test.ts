import { describe, expect, it } from "vitest";
import type { DepartmentBody } from "../../api/types";
import { departmentsBody } from "../../test/fixtures";
import { deptScore, SCORE_LEVELS, scoreLevel } from "./score";

function restaurant(over: Partial<DepartmentBody> = {}): DepartmentBody {
  const base = departmentsBody().departments.find((d) => d.code === "restaurant");
  if (!base?.last) throw new Error("restoran bez kruga");
  return { ...base, ...over };
}

describe("deptScore", () => {
  it("prosjek kvalitete usluge, stanja prostora i zadovoljstva gostiju", () => {
    const base = restaurant();
    const d = restaurant({
      condition_pts: 6000,
      guest_satisfaction_pts: 8000,
      last: base.last ? { ...base.last, quality_pts: 7000 } : null,
    });
    expect(deptScore(d)).toEqual({ points: 7000, quality: 7000, condition: 6000, satisfaction: 8000 });
  });

  it("zaokružuje prosjek na cijeli bod (0–10 000)", () => {
    const base = restaurant();
    const d = restaurant({
      condition_pts: 6901,
      guest_satisfaction_pts: 6949,
      last: base.last ? { ...base.last, quality_pts: 7000 } : null,
    });
    // (7000 + 6901 + 6949) / 3 = 6950
    expect(deptScore(d)?.points).toBe(6950);
  });

  it("bez zadovoljstva gostiju: prosjek kvalitete i stanja prostora", () => {
    const base = restaurant();
    const d = restaurant({
      condition_pts: 5000,
      guest_satisfaction_pts: null,
      last: base.last ? { ...base.last, quality_pts: 7000 } : null,
    });
    expect(deptScore(d)).toEqual({ points: 6000, quality: 7000, condition: 5000, satisfaction: null });
  });

  it("bez obračunatog kruga: prosjek stanja prostora i zadovoljstva", () => {
    const d = restaurant({ condition_pts: 6000, guest_satisfaction_pts: 7000, last: null });
    expect(deptScore(d)).toEqual({ points: 6500, quality: null, condition: 6000, satisfaction: 7000 });
  });

  it("bez kruga i bez zadovoljstva: samo stanje prostora", () => {
    const d = restaurant({ condition_pts: 4200, guest_satisfaction_pts: null, last: null });
    expect(deptScore(d)?.points).toBe(4200);
  });

  it("službe nemaju ocjenu", () => {
    const admin = departmentsBody().departments.find((d) => d.code === "admin");
    expect(admin && deptScore(admin)).toBeNull();
  });
});

describe("scoreLevel", () => {
  it("pragovi po prikazanom cijelom broju: ≥ 70 dobro, 50–69 srednje, < 50 slabo", () => {
    expect(scoreLevel(7000)).toBe("good");
    expect(scoreLevel(6950)).toBe("good"); // prikaže se 70
    expect(scoreLevel(6949)).toBe("mid"); // prikaže se 69
    expect(scoreLevel(5000)).toBe("mid");
    expect(scoreLevel(4950)).toBe("mid"); // prikaže se 50
    expect(scoreLevel(4949)).toBe("weak");
    expect(scoreLevel(0)).toBe("weak");
    expect(scoreLevel(10_000)).toBe("good");
  });

  it("svaka razina ima riječ i boju iz tokena", () => {
    expect(SCORE_LEVELS.good).toEqual({ word: "dobro", bar: "bg-up" });
    expect(SCORE_LEVELS.mid).toEqual({ word: "srednje", bar: "bg-accent" });
    expect(SCORE_LEVELS.weak).toEqual({ word: "slabo", bar: "bg-down" });
  });
});
