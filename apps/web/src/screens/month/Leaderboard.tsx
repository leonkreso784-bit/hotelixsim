import { t } from "@hotelixsim/i18n";
import { ChartColumn } from "lucide-react";
import { Avatar } from "../../avatar/Avatar";
import { lookFor, PLAYER_LOOK } from "../../avatar/look";
import { formatSigned, strategyLabel } from "../../format";
import type { LeaderRow } from "../../game";
import { Card } from "../../ui/primitives";

export function Leaderboard({ rows }: { rows: LeaderRow[] }) {
  return (
    <Card className="p-3">
      <h2 id="ljestvica" className="flex items-center gap-2 px-1 pb-2 text-lg font-bold">
        <ChartColumn size={18} aria-hidden="true" className="text-accent" />
        {t("month.leaderboard")}
      </h2>
      <ol aria-labelledby="ljestvica" className="space-y-1">
        {rows.map((r, i) => (
          <li
            key={r.hotelId}
            aria-current={r.mine ? "true" : undefined}
            className={`flex items-center gap-3 rounded-xl px-2 py-2 ${r.mine ? "bg-accent-soft font-bold text-accent" : ""}`}
          >
            <span className="w-5 tabular-nums">{i + 1}.</span>
            <Avatar look={r.mine ? PLAYER_LOOK : lookFor(r.name)} size={32} />
            <span className="min-w-0 flex-1">
              <span className="block truncate">{r.name}</span>
              {r.strategy && (
                <span className="block text-xs font-normal text-muted">{strategyLabel(r.strategy)}</span>
              )}
            </span>
            <span className={`tabular-nums font-extrabold ${r.progress < 0 ? "text-down" : "text-up"}`}>
              {formatSigned(r.progress)}
            </span>
          </li>
        ))}
      </ol>
    </Card>
  );
}
