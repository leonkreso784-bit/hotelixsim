// Brojka 0–100 s imenom i trakom (stanje prostora, zadovoljstvo gostiju, ocjena odjela); bodovi 0–10 000 se
// prikazuju kao 0–100. Uz razinu (`level`) traka dobiva boju razine, a broj i opis i riječ („72/100 · dobro“):
// boja nikad ne stoji sama (UI_PRAVILA §6.8). Bez razine (sastavnice ocjene) traka je neutralna, da se ocjena ističe.
import { t } from "@hotelixsim/i18n";
import { formatIndex } from "../../format";
import { SCORE_LEVELS, type ScoreLevel } from "./score";

export function Meter({
  label,
  points,
  level,
  strong = false,
}: {
  label: string;
  points: number;
  level?: ScoreLevel;
  /** Istaknuta ocjena (vrh ekrana odjela): veća brojka. */
  strong?: boolean;
}) {
  const value = formatIndex(points);
  const pct = Math.max(0, Math.min(100, points / 100));
  const look = level ? SCORE_LEVELS[level] : null;
  const shown = look ? t("dept.score.value", { n: value, level: look.word }) : t("dept.outOf", { n: value });
  const aria = look
    ? t("dept.score.bar", { label, value, level: look.word })
    : t("index.bar", { label, value });
  return (
    <div className="space-y-1">
      <div className={`flex items-baseline justify-between gap-2 ${strong ? "" : "text-sm"}`}>
        <span className={`min-w-0 ${strong ? "font-semibold" : ""}`}>{label}</span>
        <strong className={`whitespace-nowrap tabular-nums ${strong ? "text-xl font-extrabold" : ""}`}>
          {shown}
        </strong>
      </div>
      {/* Sastavnica (bez razine) je tanja traka, da ocjena ostane glavna (M-9). */}
      <span
        role="img"
        aria-label={aria}
        className={`relative block rounded-full bg-line ${look ? "h-2" : "h-1.5"}`}
      >
        <span
          className={`absolute inset-y-0 left-0 rounded-full ${look ? look.bar : "bg-muted"}`}
          style={{ width: `${pct}%` }}
        />
      </span>
    </div>
  );
}
