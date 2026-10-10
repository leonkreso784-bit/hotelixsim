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
  | "roomsRevenue"
  | "totalRevenue"
  | "trevpar"
  | "labor"
  | "laborShare"
  | "cpor"
  | "gop"
  | "gopMargin"
  | "goppar"
  | "breakEven";

export type KpiRow = { key: KpiKey; value: (m: KpiMonthBody) => string };

export const KPI_ROWS: readonly KpiRow[] = [
  { key: "occupancy", value: (m) => formatPercentBp(m.occupancy_bp) },
  { key: "sold", value: (m) => formatNights(m.sold_nights) },
  { key: "lost", value: (m) => formatNights(m.lost_nights) },
  { key: "adr", value: (m) => formatEurosCents(m.adr_cents) },
  { key: "revpar", value: (m) => formatEurosCents(m.revpar_cents) },
  { key: "roomsRevenue", value: (m) => formatEuros(m.rooms_revenue_cents) },
  { key: "totalRevenue", value: (m) => formatEuros(m.total_revenue_cents) },
  { key: "trevpar", value: (m) => formatEurosCents(m.trevpar_cents) },
  { key: "labor", value: (m) => formatEuros(m.labor_cents) },
  { key: "laborShare", value: (m) => formatPercentBp(m.labor_share_bp) },
  { key: "cpor", value: (m) => formatEurosCents(m.cpor_cents) },
  { key: "gop", value: (m) => formatEuros(m.gop_cents) },
  { key: "gopMargin", value: (m) => formatPercentBp(m.gop_margin_bp) },
  { key: "goppar", value: (m) => formatEurosCents(m.goppar_cents) },
  { key: "breakEven", value: (m) => formatPercentBp(m.break_even_occupancy_bp) },
];

export function rowLabel(key: KpiKey): string {
  return t(`kpi.${key}` as const);
}

export function rowMeaning(key: KpiKey): string {
  return t(`kpi.${key}.mean` as const);
}

/**
 * Točka pokrića rečenicom: „GOP je nula pri popunjenosti od X %; tvoja popunjenost bila je Y %.“ Posebno: mjesec
 * bez prodaje, GOP pozitivan i bez prodaje, prag iznad onoga što sobe u funkciji dopuštaju, prodaja bez doprinosa.
 */
export function breakEvenSentence(m: KpiMonthBody): string {
  if (m.room_nights === 0 || m.sold_nights === 0) return t("kpi.breakEven.noSales");
  const be = m.break_even_occupancy_bp;
  if (be === null) return t("kpi.breakEven.none");
  if (be === 0) return t("kpi.breakEven.zero");
  const params = {
    be: formatPercentBp(be),
    you: formatPercentBp(m.occupancy_bp),
    max: formatPercentBp(m.sellable_occupancy_bp),
  };
  const reachable = m.sellable_occupancy_bp !== null && be <= m.sellable_occupancy_bp;
  return t(reachable ? "kpi.breakEven.sentence" : "kpi.breakEven.over", params);
}
