import { t } from "@hotelixsim/i18n";
import { useEffect, useState } from "react";
import { problemMessage } from "../api/errors";
import { poll } from "../api/poll";
import { useFeed, useGameUntilResolved } from "../api/queries";
import type { GameBody, HotelBody, PublicHotelBody, RoundInfoBody } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { formatPointsCount, progressTone, roundDates, roundPhrase, roundTitle } from "../format";
import {
  indexDeltas,
  indicesAt,
  indicesBefore,
  isWeekRound,
  leaderboard,
  nextRound,
  progressAt,
  roundInfo,
} from "../game";
import { Link } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { Explanation } from "./month/Explanation";
import { Leaderboard } from "./month/Leaderboard";

/**
 * Dio ekrana poslije obračuna. Montira se tek kad je moj krug obračunat, pa dnevnik i sat za mentora kreću tek tada.
 * Mentor se javlja samo na kraju mjeseca; tjedan usred mjeseca ga ne čeka.
 */
function MonthResult({
  game,
  mine,
  round,
  progress,
}: {
  game: GameBody;
  mine: PublicHotelBody;
  round: RoundInfoBody;
  progress: number;
}) {
  const code = round.code;
  const weekly = isWeekRound(round);
  const feed = useFeed(mine.hotel_id, { untilMentorFor: code });
  // Isti odgovor poslužitelja ne pokreće ponovni prikaz (TanStack dijeli strukturu), pa „nije se javio“ ima vlastiti sat.
  const [mentorGaveUp, setMentorGaveUp] = useState(false);
  useEffect(() => {
    const id = setTimeout(() => setMentorGaveUp(true), poll.maxFeedPolls * poll.feedMs);
    return () => clearTimeout(id);
  }, []);

  const codes = game.rounds.map((r) => r.code);
  const now = indicesAt(mine, code) ?? [];
  const deltas = indexDeltas(indicesBefore(mine, codes, code), now);
  const items = feed.data ?? [];
  const mentor = items.find((i) => i.kind === "mentor" && i.round === code);
  const nextCode = nextRound(codes, code);
  const next = nextCode ? roundInfo(game, nextCode) : null;
  const mentorMissing = mentorGaveUp || feed.isError;

  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">
        {weekly
          ? t("round.withDates", {
              title: t("round.done.week", { n: round.index + 1 }),
              dates: roundDates(round),
            })
          : t("month.title", { month: roundTitle(round) })}
      </h1>
      <div className="flex flex-col gap-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6">
        {/* Na mobitelu su stupci „contents", pa redoslijed određuje `order` na SAMIM karticama (na omotaču s
            `contents` ne djeluje): rezultat, indeksi, mentor, ljestvica, objašnjenje. Od lg vrijede stupci. */}
        <div className="contents lg:block lg:space-y-3">
          <Card tone="accent" pad="lg" className="text-center">
            <p className={`whitespace-nowrap text-5xl font-extrabold tabular-nums ${progressTone(progress)}`}>
              {formatPointsCount(progress)}
            </p>
            <p className="mt-1">{t("month.progress")}</p>
          </Card>

          <Card>
            <h2 className="pb-2 font-bold">{t(weekly ? "round.indices.week" : "month.indices")}</h2>
            <IndexBars
              indices={now}
              deltas={deltas}
              deltaLabel={t(weekly ? "round.inRound.week" : "month.inMonth")}
              linked
            />
          </Card>

          <Leaderboard rows={leaderboard(game, code)} className="order-2 lg:order-none" />
        </div>

        <div className="contents lg:block lg:space-y-3">
          <Card tone="mint" className="order-1 flex items-start gap-3 lg:order-none">
            <Avatar look={MENTOR_LOOK} size={56} />
            <div>
              <p className="font-bold text-accent-ink">
                {t(weekly && !round.month_ends ? "round.mentorShort" : "month.mentor")}
              </p>
              {mentor?.kind === "mentor" ? (
                <p className="whitespace-pre-line">{mentor.text}</p>
              ) : (
                <p role="status">{mentorMissing ? t("month.mentorMissing") : t("month.mentorWaiting")}</p>
              )}
            </div>
          </Card>

          <Explanation
            hotelId={mine.hotel_id}
            round={code}
            weekly={weekly}
            className="order-3 lg:order-none"
          />
        </div>
      </div>

      {next ? (
        <Link
          to="/"
          className="flex min-h-12 items-center justify-center rounded-2xl bg-accent font-bold text-white"
        >
          {t("month.continue", { month: roundPhrase(next) })}
        </Link>
      ) : (
        <>
          <p className="font-semibold">{t(weekly ? "round.seasonOver.week" : "month.seasonOver")}</p>
          <Link
            to="/"
            className="flex min-h-12 items-center justify-center rounded-2xl bg-accent-soft font-bold text-accent-ink"
          >
            {t("month.back")}
          </Link>
        </>
      )}
    </>
  );
}

/** Ključ po krugu (App.tsx) vraća satove na nulu pri prijelazu u sljedeći krug. */
export function MonthScreen({ me, code }: { me: HotelBody; code: string }) {
  const hotelId = me.hotel.hotel_id;
  const game = useGameUntilResolved(me.game_id, hotelId, code);
  // „Traje dulje“ ima vlastiti sat jer isti odgovor poslužitelja ne pokreće ponovni prikaz.
  const [slow, setSlow] = useState(false);
  useEffect(() => {
    const id = setTimeout(() => setSlow(true), poll.slowRoundMs);
    return () => clearTimeout(id);
  }, []);
  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError && !game.data) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  const g = game.data;
  const mine = g.hotels.find((h) => h.hotel_id === hotelId);
  const round = roundInfo(g, code);
  const progress = mine ? progressAt(mine, code) : null;
  const weekly = round ? isWeekRound(round) : false;

  if (!round || !mine || progress === null) {
    const closing = g.current?.round.code === code && g.current.status === "closing";
    if (!round || !closing)
      return <p className="text-muted">{t(weekly ? "round.notYet.week" : "month.notYet")}</p>;
    return (
      <Card className="space-y-2">
        <h1 role="status" className="text-2xl font-extrabold">
          {weekly
            ? t("round.closingTitle.week", { n: round.index + 1 })
            : t("month.closingTitle", { month: roundPhrase(round) })}
        </h1>
        <p className="text-muted">{t("month.closingText")}</p>
        {slow && <ErrorText>{t("month.slow")}</ErrorText>}
      </Card>
    );
  }

  return <MonthResult game={g} mine={mine} round={round} progress={progress} />;
}
