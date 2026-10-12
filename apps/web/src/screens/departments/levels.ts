// Pragovi razina 0–100 po prikazanom cijelom broju (Leonova odluka 13.10.): ≥ 80 dobro, 60–79 srednje, < 60 slabo.
// Jedini izvor za boju ocjene odjela (`score.ts`) i za rečenice o stanju prostora (`insights.ts`), da se tekst i boja
// ne razilaze (UI N-I3). Bez Reacta.
export type ScoreLevel = "good" | "mid" | "weak";

export const LEVEL_GOOD_FROM = 80;
export const LEVEL_MID_FROM = 60;

/** Razina prikazanog cijelog broja 0–100. */
export function levelOf(shown: number): ScoreLevel {
  return shown >= LEVEL_GOOD_FROM ? "good" : shown >= LEVEL_MID_FROM ? "mid" : "weak";
}
