// FeedScreen.tsx — Tijek: dnevnik hotela, najnovije prvo, po mjesecima.
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { useFeed } from "../api/queries";
import type { FeedItemBody, HotelBody, MonthCode } from "../api/types";
import { capitalize, causeLabel, formatChange, formatSigned, monthName, varLabel } from "../format";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import type { FeedChange } from "./month/Changes";

type Plain = Exclude<FeedItemBody, FeedChange>;
export type FeedEntry =
  | { key: string; month: MonthCode | null; kind: "item"; item: Plain }
  | { key: string; month: MonthCode; kind: "changes"; changes: FeedChange[] };

export function feedEntries(items: FeedItemBody[]): FeedEntry[] {
  const out: FeedEntry[] = [];
  for (const i of items) {
    if (i.kind === "change") {
      const last = out.at(-1);
      if (last?.kind === "changes" && last.month === i.month) last.changes.push(i);
      else out.push({ key: `c${i.seq}`, month: i.month, kind: "changes", changes: [i] });
    } else {
      out.push({ key: `i${i.seq}`, month: "month" in i ? i.month : null, kind: "item", item: i });
    }
  }
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
        progress: formatSigned(i.progress_hundredths),
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
  if (feed.isError) return <ErrorText>{problemMessage(feed.error)}</ErrorText>;
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
                e.kind === "item" ? (
                  <li key={e.key} className="py-2 whitespace-pre-line">
                    {sentence(e.item)}
                  </li>
                ) : (
                  <li key={e.key} className="py-2">
                    <details>
                      <summary className="min-h-11 cursor-pointer font-semibold">
                        {t("feed.changes", { n: e.changes.length })}
                      </summary>
                      <ul className="text-sm">
                        {e.changes.map((c) => (
                          <li key={c.seq} className="flex justify-between gap-3 py-0.5">
                            <span className="text-muted">
                              {varLabel(c.var)} · {causeLabel(c.cause)}
                            </span>
                            <span className="tabular-nums">{formatChange(c.amount, c.unit)}</span>
                          </li>
                        ))}
                      </ul>
                    </details>
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
