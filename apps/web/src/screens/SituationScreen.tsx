import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations } from "../api/queries";
import type { HotelBody } from "../api/types";
import { tw } from "../format";
import { isWeekly } from "../game";
import { Link } from "../router";
import { GameError } from "../ui/GameError";
import { Badge, ErrorText, StatusText } from "../ui/primitives";
import { Conversation } from "./situation/Conversation";
import { Delegate, DelegatedNote } from "./situation/Delegate";
import { useDeptNames } from "./situation/deptNames";
import { FreeResponse } from "./situation/FreeResponse";
import { ChosenAnswer, OfferedAnswers } from "./situation/OfferedAnswers";
import { canDelegate, isSettled, showMandatory } from "./situation/rules";
import { Verdict } from "./situation/Verdict";

export function SituationScreen({ me, code }: { me: HotelBody; code: string }) {
  const hotelId = me.hotel.hotel_id;
  const sits = useSituations(hotelId);
  const gameQuery = useGame(me.game_id);
  const gameData = gameQuery.data;
  const weekly = gameData ? isWeekly(gameData) : false;
  // Je li forma za odgovor bila prikazana: tek tada novi sadržaj (izbor, presuda, prepušteno) prima fokus.
  const answeredHere = useRef(false);
  // Otvorena potvrda (izbor odgovora ili prepuštanje): dok je jedna otvorena, druga se ne može otvoriti.
  const [confirming, setConfirming] = useState<"answer" | "delegate" | null>(null);
  const current = sits.data?.items.find((i) => i.code === code);
  const formShown = sits.data !== undefined && current !== undefined && sits.data.open && !isSettled(current);
  // Ime odjela voditelja (za „Prepusti voditelju“) stiže iz odjela; stara igra bez voditelja ih ne traži.
  const deptName = useDeptNames(hotelId, (current?.leader ?? null) !== null);
  useEffect(() => {
    if (formShown) answeredHere.current = true;
  }, [formShown]);
  const back = (
    <Link to="/situacije" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent-ink">
      <ChevronLeft size={18} aria-hidden="true" />
      {t("situation.back")}
    </Link>
  );
  if (sits.isPending || gameQuery.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (!gameData) return <GameError error={gameQuery.error} onRetry={() => void gameQuery.refetch()} />;
  if (sits.isError && !sits.data) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
  const s = sits.data.items.find((i) => i.code === code);
  if (!s)
    return (
      <>
        {back}
        <p>{tw(weekly, "situation.missing")}</p>
      </>
    );
  const open = sits.data.open;
  const decided = isSettled(s);
  const leaderName = deptName(s.leader);
  const answerArea = (
    <div className="space-y-3">
      {s.delegated && <DelegatedNote dept={leaderName} focusOnMount={answeredHere.current} />}
      {s.response && <Verdict response={s.response} focusOnMount={answeredHere.current} />}
      {open && !decided && s.free_text && <FreeResponse hotelId={hotelId} s={s} weekly={weekly} />}
      <ChosenAnswer s={s} focusOnMount={answeredHere.current} />
      {open && !decided && (
        <OfferedAnswers
          hotelId={hotelId}
          s={s}
          weekly={weekly}
          heading={s.free_text ? t("answers.orTitle") : t("answers.title")}
          locked={confirming === "delegate"}
          onConfirmChange={(o) => setConfirming(o ? "answer" : null)}
        />
      )}
      {canDelegate(s, open) && confirming !== "answer" && (
        <Delegate
          hotelId={hotelId}
          s={s}
          dept={leaderName}
          weekly={weekly}
          onAskChange={(o) => setConfirming(o ? "delegate" : null)}
        />
      )}
      {!open && !decided && <p className="font-semibold">{tw(weekly, "answers.missed")}</p>}
    </div>
  );
  // Naslov, oznaka obvezne (uz rečenicu dok se još odgovara) i opis, ako postoji i nije isti kao naslov.
  const title = (
    <>
      <h1 className="text-3xl leading-tight font-extrabold tracking-tight">{s.title}</h1>
      {showMandatory(s) && (
        <div className="space-y-1">
          <Badge>{t("situation.mandatory")}</Badge>
          {open && !decided && <p className="text-sm text-muted">{t("situation.mandatoryNote")}</p>}
        </div>
      )}
      {s.description && s.description !== s.title && <p>{s.description}</p>}
    </>
  );
  // Bez lika je ekran jedan stupac za čitanje: povratak, naslov i odgovori u istom omotaču (UI_PRAVILA §4.3).
  if (!s.character)
    return (
      <div data-layout="reading" className="mx-auto max-w-2xl space-y-3">
        {back}
        {title}
        {answerArea}
      </div>
    );
  return (
    <>
      {back}
      {title}
      <div className="space-y-3 lg:grid lg:grid-cols-2 lg:items-start lg:gap-6 lg:space-y-0">
        <div>
          <Conversation hotelId={hotelId} code={s.code} canWrite={open && !decided} weekly={weekly} />
        </div>
        {answerArea}
      </div>
    </>
  );
}
