// Čista logika igre za ekrane: indeksi, promjene, ljestvica. Bez Reacta i mreže.

import type { GameBody, IndexCode, IndexEntry, MonthCode, PublicHotelBody, StrategyCode } from "./api/types";
import { wholePoints } from "./format";

export const INDEX_ORDER: readonly IndexCode[] = ["G", "R", "Z", "O", "F"];

export function sortIndices<T extends { code: IndexCode }>(xs: T[]): T[] {
  return [...xs].sort((a, b) => INDEX_ORDER.indexOf(a.code) - INDEX_ORDER.indexOf(b.code));
}

export function indicesAt(h: PublicHotelBody, month: MonthCode): IndexEntry[] | null {
  return h.months.find((m) => m.month === month)?.indices ?? null;
}

/** Zadnji obračunati mjesec, a prije prvog obračuna početno stanje. */
export function latestIndices(h: PublicHotelBody): IndexEntry[] {
  return h.months.at(-1)?.indices ?? h.start;
}

/** Indeksi prije zadanog mjeseca: prethodni mjesec igre ili početak. */
export function indicesBefore(h: PublicHotelBody, months: MonthCode[], month: MonthCode): IndexEntry[] {
  const i = months.indexOf(month);
  const prev = i > 0 ? months[i - 1] : undefined;
  return (prev && indicesAt(h, prev)) || h.start;
}

/** Promjena po indeksu u cijelim bodovima: razlika zaokruženih vrijednosti (48,76 → 42,69 = 49 → 43 = −6). */
export function indexDeltas(from: IndexEntry[], to: IndexEntry[]): Map<IndexCode, number> {
  const before = new Map(from.map((e) => [e.code, wholePoints(e.hundredths)]));
  return new Map(
    to.map((e) => [e.code, wholePoints(e.hundredths) - (before.get(e.code) ?? wholePoints(e.hundredths))]),
  );
}

export function progressAt(h: PublicHotelBody, month: MonthCode): number | null {
  return h.months.find((m) => m.month === month)?.progress_hundredths ?? null;
}

export type LeaderRow = {
  hotelId: string;
  name: string;
  mine: boolean;
  strategy: StrategyCode | null;
  progress: number;
};

/** Ljestvica napretka za mjesec: samo obračunati hoteli, najbolji prvi, jednaki po mjestu. */
export function leaderboard(game: GameBody, month: MonthCode): LeaderRow[] {
  const rows: (LeaderRow & { slot: number })[] = [];
  for (const h of game.hotels) {
    const progress = progressAt(h, month);
    if (progress === null) continue;
    rows.push({
      hotelId: h.hotel_id,
      name: h.name,
      mine: h.hotel_id === game.my_hotel_id,
      strategy: h.seat.strategy,
      progress,
      slot: h.slot,
    });
  }
  rows.sort((a, b) => b.progress - a.progress || a.slot - b.slot);
  return rows.map(({ slot: _slot, ...row }) => row);
}

export function monthPosition(game: GameBody): { month: MonthCode; index: number; total: number } | null {
  if (!game.current) return null;
  return {
    month: game.current.month,
    index: game.months.indexOf(game.current.month),
    total: game.months.length,
  };
}

export function nextMonth(months: MonthCode[], month: MonthCode): MonthCode | null {
  return months[months.indexOf(month) + 1] ?? null;
}
