// Brojka 0–100 s imenom i trakom (stanje prostora, zadovoljstvo gostiju); bodovi 0–10 000 se prikazuju kao 0–100.
import { t } from "@hotelixsim/i18n";
import { formatIndex } from "../../format";

export function Meter({ label, points }: { label: string; points: number }) {
  const value = formatIndex(points);
  const pct = Math.max(0, Math.min(100, points / 100));
  return (
    <div className="space-y-1">
      <div className="flex items-baseline justify-between gap-2 text-sm">
        <span className="min-w-0">{label}</span>
        <strong className="tabular-nums">{t("dept.outOf", { n: value })}</strong>
      </div>
      <span
        role="img"
        aria-label={t("index.bar", { label, value })}
        className="relative block h-2 rounded-full bg-line"
      >
        <span className="absolute inset-y-0 left-0 rounded-full bg-accent" style={{ width: `${pct}%` }} />
      </span>
    </div>
  );
}
