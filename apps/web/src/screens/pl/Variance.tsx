// Razlika ostvarenog i budžeta: znak + iznos + riječ (UI_PRAVILA §8). „Bolje“ dolazi iz `favorable` koji računa poslužitelj
// po vrsti retka (veći prihod je bolje, veći trošak lošije); predznak iznosa je predznak razlike, ne ocjena.
import { t } from "@hotelixsim/i18n";
import type { PlLineBody } from "../../api/types";
import { formatEurosSigned } from "../../format";

export function Variance({ line }: { line: PlLineBody }) {
  // Razlika iz prikazanih (zaokruženih) eura, da se tri broja u retku uvijek zbrajaju; ispod 1 € je „jednako“.
  const v = (Math.round(line.actual_cents / 100) - Math.round(line.budget_cents / 100)) * 100;
  if (v === 0) return <span className="text-muted">{t("pl.equal")}</span>;
  const tone = line.favorable ? "text-up" : "text-down";
  const amount = formatEurosSigned(v);
  return (
    <span className={tone}>
      <span aria-hidden="true">{v > 0 ? "▲" : "▼"} </span>
      {t(line.favorable ? "pl.better" : "pl.worse", { amount })}
    </span>
  );
}
