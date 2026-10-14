// Obnova jednog dijela (odjel ili zgrada) u otvorenom krugu: iznos se bira koracima poslužitelja (− i +), a uz iznos
// stoje rata, učinak na stanje, blagajna nakon plaćanja (s upozorenjem) i amortizacija. Odlučena obnova je zaključana
// do kraja mjeseca: samo rečenica u `role="status"`, bez izbora.
import { t } from "@hotelixsim/i18n";
import { TriangleAlert } from "lucide-react";
import type { RenewalLimitsBody, RenewalStateBody } from "../../api/types";
import { formatEuros } from "../../format";
import { Stepper } from "../departments/Stepper";
import {
  cashWarning,
  depreciationText,
  gainText,
  lockedText,
  optionFor,
  payText,
  stepAmount,
} from "./renewal";

export function RenewalLocked({ state, weekly }: { state: RenewalStateBody; weekly: boolean }) {
  return (
    <p role="status" className="text-sm font-semibold">
      {lockedText(state, weekly)}
    </p>
  );
}

export function RenewalPicker(props: {
  limits: RenewalLimitsBody;
  amount: number;
  subject: "dept" | "building";
  cashCents: number;
  /** Zbroj rata ovog kruga za cijeli plan s ovim iznosom (svi odjeli i zgrada). */
  payNowTotal: number;
  payrollWeekCents: number;
  weekly: boolean;
  /** Id imena dijela (odjel ili „Zgrada“): ponovljeni gumbi se razlikuju po opisu (UI_PRAVILA §6.6). */
  describedBy: string;
  onChange: (amount: number) => void;
}) {
  const l = props.limits;
  const title = t(props.subject === "building" ? "renewal.title.building" : "renewal.title");
  if (l.locked)
    return (
      <div className="space-y-1">
        <p className="font-semibold">{title}</p>
        <RenewalLocked state={l.locked} weekly={props.weekly} />
      </div>
    );
  const more = stepAmount(l, props.amount, 1);
  const less = stepAmount(l, props.amount, -1);
  const o = props.amount > 0 ? optionFor(l, props.amount) : null;
  const after = props.cashCents - props.payNowTotal;
  const warning = o ? cashWarning(after, props.payrollWeekCents) : null;
  return (
    <div className="space-y-1">
      <Stepper
        label={title}
        value={formatEuros(props.amount)}
        moreLabel={t("renewal.more")}
        lessLabel={t("renewal.less")}
        describedBy={props.describedBy}
        moreDisabled={more === null}
        lessDisabled={less === null}
        onMore={() => more !== null && props.onChange(more)}
        onLess={() => less !== null && props.onChange(less)}
      />
      <p className="text-sm text-muted">{t("renewal.intro")}</p>
      {o ? (
        <ul className="space-y-1 text-sm">
          <li>{payText(o, l.rounds_left)}</li>
          <li>{gainText(props.subject, l.condition_pts, o.gain_capped_pts)}</li>
          <li className="tabular-nums">
            {t("renewal.cash", { now: formatEuros(props.cashCents), after: formatEuros(after) })}
          </li>
          <li>{depreciationText(o, l)}</li>
        </ul>
      ) : (
        <p className="text-sm">
          {t("renewal.none", {
            max: formatEuros(l.max_monthly_cents),
            step: formatEuros(l.step_cents),
          })}
        </p>
      )}
      {/* Područje je uvijek u DOM-u, da čitač najavi upozorenje kad se pojavi. */}
      <div role="status">
        {warning && (
          <p className="flex items-start gap-2 rounded-xl bg-sand px-3 py-2 text-sm font-semibold text-ink">
            <TriangleAlert size={18} aria-hidden="true" className="mt-0.5 shrink-0" />
            <span className="min-w-0">{warning}</span>
          </p>
        )}
      </div>
    </div>
  );
}
