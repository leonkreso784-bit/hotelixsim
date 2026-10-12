// Kartica jednog hotela na kraju sezone: kako je igrao (robot: pravilo riječima), odluke u brojkama (isti retci za
// robota i za mene), a ispod 1024 px i indeksi početak → kraj i financije sezone (od 1024 px su u tablicama).
// Kartice robota su na mobitelu sklopljene, od 768 px otvorene; moja je uvijek otvorena.
import { t } from "@hotelixsim/i18n";
import { ChevronDown } from "lucide-react";
import type { SeasonBody, SeasonHotelBody } from "../../api/types";
import { Avatar } from "../../avatar/Avatar";
import { lookFor, PLAYER_LOOK } from "../../avatar/look";
import { formatPointsPrecise, type PointDigits } from "../../format";
import { indexDeltas } from "../../game";
import { IndexBars } from "../../ui/IndexBars";
import { Card } from "../../ui/primitives";
import { useMinWidth } from "../../ui/useMinWidth";
import { decisionRows, financeRows, hotelLabel, strategySentence } from "./rows";

const H4 = "text-sm font-bold";

function seatLabel(h: SeasonHotelBody): string {
  if (h.mine) return t("season.mine");
  return t(h.seat.kind === "robot" ? "season.robot" : "season.otherHuman");
}

function Head({ h, digits }: { h: SeasonHotelBody; digits: PointDigits }) {
  return (
    <span className="flex min-w-0 flex-1 items-center gap-3">
      <Avatar look={h.mine ? PLAYER_LOOK : lookFor(h.name)} size={40} />
      <span className="min-w-0 flex-1">
        <h3 className="text-lg font-bold break-words">
          {h.rank}. {hotelLabel(h)}
        </h3>
        <span className="block text-sm text-muted">
          {seatLabel(h)} ·{" "}
          <span className="whitespace-nowrap">{formatPointsPrecise(h.progress_hundredths, digits)}</span>
        </span>
      </span>
    </span>
  );
}

function Rows({ rows }: { rows: { key: string; label: string; value: string; note?: string | null }[] }) {
  return (
    <dl className="divide-y divide-line text-sm">
      {rows.map((r) => (
        <div key={r.key} className="flex flex-wrap items-baseline justify-between gap-x-3 py-1.5">
          <dt className="min-w-0">{r.label}</dt>
          <dd className="text-right font-semibold tabular-nums">
            {r.value}
            {r.note && <span className="block text-xs font-normal text-muted">{r.note}</span>}
          </dd>
        </div>
      ))}
    </dl>
  );
}

function Body({ h, body }: { h: SeasonHotelBody; body: SeasonBody }) {
  const sentence = strategySentence(h);
  const decisions = decisionRows(h, body);
  const money = financeRows(h);
  return (
    <div className="space-y-3 pt-3">
      {sentence && (
        <div className="space-y-1">
          <h4 className={H4}>{t("season.howPlayed")}</h4>
          <p className="text-sm">{sentence}</p>
        </div>
      )}
      {decisions ? (
        <div className="space-y-1">
          <h4 className={H4}>{t(h.mine ? "season.myDecisions" : "season.decisions")}</h4>
          <Rows rows={decisions} />
        </div>
      ) : (
        <p className="text-sm text-muted">{t("season.private")}</p>
      )}
      <div className="space-y-3 lg:hidden">
        <div className="space-y-1">
          <h4 className={H4}>{t("season.indices")}</h4>
          <IndexBars
            indices={h.end}
            from={h.start}
            deltas={indexDeltas(h.start, h.end)}
            deltaLabel={t("season.inSeason")}
            linked={h.mine}
          />
        </div>
        {money && (
          <div className="space-y-1">
            <h4 className={H4}>{t("season.money")}</h4>
            <Rows rows={money} />
          </div>
        )}
      </div>
    </div>
  );
}

export function HotelCard({
  h,
  body,
  digits,
}: {
  h: SeasonHotelBody;
  body: SeasonBody;
  digits: PointDigits;
}) {
  const wide = useMinWidth(768);
  if (h.mine)
    return (
      <Card emphasis className="min-w-0">
        <Head h={h} digits={digits} />
        <Body h={h} body={body} />
      </Card>
    );
  return (
    <Card className="min-w-0">
      <details open={wide} className="group">
        <summary className="flex min-h-11 cursor-pointer list-none items-center gap-2 rounded-xl focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent [&::-webkit-details-marker]:hidden">
          <Head h={h} digits={digits} />
          <ChevronDown size={20} aria-hidden="true" className="shrink-0 text-muted group-open:rotate-180" />
        </summary>
        <Body h={h} body={body} />
      </details>
    </Card>
  );
}
