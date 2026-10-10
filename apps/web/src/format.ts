import { t } from "@hotelixsim/i18n";
import type { MonthCode, StrategyCode } from "./api/types";

const nf2 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });
// hr-HR po zadanom ne grupira četiri znamenke (1234), pa grupiranje tražimo izričito: 1.234.
const nf0 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 0, useGrouping: "always" });
const nf1 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 1, useGrouping: "always" });
const nfCents = new Intl.NumberFormat("hr-HR", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
  useGrouping: "always",
});
const nfRating = new Intl.NumberFormat("hr-HR", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
const MINUS = "−";
const NONE = "—";

/**
 * Jedino pravilo za bodove na ekranu: stotinke → cijeli bodovi, pola od nule (4850 → 49, 4849 → 48, −450 → −5).
 * Isto pravilo koristi mentor na poslužitelju, pa se prikazane vrijednosti i promjene slažu.
 */
export function wholePoints(hundredths: number): number {
  const rounded = Math.round(Math.abs(hundredths) / 100);
  return hundredths < 0 && rounded > 0 ? -rounded : rounded;
}

/** Boja broja napretka po predznaku prikazanog (cijelog) broja: −0,3 se prikaže kao 0 i ostaje neutralno. */
export function progressTone(hundredths: number): string {
  const n = wholePoints(hundredths);
  return n > 0 ? "text-up" : n < 0 ? "text-down" : "text-muted";
}

/** Indeks u stotinkama (0–10000) → cijeli broj za prikaz, npr. 4850 → "49". */
export function formatIndex(hundredths: number): string {
  return nf0.format(wholePoints(hundredths));
}

/** Bodovi (stotinke) s predznakom, cijeli: 825 → "+8", −200 → "−2", 20 → "0". */
export function formatSignedPoints(hundredths: number): string {
  return formatWholeSigned(wholePoints(hundredths));
}

/** Cijeli broj s predznakom: 6 → "+6", −6 → "−6", 0 → "0". */
export function formatWholeSigned(n: number): string {
  return signed(n, nf0.format(Math.abs(n)));
}

function signed(value: number, abs: string): string {
  if (value === 0) return abs;
  return value > 0 ? `+${abs}` : `${MINUS}${abs}`;
}

/** Centi → cijeli euri: 9 800 000 → "98.000 €". */
export function formatEuros(cents: number): string {
  const euros = Math.round(cents / 100);
  return `${euros < 0 ? MINUS : ""}${nf0.format(Math.abs(euros))} €`;
}

/** Centi → euri s centima (pokazatelji po sobi): 4003 → "40,03 €"; bez vrijednosti crtica. */
export function formatEurosCents(cents: number | null): string {
  if (cents === null) return NONE;
  return `${cents < 0 ? MINUS : ""}${nfCents.format(Math.abs(cents) / 100)} €`;
}

/** Bazni bodovi → postotak, najviše jedna decimala: 6800 → "68 %", 3173 → "31,7 %"; bez vrijednosti crtica. */
export function formatPercentBp(bp: number | null): string {
  if (bp === null) return NONE;
  return `${bp < 0 ? MINUS : ""}${nf1.format(Math.abs(bp) / 100)} %`;
}

/** Ocjena u stotinkama na ljestvici do 10, uvijek jedna decimala: 840 → "8,4/10", 800 → "8,0/10", 898 → "9,0/10". */
export function formatRating(hundredths: number): string {
  return `${nfRating.format(hundredths / 100)}/10`;
}

/** Cijeli broj s grupiranjem, bez jedinice (samo kao dio oznake s jedinicom): 40000 → "40.000". */
export function formatWhole(n: number): string {
  return `${n < 0 ? MINUS : ""}${nf0.format(Math.abs(n))}`;
}

/** Dopušteni raspon s jedinicom: (0, 40000, "euro") → "0–40.000 €", (18, 38, "person") → "18–38 osoba". */
export function formatRange(min: number, max: number, unit: "euro" | "percent" | "person"): string {
  const span = `${formatWhole(min)}–${formatWhole(max)}`;
  if (unit === "euro") return `${span} €`;
  if (unit === "percent") return `${span} %`;
  // Imenica se slaže s gornjom granicom: „18–38 osoba“, „1–2 osobe“.
  return formatCount(max, "person", span);
}

/** Promjena u bodovima s množinom: 450 → "+5 bodova", −100 → "−1 bod". */
export function formatPointsCount(hundredths: number): string {
  const n = wholePoints(hundredths);
  return formatCount(n, "point", formatWholeSigned(n));
}

export function monthName(code: MonthCode): string {
  return t(`month.${code}` as const);
}

export function capitalize(s: string): string {
  return s.charAt(0).toLocaleUpperCase("hr-HR") + s.slice(1);
}

export function strategyLabel(code: StrategyCode): string {
  return t(`strategy.${code}` as const);
}

const pluralRules = new Intl.PluralRules("hr");

/**
 * Broj s imenicom u ispravnom hrvatskom množinskom obliku: 1 bod, 3 boda, 8 bodova.
 * Necijeli brojevi (8,25) uzimaju oblik uz 2–4 („8,25 boda“).
 */
export function formatCount(
  n: number,
  noun: "point" | "person" | "night",
  shown: string = nf2.format(n),
): string {
  const category = Number.isInteger(n) ? pluralRules.select(Math.abs(n)) : "few";
  const form = category === "one" || category === "few" ? category : "other";
  return `${shown} ${t(`plural.${noun}.${form}` as const)}`;
}

/** Noći s grupiranjem i množinom: 1 noć, 3 noći, 3.600 noći. */
export function formatNights(n: number): string {
  return formatCount(n, "night", nf0.format(n));
}
