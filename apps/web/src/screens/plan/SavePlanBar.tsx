// Traka s jedinim gumbom „Spremi plan“ (ADR-042 t. 4): stoji na Planu i na stranici odjela, a sprema plan hotela i
// odluke svih odjela odjednom. Poruke (greška, spremljeno, nespremljeno) stoje uz gumb. aria-disabled, ne disabled:
// fokus tipkovnice ostaje na gumbu dok traje spremanje i dok nema izmjena (M-6).
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../../api/errors";
import { formatCount } from "../../format";
import { Button, ErrorText, StatusText } from "../../ui/primitives";
import { roundChanged, type usePlanSave } from "./usePlanSave";

export function SavePlanBar({
  saver,
  round,
  seasonalMax,
  weekly,
  only,
  submit = false,
}: {
  saver: ReturnType<typeof usePlanSave>;
  /** Naziv kruga za potvrdu („Plan za … je spremljen.“). */
  round: string;
  seasonalMax: number;
  weekly: boolean;
  /** Stranica odjela: napomena „nespremljeno“ govori samo o ovom odjelu (gumb ipak sprema sve). */
  only?: string;
  /** Gumb je u obrascu (Plan): obrazac šalje, pa gumb nema svoj klik. */
  submit?: boolean;
}) {
  const { save, problem } = saver;
  return (
    <div className="space-y-3">
      {problem === "invalid" && <ErrorText>{t("plan.invalid")}</ErrorText>}
      {problem === "tooMany" && (
        <ErrorText>{t("plan.dept.tooMany", { n: formatCount(seasonalMax, "seasonalWorker") })}</ErrorText>
      )}
      {/* Novi krug javlja rečenica iznad odluka (RoundNotice), ne greška. */}
      {problem === "error" && !roundChanged(save.error) && (
        <ErrorText>{problemMessage(save.error, weekly)}</ErrorText>
      )}
      {saver.done && <StatusText>{t("plan.saved", { month: round })}</StatusText>}
      {(only ? saver.dirtyDepts.includes(only) : saver.dirty) && (
        <p className="text-sm font-semibold">{t("plan.unsaved")}</p>
      )}
      <Button
        type={submit ? "submit" : "button"}
        aria-disabled={!saver.enabled}
        onClick={submit ? undefined : saver.run}
      >
        {t("plan.save")}
      </Button>
    </div>
  );
}
