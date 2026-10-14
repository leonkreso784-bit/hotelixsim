// Je li prozor barem zadane širine (isti prag kao Tailwind `md:` 768 px, `lg:` 1024 px). Bez `matchMedia` (testovi)
// vrijedi uži prikaz, kao mobitel.
import { useCallback, useSyncExternalStore } from "react";

function mediaQuery(px: number): MediaQueryList | null {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return null;
  return window.matchMedia(`(min-width: ${px}px)`);
}

export function useMinWidth(px: number): boolean {
  const subscribe = useCallback(
    (onChange: () => void) => {
      const mq = mediaQuery(px);
      mq?.addEventListener("change", onChange);
      return () => mq?.removeEventListener("change", onChange);
    },
    [px],
  );
  return useSyncExternalStore(
    subscribe,
    () => mediaQuery(px)?.matches ?? false,
    () => false,
  );
}
