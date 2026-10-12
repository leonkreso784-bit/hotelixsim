// Računalo (od 1024 px): indeksi početak → kraj, financije sezone i odluke u brojkama svih hotela u tablicama. Isti
// retci kao kartice hotela ispod 1024 px (`rows.ts`), pa se brojke ne razilaze; ista odluka svih hotela stoji u istom
// retku, da se moja odluka može usporediti s robotovom (Leon 13.10.).
import { indexLabel, t } from "@hotelixsim/i18n";
import type { ReactNode } from "react";
import type { SeasonBody, SeasonHotelBody } from "../../api/types";
import { formatWhole } from "../../format";
import { INDEX_ORDER } from "../../game";
import { Delta } from "../../ui/IndexBars";
import { Card } from "../../ui/primitives";
import { type DecisionKey, decisionRows, financeRows, gopMargin, hotelLabel, indexRows } from "./rows";

const CELL = "border-b border-line px-3 py-2";
// Tablica indeksa ima šest stupaca: uži razmak, da na 1024 px stane bez pomicanja.
const TIGHT = "border-b border-line px-2 py-2";
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
        className="relative overflow-x-auto focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
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
              <th key={code} scope="col" className={`${TIGHT} text-right font-semibold`}>
                {indexLabel(code)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {hotels.map((h) => (
            <tr key={h.hotel_id} aria-current={h.mine ? "true" : undefined}>
              <HotelCell h={h} />
              {indexRows(h).map((r) => (
                <td key={r.code} className={`${TIGHT} whitespace-nowrap text-right tabular-nums`}>
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

// Retci odluka koji su rečenica („100–105 % uobičajenih“, krug obnove, situacije s ocjenom) smiju se prelomiti; ostali
// su broj s jedinicom i ostaju u jednom retku (UI_PRAVILA §4.5), a tablica se pomiče unutar svoje regije.
const PROSE_ROWS: ReadonlySet<DecisionKey> = new Set(["wage", "renovation", "situations"]);

/** Odluke u brojkama: retci = odluke, stupci = hoteli redom poretka (moj hotel podebljan). */
export function DecisionTable({ body }: { body: SeasonBody }) {
  const columns = body.hotels.map((h) => ({ h, rows: decisionRows(h, body) }));
  const keys = columns.find((c) => c.rows)?.rows ?? [];
  if (keys.length === 0) return null;
  const hidden = columns.some((c) => c.rows === null);
  return (
    <TableCard id="sezona-odluke" title={t("season.decisions")}>
      <table className="w-full border-separate border-spacing-0 text-sm">
        <caption className="sr-only">{t("season.decisions")}</caption>
        <thead>
          <tr>
            <th scope="col" className={`${CELL} pl-0 text-left font-semibold text-muted`}>
              {t("season.decision")}
            </th>
            {columns.map(({ h }) => (
              <th
                key={h.hotel_id}
                scope="col"
                className={`${CELL} text-right align-bottom ${h.mine ? "font-bold" : "font-semibold"}`}
              >
                {h.rank}. {hotelLabel(h)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {keys.map((k) => (
            <tr key={k.key}>
              <th scope="row" className={`${CELL} pl-0 text-left font-normal`}>
                {k.label}
              </th>
              {columns.map(({ h, rows }) => (
                <td
                  key={h.hotel_id}
                  className={`${CELL} text-right tabular-nums ${PROSE_ROWS.has(k.key) ? "" : "whitespace-nowrap"} ${h.mine ? "font-bold" : ""}`}
                >
                  {rows?.find((r) => r.key === k.key)?.value ?? "—"}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {hidden && <p className="text-sm text-muted">{t("season.private")}</p>}
    </TableCard>
  );
}
