import { describe, expect, it } from "vitest";
import type { GameBody, IndexEntry, PublicHotelBody } from "./api/types";
import {
  indexDeltas,
  indicesBefore,
  isV2,
  latestIndices,
  leaderboard,
  nextRound,
  progressAt,
  roundInfo as roundOf,
  roundPosition,
  sortIndices,
} from "./game";
import { roundInfo, weeklyGame } from "./test/fixtures";

const ix = (g: number, r: number, z: number, o: number, f: number): IndexEntry[] => [
  { code: "G", hundredths: g },
  { code: "R", hundredths: r },
  { code: "Z", hundredths: z },
  { code: "O", hundredths: o },
  { code: "F", hundredths: f },
];

function hotel(id: string, slot: number, months: PublicHotelBody["months"]): PublicHotelBody {
  return {
    hotel_id: id,
    slot,
    name: id === "h1" ? "Tvoj hotel" : `Robot ${slot}`,
    seat: id === "h1" ? { kind: "human", strategy: null } : { kind: "robot", strategy: "stedisa" },
    start: ix(5000, 5000, 5000, 5000, 5000),
    months,
  };
}

const jun = {
  month: "jun" as const,
  round: "jun",
  indices: ix(5300, 5200, 5100, 5400, 4800),
  progress_hundredths: 800,
};
const jul = {
  month: "jul" as const,
  round: "jul",
  indices: ix(5600, 5200, 5000, 5400, 5000),
  progress_hundredths: 1100,
};

describe("indeksi", () => {
  it("sortIndices slaže G, R, Z, O, F", () => {
    const shuffled: IndexEntry[] = [
      { code: "F", hundredths: 1 },
      { code: "G", hundredths: 2 },
      { code: "O", hundredths: 3 },
      { code: "R", hundredths: 4 },
      { code: "Z", hundredths: 5 },
    ];
    expect(sortIndices(shuffled).map((i) => i.code)).toEqual(["G", "R", "Z", "O", "F"]);
  });
  it("latestIndices je zadnji obračunati mjesec ili početak", () => {
    expect(latestIndices(hotel("h1", 1, []))).toEqual(ix(5000, 5000, 5000, 5000, 5000));
    expect(latestIndices(hotel("h1", 1, [jun, jul]))).toEqual(jul.indices);
  });
  it("indicesBefore je prethodni mjesec, a za prvi mjesec početak", () => {
    const h = hotel("h1", 1, [jun, jul]);
    expect(indicesBefore(h, ["jun", "jul"], "jun")).toEqual(h.start);
    expect(indicesBefore(h, ["jun", "jul"], "jul")).toEqual(jun.indices);
  });
  it("indexDeltas daje razliku po kodu", () => {
    const d = indexDeltas(ix(5000, 5000, 5000, 5000, 5000), jun.indices);
    expect(d.get("G")).toBe(3);
    expect(d.get("F")).toBe(-2);
  });
  it("indexDeltas je razlika zaokruženih vrijednosti u cijelim bodovima (4876 → 4269 = −6)", () => {
    const d = indexDeltas(ix(4876, 4850, 0, 4849, 10000), ix(4269, 4849, 10000, 4850, 10000));
    expect(d.get("G")).toBe(-6);
    expect(d.get("R")).toBe(-1);
    expect(d.get("Z")).toBe(100);
    expect(d.get("O")).toBe(1);
    expect(d.get("F")).toBe(0);
  });
});

describe("napredak i ljestvica", () => {
  const game: GameBody = {
    game_id: "g1",
    model: "v1",
    round_kind: "month",
    rounds: [roundInfo("jun"), roundInfo("jul")],
    months: ["jun", "jul"],
    current: { month: "jul", round: roundInfo("jul"), status: "open", unanswered: 1 },
    my_hotel_id: "h1",
    hotels: [
      hotel("h1", 1, [jun]),
      hotel("h2", 2, [{ ...jun, progress_hundredths: 600 }]),
      hotel("h3", 3, [{ ...jun, progress_hundredths: 800 }]),
      hotel("h4", 4, []),
    ],
  };
  it("progressAt vraća null za neobračunat mjesec", () => {
    expect(progressAt(game.hotels[0] as PublicHotelBody, "jun")).toBe(800);
    expect(progressAt(game.hotels[3] as PublicHotelBody, "jun")).toBeNull();
  });
  it("leaderboard slaže po napretku, jednake po mjestu, i označi moj hotel", () => {
    const rows = leaderboard(game, "jun");
    expect(rows.map((r) => r.hotelId)).toEqual(["h1", "h3", "h2"]);
    expect(rows[0]?.mine).toBe(true);
    expect(rows[1]?.strategy).toBe("stedisa");
  });
  it("roundPosition daje krug, redni broj i ukupno; null na kraju sezone", () => {
    expect(roundPosition(game)).toEqual({ round: roundInfo("jul"), index: 1, total: 2 });
    expect(roundPosition({ ...game, current: null })).toBeNull();
    expect(roundPosition(weeklyGame({ week: 5 }))?.index).toBe(4);
    expect(roundPosition(weeklyGame({ week: 5 }))?.total).toBe(17);
  });
  it("nextRound daje sljedeći krug igre ili null", () => {
    expect(nextRound(["jun", "jul"], "jun")).toBe("jul");
    expect(nextRound(["jun", "jul"], "jul")).toBeNull();
  });
  it("roundInfo traži krug po kodu, a isV2 čita model", () => {
    expect(roundOf(game, "jul")).toEqual(roundInfo("jul"));
    expect(roundOf(game, "t01")).toBeNull();
    expect(isV2(game)).toBe(false);
    expect(isV2(weeklyGame())).toBe(true);
  });
  it("ljestvica tjedne igre čita kod kruga", () => {
    const wk = (p: number) => ({
      month: "jun" as const,
      round: "t01",
      indices: jun.indices,
      progress_hundredths: p,
    });
    const g = weeklyGame({ hotels: [hotel("h1", 1, [wk(300)]), hotel("h2", 2, [wk(900)])] });
    expect(leaderboard(g, "t01").map((r) => r.hotelId)).toEqual(["h2", "h1"]);
    expect(leaderboard(g, "t02")).toEqual([]);
  });
});
