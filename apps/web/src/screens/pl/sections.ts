// Redci jedne sekcije računa dobiti: stavke, pa zbirni redak (ako ga ima). Isti redci hrane tablicu i kartice.
import { t } from "@hotelixsim/i18n";
import type { PlLineBody, PlSectionBody } from "../../api/types";

export type PlRow = { line: PlLineBody; total: boolean };

export function sectionTitle(section: PlSectionBody["section"]): string {
  return t(`pl.section.${section}` as const);
}

export function sectionRows(s: PlSectionBody): PlRow[] {
  const rows = s.lines.map((line) => ({ line, total: false }));
  return s.total ? [...rows, { line: s.total, total: true }] : rows;
}
