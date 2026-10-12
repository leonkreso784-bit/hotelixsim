import { t } from "@hotelixsim/i18n";
import { ChartColumn, Trophy } from "lucide-react";
import { Avatar } from "../../avatar/Avatar";
import { lookFor, PLAYER_LOOK } from "../../avatar/look";
import { formatPointsCount, type PointDigits, progressTone, strategyLabel } from "../../format";
import type { LeaderRow } from "../../game";
import { Card } from "../../ui/primitives";

/**
 * Redak ljestvice. Kraj sezone dodaje mjesto (jednak napredak dijeli mjesto), pobjednika, ime s oznakom „(robot)“ i
 * napredak već oblikovan preciznošću sezone; ljestvica kruga ih nema i broji mjesta redom.
 */
export type BoardRow = LeaderRow & {
  rank?: number;
  winner?: boolean;
  label?: string;
  progressText?: string;
  /** Broj decimala s kojim je `progressText` oblikovan (boja broja prati prikazani broj). */
  digits?: PointDigits;
};

export function Leaderboard({
  rows,
  className = "",
  title = t("month.leaderboard"),
}: {
  rows: BoardRow[];
  className?: string;
  title?: string;
}) {
  return (
    <Card pad="sm" className={className}>
      <h2 id="ljestvica" className="flex items-center gap-2 px-1 pb-2 text-lg font-bold">
        <ChartColumn size={18} aria-hidden="true" className="text-accent" />
        {title}
      </h2>
      <ol aria-labelledby="ljestvica" className="space-y-1">
        {rows.map((r, i) => (
          <li
            key={r.hotelId}
            aria-current={r.mine ? "true" : undefined}
            className={`flex items-center gap-3 rounded-xl px-2 py-2 ${r.mine ? "bg-accent-soft font-bold text-accent-ink" : ""}`}
          >
            <span className="w-5 tabular-nums">{r.rank ?? i + 1}.</span>
            <Avatar look={r.mine ? PLAYER_LOOK : lookFor(r.name)} size={32} />
            <span className="min-w-0 flex-1">
              <span className="block break-words">{r.label ?? r.name}</span>
              {r.strategy && (
                <span className="block text-xs font-normal text-muted">{strategyLabel(r.strategy)}</span>
              )}
              {r.winner && (
                <span className="flex items-center gap-1 text-sm font-bold text-ink">
                  <Trophy size={18} aria-hidden="true" />
                  {t("season.winner.badge")}
                </span>
              )}
            </span>
            <span
              className={`whitespace-nowrap tabular-nums font-extrabold ${progressTone(r.progress, r.digits ?? 0)}`}
            >
              {r.progressText ?? formatPointsCount(r.progress)}
            </span>
          </li>
        ))}
      </ol>
    </Card>
  );
}
