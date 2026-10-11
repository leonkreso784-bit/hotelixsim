// Uvodna animacija logotipa živi u `index.html` (crta se prije nego se aplikacija učita).
// Ovdje je samo izlaz: kad je aplikacija spremna, uvod pričeka da se vrata otvore i izblijedi.

export const SPLASH_ID = "hx-splash";
/** Ključ u sessionStorage: uvod se u istoj kartici prikazuje jednom (skripta u `index.html` ga čita). */
export const SPLASH_SEEN_KEY = "hx-uvod";
/** Vrata su otvorena u 1,62 s + 0,62 s; ranije ne prekidamo. */
export const SPLASH_MIN_MS = 2300;
/** Trajanje blijeđenja (`.hx-splash.is-leaving` u `index.html`). */
export const SPLASH_FADE_MS = 320;

export interface SplashEnv {
  doc: Document;
  /** Koliko je prošlo od početka učitavanja stranice. */
  elapsedMs: number;
  reducedMotion: boolean;
  storage: Pick<Storage, "setItem"> | null;
  schedule: (fn: () => void, ms: number) => void;
}

export function dismissSplash(env: SplashEnv): void {
  const splash = env.doc.getElementById(SPLASH_ID);
  if (!splash) return;
  try {
    env.storage?.setItem(SPLASH_SEEN_KEY, "1");
  } catch {
    // Pohrana blokirana (privatni prozor): uvod će se samo prikazati i idući put.
  }
  const wait = env.reducedMotion ? 0 : Math.max(0, SPLASH_MIN_MS - env.elapsedMs);
  env.schedule(() => {
    splash.classList.add("is-leaving");
    env.schedule(() => splash.remove(), env.reducedMotion ? 0 : SPLASH_FADE_MS);
  }, wait);
}

export function dismissSplashInBrowser(): void {
  let storage: Storage | null = null;
  try {
    storage = window.sessionStorage;
  } catch {
    storage = null;
  }
  dismissSplash({
    doc: document,
    elapsedMs: performance.now(),
    reducedMotion: window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    storage,
    schedule: (fn, ms) => window.setTimeout(fn, ms),
  });
}
