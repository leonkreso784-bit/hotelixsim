import { t } from "@hotelixsim/i18n";
import { useEffect, useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useCloseRound } from "../api/queries";
import type { RoundInfoBody } from "../api/types";
import { roundDrafts } from "../drafts";
import { roundPhrase, tw } from "../format";
import { isWeekRound } from "../game";
import { Link } from "../router";
import { Button, Card, ErrorText } from "../ui/primitives";
import { FOCUS } from "./departments/helpers";

/**
 * Gumb s potvrdom za zatvaranje kruga (mjeseca ili tjedna); upozorava na neodgovorene PRIJE klika (BACKLOG [dnevnik] M4)
 * i na nespremljene izmjene plana ili odluka odjela (N-I2): krug se obračunava sa spremljenim planom, pa potvrda to kaže
 * i vodi u plan (sažetak ondje pokazuje i odjele s nespremljenim izmjenama). Potvrda ne sprema sama: odluke odjela imaju
 * svoja pravila (korak sezonskih radnika, najveći broj), pa bi skupno spremanje moglo pasti napola.
 * `delegable`: među neodgovorenima je bar jedna koja se smije prepustiti voditelju (W-1), pa upozorenje to nudi.
 */
export function CloseMonth({
  gameId,
  hotelId,
  round,
  unanswered,
  delegable = false,
}: {
  gameId: string;
  hotelId: string;
  round: RoundInfoBody;
  unanswered: number;
  delegable?: boolean;
}) {
  const [asking, setAsking] = useState(false);
  const close = useCloseRound(gameId);
  const inFlight = useRef(false);
  const name = roundPhrase(round);
  const weekly = isWeekRound(round);
  const title = useRef<HTMLHeadingElement>(null);
  const openButton = useRef<HTMLButtonElement>(null);
  // Fokus se seli tek nakon korisnikove radnje, ne pri prvom prikazu ekrana.
  const moved = useRef(false);

  // Kliknuti gumb nestaje: fokus ide na naslov potvrde i ona se pomakne u vidno polje (iznad donje trake),
  // a nakon „Odustani“ fokus se vraća na gumb za zatvaranje (UI_PRAVILA §6.5).
  useEffect(() => {
    if (!moved.current) return;
    if (asking) {
      title.current?.focus();
      title.current?.scrollIntoView?.({ block: "center" });
    } else {
      openButton.current?.focus();
    }
  }, [asking]);

  function ask(next: boolean) {
    moved.current = true;
    setAsking(next);
  }

  function confirm() {
    if (close.isPending || inFlight.current) return;
    inFlight.current = true;
    close.mutate(round.code, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  if (!asking)
    return (
      <Button ref={openButton} onClick={() => ask(true)}>
        {t(weekly ? "close.week" : "close.button")}
      </Button>
    );
  const drafts = roundDrafts(hotelId, round.code);
  const unsaved = drafts.plan || drafts.depts.length > 0;
  return (
    <Card emphasis className="space-y-3">
      <h2 ref={title} tabIndex={-1} className="text-lg font-bold">
        {t("close.confirmTitle", { month: name })}
      </h2>
      <p className="text-sm text-muted">{tw(weekly, "close.confirmText")}</p>
      {unanswered > 0 && (
        <ErrorText>
          {delegable
            ? `${t("close.unanswered", { n: unanswered })} ${t("close.delegateHint")}`
            : t("close.unanswered", { n: unanswered })}
        </ErrorText>
      )}
      {unsaved && (
        <div className="space-y-1">
          <ErrorText>{tw(weekly, "close.unsaved")}</ErrorText>
          <Link
            to="/plan"
            className={`inline-flex min-h-11 items-center font-semibold text-accent-ink underline ${FOCUS}`}
          >
            {t("close.openPlan")}
          </Link>
        </div>
      )}
      {close.isError && <ErrorText>{problemMessage(close.error, weekly)}</ErrorText>}
      <Button disabled={close.isPending} onClick={confirm}>
        {t("close.confirm", { month: name })}
      </Button>
      <Button variant="secondary" disabled={close.isPending} onClick={() => ask(false)}>
        {t("close.cancel")}
      </Button>
    </Card>
  );
}
