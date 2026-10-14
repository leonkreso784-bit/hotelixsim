import { t } from "@hotelixsim/i18n";
import { useEffect, useRef, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useDelegate } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, Card, ErrorText } from "../../ui/primitives";

/**
 * „Prepusti voditelju“ (Leon P4) u dva koraka, obrazac `CloseMonth`: gumb otvara potvrdu (fokus na njezin naslov),
 * „Odustani“ vraća fokus na gumb, a potvrda šalje točno jedan zahtjev i na dvostruki klik. Odgovor (sve situacije)
 * ide u cache, pa ekran odmah pokaže „Prepušteno voditelju odjela …“. `dept` = ime odjela voditelja ili `null`.
 */
export function Delegate({
  hotelId,
  s,
  dept,
  weekly,
}: {
  hotelId: string;
  s: SituationBody;
  dept: string | null;
  weekly: boolean;
}) {
  const [asking, setAsking] = useState(false);
  const delegate = useDelegate(hotelId, s.code);
  const inFlight = useRef(false);
  const title = useRef<HTMLHeadingElement>(null);
  const openButton = useRef<HTMLButtonElement>(null);
  // Fokus se seli tek nakon korisnikove radnje, ne pri prvom prikazu ekrana.
  const moved = useRef(false);

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
    if (!next) delegate.reset();
    setAsking(next);
  }

  function confirm() {
    if (delegate.isPending || inFlight.current) return;
    inFlight.current = true;
    delegate.mutate(undefined, {
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  if (!asking)
    return (
      <Button ref={openButton} variant="secondary" onClick={() => ask(true)}>
        {t("situation.delegate.button")}
      </Button>
    );
  return (
    <Card emphasis className="space-y-3">
      <h2 ref={title} tabIndex={-1} className="text-lg font-bold">
        {t("situation.delegate.confirmTitle")}
      </h2>
      <p>
        {dept ? t("situation.delegate.confirmText", { dept }) : t("situation.delegate.confirmTextNoName")}
      </p>
      {delegate.isError && <ErrorText>{problemMessage(delegate.error, weekly)}</ErrorText>}
      <Button disabled={delegate.isPending} onClick={confirm}>
        {t("situation.delegate.button")}
      </Button>
      <Button variant="secondary" disabled={delegate.isPending} onClick={() => ask(false)}>
        {t("situation.delegate.cancel")}
      </Button>
    </Card>
  );
}

/** Prepuštena situacija: jedna rečenica, ništa za uređivanje; `focusOnMount` kad je upravo prepuštena ovdje (§6.5). */
export function DelegatedNote({ dept, focusOnMount }: { dept: string | null; focusOnMount: boolean }) {
  const note = useRef<HTMLParagraphElement>(null);
  useEffect(() => {
    if (focusOnMount) note.current?.focus();
  }, [focusOnMount]);
  return (
    <Card>
      <p ref={note} tabIndex={-1} className="font-semibold">
        {dept ? t("situation.delegated", { dept }) : t("situation.delegatedNoName")}
      </p>
    </Card>
  );
}
