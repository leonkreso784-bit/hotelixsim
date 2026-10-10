import { t } from "@hotelixsim/i18n";
import type { FeedItemBody, VarCode } from "../../api/types";
import { causeLabel, formatChange, varLabel } from "../../format";
import { Card } from "../../ui/primitives";

export type FeedChange = Extract<FeedItemBody, { kind: "change" }>;

/** Promjene obračuna grupirane po varijabli, redoslijedom kojim ih je jezgra zapisala. */
export function Changes({ items }: { items: FeedChange[] }) {
  const groups = new Map<VarCode, FeedChange[]>();
  for (const c of items) groups.set(c.var, [...(groups.get(c.var) ?? []), c]);
  return (
    <Card>
      <section className="space-y-3">
        <h2 className="text-lg font-bold">{t("month.changes")}</h2>
        {groups.size === 0 && <p className="text-muted">{t("month.changesNone")}</p>}
        {[...groups].map(([v, rows]) => (
          <div key={v}>
            <h3 className="font-semibold">{varLabel(v)}</h3>
            <ul className="text-sm">
              {rows.map((c) => (
                <li
                  key={c.seq}
                  className="flex justify-between gap-3 border-b border-line py-1 last:border-0"
                >
                  <span className="text-muted">
                    {c.cause === "situation" && c.situation
                      ? t("month.situationCause", { code: c.situation })
                      : causeLabel(c.cause)}
                  </span>
                  <span className={`tabular-nums ${c.amount < 0 ? "text-down" : "text-up"}`}>
                    {formatChange(c.amount, c.unit)}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>
    </Card>
  );
}
