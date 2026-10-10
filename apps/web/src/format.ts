import { t } from "@hotelixsim/i18n";
import type { CauseCode, MonthCode, StrategyCode, UnitCode, VarCode } from "./api/types";

const nf2 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });
const nf0 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 0 });
const MINUS = "−";

/** Indeks u stotinkama (0–10000) → tekst za prikaz, npr. 5025 → "50,25". */
export function formatIndex(hundredths: number): string {
  return nf2.format(hundredths / 100);
}

function signed(value: number, abs: string): string {
  if (value === 0) return abs;
  return value > 0 ? `+${abs}` : `${MINUS}${abs}`;
}

/** Stotinke s predznakom: 825 → "+8,25", -200 → "−2", 0 → "0". */
export function formatSigned(hundredths: number): string {
  return signed(hundredths, nf2.format(Math.abs(hundredths) / 100));
}

/** Centi → cijeli euri: 9 800 000 → "98.000 €". */
export function formatEuros(cents: number): string {
  const euros = Math.round(cents / 100);
  return `${euros < 0 ? MINUS : ""}${nf0.format(Math.abs(euros))} €`;
}

export function formatSignedEuros(cents: number): string {
  return signed(cents, formatEuros(Math.abs(cents)));
}

/** Ocjena u stotinkama na ljestvici do 10: 840 → "8,4/10". */
export function formatRating(hundredths: number): string {
  return `${nf2.format(hundredths / 100)}/10`;
}

export function formatOccupancy(sold: number, available: number): string {
  if (available <= 0) return "—";
  return `${Math.round((sold * 100) / available)} %`;
}

/** Iznos promjene s jedinicom iz obračuna (feed `change`). */
export function formatChange(amount: number, unit: UnitCode): string {
  switch (unit) {
    case "cents":
      return formatSignedEuros(amount);
    case "room_nights":
      return `${signed(amount, nf0.format(Math.abs(amount)))} ${t("unit.room_nights")}`;
    case "people":
      return `${signed(amount, nf0.format(Math.abs(amount)))} ${t("unit.people")}`;
    case "points":
      return `${formatSigned(amount)} ${t("unit.points")}`;
    case "rating_hundredths":
      return `${formatSigned(amount)} ${t("unit.rating_hundredths")}`;
    case "bp":
      return `${formatSigned(amount)} %`;
  }
}

export function monthName(code: MonthCode): string {
  return t(`month.${code}` as const);
}

export function capitalize(s: string): string {
  return s.charAt(0).toLocaleUpperCase("hr-HR") + s.slice(1);
}

export function causeLabel(code: CauseCode): string {
  return t(`cause.${code}` as const);
}

export function varLabel(code: VarCode): string {
  return t(`var.${code}` as const);
}

export function strategyLabel(code: StrategyCode): string {
  return t(`strategy.${code}` as const);
}

const pluralRules = new Intl.PluralRules("hr");

/**
 * Broj s imenicom u ispravnom hrvatskom množinskom obliku: 1 promjena, 2 promjene, 5 promjena.
 * Necijeli brojevi (8,25) uzimaju oblik uz 2–4 („8,25 boda“).
 */
export function formatCount(n: number, noun: "change" | "point", shown: string = nf2.format(n)): string {
  const category = Number.isInteger(n) ? pluralRules.select(Math.abs(n)) : "few";
  const form = category === "one" || category === "few" ? category : "other";
  return `${shown} ${t(`plural.${noun}.${form}`)}`;
}
