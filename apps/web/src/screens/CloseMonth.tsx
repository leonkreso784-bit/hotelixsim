import { t } from "@hotelixsim/i18n";
import { useEffect, useRef, useState } from "react";
import { problemMessage } from "../api/errors";
import { useCloseMonth } from "../api/queries";
import type { MonthCode } from "../api/types";
import { monthName } from "../format";
import { Button, Card, ErrorText } from "../ui/primitives";

/** Gumb s potvrdom; upozorava na neodgovorene PRIJE klika (BACKLOG [dnevnik] M4). */
export function CloseMonth({
  gameId,
  month,
  unanswered,
}: {
  gameId: string;
  month: MonthCode;
  unanswered: number;
}) {
  const [asking, setAsking] = useState(false);
  const close = useCloseMonth(gameId);
  const inFlight = useRef(false);
  const name = monthName(month);
  const title = useRef<HTMLHeadingElement>(null);
  const openButton = useRef<HTMLButtonElement>(null);
  // Fokus se seli tek nakon korisnikove radnje, ne pri prvom prikazu ekrana.
  const moved = useRef(false);

  // Kliknuti gumb nestaje: fokus ide na naslov potvrde i ona se pomakne u vidno polje (iznad donje trake),
  // a nakon „Odustani“ fokus se vraća na „Zatvori mjesec“ (UI_PRAVILA §6.5).
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
    close.mutate(month, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  if (!asking)
    return (
      <Button ref={openButton} onClick={() => ask(true)}>
        {t("close.button")}
      </Button>
    );
  return (
    <Card emphasis className="space-y-3">
      <h2 ref={title} tabIndex={-1} className="text-lg font-bold">
        {t("close.confirmTitle", { month: name })}
      </h2>
      <p className="text-sm text-muted">{t("close.confirmText")}</p>
      {unanswered > 0 && <ErrorText>{t("close.unanswered", { n: unanswered })}</ErrorText>}
      {close.isError && <ErrorText>{problemMessage(close.error)}</ErrorText>}
      <Button disabled={close.isPending} onClick={confirm}>
        {t("close.confirm", { month: name })}
      </Button>
      <Button variant="secondary" disabled={close.isPending} onClick={() => ask(false)}>
        {t("close.cancel")}
      </Button>
    </Card>
  );
}
