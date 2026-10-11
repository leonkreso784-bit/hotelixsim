// „Što mu fali“ i „U čemu je dobar“: rečenice iz zadnjeg obračunatog kruga odjela, svaka s brojkom i, gdje postoji,
// prijedlogom odluke (🟨 pretpostavka kontrolora uz Leonovu odluku 12.10.). Svi pragovi su u INSIGHT_LIMITS. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DeptRoundBody } from "../../api/types";
import {
  formatEuros,
  formatEurosSigned,
  formatIndex,
  formatPercentWhole,
  formatWhole,
  pluralForm,
  tw,
  wholePoints,
} from "../../format";

export type Insight = { text: string; suggestion: string | null };

export const INSIGHT_LIMITS = {
  /**
   * Opterećenje (gostiju po zaposlenom na dan, stotinke) koje je 100 % mogućnosti odjela: iznad 180 jezgra smanjuje
   * kvalitetu usluge i jače troši prostor (v1 `LOAD_LIMIT_HUNDREDTHS`, isto u v2 odjelima).
   */
  fullLoadHundredths: 180,
  /** Iznad ovog postotka mogućnosti: premalo ljudi. */
  overloadedPct: 100,
  /** Ispod ovog postotka mogućnosti: previše ljudi. */
  idlePct: 60,
  /** Stanje prostora (prikazano 0–100) ispod kojeg je prostor dotrajao. */
  conditionWorn: 50,
  /** Stanje prostora od kojeg je prostor u dobrom stanju. */
  conditionGood: 70,
} as const;

/** Postotak mogućnosti odjela koji je krug potrošio: opterećenje / 180, cijeli postotak. */
export function staffUsePct(loadHundredths: number): number {
  return Math.round((loadHundredths * 100) / INSIGHT_LIMITS.fullLoadHundredths);
}

export function insights(r: DeptRoundBody, weekly: boolean): { weak: Insight[]; strong: Insight[] } {
  const weak: Insight[] = [];
  const strong: Insight[] = [];
  const plain = (text: string): Insight => ({ text, suggestion: null });

  // Ljudi i kapacitet vrijede samo za odjel koji gosti koriste (ne najam, ne zatvoren odjel).
  if (r.demand > 0 || r.served > 0) {
    const pct = staffUsePct(r.load_hundredths);
    const p = formatPercentWhole(pct);
    if (pct > INSIGHT_LIMITS.overloadedPct)
      weak.push({ text: t("insight.overloaded", { p }), suggestion: t("insight.overloaded.hint") });
    else if (pct < INSIGHT_LIMITS.idlePct) weak.push(plain(t("insight.idle", { p })));
    else strong.push(plain(t("insight.staffOk", { p })));

    const refused = r.demand - r.served;
    if (refused > 0) {
      const n = `${formatWhole(refused)} ${t(`plural.guest.${pluralForm(refused)}` as const)}`;
      weak.push(plain(t(`insight.refused.${pluralForm(refused)}` as const, { n })));
    } else strong.push(plain(t("insight.allServed", { n: formatWhole(r.served) })));
  }

  const condition = wholePoints(r.condition_after_pts);
  const n = formatIndex(r.condition_after_pts);
  if (condition < INSIGHT_LIMITS.conditionWorn)
    weak.push({ text: t("insight.worn", { n }), suggestion: t("insight.worn.hint") });
  else if (condition >= INSIGHT_LIMITS.conditionGood) strong.push(plain(t("insight.wellKept", { n })));

  const before = wholePoints(r.satisfaction_before_pts);
  const after = wholePoints(r.satisfaction_after_pts);
  const fromTo = { from: formatIndex(r.satisfaction_before_pts), to: formatIndex(r.satisfaction_after_pts) };
  if (after < before) weak.push(plain(t("insight.satisfactionDown", fromTo)));
  else if (after > before) strong.push(plain(t("insight.satisfactionUp", fromTo)));

  const euros = Math.round(r.profit_cents / 100);
  if (euros < 0) weak.push(plain(tw(weekly, "insight.loss", { amount: formatEuros(r.profit_cents) })));
  else if (euros > 0)
    strong.push(plain(tw(weekly, "insight.profit", { amount: formatEurosSigned(r.profit_cents) })));

  return { weak, strong };
}
