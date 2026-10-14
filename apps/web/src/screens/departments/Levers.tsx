// Dvije odluke odjela s razinama: razina cijena (koraci od 5 %, u granicama poslužitelja) i održavanje (tri razine s
// cijenom kruga; uz odgođeno rečenica da prostor propada brže). Obje su onemogućene u zatvorenom odjelu.
import { t } from "@hotelixsim/i18n";
import type { PlanBody, PlanLimitsBody } from "../../api/types";
import { formatPercentWhole, formatRange, tw } from "../../format";
import { Choice } from "../plan/Choice";
import { maintenanceOptions } from "../plan/costs";
import { Stepper } from "./Stepper";

const PRICE_STEP = 5;

export function PriceLevel(props: {
  pct: number;
  min: number;
  max: number;
  open: boolean;
  /** Najam s cijenom (parking): opis bez zadovoljstva gostiju odjela. */
  lease: boolean;
  describedBy: string;
  onChange: (pct: number) => void;
}) {
  return (
    <div className="space-y-1">
      <Stepper
        label={t("plan.dept.price")}
        value={formatPercentWhole(props.pct)}
        moreLabel={t("plan.dept.morePrice")}
        lessLabel={t("plan.dept.lessPrice")}
        describedBy={props.describedBy}
        moreDisabled={!props.open || props.pct + PRICE_STEP > props.max}
        lessDisabled={!props.open || props.pct - PRICE_STEP < props.min}
        onMore={() => props.onChange(props.pct + PRICE_STEP)}
        onLess={() => props.onChange(props.pct - PRICE_STEP)}
      />
      <p className="text-xs text-muted">
        {t("plan.range", { range: formatRange(props.min, props.max, "percent") })}
      </p>
      <p className="text-sm text-muted">
        {t(props.lease ? "plan.dept.priceNote.lease" : "plan.dept.priceNote")}
      </p>
    </div>
  );
}

export function Upkeep(props: {
  code: string;
  value: PlanBody["maintenance"];
  levels: PlanLimitsBody["costs"]["maintenance"];
  open: boolean;
  weekly: boolean;
  describedBy: string;
  onChange: (v: PlanBody["maintenance"]) => void;
}) {
  return (
    <div className="space-y-1">
      <Choice
        legend={t("plan.dept.maintenance")}
        name={`dept-maintenance-${props.code}`}
        value={props.value}
        disabled={!props.open}
        describedBy={props.describedBy}
        options={maintenanceOptions(props.levels)}
        onChange={props.onChange}
      />
      <p className="text-sm text-muted">
        {tw(props.weekly, "plan.cost.levels")} {t("plan.maintenance.note")}
      </p>
      {props.value === "deferred" && <p className="text-sm font-semibold">{t("dept.decide.deferredNote")}</p>}
    </div>
  );
}
