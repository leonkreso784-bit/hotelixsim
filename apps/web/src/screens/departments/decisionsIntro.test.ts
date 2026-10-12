import { describe, expect, it } from "vitest";
import { anyDecision, decisionsIntro } from "./decisionsIntro";

const all = { close: true, seasonal: true, price: true, upkeep: true, renewal: false, lease: false };
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
  it("najam bez cijene (lokal) ima svoj uvod; bez ijedne odluke nema ni uvoda", () => {
    const none = { close: false, seasonal: false, price: false, upkeep: false, renewal: false, lease: true };
    expect(decisionsIntro({ ...none, upkeep: true })).toMatch(/^Najam ima ugovoreni prihod/);
    expect(anyDecision(none)).toBe(false);
    expect(anyDecision({ ...none, upkeep: true })).toBe(true);
    expect(anyDecision({ ...none, renewal: true })).toBe(true);
  });
  it("parking (najam s cijenom) nabraja cijenu, održavanje i obnovu", () => {
    expect(
      decisionsIntro({ close: true, seasonal: false, price: true, upkeep: true, renewal: true, lease: true }),
    ).toBe("Odredi je li odjel otvoren, razinu cijena, održavanje i koliko se ulaže u obnovu opreme.");
  });
});
