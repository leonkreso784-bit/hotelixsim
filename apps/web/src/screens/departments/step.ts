// Korak sezonskih radnika za cijeli hotel (`seasonal_step_total`): poslužitelj broji Σ |cilj − sada| svih odjela.
// Ekran jednog odjela smije potrošiti ono što ostali odjeli u planu kruga nisu. Bez Reacta.
import type { PlanBody } from "../../api/types";

export type DeptPlans = NonNullable<PlanBody["departments"]>;

/** Koliko promjene sezonskih radnika ostaje odjelu `code` kad se oduzmu pomaci svih ostalih odjela u planu. */
export function stepLeftForOthers(
  plans: DeptPlans,
  now: Record<string, number>,
  code: string,
  stepTotal: number,
): number {
  const moved = Object.entries(plans)
    .filter(([c]) => c !== code)
    .reduce((sum, [c, d]) => sum + Math.abs(d.seasonal_target - (now[c] ?? 0)), 0);
  return Math.max(0, stepTotal - moved);
}
