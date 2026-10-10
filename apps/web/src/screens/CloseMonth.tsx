import { t } from "@hotelixsim/i18n";
import { useRef, useState } from "react";
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

  function confirm() {
    if (close.isPending || inFlight.current) return;
    inFlight.current = true;
    close.mutate(month, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  if (!asking) return <Button onClick={() => setAsking(true)}>{t("close.button")}</Button>;
  return (
    <Card className="space-y-3 border-accent">
      <p className="text-lg font-bold">{t("close.confirmTitle", { month: name })}</p>
      <p className="text-sm text-muted">{t("close.confirmText")}</p>
      {unanswered > 0 && <ErrorText>{t("close.unanswered", { n: unanswered })}</ErrorText>}
      {close.isError && <ErrorText>{problemMessage(close.error)}</ErrorText>}
      <Button disabled={close.isPending} onClick={confirm}>
        {t("close.confirm", { month: name })}
      </Button>
      <Button variant="secondary" disabled={close.isPending} onClick={() => setAsking(false)}>
        {t("close.cancel")}
      </Button>
    </Card>
  );
}
