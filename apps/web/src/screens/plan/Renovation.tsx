// Obnova u planu: cijena i učinak su uvijek uz kvačicu; kad je uključena, i blagajna prije i poslije, s upozorenjem
// kad bi blagajna pala u minus ili ispod jednog tjedna plaća. Kvačica radi na jedan klik (bez skočnog prozora).
import { t } from "@hotelixsim/i18n";
import { TriangleAlert } from "lucide-react";
import type { PlanLimitsBody } from "../../api/types";
import { formatEuros, formatSignedPoints } from "../../format";
import { Toggle } from "./fields";

export function Renovation({
  checked,
  limits,
  cashCents,
  onChange,
}: {
  checked: boolean;
  limits: PlanLimitsBody;
  cashCents: number;
  onChange: (v: boolean) => void;
}) {
  const c = limits.costs;
  const after = cashCents - c.renovation_cents;
  const warning =
    after < 0
      ? t("plan.renovation.negative")
      : after < c.payroll_week_cents
        ? t("plan.renovation.thin", { payroll: formatEuros(c.payroll_week_cents) })
        : null;
  return (
    <Toggle
      id="plan-renovation"
      label={t("plan.renovation")}
      checked={checked}
      disabled={!limits.renovation_available}
      onChange={onChange}
      note={
        limits.renovation_available ? (
          <div className="space-y-1 text-sm">
            <p className="text-muted">
              {t("plan.renovation.note", {
                amount: formatEuros(c.renovation_cents),
                equipment: formatSignedPoints(c.renovation_equipment_pts),
                satisfaction: formatSignedPoints(c.renovation_satisfaction_pts),
              })}
            </p>
            {checked && (
              <p className="tabular-nums">
                {t("plan.renovation.cash", { now: formatEuros(cashCents), after: formatEuros(after) })}
              </p>
            )}
            {/* Područje je uvijek u DOM-u, da čitač najavi upozorenje kad se pojavi. */}
            <div role="status">
              {checked && warning && (
                <p className="flex items-start gap-2 rounded-xl bg-sand px-3 py-2 font-semibold text-ink">
                  <TriangleAlert size={18} aria-hidden="true" className="mt-0.5 shrink-0" />
                  <span>{warning}</span>
                </p>
              )}
            </div>
          </div>
        ) : (
          <p className="text-xs text-muted">{t("plan.renovationUsed")}</p>
        )
      }
    />
  );
}
