// Račun dobiti na mobitelu i tabletu (do 1024 px): kartica po sekciji; redak = ime i ostvareno, ispod budžet i razlika riječima.
import { t } from "@hotelixsim/i18n";
import type { PlSectionBody } from "../../api/types";
import { formatEuros } from "../../format";
import { Card } from "../../ui/primitives";
import { sectionRows, sectionTitle } from "./sections";
import { Variance } from "./Variance";

export function PlCards({ sections }: { sections: PlSectionBody[] }) {
  return (
    <div
      data-layout="pl-sections"
      className="space-y-3 md:grid md:grid-cols-2 md:items-start md:gap-3 md:space-y-0 lg:hidden"
    >
      {sections.map((s) => (
        <Card key={s.section} className="min-w-0 space-y-1">
          <h2 className="text-lg font-bold">{sectionTitle(s.section)}</h2>
          <ul className="divide-y divide-line">
            {sectionRows(s).map(({ line, total }) => (
              <li key={line.code} className={`py-2 ${total ? "font-bold" : ""}`}>
                <div className="flex items-baseline justify-between gap-3">
                  <span className="min-w-0">{line.name}</span>
                  <span className="whitespace-nowrap tabular-nums">{formatEuros(line.actual_cents)}</span>
                </div>
                <div className="flex flex-wrap items-baseline gap-x-3 text-sm font-normal">
                  <span className="whitespace-nowrap text-muted">
                    {t("pl.budgetSmall", { amount: formatEuros(line.budget_cents) })}
                  </span>
                  <Variance line={line} />
                </div>
              </li>
            ))}
          </ul>
        </Card>
      ))}
    </div>
  );
}
