// PlScreen — račun dobiti po USALI-ju (samo v2): sekcije redom, ostvareno, budžet i razlika; za jedan krug ili sezonu do
// njega. Na računalu tablica, na mobitelu kartice. U v1 igri i prije prvog obračuna poslužitelj vraća 409 → rečenica.
import { t } from "@hotelixsim/i18n";
import { useState } from "react";
import { ApiError, problemMessage } from "../api/errors";
import { useGame, usePl } from "../api/queries";
import type { GameBody, HotelBody, PlBody, PlScopeCode } from "../api/types";
import { keepRoundTogether, roundDates, roundTitle, tw } from "../format";
import { isWeekly, roundInfo } from "../game";
import { navigate } from "../router";
import { GameError } from "../ui/GameError";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { type PickerRound, RoundPicker, shortWeek, weekGroup } from "../ui/RoundPicker";
import { PlCards } from "./pl/PlCards";
import { PlRatios } from "./pl/PlRatios";
import { PlTable } from "./pl/PlTable";

const FOCUS = "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent";

function titleOf(game: GameBody | undefined, code: string): string {
  const info = game ? roundInfo(game, code) : null;
  return info ? roundTitle(info) : code;
}

/** Naslov ekrana; broj tjedna i datumi u zagradi vezani nerazdvojnim razmacima da se naslov ne lomi usred njih. */
function heading(game: GameBody | undefined, pl: PlBody): string {
  const info = game ? roundInfo(game, pl.round) : null;
  const text =
    pl.scope === "season" && info
      ? info.kind === "week"
        ? keepRoundTogether(t("pl.titleSeason.week", { n: info.index + 1, dates: roundDates(info) }))
        : t("pl.titleSeason", { round: titleOf(game, pl.round) })
      : t("pl.titleRound", { round: titleOf(game, pl.round) });
  return text;
}

function ScopeToggle({
  scope,
  weekly,
  onPick,
}: {
  scope: PlScopeCode;
  weekly: boolean;
  onPick: (s: PlScopeCode) => void;
}) {
  return (
    <fieldset className="flex min-w-0 flex-wrap gap-2">
      <legend className="sr-only">{t("pl.scope")}</legend>
      {(["round", "season"] as const).map((s) => (
        <button
          key={s}
          type="button"
          aria-pressed={s === scope}
          onClick={() => onPick(s)}
          className={`min-h-11 rounded-2xl px-4 font-semibold ${FOCUS} ${s === scope ? "bg-accent text-white" : "bg-accent-soft text-accent-ink"}`}
        >
          {s === "round" ? tw(weekly, "pl.scope.round") : t("pl.scope.season")}
        </button>
      ))}
    </fieldset>
  );
}

export function PlScreen({ me, code }: { me: HotelBody; code: string | null }) {
  const [scope, setScope] = useState<PlScopeCode>("round");
  const q = usePl(me.hotel.hotel_id, code, scope);
  const gameQuery = useGame(me.game_id);
  const game = gameQuery.data;
  const pl = q.data;
  const weekly = game ? isWeekly(game) : false;
  const title = (
    <h1 className="text-3xl font-extrabold tracking-tight">{pl ? heading(game, pl) : t("pl.title")}</h1>
  );
  const intro = (
    <div className="max-w-2xl space-y-1">
      {title}
      <p className="text-muted">{t("pl.intro")}</p>
      <p className="text-muted">{t("pl.budget")}</p>
    </div>
  );

  // Naslov se pokazuje tek s podacima (ili s rečenicom o grešci): naslov kruga ovisi o odgovoru poslužitelja.
  // Igra se čeka kao i odgovor: bez nje ne zna se ni je li igra tjedna (tekstovi, birač) ni kako se krug zove.
  if (q.isPending || gameQuery.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (!game) return <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />;
  if (!pl) {
    // 409 (stara igra, još nema obračunatog kruga) i 404 su očekivani ishodi: rečenica, ne kvar.
    const expected = q.error instanceof ApiError && q.error.status < 500;
    return (
      <div className="space-y-3">
        {title}
        {expected ? (
          <Card>
            <p>{problemMessage(q.error, weekly)}</p>
          </Card>
        ) : (
          <>
            <ErrorText>{problemMessage(q.error, weekly)}</ErrorText>
            <Button variant="secondary" onClick={() => void q.refetch()}>
              {t("error.retry")}
            </Button>
          </>
        )}
      </div>
    );
  }

  const rounds: PickerRound[] = pl.rounds.map((c) => {
    const info = game ? roundInfo(game, c) : null;
    return {
      code: c,
      title: titleOf(game, c),
      short: info ? shortWeek(info.index + 1) : c,
      ...(info && info.kind === "week" ? { group: weekGroup(info) } : {}),
    };
  });
  return (
    <div className="space-y-3" aria-busy={q.isPlaceholderData}>
      {intro}
      <ScopeToggle scope={scope} weekly={weekly} onPick={setScope} />
      {q.isPlaceholderData && <StatusText>{t("pl.loading")}</StatusText>}
      {rounds.length > 1 && (
        <RoundPicker
          rounds={rounds}
          value={pl.round}
          onPick={(c) => navigate(`/racun-dobiti/${c}`)}
          many={weekly}
          legend={tw(weekly, "pl.pickRound")}
          prevLabel={t("round.prev.week")}
          nextLabel={t("round.next.week")}
        />
      )}
      {pl.scope === "season" && (
        <p role="status" className="text-muted">
          {tw(weekly, "pl.scope.seasonNote", { round: titleOf(game, pl.round) })}
        </p>
      )}
      <PlTable sections={pl.sections} />
      <PlCards sections={pl.sections} />
      <PlRatios pl={pl} />
    </div>
  );
}
