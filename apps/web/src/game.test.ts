import { describe, expect, it } from "vitest";
import type { GameBody, IndexEntry, PublicHotelBody } from "./api/types";
import {
  indexDeltas,
  indicesBefore,
  latestIndices,
  leaderboard,
  monthPosition,
  nextMonth,
  progressAt,
  sortIndices,
} from "./game";

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

const jun = { month: "jun" as const, indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 };
const jul = { month: "jul" as const, indices: ix(5600, 5200, 5000, 5400, 5000), progress_hundredths: 1100 };

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
    expect(d.get("G")).toBe(300);
    expect(d.get("F")).toBe(-200);
  });
});

describe("napredak i ljestvica", () => {
  const game: GameBody = {
    game_id: "g1",
    months: ["jun", "jul"],
    current: { month: "jul", status: "open", unanswered: 1 },
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
  it("monthPosition daje mjesec, redni broj i ukupno; null na kraju sezone", () => {
    expect(monthPosition(game)).toEqual({ month: "jul", index: 1, total: 2 });
    expect(monthPosition({ ...game, current: null })).toBeNull();
  });
  it("nextMonth daje sljedeći mjesec igre ili null", () => {
    expect(nextMonth(["jun", "jul"], "jun")).toBe("jul");
    expect(nextMonth(["jun", "jul"], "jul")).toBeNull();
  });
});
