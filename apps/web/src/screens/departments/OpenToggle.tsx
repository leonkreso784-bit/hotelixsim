// Kvačica „Otvoren“ odjela. Kad zatvaranje ne stane u korak sezonskih radnika, kvačica ostaje u redu tipkovnice
// (`aria-disabled`, kao gumbi brojača, M5) i klik je ne mijenja; razlog piše ispod.
import { t } from "@hotelixsim/i18n";
import { tw } from "../../format";
import { FOCUS } from "./helpers";

export function OpenToggle(props: {
  code: string;
  open: boolean;
  /** Zatvaranje otpušta sve sezonske radnike: stane li to u preostali korak. */
  closeFits: boolean;
  describedBy: string;
  weekly: boolean;
  onChange: (open: boolean) => void;
}) {
  const locked = props.open && !props.closeFits;
  return (
    <div>
      <label
        htmlFor={`dept-open-${props.code}`}
        className={`flex min-h-11 items-center gap-3 font-semibold ${locked ? "cursor-not-allowed" : ""}`}
      >
        <input
          id={`dept-open-${props.code}`}
          type="checkbox"
          checked={props.open}
          aria-disabled={locked}
          aria-describedby={props.describedBy}
          onChange={(e) => {
            if (!locked) props.onChange(e.target.checked);
          }}
          className={`size-5 accent-accent aria-disabled:opacity-40 ${FOCUS}`}
        />
        {t("plan.dept.open")}
      </label>
      {!props.open && <p className="text-sm text-muted">{t("plan.dept.closedNote")}</p>}
      {locked && <p className="text-sm text-muted">{tw(props.weekly, "plan.dept.noClose")}</p>}
    </div>
  );
}
