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
  gamePending,
  weekly,
}: {
  hotelId: string;
  code: string;
  game: GameBody | undefined;
  /** Igra još nije stigla: tekstovi ovise o tome je li igra tjedna, pa se čeka kao i odjel. */
  gamePending: boolean;
  weekly: boolean;
}) {
  const q = useDepartment(hotelId, code);
  const titleOf = (round: string) => {
    const info = game ? roundInfo(game, round) : null;
    return info ? roundTitle(info) : t("dept.history.fallback");
  };
  const back = (
    <Link
      to="/odjeli"
      className={`inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink lg:hidden ${FOCUS}`}
    >
      <ChevronLeft size={18} aria-hidden="true" />
      {t("dept.back")}
    </Link>
  );
  // Povratak i naslov prve razine stoje iznad svakog stanja (učitavanje, greška, nepoznat odjel).
  if (q.isPending || gamePending)
    return (
      <div className="space-y-3">
        {back}
        <h1 className="sr-only">{t("dept.detail.title")}</h1>
        <StatusText>{t("status.loading")}</StatusText>
      </div>
    );
  if (q.isError) {
    // Odjel kojeg hotel nema je očekivan ishod (rečenica), ne kvar.
    const unknown = q.error instanceof ApiError && q.error.status === 404;
    return (
      <div className="space-y-3">
        {back}
        <h1 className="sr-only">{t("dept.detail.title")}</h1>
        {unknown ? (
          <Card>
            <p>{problemMessage(q.error)}</p>
          </Card>
        ) : (
          <>
            <ErrorText>{problemMessage(q.error)}</ErrorText>
            <Button variant="secondary" onClick={() => void q.refetch()}>
              {t("error.retry")}
            </Button>
          </>
        )}
      </div>
    );
  }
  const { department: dept, history } = q.data;
  const service = isService(dept.class);
  const empty = service ? dept.service_last === null : dept.last === null;
  return (
    <div className="space-y-3">
      {back}
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
      {!service && dept.last && <DepartmentFacts dept={dept} r={dept.last} weekly={weekly} />}
      {(service || empty) && (
        <People
          dept={dept}
          r={service ? null : dept.last}
          departures={service ? (dept.service_last?.departures ?? null) : (dept.last?.departures ?? null)}
          weekly={weekly}
        />
      )}
      <DepartmentHistory history={history} titleOf={titleOf} weekly={weekly} />
    </div>
  );
}
