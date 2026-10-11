// Četiri kartice odjela s prihodom: Prostor, Gosti, Novac, Ljudi (zadnji obračunati krug).
import { t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import type { DepartmentBody, DeptRoundBody } from "../../api/types";
import {
  formatCount,
  formatEuros,
  formatIndex,
  formatLoad,
  formatPercentBp,
  formatWhole,
  formatWholeSigned,
  tw,
} from "../../format";
import { Card } from "../../ui/primitives";
import { resultText, shownChanges } from "./helpers";

function Lines({ children }: { children: ReactNode }) {
  return <dl className="divide-y divide-line">{children}</dl>;
}

function Line({ label, value, strong = false }: { label: string; value: string; strong?: boolean }) {
  return (
    <div className="flex items-start justify-between gap-3 py-2">
      <dt className={`min-w-0 ${strong ? "font-bold" : ""}`}>{label}</dt>
      <dd className="whitespace-nowrap font-bold tabular-nums">{value}</dd>
    </div>
  );
}

function fromTo(before: number, after: number): string {
  return t("dept.fromTo", { from: formatIndex(before), to: formatIndex(after) });
}

function Space({ r }: { r: DeptRoundBody }) {
  // Poslužitelj šalje promjene s predznakom (trošenje −, održavanje +); ne okreću se, samo se zaokružuju zajedno.
  const [wear = 0, loadWear = 0, upkeep = 0] = shownChanges(
    r.condition_before_pts,
    [r.wear_pts, r.load_wear_pts, r.maintenance_pts],
    r.condition_after_pts,
  );
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.space")}</h2>
      <Lines>
        <Line label={t("dept.condition")} value={fromTo(r.condition_before_pts, r.condition_after_pts)} />
        <Line label={t("dept.upkeepCost")} value={formatEuros(r.maintenance_cents)} />
      </Lines>
      <ul className="space-y-1">
        <li>{t("dept.wear", { n: formatWholeSigned(wear) })}</li>
        <li>{t("dept.loadWear", { n: formatWholeSigned(loadWear) })}</li>
        <li>{t("dept.upkeep", { n: formatWholeSigned(upkeep) })}</li>
      </ul>
      <p className="text-sm text-muted">{t("dept.space.note")}</p>
    </Card>
  );
}

function Guests({ r }: { r: DeptRoundBody }) {
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.guests")}</h2>
      <Lines>
        <Line label={t("dept.served")} value={formatWhole(r.served)} />
        <Line
          label={t("dept.satisfaction")}
          value={fromTo(r.satisfaction_before_pts, r.satisfaction_after_pts)}
        />
        <Line label={t("dept.value")} value={t("dept.outOf", { n: formatIndex(r.value_pts) })} />
        <Line label={t("dept.priceLevel")} value={formatPercentBp(r.price_level_pct * 100)} />
      </Lines>
      <p>{t("dept.usage", { p: formatPercentBp(r.usage_bp) })}</p>
    </Card>
  );
}

function Money({ r, foodAndBeverage }: { r: DeptRoundBody; foodAndBeverage: boolean }) {
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.money")}</h2>
      <Lines>
        <Line label={t("dept.revenue")} value={formatEuros(r.revenue_cents)} />
        <Line label={t("dept.payroll")} value={formatEuros(r.payroll_cents)} />
        <Line
          label={foodAndBeverage ? t("dept.costOfSales.fb") : t("dept.costOfSales")}
          value={formatEuros(r.cost_of_sales_cents)}
        />
        <Line label={t("dept.otherDirect")} value={formatEuros(r.other_cents)} />
        <Line label={t("dept.profit")} value={resultText(r.profit_cents)} strong />
        <Line label={t("dept.margin")} value={formatPercentBp(r.margin_bp)} />
      </Lines>
    </Card>
  );
}

/**
 * Ljudi odjela; kapacitet i opterećenje se pokazuju samo kad postoji obračunati krug. Odlasci zadnjeg kruga
 * (`departures`, `null` prije prvog obračuna) stoje i kad je 0 („nitko“): i mirno osoblje je ishod.
 */
export function People({
  dept,
  r,
  departures,
  weekly,
}: {
  dept: DepartmentBody;
  r: DeptRoundBody | null;
  departures: number | null;
  weekly: boolean;
}) {
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.staff")}</h2>
      <Lines>
        <Line label={t("dept.permanent")} value={formatCount(dept.permanent, "person")} />
        <Line label={t("dept.seasonal")} value={formatCount(dept.seasonal, "person")} />
        {r && (
          <Line label={t("dept.capacity")} value={t("dept.capacityUses", { n: formatWhole(r.capacity) })} />
        )}
        {departures !== null && (
          <Line
            label={tw(weekly, "dept.departures")}
            value={departures > 0 ? formatCount(departures, "person") : t("dept.departures.none")}
          />
        )}
      </Lines>
      {r && <p>{t("dept.load", { load: formatLoad(r.load_hundredths) })}</p>}
      {departures !== null && departures > 0 && (
        <p className="text-sm text-muted">{t("dept.departures.note")}</p>
      )}
    </Card>
  );
}

export function DepartmentFacts({
  dept,
  r,
  weekly,
}: {
  dept: DepartmentBody;
  r: DeptRoundBody;
  weekly: boolean;
}) {
  return (
    <div className="space-y-3 xl:grid xl:grid-cols-2 xl:items-start xl:gap-3 xl:space-y-0">
      <Space r={r} />
      <Guests r={r} />
      <Money r={r} foodAndBeverage={dept.class === "food_beverage"} />
      <People dept={dept} r={r} departures={r.departures} weekly={weekly} />
    </div>
  );
}
