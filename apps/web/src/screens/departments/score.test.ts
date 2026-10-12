import { describe, expect, it } from "vitest";
import type { DepartmentBody, DeptRoundBody } from "../../api/types";
import { departmentsBody } from "../../test/fixtures";
import { deptScore, SCORE_LEVELS, scoreLevel, scoreParts, staffFit } from "./score";

function restaurant(over: Partial<DepartmentBody> = {}, last: Partial<DeptRoundBody> = {}): DepartmentBody {
  const base = departmentsBody().departments.find((d) => d.code === "restaurant");
  if (!base?.last) throw new Error("restoran bez kruga");
  return { ...base, ...over, last: { ...base.last, load_hundredths: 144, ...last } };
}

const round = (load_hundredths: number, served = 2000, demand = 2000) =>
  restaurant({}, { load_hundredths, served, demand }).last as DeptRoundBody;

describe("staffFit: usklađenost broja ljudi s poslom (🟨)", () => {
  it("zona bez kazne (60–100 % granice za dobru uslugu) je 100", () => {
    expect(staffFit(round(108))).toBe(10_000); // 60 %
    expect(staffFit(round(144))).toBe(10_000); // 80 %
    expect(staffFit(round(180))).toBe(10_000); // 100 %
  });
  it("premalo ljudi: iznad 180 pada kao kazna opterećenja u jezgri, na 0 pri 240 (puni kapacitet)", () => {
    expect(staffFit(round(181))).toBe(9833);
    expect(staffFit(round(210))).toBe(5000);
    expect(staffFit(round(240))).toBe(0);
    expect(staffFit(round(300))).toBe(0); // odjel bez ljudi s gostima
  });
  it("previše ljudi: ispod 60 % granice pada prema 0 pri praznom odjelu", () => {
    expect(staffFit(round(107))).toBe(9907);
    expect(staffFit(round(54))).toBe(5000);
    expect(staffFit(round(0, 0, 10))).toBe(0);
  });
  it("odjel bez posla (najam, zatvoren, bez potražnje): nema usklađenosti", () => {
    expect(staffFit(round(0, 0, 0))).toBeNull();
  });
});

describe("deptScore", () => {
  it("prosjek stanja prostora, zadovoljstva gostiju i usklađenosti ljudi; kvaliteta hotela ne ulazi", () => {
    const d = restaurant({ condition_pts: 6000, guest_satisfaction_pts: 8000 }, { quality_pts: 1000 });
    expect(deptScore(d)).toEqual({ points: 8000, condition: 6000, satisfaction: 8000, staffFit: 10_000 });
  });

  it("premalo ljudi spušta ocjenu", () => {
    const d = restaurant({ condition_pts: 6000, guest_satisfaction_pts: 8000 }, { load_hundredths: 210 });
    // (6000 + 8000 + 5000) / 3 = 6333
    expect(deptScore(d)?.points).toBe(6333);
  });

  it("previše ljudi spušta ocjenu", () => {
    const d = restaurant({ condition_pts: 6000, guest_satisfaction_pts: 8000 }, { load_hundredths: 54 });
    expect(deptScore(d)?.points).toBe(6333);
  });

  it("bez zadovoljstva gostiju: prosjek stanja prostora i usklađenosti", () => {
    const d = restaurant({ condition_pts: 5000, guest_satisfaction_pts: null });
    expect(deptScore(d)).toEqual({ points: 7500, condition: 5000, satisfaction: null, staffFit: 10_000 });
  });

  it("bez obračunatog kruga: prosjek stanja prostora i zadovoljstva", () => {
    const d = { ...restaurant({ condition_pts: 6000, guest_satisfaction_pts: 7000 }), last: null };
    expect(deptScore(d)).toEqual({ points: 6500, condition: 6000, satisfaction: 7000, staffFit: null });
  });

  it("najam (parking, lokal): samo stanje prostora", () => {
    const d = restaurant(
      { class: "rentals", open: true, condition_pts: 4200, guest_satisfaction_pts: 7000 },
      { served: 0, demand: 0, load_hundredths: 0 },
    );
    expect(deptScore(d)).toEqual({ points: 4200, condition: 4200, satisfaction: null, staffFit: null });
  });

  it("zatvoren odjel i službe nemaju ocjenu", () => {
    expect(deptScore(restaurant({ open: false }))).toBeNull();
    const admin = departmentsBody().departments.find((d) => d.code === "admin");
    expect(admin && deptScore(admin)).toBeNull();
  });
});

describe("scoreParts: napomena nabraja samo prikazane sastavnice (M1)", () => {
  it("sve tri, dvije i jedna", () => {
    expect(scoreParts({ points: 1, condition: 1, satisfaction: 1, staffFit: 1 })).toBe(
      "Prosjek stanja prostora, zadovoljstva gostiju i usklađenosti ljudi s poslom.",
    );
    expect(scoreParts({ points: 1, condition: 1, satisfaction: 1, staffFit: null })).toBe(
      "Prosjek stanja prostora i zadovoljstva gostiju.",
    );
    expect(scoreParts({ points: 1, condition: 1, satisfaction: null, staffFit: null })).toBe(
      "Ocjena je stanje prostora.",
    );
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

  it("svaka razina ima riječ i svoju boju iz tokena (srednje je jantarno)", () => {
    expect(SCORE_LEVELS.good).toEqual({ word: "dobro", bar: "bg-up" });
    expect(SCORE_LEVELS.mid).toEqual({ word: "srednje", bar: "bg-warn" });
    expect(SCORE_LEVELS.weak).toEqual({ word: "slabo", bar: "bg-down" });
  });
});
