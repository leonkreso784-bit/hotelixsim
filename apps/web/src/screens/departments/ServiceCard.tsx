// Službe bez prihoda: popis s troškom zadnjeg kruga, a na ekranu službe trošak po stavkama.
import { t } from "@hotelixsim/i18n";
import { ChevronRight } from "lucide-react";
import type { DepartmentBody, ServiceRoundBody } from "../../api/types";
import { formatEuros, tw } from "../../format";
import { Link } from "../../router";
import { Card } from "../../ui/primitives";
import { FOCUS } from "./helpers";
import { DeptIcon } from "./icons";

export function ServiceList({
  services,
  weekly,
  selected,
  className = "",
}: {
  services: DepartmentBody[];
  weekly: boolean;
  selected: string | null;
  className?: string;
}) {
  if (services.length === 0) return null;
  return (
    <Card className={`space-y-2 ${className}`}>
      <h2 className="text-lg font-bold">{t("dept.services")}</h2>
      <p className="text-sm text-muted">{t("dept.services.intro")}</p>
      <ul className="divide-y divide-line">
        {services.map((s) => (
          <li key={s.code}>
            <Link
              to={`/odjeli/${s.code}`}
              aria-current={s.code === selected ? "page" : undefined}
              className={`flex min-h-12 items-center gap-3 rounded-xl py-2 ${s.code === selected ? "bg-accent-soft px-2" : ""} ${FOCUS}`}
            >
              <DeptIcon code={s.code} />
              <span className="min-w-0 flex-1 font-semibold">{s.name}</span>
              <span className="text-right text-sm">
                <span className="block text-muted">{tw(weekly, "dept.cost")}</span>
                <strong className="whitespace-nowrap tabular-nums">
                  {s.service_last ? formatEuros(s.service_last.total_cents) : t("dept.costNone")}
                </strong>
              </span>
              <ChevronRight size={18} aria-hidden="true" className="shrink-0 text-muted" />
            </Link>
          </li>
        ))}
      </ul>
    </Card>
  );
}

/** Trošak službe zadnjeg kruga po stavkama; stavke bez iznosa se ne ispisuju. */
export function ServiceCosts({ cost }: { cost: ServiceRoundBody }) {
  const rows: [string, number][] = [
    [t("dept.cost.payroll"), cost.payroll_cents],
    [t("dept.cost.fixed"), cost.fixed_cents],
    [t("dept.cost.perRoom"), cost.per_room_cents],
    [t("dept.cost.marketing"), cost.marketing_cents],
    [t("dept.cost.reply"), cost.reply_cents],
    [t("dept.cost.maintenance"), cost.maintenance_cents],
    [t("dept.cost.training"), cost.training_cents],
    [t("dept.cost.hiring"), cost.hiring_cents],
    [t("dept.cost.situation"), cost.situation_cents],
  ];
  return (
    <Card className="space-y-2">
      <h2 className="text-lg font-bold">{t("dept.cost.title")}</h2>
      <dl className="divide-y divide-line">
        {rows
          .filter(([, cents]) => cents !== 0)
          .map(([label, cents]) => (
            <div key={label} className="flex items-start justify-between gap-3 py-2">
              <dt className="min-w-0">{label}</dt>
              <dd className="whitespace-nowrap font-bold tabular-nums">{formatEuros(cents)}</dd>
            </div>
          ))}
        <div className="flex items-start justify-between gap-3 py-2">
          <dt className="min-w-0 font-bold">{t("dept.cost.total")}</dt>
          <dd className="whitespace-nowrap font-extrabold tabular-nums">{formatEuros(cost.total_cents)}</dd>
        </div>
      </dl>
    </Card>
  );
}
