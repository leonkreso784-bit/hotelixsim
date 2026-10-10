// KpiScreen — hotelski pokazatelji po obračunatom mjesecu (sloj 0): na računalu tablica pokazatelji × mjeseci,
// na mobitelu kartica po mjesecu. Uz svaki pokazatelj jedna rečenica što znači; brojke su s poslužitelja.
import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useKpi } from "../api/queries";
import type { HotelBody, KpiMonthBody } from "../api/types";
import { capitalize, monthName } from "../format";
import { Link } from "../router";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { breakEvenSentence, KPI_ROWS, rowLabel, rowMeaning } from "./kpi/rows";

function monthLabel(m: KpiMonthBody): string {
  return capitalize(monthName(m.month));
}

/** Računalo (od 1024 px): jedan red po pokazatelju, stupac po mjesecu. */
function KpiTable({ months }: { months: KpiMonthBody[] }) {
  return (
    <Card className="hidden overflow-x-auto lg:block">
      <table className="w-full text-left">
        <caption className="sr-only">{t("kpi.table")}</caption>
        <thead>
          <tr className="border-b border-line">
            <th scope="col" className="py-2 pr-4 text-sm font-semibold text-muted">
              {t("kpi.indicator")}
            </th>
            {months.map((m) => (
              <th key={m.month} scope="col" className="px-3 py-2 text-right font-bold">
                {monthLabel(m)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {KPI_ROWS.map((r) => (
            <tr key={r.key}>
              <th scope="row" className="max-w-md py-2 pr-4 align-top font-normal">
                <span className="block font-semibold">{rowLabel(r.key)}</span>
                <span className="block text-sm text-muted">{rowMeaning(r.key)}</span>
              </th>
              {months.map((m) => (
                <td key={m.month} className="whitespace-nowrap px-3 py-2 text-right align-top tabular-nums">
                  {r.value(m)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  );
}

/** Računalo: točka pokrića rečenicom za svaki mjesec, ispod tablice. */
function BreakEvenList({ months }: { months: KpiMonthBody[] }) {
  return (
    <Card className="hidden space-y-2 lg:block">
      <h2 className="text-lg font-bold">{t("kpi.breakEven")}</h2>
      <ul className="space-y-1">
        {months.map((m) => (
          <li key={m.month}>
            <strong>{monthLabel(m)}:</strong> {breakEvenSentence(m)}
          </li>
        ))}
      </ul>
    </Card>
  );
}

/** Mobitel i tablet: kartica po mjesecu, pokazatelj s objašnjenjem i vrijednošću. */
function KpiCards({ months }: { months: KpiMonthBody[] }) {
  return (
    <div className="space-y-3 md:grid md:grid-cols-2 md:items-start md:gap-3 md:space-y-0 lg:hidden">
      {months.map((m) => (
        <Card key={m.month} className="space-y-2">
          <h2 className="text-lg font-bold">{monthLabel(m)}</h2>
          <dl className="divide-y divide-line">
            {KPI_ROWS.map((r) => (
              <div key={r.key} className="flex items-start justify-between gap-3 py-2">
                <dt className="min-w-0">
                  <span className="block font-semibold">{rowLabel(r.key)}</span>
                  <span className="block text-xs text-muted">{rowMeaning(r.key)}</span>
                </dt>
                <dd className="whitespace-nowrap font-bold tabular-nums">{r.value(m)}</dd>
              </div>
            ))}
          </dl>
          <p className="rounded-xl bg-accent-soft p-3 text-sm">{breakEvenSentence(m)}</p>
        </Card>
      ))}
    </div>
  );
}

export function KpiScreen({ me }: { me: HotelBody }) {
  const q = useKpi(me.hotel.hotel_id);
  return (
    <>
      <Link to="/" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink">
        <ChevronLeft size={18} aria-hidden="true" />
        {t("kpi.back")}
      </Link>
      <div className="max-w-2xl">
        <h1 className="text-3xl font-extrabold tracking-tight">{t("kpi.title")}</h1>
        <p className="text-muted">{t("kpi.intro")}</p>
      </div>
      {q.isPending ? (
        <StatusText>{t("status.loading")}</StatusText>
      ) : q.isError ? (
        <ErrorText>{problemMessage(q.error)}</ErrorText>
      ) : q.data.months.length === 0 ? (
        <Card>
          <p>{t("kpi.empty")}</p>
        </Card>
      ) : (
        <>
          <KpiTable months={q.data.months} />
          <BreakEvenList months={q.data.months} />
          <KpiCards months={q.data.months} />
        </>
      )}
    </>
  );
}
