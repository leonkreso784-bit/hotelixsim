// Odluke odjela kad krug nije otvoren: samo za čitanje (sadašnje stanje odjela, cijene zadnjeg obračunatog kruga) i
// rečenica kada se mijenjaju. Razinu održavanja poslužitelj ne šalje izvan otvorenog plana, pa je ovdje nema.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody } from "../../api/types";
import { formatCount, formatPercentWhole, tw } from "../../format";
import { Card } from "../../ui/primitives";

export function DecisionsReadOnly({ dept, weekly }: { dept: DepartmentBody; weekly: boolean }) {
  // Smještaj nema razinu cijena (cijena sobe je u planu hotela); najam je ima samo s gostima po korištenju (parking),
  // a lokal ima ugovorenu zakupninu (potražnja 0).
  const priced = dept.last && (dept.class !== "rentals" || dept.last.demand > 0);
  const price =
    dept.last && priced && dept.class !== "rooms"
      ? [[tw(weekly, "dept.decide.lastPrice"), formatPercentWhole(dept.last.price_level_pct)]]
      : [];
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{tw(weekly, "dept.decide")}</h2>
      <p className="text-muted">{tw(weekly, "dept.decide.closed")}</p>
      <dl className="divide-y divide-line">
        {[
          [t("plan.dept.open"), dept.open ? t("dept.decide.yes") : t("dept.decide.no")],
          [t("plan.dept.seasonal"), formatCount(dept.seasonal, "seasonalWorker")],
          ...price,
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
