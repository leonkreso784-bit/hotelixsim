// Odluke odjela za otvoreni krug (samo v2). Granice i ukupni korak sezonskih radnika dolaze od poslužitelja. Kad krug
// nije otvoren, odluke su samo za čitanje. Ako se dok je ekran otvoren otvori novi krug, rečenica to kaže.
import type { DepartmentBody, PlanStateBody } from "../../api/types";
import { useRoundChanged } from "../../drafts";
import { tw } from "../../format";
import { StatusText } from "../../ui/primitives";
import { DecisionsEditor } from "./DecisionsEditor";
import { DecisionsReadOnly } from "./DecisionsReadOnly";

export function DepartmentDecisions({
  hotelId,
  dept,
  planState,
  weekly,
}: {
  hotelId: string;
  dept: DepartmentBody;
  /** Plan otvorenog kruga (moj hotel); `null` kad krug nije otvoren. */
  planState: PlanStateBody | null;
  weekly: boolean;
}) {
  const newRound = useRoundChanged(planState?.round ?? null);
  const plans = planState?.plan.departments ?? null;
  const entry = plans?.[dept.code];
  const limits = planState?.limits.departments?.[dept.code];
  if (!planState) return <DecisionsReadOnly dept={dept} weekly={weekly} />;
  // Odjel koji se u ovom krugu ne planira (nema ga u planu ni u granicama): nema odluka.
  if (!plans || !entry || !limits) return null;
  return (
    <>
      {newRound && <StatusText>{tw(weekly, "plan.newRound")}</StatusText>}
      <DecisionsEditor
        key={planState.round}
        hotelId={hotelId}
        dept={dept}
        round={planState.round}
        plans={plans}
        entry={entry}
        limits={limits}
        stepTotal={planState.limits.seasonal_step_total ?? 0}
        seasonalMax={planState.limits.seasonal_max}
        weekly={weekly}
      />
    </>
  );
}
