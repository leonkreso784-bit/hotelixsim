// Čista logika igre za ekrane: indeksi, promjene, ljestvica. Bez Reacta i mreže.

import type {
  GameBody,
  IndexCode,
  IndexEntry,
  PublicHotelBody,
  RoundInfoBody,
  StrategyCode,
} from "./api/types";
import { wholePoints } from "./format";

export const INDEX_ORDER: readonly IndexCode[] = ["G", "R", "Z", "O", "F"];

export function sortIndices<T extends { code: IndexCode }>(xs: T[]): T[] {
  return [...xs].sort((a, b) => INDEX_ORDER.indexOf(a.code) - INDEX_ORDER.indexOf(b.code));
}

/** Krug igre po kodu (u v1 jednak kodu mjeseca); null ako ga igra nema. */
export function roundInfo(game: GameBody, code: string): RoundInfoBody | null {
  return game.rounds.find((r) => r.code === code) ?? null;
}

export function isV2(game: GameBody): boolean {
  return game.model === "v2";
}

/** Igra po tjednima (čip, tekstovi i gumb govore o tjednu); v2 po mjesecima se prikazuje kao v1. */
export function isWeekly(game: GameBody): boolean {
  return game.round_kind === "week";
}

/** Obračun hotela za krug po kodu kruga. */
function scoreAt(h: PublicHotelBody, code: string) {
  return h.months.find((m) => m.round === code);
}

export function indicesAt(h: PublicHotelBody, code: string): IndexEntry[] | null {
  return scoreAt(h, code)?.indices ?? null;
}

/** Zadnji obračunati mjesec, a prije prvog obračuna početno stanje. */
export function latestIndices(h: PublicHotelBody): IndexEntry[] {
  return h.months.at(-1)?.indices ?? h.start;
}

/** Indeksi prije zadanog kruga: prethodni krug igre ili početak. */
export function indicesBefore(h: PublicHotelBody, codes: string[], code: string): IndexEntry[] {
  const i = codes.indexOf(code);
  const prev = i > 0 ? codes[i - 1] : undefined;
  return (prev && indicesAt(h, prev)) || h.start;
}

/** Promjena po indeksu u cijelim bodovima: razlika zaokruženih vrijednosti (48,76 → 42,69 = 49 → 43 = −6). */
export function indexDeltas(from: IndexEntry[], to: IndexEntry[]): Map<IndexCode, number> {
  const before = new Map(from.map((e) => [e.code, wholePoints(e.hundredths)]));
  return new Map(
    to.map((e) => [e.code, wholePoints(e.hundredths) - (before.get(e.code) ?? wholePoints(e.hundredths))]),
  );
}

export function progressAt(h: PublicHotelBody, code: string): number | null {
  return scoreAt(h, code)?.progress_hundredths ?? null;
}

export type LeaderRow = {
  hotelId: string;
  name: string;
  mine: boolean;
  strategy: StrategyCode | null;
  progress: number;
};

/** Ljestvica napretka za krug: samo obračunati hoteli, najbolji prvi, jednaki po mjestu. */
export function leaderboard(game: GameBody, code: string): LeaderRow[] {
  const rows: (LeaderRow & { slot: number })[] = [];
  for (const h of game.hotels) {
    const progress = progressAt(h, code);
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

/** Trenutni krug, njegov redni broj (od 0) i ukupan broj krugova; null kad je sezona završena. */
export function roundPosition(game: GameBody): { round: RoundInfoBody; index: number; total: number } | null {
  if (!game.current) return null;
  const round = game.current.round;
  const index = game.rounds.findIndex((r) => r.code === round.code);
  return { round, index: index < 0 ? round.index : index, total: game.rounds.length };
}

export function nextRound(codes: string[], code: string): string | null {
  return codes[codes.indexOf(code) + 1] ?? null;
}
