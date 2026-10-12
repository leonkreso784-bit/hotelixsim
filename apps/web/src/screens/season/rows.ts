// Retci ekrana „Kraj sezone“: poredak, pobjednik, krivulja, indeksi, financije i odluke svakog hotela. Jedan izvor za
// kartice (mobitel) i tablice (računalo); brojke dolaze s poslužitelja (`GET /v1/games/{id}/season`), ovdje se samo
// slažu i oblikuju kroz format.ts. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { IndexCode, SeasonBody, SeasonHotelBody } from "../../api/types";
import {
  formatCount,
  formatEuros,
  formatPercentBpFixed,
  formatPercentWhole,
  formatPointsPrecise,
  formatQuality,
  formatRange,
  formatRoundsOf,
  monthName,
  type PointDigits,
  progressDigits,
  roundPhrase,
  wholePoints,
} from "../../format";
import { type LeaderRow, sortIndices } from "../../game";
import type { ChartColumn, ChartLine, ChartTick } from "../../ui/ProgressChart";

/** Ime hotela na ekranu: robot „Brižni (robot)“, tuđi ljudski „… (drugi student)“, moj kakav jest. */
export function hotelLabel(h: SeasonHotelBody): string {
  if (h.mine) return h.name;
  return t(h.seat.kind === "robot" ? "season.robotName" : "season.humanName", { name: h.name });
}

/** Preciznost bodova za cijeli ekran: ista kao kod sezonskog mentora (vidi `progressDigits`). */
export function seasonDigits(body: SeasonBody): PointDigits {
  return progressDigits(body.hotels.map((h) => h.progress_hundredths));
}

function isWeekly(body: SeasonBody): boolean {
  return body.round_kind === "week";
}

/** Imaju li svi hoteli isto mjesto (jednak napredak): tada nitko nije posebno istaknut kao pobjednik (ni pehar). */
export function allTied(body: SeasonBody): boolean {
  return body.hotels.length > 1 && body.hotels.every((h) => h.rank === 1);
}

export type SeasonLeaderRow = LeaderRow & {
  label: string;
  rank: number;
  winner: boolean;
  progressText: string;
  digits: PointDigits;
};

/** Konačni poredak redom mjesta (poslužitelj već šalje redom mjesta, pa slota). */
export function rankingRows(body: SeasonBody): SeasonLeaderRow[] {
  const digits = seasonDigits(body);
  const tied = allTied(body);
  return body.hotels.map((h) => ({
    hotelId: h.hotel_id,
    name: h.name,
    label: hotelLabel(h),
    mine: h.mine,
    strategy: h.seat.strategy,
    progress: h.progress_hundredths,
    rank: h.rank,
    winner: h.winner && !tied,
    progressText: formatPointsPrecise(h.progress_hundredths, digits),
    digits,
  }));
}

export function myHotel(body: SeasonBody): SeasonHotelBody | null {
  return body.hotels.find((h) => h.mine) ?? null;
}

/** „Napredak od početka · 2. mjesto od 4“; kad još netko ima isto mjesto „dijeliš 2. mjesto od 4“. */
export function rankLine(body: SeasonBody): string {
  const me = myHotel(body);
  if (!me) return "";
  const shared = body.hotels.some((h) => !h.mine && h.rank === me.rank);
  return t(shared ? "season.rankShared" : "season.rank", { rank: me.rank, total: body.hotels.length });
}

function joinAnd(items: string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")}${t("season.and")}${items.at(-1)}`;
}

/** Tko je pobijedio (prvi po napretku, Leon 13.10.), rečenicom bez roda prema studentu; isti oblici kao mentor. */
export function winnerLine(body: SeasonBody): string {
  if (allTied(body)) return t("season.winner.all");
  const digits = seasonDigits(body);
  const me = myHotel(body);
  const others = body.hotels.filter((h) => h.winner && !h.mine);
  const names = joinAnd(others.map(hotelLabel));
  if (me?.winner) {
    if (others.length === 0) return t("season.winner.mine");
    return t(others.length === 1 ? "season.winner.mineWith" : "season.winner.mineWithMany", { names });
  }
  const first = others[0];
  if (!first) return "";
  return t(others.length === 1 ? "season.winner.other" : "season.winner.others", {
    names,
    progress: formatPointsPrecise(first.progress_hundredths, digits),
  });
}

/** Crte krivulje redom poretka: stil 0 = moj hotel, 1–3 = ostali redom slota (stil prati hotel, ne mjesto). */
export function curveSeries(body: SeasonBody): ChartLine[] {
  const digits = seasonDigits(body);
  // Stil po slotu (stalan za hotel), a redoslijed legende i tablice po mjestu, kao poredak (UI KS-3 M-9).
  const robotStyles = [1, 2, 3] as const;
  const styleOf = new Map<string, ChartLine["style"]>();
  let next = 0;
  for (const h of [...body.hotels].sort((a, b) => a.slot - b.slot))
    styleOf.set(h.hotel_id, h.mine ? 0 : (robotStyles[Math.min(2, next++)] ?? 3));
  return body.hotels.map((h) => {
    const style = styleOf.get(h.hotel_id) ?? 3;
    return {
      id: h.hotel_id,
      label: hotelLabel(h),
      short: h.name,
      style,
      values: [0, ...h.curve.map((p) => p.progress_hundredths)],
      valueText: formatPointsPrecise(h.progress_hundredths, digits),
    };
  });
}

/** Oznake osi X: „Početak“ (0), pa ime mjeseca na krugu u kojem taj mjesec završava. Nikad kod kruga. */
export function axisTicks(body: SeasonBody): ChartTick[] {
  const ticks: ChartTick[] = [{ at: 0, text: t("season.curve.start"), short: t("season.curve.startShort") }];
  body.rounds.forEach((r, i) => {
    if (r.kind === "month" || r.month_ends)
      ticks.push({
        at: i + 1,
        text: monthName(r.first_month),
        short: t(`season.month.${r.first_month}` as const),
      });
  });
  return ticks;
}

/** Zaglavlja tablice krivulje: kratko („T5“, „Lipanj“) i puni naslov za čitač zaslona. */
export function curveColumns(body: SeasonBody): ChartColumn[] {
  return [
    { short: t("season.curve.start"), full: t("season.curve.start") },
    ...body.rounds.map((r) => {
      const full = roundPhrase(r);
      return r.kind === "week"
        ? { short: t("round.short", { n: r.index + 1 }), full }
        : { short: monthName(r.first_month), full };
    }),
  ];
}

export type IndexRow = { code: IndexCode; start: number; end: number; delta: number };

/** Pet indeksa: početak → kraj u cijelim bodovima 0–100 i promjena kao razlika prikazanih brojeva. */
export function indexRows(h: SeasonHotelBody): IndexRow[] {
  const start = new Map(h.start.map((e) => [e.code, wholePoints(e.hundredths)]));
  return sortIndices(h.end).map((e) => {
    const end = wholePoints(e.hundredths);
    const from = start.get(e.code) ?? end;
    return { code: e.code, start: from, end, delta: end - from };
  });
}

export type FinanceRow = {
  key: "revenue" | "gop" | "cash";
  label: string;
  value: string;
  note: string | null;
};

/** Financije sezone (isti brojevi kao „Račun dobiti → Sezona“); null kad ih poslužitelj ne daje (tuđi student). */
export function financeRows(h: SeasonHotelBody): FinanceRow[] | null {
  const f = h.finances;
  if (!f) return null;
  return [
    { key: "revenue", label: t("season.revenue"), value: formatEuros(f.total_revenue_cents), note: null },
    {
      key: "gop",
      label: t("season.gop"),
      value: formatEuros(f.gop_cents),
      note:
        f.gop_margin_bp === null
          ? null
          : t("season.gopShare", { pct: formatPercentBpFixed(f.gop_margin_bp) }),
    },
    {
      key: "cash",
      label: t("season.cash"),
      value: t("season.cashRange", {
        start: formatEuros(f.cash_start_cents),
        end: formatEuros(f.cash_end_cents),
      }),
      note: null,
    },
  ];
}

export type DecisionKey =
  | "price"
  | "marketing"
  | "wage"
  | "seasonal"
  | "training"
  | "deferred"
  | "reply"
  | "renovation"
  | "situations"
  | "unanswered";

export type DecisionRow = { key: DecisionKey; label: string; value: string };

/** Odluke u brojkama — isti retci za robota i za mene, da se mogu usporediti; null za tuđeg studenta. */
export function decisionRows(h: SeasonHotelBody, body: SeasonBody): DecisionRow[] | null {
  const d = h.decisions;
  if (!d) return null;
  const weekly = isWeekly(body);
  const total = body.rounds.length;
  const of = (n: number) => formatRoundsOf(n, total, weekly);
  const wage =
    d.wage_pct_min === d.wage_pct_max
      ? formatPercentWhole(d.wage_pct_min)
      : formatRange(d.wage_pct_min, d.wage_pct_max, "percent");
  const renovation = d.renovation_round ? body.rounds.find((r) => r.code === d.renovation_round) : undefined;
  const count = formatCount(d.situations, "situation");
  const row = (key: DecisionKey, value: string): DecisionRow => ({
    key,
    label: t(`season.decision.${key}` as const),
    value,
  });
  return [
    row("price", formatEuros(d.avg_price_cents)),
    row("marketing", formatEuros(d.marketing_total_cents)),
    row("wage", t("season.decision.wageValue", { pct: wage })),
    row("seasonal", formatCount(d.seasonal_peak, "person")),
    row("training", of(d.training_rounds)),
    row("deferred", of(d.maintenance_deferred_rounds)),
    row("reply", of(d.reply_reviews_rounds)),
    row("renovation", renovation ? roundPhrase(renovation) : t("season.decision.noRenovation")),
    row(
      "situations",
      d.avg_quality_hundredths === null
        ? count
        : t("season.decision.quality", { count, quality: formatQuality(d.avg_quality_hundredths) }),
    ),
    row("unanswered", formatCount(d.unanswered, "situation")),
  ];
}

/**
 * Kako je robot igrao, riječima (pravilo robota iz `sim-hotel-v2/src/robots.rs`); null za ljudski hotel. Brižni u
 * mjesečnoj igri nema rečenicu o tjednom zapošljavanju (UI KS-3 M-8).
 */
export function strategySentence(h: SeasonHotelBody, weekly: boolean): string | null {
  const s = h.seat.strategy;
  if (h.seat.kind !== "robot" || !s) return null;
  return s === "brizni" && !weekly ? t("season.strategy.brizni.month") : t(`season.strategy.${s}` as const);
}

/** GOP marža za tablicu financija, uvijek s jednom decimalom da se stupac može usporediti („32,0 %“). */
export function gopMargin(h: SeasonHotelBody): string | null {
  const bp = h.finances?.gop_margin_bp;
  return bp === null || bp === undefined ? null : formatPercentBpFixed(bp);
}
