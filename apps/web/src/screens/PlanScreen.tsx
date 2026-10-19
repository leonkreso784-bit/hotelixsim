// Plan kruga: odluke za cijeli hotel (cijena sobe, marketing, plaće, obuka, obnova, recenzije), svaka s cijenom ili
// učinkom. U v1 igri i sezonski radnici i održavanje hotela; u v2 su oni po odjelu, na ekranu odjela, a ovdje stoji samo
// sažetak promjena po odjelima i (v2 s obnovom) zgrada s obnovom iznosom. Nespremljene izmjene čuva nacrt (`useDraft`): odlazak u odjel i povratak ih ne briše.
import { t } from "@hotelixsim/i18n";
import { useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useEffect } from "react";
import { keys, useGame } from "../api/queries";
import type { HotelBody } from "../api/types";
import { draftKey, useDraft, useRoundChanged, useStaleDrafts } from "../drafts";
import { roundPhrase, roundWithoutGame, tw } from "../format";
import { isWeekly, roundInfo } from "../game";
import { GameError } from "../ui/GameError";
import { Card, StatusText } from "../ui/primitives";
import { BuildingRenewal } from "./plan/BuildingRenewal";
import { Choice } from "./plan/Choice";
import { maintenanceOptions } from "./plan/costs";
import { DepartmentSummary } from "./plan/DepartmentSummary";
import { NumberField } from "./plan/fields";
import { type Draft, toDraft } from "./plan/hotelDraft";
import { MoreDecisions } from "./plan/MoreDecisions";
import { RoundNotice } from "./plan/RoundNotice";
import { SavePlanBar } from "./plan/SavePlanBar";
import { roundChanged, usePlanSave } from "./plan/usePlanSave";
import { BUILDING, planPayNow } from "./renewal/renewal";

export function PlanScreen({ me }: { me: HotelBody }) {
  const state = me.private?.plan ?? null;
  const saver = usePlanSave(me.hotel.hotel_id, state);
  const gameQuery = useGame(me.game_id);
  const game = gameQuery.data;
  const weekly = game ? isWeekly(game) : false;
  const saved = state ? toDraft(state.plan) : null;
  const changes = useDraft<Draft>(
    draftKey("plan", me.hotel.hotel_id, state?.round ?? "-"),
    saved ?? ({} as Draft),
  );
  const newRound = useRoundChanged(state?.round ?? null);
  const stale = useStaleDrafts("plan", me.hotel.hotel_id, state?.round ?? null);
  // Plana nema, a spremljena igra još kaže da je krug otvoren: krug je zatvoren izvana (druga kartica, profesor).
  // Ponovni dohvat igre vidi „closing“, ispituje do novog kruga i tada osvježi moj hotel (UI krug 3, M-2).
  const qc = useQueryClient();
  const staleOpen = !state && game?.current?.status === "open";
  useEffect(() => {
    if (staleOpen) void qc.invalidateQueries({ queryKey: keys.game(me.game_id) });
  }, [staleOpen, qc, me.game_id]);
  // Tekstovi ovise o tome je li igra tjedna, pa se čeka igra (kao Pokazatelji i Rezultat).
  if (gameQuery.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (!game) return <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />;
  if (!state || !saved)
    return (
      <>
        <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
        <p className="text-muted">{tw(weekly, "plan.closed")}</p>
      </>
    );
  const draft: Draft = { ...saved, ...changes.changes };
  const l = state.limits;
  const depts = state.plan.departments ?? null;
  const v2 = depts !== null && !!l.departments;
  const buildingRenewal = v2 ? (l.costs.building_renewal ?? null) : null;
  const cash = me.private?.now.cash_cents ?? 0;
  const set = (patch: Partial<Draft>) => {
    changes.update(patch);
  };

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    saver.run();
  }

  const info = game ? roundInfo(game, state.round) : null;
  const round = info ? roundPhrase(info) : roundWithoutGame(state.round, state.month);
  return (
    <form onSubmit={onSubmit} className="space-y-3">
      <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
      <p className="text-sm text-muted">
        {weekly ? t("plan.for.week", { round }) : t("plan.for", { month: round })}
      </p>
      <RoundNotice show={newRound || stale || roundChanged(saver.save.error)} weekly={weekly} />
      <Card className="space-y-4">
        <NumberField
          id="plan-price"
          label={t("plan.price")}
          value={draft.price}
          min={l.price_min_cents / 100}
          max={l.price_max_cents / 100}
          unit="euro"
          note={t("plan.price.note")}
          onChange={(v) => set({ price: v })}
        />
        {!v2 && (
          <>
            <NumberField
              id="plan-seasonal"
              label={t("plan.seasonal")}
              value={draft.seasonal}
              min={l.seasonal_min}
              max={l.seasonal_max}
              unit="person"
              onChange={(v) => set({ seasonal: v })}
            />
            <div className="space-y-1">
              <Choice
                legend={t("plan.maintenance")}
                name="maintenance"
                value={draft.maintenance}
                options={maintenanceOptions(l.costs.maintenance)}
                onChange={(v) => set({ maintenance: v })}
              />
              <p className="text-sm text-muted">
                {tw(weekly, "plan.cost.levels")} {t("plan.maintenance.note")}
              </p>
            </div>
          </>
        )}
      </Card>
      {v2 && depts && (
        <DepartmentSummary hotelId={me.hotel.hotel_id} round={state.round} plans={depts} weekly={weekly} />
      )}
      {buildingRenewal && (
        <BuildingRenewal
          limits={buildingRenewal}
          building={me.private?.building ?? null}
          guide={l.costs.renewal_guide ?? null}
          amount={draft.building}
          cashCents={cash}
          payNowTotal={planPayNow(state, { code: BUILDING, amount: draft.building })}
          payrollWeekCents={l.costs.payroll_week_cents}
          weekly={weekly}
          onChange={(v) => set({ building: v })}
        />
      )}
      <MoreDecisions
        draft={draft}
        limits={l}
        cashCents={cash}
        weekly={weekly}
        // v2 s obnovom: zgrada se obnavlja iznosom (gore), a stara kvačica obnove se ne nudi.
        renovation={!(buildingRenewal && !l.renovation_available)}
        set={set}
      />
      <SavePlanBar saver={saver} round={round} seasonalMax={l.seasonal_max} weekly={weekly} submit />
    </form>
  );
}
