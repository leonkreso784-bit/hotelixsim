import { t } from "@hotelixsim/i18n";
import type { MonthCode, StrategyCode } from "./api/types";

const nf2 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });
// hr-HR po zadanom ne grupira četiri znamenke (1234), pa grupiranje tražimo izričito: 1.234.
const nf0 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 0, useGrouping: "always" });
const MINUS = "−";

/**
 * Jedino pravilo za bodove na ekranu: stotinke → cijeli bodovi, pola od nule (4850 → 49, 4849 → 48, −450 → −5).
 * Isto pravilo koristi mentor na poslužitelju, pa se prikazane vrijednosti i promjene slažu.
 */
export function wholePoints(hundredths: number): number {
  const rounded = Math.round(Math.abs(hundredths) / 100);
  return hundredths < 0 && rounded > 0 ? -rounded : rounded;
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

/** Ocjena u stotinkama na ljestvici do 10: 840 → "8,4/10". */
export function formatRating(hundredths: number): string {
  return `${nf2.format(hundredths / 100)}/10`;
}

export function formatOccupancy(sold: number, available: number): string {
  if (available <= 0) return "—";
  return `${Math.round((sold * 100) / available)} %`;
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
export function formatCount(n: number, noun: "point", shown: string = nf2.format(n)): string {
  const category = Number.isInteger(n) ? pluralRules.select(Math.abs(n)) : "few";
  const form = category === "one" || category === "few" ? category : "other";
  return `${shown} ${t(`plural.${noun}.${form}`)}`;
}
