// Račun dobiti na računalu (od 1024 px): jedna tablica, sekcije USALI-ja kao grupe redaka, zbirni redak podebljan.
import { t } from "@hotelixsim/i18n";
import type { PlSectionBody } from "../../api/types";
import { formatEuros } from "../../format";
import { Card } from "../../ui/primitives";
import { sectionRows, sectionTitle } from "./sections";
import { Variance } from "./Variance";

const HEAD = "border-b border-line px-3 py-2 text-sm font-semibold text-muted";
const NUM = "whitespace-nowrap px-3 py-2 text-right align-top tabular-nums";

export function PlTable({ sections }: { sections: PlSectionBody[] }) {
  const caption = t("pl.table");
  return (
    <Card className="hidden lg:block" pad="sm">
      {/* biome-ignore lint/a11y/useSemanticElements: vanjski <section> je Card; regija se ne ugnježđuje u drugi section */}
      <div
        role="region"
        aria-label={caption}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: pomična tablica mora biti dostupna tipkovnicom
        tabIndex={0}
        className="overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <table className="w-full border-separate border-spacing-0 text-left">
          <caption className="sr-only">{caption}</caption>
          <thead>
            <tr>
              <th scope="col" className={HEAD}>
                {t("pl.col.item")}
              </th>
              <th scope="col" className={`${HEAD} text-right`}>
                {t("pl.col.actual")}
              </th>
              <th scope="col" className={`${HEAD} text-right`}>
                {t("pl.col.budget")}
              </th>
              <th scope="col" className={`${HEAD} text-right`}>
                {t("pl.col.variance")}
              </th>
            </tr>
          </thead>
          {sections.map((s) => (
            <tbody key={s.section}>
              <tr>
                <th scope="rowgroup" colSpan={4} className="bg-accent-soft px-3 py-2 text-left font-bold">
                  {sectionTitle(s.section)}
                </th>
              </tr>
              {sectionRows(s).map(({ line, total }) => {
                const top = total ? "border-t border-line" : "";
                return (
                  <tr key={line.code} className={total ? "font-bold" : ""}>
                    <th
                      scope="row"
                      className={`w-full min-w-0 px-3 py-2 text-left align-top ${top} ${total ? "font-bold" : "font-normal"}`}
                    >
                      {line.name}
                    </th>
                    <td className={`${NUM} ${top}`}>{formatEuros(line.actual_cents)}</td>
                    <td className={`${NUM} ${top}`}>{formatEuros(line.budget_cents)}</td>
                    <td className={`whitespace-nowrap px-3 py-2 text-right align-top font-semibold ${top}`}>
                      <Variance line={line} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          ))}
        </table>
      </div>
    </Card>
  );
}
