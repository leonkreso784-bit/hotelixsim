// Obnova v2 (odjel ili zgrada): rata, učinak na stanje, blagajna nakon plaćanja i amortizacija. Sve brojke izbora računa
// poslužitelj (`RenewalLimitsBody.options`, iste funkcije kao obračun); ekran bira izbor i od blagajne samo oduzima
// rate ovog kruga. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { PlanStateBody, RenewalLimitsBody, RenewalOptionBody, RenewalStateBody } from "../../api/types";
import { formatCount, formatEuros, formatIndex, formatWholeSigned, wholePoints } from "../../format";

/** Oznaka zgrade u `planPayNow` (kodovi odjela su nazivi odjela, ne „building“). */
export const BUILDING = "building";

/** Izbor za iznos; iznos izvan ponude (ne bi smio biti) nema izbora. */
export function optionFor(l: RenewalLimitsBody, amount: number): RenewalOptionBody | null {
  return l.options.find((o) => o.amount_cents === amount) ?? null;
}

/** Iznos jedan korak gore (`1`) ili dolje (`-1`); na rubu ponude `null`. */
export function stepAmount(l: RenewalLimitsBody, amount: number, dir: 1 | -1): number | null {
  const i = l.options.findIndex((o) => o.amount_cents === amount);
  if (i < 0) return l.options[0]?.amount_cents ?? null;
  return l.options[i + dir]?.amount_cents ?? null;
}

/** Rata zaključane obnove u ovom krugu: zadnji krug mjeseca plaća ostatak. */
function lockedPayNow(s: RenewalStateBody): number {
  return s.rounds_left <= 1 ? s.total_cents - s.paid_cents : s.installment_cents;
}

/** Rata izbora u ovom krugu: zadnji krug mjeseca (u mjesečnoj igri jedini) plaća ostatak, tj. cijeli iznos. */
function optionPayNow(o: RenewalOptionBody, roundsLeft: number): number {
  return roundsLeft <= 1 ? o.last_installment_cents : o.installment_cents;
}

/** Što jedan dio plaća ovaj krug za iznos `amount` (zaključana obnova plaća svoju ratu bez obzira na iznos). */
export function payNow(l: RenewalLimitsBody, amount: number): number {
  if (l.locked) return lockedPayNow(l.locked);
  const o = optionFor(l, amount);
  return o ? optionPayNow(o, l.rounds_left) : 0;
}

/**
 * Zbroj rata ovog kruga za cijeli plan (svi odjeli i zgrada). `override` zamjenjuje iznos jednog dijela
 * (nespremljeni nacrt odjela ili zgrade, kod `BUILDING`).
 */
export function planPayNow(state: PlanStateBody, override?: { code: string; amount: number }): number {
  const amountOf = (code: string, saved: number | null | undefined) =>
    override?.code === code ? override.amount : (saved ?? 0);
  let sum = 0;
  for (const [code, lim] of Object.entries(state.limits.departments ?? {})) {
    if (lim.renewal)
      sum += payNow(lim.renewal, amountOf(code, state.plan.departments?.[code]?.renewal_cents));
  }
  const b = state.limits.costs.building_renewal;
  if (b) sum += payNow(b, amountOf(BUILDING, state.plan.building_renewal_cents));
  return sum;
}

/** „Plaćaš 10.000 € tjedno, 4 tjedna.“; jedan krug (mjesečna igra, zadnji tjedan) „Plaćaš 40.000 € odjednom.“ */
export function payText(o: RenewalOptionBody, roundsLeft: number): string {
  if (roundsLeft <= 1) return t("renewal.pay.once", { amount: formatEuros(o.amount_cents) });
  const params = {
    installment: formatEuros(o.installment_cents),
    weeks: formatCount(roundsLeft, "week", String(roundsLeft)),
  };
  return o.last_installment_cents === o.installment_cents
    ? t("renewal.pay.week", params)
    : t("renewal.pay.weekLast", { ...params, last: formatEuros(o.last_installment_cents) });
}

/** „Stanje prostora +21 (s 64 na 85/100).“ — razlika prikazanih cijelih bodova, da se brojke slažu. */
export function gainText(subject: "dept" | "building", conditionPts: number, gainPts: number): string {
  const after = conditionPts + gainPts;
  return t("renewal.gain", {
    subject: t(`renewal.subject.${subject}` as const),
    delta: formatWholeSigned(wholePoints(after) - wholePoints(conditionPts)),
    from: formatIndex(conditionPts),
    to: formatIndex(after),
  });
}

/** „Amortizacija 476 € mjesečno kroz 7 godina, od kolovoza.“ (vijek u godinama kad je djeljiv s 12). */
export function depreciationText(o: RenewalOptionBody, l: RenewalLimitsBody): string {
  const m = l.life_months;
  const life =
    m % 12 === 0 ? formatCount(m / 12, "yearAcc", String(m / 12)) : formatCount(m, "month", String(m));
  return t("renewal.depreciation", {
    amount: formatEuros(o.depreciation_monthly_cents),
    life,
    from: t(`renewal.from.${l.month}` as const),
  });
}

/** Upozorenje kad bi blagajna nakon plaćanja bila u minusu ili ispod jednog tjedna plaća (kao stara obnova). */
export function cashWarning(afterCents: number, payrollWeekCents: number): string | null {
  if (afterCents < 0) return t("plan.renovation.negative");
  if (afterCents < payrollWeekCents)
    return t("plan.renovation.thin", { payroll: formatEuros(payrollWeekCents) });
  return null;
}

/**
 * Zaključana obnova kratko: „40.000 € u srpnju, 10.000 € tjedno, još 2 tjedna“; mjesečna igra „40.000 € u srpnju,
 * plaćeno“. `rounds_left` uključuje ovaj krug.
 */
export function lockedPhrase(s: RenewalStateBody, weekly: boolean): string {
  const total = formatEuros(s.total_cents);
  const month = t(`renewal.in.${s.month}` as const);
  if (!weekly || s.rounds_left <= 0) return t("renewal.phrase.paid", { total, month });
  if (s.rounds_left === 1)
    return t("renewal.phrase.last", { total, month, last: formatEuros(s.total_cents - s.paid_cents) });
  return t("renewal.phrase.week", {
    total,
    month,
    installment: formatEuros(s.installment_cents),
    weeks: formatCount(s.rounds_left, "week", String(s.rounds_left)),
  });
}

/** Rečenica zaključane obnove na ekranu odjela i u planu. */
export function lockedText(s: RenewalStateBody, weekly: boolean): string {
  return t("renewal.locked", { phrase: lockedPhrase(s, weekly) });
}
