import { t } from "@hotelixsim/i18n";
import { type FormEvent, type ReactNode, useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useGame, useSavePlan } from "../api/queries";
import type { HotelBody, PlanBody, PlanLimitsBody } from "../api/types";
import { formatRange, monthName, roundPhrase, tw } from "../format";
import { isWeekly, roundInfo } from "../game";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { Choice } from "./plan/Choice";
import { DepartmentPlan, type DeptPlans } from "./plan/DepartmentPlan";

type Draft = {
  price: string;
  marketing: string;
  seasonal: string;
  wage: string;
  training: PlanBody["training"];
  maintenance: PlanBody["maintenance"];
  renovation: boolean;
  reply: boolean;
};

const toDraft = (p: PlanBody): Draft => ({
  price: String(p.price_cents / 100),
  marketing: String(Math.round(p.marketing_cents / 100)),
  seasonal: String(p.seasonal_target),
  wage: String(p.wage_pct),
  training: p.training,
  maintenance: p.maintenance,
  renovation: p.renovation,
  reply: p.reply_reviews,
});

/** Cijeli broj u granicama ili null (prazno, decimalno, izvan raspona). */
function int(s: string, min: number, max: number): number | null {
  if (!/^\d+$/.test(s.trim())) return null;
  const n = Number(s.trim());
  return n >= min && n <= max ? n : null;
}

function toPlan(d: Draft, l: PlanLimitsBody, depts: DeptPlans | null): PlanBody | null {
  const price = int(d.price, l.price_min_cents / 100, l.price_max_cents / 100);
  const marketing = int(d.marketing, 0, l.marketing_max_cents / 100);
  // v2: sezonci i održavanje su po odjelu; ukupni sezonci i održavanje Smještaja izvode se iz njih (kao na poslužitelju).
  const seasonal = depts
    ? Object.values(depts).reduce((sum, x) => sum + x.seasonal_target, 0)
    : int(d.seasonal, l.seasonal_min, l.seasonal_max);
  const wage = int(d.wage, l.wage_min_pct, l.wage_max_pct);
  if (price === null || marketing === null || seasonal === null || wage === null) return null;
  return {
    price_cents: price * 100,
    marketing_cents: marketing * 100,
    seasonal_target: seasonal,
    wage_pct: wage,
    training: d.training,
    maintenance: depts?.rooms?.maintenance ?? d.maintenance,
    renovation: d.renovation,
    reply_reviews: d.reply,
    ...(depts ? { departments: depts } : {}),
  };
}

function NumberField(props: {
  id: string;
  label: string;
  value: string;
  min: number;
  max: number;
  /** Jedinica raspona ispod polja (UI_PRAVILA §2.1: broj ima ime i jedinicu, i u rasponu). */
  unit: "euro" | "percent" | "person";
  onChange: (v: string) => void;
}) {
  return (
    <div className="space-y-1">
      <label htmlFor={props.id} className="block font-semibold">
        {props.label}
      </label>
      <input
        id={props.id}
        type="number"
        inputMode="numeric"
        step={1}
        min={props.min}
        max={props.max}
        value={props.value}
        onChange={(e) => props.onChange(e.target.value)}
        className="min-h-12 w-full rounded-2xl border border-line bg-surface px-3 text-lg tabular-nums"
      />
      <p className="text-xs text-muted">
        {t("plan.range", { range: formatRange(props.min, props.max, props.unit) })}
      </p>
    </div>
  );
}

function Toggle(props: {
  id: string;
  label: string;
  checked: boolean;
  disabled?: boolean | undefined;
  note?: ReactNode | undefined;
  onChange: (v: boolean) => void;
}) {
  return (
    <div>
      <label htmlFor={props.id} className="flex min-h-11 items-center gap-3 font-semibold">
        <input
          id={props.id}
          type="checkbox"
          checked={props.checked}
          disabled={props.disabled}
          onChange={(e) => props.onChange(e.target.checked)}
          className="size-5 accent-accent"
        />
        {props.label}
      </label>
      {props.note}
    </div>
  );
}

export function PlanScreen({ me }: { me: HotelBody }) {
  const state = me.private?.plan ?? null;
  const save = useSavePlan(me.hotel.hotel_id);
  const game = useGame(me.game_id).data;
  const weekly = game ? isWeekly(game) : false;
  const inFlight = useRef(false);
  const [draft, setDraft] = useState<Draft | null>(state ? toDraft(state.plan) : null);
  const [depts, setDepts] = useState<DeptPlans | null>(state?.plan.departments ?? null);
  const [invalid, setInvalid] = useState(false);
  if (!state || !draft)
    return (
      <>
        <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
        <p className="text-muted">{tw(weekly, "plan.closed")}</p>
      </>
    );
  const l = state.limits;
  const v2 = depts !== null && !!l.departments;
  const set = (patch: Partial<Draft>) => {
    setDraft({ ...draft, ...patch });
    setInvalid(false);
    save.reset();
  };

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!state || !draft || save.isPending || inFlight.current) return;
    const plan = toPlan(draft, l, v2 ? depts : null);
    if (!plan) {
      setInvalid(true);
      return;
    }
    inFlight.current = true;
    save.mutate(
      { round: state.round, plan },
      {
        onSettled: () => {
          inFlight.current = false;
        },
      },
    );
  }

  const info = game ? roundInfo(game, state.round) : null;
  const round = info ? roundPhrase(info) : monthName(state.month);
  return (
    <form onSubmit={onSubmit} className="space-y-3">
      <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "plan.title")}</h1>
      <p className="text-sm text-muted">
        {weekly ? t("plan.for.week", { round }) : t("plan.for", { month: round })}
      </p>
      <Card className="space-y-4">
        <NumberField
          id="plan-price"
          label={t("plan.price")}
          value={draft.price}
          min={l.price_min_cents / 100}
          max={l.price_max_cents / 100}
          unit="euro"
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
            <Choice
              legend={t("plan.maintenance")}
              name="maintenance"
              value={draft.maintenance}
              options={[
                { v: "deferred", label: t("maintenance.deferred") },
                { v: "regular", label: t("maintenance.regular") },
                { v: "intensive", label: t("maintenance.intensive") },
              ]}
              onChange={(v) => set({ maintenance: v })}
            />
          </>
        )}
      </Card>
      {v2 && depts && l.departments && (
        <DepartmentPlan
          hotelId={me.hotel.hotel_id}
          plan={depts}
          limits={l.departments}
          stepTotal={l.seasonal_step_total ?? 0}
          onChange={(next) => {
            setDepts(next);
            setInvalid(false);
            save.reset();
          }}
        />
      )}
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
            onChange={(v) => set({ marketing: v })}
          />
          <NumberField
            id="plan-wage"
            label={t("plan.wage")}
            value={draft.wage}
            min={l.wage_min_pct}
            max={l.wage_max_pct}
            unit="percent"
            onChange={(v) => set({ wage: v })}
          />
          <Choice
            legend={t("plan.training")}
            name="training"
            value={draft.training}
            options={[
              { v: "none", label: t("training.none") },
              { v: "basic", label: t("training.basic") },
              { v: "full", label: t("training.full") },
            ]}
            onChange={(v) => set({ training: v })}
          />
          <Toggle
            id="plan-renovation"
            label={t("plan.renovation")}
            checked={draft.renovation}
            disabled={!l.renovation_available}
            note={!l.renovation_available && <p className="text-xs text-muted">{t("plan.renovationUsed")}</p>}
            onChange={(v) => set({ renovation: v })}
          />
          <Toggle
            id="plan-reply"
            label={t("plan.replyReviews")}
            checked={draft.reply}
            onChange={(v) => set({ reply: v })}
          />
        </div>
      </details>
      {invalid && <ErrorText>{t("plan.invalid")}</ErrorText>}
      {save.isError && <ErrorText>{problemMessage(save.error, weekly)}</ErrorText>}
      {save.isSuccess && <StatusText>{t("plan.saved", { month: round })}</StatusText>}
      <Button type="submit" disabled={save.isPending}>
        {t("plan.save")}
      </Button>
    </form>
  );
}
