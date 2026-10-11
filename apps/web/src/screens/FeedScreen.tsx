// FeedScreen.tsx — Tijek: dnevnik hotela, najnovije prvo, po krugovima (mjesecima ili tjednima).
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { useFeed, useGame } from "../api/queries";
import type { FeedItemBody, GameBody, HotelBody, MonthCode } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { capitalize, formatPointsCount, roundPhrase, roundWithoutGame } from "../format";
import { roundInfo } from "../game";
import { Link, roundPath } from "../router";
import { Card, ErrorText, StatusText } from "../ui/primitives";

type FeedChange = Extract<FeedItemBody, { kind: "change" }>;
type Plain = Exclude<FeedItemBody, FeedChange>;
export type FeedEntry =
  | { key: string; month: MonthCode | null; round: string | null; kind: "item"; item: Plain }
  | { key: string; month: MonthCode; round: string; kind: "changes" };

/**
 * Stavke za prikaz, najnovije prvo; uzastopne promjene istog kruga su jedan redak (poveznica na ekran kruga).
 * Jedan događaj daje više stavki s istim `seq` (sve promjene obračuna, neodgovorene situacije), pa ključ uz `seq`
 * nosi i mjesto u dnevniku, koje se ne mijenja jer se dnevnik samo nadopisuje.
 */
export function feedEntries(items: FeedItemBody[]): FeedEntry[] {
  const out: FeedEntry[] = [];
  items.forEach((i, n) => {
    const key = `${i.seq}.${n}`;
    if (i.kind === "change") {
      const last = out.at(-1);
      if (last?.kind !== "changes" || last.round !== i.round)
        out.push({ key: `c${key}`, month: i.month, round: i.round, kind: "changes" });
    } else {
      out.push({
        key: `i${key}`,
        month: "month" in i ? i.month : null,
        round: "round" in i ? i.round : null,
        kind: "item",
        item: i,
      });
    }
  });
  return out.reverse();
}

/**
 * Krug usred rečenice: iz igre („tjedan 5 (29. 6. – 5. 7.)“). Dok igra ne stigne, tjedni krug (kod „t05“) nosi samo
 * broj („tjedan 5“), a mjesečni ime mjeseca; mjesec iz dnevnika za tjedan bi bio kriv („lipanj“ za svih 17 krugova).
 */
function phrase(game: GameBody | undefined, round: string, month: MonthCode): string {
  const info = game ? roundInfo(game, round) : null;
  return info ? roundPhrase(info) : roundWithoutGame(round, month);
}

function sentence(i: Plain, game: GameBody | undefined): string {
  switch (i.kind) {
    case "hotel_received":
      return t("feed.hotel_received", { name: i.name });
    case "plan_changed":
      return t(i.by === "robot" ? "feed.plan_changed.robot" : "feed.plan_changed", {
        month: phrase(game, i.round, i.month),
      });
    case "answer_chosen":
      return t("feed.answer_chosen", { title: i.title, label: i.label });
    case "round_closed":
      return t("feed.round_closed", { month: capitalize(phrase(game, i.round, i.month)) });
    case "situation_ignored":
      return t("feed.situation_ignored", { title: i.title });
    case "month_resolved":
      return t("feed.month_resolved", {
        month: capitalize(phrase(game, i.round, i.month)),
        progress: formatPointsCount(i.progress_hundredths),
      });
    case "response":
      return t("feed.response", { title: i.title });
    case "verdict":
      return t(i.late ? "feed.verdict.late" : i.neutral ? "feed.verdict.neutral" : "feed.verdict", {
        title: i.title,
        q: i.quality,
      });
    case "mentor":
      return t("feed.mentor", { text: i.text });
  }
}

export function FeedScreen({ me }: { me: HotelBody }) {
  const feed = useFeed(me.hotel.hotel_id);
  const game = useGame(me.game_id).data;
  if (feed.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (feed.isError && !feed.data) return <ErrorText>{problemMessage(feed.error)}</ErrorText>;
  const entries = feedEntries(feed.data);
  const groups: { round: string | null; month: MonthCode | null; entries: FeedEntry[] }[] = [];
  for (const e of entries) {
    const g = groups.at(-1);
    if (g && g.round === e.round) g.entries.push(e);
    else groups.push({ round: e.round, month: e.month, entries: [e] });
  }
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("feed.title")}</h1>
      {entries.length === 0 && <p className="text-muted">{t("feed.empty")}</p>}
      {groups.map((g) => (
        <section key={`${g.round ?? "start"}-${g.entries[0]?.key}`} className="space-y-2">
          <h2 className="pt-2 text-sm font-bold tracking-wide text-muted uppercase">
            {g.round && g.month ? capitalize(phrase(game, g.round, g.month)) : t("feed.start")}
          </h2>
          <Card pad="list" className="divide-y divide-line">
            <ol>
              {g.entries.map((e) =>
                e.kind === "item" && e.item.kind === "mentor" ? (
                  <li key={e.key} className="flex items-start gap-2 py-2">
                    <Avatar look={MENTOR_LOOK} size={28} />
                    <span className="min-w-0 flex-1 whitespace-pre-line">{sentence(e.item, game)}</span>
                  </li>
                ) : e.kind === "item" ? (
                  <li key={e.key} className="py-2 whitespace-pre-line">
                    {sentence(e.item, game)}
                  </li>
                ) : (
                  <li key={e.key}>
                    <Link
                      to={roundPath(e.round)}
                      className="flex min-h-11 items-center font-semibold text-accent-ink"
                    >
                      {t("feed.why")}
                    </Link>
                  </li>
                ),
              )}
            </ol>
          </Card>
        </section>
      ))}
    </>
  );
}
