// IndicatorScreen — detalj jednog indeksa: što mjeri, put kroz krugove (mjesece ili tjedne) i od čega se sastoji (iz objašnjenja poslužitelja).
import { indexLabel, t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { useState } from "react";
import { problemMessage } from "../api/errors";
import { useExplanation, useGame } from "../api/queries";
import type { DriverCode, GameBody, HotelBody, IndexCode, IndexEntry } from "../api/types";
import { formatIndex, monthName, roundPhrase, roundTitle } from "../format";
import { indexDeltas, isWeekly, roundInfo } from "../game";
import { Link } from "../router";
import { Delta, INDEX_ICON } from "../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { DriverList } from "./month/Explanation";

type HistoryRow = { key: string; label: string; entry: IndexEntry; delta: number | null };
type Scored = HotelBody["hotel"]["months"][number];

/** Naslov obračunatog kruga („Lipanj“, „Tjedan 5 (29. 6. – 5. 7.)“); bez kruga u igri ime mjeseca. */
function scoredTitle(game: GameBody, m: Scored): string {
  const info = roundInfo(game, m.round);
  return info ? roundTitle(info) : monthName(m.month);
}

/** Početak, pa svaki obračunati krug: vrijednost indeksa i promjena prema prethodnom redu. */
function historyRows(hotel: HotelBody["hotel"], code: IndexCode, game: GameBody): HistoryRow[] {
  const rows: HistoryRow[] = [];
  let prev: IndexEntry[] | null = null;
  const steps: { key: string; label: string; indices: IndexEntry[] }[] = [
    { key: "start", label: t("indicator.start"), indices: hotel.start },
    ...hotel.months.map((m) => ({ key: m.round, label: scoredTitle(game, m), indices: m.indices })),
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

function History({ rows, code, weekly }: { rows: HistoryRow[]; code: IndexCode; weekly: boolean }) {
  const label = indexLabel(code);
  return (
    <Card>
      <h2 className="pb-1 text-lg font-bold">{t(weekly ? "indicator.history.week" : "indicator.history")}</h2>
      <ul aria-label={t(weekly ? "indicator.history.week" : "indicator.history")}>
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

/** Izbor obračunatog kruga; tjedni se lome u više redaka (do 17 gumba), mjeseci stanu u jedan. */
function RoundPicker({
  rounds,
  value,
  weekly,
  onPick,
}: {
  rounds: { code: string; label: string }[];
  value: string;
  weekly: boolean;
  onPick: (code: string) => void;
}) {
  return (
    <fieldset className="flex flex-wrap gap-2 pb-2">
      <legend className="sr-only">{t(weekly ? "indicator.pickMonth.week" : "indicator.pickMonth")}</legend>
      {rounds.map((r) => (
        <button
          key={r.code}
          type="button"
          aria-pressed={r.code === value}
          onClick={() => onPick(r.code)}
          className={`min-h-11 rounded-2xl px-4 font-semibold ${r.code === value ? "bg-accent text-white" : "bg-accent-soft text-accent-ink"}`}
        >
          {r.label}
        </button>
      ))}
    </fieldset>
  );
}

function Why({
  hotelId,
  round,
  code,
  highlight,
}: {
  hotelId: string;
  round: string;
  code: IndexCode;
  highlight: DriverCode | null;
}) {
  const q = useExplanation(hotelId, round);
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
  const gameQuery = useGame(me.game_id);
  const [picked, setPicked] = useState<string | null>(null);
  if (gameQuery.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (gameQuery.isError && !gameQuery.data) return <ErrorText>{problemMessage(gameQuery.error)}</ErrorText>;
  const game = gameQuery.data;
  const weekly = isWeekly(game);
  const scored = hotel.months.map((m) => ({ code: m.round, label: scoredTitle(game, m) }));
  const last = scored.at(-1)?.code ?? null;
  const round = picked && scored.some((s) => s.code === picked) ? picked : last;
  const info = round ? roundInfo(game, round) : null;
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
      <History rows={historyRows(hotel, code, game)} code={code} weekly={weekly} />
      <Card>
        {round === null ? (
          <p>{t(weekly ? "indicator.noMonthYet.week" : "indicator.noMonthYet")}</p>
        ) : (
          <>
            <h2 className="pb-2 text-lg font-bold">
              {t("indicator.why", {
                month: info ? roundPhrase(info) : (scored.find((s) => s.code === round)?.label ?? ""),
              })}
            </h2>
            {scored.length > 1 && (
              <RoundPicker rounds={scored} value={round} weekly={weekly} onPick={setPicked} />
            )}
            <Why hotelId={hotel.hotel_id} round={round} code={code} highlight={highlight} />
          </>
        )}
      </Card>
    </>
  );
}
