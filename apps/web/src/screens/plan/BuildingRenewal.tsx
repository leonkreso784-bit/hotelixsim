// Zgrada u planu kruga (v2 s obnovom; zamjenjuje staru kvačicu obnove): stanje zgrade i koliko se troši, izbor iznosa
// obnove zgrade kao u odjelu i smjernica ulaganja u sezoni (pričuva za obnovu, FF&E).
import { t } from "@hotelixsim/i18n";
import type { BuildingBody, RenewalGuideBody, RenewalLimitsBody } from "../../api/types";
import { formatCount, formatEuros, formatPercentBp } from "../../format";
import { Card } from "../../ui/primitives";
import { Meter } from "../departments/Meter";
import { RenewalPicker } from "../renewal/RenewalPicker";

const TITLE_ID = "building-title";

export function BuildingRenewal(props: {
  limits: RenewalLimitsBody;
  building: BuildingBody | null;
  guide: RenewalGuideBody | null;
  amount: number;
  cashCents: number;
  payNowTotal: number;
  payrollWeekCents: number;
  weekly: boolean;
  onChange: (amount: number) => void;
}) {
  const g = props.guide;
  const wear = props.building?.wear_pts_per_month ?? 0;
  return (
    <Card className="space-y-3">
      <h2 id={TITLE_ID} className="text-lg font-bold">
        {t("building.title")}
      </h2>
      <div className="space-y-1">
        <Meter label={t("building.condition")} points={props.limits.condition_pts} />
        {wear > 0 && (
          <p className="text-sm text-muted">
            {t("building.wear", { pts: formatCount(wear / 100, "point") })}
          </p>
        )}
      </div>
      <div className="md:max-w-md">
        <RenewalPicker
          limits={props.limits}
          amount={props.amount}
          subject="building"
          cashCents={props.cashCents}
          payNowTotal={props.payNowTotal}
          payrollWeekCents={props.payrollWeekCents}
          weekly={props.weekly}
          describedBy={TITLE_ID}
          onChange={props.onChange}
        />
      </div>
      {g && (
        <div className="space-y-1">
          <p className="text-sm font-semibold">
            {t("building.guide", {
              pct: formatPercentBp(g.reserve_bp),
              amount: formatEuros(g.guide_cents),
              invested: formatEuros(g.invested_cents),
            })}
          </p>
          <p className="text-sm text-muted">{t("building.guide.mean")}</p>
        </div>
      )}
    </Card>
  );
}
