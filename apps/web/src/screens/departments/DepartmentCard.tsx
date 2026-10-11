// Kartica odjela na popisu: ime (poveznica na cijelu karticu), dvije trake, rezultat zadnjeg kruga i broj ljudi.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { tw } from "../../format";
import { Link } from "../../router";
import { Badge } from "../../ui/primitives";
import { FOCUS, peopleText, resultText } from "./helpers";
import { Meter } from "./Meter";

export function DepartmentCard({
  dept,
  weekly,
  selected,
}: {
  dept: DepartmentBody;
  weekly: boolean;
  selected: boolean;
}) {
  return (
    <article
      className={`relative space-y-3 rounded-card border border-line bg-surface p-4 ${selected ? "ring-2 ring-accent" : ""}`}
    >
      <div className="flex items-start justify-between gap-2">
        <h2 className="min-w-0 text-lg font-bold">
          {/* after:inset-0 razvlači poveznicu preko cijele kartice: jedna meta, jedan Link. */}
          <Link
            to={`/odjeli/${dept.code}`}
            aria-current={selected ? "page" : undefined}
            className={`flex min-h-11 items-center rounded-card after:absolute after:inset-0 after:rounded-card ${FOCUS}`}
          >
            {dept.name}
          </Link>
        </h2>
        {!dept.open && <Badge tone="muted">{t("dept.closed")}</Badge>}
      </div>
      <Meter label={t("dept.condition")} points={dept.condition_pts} />
      {dept.guest_satisfaction_pts !== null && (
        <Meter label={t("dept.satisfaction")} points={dept.guest_satisfaction_pts} />
      )}
      {dept.last && (
        <p className="flex items-baseline justify-between gap-3 text-sm">
          <span className="min-w-0">{tw(weekly, "dept.result")}</span>
          <strong className="whitespace-nowrap tabular-nums">{resultText(dept.last.profit_cents)}</strong>
        </p>
      )}
      <p className="text-sm text-muted">{peopleText(dept.permanent, dept.seasonal)}</p>
    </article>
  );
}
