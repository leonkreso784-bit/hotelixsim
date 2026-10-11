// DepartmentPlan — dio „Odjeli“ u planu kruga (samo v2). Kontrolirana komponenta: ne šalje ništa sama, plan šalje
// ekran jednim „Spremi“. Granice (sezonci, razina cijene, smije li se zatvoriti, ukupni korak) dolaze od poslužitelja.
import { t } from "@hotelixsim/i18n";
import { useEffect } from "react";
import { problemMessage } from "../../api/errors";
import { useDepartments } from "../../api/queries";
import type { PlanBody, PlanLimitsBody } from "../../api/types";
import { formatCount, formatPercentWhole, formatRange, tw } from "../../format";
import { Button, Card, ErrorText, StatusText } from "../../ui/primitives";
import { FOCUS, isService } from "../departments/helpers";
import { Choice } from "./Choice";

export type DeptPlans = NonNullable<PlanBody["departments"]>;
type DeptLimits = NonNullable<PlanLimitsBody["departments"]>[string];

const PRICE_STEP = 5;

const STEP_BUTTON =
  "inline-flex size-11 items-center justify-center rounded-xl border border-line bg-surface text-xl font-bold focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40";

/** Koliko je sezonaca ukupno pomaknuto od sadašnjeg stanja (Σ |cilj − sada|), kako računa poslužitelj. */
function movedSeasonals(plans: DeptPlans, now: Record<string, number>): number {
  return Object.entries(plans).reduce(
    (sum, [code, d]) => sum + Math.abs(d.seasonal_target - (now[code] ?? 0)),
    0,
  );
}

function Stepper(props: {
  label: string;
  value: string;
  moreLabel: string;
  lessLabel: string;
  moreDisabled: boolean;
  lessDisabled: boolean;
  /** Id legende odjela: gumbi se razlikuju po opisu, vidljiva imena ostaju kratka (UI_PRAVILA §6.6). */
  describedBy: string;
  onMore: () => void;
  onLess: () => void;
}) {
  return (
    <div className="space-y-1">
      <p className="font-semibold">{props.label}</p>
      <div className="flex items-center gap-2">
        <button
          type="button"
          aria-label={props.lessLabel}
          aria-describedby={props.describedBy}
          disabled={props.lessDisabled}
          onClick={props.onLess}
          className={STEP_BUTTON}
        >
          <span aria-hidden="true">−</span>
        </button>
        <output className="min-w-0 flex-1 text-center text-base font-bold tabular-nums whitespace-nowrap">
          {props.value}
        </output>
        <button
          type="button"
          aria-label={props.moreLabel}
          aria-describedby={props.describedBy}
          disabled={props.moreDisabled}
          onClick={props.onMore}
          className={STEP_BUTTON}
        >
          <span aria-hidden="true">+</span>
        </button>
      </div>
    </div>
  );
}

export function DepartmentPlan(props: {
  hotelId: string;
  plan: DeptPlans;
  limits: NonNullable<PlanLimitsBody["departments"]>;
  /** Ukupna najveća promjena sezonaca u krugu (`seasonal_step_total`). */
  stepTotal: number;
  weekly: boolean;
  onChange: (next: DeptPlans) => void;
  /** Javlja jesu li odjeli učitani (dok nisu, ekran ne dopušta spremanje). */
  onReady: (ready: boolean) => void;
}) {
  const { plan, limits, stepTotal, onChange, onReady, weekly } = props;
  const list = useDepartments(props.hotelId);
  const ready = list.isSuccess;
  useEffect(() => {
    onReady(ready);
  }, [ready, onReady]);
  if (list.isPending)
    return (
      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{t("plan.dept.title")}</h2>
        <StatusText>{t("status.loading")}</StatusText>
      </Card>
    );
  if (list.isError)
    return (
      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{t("plan.dept.title")}</h2>
        <ErrorText>{problemMessage(list.error)}</ErrorText>
        <Button type="button" variant="secondary" onClick={() => void list.refetch()}>
          {t("error.retry")}
        </Button>
      </Card>
    );

  const rows = list.data.departments.filter((d) => !isService(d.class) && plan[d.code] && limits[d.code]);
  const now: Record<string, number> = Object.fromEntries(
    list.data.departments.map((d) => [d.code, d.seasonal]),
  );
  const left = Math.max(0, stepTotal - movedSeasonals(plan, now));
  const set = (code: string, patch: Partial<DeptPlans[string]>) => {
    const cur = plan[code];
    if (cur) onChange({ ...plan, [code]: { ...cur, ...patch } });
  };

  return (
    <Card className="space-y-3">
      <div className="space-y-1">
        <h2 className="text-lg font-bold">{t("plan.dept.title")}</h2>
        <p className="text-sm text-muted">{t("plan.dept.intro")}</p>
        <p role="status" className="text-sm font-semibold">
          {left > 0
            ? tw(weekly, "plan.dept.stepLeft", { n: formatCount(left, "seasonalWorker") })
            : tw(weekly, "plan.dept.stepNone")}
        </p>
      </div>
      <div className="space-y-3 md:grid md:grid-cols-2 md:items-start md:gap-3 md:space-y-0">
        {rows.map((d) => {
          const p = plan[d.code];
          const l: DeptLimits | undefined = limits[d.code];
          if (!p || !l) return null;
          const base = now[d.code] ?? 0;
          // Pomak od sadašnjeg stanja troši korak; vraćanje prema stanju ga oslobađa.
          const cost = (target: number) => Math.abs(target - base) - Math.abs(p.seasonal_target - base);
          const canMove = (target: number) =>
            p.open && target >= l.seasonal_min && target <= l.seasonal_max && cost(target) <= left;
          // Zatvaranje (cilj 0) smije samo ako stane u preostali korak.
          const closeFits = cost(0) <= left;
          const legendId = `dept-name-${d.code}`;
          const off = !p.open;
          const fixedPrice = l.price_level_min_pct === l.price_level_max_pct;
          return (
            <fieldset key={d.code} className="min-w-0 space-y-3 rounded-2xl border border-line p-3">
              <legend id={legendId} className="px-1 text-base font-bold">
                {d.name}
              </legend>
              {l.can_close && (
                <div>
                  <label
                    htmlFor={`dept-open-${d.code}`}
                    className="flex min-h-11 items-center gap-3 font-semibold"
                  >
                    <input
                      id={`dept-open-${d.code}`}
                      type="checkbox"
                      checked={p.open}
                      disabled={p.open && !closeFits}
                      aria-describedby={legendId}
                      onChange={(e) =>
                        set(
                          d.code,
                          e.target.checked
                            ? { open: true, seasonal_target: base }
                            : { open: false, seasonal_target: 0 },
                        )
                      }
                      className={`size-5 accent-accent ${FOCUS}`}
                    />
                    {t("plan.dept.open")}
                  </label>
                  {!p.open && <p className="text-sm text-muted">{t("plan.dept.closedNote")}</p>}
                  {p.open && !closeFits && (
                    <p className="text-sm text-muted">{tw(weekly, "plan.dept.noClose")}</p>
                  )}
                </div>
              )}
              <Stepper
                label={t("plan.dept.seasonal")}
                value={formatCount(p.seasonal_target, "seasonalWorker")}
                moreLabel={t("plan.dept.moreSeasonal")}
                lessLabel={t("plan.dept.lessSeasonal")}
                describedBy={legendId}
                moreDisabled={!canMove(p.seasonal_target + 1)}
                lessDisabled={!canMove(p.seasonal_target - 1)}
                onMore={() => set(d.code, { seasonal_target: p.seasonal_target + 1 })}
                onLess={() => set(d.code, { seasonal_target: p.seasonal_target - 1 })}
              />
              {!fixedPrice && (
                <div className="space-y-1">
                  <Stepper
                    label={t("plan.dept.price")}
                    value={formatPercentWhole(p.price_level_pct)}
                    moreLabel={t("plan.dept.morePrice")}
                    lessLabel={t("plan.dept.lessPrice")}
                    describedBy={legendId}
                    moreDisabled={!p.open || p.price_level_pct + PRICE_STEP > l.price_level_max_pct}
                    lessDisabled={!p.open || p.price_level_pct - PRICE_STEP < l.price_level_min_pct}
                    onMore={() => set(d.code, { price_level_pct: p.price_level_pct + PRICE_STEP })}
                    onLess={() => set(d.code, { price_level_pct: p.price_level_pct - PRICE_STEP })}
                  />
                  <p className="text-xs text-muted">
                    {t("plan.range", {
                      range: formatRange(l.price_level_min_pct, l.price_level_max_pct, "percent"),
                    })}
                  </p>
                </div>
              )}
              <Choice
                legend={t("plan.dept.maintenance")}
                name={`dept-maintenance-${d.code}`}
                value={p.maintenance}
                disabled={off}
                describedBy={legendId}
                options={[
                  { v: "deferred", label: t("maintenance.deferred") },
                  { v: "regular", label: t("maintenance.regular") },
                  { v: "intensive", label: t("maintenance.intensive") },
                ]}
                onChange={(v) => set(d.code, { maintenance: v })}
              />
            </fieldset>
          );
        })}
      </div>
    </Card>
  );
}
