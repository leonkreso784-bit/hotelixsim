// „Što mu fali“ i „U čemu je dobar“: rečenice iz zadnjeg obračunatog kruga odjela, svaka s brojkom i, gdje postoji,
// prijedlogom odluke (🟨 pretpostavka kontrolora uz Leonovu odluku 12.10.). Svi pragovi su u INSIGHT_LIMITS. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody, DeptRoundBody } from "../../api/types";
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

/** Što pravila trebaju znati o odjelu: razred (Smještaj broji noćenja) i je li otvoren. */
export type DeptKind = Pick<DepartmentBody, "class" | "open">;

export const INSIGHT_LIMITS = {
  /**
   * Opterećenje (gostiju po zaposlenom na dan, stotinke) koje je 100 % granice za dobru uslugu: iznad 180 jezgra
   * smanjuje kvalitetu usluge i jače troši prostor (v1 `LOAD_LIMIT_HUNDREDTHS`, isto u v2 odjelima).
   */
  fullLoadHundredths: 180,
  /** Opterećenje pri punom kapacitetu osoblja: kazna opterećenja u jezgri je tu najveća (6.000 = 100 · 60). */
  worstLoadHundredths: 240,
  /** Iznad ovog postotka granice: osoblje je preopterećeno. */
  overloadedPct: 100,
  /** Ispod ovog postotka granice: previše ljudi. */
  idlePct: 60,
  /** Stanje prostora (prikazano 0–100) ispod kojeg je prostor dotrajao. */
  conditionWorn: 50,
  /** Stanje prostora od kojeg je prostor u dobrom stanju. */
  conditionGood: 70,
  /** Najmanja promjena zadovoljstva (prikazani bodovi) koja je slaba ili jaka strana; 1 bod je šum zaokruživanja. */
  satisfactionStep: 2,
} as const;

/** Postotak granice za dobru uslugu koji je krug potrošio: opterećenje / 180, cijeli postotak. */
export function staffUsePct(loadHundredths: number): number {
  return Math.round((loadHundredths * 100) / INSIGHT_LIMITS.fullLoadHundredths);
}

/** „605 noćenja“ za Smještaj, „453 korištenja“ za ostale odjele. */
export function usesText(n: number, cls: DepartmentBody["class"]): string {
  const noun = cls === "rooms" ? "stay" : "use";
  return `${formatWhole(n)} ${t(`plural.${noun}.${pluralForm(n)}` as const)}`;
}

export function insights(
  dept: DeptKind,
  r: DeptRoundBody,
  weekly: boolean,
): { weak: Insight[]; strong: Insight[] } {
  const weak: Insight[] = [];
  const strong: Insight[] = [];
  const plain = (text: string): Insight => ({ text, suggestion: null });

  // Zatvoren odjel: gubitak su plaće stalnih radnika, pa to i piše (umjesto gubitka bez razloga).
  if (!dept.open) weak.push(plain(tw(weekly, "insight.closed", { amount: formatEuros(r.payroll_cents) })));

  // Ljudi i kapacitet vrijede samo za odjel koji gosti koriste (ne najam, ne zatvoren odjel).
  if (dept.open && (r.demand > 0 || r.served > 0)) {
    const pct = staffUsePct(r.load_hundredths);
    const p = formatPercentWhole(pct);
    if (pct > INSIGHT_LIMITS.overloadedPct)
      weak.push({ text: t("insight.overloaded", { p }), suggestion: t("insight.overloaded.hint") });
    else if (pct < INSIGHT_LIMITS.idlePct) weak.push(plain(t("insight.idle", { p })));
    else strong.push(plain(t("insight.staffOk", { p })));

    const refused = r.demand - r.served;
    if (refused > 0)
      weak.push(
        plain(t(`insight.refused.${pluralForm(refused)}` as const, { n: usesText(refused, dept.class) })),
      );
    else strong.push(plain(t("insight.allServed", { n: usesText(r.served, dept.class) })));
  }

  const condition = wholePoints(r.condition_after_pts);
  const n = formatIndex(r.condition_after_pts);
  if (condition < INSIGHT_LIMITS.conditionWorn)
    weak.push({ text: t("insight.worn", { n }), suggestion: t("insight.worn.hint") });
  else if (condition >= INSIGHT_LIMITS.conditionGood && dept.open)
    strong.push(plain(t("insight.wellKept", { n })));

  if (!dept.open) return { weak, strong };

  const change = wholePoints(r.satisfaction_after_pts) - wholePoints(r.satisfaction_before_pts);
  const fromTo = { from: formatIndex(r.satisfaction_before_pts), to: formatIndex(r.satisfaction_after_pts) };
  if (change <= -INSIGHT_LIMITS.satisfactionStep) weak.push(plain(t("insight.satisfactionDown", fromTo)));
  else if (change >= INSIGHT_LIMITS.satisfactionStep) strong.push(plain(t("insight.satisfactionUp", fromTo)));

  const euros = Math.round(r.profit_cents / 100);
  if (euros < 0) weak.push(plain(tw(weekly, "insight.loss", { amount: formatEuros(r.profit_cents) })));
  else if (euros > 0)
    strong.push(plain(tw(weekly, "insight.profit", { amount: formatEurosSigned(r.profit_cents) })));

  return { weak, strong };
}
