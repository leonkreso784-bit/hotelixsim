// Uređivanje odluka odjela u otvorenom krugu: otvoren/zatvoren, sezonski radnici, razina cijena i održavanje.
// „Spremi odluke odjela“ šalje cijeli plan kruga: svježe dohvaćen važeći plan + izmjene ovog odjela (`useSavePlan`).
import { t } from "@hotelixsim/i18n";
import { useRef, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useDepartments, useSavePlan } from "../../api/queries";
import type { DepartmentBody, PlanStateBody } from "../../api/types";
import { formatCount, formatPercentWhole, formatRange, tw } from "../../format";
import { Button, Card, ErrorText, StatusText } from "../../ui/primitives";
import { Choice } from "../plan/Choice";
import { maintenanceOptions } from "../plan/costs";
import { FOCUS } from "./helpers";
import { Stepper } from "./Stepper";
import { type DeptPlans, stepLeftForOthers } from "./step";

const PRICE_STEP = 5;

/** Ukupno sezonskih radnika u planu (poslužitelj ga drži unutar `seasonal_max` hotela). */
function seasonalTotal(plans: DeptPlans): number {
  return Object.values(plans).reduce((sum, d) => sum + d.seasonal_target, 0);
}

type DeptLimits = NonNullable<PlanStateBody["limits"]["departments"]>[string];

export function DecisionsEditor(props: {
  hotelId: string;
  dept: DepartmentBody;
  round: string;
  plans: DeptPlans;
  entry: DeptPlans[string];
  limits: DeptLimits;
  stepTotal: number;
  seasonalMax: number;
  weekly: boolean;
}) {
  const { dept, plans, limits: l, weekly } = props;
  const code = dept.code;
  const list = useDepartments(props.hotelId);
  const save = useSavePlan(props.hotelId);
  const inFlight = useRef(false);
  const [draft, setDraft] = useState(props.entry);
  const [tooMany, setTooMany] = useState(false);

  // Sadašnji broj sezonskih radnika po odjelu: korak troši pomak od njega (Σ |cilj − sada|, kao poslužitelj).
  const now: Record<string, number> = Object.fromEntries(
    (list.data?.departments ?? [dept]).map((d) => [d.code, d.seasonal]),
  );
  const base = dept.seasonal;
  const forMe = stepLeftForOthers(plans, now, code, props.stepTotal);
  const left = Math.max(0, forMe - Math.abs(draft.seasonal_target - base));
  const cost = (target: number) => Math.abs(target - base) - Math.abs(draft.seasonal_target - base);
  const canMove = (target: number) =>
    draft.open && target >= l.seasonal_min && target <= l.seasonal_max && cost(target) <= left;
  const closeFits = cost(0) <= left;
  const fixedPrice = l.price_level_min_pct === l.price_level_max_pct;
  const legendId = `dept-name-${code}`;

  const set = (patch: Partial<DeptPlans[string]>) => {
    setDraft({ ...draft, ...patch });
    setTooMany(false);
    save.reset();
  };

  function onSave() {
    if (save.isPending || inFlight.current || !list.isSuccess) return;
    if (seasonalTotal({ ...plans, [code]: draft }) > props.seasonalMax) {
      setTooMany(true);
      return;
    }
    inFlight.current = true;
    save.mutate(
      {
        round: props.round,
        patch: (cur) => {
          const departments = { ...(cur.departments ?? {}), [code]: draft };
          // Ukupni sezonski radnici i održavanje Smještaja izvode se iz odjela, kao na poslužitelju.
          return {
            ...cur,
            departments,
            seasonal_target: seasonalTotal(departments),
            maintenance: departments.rooms?.maintenance ?? cur.maintenance,
          };
        },
      },
      {
        onSuccess: (saved) => {
          const mine = saved.departments?.[code];
          if (mine) setDraft(mine);
        },
        onSettled: () => {
          inFlight.current = false;
        },
      },
    );
  }

  return (
    <Card className="space-y-3">
      <div className="space-y-1">
        <h2 className="text-lg font-bold">{tw(weekly, "dept.decide")}</h2>
        <p className="text-sm text-muted">{t("dept.decide.intro")}</p>
        <p role="status" className="text-sm font-semibold">
          {left > 0
            ? tw(weekly, "plan.dept.stepLeft", { n: formatCount(left, "seasonalWorkerAcc") })
            : tw(weekly, "plan.dept.stepNone")}
        </p>
      </div>
      <fieldset className="min-w-0 space-y-3">
        <legend id={legendId} className="sr-only">
          {dept.name}
        </legend>
        {l.can_close && (
          <div>
            <label htmlFor={`dept-open-${code}`} className="flex min-h-11 items-center gap-3 font-semibold">
              <input
                id={`dept-open-${code}`}
                type="checkbox"
                checked={draft.open}
                disabled={draft.open && !closeFits}
                aria-describedby={legendId}
                onChange={(e) =>
                  set(
                    e.target.checked
                      ? { open: true, seasonal_target: base }
                      : { open: false, seasonal_target: 0 },
                  )
                }
                className={`size-5 accent-accent ${FOCUS}`}
              />
              {t("plan.dept.open")}
            </label>
            {!draft.open && <p className="text-sm text-muted">{t("plan.dept.closedNote")}</p>}
            {draft.open && !closeFits && (
              <p className="text-sm text-muted">{tw(weekly, "plan.dept.noClose")}</p>
            )}
          </div>
        )}
        <Stepper
          named
          label={t("plan.dept.seasonal")}
          value={formatCount(draft.seasonal_target, "seasonalWorker")}
          moreLabel={t("plan.dept.moreSeasonal")}
          lessLabel={t("plan.dept.lessSeasonal")}
          describedBy={legendId}
          moreDisabled={!canMove(draft.seasonal_target + 1)}
          lessDisabled={!canMove(draft.seasonal_target - 1)}
          onMore={() => set({ seasonal_target: draft.seasonal_target + 1 })}
          onLess={() => set({ seasonal_target: draft.seasonal_target - 1 })}
        />
        {!fixedPrice && (
          <div className="space-y-1">
            <Stepper
              label={t("plan.dept.price")}
              value={formatPercentWhole(draft.price_level_pct)}
              moreLabel={t("plan.dept.morePrice")}
              lessLabel={t("plan.dept.lessPrice")}
              describedBy={legendId}
              moreDisabled={!draft.open || draft.price_level_pct + PRICE_STEP > l.price_level_max_pct}
              lessDisabled={!draft.open || draft.price_level_pct - PRICE_STEP < l.price_level_min_pct}
              onMore={() => set({ price_level_pct: draft.price_level_pct + PRICE_STEP })}
              onLess={() => set({ price_level_pct: draft.price_level_pct - PRICE_STEP })}
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
          name={`dept-maintenance-${code}`}
          value={draft.maintenance}
          disabled={!draft.open}
          describedBy={legendId}
          options={maintenanceOptions(l.maintenance)}
          onChange={(v) => set({ maintenance: v })}
        />
        <p className="text-xs text-muted">{tw(weekly, "plan.cost.levels")}</p>
      </fieldset>
      {/* Ponovni pokušaj nudi stupac popisa (isti upit), da gumb ne bude dvaput na ekranu. */}
      {list.isError && <ErrorText>{problemMessage(list.error)}</ErrorText>}
      {tooMany && (
        <ErrorText>
          {t("plan.dept.tooMany", { n: formatCount(props.seasonalMax, "seasonalWorker") })}
        </ErrorText>
      )}
      {save.isError && <ErrorText>{problemMessage(save.error, weekly)}</ErrorText>}
      {save.isSuccess && <StatusText>{t("dept.decide.saved")}</StatusText>}
      <Button disabled={save.isPending || !list.isSuccess} onClick={onSave}>
        {t("dept.decide.save")}
      </Button>
    </Card>
  );
}
