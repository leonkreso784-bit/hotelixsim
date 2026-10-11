// Vrh ekrana odjela: ocjena odjela i tri sastavnice kao trake (sastavnica koje odjel nema se ne crta).
import { t } from "@hotelixsim/i18n";
import { Card } from "../../ui/primitives";
import { Meter } from "./Meter";
import { type DeptScore, scoreLevel } from "./score";

export function ScoreCard({ score }: { score: DeptScore }) {
  return (
    <Card className="space-y-3">
      <Meter label={t("dept.score")} points={score.points} level={scoreLevel(score.points)} strong />
      <p className="text-sm text-muted">{t("dept.score.note")}</p>
      <div className="space-y-2">
        {score.quality !== null && <Meter label={t("dept.quality")} points={score.quality} />}
        <Meter label={t("dept.condition")} points={score.condition} />
        {score.satisfaction !== null && <Meter label={t("dept.satisfaction")} points={score.satisfaction} />}
      </div>
    </Card>
  );
}
