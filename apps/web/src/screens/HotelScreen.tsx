import { t } from "@hotelixsim/i18n";
import { BedDouble, CalendarDays, ChartColumn, FileText, Gauge, Star, Users, Wallet } from "lucide-react";
import { useRef } from "react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations, useStartDemo } from "../api/queries";
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
} from "../format";
import { indexDeltas, latestIndices } from "../game";
import { Link } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Badge, Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { CloseMonth } from "./CloseMonth";
import { Row, Stat } from "./hotel/links";

export function HotelScreen({ me }: { me: HotelBody }) {
  const game = useGame(me.game_id);
  const sits = useSituations(me.hotel.hotel_id);
  const start = useStartDemo();
  const inFlight = useRef(false);
  const priv = me.private;
  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError && !game.data) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  if (!priv) return <ErrorText>{t("error.generic")}</ErrorText>;

  const h = me.hotel;
  const current = game.data.current;
  const lastScored = h.months.at(-1);
  const deltas = lastScored ? indexDeltas(h.start, lastScored.indices) : null;
  const lm = priv.last_month;

  function newGame() {
    if (start.isPending || inFlight.current) return;
    inFlight.current = true;
    start.mutate(undefined, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

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
              note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
            />
            <Stat
              to="/pokazatelj/F/revenue"
              icon={<ChartColumn size={20} aria-hidden="true" />}
              label={t("hotel.revenue")}
              value={lm ? formatEuros(lm.rooms_revenue_cents) : "—"}
              note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
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

          <Card className="divide-y divide-line py-1">
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

          <Card className="divide-y divide-line py-1">
            {current && (
              <Row
                to="/situacije"
                icon={<FileText size={20} aria-hidden="true" />}
                label={t("hotel.situations")}
                end={
                  sits.data && sits.data.unanswered > 0 ? <Badge>{sits.data.unanswered}</Badge> : undefined
                }
              />
            )}
            {current?.status === "open" && (
              <Row to="/plan" icon={<CalendarDays size={20} aria-hidden="true" />} label={t("hotel.plan")} />
            )}
            {lastScored && (
              <Row
                to={`/mjesec/${lastScored.month}`}
                icon={<ChartColumn size={20} aria-hidden="true" />}
                label={t("hotel.result", { month: monthName(lastScored.month) })}
              />
            )}
          </Card>

          {current?.status === "open" && (
            <CloseMonth
              gameId={me.game_id}
              month={current.month}
              unanswered={sits.data?.unanswered ?? current.unanswered}
            />
          )}
          {current?.status === "closing" && (
            <Card className="space-y-2">
              <p role="status">{t("hotel.closing", { month: monthName(current.month) })}</p>
              <Link
                to={`/mjesec/${current.month}`}
                className="inline-flex min-h-11 items-center font-semibold text-accent"
              >
                {t("hotel.closingLink")}
              </Link>
            </Card>
          )}
          {!current && (
            <Card className="space-y-3">
              <p className="text-lg font-bold">{t("hotel.seasonOver")}</p>
              {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
              <Button disabled={start.isPending} onClick={newGame}>
                {t("hotel.newGame")}
              </Button>
            </Card>
          )}
        </div>
      </div>
    </>
  );
}
