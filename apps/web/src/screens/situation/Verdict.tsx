import { t } from "@hotelixsim/i18n";
import { Check, Info, Sparkles, X } from "lucide-react";
import type { ResponseBody } from "../../api/types";
import { Card, StatusText } from "../../ui/primitives";

export function Verdict({ response }: { response: ResponseBody }) {
  return (
    <section className="space-y-3">
      <Card className="space-y-1">
        <p className="text-sm font-bold text-accent">{t("response.title")}</p>
        <p>{response.text}</p>
      </Card>
      {response.status === "pending" ? (
        <StatusText>{t("verdict.pending")}</StatusText>
      ) : (
        <Card className="space-y-3 bg-accent-soft">
          <h2 className="flex items-center gap-2 text-lg font-bold">
            <Sparkles size={18} aria-hidden="true" className="text-accent" />
            {t("verdict.title")}
          </h2>
          {response.status === "neutral" && <p className="text-sm">{t("verdict.neutral")}</p>}
          {response.quality !== null && (
            <p className="text-2xl font-extrabold">{t("verdict.total", { q: response.quality })}</p>
          )}
          {response.criteria.length > 0 && (
            <ul className="divide-y divide-line rounded-xl bg-surface">
              {response.criteria.map((c) => (
                <li key={c.n} className="flex gap-3 p-3">
                  <span className={c.met ? "text-up" : "text-down"}>
                    {c.met ? <Check size={20} aria-hidden="true" /> : <X size={20} aria-hidden="true" />}
                  </span>
                  <span className="min-w-0 space-y-0.5">
                    <span className="block font-semibold">{c.text}</span>
                    <span className={`block text-xs font-bold ${c.met ? "text-up" : "text-down"}`}>
                      {c.met ? t("verdict.met") : t("verdict.notMet")}
                    </span>
                    <span className="block text-sm text-muted">{c.reason}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}
          {response.rationale && <p className="text-sm">{response.rationale}</p>}
        </Card>
      )}
      <p className="flex items-center gap-2 text-xs text-muted">
        <Info size={14} aria-hidden="true" />
        {t("verdict.note")}
      </p>
    </section>
  );
}
