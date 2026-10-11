// Ekran jednog odjela: povratak, naslov, kartice Prostor/Gosti/Novac/Ljudi (ili trošak službe) i povijest po krugovima.
import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { ApiError, problemMessage } from "../../api/errors";
import { useDepartment } from "../../api/queries";
import type { GameBody } from "../../api/types";
import { roundTitle, tw } from "../../format";
import { roundInfo } from "../../game";
import { Link } from "../../router";
import { Badge, Button, Card, ErrorText, StatusText } from "../../ui/primitives";
import { DepartmentFacts, People } from "./DepartmentFacts";
import { DepartmentHistory } from "./DepartmentHistory";
import { FOCUS, isService } from "./helpers";
import { ServiceCosts } from "./ServiceCard";

export function DepartmentDetail({
  hotelId,
  code,
  game,
  weekly,
}: {
  hotelId: string;
  code: string;
  game: GameBody | undefined;
  weekly: boolean;
}) {
  const q = useDepartment(hotelId, code);
  const titleOf = (round: string) => {
    const info = game ? roundInfo(game, round) : null;
    return info ? roundTitle(info) : t("dept.history.fallback");
  };
  if (q.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (q.isError) {
    // Odjel kojeg hotel nema je očekivan ishod (rečenica), ne kvar.
    if (q.error instanceof ApiError && q.error.status === 404)
      return (
        <Card>
          <p>{problemMessage(q.error)}</p>
        </Card>
      );
    return (
      <div className="space-y-3">
        <ErrorText>{problemMessage(q.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void q.refetch()}>
          {t("error.retry")}
        </Button>
      </div>
    );
  }
  const { department: dept, history } = q.data;
  const service = isService(dept.class);
  const empty = service ? dept.service_last === null : dept.last === null;
  return (
    <div className="space-y-3">
      <Link
        to="/odjeli"
        className={`inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink lg:hidden ${FOCUS}`}
      >
        <ChevronLeft size={18} aria-hidden="true" />
        {t("dept.back")}
      </Link>
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-3xl font-extrabold tracking-tight">{dept.name}</h1>
        {!dept.open && <Badge tone="muted">{t("dept.closed")}</Badge>}
      </div>
      {empty && (
        <Card>
          <p>{tw(weekly, "dept.noResult")}</p>
        </Card>
      )}
      {dept.service_last && <ServiceCosts cost={dept.service_last} />}
      {!service && dept.last && <DepartmentFacts dept={dept} r={dept.last} />}
      {(service || empty) && <People dept={dept} r={service ? null : dept.last} />}
      <DepartmentHistory history={history} titleOf={titleOf} />
    </div>
  );
}
