import { t } from "@hotelixsim/i18n";
import { useRef } from "react";
import { problemMessage } from "../../api/errors";
import { useAnswer } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, Card, ErrorText } from "../../ui/primitives";

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

  function choose(i: number) {
    if (answer.isPending || inFlight.current) return;
    inFlight.current = true;
    answer.mutate(i, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  return (
    <section className="space-y-3">
      <h2 className="text-lg font-bold">{heading}</h2>
      {answer.isError && <ErrorText>{problemMessage(answer.error)}</ErrorText>}
      <ol className="space-y-2">
        {s.answers.map((label, i) => (
          <li key={label}>
            <Card className="space-y-2 p-3">
              <p>{label}</p>
              <Button variant="secondary" disabled={answer.isPending} onClick={() => choose(i)}>
                {t("answers.choose")}
              </Button>
            </Card>
          </li>
        ))}
      </ol>
    </section>
  );
}

export function ChosenAnswer({ s }: { s: SituationBody }) {
  if (s.chosen === null) return null;
  return (
    <Card className="space-y-1 border-accent">
      <p className="text-sm font-bold text-accent">{t("answers.chosen")}</p>
      <p>{s.answers[s.chosen]}</p>
    </Card>
  );
}
