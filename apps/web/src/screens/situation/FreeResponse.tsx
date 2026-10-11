import { t } from "@hotelixsim/i18n";
import { type FormEvent, useRef, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useSubmitResponse } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, ErrorText } from "../../ui/primitives";

export function FreeResponse({ hotelId, s, weekly }: { hotelId: string; s: SituationBody; weekly: boolean }) {
  const submit = useSubmitResponse(hotelId, s.code);
  const inFlight = useRef(false);
  const [text, setText] = useState("");
  function onSubmit(e: FormEvent) {
    e.preventDefault();
    const answer = text.trim();
    if (!answer || submit.isPending || inFlight.current) return;
    inFlight.current = true;
    submit.mutate(answer, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }
  return (
    <form onSubmit={onSubmit} className="space-y-2">
      <h2 className="text-lg font-bold">{t("response.title")}</h2>
      <textarea
        aria-label={t("response.label")}
        placeholder={t("response.placeholder")}
        maxLength={2000}
        rows={5}
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="w-full rounded-2xl border border-line bg-surface p-3"
      />
      <div className="flex justify-between text-xs text-muted">
        <span>{t("response.hint")}</span>
        <span>{t("response.count", { n: text.length })}</span>
      </div>
      {submit.isError && <ErrorText>{problemMessage(submit.error, weekly)}</ErrorText>}
      <Button type="submit" disabled={submit.isPending || !text.trim()}>
        {t("response.submit")}
      </Button>
    </form>
  );
}
