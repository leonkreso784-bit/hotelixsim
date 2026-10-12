import { describe, expect, it } from "vitest";
import { anyDecision, decisionsIntro } from "./decisionsIntro";

const all = { close: true, seasonal: true, price: true, upkeep: true, lease: false };
const STEP = "Broj sezonskih radnika smije se ukupno promijeniti samo malo odjednom, za cijeli hotel.";

describe("uvod u odluke nabraja samo ponuđeno (UI krug 3, M-4)", () => {
  it("odjel koji se smije zatvoriti", () => {
    expect(decisionsIntro(all)).toBe(
      `Odredi je li odjel otvoren, koliko sezonskih radnika ima, razinu cijena i održavanje. ${STEP}`,
    );
  });
  it("Restoran (ne zatvara se) i Smještaj (bez razine cijena)", () => {
    expect(decisionsIntro({ ...all, close: false })).toBe(
      `Odredi koliko sezonskih radnika ima, razinu cijena i održavanje. ${STEP}`,
    );
    expect(decisionsIntro({ ...all, close: false, price: false })).toBe(
      `Odredi koliko sezonskih radnika ima i održavanje. ${STEP}`,
    );
  });
  it("najam ima svoj uvod; bez ijedne odluke nema ni uvoda", () => {
    expect(decisionsIntro({ ...all, lease: true })).toMatch(/^Najam ima ugovoreni prihod/);
    expect(anyDecision({ close: false, seasonal: false, price: false, upkeep: false, lease: true })).toBe(
      false,
    );
    expect(anyDecision({ close: false, seasonal: false, price: false, upkeep: true, lease: true })).toBe(
      true,
    );
  });
});
