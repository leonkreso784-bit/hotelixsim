import { t } from "@hotelixsim/i18n";
import { useEffect, useState } from "react";
import { problemMessage } from "../api/errors";
import { poll } from "../api/poll";
import { useFeed, useGameUntilResolved } from "../api/queries";
import type { GameBody, HotelBody, MonthCode, PublicHotelBody } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { capitalize, formatPointsCount, monthName, progressTone } from "../format";
import { indexDeltas, indicesAt, indicesBefore, leaderboard, nextMonth, progressAt } from "../game";
import { Link } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { Explanation } from "./month/Explanation";
import { Leaderboard } from "./month/Leaderboard";

/** Dio ekrana poslije obračuna. Montira se tek kad je moj mjesec obračunat, pa dnevnik i sat za mentora kreću tek tada. */
function MonthResult({
  game,
  mine,
  month,
  progress,
}: {
  game: GameBody;
  mine: PublicHotelBody;
  month: MonthCode;
  progress: number;
}) {
  const feed = useFeed(mine.hotel_id, { untilMentorFor: month });
  // Isti odgovor poslužitelja ne pokreće ponovni prikaz (TanStack dijeli strukturu), pa „nije se javio“ ima vlastiti sat.
  const [mentorGaveUp, setMentorGaveUp] = useState(false);
  useEffect(() => {
    const id = setTimeout(() => setMentorGaveUp(true), poll.maxFeedPolls * poll.feedMs);
    return () => clearTimeout(id);
  }, []);

  const now = indicesAt(mine, month) ?? [];
  const deltas = indexDeltas(indicesBefore(mine, game.months, month), now);
  const items = feed.data ?? [];
  const mentor = items.find((i) => i.kind === "mentor" && i.month === month);
  const next = nextMonth(game.months, month);
  const mentorMissing = mentorGaveUp || feed.isError;

  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">
        {t("month.title", { month: capitalize(monthName(month)) })}
      </h1>
      <div className="flex flex-col gap-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6">
        {/* Na mobitelu stupci su „contents", a order čuva redoslijed: rezultat, indeksi, mentor, ljestvica, objašnjenje. */}
        <div className="contents lg:block lg:space-y-3">
          <Card className="bg-accent-soft py-6 text-center">
            <p className={`whitespace-nowrap text-5xl font-extrabold tabular-nums ${progressTone(progress)}`}>
              {formatPointsCount(progress)}
            </p>
            <p className="mt-1">{t("month.progress")}</p>
          </Card>

          <Card>
            <h2 className="pb-2 font-bold">{t("month.indices")}</h2>
            <IndexBars indices={now} deltas={deltas} deltaLabel={t("month.inMonth")} linked />
          </Card>

          <div className="order-2 contents lg:order-none lg:block">
            <Leaderboard rows={leaderboard(game, month)} />
          </div>
        </div>

        <div className="contents lg:block lg:space-y-3">
          <Card className="order-1 flex items-start gap-3 bg-mint lg:order-none">
            <Avatar look={MENTOR_LOOK} size={56} />
            <div>
              <p className="font-bold text-accent">{t("month.mentor")}</p>
              {mentor?.kind === "mentor" ? (
                <p className="whitespace-pre-line">{mentor.text}</p>
              ) : (
                <p role="status">{mentorMissing ? t("month.mentorMissing") : t("month.mentorWaiting")}</p>
              )}
            </div>
          </Card>

          <div className="order-2 contents lg:order-none lg:block">
            <Explanation hotelId={mine.hotel_id} month={month} />
          </div>
        </div>
      </div>

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

/** Ključ po mjesecu (App.tsx) vraća satove na nulu pri prijelazu jun → jul. */
export function MonthScreen({ me, month }: { me: HotelBody; month: MonthCode }) {
  const hotelId = me.hotel.hotel_id;
  const game = useGameUntilResolved(me.game_id, hotelId, month);
  // „Traje dulje“ ima vlastiti sat jer isti odgovor poslužitelja ne pokreće ponovni prikaz.
  const [slow, setSlow] = useState(false);
  useEffect(() => {
    const id = setTimeout(() => setSlow(true), poll.slowRoundMs);
    return () => clearTimeout(id);
  }, []);
  const name = monthName(month);

  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError && !game.data) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
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

  return <MonthResult game={g} mine={mine} month={month} progress={progress} />;
}
