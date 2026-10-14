// „Još odluka“ u planu kruga: marketing, plaće, obuka, stara obnova (v1 i v2 bez obnove iznosom) i odgovaranje na
// recenzije, svaka s cijenom ili učinkom.
import { t } from "@hotelixsim/i18n";
import type { PlanLimitsBody } from "../../api/types";
import { formatEuros, tw } from "../../format";
import { Choice } from "./Choice";
import { trainingEffect, trainingOptions } from "./costs";
import { NumberField, Toggle } from "./fields";
import type { Draft } from "./hotelDraft";
import { Renovation } from "./Renovation";

export function MoreDecisions({
  draft,
  limits: l,
  cashCents,
  weekly,
  renovation,
  set,
}: {
  draft: Draft;
  limits: PlanLimitsBody;
  cashCents: number;
  weekly: boolean;
  /** Prikazuje se stara kvačica obnove. */
  renovation: boolean;
  set: (patch: Partial<Draft>) => void;
}) {
  return (
    <details className="rounded-card border border-line bg-surface p-4">
      <summary className="min-h-11 cursor-pointer font-bold">{t("plan.more")}</summary>
      <div className="mt-3 space-y-4">
        <NumberField
          id="plan-marketing"
          label={t("plan.marketing")}
          value={draft.marketing}
          min={0}
          max={l.marketing_max_cents / 100}
          unit="euro"
          note={tw(weekly, "plan.marketing.note")}
          onChange={(v) => set({ marketing: v })}
        />
        <NumberField
          id="plan-wage"
          label={t("plan.wage")}
          value={draft.wage}
          min={l.wage_min_pct}
          max={l.wage_max_pct}
          unit="percent"
          note={tw(weekly, "plan.wage.note", { amount: formatEuros(l.costs.payroll_round_cents) })}
          onChange={(v) => set({ wage: v })}
        />
        <div className="space-y-1">
          <Choice
            legend={t("plan.training")}
            name="training"
            value={draft.training}
            options={trainingOptions(l.costs)}
            onChange={(v) => set({ training: v })}
          />
          <p className="text-sm">
            {trainingEffect(draft.training, l.costs, weekly) ?? tw(weekly, "plan.cost.levels")}
          </p>
        </div>
        {renovation && (
          <Renovation
            checked={draft.renovation}
            limits={l}
            cashCents={cashCents}
            onChange={(v) => set({ renovation: v })}
          />
        )}
        <Toggle
          id="plan-reply"
          label={t("plan.replyReviews")}
          checked={draft.reply}
          note={
            <p className="text-sm text-muted">
              {tw(weekly, "plan.cost.reply", { amount: formatEuros(l.costs.reply_cents) })}
            </p>
          }
          onChange={(v) => set({ reply: v })}
        />
      </div>
    </details>
  );
}
