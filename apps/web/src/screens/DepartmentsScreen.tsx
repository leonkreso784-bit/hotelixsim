// DepartmentsScreen — odjeli hotela (samo v2): popis s karticama i ekran jednog odjela. Od 1024 px popis lijevo i detalj
// desno, ispod toga je detalj zaseban ekran. U v1 igri poslužitelj vraća 409 `model_v1`, pa ekran pokaže rečenicu.
import { t } from "@hotelixsim/i18n";
import { ApiError, problemMessage } from "../api/errors";
import { useDepartments, useGame } from "../api/queries";
import type { HotelBody } from "../api/types";
import { isWeekly } from "../game";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { DepartmentCard } from "./departments/DepartmentCard";
import { DepartmentDetail } from "./departments/DepartmentDetail";
import { isService } from "./departments/helpers";
import { ServiceList } from "./departments/ServiceCard";

export function DepartmentsScreen({ me, code }: { me: HotelBody; code: string | null }) {
  const hotelId = me.hotel.hotel_id;
  const list = useDepartments(hotelId);
  const gameQuery = useGame(me.game_id);
  const game = gameQuery.data;
  const weekly = game ? isWeekly(game) : false;
  const Heading = code === null ? "h1" : "h2";

  if (list.isError)
    return (
      <div className="mx-auto max-w-2xl space-y-3">
        <h1 className="text-3xl font-extrabold tracking-tight">{t("dept.title")}</h1>
        {list.error instanceof ApiError && list.error.status === 409 ? (
          <Card>
            <p>{problemMessage(list.error)}</p>
          </Card>
        ) : (
          <>
            <ErrorText>{problemMessage(list.error)}</ErrorText>
            <Button variant="secondary" onClick={() => void list.refetch()}>
              {t("error.retry")}
            </Button>
          </>
        )}
      </div>
    );

  const departments = list.data?.departments ?? [];
  const revenue = departments.filter((d) => !isService(d.class));
  const services = departments.filter((d) => isService(d.class));
  return (
    <div
      data-layout="list-detail"
      className="space-y-3 lg:grid lg:grid-cols-[22rem_1fr] lg:items-start lg:gap-6 lg:space-y-0"
    >
      <div className={`min-w-0 space-y-3 ${code === null ? "" : "hidden lg:block"}`}>
        <div>
          <Heading className="text-3xl font-extrabold tracking-tight">{t("dept.title")}</Heading>
          <p className="text-muted">{t("dept.intro")}</p>
        </div>
        {list.isPending ? (
          <StatusText>{t("status.loading")}</StatusText>
        ) : (
          <>
            <div className="space-y-3">
              {revenue.map((d) => (
                <DepartmentCard key={d.code} dept={d} weekly={weekly} selected={d.code === code} />
              ))}
            </div>
            <ServiceList services={services} weekly={weekly} selected={code} />
          </>
        )}
      </div>
      <div className={`min-w-0 ${code === null ? "hidden lg:block" : ""}`}>
        {code === null ? (
          <Card>
            <p className="text-muted">{t("dept.pick")}</p>
          </Card>
        ) : (
          <DepartmentDetail key={code} hotelId={hotelId} code={code} game={game} weekly={weekly} />
        )}
      </div>
    </div>
  );
}
