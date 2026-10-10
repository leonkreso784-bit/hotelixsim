// screens/StartScreen.tsx — gost bez igre (DEMO §1.4, ADR-022 t. 3)
import { t } from "@hotelixsim/i18n";
import { useRef } from "react";
import { problemMessage } from "../api/errors";
import { useStartDemo } from "../api/queries";
import { Button, ErrorText, StatusText } from "../ui/primitives";

export function StartScreen() {
  const start = useStartDemo();
  const inFlight = useRef(false);

  function begin() {
    if (start.isPending || inFlight.current) return;
    inFlight.current = true;
    start.mutate(undefined, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col justify-center gap-6 px-6 py-10">
      <img src="/logo-znak.png" alt="" className="size-16 rounded-2xl" />
      <h1 className="text-4xl leading-tight font-extrabold tracking-tight">
        {t("start.title1")} <span className="text-accent">{t("start.title2")}</span>
      </h1>
      <p className="text-lg text-muted">{t("start.lead")}</p>
      <p className="font-semibold">{t("start.measure")}</p>
      {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
      {start.isPending && <StatusText>{t("start.busy")}</StatusText>}
      <Button disabled={start.isPending} onClick={begin}>
        {t("start.button")}
      </Button>
    </main>
  );
}
