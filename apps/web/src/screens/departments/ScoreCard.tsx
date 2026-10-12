// Vrh ekrana odjela: ocjena odjela u boji razine i njezine sastavnice kao neutralne trake (sastavnica koje odjel nema
// se ne crta, a napomena nabraja samo nacrtane).
import { t } from "@hotelixsim/i18n";
import { Card } from "../../ui/primitives";
import { Meter } from "./Meter";
import { type DeptScore, scoreLevel, scoreParts } from "./score";

export function ScoreCard({ score }: { score: DeptScore }) {
  return (
    <Card className="space-y-3">
      <Meter label={t("dept.score")} points={score.points} level={scoreLevel(score.points)} strong />
      <p className="text-sm text-muted">{scoreParts(score)}</p>
      {/* Jedna sastavnica (najam: samo prostor) bila bi ista brojka dvaput; napomena je dovoljna (M-10). */}
      {(score.satisfaction !== null || score.staffFit !== null) && (
        <div className="space-y-2">
          <Meter label={t("dept.condition")} points={score.condition} />
          {score.satisfaction !== null && (
            <Meter label={t("dept.satisfaction")} points={score.satisfaction} />
          )}
          {score.staffFit !== null && <Meter label={t("dept.staffFit")} points={score.staffFit} />}
        </div>
      )}
      {score.staffFit !== null && <p className="text-sm text-muted">{t("dept.score.fitNote")}</p>}
    </Card>
  );
}
