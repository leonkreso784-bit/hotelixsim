// Izbori plana s cijenom kruga (iz granica plana; poslužitelj ih čita iz kataloga). Bez Reacta.
import { t } from "@hotelixsim/i18n";
import type { PlanBody, PlanLimitsBody } from "../../api/types";
import { capitalize, formatEuros, formatSignedPoints, tw } from "../../format";

type Costs = PlanLimitsBody["costs"];
type Levels = Costs["maintenance"];

export function maintenanceOptions(
  levels: Levels,
): { v: PlanBody["maintenance"]; label: string; note: string }[] {
  return [
    { v: "deferred", label: t("maintenance.deferred"), note: formatEuros(levels.deferred_cents) },
    { v: "regular", label: t("maintenance.regular"), note: formatEuros(levels.regular_cents) },
    { v: "intensive", label: t("maintenance.intensive"), note: formatEuros(levels.intensive_cents) },
  ];
}

export function trainingOptions(c: Costs): { v: PlanBody["training"]; label: string; note: string }[] {
  return [
    { v: "none", label: t("training.none"), note: formatEuros(0) },
    { v: "basic", label: t("training.basic"), note: formatEuros(c.training_basic.cost_cents) },
    { v: "full", label: t("training.full"), note: formatEuros(c.training_full.cost_cents) },
  ];
}

/** „Puna obuka: 10.000 € ovaj mjesec; iskustvo osoblja +7, moral +2 od idućeg mjeseca.“; bez obuke ništa. */
export function trainingEffect(level: PlanBody["training"], c: Costs, weekly: boolean): string | null {
  if (level === "none") return null;
  const x = level === "basic" ? c.training_basic : c.training_full;
  return tw(weekly, "plan.cost.training", {
    level: capitalize(t(`training.${level}` as const)),
    amount: formatEuros(x.cost_cents),
    experience: formatSignedPoints(x.experience_pts),
    morale: formatSignedPoints(x.morale_pts),
  });
}
