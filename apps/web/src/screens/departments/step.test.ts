import { describe, expect, it } from "vitest";
import { stepLeftForOthers } from "./step";

const plan = (targets: Record<string, number>) =>
  Object.fromEntries(
    Object.entries(targets).map(([code, seasonal_target]) => [
      code,
      { open: true, seasonal_target, price_level_pct: 100, maintenance: "regular" as const },
    ]),
  );

describe("stepLeftForOthers", () => {
  const now = { rooms: 10, restaurant: 2, lobby_bar: 3 };

  it("bez pomaka drugih odjela: cijeli korak hotela", () => {
    expect(stepLeftForOthers(plan({ rooms: 10, restaurant: 2, lobby_bar: 3 }), now, "restaurant", 5)).toBe(5);
  });

  it("pomaci ostalih odjela (u oba smjera) troše korak, vlastiti ne", () => {
    // Smještaj +2, bar −1 = 3; restoran +4 se ne broji.
    expect(stepLeftForOthers(plan({ rooms: 12, restaurant: 6, lobby_bar: 2 }), now, "restaurant", 5)).toBe(2);
  });

  it("nikad ispod nule", () => {
    expect(stepLeftForOthers(plan({ rooms: 20, restaurant: 2, lobby_bar: 3 }), now, "restaurant", 5)).toBe(0);
  });

  it("odjel kojeg nema u stanju broji se od nule", () => {
    expect(stepLeftForOthers(plan({ rooms: 10, restaurant: 2, pool_bar: 1 }), now, "restaurant", 5)).toBe(4);
  });
});
