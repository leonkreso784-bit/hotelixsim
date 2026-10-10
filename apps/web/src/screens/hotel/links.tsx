// Poveznice Mog hotela: kartica s brojkom (Stat) i redak popisa (Row); obje vode na detalj.
import { ChevronRight } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "../../router";
import { Card } from "../../ui/primitives";

export function Stat({
  to,
  icon,
  label,
  value,
  note,
}: {
  to: string;
  icon: ReactNode;
  label: string;
  value: string;
  note?: string;
}) {
  return (
    <Link
      to={to}
      className="block rounded-card focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <Card className="space-y-1 p-3">
        <span className="flex items-center gap-2">
          <span className="grid size-8 shrink-0 place-items-center rounded-xl bg-accent-soft text-accent">
            {icon}
          </span>
          <span className="min-w-0 text-xs text-muted">{label}</span>
        </span>
        <strong className="block whitespace-nowrap text-xl tabular-nums">{value}</strong>
        {note && <span className="line-clamp-2 block text-xs text-muted">{note}</span>}
      </Card>
    </Link>
  );
}

export function Row({
  to,
  icon,
  label,
  end,
}: {
  to: string;
  icon: ReactNode;
  label: string;
  end?: ReactNode | undefined;
}) {
  return (
    <Link
      to={to}
      className="flex min-h-12 items-center gap-3 py-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <span className="text-accent">{icon}</span>
      <span className="flex-1 font-semibold">{label}</span>
      {end}
      <ChevronRight size={18} aria-hidden="true" className="text-muted" />
    </Link>
  );
}
