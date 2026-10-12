// Sažetak odluka po odjelima u planu kruga (samo v2): što se mijenja u kojem odjelu, s poveznicom na odjel gdje se
// odluke donose. Plan kruga sam ne mijenja odjele.
import { t } from "@hotelixsim/i18n";
import { ChevronRight } from "lucide-react";
import { problemMessage } from "../../api/errors";
import { useDepartments } from "../../api/queries";
import type { PlanBody } from "../../api/types";
import { roundDrafts } from "../../drafts";
import { tw } from "../../format";
import { Link } from "../../router";
import { Button, Card, ErrorText, StatusText } from "../../ui/primitives";
import { FOCUS } from "../departments/helpers";
import { DeptIcon } from "../departments/icons";
import { deptChanges } from "./summary";

const ROW = `flex min-h-12 items-center gap-3 rounded-xl py-2 ${FOCUS}`;

export function DepartmentSummary({
  hotelId,
  round,
  plans,
  weekly,
}: {
  hotelId: string;
  round: string;
  plans: NonNullable<PlanBody["departments"]>;
  weekly: boolean;
}) {
  const list = useDepartments(hotelId);
  const heading = <h2 className="text-lg font-bold">{t("plan.dept.title")}</h2>;
  if (list.isPending)
    return (
      <Card className="space-y-2">
        {heading}
        <StatusText>{t("status.loading")}</StatusText>
      </Card>
    );
  if (list.isError)
    return (
      <Card className="space-y-2">
        {heading}
        <ErrorText>{problemMessage(list.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void list.refetch()}>
          {t("error.retry")}
        </Button>
      </Card>
    );
  const changes = deptChanges(plans, list.data.departments, roundDrafts(hotelId, round).depts);
  return (
    <Card className="space-y-2">
      {heading}
      <p className="text-sm text-muted">{t("plan.dept.intro")}</p>
      {changes.length === 0 && <p>{tw(weekly, "plan.dept.none")}</p>}
      <ul className="divide-y divide-line">
        {changes.map((c) => (
          <li key={c.code}>
            <Link to={`/odjeli/${c.code}`} className={ROW}>
              <DeptIcon code={c.code} />
              <span className="min-w-0 flex-1">
                <span className="font-semibold">{c.name}:</span> {c.text}
              </span>
              <ChevronRight size={18} aria-hidden="true" className="shrink-0 text-muted" />
            </Link>
          </li>
        ))}
        <li>
          <Link to="/odjeli" className={`${ROW} font-semibold text-accent-ink`}>
            <span className="min-w-0 flex-1">{t("plan.dept.all")}</span>
            <ChevronRight size={18} aria-hidden="true" className="shrink-0 text-muted" />
          </Link>
        </li>
      </ul>
    </Card>
  );
}
