// KpiScreen — hotelski pokazatelji po obračunatom krugu (mjesecu ili tjednu): na računalu tablica pokazatelji × krugovi,
// na mobitelu kartica po krugu. Uz svaki pokazatelj jedna rečenica što znači; brojke su s poslužitelja.
import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useGame, useKpi } from "../api/queries";
import type { GameBody, HotelBody, KpiMonthBody } from "../api/types";
import { capitalize, monthName, roundTitle, tw } from "../format";
import { isWeekly, roundInfo } from "../game";
import { Link } from "../router";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { breakEvenSentence, KPI_ROWS, rowLabel, rowMeaning } from "./kpi/rows";

/** Naslov kruga („Lipanj“, „Tjedan 5 (29. 6. – 5. 7.)“); bez igre ili kruga u njoj ime mjeseca. */
function roundLabel(game: GameBody | undefined, m: KpiMonthBody): string {
  const info = game ? roundInfo(game, m.round) : null;
  return info ? roundTitle(info) : capitalize(monthName(m.month));
}

type View = { game: GameBody | undefined; weekly: boolean; months: KpiMonthBody[] };

const CELL = "border-b border-line";

/**
 * Računalo (od 1024 px): jedan red po pokazatelju, stupac po krugu. Pomična regija prima tipkovnicu, prvi stupac
 * (naziv pokazatelja) ostaje vidljiv dok se pomiču tjedni, a objašnjenje je sklopljeno da redovi ostanu niski.
 */
function KpiTable({ game, weekly, months }: View) {
  const caption = tw(weekly, "kpi.table");
  return (
    <Card className="hidden lg:block" pad="sm">
      {/* biome-ignore lint/a11y/useSemanticElements: vanjski <section> je Card; regija se ne ugnježđuje u drugi section */}
      <div
        role="region"
        aria-label={caption}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: pomična tablica mora biti dostupna tipkovnicom
        tabIndex={0}
        className="overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <table className="w-full border-separate border-spacing-0 text-left">
          <caption className="sr-only">{caption}</caption>
          <thead>
            <tr>
              <th
                scope="col"
                className={`${CELL} sticky left-0 z-10 w-64 min-w-64 bg-surface py-2 pr-4 pl-2 text-sm font-semibold text-muted`}
              >
                {t("kpi.indicator")}
              </th>
              {months.map((m) => (
                <th
                  key={m.round}
                  scope="col"
                  className={`${CELL} min-w-28 whitespace-nowrap px-3 py-2 text-right font-bold`}
                >
                  {roundLabel(game, m)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {KPI_ROWS.map((r) => (
              <tr key={r.key}>
                <th
                  scope="row"
                  className={`${CELL} sticky left-0 z-10 w-64 min-w-64 bg-surface py-2 pr-4 pl-2 align-top font-normal`}
                >
                  <span id={`kpi-${r.key}`} className="block font-semibold">
                    {rowLabel(r.key)}
                  </span>
                  <details className="text-sm text-muted">
                    <summary
                      aria-describedby={`kpi-${r.key}`}
                      className="inline-flex min-h-11 cursor-pointer items-center rounded-md focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                    >
                      {t("kpi.whatMeans")}
                    </summary>
                    {rowMeaning(r.key, weekly)}
                  </details>
                </th>
                {months.map((m) => (
                  <td
                    key={m.round}
                    className={`${CELL} whitespace-nowrap px-3 py-2 text-right align-top tabular-nums`}
                  >
                    {r.value(m)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

/** Računalo: točka pokrića rečenicom za svaki krug, ispod tablice. */
function BreakEvenList({ game, weekly, months }: View) {
  return (
    <Card className="hidden space-y-2 lg:block">
      <h2 className="text-lg font-bold">{t("kpi.breakEven")}</h2>
      <ul className="space-y-1">
        {months.map((m) => (
          <li key={m.round}>
            <strong>{roundLabel(game, m)}:</strong> {breakEvenSentence(m, weekly)}
          </li>
        ))}
      </ul>
    </Card>
  );
}

/** Mobitel i tablet: kartica po krugu, pokazatelj s objašnjenjem i vrijednošću. */
function KpiCards({ game, weekly, months }: View) {
  return (
    <div className="space-y-3 md:grid md:grid-cols-2 md:items-start md:gap-3 md:space-y-0 lg:hidden">
      {months.map((m) => (
        <Card key={m.round} className="space-y-2">
          <h2 className="text-lg font-bold">{roundLabel(game, m)}</h2>
          <dl className="divide-y divide-line">
            {KPI_ROWS.map((r) => (
              <div key={r.key} className="flex items-start justify-between gap-3 py-2">
                <dt className="min-w-0">
                  <span className="block font-semibold">{rowLabel(r.key)}</span>
                  <span className="block text-xs text-muted">{rowMeaning(r.key, weekly)}</span>
                </dt>
                <dd className="whitespace-nowrap font-bold tabular-nums">{r.value(m)}</dd>
              </div>
            ))}
          </dl>
          <p className="rounded-xl bg-accent-soft p-3 text-sm">{breakEvenSentence(m, weekly)}</p>
        </Card>
      ))}
    </div>
  );
}

export function KpiScreen({ me }: { me: HotelBody }) {
  const q = useKpi(me.hotel.hotel_id);
  const gameQuery = useGame(me.game_id);
  // Bez igre (greška) pokazatelji se ipak vide, s imenima mjeseci umjesto naslova krugova.
  const game = gameQuery.data;
  const weekly = game ? isWeekly(game) : false;
  return (
    <>
      <Link to="/" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink">
        <ChevronLeft size={18} aria-hidden="true" />
        {t("kpi.back")}
      </Link>
      <div className="max-w-2xl">
        <h1 className="text-3xl font-extrabold tracking-tight">{t("kpi.title")}</h1>
        <p className="text-muted">{tw(weekly, "kpi.intro")}</p>
      </div>
      {q.isPending || gameQuery.isPending ? (
        <StatusText>{t("status.loading")}</StatusText>
      ) : q.isError ? (
        <ErrorText>{problemMessage(q.error)}</ErrorText>
      ) : q.data.months.length === 0 ? (
        <Card>
          <p>{tw(weekly, "kpi.empty")}</p>
        </Card>
      ) : (
        <>
          <KpiTable game={game} weekly={weekly} months={q.data.months} />
          <BreakEvenList game={game} weekly={weekly} months={q.data.months} />
          <KpiCards game={game} weekly={weekly} months={q.data.months} />
        </>
      )}
    </>
  );
}
