import { t } from "@hotelixsim/i18n";
import {
  BedDouble,
  CalendarDays,
  ChartColumn,
  FileText,
  Gauge,
  Star,
  Trophy,
  Users,
  Wallet,
} from "lucide-react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations } from "../api/queries";
import type { HotelBody } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { PLAYER_LOOK } from "../avatar/look";
import {
  formatCount,
  formatEuros,
  formatEurosCents,
  formatPercentBp,
  formatRating,
  monthName,
  roundPhrase,
  tw,
} from "../format";
import { indexDeltas, isV2, isWeekly, latestIndices, roundInfo } from "../game";
import { Link, roundPath } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Badge, Card, ErrorText, StatusText } from "../ui/primitives";
import { CloseMonth } from "./CloseMonth";
import { Row, Stat } from "./hotel/links";

export function HotelScreen({ me }: { me: HotelBody }) {
  const game = useGame(me.game_id);
  const sits = useSituations(me.hotel.hotel_id);
  const priv = me.private;
  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError && !game.data) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  if (!priv) return <ErrorText>{t("error.generic")}</ErrorText>;

  const h = me.hotel;
  const current = game.data.current;
  const weekly = isWeekly(game.data);
  const lastScored = h.months.at(-1);
  const deltas = lastScored ? indexDeltas(h.start, lastScored.indices) : null;
  const lm = priv.last_month;
  const lastInfo = lastScored ? roundInfo(game.data, lastScored.round) : null;
  const lastNote = lm
    ? weekly
      ? t("hotel.lastWeek", { week: roundPhrase(lm.round) })
      : t("hotel.lastMonth", { month: monthName(lm.month) })
    : t(weekly ? "hotel.noWeekYet" : "hotel.noMonthYet");

  return (
    <>
      <div className="flex items-center gap-3">
        <Avatar look={PLAYER_LOOK} size={48} />
        <div className="min-w-0">
          <h1 className="text-3xl font-extrabold tracking-tight">{t("hotel.title")}</h1>
          <p className="text-muted">{t("hotel.you")}</p>
          <p className="text-muted">{t("hotel.rooms", { rooms: priv.now.rooms })}</p>
        </div>
      </div>

      <div className="space-y-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6 lg:space-y-0">
        <Card>
          <IndexBars
            indices={latestIndices(h)}
            deltas={deltas}
            deltaLabel={deltas ? t("hotel.fromStart") : null}
            linked
          />
        </Card>

        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <Stat
              to="/pokazatelj/F/sold"
              icon={<BedDouble size={20} aria-hidden="true" />}
              label={t("hotel.occupancy")}
              value={lm ? formatPercentBp(lm.occupancy_bp) : "—"}
              note={lastNote}
            />
            <Stat
              to="/pokazatelj/F/revenue"
              icon={<ChartColumn size={20} aria-hidden="true" />}
              label={t("hotel.revenue")}
              value={lm ? formatEuros(lm.rooms_revenue_cents) : "—"}
              note={lastNote}
            />
            <Stat
              to="/pokazatelj/F/cash"
              icon={<Wallet size={20} aria-hidden="true" />}
              label={t("hotel.cash")}
              value={formatEuros(priv.now.cash_cents)}
            />
            <Stat
              to="/pokazatelj/Z/staff"
              icon={<Users size={20} aria-hidden="true" />}
              label={t("hotel.staff")}
              value={formatCount(priv.now.permanent + priv.now.seasonal, "person")}
            />
          </div>

          <Card pad="list" className="divide-y divide-line">
            <Row
              to="/pokazatelj/R/rating"
              icon={<Star size={20} aria-hidden="true" />}
              label={t("hotel.rating")}
              end={<strong className="text-lg tabular-nums">{formatRating(priv.now.rating)}</strong>}
            />
            <Row
              to="/pokazatelji"
              icon={<Gauge size={20} aria-hidden="true" />}
              label={t("hotel.kpi")}
              end={
                lm ? (
                  <span className="text-right text-sm text-muted tabular-nums">
                    {t("hotel.kpiValues", {
                      adr: formatEurosCents(lm.adr_cents),
                      revpar: formatEurosCents(lm.revpar_cents),
                    })}
                  </span>
                ) : undefined
              }
            />
          </Card>

          <Card pad="list" className="divide-y divide-line">
            {current && (
              <Row
                to="/situacije"
                icon={<FileText size={20} aria-hidden="true" />}
                label={tw(weekly, "hotel.situations")}
                end={
                  sits.data && sits.data.unanswered > 0 ? <Badge>{sits.data.unanswered}</Badge> : undefined
                }
              />
            )}
            {current?.status === "open" && (
              <Row
                to="/plan"
                icon={<CalendarDays size={20} aria-hidden="true" />}
                label={tw(weekly, "hotel.plan")}
              />
            )}
            {lastScored && (
              <Row
                to={roundPath(lastScored.round)}
                icon={<ChartColumn size={20} aria-hidden="true" />}
                label={t("hotel.result", {
                  month: lastInfo ? roundPhrase(lastInfo) : monthName(lastScored.month),
                })}
              />
            )}
          </Card>

          {current?.status === "open" && (
            <CloseMonth
              gameId={me.game_id}
              hotelId={me.hotel.hotel_id}
              round={current.round}
              unanswered={sits.data?.unanswered ?? current.unanswered}
            />
          )}
          {current?.status === "closing" && (
            <Card className="space-y-2">
              <p role="status">
                {weekly
                  ? t("hotel.closing.week", { round: roundPhrase(current.round) })
                  : t("hotel.closing", { month: monthName(current.month) })}
              </p>
              <Link
                to={roundPath(current.round.code)}
                className="inline-flex min-h-11 items-center font-semibold text-accent-ink"
              >
                {t("hotel.closingLink")}
              </Link>
            </Card>
          )}
          {!current && (
            <Card className="space-y-3">
              <p className="text-lg font-bold">{t("hotel.seasonOver")}</p>
              {isV2(game.data) && (
                <div className="border-y border-line">
                  <Row
                    to="/kraj-sezone"
                    icon={<Trophy size={20} aria-hidden="true" />}
                    label={t("season.results")}
                  />
                </div>
              )}
              <Link
                to="/nova"
                className="flex min-h-12 w-full items-center justify-center rounded-2xl bg-accent px-4 py-3 text-base font-bold text-white"
              >
                {t("hotel.newGame")}
              </Link>
            </Card>
          )}
        </div>
      </div>
    </>
  );
}
