// Pomoćnici odjela: razlikovanje službi bez prihoda i rečenica o rezultatu. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DeptClassCode } from "../../api/types";
import { formatCount, formatEurosSigned, wholePoints } from "../../format";

/** Vidljiv fokus tipkovnice (UI_PRAVILA §6.4). */
export const FOCUS = "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";

const SERVICE_CLASSES: readonly DeptClassCode[] = [
  "admin_general",
  "sales_marketing",
  "maintenance",
  "utilities",
];

/** Služba bez prihoda (uprava, prodaja i marketing, održavanje, energija i voda): ima samo trošak. */
export function isService(cls: DeptClassCode): boolean {
  return SERVICE_CLASSES.includes(cls);
}

/** Najam bez ljudi (parking, lokal u najmu): nema goste po korištenju ni zaposlene, pa ni redak o ljudima. */
export function leaseWithoutPeople(d: {
  class: DeptClassCode;
  permanent: number;
  seasonal: number;
}): boolean {
  return d.class === "rentals" && d.permanent === 0 && d.seasonal === 0;
}

/**
 * Rezultat sa strelicom i predznakom, da se rast i pad ne razlikuju samo bojom: „▲ +7.896 €“. Strelica ide po
 * prikazanim (zaokruženim) eurima, pa blizu nule nema „▼ 0 €“.
 */
export function resultText(cents: number): string {
  const euros = Math.round(cents / 100);
  const arrow = euros > 0 ? "▲ " : euros < 0 ? "▼ " : "";
  return `${arrow}${formatEurosSigned(cents)}`;
}

/** „Ljudi: 2 stalna, 1 sezonski“. */
export function peopleText(permanent: number, seasonal: number): string {
  return t("dept.people", {
    permanent: formatCount(permanent, "permanent"),
    seasonal: formatCount(seasonal, "seasonal"),
  });
}

/**
 * Promjene stanja prostora (stotinke, s predznakom) u cijelim bodovima koje se zbrajaju u prikazanu razliku
 * „prije → poslije“: svaka je razlika zaokruženih međuzbrojeva (prije + promjene do nje, u granicama 0–100), a
 * zadnja završava točno na „poslije“. Zasebno zaokružene male promjene bile bi 0, 0, 0 uz „78 → 77“.
 */
export function shownChanges(before: number, changes: number[], after: number): number[] {
  let running = before;
  let shown = wholePoints(before);
  return changes.map((change, i) => {
    running += change;
    const point = i === changes.length - 1 ? after : Math.min(10_000, Math.max(0, running));
    const next = wholePoints(point);
    const step = next - shown;
    shown = next;
    return step;
  });
}
