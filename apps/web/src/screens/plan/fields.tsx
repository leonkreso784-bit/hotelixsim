// Polja obrasca plana: broj s rasponom (i rečenicom što odluka mijenja) i potvrdni okvir s napomenom.
import { t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import { formatRange } from "../../format";

export function NumberField(props: {
  id: string;
  label: string;
  value: string;
  min: number;
  max: number;
  /** Jedinica raspona ispod polja (UI_PRAVILA §2.1: broj ima ime i jedinicu, i u rasponu). */
  unit: "euro" | "percent" | "person";
  /** Što odluka mijenja i koliko stoji (UI I3); opis polja za čitač zaslona. */
  note?: string | undefined;
  onChange: (v: string) => void;
}) {
  const rangeId = `${props.id}-range`;
  const noteId = props.note ? `${props.id}-note` : null;
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
        aria-describedby={noteId ? `${rangeId} ${noteId}` : rangeId}
        onChange={(e) => props.onChange(e.target.value)}
        className="min-h-12 w-full rounded-2xl border border-line bg-surface px-3 text-lg tabular-nums"
      />
      <p id={rangeId} className="text-xs text-muted">
        {t("plan.range", { range: formatRange(props.min, props.max, props.unit) })}
      </p>
      {noteId && (
        <p id={noteId} className="text-sm text-muted">
          {props.note}
        </p>
      )}
    </div>
  );
}

export function Toggle(props: {
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
