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
    <Link to="/" className="flex min-h-11 items-center gap-2">
      <img src="/logo-znak.png" alt="" className="size-9 rounded-xl" />
      <span className="text-xl font-extrabold tracking-tight">
        Hotelix<span className="text-accent">Sim</span>
      </span>
    </Link>
  );
}

const TABS = [
  { to: "/", label: "nav.hotel", icon: House, match: ["hotel", "plan", "month", "indicator"] },
  { to: "/situacije", label: "nav.situations", icon: MessageCircle, match: ["situations", "situation"] },
  { to: "/tijek", label: "nav.feed", icon: ChartNoAxesColumn, match: ["feed"] },
] as const;

function BottomNav({ route }: { route: Route }) {
  return (
    <nav
      aria-label={t("nav.label")}
      className="fixed inset-x-0 bottom-0 z-10 border-t border-line bg-surface pb-[env(safe-area-inset-bottom)] md:inset-x-auto md:top-28 md:bottom-auto md:left-0 md:w-60 md:border-t-0 md:bg-transparent md:px-3 md:pb-0"
    >
      <ul className="mx-auto grid max-w-md grid-cols-3 md:block md:max-w-none md:space-y-1">
        {TABS.map((tab) => {
          const active = (tab.match as readonly string[]).includes(route.name);
          const Icon = tab.icon;
          return (
            <li key={tab.to}>
              <Link
                to={tab.to}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-14 flex-col items-center justify-center gap-0.5 text-xs font-semibold md:min-h-12 md:flex-row md:justify-start md:gap-3 md:rounded-2xl md:px-4 md:text-base ${
                  active ? "text-accent md:bg-accent-soft" : "text-muted md:hover:bg-sand"
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
    <div className="mx-auto min-h-dvh max-w-md md:ml-60 md:max-w-none">
      {/* Podloga bočne trake (samo od md); izbornik i zaglavlje su iznad nje. */}
      <div
        aria-hidden="true"
        className="fixed inset-y-0 left-0 hidden w-60 border-r border-line bg-surface md:block"
      />
      <header className="flex items-center justify-between gap-2 px-4 pt-4 pb-2 md:fixed md:top-0 md:left-0 md:z-10 md:w-60 md:flex-col md:items-start md:gap-3 md:px-5 md:pt-5">
        <Logo />
        {chip && (
          <span className="rounded-full bg-accent px-3 py-1 text-xs font-bold tracking-wide text-white">
            {chip}
          </span>
        )}
      </header>
      <main className="space-y-3 px-4 pb-28 md:mx-auto md:max-w-5xl md:px-8 md:pt-6 md:pb-10">
        {children}
      </main>
      <BottomNav route={route} />
    </div>
  );
}
