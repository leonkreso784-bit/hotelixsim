// Odluke odjela za otvoreni krug (samo v2). Granice i ukupni korak sezonskih radnika dolaze od poslužitelja. Kad krug
// nije otvoren, odluke su samo za čitanje. Ako se dok je ekran otvoren otvori novi krug, rečenica to kaže.
import type { DepartmentBody, PlanStateBody } from "../../api/types";
import { useRoundChanged, useStaleDrafts } from "../../drafts";
import { RoundNotice } from "../plan/RoundNotice";
import { DecisionsEditor } from "./DecisionsEditor";
import { DecisionsReadOnly } from "./DecisionsReadOnly";

export function DepartmentDecisions({
  hotelId,
  dept,
  planState,
  weekly,
  cashCents,
}: {
  hotelId: string;
  dept: DepartmentBody;
  /** Plan otvorenog kruga (moj hotel); `null` kad krug nije otvoren. */
  planState: PlanStateBody | null;
  weekly: boolean;
  /** Blagajna sada (moj hotel), za blagajnu nakon plaćanja obnove. */
  cashCents: number;
}) {
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
      <RoundNotice show={newRound || stale} weekly={weekly} />
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
        seasonalMax={planState.limits.seasonal_max}
        weekly={weekly}
        state={planState}
        cashCents={cashCents}
      />
    </>
  );
}
