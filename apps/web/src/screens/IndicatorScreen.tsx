// IndicatorScreen — detalj jednog indeksa: što mjeri, put kroz mjesece i od čega se sastoji (iz objašnjenja poslužitelja).
import { indexLabel, t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { useState } from "react";
import { problemMessage } from "../api/errors";
import { useExplanation } from "../api/queries";
import type { DriverCode, HotelBody, IndexCode, IndexEntry, MonthCode } from "../api/types";
import { capitalize, formatIndex, monthName } from "../format";
import { indexDeltas } from "../game";
import { Link } from "../router";
import { Delta, INDEX_ICON } from "../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { DriverList } from "./month/Explanation";

type HistoryRow = { key: string; label: string; entry: IndexEntry; delta: number | null };

/** Početak, pa svaki obračunati mjesec: vrijednost indeksa i promjena prema prethodnom redu. */
function historyRows(hotel: HotelBody["hotel"], code: IndexCode): HistoryRow[] {
  const rows: HistoryRow[] = [];
  let prev: IndexEntry[] | null = null;
  const steps: { key: string; label: string; indices: IndexEntry[] }[] = [
    { key: "start", label: t("indicator.start"), indices: hotel.start },
    ...hotel.months.map((m) => ({ key: m.month, label: capitalize(monthName(m.month)), indices: m.indices })),
  ];
  for (const s of steps) {
    const entry = s.indices.find((e) => e.code === code);
    if (!entry) continue;
    rows.push({
      key: s.key,
      label: s.label,
      entry,
      delta: prev ? (indexDeltas(prev, s.indices).get(code) ?? null) : null,
    });
    prev = s.indices;
  }
  return rows;
}

function History({ rows, code }: { rows: HistoryRow[]; code: IndexCode }) {
  const label = indexLabel(code);
  return (
    <Card>
      <h2 className="pb-1 text-lg font-bold">{t("indicator.history")}</h2>
      <ul aria-label={t("indicator.history")}>
        {rows.map((r) => {
          const pct = Math.max(0, Math.min(100, r.entry.hundredths / 100));
          return (
            <li key={r.key} className="grid grid-cols-[1fr_auto_auto] items-center gap-x-2 gap-y-1.5 py-2">
              <span className="min-w-0 text-sm">{r.label}</span>
              <strong className="text-right tabular-nums">{formatIndex(r.entry.hundredths)}</strong>
              <span className="min-w-12 text-right">
                <Delta value={r.delta} />
              </span>
              <span
                role="img"
                aria-label={t("index.bar", { label, value: formatIndex(r.entry.hundredths) })}
                className="relative col-span-full h-2 rounded-full bg-line"
              >
                <span
                  className="absolute inset-y-0 left-0 rounded-full bg-accent"
                  style={{ width: `${pct}%` }}
                />
                <span className="absolute -top-1 left-1/2 h-4 w-0.5 rounded bg-ink" />
              </span>
            </li>
          );
        })}
      </ul>
    </Card>
  );
}

function MonthPicker({
  months,
  value,
  onPick,
}: {
  months: MonthCode[];
  value: MonthCode;
  onPick: (m: MonthCode) => void;
}) {
  return (
    <fieldset className="flex gap-2 pb-2">
      <legend className="sr-only">{t("indicator.pickMonth")}</legend>
      {months.map((m) => (
        <button
          key={m}
          type="button"
          aria-pressed={m === value}
          onClick={() => onPick(m)}
          className={`min-h-11 rounded-2xl px-4 font-semibold ${m === value ? "bg-accent text-white" : "bg-accent-soft text-accent-ink"}`}
        >
          {capitalize(monthName(m))}
        </button>
      ))}
    </fieldset>
  );
}

function Why({
  hotelId,
  month,
  code,
  highlight,
}: {
  hotelId: string;
  month: MonthCode;
  code: IndexCode;
  highlight: DriverCode | null;
}) {
  const q = useExplanation(hotelId, month);
  const index = q.data?.indices.find((i) => i.code === code);
  return q.isPending ? (
    <StatusText>{t("status.loading")}</StatusText>
  ) : q.isError ? (
    <ErrorText>{problemMessage(q.error)}</ErrorText>
  ) : (
    <DriverList drivers={index?.drivers ?? []} highlight={highlight} />
  );
}

export function IndicatorScreen({
  me,
  code,
  highlight,
}: {
  me: HotelBody;
  code: IndexCode;
  highlight: DriverCode | null;
}) {
  const hotel = me.hotel;
  const scored = hotel.months.map((m) => m.month);
  const last = scored.at(-1) ?? null;
  const [picked, setPicked] = useState<MonthCode | null>(null);
  const month = picked && scored.includes(picked) ? picked : last;
  return (
    <>
      <Link to="/" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink">
        <ChevronLeft size={18} aria-hidden="true" />
        {t("indicator.back")}
      </Link>
      <div>
        <h1 className="flex items-center gap-2 text-3xl font-extrabold tracking-tight">
          <span className="text-accent">{INDEX_ICON[code]}</span>
          {indexLabel(code)}
        </h1>
        <p>{t(`indicator.mean.${code}` as const)}</p>
        <p className="text-xs text-muted">{t("index.scale")}</p>
      </div>
      <History rows={historyRows(hotel, code)} code={code} />
      <Card>
        {month === null ? (
          <p>{t("indicator.noMonthYet")}</p>
        ) : (
          <>
            <h2 className="pb-2 text-lg font-bold">{t("indicator.why", { month: monthName(month) })}</h2>
            {scored.length > 1 && <MonthPicker months={scored} value={month} onPick={setPicked} />}
            <Why hotelId={hotel.hotel_id} month={month} code={code} highlight={highlight} />
          </>
        )}
      </Card>
    </>
  );
}
