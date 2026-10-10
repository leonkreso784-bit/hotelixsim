import { t } from "@hotelixsim/i18n";
import {
  BedDouble,
  CalendarDays,
  ChartColumn,
  ChevronRight,
  FileText,
  Star,
  Users,
  Wallet,
} from "lucide-react";
import { type ReactNode, useRef } from "react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations, useStartDemo } from "../api/queries";
import type { HotelBody } from "../api/types";
import { formatEuros, formatOccupancy, formatRating, monthName } from "../format";
import { indexDeltas, latestIndices } from "../game";
import { Link } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Badge, Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { CloseMonth } from "./CloseMonth";

function Stat({
  icon,
  label,
  value,
  note,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  note?: string;
}) {
  return (
    <Card className="flex items-center gap-3 p-3">
      <span className="grid size-10 place-items-center rounded-xl bg-accent-soft text-accent">{icon}</span>
      <span className="min-w-0">
        <span className="block text-xs text-muted">{label}</span>
        <strong className="block text-lg tabular-nums">{value}</strong>
        {note && <span className="block truncate text-xs text-muted">{note}</span>}
      </span>
    </Card>
  );
}

function Row({
  to,
  icon,
  label,
  end,
}: {
  to: string;
  icon: ReactNode;
  label: string;
  end?: ReactNode | undefined;
}) {
  return (
    <Link to={to} className="flex min-h-12 items-center gap-3 py-2">
      <span className="text-accent">{icon}</span>
      <span className="flex-1 font-semibold">{label}</span>
      {end}
      <ChevronRight size={18} aria-hidden="true" className="text-muted" />
    </Link>
  );
}

export function HotelScreen({ me }: { me: HotelBody }) {
  const game = useGame(me.game_id);
  const sits = useSituations(me.hotel.hotel_id);
  const start = useStartDemo();
  const inFlight = useRef(false);
  const priv = me.private;
  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
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
      <div>
        <h1 className="text-3xl font-extrabold tracking-tight">{t("hotel.title")}</h1>
        <p className="text-muted">{t("hotel.rooms", { rooms: priv.now.rooms })}</p>
      </div>

      <Card>
        <IndexBars
          indices={latestIndices(h)}
          deltas={deltas}
          deltaLabel={deltas ? t("hotel.fromStart") : null}
        />
      </Card>

      <div className="grid grid-cols-2 gap-3">
        <Stat
          icon={<BedDouble size={20} aria-hidden="true" />}
          label={t("hotel.occupancy")}
          value={lm ? formatOccupancy(lm.sold, lm.available) : "—"}
          note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
        />
        <Stat
          icon={<ChartColumn size={20} aria-hidden="true" />}
          label={t("hotel.revenue")}
          value={lm ? formatEuros(lm.rooms_revenue_cents) : "—"}
          note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
        />
        <Stat
          icon={<Wallet size={20} aria-hidden="true" />}
          label={t("hotel.cash")}
          value={formatEuros(priv.now.cash_cents)}
        />
        <Stat
          icon={<Users size={20} aria-hidden="true" />}
          label={t("hotel.staff")}
          value={String(priv.now.permanent + priv.now.seasonal)}
        />
      </div>

      <Card className="flex items-center gap-3 py-3">
        <Star size={20} aria-hidden="true" className="text-accent" />
        <span className="flex-1 font-semibold">{t("hotel.rating")}</span>
        <strong className="text-lg tabular-nums">{formatRating(priv.now.rating)}</strong>
      </Card>

      <Card className="divide-y divide-line py-1">
        {current && (
          <Row
            to="/situacije"
            icon={<FileText size={20} aria-hidden="true" />}
            label={t("hotel.situations")}
            end={sits.data && sits.data.unanswered > 0 ? <Badge>{sits.data.unanswered}</Badge> : undefined}
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
    </>
  );
}
