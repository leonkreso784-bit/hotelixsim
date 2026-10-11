// Pokazatelji ispod računa: GOP marža, GOPPAR i EBITDA marža, ostvareno i budžet, svaki s rečenicom što znači.
import { t } from "@hotelixsim/i18n";
import type { PlBody } from "../../api/types";
import { formatEurosCents, formatPercentBp } from "../../format";
import { Card } from "../../ui/primitives";

export function PlRatios({ pl }: { pl: PlBody }) {
  const rows = [
    {
      key: "gopMargin",
      actual: formatPercentBp(pl.gop_margin_bp.actual),
      budget: formatPercentBp(pl.gop_margin_bp.budget),
    },
    {
      key: "goppar",
      actual: formatEurosCents(pl.goppar_cents.actual),
      budget: formatEurosCents(pl.goppar_cents.budget),
    },
    {
      key: "ebitdaMargin",
      actual: formatPercentBp(pl.ebitda_margin_bp.actual),
      budget: formatPercentBp(pl.ebitda_margin_bp.budget),
    },
  ] as const;
  return (
    <Card className="space-y-1">
      {/* Namjerno <p>, ne h2: test i čitanje ekrana drže točno sedam naslova sekcija računa. */}
      <p className="text-lg font-bold">{t("pl.ratios")}</p>
      <ul className="divide-y divide-line">
        {rows.map((r) => (
          <li key={r.key} className="space-y-1 py-2">
            <p className="font-semibold">{t(`pl.ratio.${r.key}` as const)}</p>
            <p className="text-sm text-muted">{t(`pl.ratio.${r.key}.mean` as const)}</p>
            <p className="flex flex-wrap gap-x-4 tabular-nums">
              <span className="whitespace-nowrap">
                {t("pl.ratios.actual")} <strong>{r.actual}</strong>
              </span>
              <span className="whitespace-nowrap">
                {t("pl.ratios.budget")} <strong>{r.budget}</strong>
              </span>
            </p>
          </li>
        ))}
      </ul>
    </Card>
  );
}
