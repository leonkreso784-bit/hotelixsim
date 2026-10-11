// Rezultati odjela po obračunatim krugovima: tablica od 1024 px, ispod toga popis (iste brojke, UI_PRAVILA §4.5).
import { t } from "@hotelixsim/i18n";
import type { DeptRoundBody } from "../../api/types";
import { formatIndex, formatPercentBp, tw } from "../../format";
import { Card } from "../../ui/primitives";
import { resultText } from "./helpers";

const NUM = "whitespace-nowrap px-2 py-2 text-right tabular-nums";
// Zaglavlja stupaca se smiju prelomiti i ćelije imaju uži razmak (px-2), da tablica na 1280 px stane u stupac detalja
// od 550 px (izmjereno: prije 674 px, s prelamanjem 585 px, s px-2 550 px); ne lomi se samo naziv kruga.
const HEAD = "border-b border-line px-2 py-2 text-right text-sm font-semibold text-muted";

function points(pts: number): string {
  return t("dept.outOf", { n: formatIndex(pts) });
}

export function DepartmentHistory({
  history,
  titleOf,
  weekly,
}: {
  history: DeptRoundBody[];
  titleOf: (code: string) => string;
  weekly: boolean;
}) {
  if (history.length === 0) return null;
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{tw(weekly, "dept.history")}</h2>
      {/* biome-ignore lint/a11y/useSemanticElements: vanjski <section> je Card; regija se ne ugnježđuje u drugi section */}
      <div
        role="region"
        aria-label={tw(weekly, "dept.history.table")}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: pomična tablica mora biti dostupna tipkovnicom
        tabIndex={0}
        className="hidden overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent xl:block"
      >
        <table className="w-full text-left">
          <caption className="sr-only">{tw(weekly, "dept.history.table")}</caption>
          <thead>
            <tr>
              <th scope="col" className="border-b border-line px-2 py-2 text-sm font-semibold text-muted">
                {tw(weekly, "dept.history.round")}
              </th>
              <th scope="col" className={HEAD}>
                {t("dept.profit")}
              </th>
              <th scope="col" className={HEAD}>
                {t("dept.margin")}
              </th>
              <th scope="col" className={HEAD}>
                {t("dept.condition")}
              </th>
              <th scope="col" className={HEAD}>
                {t("dept.satisfaction")}
              </th>
            </tr>
          </thead>
          <tbody>
            {history.map((h) => (
              <tr key={h.round} className="border-b border-line">
                <th scope="row" className="whitespace-nowrap px-2 py-2 font-semibold">
                  {titleOf(h.round)}
                </th>
                <td className={NUM}>{resultText(h.profit_cents)}</td>
                <td className={NUM}>{formatPercentBp(h.margin_bp)}</td>
                <td className={NUM}>{points(h.condition_after_pts)}</td>
                <td className={NUM}>{points(h.satisfaction_after_pts)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <ul className="divide-y divide-line xl:hidden">
        {history.map((h) => (
          <li key={h.round} className="space-y-1 py-2">
            <h3 className="font-semibold">{titleOf(h.round)}</h3>
            <dl className="space-y-1 text-sm">
              {[
                [t("dept.profit"), resultText(h.profit_cents)],
                [t("dept.margin"), formatPercentBp(h.margin_bp)],
                [t("dept.condition"), points(h.condition_after_pts)],
                [t("dept.satisfaction"), points(h.satisfaction_after_pts)],
              ].map(([label, value]) => (
                <div key={label} className="flex items-baseline justify-between gap-3">
                  <dt className="min-w-0 text-muted">{label}</dt>
                  <dd className="whitespace-nowrap font-bold tabular-nums">{value}</dd>
                </div>
              ))}
            </dl>
          </li>
        ))}
      </ul>
    </Card>
  );
}
