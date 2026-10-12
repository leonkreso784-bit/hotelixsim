import { type MessageKey, type MessageParams, t } from "@hotelixsim/i18n";
import type { MonthCode, RoundInfoBody, StrategyCode } from "./api/types";

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
const nfFixed1 = new Intl.NumberFormat("hr-HR", {
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
  useGrouping: "always",
});

/**
 * Jedino pravilo za bodove na ekranu: stotinke → cijeli bodovi, pola od nule (4850 → 49, 4849 → 48, −450 → −5).
 * Isto pravilo koristi mentor na poslužitelju, pa se prikazane vrijednosti i promjene slažu.
 */
export function wholePoints(hundredths: number): number {
  const rounded = Math.round(Math.abs(hundredths) / 100);
  return hundredths < 0 && rounded > 0 ? -rounded : rounded;
}

/** Broj decimala bodova napretka: 0 (zadano, cijeli bodovi), 1 ili 2. */
export type PointDigits = 0 | 1 | 2;

/**
 * Bodovi (stotinke) zaokruženi na `digits` decimala, pola od nule (845 → 8 / 8,5): isto pravilo kao sezonski mentor
 * na poslužitelju (`season_mentor.rs` `points_with`), pa se brojke ekrana i mentora slažu.
 */
export function pointsAt(hundredths: number, digits: PointDigits): number {
  const scale = 10 ** (2 - digits);
  const v = Math.round(Math.abs(hundredths) / scale);
  const signed = hundredths < 0 && v > 0 ? -v : v;
  return signed / 10 ** digits;
}

/**
 * Najmanje decimala (0, 1 ili 2) uz koje hoteli s različitim napretkom i dalje imaju različit prikazani broj: +8,4 i
 * +7,6 nisu oba „+8“. Isto pravilo kao `progress_digits` sezonskog mentora; jednak napredak ne traži decimale.
 */
export function progressDigits(values: number[]): PointDigits {
  for (const d of [0, 1] as const) {
    const ok = values.every((a) => values.every((b) => a === b || pointsAt(a, d) !== pointsAt(b, d)));
    if (ok) return d;
  }
  return 2;
}

/** Boja broja napretka po predznaku prikazanog broja: −0,3 se uz cijele bodove prikaže kao 0 i ostaje neutralno. */
export function progressTone(hundredths: number, digits: PointDigits = 0): string {
  const n = pointsAt(hundredths, digits);
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
  return `${euros < 0 ? MINUS : ""}${nf0.format(Math.abs(euros))}${NBSP}€`;
}

/** Iznos s predznakom, cijeli euri: 789 600 → "+7.896 €", −80 000 → "−800 €", 0 → "0 €". */
export function formatEurosSigned(cents: number): string {
  const euros = Math.round(cents / 100);
  return `${euros > 0 ? "+" : ""}${formatEuros(cents)}`;
}

/** Gostiju po zaposlenom na dan (stotinke): 190 → "1,9 gosta", 100 → "1 gost". */
export function formatLoad(hundredths: number): string {
  return formatCount(hundredths / 100, "guest", nf1.format(hundredths / 100));
}

/** Centi → euri s centima (pokazatelji po sobi): 4003 → "40,03 €"; bez vrijednosti crtica. */
export function formatEurosCents(cents: number | null): string {
  if (cents === null) return NONE;
  return `${cents < 0 ? MINUS : ""}${nfCents.format(Math.abs(cents) / 100)}${NBSP}€`;
}

/** Bazni bodovi → postotak, najviše jedna decimala: 6800 → "68 %", 3173 → "31,7 %"; bez vrijednosti crtica. */
export function formatPercentBp(bp: number | null): string {
  if (bp === null) return NONE;
  return `${bp < 0 ? MINUS : ""}${nf1.format(Math.abs(bp) / 100)}${NBSP}%`;
}

/** Bazni bodovi → postotak, uvijek jedna decimala (usporedba ostvareno/budžet u istom retku): 1300 → "13,0 %". */
export function formatPercentBpFixed(bp: number | null): string {
  if (bp === null) return NONE;
  return `${bp < 0 ? MINUS : ""}${nfFixed1.format(Math.abs(bp) / 100)}${NBSP}%`;
}

/** Ocjena u stotinkama na ljestvici do 10, uvijek jedna decimala: 840 → "8,4/10", 800 → "8,0/10", 898 → "9,0/10". */
export function formatRating(hundredths: number): string {
  return `${nfRating.format(hundredths / 100)}/10`;
}

/** Cijeli postotak (razina cijene u odjelu): 110 → "110 %". */
export function formatPercentWhole(pct: number): string {
  return `${formatWhole(pct)}${NBSP}%`;
}

/** Cijeli broj s grupiranjem, bez jedinice (samo kao dio oznake s jedinicom): 40000 → "40.000". */
export function formatWhole(n: number): string {
  return `${n < 0 ? MINUS : ""}${nf0.format(Math.abs(n))}`;
}

/** Dopušteni raspon s jedinicom: (0, 40000, "euro") → "0–40.000 €", (18, 38, "person") → "18–38 osoba". */
export function formatRange(min: number, max: number, unit: "euro" | "percent" | "person"): string {
  const span = `${formatWhole(min)}–${formatWhole(max)}`;
  if (unit === "euro") return `${span}${NBSP}€`;
  if (unit === "percent") return `${span}${NBSP}%`;
  // Imenica se slaže s gornjom granicom: „18–38 osoba“, „1–2 osobe“.
  return formatCount(max, "person", span);
}

/**
 * Napredak s točno `digits` decimala i množinom: (840, 1) → "+8,4 boda", (800, 1) → "+8,0 boda", (800, 0) → "+8 bodova".
 * Isti broj decimala za sve brojke jedne usporedbe (UI_PRAVILA §2.3), pa „−4,0“ stoji uz „−3,7“, ne „−4“.
 */
export function formatPointsPrecise(hundredths: number, digits: PointDigits): string {
  const n = pointsAt(hundredths, digits);
  const abs = new Intl.NumberFormat("hr-HR", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
    useGrouping: "always",
  }).format(Math.abs(n));
  // Uz decimale („8,0“) imenica je uvijek „boda“, kao uz svaki necijeli broj.
  return digits > 0 ? `${signed(n, abs)} ${t("plural.point.few")}` : formatCount(n, "point", signed(n, abs));
}

/** Prosjek ocjena situacija 1–5 (stotinke), jedna decimala: 360 → "3,6/5". */
export function formatQuality(hundredths: number): string {
  return `${nfRating.format(hundredths / 100)}/5`;
}

/** Koliko krugova od svih: (6, 17, tjedni) → "6 od 17 tjedana", (1, 4, mjesečni) → "1 od 4 mjeseca". */
export function formatRoundsOf(n: number, total: number, weekly: boolean): string {
  return formatCount(
    total,
    weekly ? "week" : "month",
    t("count.of", { n: formatWhole(n), total: formatWhole(total) }),
  );
}

/** Promjena u bodovima s množinom: 450 → "+5 bodova", −100 → "−1 bod". */
export function formatPointsCount(hundredths: number): string {
  const n = wholePoints(hundredths);
  return formatCount(n, "point", formatWholeSigned(n));
}

export function monthName(code: MonthCode): string {
  return t(`month.${code}` as const);
}

/**
 * Krug kad igra još nije stigla: tjedni krug (kod „t05“) nosi samo broj („tjedan 5“), a mjesečni ime mjeseca.
 * Mjesec iz dnevnika za tjedan bio bi kriv („lipanj“ za pet krugova).
 */
export function roundWithoutGame(round: string, month: MonthCode): string {
  const week = /^t(\d+)$/.exec(round);
  return week ? keepRoundTogether(t("round.weekBare", { n: Number(week[1]) })) : monthName(month);
}

const NBSP = "\u00a0";
const WORD_JOINER = "\u2060";

/** Datumi u zagradi nelomljivi: razmaci postaju nerazdvojni, a iza crtice dolazi spojnik riječi. Idempotentno. */
function keepDatesTogether(dates: string): string {
  return dates.replace(/ /g, NBSP).replace(/–\u2060?/g, `–${WORD_JOINER}`);
}

/**
 * Naslov kruga bez neželjenih prijeloma: „tjedan 4“ i sve u zagradi (datumi) postaju jedna nelomljiva cjelina, a
 * smije se prijeći u novi redak samo ispred zagrade. Iza crtice dolazi spojnik riječi (U+2060): bez njega preglednik
 * smije prelomiti iza „–“ čak i kad iza nje stoji nerazdvojni razmak. Može se pozvati više puta na isti tekst.
 */
export function keepRoundTogether(text: string): string {
  return text
    .replace(/(tjedan|tjedna|tjednu) (\d+)/gi, `$1${NBSP}$2`)
    .replace(/\([^)]*\)/g, keepDatesTogether);
}

export function capitalize(s: string): string {
  return s.charAt(0).toLocaleUpperCase("hr-HR") + s.slice(1);
}

const MONTH_NUMBER: Record<MonthCode, number> = { jun: 6, jul: 7, aug: 8, sep: 9 };

/**
 * Mjesec: „lipanj“; tjedan: „29. 6. – 5. 7.“ (s nerazdvojnim razmacima, vidi `keepRoundTogether`). Datumi dolaze iz
 * tijela kruga, ekran ne računa kalendar.
 */
export function roundDates(r: RoundInfoBody): string {
  if (r.kind === "month") return monthName(r.first_month);
  return keepDatesTogether(
    t("round.dates", {
      d1: r.first_day,
      m1: MONTH_NUMBER[r.first_month],
      d2: r.last_day,
      m2: MONTH_NUMBER[r.last_month],
    }),
  );
}

/** Krug usred rečenice: „lipanj“ ili „tjedan 5 (29. 6. – 5. 7.)“. */
export function roundPhrase(r: RoundInfoBody): string {
  if (r.kind === "month") return monthName(r.first_month);
  return keepRoundTogether(t("round.weekPhrase", { n: r.index + 1, dates: roundDates(r) }));
}

/** Krug kao naslov: „Lipanj“ ili „Tjedan 5 (29. 6. – 5. 7.)“. */
export function roundTitle(r: RoundInfoBody): string {
  return capitalize(roundPhrase(r));
}

export function strategyLabel(code: StrategyCode): string {
  return t(`strategy.${code}` as const);
}

type WeekBase<K> = K extends `${infer B}.week` ? (B extends MessageKey ? B : never) : never;

/** Tekst za tjedni krug: ako je `weekly`, uzima se `<ključ>.week`, inače `<ključ>` (v1 tekstovi ostaju isti). */
export function tw(weekly: boolean, base: WeekBase<MessageKey>, params?: MessageParams): string {
  return t((weekly ? `${base}.week` : base) as MessageKey, params);
}

const pluralRules = new Intl.PluralRules("hr");

/** Hrvatski množinski oblik broja („one“: 1, 21; „few“: 2–4, 22–24, necijeli; „other“: ostalo). */
export function pluralForm(n: number): "one" | "few" | "other" {
  const category = Number.isInteger(n) ? pluralRules.select(Math.abs(n)) : "few";
  return category === "one" || category === "few" ? category : "other";
}

/**
 * Broj s imenicom u ispravnom hrvatskom množinskom obliku: 1 bod, 3 boda, 8 bodova.
 * Necijeli brojevi (8,25) uzimaju oblik uz 2–4 („8,25 boda“).
 */
export function formatCount(
  n: number,
  noun:
    | "point"
    | "person"
    | "night"
    | "guest"
    | "permanent"
    | "seasonal"
    | "seasonalWorker"
    | "seasonalWorkerAcc"
    | "week"
    | "month"
    | "situation",
  shown: string = nf2.format(n),
): string {
  return `${shown} ${t(`plural.${noun}.${pluralForm(n)}` as const)}`;
}

/** Noći s grupiranjem i množinom: 1 noć, 3 noći, 3.600 noći. */
export function formatNights(n: number): string {
  return formatCount(n, "night", nf0.format(n));
}
