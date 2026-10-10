// Mali usmjerivač nad History API-jem (ADR-022 t. 1): devet ruta, bez učitavača.
import { type AnchorHTMLAttributes, type MouseEvent, useSyncExternalStore } from "react";
import type { DriverCode, IndexCode, MonthCode } from "./api/types";

export type Route =
  | { name: "hotel" }
  | { name: "situations" }
  | { name: "situation"; code: string }
  | { name: "plan" }
  | { name: "month"; month: MonthCode }
  | { name: "indicator"; code: IndexCode; highlight: DriverCode | null }
  | { name: "kpi" }
  | { name: "feed" }
  | { name: "avatars" }
  | { name: "notFound" };

const MONTHS: readonly MonthCode[] = ["jun", "jul", "aug", "sep"];

const INDEX_CODES: readonly IndexCode[] = ["G", "R", "Z", "O", "F"];
const DRIVER_CODES: readonly DriverCode[] = [
  "sold",
  "revenue",
  "costs",
  "cash",
  "staff",
  "morale",
  "experience",
  "equipment",
  "satisfaction",
  "rating",
  "staff_load",
];

function isIndex(s: string): s is IndexCode {
  return (INDEX_CODES as readonly string[]).includes(s);
}

function isDriver(s: string): s is DriverCode {
  return (DRIVER_CODES as readonly string[]).includes(s);
}

function isMonth(s: string): s is MonthCode {
  return (MONTHS as readonly string[]).includes(s);
}

export function matchRoute(pathname: string): Route {
  const [a, b, c, ...rest] = pathname.split("/").filter(Boolean);
  if (rest.length > 0) return { name: "notFound" };
  if (a === undefined || (a === "demo" && b === undefined)) return { name: "hotel" };
  if (a === "pokazatelj") {
    if (b === undefined || !isIndex(b)) return { name: "notFound" };
    if (c === undefined) return { name: "indicator", code: b, highlight: null };
    return isDriver(c) ? { name: "indicator", code: b, highlight: c } : { name: "notFound" };
  }
  if (c !== undefined) return { name: "notFound" };
  if (a === "situacije") {
    if (b === undefined) return { name: "situations" };
    try {
      return { name: "situation", code: decodeURIComponent(b) };
    } catch {
      return { name: "notFound" };
    }
  }
  if (a === "plan" && b === undefined) return { name: "plan" };
  if (a === "pokazatelji" && b === undefined) return { name: "kpi" };
  if (a === "mjesec" && b !== undefined && isMonth(b)) return { name: "month", month: b };
  if (a === "tijek" && b === undefined) return { name: "feed" };
  if (a === "likovi" && b === undefined) return { name: "avatars" };
  return { name: "notFound" };
}

const listeners = new Set<() => void>();

function subscribe(onChange: () => void) {
  listeners.add(onChange);
  window.addEventListener("popstate", onChange);
  return () => {
    listeners.delete(onChange);
    window.removeEventListener("popstate", onChange);
  };
}

export function navigate(to: string): void {
  if (to !== window.location.pathname) window.history.pushState(null, "", to);
  for (const l of listeners) l();
  document.documentElement.scrollTop = 0;
}

export function useRoute(): Route {
  const path = useSyncExternalStore(subscribe, () => window.location.pathname);
  return matchRoute(path);
}

type LinkProps = AnchorHTMLAttributes<HTMLAnchorElement> & { to: string };

export function Link({ to, onClick, ...rest }: LinkProps) {
  function handle(e: MouseEvent<HTMLAnchorElement>) {
    onClick?.(e);
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    navigate(to);
  }
  return <a href={to} onClick={handle} {...rest} />;
}
