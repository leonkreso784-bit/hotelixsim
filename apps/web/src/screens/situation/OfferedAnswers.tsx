import { t } from "@hotelixsim/i18n";
import { useEffect, useRef, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useAnswer } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, Card, ErrorText } from "../../ui/primitives";

/** Izbor u dva koraka (UI_PRAVILA §6.9): klik označi odgovor, tek „Potvrdi izbor“ ga šalje — izbor je nepovratan. */
export function OfferedAnswers({
  hotelId,
  s,
  heading,
}: {
  hotelId: string;
  s: SituationBody;
  heading: string;
}) {
  const answer = useAnswer(hotelId, s.code);
  const inFlight = useRef(false);
  const [picked, setPicked] = useState<number | null>(null);
  const confirmTitle = useRef<HTMLHeadingElement>(null);
  const pickButtons = useRef<(HTMLButtonElement | null)[]>([]);
  const textId = (i: number) => `odgovor-${s.code}-${i}`;

  // Potvrda se pojavi ispod popisa: fokus i pogled idu na nju, da tipkovnica i mobitel ne ostanu iznad.
  useEffect(() => {
    if (picked === null) return;
    confirmTitle.current?.focus();
    confirmTitle.current?.scrollIntoView?.({ block: "center" });
  }, [picked]);

  function cancel() {
    const back = picked;
    setPicked(null);
    answer.reset();
    if (back !== null) pickButtons.current[back]?.focus();
  }

  function confirm() {
    if (picked === null || answer.isPending || inFlight.current) return;
    inFlight.current = true;
    answer.mutate(picked, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  return (
    <section className="space-y-3">
      <h2 className="text-lg font-bold">{heading}</h2>
      <ol className="space-y-2">
        {s.answers.map((label, i) => (
          <li key={label}>
            <Card pad="sm" emphasis={picked === i} className="space-y-2">
              <p id={textId(i)}>{label}</p>
              <Button
                ref={(el) => {
                  pickButtons.current[i] = el;
                }}
                variant="secondary"
                aria-describedby={textId(i)}
                aria-pressed={picked === i}
                disabled={answer.isPending}
                onClick={() => setPicked(i)}
              >
                {picked === i ? t("answers.picked") : t("answers.choose")}
              </Button>
            </Card>
          </li>
        ))}
      </ol>
      {picked !== null && (
        <Card emphasis className="space-y-3">
          <h3 ref={confirmTitle} tabIndex={-1} className="text-lg font-bold">
            {t("answers.confirmTitle")}
          </h3>
          <p>{s.answers[picked]}</p>
          <p className="text-sm text-muted">{t("answers.confirmText")}</p>
          {answer.isError && <ErrorText>{problemMessage(answer.error)}</ErrorText>}
          <Button disabled={answer.isPending} onClick={confirm}>
            {t("answers.confirm")}
          </Button>
          <Button variant="secondary" disabled={answer.isPending} onClick={cancel}>
            {t("answers.cancel")}
          </Button>
        </Card>
      )}
    </section>
  );
}

/** `focusOnMount`: izbor je upravo potvrđen na ovom ekranu, pa fokus ide na novi sadržaj (UI_PRAVILA §6.5). */
export function ChosenAnswer({ s, focusOnMount = false }: { s: SituationBody; focusOnMount?: boolean }) {
  const label = useRef<HTMLParagraphElement>(null);
  const chosen = s.chosen;
  useEffect(() => {
    if (focusOnMount && chosen !== null) label.current?.focus();
  }, [focusOnMount, chosen]);
  if (s.chosen === null) return null;
  return (
    <Card emphasis className="space-y-1">
      <p ref={label} tabIndex={-1} className="text-sm font-bold text-accent-ink">
        {t("answers.chosen")}
      </p>
      <p>{s.answers[s.chosen]}</p>
    </Card>
  );
}
