export function Choice<T extends string>(props: {
  legend: string;
  name: string;
  value: T;
  options: { v: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <fieldset className="space-y-1">
      <legend className="font-semibold">{props.legend}</legend>
      <div className="grid grid-cols-3 gap-2">
        {props.options.map((o) => (
          <label
            key={o.v}
            className={`flex min-h-11 cursor-pointer items-center justify-center rounded-xl border text-sm font-semibold has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent ${
              props.value === o.v ? "border-accent bg-accent-soft text-accent-ink" : "border-line bg-surface"
            }`}
          >
            <input
              type="radio"
              name={props.name}
              value={o.v}
              checked={props.value === o.v}
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
