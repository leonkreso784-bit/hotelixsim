// SeasonScreen — „Kraj sezone“ (/kraj-sezone): rezultati svih hotela nakon zadnjeg kruga v2 igre. Stara (v1) igra i
// sezona u tijeku dobivaju rečenicu i povratak na hotel, ne grešku; podatke daje `GET /v1/games/{id}/season`.
import { t } from "@hotelixsim/i18n";
import { ApiError, problemMessage } from "../api/errors";
import { useGame, useSeason } from "../api/queries";
import type { HotelBody } from "../api/types";
import { isV2, isWeekly } from "../game";
import { Link } from "../router";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { SeasonView } from "./season/SeasonView";

function Notice({ text }: { text: string }) {
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("season.title")}</h1>
      <Card className="space-y-2">
        <p>{text}</p>
        <Link to="/" className="inline-flex min-h-11 items-center font-semibold text-accent-ink">
          {t("season.back")}
        </Link>
      </Card>
    </>
  );
}

export function SeasonScreen({ me }: { me: HotelBody }) {
  const game = useGame(me.game_id);
  const g = game.data;
  const ready = g !== undefined && isV2(g) && g.current === null;
  const season = useSeason(me.game_id, ready);

  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (!g)
    return (
      <div className="space-y-3">
        <ErrorText>{problemMessage(game.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void game.refetch()}>
          {t("error.retry")}
        </Button>
      </div>
    );
  const weekly = isWeekly(g);
  const notOver = t(weekly ? "season.notOver.week" : "season.notOver");
  if (!isV2(g)) return <Notice text={t("season.v1")} />;
  if (g.current !== null) return <Notice text={notOver} />;
  if (season.data) return <SeasonView body={season.data} />;
  if (season.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  const e = season.error;
  if (e instanceof ApiError && e.code === "season_not_over") return <Notice text={notOver} />;
  if (e instanceof ApiError && e.code === "model_v1") return <Notice text={t("season.v1")} />;
  return (
    <div className="space-y-3">
      <ErrorText>{problemMessage(e, weekly)}</ErrorText>
      <Button variant="secondary" onClick={() => void season.refetch()}>
        {t("error.retry")}
      </Button>
    </div>
  );
}
