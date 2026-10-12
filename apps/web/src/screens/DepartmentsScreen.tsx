// DepartmentsScreen — odjeli hotela (samo v2): popis s karticama i ekran jednog odjela. Od 1024 px popis lijevo i detalj
// desno, ispod toga je detalj zaseban ekran. U v1 igri poslužitelj vraća 409 `model_v1`, pa ekran pokaže rečenicu.
import { t } from "@hotelixsim/i18n";
import { ApiError, problemMessage } from "../api/errors";
import { useDepartments, useGame } from "../api/queries";
import type { HotelBody } from "../api/types";
import { tw } from "../format";
import { isWeekly } from "../game";
import { GameError } from "../ui/GameError";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { DepartmentCard } from "./departments/DepartmentCard";
import { DepartmentDetail } from "./departments/DepartmentDetail";
import { isService } from "./departments/helpers";
import { LEVEL_GOOD_FROM, LEVEL_MID_FROM } from "./departments/levels";
import { ServiceList } from "./departments/ServiceCard";

export function DepartmentsScreen({ me, code }: { me: HotelBody; code: string | null }) {
  const hotelId = me.hotel.hotel_id;
  const list = useDepartments(hotelId);
  const gameQuery = useGame(me.game_id);
  const game = gameQuery.data;
  const weekly = game ? isWeekly(game) : false;
  // Igra se nije učitala: tekstovi ovise o njoj, pa ekran nudi ponovni pokušaj umjesto mjesečnih tekstova.
  const gameFailed = gameQuery.isError && !game;
  const Heading = code === null ? "h1" : "h2";

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
        {list.isError ? (
          list.error instanceof ApiError && list.error.status === 409 ? (
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
          )
        ) : gameFailed ? (
          // Uz otvoren detalj ponovni pokušaj nudi stupac detalja, da gumb ne bude dvaput na ekranu.
          code === null && <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />
        ) : list.isPending || gameQuery.isPending ? (
          <StatusText>{t("status.loading")}</StatusText>
        ) : (
          <>
            {list.data?.round === null && <p className="text-muted">{tw(weekly, "dept.listEmpty")}</p>}
            {/* Jedan stupac: od 768 px uz bočnu traku dva stupca bila bi preuska (lomljena ocjena i rezultat, M-4). */}
            <div className="space-y-3">
              {revenue.map((d) => (
                <DepartmentCard
                  key={d.code}
                  dept={d}
                  weekly={weekly}
                  selected={d.code === code}
                  level={code === null ? 2 : 3}
                />
              ))}
              <ServiceList services={services} weekly={weekly} selected={code} />
            </div>
          </>
        )}
      </div>
      <div className={`min-w-0 ${code === null ? "hidden lg:block" : ""}`}>
        {code === null ? (
          // Desno okno od 1024 px: uz uputu i kako čitati ocjenu, da okno ne bude prazno (M-5).
          <Card className="space-y-2">
            <p className="text-muted">{t("dept.pick")}</p>
            <p className="text-sm text-muted">
              {t("dept.legend", {
                good: LEVEL_GOOD_FROM,
                mid: LEVEL_MID_FROM,
                midTo: LEVEL_GOOD_FROM - 1,
              })}
            </p>
          </Card>
        ) : gameFailed ? (
          <div className="space-y-3">
            <h1 className="sr-only">{t("dept.detail.title")}</h1>
            <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />
          </div>
        ) : (
          <DepartmentDetail
            key={code}
            hotelId={hotelId}
            code={code}
            game={game}
            gamePending={gameQuery.isPending}
            weekly={weekly}
            planState={me.private?.plan ?? null}
          />
        )}
      </div>
    </div>
  );
}
