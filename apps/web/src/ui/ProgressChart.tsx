// Krivulja napretka (ručni SVG, bez knjižnice): jedna crta po hotelu od početka (0) kroz sve krugove. Boja nikad ne
// stoji sama — svaka crta ima svoju debljinu, uzorak (puna, crtkana, točkasta) i znak na zadnjoj točki, a legenda i
// tablica „Prikaži brojke“ nose ista imena i brojke. SVG se crta u pikselima svoje stvarne širine, pa slova i
// debljine ostaju iste na mobitelu i računalu; od 680 px širine okvira (računalo) ime i konačni broj stoje i uz kraj crte.
import { useId, useLayoutEffect, useRef, useState } from "react";
import { ChartTable } from "./ChartTable";
import { spreadLabels, yRange } from "./chartScale";

export type ChartLine = {
  id: string;
  /** Puno ime (legenda, tablica, opis): „Brižni (robot)“. */
  label: string;
  /** Kratko ime uz kraj crte: „Brižni“. */
  short: string;
  /** 0 = istaknuta crta (moj hotel), 1–3 ostale. */
  style: 0 | 1 | 2 | 3;
  /** Vrijednosti u stotinkama, redom točaka (prva je početak). */
  values: number[];
  /** Konačna vrijednost riječima. */
  valueText: string;
};

/** Granica na osi X (indeks točke); oznaka stoji nad dijelom od prethodne granice do ove, prva je sama točka 0. */
export type ChartTick = { at: number; text: string; short: string };

export type ChartColumn = { short: string; full: string };

export type ChartText = {
  title: string;
  desc: (label: string, value: string) => string;
  zero: string;
  show: string;
  hide: string;
  table: string;
  rowHeader: string;
};

type Look = {
  stroke: string;
  fill: string;
  width: number;
  dash: string;
  marker: "circle" | "square" | "triangle" | "diamond";
};

const LOOKS: Record<ChartLine["style"], Look> = {
  0: { stroke: "stroke-accent", fill: "fill-accent", width: 3, dash: "none", marker: "circle" },
  1: { stroke: "stroke-ink", fill: "fill-ink", width: 2, dash: "none", marker: "square" },
  2: { stroke: "stroke-ink", fill: "fill-ink", width: 2, dash: "6 4", marker: "triangle" },
  3: { stroke: "stroke-muted", fill: "fill-muted", width: 2, dash: "2 4", marker: "diamond" },
};

const WIDE_FROM = 680;
// Desni rub za ime i broj uz kraj crte („Lovac na popunjenost −7 bodova“ stane u 230 px slovima od 12 px).
const LABELS_W = 230;
const PAD = { left: 56, top: 12, bottom: 28 };

function Marker({ look, x, y }: { look: Look; x: number; y: number }) {
  const cls = look.fill;
  if (look.marker === "circle") return <circle data-marker cx={x} cy={y} r={5} className={cls} />;
  if (look.marker === "square")
    return <rect data-marker x={x - 4} y={y - 4} width={8} height={8} className={cls} />;
  if (look.marker === "triangle")
    return (
      <polygon data-marker points={`${x},${y - 5} ${x + 5},${y + 4} ${x - 5},${y + 4}`} className={cls} />
    );
  return (
    <polygon data-marker points={`${x},${y - 5} ${x + 5},${y} ${x},${y + 5} ${x - 5},${y}`} className={cls} />
  );
}

/** Mali uzorak crte za legendu (dekorativan; ime i broj su tekst uz njega). */
function Swatch({ look }: { look: Look }) {
  return (
    <svg width="36" height="12" viewBox="0 0 36 12" aria-hidden="true" className="shrink-0">
      <line
        x1="1"
        y1="6"
        x2="27"
        y2="6"
        className={look.stroke}
        strokeWidth={look.width}
        strokeDasharray={look.dash}
        strokeLinecap="round"
      />
      <Marker look={look} x={30} y={6} />
    </svg>
  );
}

/** Stvarna širina okvira (u testu bez ResizeObservera ostaje zadana). */
function useWidth(initial: number) {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(initial);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(([entry]) => {
      const w = Math.round(entry?.contentRect.width ?? 0);
      if (w > 0) setWidth(w);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);
  return [ref, width] as const;
}

export function ProgressChart({
  lines,
  ticks,
  columns,
  formatValue,
  formatAxis,
  text,
}: {
  lines: ChartLine[];
  ticks: ChartTick[];
  columns: ChartColumn[];
  formatValue: (hundredths: number) => string;
  /** Oznaka osi Y za cijeli broj bodova: „+5 bod.“ */
  formatAxis: (points: number) => string;
  text: ChartText;
}) {
  const id = useId();
  const [ref, width] = useWidth(WIDE_FROM);
  const [showTable, setShowTable] = useState(false);
  const wide = width >= WIDE_FROM;
  const height = wide ? 240 : 180;
  const right = wide ? LABELS_W : 16;
  const plotW = Math.max(1, width - PAD.left - right);
  const plotH = height - PAD.top - PAD.bottom;
  const points = Math.max(2, ...lines.map((l) => l.values.length));
  const { lo, hi, step } = yRange(lines.flatMap((l) => l.values));
  const x = (i: number) => PAD.left + (i * plotW) / (points - 1);
  const y = (v: number) => PAD.top + ((hi - v) * plotH) / (hi - lo);
  const gridValues: number[] = [];
  for (let v = lo; v <= hi; v += step) gridValues.push(v);
  const ends = lines.map((l) => y(l.values.at(-1) ?? 0));
  const labelYs = spreadLabels(ends, 15, PAD.top + 4, PAD.top + plotH);

  return (
    <div className="space-y-3">
      <div ref={ref} className="w-full">
        <svg
          role="img"
          aria-labelledby={`${id}-t`}
          aria-describedby={`${id}-d`}
          width="100%"
          height={height}
          viewBox={`0 0 ${width} ${height}`}
          className="block"
        >
          <title id={`${id}-t`}>{text.title}</title>
          <desc id={`${id}-d`}>{lines.map((l) => text.desc(l.label, l.valueText)).join(" ")}</desc>
          {gridValues.map((v) => (
            <g key={v}>
              <line
                x1={PAD.left}
                x2={PAD.left + plotW}
                y1={y(v)}
                y2={y(v)}
                className={v === 0 ? "stroke-muted" : "stroke-line"}
                strokeWidth={v === 0 ? 1.5 : 1}
              />
              <text x={PAD.left - 6} y={y(v) + 4} textAnchor="end" className="fill-muted text-[11px]">
                {formatAxis(v / 100)}
              </text>
            </g>
          ))}
          {ticks.map((tk, i) => {
            const prev = ticks[i - 1];
            const label = wide ? tk.text : tk.short;
            const tx = prev ? (x(prev.at) + x(tk.at)) / 2 : x(tk.at);
            return (
              <g key={`${tk.at}-${tk.text}`}>
                {i > 0 && (
                  <line
                    x1={x(tk.at)}
                    x2={x(tk.at)}
                    y1={PAD.top + plotH}
                    y2={PAD.top + plotH + 5}
                    className="stroke-muted"
                  />
                )}
                <text
                  x={tx}
                  y={height - 8}
                  textAnchor={prev ? "middle" : "start"}
                  className="fill-muted text-[11px]"
                >
                  {label}
                </text>
              </g>
            );
          })}
          {lines.map((l) => {
            const look = LOOKS[l.style];
            const d = l.values.map((v, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(v)}`).join(" ");
            return (
              <path
                key={l.id}
                data-line={l.id}
                data-style={l.style}
                d={d}
                fill="none"
                className={look.stroke}
                strokeWidth={look.width}
                strokeDasharray={look.dash}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            );
          })}
          {lines.map((l) => {
            const last = l.values.length - 1;
            return <Marker key={l.id} look={LOOKS[l.style]} x={x(last)} y={y(l.values[last] ?? 0)} />;
          })}
          {wide &&
            lines.map((l, i) => (
              <text
                key={l.id}
                x={PAD.left + plotW + 10}
                y={(labelYs[i] ?? 0) + 4}
                className={`text-xs ${l.style === 0 ? "fill-ink font-bold" : "fill-ink"}`}
              >
                {`${l.short} ${l.valueText}`}
              </text>
            ))}
        </svg>
      </div>
      <p className="text-sm text-muted">{text.zero}</p>
      <ul aria-label={text.title} className="grid gap-x-4 gap-y-1 text-sm sm:grid-cols-2">
        {lines.map((l) => (
          <li key={l.id} className="flex min-w-0 items-center gap-2">
            <Swatch look={LOOKS[l.style]} />
            <span className={`min-w-0 ${l.style === 0 ? "font-bold" : ""}`}>{l.label}</span>
            <span className="ml-auto whitespace-nowrap font-semibold tabular-nums">{l.valueText}</span>
          </li>
        ))}
      </ul>
      <button
        type="button"
        aria-expanded={showTable}
        aria-controls={`${id}-table`}
        onClick={() => setShowTable((v) => !v)}
        className="inline-flex min-h-11 items-center rounded-xl px-1 font-semibold text-accent-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        {showTable ? text.hide : text.show}
      </button>
      <div id={`${id}-table`} hidden={!showTable}>
        {showTable && <ChartTable lines={lines} columns={columns} formatValue={formatValue} text={text} />}
      </div>
    </div>
  );
}
