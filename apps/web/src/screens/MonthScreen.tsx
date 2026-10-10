import { indexLabel, t } from "@hotelixsim/i18n";
import { useEffect, useState } from "react";
import { problemMessage } from "../api/errors";
import { poll } from "../api/poll";
import { useFeed, useGameUntilResolved } from "../api/queries";
import type { HotelBody, MonthCode } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { capitalize, formatSigned, monthName } from "../format";
import {
  indexDeltas,
  indicesAt,
  indicesBefore,
  leaderboard,
  nextMonth,
  progressAt,
  sortIndices,
} from "../game";
import { Link } from "../router";
import { Delta } from "../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { Changes, type FeedChange } from "./month/Changes";
import { Leaderboard } from "./month/Leaderboard";

export function MonthScreen({ me, month }: { me: HotelBody; month: MonthCode }) {
  const hotelId = me.hotel.hotel_id;
  const game = useGameUntilResolved(me.game_id, hotelId, month);
  const feed = useFeed(hotelId, { untilMentorFor: month });
  // Isti odgovor poslužitelja ne pokreće ponovni prikaz (TanStack dijeli strukturu), pa „traje dulje“ ima vlastiti sat.
  const [slow, setSlow] = useState(false);
  const [mentorGaveUp, setMentorGaveUp] = useState(false);
  useEffect(() => {
    const slowId = setTimeout(() => setSlow(true), poll.slowRoundMs);
    // Ispitivanje mentora staje nakon `maxFeedPolls` pokušaja; isti odgovor ne mijenja prikaz pa ga mjerimo satom.
    const mentorId = setTimeout(() => setMentorGaveUp(true), poll.maxFeedPolls * poll.feedMs);
    return () => {
      clearTimeout(slowId);
      clearTimeout(mentorId);
    };
  }, []);
  const name = monthName(month);

  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  const g = game.data;
  const mine = g.hotels.find((h) => h.hotel_id === hotelId);
  const progress = mine ? progressAt(mine, month) : null;

  if (!mine || progress === null) {
    const closing = g.current?.month === month && g.current.status === "closing";
    if (!closing) return <p className="text-muted">{t("month.notYet")}</p>;
    return (
      <Card className="space-y-2">
        <h1 role="status" className="text-2xl font-extrabold">
          {t("month.closingTitle", { month: name })}
        </h1>
        <p className="text-muted">{t("month.closingText")}</p>
        {slow && <ErrorText>{t("month.slow")}</ErrorText>}
      </Card>
    );
  }

  const now = indicesAt(mine, month) ?? [];
  const deltas = indexDeltas(indicesBefore(mine, g.months, month), now);
  const items = feed.data ?? [];
  const changes = items.filter((i): i is FeedChange => i.kind === "change" && i.month === month);
  const mentor = items.find((i) => i.kind === "mentor" && i.month === month);
  const next = nextMonth(g.months, month);

  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">
        {t("month.title", { month: capitalize(name) })}
      </h1>
      <Card className="bg-accent-soft py-6 text-center">
        <p className="text-6xl font-extrabold text-accent tabular-nums">{formatSigned(progress)}</p>
        <p className="mt-1">{t("month.progress")}</p>
      </Card>

      <Card>
        <h2 className="pb-2 font-bold">{t("month.indices")}</h2>
        <ul className="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {sortIndices(now).map((e) => (
            <li key={e.code} className="rounded-xl border border-line p-2">
              <span className="block text-sm font-semibold">{indexLabel(e.code)}</span>
              <Delta value={deltas.get(e.code) ?? 0} size="lg" />
            </li>
          ))}
        </ul>
      </Card>

      <Card className="flex items-center gap-3 bg-mint">
        <Avatar look={MENTOR_LOOK} size={56} />
        <div>
          <p className="font-bold text-accent">{t("month.mentor")}</p>
          {mentor?.kind === "mentor" ? (
            <p>{mentor.text}</p>
          ) : (
            <p role="status">{mentorGaveUp ? t("month.mentorMissing") : t("month.mentorWaiting")}</p>
          )}
        </div>
      </Card>

      <Leaderboard rows={leaderboard(g, month)} />
      <Changes items={changes} />

      {next ? (
        <Link
          to="/"
          className="flex min-h-12 items-center justify-center rounded-2xl bg-accent font-bold text-white"
        >
          {t("month.continue", { month: monthName(next) })}
        </Link>
      ) : (
        <>
          <p className="font-semibold">{t("month.seasonOver")}</p>
          <Link
            to="/"
            className="flex min-h-12 items-center justify-center rounded-2xl bg-accent-soft font-bold text-accent"
          >
            {t("month.back")}
          </Link>
        </>
      )}
    </>
  );
}
