// FeedScreen.tsx — Tijek: dnevnik hotela, najnovije prvo, po mjesecima.
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { useFeed } from "../api/queries";
import type { FeedItemBody, HotelBody, MonthCode } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { capitalize, formatPointsCount, monthName } from "../format";
import { Link } from "../router";
import { Card, ErrorText, StatusText } from "../ui/primitives";

type FeedChange = Extract<FeedItemBody, { kind: "change" }>;
type Plain = Exclude<FeedItemBody, FeedChange>;
export type FeedEntry =
  | { key: string; month: MonthCode | null; kind: "item"; item: Plain }
  | { key: string; month: MonthCode; kind: "changes" };

/**
 * Stavke za prikaz, najnovije prvo; uzastopne promjene istog mjeseca su jedan redak (poveznica na „Mjesec završen“).
 * Jedan događaj daje više stavki s istim `seq` (sve promjene obračuna, neodgovorene situacije), pa ključ uz `seq`
 * nosi i mjesto u dnevniku, koje se ne mijenja jer se dnevnik samo nadopisuje.
 */
export function feedEntries(items: FeedItemBody[]): FeedEntry[] {
  const out: FeedEntry[] = [];
  items.forEach((i, n) => {
    const key = `${i.seq}.${n}`;
    if (i.kind === "change") {
      const last = out.at(-1);
      if (last?.kind !== "changes" || last.month !== i.month)
        out.push({ key: `c${key}`, month: i.month, kind: "changes" });
    } else {
      out.push({ key: `i${key}`, month: "month" in i ? i.month : null, kind: "item", item: i });
    }
  });
  return out.reverse();
}

function sentence(i: Plain): string {
  switch (i.kind) {
    case "hotel_received":
      return t("feed.hotel_received", { name: i.name });
    case "plan_changed":
      return t(i.by === "robot" ? "feed.plan_changed.robot" : "feed.plan_changed", {
        month: monthName(i.month),
      });
    case "answer_chosen":
      return t("feed.answer_chosen", { title: i.title, label: i.label });
    case "round_closed":
      return t("feed.round_closed", { month: capitalize(monthName(i.month)) });
    case "situation_ignored":
      return t("feed.situation_ignored", { title: i.title });
    case "month_resolved":
      return t("feed.month_resolved", {
        month: capitalize(monthName(i.month)),
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
  if (feed.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (feed.isError && !feed.data) return <ErrorText>{problemMessage(feed.error)}</ErrorText>;
  const entries = feedEntries(feed.data);
  const groups: { month: MonthCode | null; entries: FeedEntry[] }[] = [];
  for (const e of entries) {
    const g = groups.at(-1);
    if (g && g.month === e.month) g.entries.push(e);
    else groups.push({ month: e.month, entries: [e] });
  }
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("feed.title")}</h1>
      {entries.length === 0 && <p className="text-muted">{t("feed.empty")}</p>}
      {groups.map((g) => (
        <section key={`${g.month ?? "start"}-${g.entries[0]?.key}`} className="space-y-2">
          <h2 className="pt-2 text-sm font-bold tracking-wide text-muted uppercase">
            {g.month ? capitalize(monthName(g.month)) : t("feed.start")}
          </h2>
          <Card className="divide-y divide-line py-1">
            <ol>
              {g.entries.map((e) =>
                e.kind === "item" && e.item.kind === "mentor" ? (
                  <li key={e.key} className="flex items-start gap-2 py-2">
                    <Avatar look={MENTOR_LOOK} size={28} />
                    <span className="min-w-0 flex-1 whitespace-pre-line">{sentence(e.item)}</span>
                  </li>
                ) : e.kind === "item" ? (
                  <li key={e.key} className="py-2 whitespace-pre-line">
                    {sentence(e.item)}
                  </li>
                ) : (
                  <li key={e.key}>
                    <Link
                      to={`/mjesec/${e.month}`}
                      className="flex min-h-11 items-center font-semibold text-accent"
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
