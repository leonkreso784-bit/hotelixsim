// Računalo (od 1024 px): indeksi početak → kraj i financije sezone svih hotela u dvije tablice. Isti retci kao
// kartice hotela ispod 1024 px (`rows.ts`), pa se brojke ne razilaze.
import { indexLabel, t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import type { SeasonHotelBody } from "../../api/types";
import { formatWhole } from "../../format";
import { INDEX_ORDER } from "../../game";
import { Delta, INDEX_ICON } from "../../ui/IndexBars";
import { Card } from "../../ui/primitives";
import { financeRows, gopMargin, hotelLabel, indexRows } from "./rows";

const CELL = "border-b border-line px-3 py-2";
const NUM = `${CELL} whitespace-nowrap text-right tabular-nums`;

function TableCard({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <Card className="hidden space-y-2 lg:block">
      <h2 id={id} className="text-lg font-bold">
        {title}
      </h2>
      {/* biome-ignore lint/a11y/useSemanticElements: vanjski <section> je Card; regija se ne ugnježđuje u drugi section */}
      <div
        role="region"
        aria-labelledby={id}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: pomična tablica mora biti dostupna tipkovnicom
        tabIndex={0}
        className="overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        {children}
      </div>
    </Card>
  );
}

function HotelCell({ h }: { h: SeasonHotelBody }) {
  return (
    <th scope="row" className={`${CELL} pl-0 text-left ${h.mine ? "font-bold" : "font-semibold"}`}>
      {h.rank}. {hotelLabel(h)}
    </th>
  );
}

export function IndexTable({ hotels }: { hotels: SeasonHotelBody[] }) {
  return (
    <TableCard id="sezona-indeksi" title={t("season.indicesTable")}>
      <table className="w-full border-separate border-spacing-0 text-sm">
        <caption className="sr-only">{t("season.indicesTable")}</caption>
        <thead>
          <tr>
            <th scope="col" className={`${CELL} pl-0 text-left font-semibold text-muted`}>
              {t("season.curve.hotel")}
            </th>
            {INDEX_ORDER.map((code) => (
              <th key={code} scope="col" className={`${CELL} text-right font-semibold`}>
                <span className="inline-flex items-center gap-1">
                  <span className="text-accent">{INDEX_ICON[code]}</span>
                  {indexLabel(code)}
                </span>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {hotels.map((h) => (
            <tr key={h.hotel_id} aria-current={h.mine ? "true" : undefined}>
              <HotelCell h={h} />
              {indexRows(h).map((r) => (
                <td key={r.code} className={NUM}>
                  {t("index.range", { start: formatWhole(r.start), end: formatWhole(r.end) })}{" "}
                  <Delta value={r.delta} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </TableCard>
  );
}

export function MoneyTable({ hotels }: { hotels: SeasonHotelBody[] }) {
  return (
    <TableCard id="sezona-financije" title={t("season.money")}>
      <table className="w-full border-separate border-spacing-0 text-sm">
        <caption className="sr-only">{t("season.money")}</caption>
        <thead>
          <tr>
            <th scope="col" className={`${CELL} pl-0 text-left font-semibold text-muted`}>
              {t("season.curve.hotel")}
            </th>
            <th scope="col" className={`${CELL} text-right font-semibold`}>
              {t("season.revenue")}
            </th>
            <th scope="col" className={`${CELL} text-right font-semibold`}>
              {t("season.gop")}
            </th>
            <th scope="col" className={`${CELL} text-right font-semibold`}>
              {t("season.gopShort")}
            </th>
            <th scope="col" className={`${CELL} text-right font-semibold`}>
              {t("season.cash")}
            </th>
          </tr>
        </thead>
        <tbody>
          {hotels.map((h) => {
            const rows = financeRows(h);
            return (
              <tr key={h.hotel_id} aria-current={h.mine ? "true" : undefined}>
                <HotelCell h={h} />
                {rows ? (
                  <>
                    <td className={NUM}>{rows.find((r) => r.key === "revenue")?.value}</td>
                    <td className={NUM}>{rows.find((r) => r.key === "gop")?.value}</td>
                    <td className={NUM}>{gopMargin(h) ?? "—"}</td>
                    <td className={NUM}>{rows.find((r) => r.key === "cash")?.value}</td>
                  </>
                ) : (
                  <td colSpan={4} className={`${CELL} text-muted`}>
                    {t("season.private")}
                  </td>
                )}
              </tr>
            );
          })}
        </tbody>
      </table>
    </TableCard>
  );
}
