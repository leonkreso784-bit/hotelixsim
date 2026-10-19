// Odluke odjela za otvoreni krug (samo v2). Granice i ukupni korak sezonskih radnika dolaze od poslužitelja. Kad krug
// nije otvoren, odluke su samo za čitanje. Ako se dok je ekran otvoren otvori novi krug, rečenica to kaže. Odluke sprema
// jedini gumb „Spremi plan“ (traka ispod), zajedno s ostalim odjelima i planom hotela.
import type { DepartmentBody, PlanStateBody } from "../../api/types";
import { useRoundChanged, useStaleDrafts } from "../../drafts";
import { RoundNotice } from "../plan/RoundNotice";
import { SavePlanBar } from "../plan/SavePlanBar";
import { roundChanged, usePlanSave } from "../plan/usePlanSave";
import { DecisionsEditor } from "./DecisionsEditor";
import { DecisionsReadOnly } from "./DecisionsReadOnly";

export function DepartmentDecisions({
  hotelId,
  dept,
  planState,
  weekly,
  cashCents,
  roundLabel,
}: {
  hotelId: string;
  dept: DepartmentBody;
  /** Plan otvorenog kruga (moj hotel); `null` kad krug nije otvoren. */
  planState: PlanStateBody | null;
  weekly: boolean;
  /** Blagajna sada (moj hotel), za blagajnu nakon plaćanja obnove. */
  cashCents: number;
  /** Naziv otvorenog kruga za potvrdu spremanja. */
  roundLabel: string;
}) {
  const saver = usePlanSave(hotelId, planState);
  const newRound = useRoundChanged(planState?.round ?? null);
  const stale = useStaleDrafts("dept", hotelId, planState?.round ?? null);
  const plans = planState?.plan.departments ?? null;
  const entry = plans?.[dept.code];
  const limits = planState?.limits.departments?.[dept.code];
  if (!planState) return <DecisionsReadOnly dept={dept} weekly={weekly} />;
  // Odjel koji se u ovom krugu ne planira (nema ga u planu ni u granicama): nema odluka.
  if (!plans || !entry || !limits) return null;
  return (
    <>
      <RoundNotice show={newRound || stale || roundChanged(saver.save.error)} weekly={weekly} />
      <DecisionsEditor
        // Po krugu i odjelu: stanje spremanja i greške ne prelaze s odjela na odjel (minor pregleda 7).
        key={`${planState.round}:${dept.code}`}
        hotelId={hotelId}
        dept={dept}
        round={planState.round}
        plans={plans}
        entry={entry}
        limits={limits}
        stepTotal={planState.limits.seasonal_step_total ?? 0}
        weekly={weekly}
        state={planState}
        cashCents={cashCents}
      />
      <SavePlanBar
        saver={saver}
        round={roundLabel}
        only={dept.code}
        seasonalMax={planState.limits.seasonal_max}
        weekly={weekly}
      />
    </>
  );
}
