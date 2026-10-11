// Uvodna animacija logotipa živi u `index.html` (crta se dok se aplikacija učitava).
// Ovdje je samo izlaz: kad je aplikacija spremna, uvod pričeka da se priča završi i izblijedi.
// Skripta u `index.html` upiše trenutak prvog crtanja uvoda (`data-start`), stavi `inert` na `#root` dok je uvod
// vidljiv i drži sigurnosni rok (9 s od prvog crtanja) ako se aplikacija nikad ne učita — tada uvod makne sama,
// zapamti ga kao viđen i, ako je `#root` još prazan, ostavi kratku poruku.

export const SPLASH_ID = "hx-splash";
/** Ključ u sessionStorage: uvod se u istoj kartici prikazuje jednom (skripta u `index.html` ga čita, a upiše ga i
 * sigurnosni rok kad sam makne uvod). */
export const SPLASH_SEEN_KEY = "hx-uvod";
/**
 * Koliko dugo se uvod vidi prije izlaza, od prvog crtanja: zadnja faza (svjetlo pred vratima) završi u 2,48 s,
 * a ime je potpuno vidljivo od 2,45 s; još ~0,3 s da ime i svjetlo „sjednu".
 */
export const SPLASH_MIN_MS = 2800;
/** Trajanje blijeđenja (`.hx-splash.is-leaving` u `index.html`). */
export const SPLASH_FADE_MS = 320;

export interface SplashEnv {
  doc: Document;
  /** `performance.now()` u trenutku poziva — isti sat kao `data-start` na uvodu. */
  now: number;
  reducedMotion: boolean;
  storage: Pick<Storage, "setItem"> | null;
  schedule: (fn: () => void, ms: number) => void;
}

/** Koliko se uvod već vidi; još nije nacrtan (nema `data-start`) → 0, priča tek počinje. */
function shownMs(splash: HTMLElement, now: number): number {
  const start = Number(splash.dataset.start);
  return Number.isFinite(start) ? Math.max(0, now - start) : 0;
}

export function dismissSplash(env: SplashEnv): void {
  const splash = env.doc.getElementById(SPLASH_ID);
  if (!splash) return;
  try {
    env.storage?.setItem(SPLASH_SEEN_KEY, "1");
  } catch {
    // Pohrana blokirana (privatni prozor): uvod će se samo prikazati i idući put.
  }
  // Sigurnosni rok iz `index.html` već vodi izlaz (i sam ukloni uvod): blijedi se samo jednom.
  const leaving = () => splash.classList.contains("is-leaving");
  if (leaving()) return;
  const wait = env.reducedMotion ? 0 : Math.max(0, SPLASH_MIN_MS - shownMs(splash, env.now));
  env.schedule(() => {
    if (leaving()) return;
    splash.classList.add("is-leaving");
    env.schedule(
      () => {
        splash.remove();
        env.doc.getElementById("root")?.removeAttribute("inert");
      },
      env.reducedMotion ? 0 : SPLASH_FADE_MS,
    );
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
    now: performance.now(),
    reducedMotion: window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    storage,
    schedule: (fn, ms) => window.setTimeout(fn, ms),
  });
}
