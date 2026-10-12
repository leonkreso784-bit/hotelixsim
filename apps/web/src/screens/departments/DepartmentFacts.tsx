// Kartice odjela s prihodom: Prostor, Gosti, Novac, Ljudi (zadnji obračunati krug). Najam bez ljudi (parking, lokal)
// nema ni goste po korištenju ni ljude, pa ima samo Prostor i Novac.
import { t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import type { DepartmentBody, DeptRoundBody } from "../../api/types";
import {
  formatCount,
  formatEuros,
  formatIndex,
  formatPercentBp,
  formatPercentWhole,
  formatWholeSigned,
  tw,
} from "../../format";
import { Card } from "../../ui/primitives";
import { isLease, leaseWithoutPeople, resultText, shownChanges } from "./helpers";
import { staffUsePct, usesText } from "./insights";

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

/** Promjena stanja prostora s jedinicom: „−1 bod“, „+2 boda“, „0 bodova“ (UI_PRAVILA §2.1, M8). */
function points(n: number): string {
  return formatCount(n, "point", formatWholeSigned(n));
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
        <li>{t("dept.wear", { n: points(wear) })}</li>
        <li>{t("dept.loadWear", { n: points(loadWear) })}</li>
        <li>{t("dept.upkeep", { n: points(upkeep) })}</li>
      </ul>
      <p className="text-sm text-muted">{t("dept.space.note")}</p>
    </Card>
  );
}

function Guests({ dept, r }: { dept: DepartmentBody; r: DeptRoundBody }) {
  const rooms = dept.class === "rooms";
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.guests")}</h2>
      <Lines>
        <Line label={t("dept.served")} value={usesText(r.served, dept.class)} />
        <Line label={rooms ? t("dept.usage.rooms") : t("dept.usage")} value={formatPercentBp(r.usage_bp)} />
        <Line
          label={t("dept.satisfaction")}
          value={fromTo(r.satisfaction_before_pts, r.satisfaction_after_pts)}
        />
        <Line label={t("dept.value")} value={t("dept.outOf", { n: formatIndex(r.value_pts) })} />
        <Line label={t("dept.priceLevel")} value={formatPercentBp(r.price_level_pct * 100)} />
      </Lines>
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
 * Ljudi odjela; opterećenje osoblja se pokazuje samo kad postoji obračunati krug s gostima, i to istom mjerom kao
 * „Što mu fali / U čemu je dobar“ (postotak granice za dobru uslugu, UI I4). Odlasci zadnjeg kruga (`departures`,
 * `null` prije prvog obračuna) stoje i kad je 0 („nitko“): i mirno osoblje je ishod.
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
  const busy = r !== null && (r.demand > 0 || r.served > 0);
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.staff")}</h2>
      <Lines>
        <Line label={t("dept.permanent")} value={formatCount(dept.permanent, "person")} />
        <Line label={t("dept.seasonal")} value={formatCount(dept.seasonal, "person")} />
        {departures !== null && (
          <Line
            label={tw(weekly, "dept.departures")}
            value={departures > 0 ? formatCount(departures, "person") : t("dept.departures.none")}
          />
        )}
      </Lines>
      <p className="text-sm text-muted">{t("dept.staff.note")}</p>
      {busy && r && (
        <div className="space-y-1">
          <p>{t("dept.load", { p: formatPercentWhole(staffUsePct(r.load_hundredths)) })}</p>
          <p className="text-sm text-muted">{t("dept.load.note")}</p>
        </div>
      )}
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
  // Najam nema goste po korištenju (prihod je ugovoren), a zatvoren odjel ih ovaj krug nije imao (M-11); ljudi se
  // pokazuju kad ih odjel ima (i najam s ljudima ih plaća).
  const guests = !isLease(dept.class) && dept.open;
  return (
    <div className="space-y-3 xl:grid xl:grid-cols-2 xl:items-start xl:gap-3 xl:space-y-0">
      <Space r={r} />
      {guests && <Guests dept={dept} r={r} />}
      <Money r={r} foodAndBeverage={dept.class === "food_beverage"} />
      {!leaseWithoutPeople(dept) && <People dept={dept} r={r} departures={r.departures} weekly={weekly} />}
    </div>
  );
}
