import { t } from "@hotelixsim/i18n";
import { type FormEvent, useRef, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useConversation, useSendMessage } from "../../api/queries";
import { Avatar } from "../../avatar/Avatar";
import { lookFor, PLAYER_LOOK } from "../../avatar/look";
import { Button, ErrorText, StatusText } from "../../ui/primitives";

/** Razgovor s likom (do 6 poruka). Prva rečenica lika je ujedno opis situacije. */
export function Conversation({
  hotelId,
  code,
  canWrite,
}: {
  hotelId: string;
  code: string;
  canWrite: boolean;
}) {
  const conv = useConversation(hotelId, code, true);
  const send = useSendMessage(hotelId, code);
  const inFlight = useRef(false);
  const [text, setText] = useState("");
  if (conv.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (conv.isError) return <ErrorText>{problemMessage(conv.error)}</ErrorText>;
  const c = conv.data;
  const used = c.lines.filter((l) => l.from === "student").length;
  const total = used + c.remaining;
  const look = lookFor(c.character_name);

  function submit(e: FormEvent) {
    e.preventDefault();
    const msg = text.trim();
    if (!msg || send.isPending || inFlight.current) return;
    inFlight.current = true;
    send.mutate(msg, {
      onSuccess: () => setText(""),
      onSettled: () => {
        inFlight.current = false;
      },
    });
  }

  return (
    <section className="space-y-3">
      <div className="flex items-center gap-3">
        <Avatar look={look} size={56} />
        <div>
          <p className="font-bold">{c.character_name}</p>
          <p className="text-sm text-muted">{c.role}</p>
        </div>
      </div>
      <ul className="space-y-2">
        <li className="flex items-end gap-2">
          <Avatar look={look} size={36} />
          <p className="rounded-2xl rounded-bl-sm bg-surface px-3 py-2 shadow-sm">{c.opening}</p>
        </li>
        {c.lines.map((l, i) =>
          l.from === "student" ? (
            // biome-ignore lint/suspicious/noArrayIndexKey: retci razgovora nemaju id, poredak se ne mijenja
            <li key={i} className="flex items-end justify-end gap-2">
              <p className="rounded-2xl rounded-br-sm bg-accent-soft px-3 py-2">{l.text}</p>
              <Avatar look={PLAYER_LOOK} size={36} />
            </li>
          ) : (
            // biome-ignore lint/suspicious/noArrayIndexKey: retci razgovora nemaju id, poredak se ne mijenja
            <li key={i} className="flex items-end gap-2">
              <Avatar look={look} size={36} />
              <p className="rounded-2xl rounded-bl-sm bg-surface px-3 py-2 shadow-sm">{l.text}</p>
            </li>
          ),
        )}
      </ul>
      {c.awaiting_reply && <StatusText>{t("chat.typing", { name: c.character_name })}</StatusText>}
      <div>
        <p className="text-sm text-muted">{t("chat.progress", { used, total })}</p>
        <div className="mt-1 flex gap-1" aria-hidden="true">
          {Array.from({ length: total }, (_, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: crtice brojača
            <span key={i} className={`h-1.5 flex-1 rounded-full ${i < used ? "bg-accent" : "bg-line"}`} />
          ))}
        </div>
      </div>
      {send.isError && <ErrorText>{problemMessage(send.error)}</ErrorText>}
      {canWrite && c.open && c.remaining === 0 && <p className="text-sm font-semibold">{t("chat.limit")}</p>}
      {canWrite && c.open && c.remaining > 0 && (
        <form onSubmit={submit} className="flex gap-2">
          <input
            aria-label={t("chat.label")}
            placeholder={t("chat.placeholder")}
            maxLength={2000}
            value={text}
            onChange={(e) => setText(e.target.value)}
            className="min-h-12 min-w-0 flex-1 rounded-2xl border border-line bg-surface px-3"
          />
          <Button
            type="submit"
            className="w-auto"
            disabled={send.isPending || c.awaiting_reply || !text.trim()}
          >
            {t("chat.send")}
          </Button>
        </form>
      )}
    </section>
  );
}
