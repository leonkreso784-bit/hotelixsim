export function Choice<T extends string>(props: {
  legend: string;
  name: string;
  value: T;
  options: { v: T; label: string }[];
  onChange: (v: T) => void;
  /** Onemogući sve izbore (zatvoren odjel). */
  disabled?: boolean;
  /** Id elementa koji imenuje skupinu (ime odjela), da ponovljeni izbori budu razlikovni (UI_PRAVILA §6.6). */
  describedBy?: string;
}) {
  return (
    <fieldset className="space-y-1">
      <legend className="font-semibold">{props.legend}</legend>
      <div className="grid grid-cols-3 gap-2">
        {props.options.map((o) => (
          <label
            key={o.v}
            className={`flex min-h-11 cursor-pointer has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-40 items-center justify-center rounded-xl border text-sm font-semibold has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent ${
              props.value === o.v ? "border-accent bg-accent-soft text-accent-ink" : "border-line bg-surface"
            }`}
          >
            <input
              type="radio"
              name={props.name}
              value={o.v}
              checked={props.value === o.v}
              disabled={props.disabled}
              aria-describedby={props.describedBy}
              onChange={() => props.onChange(o.v)}
              className="sr-only"
            />
            {o.label}
          </label>
        ))}
      </div>
    </fieldset>
  );
}
