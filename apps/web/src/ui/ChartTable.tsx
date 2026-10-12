// Tablica „Prikaži brojke“ uz krivulju napretka: retci = crte, stupci = točke. Na uskom ekranu pomiče se samo
// tablica (regija s fokusom), nikad stranica.
import type { ChartColumn, ChartLine, ChartText } from "./ProgressChart";

export function ChartTable({
  lines,
  columns,
  formatValue,
  text,
}: {
  lines: ChartLine[];
  columns: ChartColumn[];
  formatValue: (hundredths: number) => string;
  text: ChartText;
}) {
  return (
    // biome-ignore lint/a11y/useSemanticElements: regija oko pomične tablice, unutar kartice koja je već <section>
    <div
      role="region"
      aria-label={text.table}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: pomična tablica mora biti dostupna tipkovnicom
      tabIndex={0}
      className="overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <table className="w-full border-separate border-spacing-0 text-left text-sm">
        <caption className="sr-only">{text.table}</caption>
        <thead>
          <tr>
            <th
              scope="col"
              className="sticky left-0 z-10 border-b border-line bg-surface py-2 pr-3 font-semibold text-muted"
            >
              {text.rowHeader}
            </th>
            {columns.map((c) => (
              <th
                key={c.full}
                scope="col"
                className="whitespace-nowrap border-b border-line px-2 py-2 text-right font-semibold"
              >
                <span aria-hidden="true">{c.short}</span>
                <span className="sr-only">{c.full}</span>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {lines.map((l) => (
            <tr key={l.id}>
              <th
                scope="row"
                className="sticky left-0 z-10 min-w-36 border-b border-line bg-surface py-2 pr-3 font-semibold"
              >
                {l.label}
              </th>
              {l.values.map((v, i) => (
                <td
                  // biome-ignore lint/suspicious/noArrayIndexKey: stupac je redni broj točke, stalan
                  key={i}
                  className="whitespace-nowrap border-b border-line px-2 py-2 text-right tabular-nums"
                >
                  {formatValue(v)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
