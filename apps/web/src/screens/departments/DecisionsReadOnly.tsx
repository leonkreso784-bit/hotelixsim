// Odluke odjela kad krug nije otvoren: samo za čitanje (sadašnje stanje odjela) i rečenica kada se mijenjaju.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { formatCount, tw } from "../../format";
import { Card } from "../../ui/primitives";

export function DecisionsReadOnly({ dept, weekly }: { dept: DepartmentBody; weekly: boolean }) {
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{tw(weekly, "dept.decide")}</h2>
      <p className="text-muted">{tw(weekly, "dept.decide.closed")}</p>
      <dl className="divide-y divide-line">
        {[
          [t("plan.dept.open"), dept.open ? t("dept.decide.yes") : t("dept.decide.no")],
          [t("plan.dept.seasonal"), formatCount(dept.seasonal, "seasonalWorker")],
        ].map(([label, value]) => (
          <div key={label} className="flex items-start justify-between gap-3 py-2">
            <dt className="min-w-0">{label}</dt>
            <dd className="whitespace-nowrap font-bold tabular-nums">{value}</dd>
          </div>
        ))}
      </dl>
    </Card>
  );
}
