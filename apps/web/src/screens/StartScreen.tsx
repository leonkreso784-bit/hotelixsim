// screens/StartScreen.tsx — gost bez igre (DEMO §1.4, ADR-022 t. 3); sklopivi dio „Za profesora“ bira model igre
import { t } from "@hotelixsim/i18n";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useStartDemo } from "../api/queries";
import type { ModelCode, NewGameBody, RoundKindCode } from "../api/types";
import { Wordmark } from "../ui/Layout";
import { Button, ErrorText, StatusText } from "../ui/primitives";

/** Zadani model je demo (prazno tijelo); tijelo se šalje samo kad je izabran novi model. */
function settingsFor(model: ModelCode, kind: RoundKindCode): NewGameBody | undefined {
  return model === "v2" ? { model, round_kind: kind } : undefined;
}

function Choice({
  name,
  value,
  checked,
  label,
  onChange,
}: {
  name: string;
  value: string;
  checked: boolean;
  label: string;
  onChange: () => void;
}) {
  return (
    <label className="flex min-h-11 items-center gap-3 rounded-2xl px-1 font-semibold has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent">
      <input
        type="radio"
        name={name}
        value={value}
        checked={checked}
        onChange={onChange}
        className="size-5 shrink-0 accent-accent"
      />
      {label}
    </label>
  );
}

/** Postavke nove igre za profesora: model (demo ili novi) i, za novi, duljina kruga. */
function ProfessorSettings({
  model,
  kind,
  onModel,
  onKind,
}: {
  model: ModelCode;
  kind: RoundKindCode;
  onModel: (m: ModelCode) => void;
  onKind: (k: RoundKindCode) => void;
}) {
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
      <div className="space-y-4 px-4 pb-4">
        <fieldset className="space-y-1">
          <legend className="pb-1 text-sm font-bold">{t("start.model.legend")}</legend>
          <Choice
            name="model"
            value="v1"
            checked={model === "v1"}
            label={t("start.model.v1")}
            onChange={() => onModel("v1")}
          />
          <Choice
            name="model"
            value="v2"
            checked={model === "v2"}
            label={t("start.model.v2")}
            onChange={() => onModel("v2")}
          />
        </fieldset>
        {model === "v2" && (
          <fieldset className="space-y-1">
            <legend className="pb-1 text-sm font-bold">{t("start.kind.legend")}</legend>
            <Choice
              name="round-kind"
              value="week"
              checked={kind === "week"}
              label={t("start.kind.week")}
              onChange={() => onKind("week")}
            />
            <Choice
              name="round-kind"
              value="month"
              checked={kind === "month"}
              label={t("start.kind.month")}
              onChange={() => onKind("month")}
            />
            <p className="pt-1 text-sm text-muted">{t("start.kind.help")}</p>
          </fieldset>
        )}
      </div>
    </details>
  );
}

export function StartScreen() {
  const start = useStartDemo();
  const inFlight = useRef(false);
  const [model, setModel] = useState<ModelCode>("v1");
  const [kind, setKind] = useState<RoundKindCode>("week");

  function begin() {
    if (start.isPending || inFlight.current) return;
    inFlight.current = true;
    start.mutate(settingsFor(model, kind), {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col md:max-w-xl justify-center gap-6 px-6 py-10">
      <Wordmark size="lg" />
      <h1 className="text-4xl leading-tight font-extrabold tracking-tight">
        {t("start.title1")} <span className="text-accent">{t("start.title2")}</span>
      </h1>
      <p className="text-lg text-muted">{t("start.lead")}</p>
      <p className="font-semibold">{t("start.measure")}</p>
      {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
      {start.isPending && <StatusText>{t("start.busy")}</StatusText>}
      <Button disabled={start.isPending} onClick={begin}>
        {t("start.button")}
      </Button>
      <ProfessorSettings model={model} kind={kind} onModel={setModel} onKind={setKind} />
    </main>
  );
}
