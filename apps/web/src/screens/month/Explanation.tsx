// month/Explanation.tsx — „Zašto se indeksi pomaknuli“: po indeksu pokretači i retci s poslužitelja (iste činjenice
// kao mentor). Ništa se ne računa ovdje; tekstovi se prikazuju kakvi jesu.
import { t } from "@hotelixsim/i18n";
import { ChevronDown } from "lucide-react";
import { problemMessage } from "../../api/errors";
import { useExplanation } from "../../api/queries";
import type { DriverCode, EffectCode, ExplanationIndexBody } from "../../api/types";
import { tw } from "../../format";
import { sortIndices } from "../../game";
import { Delta, INDEX_ICON } from "../../ui/IndexBars";
import { Card, ErrorText, StatusText } from "../../ui/primitives";

const TONE: Record<EffectCode, string> = { up: "text-up", down: "text-down", neutral: "text-muted" };

/** Prvi indeks s najvećom apsolutnom promjenom; on je otvoren po zadanom. */
function mostMoved(indices: ExplanationIndexBody[]): ExplanationIndexBody["code"] | null {
  let best: ExplanationIndexBody | null = null;
  for (const i of indices) if (!best || Math.abs(i.change) > Math.abs(best.change)) best = i;
  return best?.code ?? null;
}

/** Varijable koje čine indeks, s redcima i bojom po učinku. Istaknuta (ako je ima) ide na vrh i označena je. */
export function DriverList({
  drivers,
  highlight = null,
  weekly = false,
}: {
  drivers: ExplanationIndexBody["drivers"];
  highlight?: DriverCode | null;
  /** Tjedni krug: rečenice govore o tjednu. */
  weekly?: boolean;
}) {
  const ordered = [...drivers].sort((a, b) => Number(b.var === highlight) - Number(a.var === highlight));
  return (
    <div className="space-y-3 pb-3">
      {ordered.length === 0 && <p className="text-sm text-muted">{tw(weekly, "month.whyNone")}</p>}
      {ordered.map((d) => (
        <div
          key={d.var}
          aria-current={d.var === highlight ? "true" : undefined}
          className={d.var === highlight ? "rounded-xl bg-accent-soft p-2" : undefined}
        >
          <h3 className="font-semibold">{d.name}</h3>
          <p className="text-xs text-muted">{d.meaning}</p>
          <ul className="mt-1 text-sm">
            {d.lines.map((l) => (
              <li
                // Poslužitelj spaja isti uzrok iste varijable u jedan redak, pa je uzrok jedinstven.
                key={`${l.cause}|${l.amount}`}
                className="flex flex-wrap justify-between gap-x-3 border-b border-line py-1 last:border-0"
              >
                <span className="min-w-0">{l.cause}</span>
                <span className={`ml-auto text-right font-semibold tabular-nums ${TONE[l.effect]}`}>
                  {l.amount}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

function IndexWhy({ index, open, weekly }: { index: ExplanationIndexBody; open: boolean; weekly: boolean }) {
  return (
    <details open={open} className="group border-b border-line last:border-0">
      <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 py-2 [&::-webkit-details-marker]:hidden">
        <span className="text-accent">{INDEX_ICON[index.code]}</span>
        <span className="min-w-0 flex-1 font-semibold">{index.name}</span>
        <span className="text-sm whitespace-nowrap text-muted tabular-nums">
          {t("month.whyRange", { before: index.before, after: index.after })}
        </span>
        <span className="min-w-12 text-right">
          <Delta value={index.change} />
        </span>
        <ChevronDown size={18} aria-hidden="true" className="shrink-0 text-muted group-open:rotate-180" />
      </summary>
      <DriverList drivers={index.drivers} weekly={weekly} />
    </details>
  );
}

export function Explanation({
  hotelId,
  round,
  weekly = false,
  className = "",
}: {
  hotelId: string;
  /** Kod kruga: mjesec („jun“) ili tjedan („t05“). */
  round: string;
  weekly?: boolean;
  /** Samo raspored (npr. `order-*` na mobitelu). */
  className?: string;
}) {
  const q = useExplanation(hotelId, round);
  const indices = q.data ? sortIndices(q.data.indices) : [];
  const open = mostMoved(indices);
  return (
    <Card className={className}>
      <h2 className="pb-1 text-lg font-bold">{t("month.why")}</h2>
      {q.isPending ? (
        <StatusText>{t("status.loading")}</StatusText>
      ) : q.isError ? (
        <ErrorText>{problemMessage(q.error)}</ErrorText>
      ) : (
        indices.map((i) => <IndexWhy key={i.code} index={i} open={i.code === open} weekly={weekly} />)
      )}
    </Card>
  );
}
