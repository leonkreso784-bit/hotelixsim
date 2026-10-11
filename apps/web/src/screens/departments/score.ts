// Ocjena odjela 0–100 (🟨 pretpostavka kontrolora, Leonova odluka 12.10.: jedna traka po odjelu, boja uvijek uz broj
// i riječ). Ocjena = prosjek onoga što odjel ima od triju sastavnica: kvaliteta usluge (`quality_pts` zadnjeg kruga),
// stanje prostora i zadovoljstvo gostiju. Službe (bez gostiju) nemaju ocjenu. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { wholePoints } from "../../format";
import { isService } from "./helpers";

export type DeptScore = {
  /** Ocjena u bodovima 0–10 000 (na ekranu 0–100). */
  points: number;
  quality: number | null;
  condition: number;
  satisfaction: number | null;
};

export type ScoreLevel = "good" | "mid" | "weak";

/** Pragovi boje po prikazanom cijelom broju (🟨): ≥ 70 dobro, 50–69 srednje, < 50 slabo. */
const GOOD_FROM = 70;
const MID_FROM = 50;

/** Riječ i boja trake po razini; boje su tokeni iz `styles.css` (UI_PRAVILA §1.1), riječ stoji uz boju (§6.8). */
export const SCORE_LEVELS: Record<ScoreLevel, { word: string; bar: string }> = {
  good: { word: t("dept.score.good"), bar: "bg-up" },
  mid: { word: t("dept.score.mid"), bar: "bg-accent" },
  weak: { word: t("dept.score.weak"), bar: "bg-down" },
};

export function deptScore(d: DepartmentBody): DeptScore | null {
  if (isService(d.class)) return null;
  const quality = d.last?.quality_pts ?? null;
  const satisfaction = d.guest_satisfaction_pts;
  const parts = [quality, d.condition_pts, satisfaction].filter((x): x is number => x !== null);
  const points = Math.round(parts.reduce((sum, x) => sum + x, 0) / parts.length);
  return { points, quality, condition: d.condition_pts, satisfaction };
}

/** Razina po prikazanom broju, da „70/100“ nikad ne bude „srednje“. */
export function scoreLevel(points: number): ScoreLevel {
  const shown = wholePoints(points);
  return shown >= GOOD_FROM ? "good" : shown >= MID_FROM ? "mid" : "weak";
}
