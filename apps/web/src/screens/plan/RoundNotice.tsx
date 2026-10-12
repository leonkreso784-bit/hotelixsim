// Rečenica „Otvoren je novi tjedan…“ (plan i odluke odjela). Kad se pojavi nakon radnje (spremanje u stari krug, krug
// otvoren dok je ekran otvoren), fokus ide na nju i ona se pomakne u vidno polje: gumb za spremanje je daleko ispod
// (UI_PRAVILA §6.5, M-7). Pri prvom prikazu ekrana fokus se ne seli.
import { useEffect, useRef } from "react";
import { tw } from "../../format";

export function RoundNotice({ show, weekly }: { show: boolean; weekly: boolean }) {
  const ref = useRef<HTMLParagraphElement>(null);
  const was = useRef(show);
  useEffect(() => {
    if (show && !was.current) {
      ref.current?.focus();
      ref.current?.scrollIntoView?.({ block: "center" });
    }
    was.current = show;
  }, [show]);
  if (!show) return null;
  return (
    <p ref={ref} role="status" tabIndex={-1} className="py-2 font-semibold">
      {tw(weekly, "plan.newRound")}
    </p>
  );
}
