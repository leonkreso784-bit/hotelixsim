// Redovi ekrana „Pokazatelji“: naziv, rečenica što znači (formula riječima) i vrijednost s jedinicom. Brojke
// računa poslužitelj (`/v1/hotels/{id}/kpi`); ovdje se samo oblikuju.
import { t } from "@hotelixsim/i18n";
import type { KpiMonthBody } from "../../api/types";
import { formatEuros, formatEurosCents, formatNights, formatPercentBp } from "../../format";

type KpiKey =
  | "occupancy"
  | "sold"
  | "lost"
  | "adr"
  | "revpar"
  | "trevpar"
  | "gop"
  | "gopMargin"
  | "goppar"
  | "laborShare"
  | "cpor"
  | "breakEven";

export type KpiRow = { key: KpiKey; value: (m: KpiMonthBody) => string };

export const KPI_ROWS: readonly KpiRow[] = [
  { key: "occupancy", value: (m) => formatPercentBp(m.occupancy_bp) },
  { key: "sold", value: (m) => formatNights(m.sold_nights) },
  { key: "lost", value: (m) => formatNights(m.lost_nights) },
  { key: "adr", value: (m) => formatEurosCents(m.adr_cents) },
  { key: "revpar", value: (m) => formatEurosCents(m.revpar_cents) },
  { key: "trevpar", value: (m) => formatEurosCents(m.trevpar_cents) },
  { key: "gop", value: (m) => formatEuros(m.gop_cents) },
  { key: "gopMargin", value: (m) => formatPercentBp(m.gop_margin_bp) },
  { key: "goppar", value: (m) => formatEurosCents(m.goppar_cents) },
  { key: "laborShare", value: (m) => formatPercentBp(m.labor_share_bp) },
  { key: "cpor", value: (m) => formatEurosCents(m.cpor_cents) },
  { key: "breakEven", value: (m) => formatPercentBp(m.break_even_occupancy_bp) },
];

export function rowLabel(key: KpiKey): string {
  return t(`kpi.${key}` as const);
}

export function rowMeaning(key: KpiKey): string {
  return t(`kpi.${key}.mean` as const);
}

/** „GOP je nula pri popunjenosti od X %; ti si bio na Y %.“ (iznad 100 % i bez točke pokrića svojim riječima). */
export function breakEvenSentence(m: KpiMonthBody): string {
  const be = m.break_even_occupancy_bp;
  if (be === null) return t("kpi.breakEven.none");
  const params = { be: formatPercentBp(be), you: formatPercentBp(m.occupancy_bp) };
  return t(be > 10_000 ? "kpi.breakEven.over" : "kpi.breakEven.sentence", params);
}
