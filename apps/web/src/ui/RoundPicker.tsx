// ui/RoundPicker.tsx — izbor kruga (UI_PRAVILA §8); jedna komponenta za sve ekrane s krugovima.
// Mjeseci (`many` je false): gumbi u redu na svim širinama. Tjedni (`many`): ispod 1024 px ‹ › i izbornik,
// od 1024 px sažeti čipovi („T5“) s punim nazivom u `aria-label` i `title`, grupirani po mjesecima (§8).
import { t } from "@hotelixsim/i18n";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useRef } from "react";
import type { RoundInfoBody } from "../api/types";
import { capitalize, monthName } from "../format";

export type PickerRound = {
  code: string;
  /** Puni naziv: „Tjedan 5 (29. 6. – 5. 7.)“. */
  title: string;
  /** Sažetak za čip: „T5“. */
  short: string;
  /** Mjesec u kojem tjedan počinje („Lipanj“): čipovi na računalu stoje u grupama po mjesecima. */
  group?: string;
};

/** Mjesec kojem tjedan pripada za grupiranje čipova (onaj u kojem počinje). */
export function weekGroup(info: RoundInfoBody): string {
  return capitalize(monthName(info.first_month));
}

/** Uzastopni krugovi istog mjeseca u jednoj grupi; bez oznake mjeseca na svim krugovima ostaje jedna grupa bez imena. */
function groupRounds(rounds: PickerRound[]): { name: string | null; rounds: PickerRound[] }[] {
  const groups: { name: string | null; rounds: PickerRound[] }[] = [];
  for (const r of rounds) {
    const name = r.group ?? null;
    const last = groups.at(-1);
    if (last && last.name === name) last.rounds.push(r);
    else groups.push({ name, rounds: [r] });
  }
  return groups;
}

const BASE = "min-h-11 rounded-2xl font-semibold";
const ON = "bg-accent text-white";
const OFF = "bg-accent-soft text-accent-ink";
const FOCUS = "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";

export function RoundPicker({
  rounds,
  value,
  onPick,
  legend,
  prevLabel,
  nextLabel,
  many,
}: {
  rounds: PickerRound[];
  value: string;
  onPick: (code: string) => void;
  legend: string;
  prevLabel: string;
  nextLabel: string;
  /** Mnogo krugova (tjedni): ‹ › i izbornik na mobitelu, čipovi na računalu. */
  many: boolean;
}) {
  const selectRef = useRef<HTMLSelectElement>(null);
  if (!many)
    return (
      <fieldset className="flex min-w-0 flex-wrap gap-2 pb-2">
        <legend className="sr-only">{legend}</legend>
        {rounds.map((r) => (
          <button
            key={r.code}
            type="button"
            aria-pressed={r.code === value}
            onClick={() => onPick(r.code)}
            className={`${BASE} ${FOCUS} px-4 ${r.code === value ? ON : OFF}`}
          >
            {r.title}
          </button>
        ))}
      </fieldset>
    );

  const index = Math.max(
    0,
    rounds.findIndex((r) => r.code === value),
  );
  const step = (d: number) => {
    const next = rounds[index + d];
    if (!next) return;
    onPick(next.code);
    // Gumb koji sad postaje onemogućen ne smije odnijeti fokus: prelazi na izbornik.
    if (index + d === 0 || index + d === rounds.length - 1) selectRef.current?.focus();
  };
  return (
    <fieldset className="min-w-0 pb-2">
      <legend className="sr-only">{legend}</legend>
      <div className="flex min-w-0 items-center gap-2 lg:hidden">
        <button
          type="button"
          aria-label={prevLabel}
          disabled={index === 0}
          onClick={() => step(-1)}
          className={`${BASE} ${FOCUS} ${OFF} grid min-w-11 place-items-center disabled:opacity-40`}
        >
          <ChevronLeft size={20} aria-hidden="true" />
        </button>
        <select
          ref={selectRef}
          aria-label={legend}
          value={value}
          onChange={(e) => onPick(e.target.value)}
          className={`${BASE} ${FOCUS} min-w-0 flex-1 border border-line bg-surface px-3 text-ink`}
        >
          {rounds.map((r) => (
            <option key={r.code} value={r.code}>
              {r.title}
            </option>
          ))}
        </select>
        <button
          type="button"
          aria-label={nextLabel}
          disabled={index === rounds.length - 1}
          onClick={() => step(1)}
          className={`${BASE} ${FOCUS} ${OFF} grid min-w-11 place-items-center disabled:opacity-40`}
        >
          <ChevronRight size={20} aria-hidden="true" />
        </button>
      </div>
      <div className="hidden flex-wrap gap-x-5 gap-y-2 lg:flex">
        {groupRounds(rounds).map((g) => (
          // biome-ignore lint/a11y/useSemanticElements: unutar fieldseta birača; grupa mjeseca nosi samo ime
          <div
            key={g.name ?? "svi"}
            role="group"
            aria-label={g.name ?? legend}
            className="flex flex-wrap items-center gap-2"
          >
            {g.name && (
              <span aria-hidden="true" className="text-sm font-semibold text-muted">
                {g.name}
              </span>
            )}
            {g.rounds.map((r) => (
              <button
                key={r.code}
                type="button"
                aria-label={r.title}
                title={r.title}
                aria-pressed={r.code === value}
                onClick={() => onPick(r.code)}
                className={`${BASE} ${FOCUS} min-w-11 px-3 ${r.code === value ? ON : OFF}`}
              >
                {r.short}
              </button>
            ))}
          </div>
        ))}
      </div>
    </fieldset>
  );
}

/** Sažeta oznaka čipa za tjedan („T5“). */
export function shortWeek(n: number): string {
  return t("round.short", { n });
}
