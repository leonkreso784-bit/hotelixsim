import { t } from "@hotelixsim/i18n";
import { ChartNoAxesColumn, House, MessageCircle } from "lucide-react";
import type { ReactNode } from "react";
import type { GameBody } from "../api/types";
import { monthName } from "../format";
import { monthPosition } from "../game";
import { Link, type Route, useRoute } from "../router";

export function chipText(game: GameBody): string {
  const pos = monthPosition(game);
  if (!pos) return t("chip.over");
  return t("chip.position", {
    month: monthName(pos.month).toLocaleUpperCase("hr-HR"),
    n: pos.index + 1,
    total: pos.total,
  });
}

function Logo() {
  return (
    <Link to="/" className="flex items-center gap-2">
      <img src="/logo-znak.png" alt="" className="size-9 rounded-xl" />
      <span className="text-xl font-extrabold tracking-tight">
        Hotelix<span className="text-accent">Sim</span>
      </span>
    </Link>
  );
}

const TABS = [
  { to: "/", label: "nav.hotel", icon: House, match: ["hotel", "plan", "month"] },
  { to: "/situacije", label: "nav.situations", icon: MessageCircle, match: ["situations", "situation"] },
  { to: "/tijek", label: "nav.feed", icon: ChartNoAxesColumn, match: ["feed"] },
] as const;

function BottomNav({ route }: { route: Route }) {
  return (
    <nav
      aria-label={t("nav.label")}
      className="fixed inset-x-0 bottom-0 border-t border-line bg-surface pb-[env(safe-area-inset-bottom)]"
    >
      <ul className="mx-auto grid max-w-md grid-cols-3">
        {TABS.map((tab) => {
          const active = (tab.match as readonly string[]).includes(route.name);
          const Icon = tab.icon;
          return (
            <li key={tab.to}>
              <Link
                to={tab.to}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-14 flex-col items-center justify-center gap-0.5 text-xs font-semibold ${
                  active ? "text-accent" : "text-muted"
                }`}
              >
                <Icon size={22} aria-hidden="true" />
                {t(tab.label)}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

export function Layout({ chip, children }: { chip: string | null; children: ReactNode }) {
  const route = useRoute();
  return (
    <div className="mx-auto min-h-dvh max-w-md">
      <header className="flex items-center justify-between gap-2 px-4 pt-4 pb-2">
        <Logo />
        {chip && (
          <span className="rounded-full bg-accent px-3 py-1 text-xs font-bold tracking-wide text-white">
            {chip}
          </span>
        )}
      </header>
      <main className="space-y-3 px-4 pb-28">{children}</main>
      <BottomNav route={route} />
    </div>
  );
}
