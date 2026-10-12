// Ekran „Kraj sezone“ s podacima: moj rezultat i pobjednik, konačni poredak, sezonski mentor, krivulja napretka,
// tablice (od 1024 px) i kartice „Kako su igrali“. Mobitel: jedan stupac tim redom; od 1024 px gore dva stupca.
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
  axisTicks,
  curveColumns,
  curveSeries,
  myHotel,
  rankingRows,
  rankLine,
  seasonDigits,
  winnerLine,
} from "./rows";
import { SeasonMentor } from "./SeasonMentor";
import { IndexTable, MoneyTable } from "./SeasonTables";

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
        {me.winner && <Trophy size={20} aria-hidden="true" className="shrink-0" />}
        <span>{winnerLine(body)}</span>
      </p>
      <p className="text-sm text-muted">{t("season.fairNote")}</p>
    </Card>
  );
}

export function SeasonView({ body }: { body: SeasonBody }) {
  const weekly = body.round_kind === "week";
  const digits = seasonDigits(body);
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

      <section aria-labelledby="kako-su-igrali" className="space-y-3">
        <div>
          <h2 id="kako-su-igrali" className="text-lg font-bold">
            {t("season.hotels")}
          </h2>
          <p className="text-sm text-muted">{t("season.robotsNote")}</p>
        </div>
        <div className="space-y-3 md:grid md:grid-cols-2 md:items-start md:gap-3 md:space-y-0 xl:grid-cols-4">
          {body.hotels.map((h) => (
            <HotelCard key={h.hotel_id} h={h} body={body} digits={digits} />
          ))}
        </div>
      </section>

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
