import { indexLabel, t } from "@hotelixsim/i18n";
import { Coins, MessageSquare, Settings, User, Users } from "lucide-react";
import type { ReactNode } from "react";
import type { IndexCode, IndexEntry } from "../api/types";
import { formatIndex, formatWholeSigned } from "../format";
import { sortIndices } from "../game";
import { Link } from "../router";

export const INDEX_ICON: Record<IndexCode, ReactNode> = {
  G: <User size={18} aria-hidden="true" />,
  R: <MessageSquare size={18} aria-hidden="true" />,
  Z: <Users size={18} aria-hidden="true" />,
  O: <Settings size={18} aria-hidden="true" />,
  F: <Coins size={18} aria-hidden="true" />,
};

export function Delta({ value, size = "sm" }: { value: number | null; size?: "sm" | "lg" }) {
  if (value === null) return <span />;
  const tone = value > 0 ? "text-up" : value < 0 ? "text-down" : "text-muted";
  const arrow = value > 0 ? "▲ " : value < 0 ? "▼ " : "";
  const text = size === "lg" ? "text-3xl font-extrabold" : "text-sm font-bold";
  return (
    <span
      className={`whitespace-nowrap tabular-nums ${tone} ${text}`}
    >{`${arrow}${formatWholeSigned(value)}`}</span>
  );
}

const ROW = "grid grid-cols-[1.25rem_1fr_auto_auto] items-center gap-x-2 gap-y-1.5 py-2";

export function IndexBars({
  indices,
  deltas,
  deltaLabel,
  linked = false,
}: {
  indices: IndexEntry[];
  deltas: Map<IndexCode, number> | null;
  deltaLabel: string | null;
  /** Cijeli redak vodi na detalj pokazatelja. */
  linked?: boolean;
}) {
  return (
    <div>
      <div className="flex items-baseline justify-between pb-1 text-xs text-muted">
        <span>{t("index.scale")}</span>
        {deltaLabel && <span>{deltaLabel}</span>}
      </div>
      <ul>
        {sortIndices(indices).map((e) => {
          const label = indexLabel(e.code);
          const value = formatIndex(e.hundredths);
          const pct = Math.max(0, Math.min(100, e.hundredths / 100));
          const body = (
            <>
              <span className="text-accent">{INDEX_ICON[e.code]}</span>
              <span className="min-w-0 text-sm">{label}</span>
              <strong className="text-right tabular-nums">{value}</strong>
              <span className="min-w-12 text-right">
                <Delta value={deltas?.get(e.code) ?? null} />
              </span>
              <span
                role="img"
                aria-label={t("index.bar", { label, value })}
                className="relative col-span-full h-2 rounded-full bg-line"
              >
                <span
                  className="absolute inset-y-0 left-0 rounded-full bg-accent"
                  style={{ width: `${pct}%` }}
                />
                <span className="absolute -top-1 left-1/2 h-4 w-0.5 rounded bg-ink" />
              </span>
            </>
          );
          return (
            <li key={e.code}>
              {linked ? (
                <Link
                  to={`/pokazatelj/${e.code}`}
                  className={`${ROW} min-h-11 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent`}
                >
                  {body}
                </Link>
              ) : (
                <div className={ROW}>{body}</div>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
