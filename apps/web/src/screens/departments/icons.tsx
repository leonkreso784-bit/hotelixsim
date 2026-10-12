// Ikona odjela po kodu (UI_PRAVILA §1.5: lucide-react, 18–22 px, uz tekst, aria-hidden, u krugu kao u `Stat`).
import {
  BedDouble,
  Briefcase,
  Building2,
  Car,
  GlassWater,
  type LucideIcon,
  Martini,
  Megaphone,
  Sparkles,
  Store,
  UtensilsCrossed,
  Wine,
  Wrench,
  Zap,
} from "lucide-react";

const DEPT_ICON: Record<string, LucideIcon> = {
  rooms: BedDouble,
  restaurant: UtensilsCrossed,
  lobby_bar: Martini,
  rooftop_bar: Wine,
  pool_bar: GlassWater,
  spa: Sparkles,
  parking: Car,
  retail_lease: Store,
  admin: Briefcase,
  sales: Megaphone,
  maintenance: Wrench,
  energy: Zap,
};

/** Ikona za kod odjela; nepoznat kod (novi odjel u sadržaju) dobiva zgradu. */
export function deptIcon(code: string): LucideIcon {
  return DEPT_ICON[code] ?? Building2;
}

export function DeptIcon({ code }: { code: string }) {
  const Icon = deptIcon(code);
  return (
    <span className="inline-flex size-8 shrink-0 items-center justify-center rounded-xl bg-accent-soft text-accent">
      <Icon size={18} aria-hidden="true" data-dept-icon={code} />
    </span>
  );
}
