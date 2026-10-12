// Ekran „Kraj sezone“ s podacima: moj rezultat i pobjednik, konačni poredak, sezonski mentor, krivulja napretka,
// tablice (od 1024 px) i kartice „Kako su igrali“. Do 1024 px jedan stupac tim redom; od 1024 px gore dva stupca.
import { t } from "@hotelixsim/i18n";
import { Trophy } from "lucide-react";
import type { SeasonBody } from "../../api/types";
import { formatPointsPrecise, formatWholeSigned, progressTone } from "../../format";
import { Link } from "../../router";
import { ProgressChart } from "../../ui/ProgressChart";
import { Card } from "../../ui/primitives";
import { Leaderboard } from "../month/Leaderboard";
import { HotelCard } from "./HotelCard";
import {
  allTied,
  axisTicks,
  curveColumns,
  curveSeries,
  myHotel,
  rankingRows,
  rankLine,
  seasonDigits,
  strategySentence,
  winnerLine,
} from "./rows";
import { SeasonMentor } from "./SeasonMentor";
import { DecisionTable, IndexTable, MoneyTable } from "./SeasonTables";

const ACTION =
  "flex min-h-12 items-center justify-center rounded-2xl px-6 py-3 text-base font-bold focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";

function Result({ body }: { body: SeasonBody }) {
  const me = myHotel(body);
  const digits = seasonDigits(body);
  if (!me) return null;
  return (
    <Card tone="accent" pad="lg" className="space-y-2 text-center">
      <p
        className={`whitespace-nowrap text-5xl font-extrabold tabular-nums ${progressTone(me.progress_hundredths, digits)}`}
      >
        {formatPointsPrecise(me.progress_hundredths, digits)}
      </p>
      <p>{rankLine(body)}</p>
      <p className="flex items-center justify-center gap-2 font-bold text-ink">
        {/* Pehar uz svakog pobjednika, ali ne kad su svi jednaki (pregled koda KS-3 Minor 1, UI M-12). */}
        {!allTied(body) && <Trophy size={20} aria-hidden="true" className="shrink-0" />}
        <span>{winnerLine(body)}</span>
      </p>
      <p className="text-sm text-muted">{t("season.fairNote")}</p>
    </Card>
  );
}

export function SeasonView({ body }: { body: SeasonBody }) {
  const weekly = body.round_kind === "week";
  const digits = seasonDigits(body);
  const robots = body.hotels.some((h) => strategySentence(h, weekly) !== null);
  return (
    <>
      <div>
        <h1 className="text-3xl font-extrabold tracking-tight">{t("season.title")}</h1>
        <p className="text-muted">{t(weekly ? "season.intro.week" : "season.intro")}</p>
      </div>

      <div className="flex flex-col gap-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6">
        <div className="space-y-3">
          <Result body={body} />
          <Leaderboard rows={rankingRows(body)} title={t("season.ranking")} />
        </div>
        <SeasonMentor text={body.mentor?.text ?? null} />
      </div>

      <Card className="space-y-2">
        <h2 className="text-lg font-bold">{t("season.curve")}</h2>
        <ProgressChart
          lines={curveSeries(body)}
          ticks={axisTicks(body)}
          columns={curveColumns(body)}
          formatValue={(v) => formatPointsPrecise(v, digits)}
          formatAxis={(n) => t("season.curve.axis", { n: formatWholeSigned(n) })}
          text={{
            title: t("season.curve"),
            desc: (hotel, progress) => t("season.curve.desc", { hotel, progress }),
            zero: t("season.curve.zero"),
            show: t("season.curve.show"),
            hide: t("season.curve.hide"),
            table: t("season.curve.table"),
            rowHeader: t("season.curve.hotel"),
          }}
        />
      </Card>

      <IndexTable hotels={body.hotels} />
      <MoneyTable hotels={body.hotels} />

      {/* Od 1024 px kartice nose samo „Kako je igrao“ robota; bez robota tablice kažu sve, pa se sekcija skriva (N-7). */}
      <section aria-labelledby="kako-su-igrali" className={`space-y-3 ${robots ? "" : "lg:hidden"}`}>
        <div>
          <h2 id="kako-su-igrali" className="text-lg font-bold">
            {t("season.hotels")}
          </h2>
          {robots && <p className="text-sm text-muted">{t("season.robotsNote")}</p>}
        </div>
        {/* Do 1024 px jedan stupac: uz bočnu traku dvije kartice od 226 px stisnu imena indeksa preko brojki (UI C-A).
            Rečenica strategije je tekst za čitanje: od 1024 px dvije kartice u redu, tri tek od 1280 px (N-2). */}
        <div className="space-y-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-3 lg:space-y-0 xl:grid-cols-3">
          {body.hotels.map((h) => (
            <HotelCard key={h.hotel_id} h={h} body={body} digits={digits} />
          ))}
        </div>
      </section>
      <DecisionTable body={body} />

      <div className="flex flex-col gap-3 md:flex-row">
        <Link to="/nova" className={`${ACTION} bg-accent text-white`}>
          {t("season.newGame")}
        </Link>
        <Link to="/" className={`${ACTION} bg-accent-soft text-accent-ink`}>
          {t("season.back")}
        </Link>
      </div>
    </>
  );
}
