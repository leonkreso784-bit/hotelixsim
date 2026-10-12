// Plan kruga: odluke za cijeli hotel (cijena sobe, marketing, plaće, obuka, obnova, recenzije), svaka s cijenom ili
// učinkom. U v1 igri i sezonski radnici i održavanje hotela; u v2 su oni po odjelu, na ekranu odjela, a ovdje stoji samo
// sažetak promjena po odjelima. Nespremljene izmjene čuva nacrt (`useDraft`): odlazak u odjel i povratak ih ne briše.
import { t } from "@hotelixsim/i18n";
import { type FormEvent, useRef, useState } from "react";
import { ApiError, problemMessage } from "../api/errors";
import { useGame, useSavePlan } from "../api/queries";
import type { HotelBody } from "../api/types";
import { differs, draftKey, useDraft, useRoundChanged } from "../drafts";
import { formatEuros, roundPhrase, roundWithoutGame, tw } from "../format";
import { isWeekly, roundInfo } from "../game";
import { GameError } from "../ui/GameError";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { Choice } from "./plan/Choice";
import { maintenanceOptions, trainingEffect, trainingOptions } from "./plan/costs";
import { DepartmentSummary } from "./plan/DepartmentSummary";
import { NumberField, Toggle } from "./plan/fields";
import { type Draft, hotelPatch, toDraft } from "./plan/hotelDraft";
import { Renovation } from "./plan/Renovation";

const roundChanged = (e: unknown) => e instanceof ApiError && e.code === "round_changed";

export function PlanScreen({ me }: { me: HotelBody }) {
  const state = me.private?.plan ?? null;
  const save = useSavePlan(me.hotel.hotel_id);
  const gameQuery = useGame(me.game_id);
  const game = gameQuery.data;
  const weekly = game ? isWeekly(game) : false;
  const inFlight = useRef(false);
  const changes = useDraft<Draft>(draftKey("plan", me.hotel.hotel_id, state?.round ?? "-"));
  const newRound = useRoundChanged(state?.round ?? null);
  const [invalid, setInvalid] = useState(false);
  // Tekstovi ovise o tome je li igra tjedna, pa se čeka igra (kao Pokazatelji i Rezultat).
  if (gameQuery.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (!game) return <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />;
  if (!state)
    return (
      <>
        <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
        <p className="text-muted">{tw(weekly, "plan.closed")}</p>
      </>
    );
  const saved = toDraft(state.plan);
  const draft: Draft = { ...saved, ...changes.changes };
  const l = state.limits;
  const depts = state.plan.departments ?? null;
  const v2 = depts !== null && !!l.departments;
  const set = (patch: Partial<Draft>) => {
    changes.update(patch);
    setInvalid(false);
    save.reset();
  };

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!state || save.isPending || inFlight.current) return;
    const hotel = hotelPatch(draft, l, v2);
    if (!hotel) {
      setInvalid(true);
      return;
    }
    inFlight.current = true;
    save.mutate(
      { round: state.round, patch: (cur) => ({ ...cur, ...hotel }) },
      {
        onSuccess: () => changes.clear(),
        onSettled: () => {
          inFlight.current = false;
        },
      },
    );
  }

  const info = game ? roundInfo(game, state.round) : null;
  const round = info ? roundPhrase(info) : roundWithoutGame(state.round, state.month);
  return (
    <form onSubmit={onSubmit} className="space-y-3">
      <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
      <p className="text-sm text-muted">
        {weekly ? t("plan.for.week", { round }) : t("plan.for", { month: round })}
      </p>
      {newRound && <StatusText>{tw(weekly, "plan.newRound")}</StatusText>}
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
      {v2 && depts && <DepartmentSummary hotelId={me.hotel.hotel_id} plans={depts} weekly={weekly} />}
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
          <Renovation
            checked={draft.renovation}
            limits={l}
            cashCents={me.private?.now.cash_cents ?? 0}
            onChange={(v) => set({ renovation: v })}
          />
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
      {invalid && <ErrorText>{t("plan.invalid")}</ErrorText>}
      {save.isError && !(newRound && roundChanged(save.error)) && (
        <ErrorText>{problemMessage(save.error, weekly)}</ErrorText>
      )}
      {save.isSuccess && <StatusText>{t("plan.saved", { month: round })}</StatusText>}
      {differs(saved, changes.changes) && <p className="text-sm font-semibold">{t("plan.unsaved")}</p>}
      <Button type="submit" disabled={save.isPending}>
        {t("plan.save")}
      </Button>
    </form>
  );
}
