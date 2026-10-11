// screens/StartScreen.tsx — gost bez igre (DEMO §1.4, ADR-022 t. 3); student bira tjednu ili mjesečnu igru (novi model), sklopivi „Za profesora“ vraća stari model
import { t } from "@hotelixsim/i18n";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useStartDemo } from "../api/queries";
import type { NewGameBody, RoundKindCode } from "../api/types";
import { Link } from "../router";
import { Wordmark } from "../ui/Layout";
import { Button, ErrorText, StatusText } from "../ui/primitives";

/** Zadani model je novi (v2); stari (v1) se šalje praznim tijelom i bira se samo kod profesora. */
function settingsFor(legacy: boolean, kind: RoundKindCode): NewGameBody | undefined {
  return legacy ? undefined : { model: "v2", round_kind: kind };
}

/** Izbor vrste igre: kartica s radio gumbom, imenom i kratkim opisom (opis je `aria-describedby`, ne dio imena). */
function KindChoice({
  value,
  checked,
  label,
  help,
  onChange,
}: {
  value: RoundKindCode;
  checked: boolean;
  label: string;
  help: string;
  onChange: () => void;
}) {
  const nameId = `round-kind-${value}-name`;
  const helpId = `round-kind-${value}-help`;
  return (
    <label className="flex min-h-12 cursor-pointer items-start gap-3 rounded-2xl border border-line bg-surface p-3 has-[:checked]:border-accent has-[:checked]:bg-accent-soft has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent">
      <input
        type="radio"
        name="round-kind"
        value={value}
        checked={checked}
        onChange={onChange}
        aria-labelledby={nameId}
        aria-describedby={helpId}
        className="mt-0.5 size-5 shrink-0 accent-accent"
      />
      <span className="min-w-0">
        <span id={nameId} className="block font-bold">
          {label}
        </span>
        <span id={helpId} className="block text-sm text-muted">
          {help}
        </span>
      </span>
    </label>
  );
}

/** Student odmah bira tjednu ili mjesečnu igru; obje su novi model (odjeli i račun dobiti). */
function KindPicker({ kind, onKind }: { kind: RoundKindCode; onKind: (k: RoundKindCode) => void }) {
  return (
    <fieldset className="space-y-2">
      <legend className="pb-1 text-sm font-bold">{t("start.kind.legend")}</legend>
      <KindChoice
        value="week"
        checked={kind === "week"}
        label={t("start.kind.week")}
        help={t("start.kind.week.help")}
        onChange={() => onKind("week")}
      />
      <KindChoice
        value="month"
        checked={kind === "month"}
        label={t("start.kind.month")}
        help={t("start.kind.month.help")}
        onChange={() => onKind("month")}
      />
    </fieldset>
  );
}

/** Sklopivo „Za profesora“: jedino mjesto s kojeg se pokreće stari model (v1) radi usporedbe. */
function ProfessorSettings({ legacy, onLegacy }: { legacy: boolean; onLegacy: (v: boolean) => void }) {
  return (
    <details className="group rounded-card border border-line bg-surface">
      <summary className="flex min-h-12 cursor-pointer list-none items-center justify-between gap-2 rounded-card px-4 font-semibold focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent [&::-webkit-details-marker]:hidden">
        {t("start.pro")}
        <ChevronDown
          size={18}
          aria-hidden="true"
          className="shrink-0 transition-transform group-open:rotate-180"
        />
      </summary>
      <div className="space-y-1 px-4 pb-4">
        <label className="flex min-h-12 items-center gap-3 rounded-2xl font-semibold has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent">
          <input
            type="checkbox"
            checked={legacy}
            onChange={(e) => onLegacy(e.target.checked)}
            aria-describedby="legacy-help"
            className="size-5 shrink-0 accent-accent"
          />
          {t("start.model.v1")}
        </label>
        <p id="legacy-help" className="text-sm text-muted">
          {t("start.model.v1.help")}
        </p>
      </div>
    </details>
  );
}

/**
 * Početni ekran. Uz `inGame` (ruta „/nova“, iz igre) igra koja traje ostaje dok igrač ne započne novu, pa ekran
 * nema vlastiti okvir i ima izlaz „Natrag na igru“ koji ne šalje zahtjev.
 */
export function StartScreen({ inGame = false }: { inGame?: boolean }) {
  const start = useStartDemo();
  const inFlight = useRef(false);
  const [legacy, setLegacy] = useState(false);
  const [kind, setKind] = useState<RoundKindCode>("week");

  function begin() {
    if (start.isPending || inFlight.current) return;
    inFlight.current = true;
    start.mutate(settingsFor(legacy, kind), {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  const body = (
    <>
      {!legacy && <KindPicker kind={kind} onKind={setKind} />}
      {legacy && <p className="text-sm text-muted">{t("start.model.v1.active")}</p>}
      {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
      {start.isPending && <StatusText>{t("start.busy")}</StatusText>}
      <Button disabled={start.isPending} onClick={begin}>
        {t("start.button")}
      </Button>
      <ProfessorSettings legacy={legacy} onLegacy={setLegacy} />
    </>
  );

  if (inGame)
    return (
      <div className="mx-auto max-w-md space-y-4 md:max-w-xl">
        <h1 className="text-3xl font-extrabold tracking-tight">{t("start.newTitle")}</h1>
        <p className="text-muted">{t("start.newLead")}</p>
        {body}
        <Link
          to="/"
          className="flex min-h-12 items-center justify-center rounded-2xl bg-accent-soft font-bold text-accent-ink"
        >
          {t("start.back")}
        </Link>
      </div>
    );

  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col md:max-w-xl justify-center gap-6 px-6 py-10">
      <Wordmark size="lg" />
      <h1 className="text-4xl leading-tight font-extrabold tracking-tight">
        {t("start.title1")} <span className="text-accent">{t("start.title2")}</span>
      </h1>
      <p className="text-lg text-muted">{t("start.lead")}</p>
      <p className="font-semibold">{t("start.measure")}</p>
      {body}
    </main>
  );
}
