// Ocjena odjela 0–100 (Leonova odluka 12.10.: jedna traka po odjelu, boja uvijek uz broj i riječ; sastavnice su ono što
// je SVOJE odjelu). Ocjena = prosjek onoga što odjel ima od triju sastavnica: stanje prostora, zadovoljstvo gostiju
// odjela i usklađenost broja ljudi s poslom (🟨 formula, `staffFit`). Kvaliteta usluge hotela ne ulazi (ista je za sve
// odjele). Najam (parking, lokal) ima samo stanje prostora; zatvoren odjel i službe nemaju ocjenu. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody, DeptRoundBody } from "../../api/types";
import { wholePoints } from "../../format";
import { isService } from "./helpers";
import { INSIGHT_LIMITS } from "./insights";

export type DeptScore = {
  /** Ocjena u bodovima 0–10 000 (na ekranu 0–100). */
  points: number;
  condition: number;
  satisfaction: number | null;
  staffFit: number | null;
};

export type ScoreLevel = "good" | "mid" | "weak";

/** Pragovi boje po prikazanom cijelom broju (🟨): ≥ 70 dobro, 50–69 srednje, < 50 slabo. */
const GOOD_FROM = 70;
const MID_FROM = 50;

/** Riječ i boja trake po razini; boje su tokeni iz `styles.css` (UI_PRAVILA §1.1), riječ stoji uz boju (§6.8). */
export const SCORE_LEVELS: Record<ScoreLevel, { word: string; bar: string }> = {
  good: { word: t("dept.score.good"), bar: "bg-up" },
  mid: { word: t("dept.score.mid"), bar: "bg-warn" },
  weak: { word: t("dept.score.weak"), bar: "bg-down" },
};

/**
 * Usklađenost broja ljudi s poslom 0–10 000 (🟨), ista mjera kao rečenice o ljudima: opterećenje osoblja prema granici
 * za dobru uslugu (180 = 100 %). U zoni bez kazne (60–100 % granice) je 10 000. Iznad granice pada kao kazna
 * opterećenja u jezgri (100 po stotinki iznad 180) i dolazi do 0 pri 240, kad je kazna najveća i odjel radi punim
 * kapacitetom. Ispod 60 % pada ravnomjerno do 0 kod odjela bez ijednog gosta. Odjel bez posla (najam, zatvoren, bez
 * potražnje) nema usklađenost (`null`).
 */
export function staffFit(r: DeptRoundBody): number | null {
  if (r.demand === 0 && r.served === 0) return null;
  const { fullLoadHundredths: full, worstLoadHundredths: worst, idlePct } = INSIGHT_LIMITS;
  const load = r.load_hundredths;
  const idleBelow = (full * idlePct) / 100;
  if (load > full) return Math.max(0, Math.round(10_000 - ((load - full) * 10_000) / (worst - full)));
  if (load < idleBelow) return Math.round((load * 10_000) / idleBelow);
  return 10_000;
}

export function deptScore(d: DepartmentBody): DeptScore | null {
  if (isService(d.class) || !d.open) return null;
  const lease = d.class === "rentals";
  const satisfaction = lease ? null : d.guest_satisfaction_pts;
  const fit = lease || !d.last ? null : staffFit(d.last);
  const parts = [d.condition_pts, satisfaction, fit].filter((x): x is number => x !== null);
  const points = Math.round(parts.reduce((sum, x) => sum + x, 0) / parts.length);
  return { points, condition: d.condition_pts, satisfaction, staffFit: fit };
}

/** Napomena ispod ocjene: nabraja samo sastavnice koje se crtaju (M1). */
export function scoreParts(s: DeptScore): string {
  const names = [
    t("dept.score.part.condition"),
    ...(s.satisfaction !== null ? [t("dept.score.part.satisfaction")] : []),
    ...(s.staffFit !== null ? [t("dept.score.part.staffFit")] : []),
  ];
  if (names.length === 1) return t("dept.score.note.one");
  const last = names.pop() ?? "";
  return t("dept.score.note", { parts: `${names.join(", ")} i ${last}` });
}

/** Razina po prikazanom broju, da „70/100“ nikad ne bude „srednje“. */
export function scoreLevel(points: number): ScoreLevel {
  const shown = wholePoints(points);
  return shown >= GOOD_FROM ? "good" : shown >= MID_FROM ? "mid" : "weak";
}
