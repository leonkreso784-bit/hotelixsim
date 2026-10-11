// ui/GameError.tsx — igra se nije učitala: rečenica o grešci i „Pokušaj ponovno“, da ekran koji čeka igru nikad ne ostane
// bez izlaza.
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { Button, ErrorText } from "./primitives";

export function GameError({ error, onRetry }: { error: unknown; onRetry: () => void }) {
  return (
    <div className="space-y-3">
      <ErrorText>{problemMessage(error)}</ErrorText>
      <Button variant="secondary" onClick={onRetry}>
        {t("error.retry")}
      </Button>
    </div>
  );
}
