// Pokazatelji ispod računa: GOP marža, GOPPAR, EBITDA marža i (v2) EBIT marža, ostvareno i budžet, svaki s rečenicom
// što znači.
import { t } from "@hotelixsim/i18n";
import type { PlBody } from "../../api/types";
import { formatEurosCents, formatPercentBpFixed } from "../../format";
import { Card } from "../../ui/primitives";

export function PlRatios({ pl }: { pl: PlBody }) {
  const ebit = pl.ebit_margin_bp;
  const rows: {
    key: "gopMargin" | "goppar" | "ebitdaMargin" | "ebitMargin";
    actual: string;
    budget: string;
  }[] = [
    {
      key: "gopMargin",
      actual: formatPercentBpFixed(pl.gop_margin_bp.actual),
      budget: formatPercentBpFixed(pl.gop_margin_bp.budget),
    },
    {
      key: "goppar",
      actual: formatEurosCents(pl.goppar_cents.actual),
      budget: formatEurosCents(pl.goppar_cents.budget),
    },
    {
      key: "ebitdaMargin",
      actual: formatPercentBpFixed(pl.ebitda_margin_bp.actual),
      budget: formatPercentBpFixed(pl.ebitda_margin_bp.budget),
    },
    ...(ebit
      ? [
          {
            key: "ebitMargin" as const,
            actual: formatPercentBpFixed(ebit.actual),
            budget: formatPercentBpFixed(ebit.budget),
          },
        ]
      : []),
  ];
  return (
    <Card className="space-y-1">
      <h2 className="text-lg font-bold">{t("pl.ratios")}</h2>
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
