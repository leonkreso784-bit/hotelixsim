// Uređivanje odluka odjela u otvorenom krugu: otvoren/zatvoren, sezonski radnici, razina cijena, održavanje i obnova
// opreme, svaka s cijenom i učinkom. Nespremljene izmjene čuva nacrt (`useDraft`, preživi odlazak u plan i natrag) i to
// su samo dirana polja: „Spremi odluke odjela“ šalje svježe dohvaćen plan kruga s izmjenama preko odjela
// (`useSavePlan`), pa polja koja student nije dirao ostaju kakva su na poslužitelju.
import { t } from "@hotelixsim/i18n";
import { useRef, useState } from "react";
import { ApiError, problemMessage } from "../../api/errors";
import { useDepartments, useSavePlan } from "../../api/queries";
import type { DepartmentBody, PlanStateBody } from "../../api/types";
import { differs, draftKey, useDraft } from "../../drafts";
import { formatCount, formatEuros, tw } from "../../format";
import { Button, Card, ErrorText, StatusText } from "../../ui/primitives";
import { RenewalLocked, RenewalPicker } from "../renewal/RenewalPicker";
import { planPayNow } from "../renewal/renewal";
import { anyDecision, decisionsIntro } from "./decisionsIntro";
import { isLease } from "./helpers";
import { PriceLevel, Upkeep } from "./Levers";
import { OpenToggle } from "./OpenToggle";
import { Stepper } from "./Stepper";
import { type DeptPlans, stepLeftForOthers } from "./step";

/** Ukupno sezonskih radnika u planu (poslužitelj ga drži unutar `seasonal_max` hotela). */
function seasonalTotal(plans: DeptPlans): number {
  return Object.values(plans).reduce((sum, d) => sum + d.seasonal_target, 0);
}

type DeptLimits = NonNullable<PlanStateBody["limits"]["departments"]>[string];
type Entry = DeptPlans[string];

export function DecisionsEditor(props: {
  hotelId: string;
  dept: DepartmentBody;
  round: string;
  plans: DeptPlans;
  entry: Entry;
  limits: DeptLimits;
  stepTotal: number;
  seasonalMax: number;
  weekly: boolean;
  /** Plan otvorenog kruga: rate obnove ostalih odjela i zgrade za blagajnu nakon plaćanja. */
  state: PlanStateBody;
  cashCents: number;
}) {
  const { dept, plans, entry, limits: l, weekly } = props;
  const code = dept.code;
  const list = useDepartments(props.hotelId);
  const save = useSavePlan(props.hotelId);
  const inFlight = useRef(false);
  const changes = useDraft<Entry>(draftKey("dept", props.hotelId, props.round, code), entry);
  const draft: Entry = { ...entry, ...changes.changes };
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
  // Razina cijena samo gdje poslužitelj nudi raspon: odjel bez prihoda po gostu (lokal s ugovorenom zakupninom) ima
  // 100–100, a parking cijenu ima kao restoran (O-P). Brojač sezonskih radnika je samo za odjel koji ih smije imati
  // (najam i odjel bez sezonskih imaju 0–0).
  const lease = isLease(dept.class);
  const fixedPrice = l.price_level_min_pct === l.price_level_max_pct;
  const fixedSeasonal = l.seasonal_min === l.seasonal_max;
  // Održavanje koje ništa ne stoji (lokal u najmu: brine zakupac) nije izbor.
  const m = l.maintenance;
  const freeUpkeep = m.deferred_cents === 0 && m.regular_cents === 0 && m.intensive_cents === 0;
  const offered = {
    close: l.can_close,
    seasonal: !fixedSeasonal,
    price: !fixedPrice,
    upkeep: !freeUpkeep,
    renewal: !!l.renewal && !l.renewal.locked,
    lease,
  };
  const renewalAmount = draft.renewal_cents ?? 0;
  const legendId = `dept-name-${code}`;

  const set = (patch: Partial<Entry>) => {
    changes.update(patch);
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
    const mine = changes.changes;
    const sent = { ...mine };
    save.mutate(
      {
        round: props.round,
        patch: (cur) => {
          const fresh = cur.departments?.[code] ?? entry;
          const departments = { ...(cur.departments ?? {}), [code]: { ...fresh, ...mine } };
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
        onSuccess: () => changes.settle(sent),
        onSettled: () => {
          inFlight.current = false;
        },
      },
    );
  }

  if (!anyDecision(offered))
    return (
      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{tw(weekly, "dept.decide")}</h2>
        <p className="text-muted">{t("dept.decide.none")}</p>
        {l.renewal?.locked && <RenewalLocked state={l.renewal.locked} weekly={weekly} />}
      </Card>
    );
  return (
    <Card className="space-y-3">
      <div className="space-y-1">
        <h2 className="text-lg font-bold">{tw(weekly, "dept.decide")}</h2>
        <p className="text-sm text-muted">{decisionsIntro(offered)}</p>
        {!fixedSeasonal && (
          <p role="status" className="text-sm font-semibold">
            {left > 0
              ? tw(weekly, "plan.dept.stepLeft", { n: formatCount(left, "seasonalWorkerAcc") })
              : tw(weekly, "plan.dept.stepNone")}
          </p>
        )}
      </div>
      {/* Od 768 px sve odluke u istom uskom stupcu, poravnate lijevo (M-3). */}
      <fieldset className="min-w-0 space-y-3 md:max-w-md">
        <legend id={legendId} className="sr-only">
          {dept.name}
        </legend>
        {l.can_close && (
          <OpenToggle
            code={code}
            open={draft.open}
            closeFits={cost(0) <= left}
            describedBy={legendId}
            weekly={weekly}
            onChange={(open) => set(open ? { open, seasonal_target: base } : { open, seasonal_target: 0 })}
          />
        )}
        {!fixedSeasonal && (
          <div className="space-y-1">
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
            <p className="text-sm text-muted">
              {tw(weekly, "plan.dept.seasonalCost", { amount: formatEuros(l.seasonal_worker_cents) })}
            </p>
          </div>
        )}
        {!fixedPrice && (
          <PriceLevel
            pct={draft.price_level_pct}
            min={l.price_level_min_pct}
            max={l.price_level_max_pct}
            open={draft.open}
            lease={lease}
            describedBy={legendId}
            onChange={(price_level_pct) => set({ price_level_pct })}
          />
        )}
        {freeUpkeep ? (
          <p className="text-sm text-muted">{t("plan.dept.noUpkeep")}</p>
        ) : (
          <Upkeep
            code={code}
            value={draft.maintenance}
            levels={l.maintenance}
            open={draft.open}
            weekly={weekly}
            describedBy={legendId}
            onChange={(maintenance) => set({ maintenance })}
          />
        )}
        {l.renewal && (
          <RenewalPicker
            limits={l.renewal}
            amount={renewalAmount}
            subject="dept"
            cashCents={props.cashCents}
            payNowTotal={planPayNow(props.state, { code, amount: renewalAmount })}
            payrollWeekCents={props.state.limits.costs.payroll_week_cents}
            weekly={weekly}
            describedBy={legendId}
            onChange={(v) => set({ renewal_cents: v })}
          />
        )}
      </fieldset>
      {list.isError && (
        <div className="space-y-2">
          <ErrorText>{problemMessage(list.error)}</ErrorText>
          {/* Od 1024 px isti ponovni pokušaj nudi stupac popisa; ispod toga popis nije na ekranu. */}
          <Button variant="secondary" className="lg:hidden" onClick={() => void list.refetch()}>
            {t("dept.decide.retry")}
          </Button>
        </div>
      )}
      {tooMany && (
        <ErrorText>
          {t("plan.dept.tooMany", { n: formatCount(props.seasonalMax, "seasonalWorker") })}
        </ErrorText>
      )}
      {/* Novi krug javlja rečenica iznad odluka (DepartmentDecisions), ne greška. */}
      {save.isError && !(save.error instanceof ApiError && save.error.code === "round_changed") && (
        <ErrorText>{problemMessage(save.error, weekly)}</ErrorText>
      )}
      {save.isSuccess && <StatusText>{t("dept.decide.saved")}</StatusText>}
      {differs(entry, changes.changes) && <p className="text-sm font-semibold">{t("plan.unsaved")}</p>}
      {/* aria-disabled, ne disabled: fokus tipkovnice ostaje na gumbu dok traje spremanje (M-6). */}
      <Button aria-disabled={save.isPending || !list.isSuccess} className="md:max-w-md" onClick={onSave}>
        {t("dept.decide.save")}
      </Button>
    </Card>
  );
}
