// Sažetak odluka po odjelima u tjednom planu: što se u planu kruga razlikuje od sadašnjeg stanja odjela
// („Restoran: +2 sezonska radnika, cijene 105 %“). Kontrole su na ekranu odjela. Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { DepartmentBody, PlanBody } from "../../api/types";
import { formatCount, formatPercentWhole, formatWholeSigned } from "../../format";
import { isService } from "../departments/helpers";

type DeptPlans = NonNullable<PlanBody["departments"]>;

export type DeptChange = { code: string; name: string; text: string };

/** Uobičajena razina cijene u odjelu; druga razina je odluka koja se vidi u sažetku. */
const USUAL_PRICE_PCT = 100;

export function deptChanges(plans: DeptPlans, departments: DepartmentBody[]): DeptChange[] {
  const out: DeptChange[] = [];
  for (const d of departments) {
    const p = plans[d.code];
    if (!p || isService(d.class)) continue;
    const parts: string[] = [];
    if (d.open && !p.open) parts.push(t("plan.dept.closing"));
    else if (!d.open && p.open) parts.push(t("plan.dept.reopening"));
    if (p.open) {
      const delta = p.seasonal_target - d.seasonal;
      if (delta !== 0 && d.open)
        parts.push(formatCount(Math.abs(delta), "seasonalWorker", formatWholeSigned(delta)));
      if (p.price_level_pct !== USUAL_PRICE_PCT)
        parts.push(t("plan.dept.priceChange", { p: formatPercentWhole(p.price_level_pct) }));
      if (p.maintenance !== "regular")
        parts.push(t("plan.dept.maintenanceChange", { level: t(`maintenance.${p.maintenance}` as const) }));
    }
    if (parts.length > 0) out.push({ code: d.code, name: d.name, text: parts.join(", ") });
  }
  return out;
}
