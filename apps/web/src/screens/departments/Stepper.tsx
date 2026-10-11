// Brojač s dva gumba (sezonski radnici, razina cijene u odjelu). Gumb koji ne smije dalje ostaje u redu tipkovnice:
// pravi `disabled` bi uzeo fokus pa bi pao na `<body>` (UI_PRAVILA §6.4), pa onemogućenost čitaču daje `aria-disabled`,
// a klik ne radi ništa. Prigušuje se samo sadržaj gumba, ne obris fokusa.
import { FOCUS } from "./helpers";

const BUTTON = `group inline-flex min-h-11 items-center justify-center rounded-xl border border-line bg-surface font-bold aria-disabled:cursor-not-allowed aria-disabled:border-line/40 ${FOCUS}`;
const DIM = "group-aria-disabled:opacity-40";

function StepButton(props: {
  /** Vidljivo ime („Zaposli sezonskog radnika“) ili samo znak uz `aria-label`. */
  text: string | null;
  mark: "+" | "−";
  label: string;
  disabled: boolean;
  describedBy: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={props.text ? undefined : props.label}
      aria-describedby={props.describedBy}
      aria-disabled={props.disabled}
      onClick={props.disabled ? undefined : props.onClick}
      className={`${BUTTON} ${props.text ? "w-full px-3 py-2 text-sm" : "size-11 text-xl"}`}
    >
      <span aria-hidden={props.text ? undefined : "true"} className={DIM}>
        {props.text ?? props.mark}
      </span>
    </button>
  );
}

export function Stepper(props: {
  label: string;
  value: string;
  moreLabel: string;
  lessLabel: string;
  moreDisabled: boolean;
  lessDisabled: boolean;
  /** Id imena odjela: ponovljeni gumbi se razlikuju po opisu (UI_PRAVILA §6.6). */
  describedBy: string;
  /** Gumbi s vidljivim imenom ispod vrijednosti (sezonski radnici); inače − i + uz vrijednost. */
  named?: boolean;
  onMore: () => void;
  onLess: () => void;
}) {
  const less = (
    <StepButton
      text={props.named ? props.lessLabel : null}
      mark="−"
      label={props.lessLabel}
      disabled={props.lessDisabled}
      describedBy={props.describedBy}
      onClick={props.onLess}
    />
  );
  const more = (
    <StepButton
      text={props.named ? props.moreLabel : null}
      mark="+"
      label={props.moreLabel}
      disabled={props.moreDisabled}
      describedBy={props.describedBy}
      onClick={props.onMore}
    />
  );
  const output = (
    <output className="min-w-0 flex-1 text-center text-base font-bold tabular-nums whitespace-nowrap">
      {props.value}
    </output>
  );
  return (
    <div className="space-y-1">
      <p className="font-semibold">{props.label}</p>
      {props.named ? (
        <div className="space-y-2">
          <div className="flex">{output}</div>
          <div className="grid grid-cols-2 gap-2">
            {less}
            {more}
          </div>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          {less}
          {output}
          {more}
        </div>
      )}
    </div>
  );
}
