import { t } from "@hotelixsim/i18n";
import { ChevronRight, MessageCircle } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations } from "../api/queries";
import type { HotelBody, SituationBody } from "../api/types";
import { Avatar } from "../avatar/Avatar";
import { lookFor } from "../avatar/look";
import { tw } from "../format";
import { isWeekly } from "../game";
import { Link } from "../router";
import { Badge, Card, ErrorText, StatusText } from "../ui/primitives";

export function situationStatus(s: SituationBody): { text: string; tone: "accent" | "muted" | "up" } {
  if (s.response?.status === "pending") return { text: t("situation.status.pending"), tone: "accent" };
  if (s.response?.quality != null)
    return { text: t("situation.status.quality", { q: s.response.quality }), tone: "up" };
  if (s.chosen !== null || s.response) return { text: t("situation.status.answered"), tone: "up" };
  return { text: t("situation.status.open"), tone: "muted" };
}

export function SituationsScreen({ me }: { me: HotelBody }) {
  const sits = useSituations(me.hotel.hotel_id);
  const game = useGame(me.game_id);
  if (sits.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (sits.isError && !sits.data) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
  const seasonOver = game.data?.current === null;
  const weekly = game.data ? isWeekly(game.data) : false;
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{tw(weekly, "situations.title")}</h1>
      {/* Uvijek najviše jedna rečenica o stanju, da se ne proturječe (UI_PRAVILA §7). */}
      {seasonOver ? (
        <div>
          <p className="text-muted">{t("situations.seasonOver")}</p>
          <Link to="/tijek" className="inline-flex min-h-11 items-center font-semibold text-accent-ink">
            {t("situations.toFeed")}
          </Link>
        </div>
      ) : sits.data.items.length === 0 ? (
        <p className="text-muted">{tw(weekly, "situations.none")}</p>
      ) : (
        !sits.data.open && <p className="text-muted">{tw(weekly, "situations.closed")}</p>
      )}
      {sits.data.items.length > 0 && (
        <ul className="space-y-3">
          {sits.data.items.map((s) => {
            const st = situationStatus(s);
            return (
              <li key={s.code}>
                <Link to={`/situacije/${encodeURIComponent(s.code)}`} className="block">
                  <Card className="flex items-center gap-3">
                    {s.character && <Avatar look={lookFor(s.character.name)} size={40} />}
                    <span className="min-w-0 flex-1 space-y-1">
                      <span className="block font-bold">{s.title}</span>
                      {s.character && (
                        <span className="flex items-center gap-1 text-sm text-muted">
                          <MessageCircle size={14} aria-hidden="true" />
                          {t("situations.talk")}
                        </span>
                      )}
                      <Badge tone={st.tone}>{st.text}</Badge>
                    </span>
                    <ChevronRight size={18} aria-hidden="true" className="text-muted" />
                  </Card>
                </Link>
              </li>
            );
          })}
        </ul>
      )}
    </>
  );
}
