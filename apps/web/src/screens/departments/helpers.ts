// Pomoćnici odjela: razlikovanje službi bez prihoda i rečenica o rezultatu. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DeptClassCode } from "../../api/types";
import { formatCount, formatEurosSigned } from "../../format";

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

/** Rezultat sa strelicom i predznakom, da se rast i pad ne razlikuju samo bojom: „▲ +7.896 €“. */
export function resultText(cents: number): string {
  const arrow = cents > 0 ? "▲ " : cents < 0 ? "▼ " : "";
  return `${arrow}${formatEurosSigned(cents)}`;
}

/** „Ljudi: 2 stalna, 1 sezonski“. */
export function peopleText(permanent: number, seasonal: number): string {
  return t("dept.people", {
    permanent: formatCount(permanent, "permanent"),
    seasonal: formatCount(seasonal, "seasonal"),
  });
}
