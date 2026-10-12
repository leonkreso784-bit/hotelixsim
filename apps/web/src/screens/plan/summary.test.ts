import { describe, expect, it } from "vitest";
import type { DepartmentBody } from "../../api/types";
import { departmentsBody } from "../../test/fixtures";
import { deptChanges } from "./summary";

type DeptPlan = {
  open: boolean;
  seasonal_target: number;
  price_level_pct: number;
  maintenance: "deferred" | "regular" | "intensive";
};

const now: DepartmentBody[] = departmentsBody().departments;
const same = (): Record<string, DeptPlan> =>
  Object.fromEntries(
    now.map((d) => [
      d.code,
      { open: d.open, seasonal_target: d.seasonal, price_level_pct: 100, maintenance: "regular" as const },
    ]),
  );

describe("deptChanges", () => {
  it("bez promjena: prazan popis", () => {
    expect(deptChanges(same(), now)).toEqual([]);
  });

  it("sezonski radnici, cijene i održavanje u jednoj rečenici po odjelu, redom popisa", () => {
    const p = same();
    p.restaurant = { open: true, seasonal_target: 4, price_level_pct: 105, maintenance: "regular" };
    p.rooms = { open: true, seasonal_target: 9, price_level_pct: 100, maintenance: "intensive" };
    expect(deptChanges(p, now)).toEqual([
      { code: "rooms", name: "Smještaj", text: "−1 sezonski radnik, održavanje pojačano" },
      { code: "restaurant", name: "Restoran", text: "+2 sezonska radnika, cijene 105\u00a0%" },
    ]);
  });

  it("zatvaranje i ponovno otvaranje", () => {
    const p = same();
    p.lobby_bar = { open: false, seasonal_target: 0, price_level_pct: 100, maintenance: "regular" };
    p.parking = { open: true, seasonal_target: 0, price_level_pct: 100, maintenance: "regular" };
    expect(deptChanges(p, now)).toEqual([
      { code: "lobby_bar", name: "Lobby bar", text: "zatvara se" },
      { code: "parking", name: "Parking", text: "ponovno se otvara" },
    ]);
  });

  it("službe se ne navode", () => {
    const p = same();
    p.admin = { open: true, seasonal_target: 5, price_level_pct: 100, maintenance: "regular" };
    expect(deptChanges(p, now)).toEqual([]);
  });
});
