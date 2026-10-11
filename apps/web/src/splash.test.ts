import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import indexHtml from "../index.html?raw";
import { dismissSplash, SPLASH_FADE_MS, SPLASH_ID, SPLASH_MIN_MS, SPLASH_SEEN_KEY } from "./splash";

type Job = { fn: () => void; ms: number };

function env(over: { now?: number; reducedMotion?: boolean; storage?: Pick<Storage, "setItem"> | null }) {
  const jobs: Job[] = [];
  const saved: Record<string, string> = {};
  return {
    jobs,
    saved,
    env: {
      doc: document,
      now: over.now ?? 0,
      reducedMotion: over.reducedMotion ?? false,
      storage:
        over.storage === undefined
          ? {
              setItem: (k: string, v: string) => {
                saved[k] = v;
              },
            }
          : over.storage,
      schedule: (fn: () => void, ms: number) => {
        jobs.push({ fn, ms });
      },
    },
  };
}

function runNext(jobs: Job[]): number {
  const job = jobs.shift();
  if (!job) throw new Error("nema zakazanog posla");
  job.fn();
  return job.ms;
}

/** Uvod koji je skripta u `index.html` prvi put nacrtala u `startMs` (isti sat kao `performance.now()`). */
function paintedAt(startMs: number) {
  const splash = document.getElementById(SPLASH_ID);
  if (!splash) throw new Error("nema uvoda");
  splash.dataset.start = String(startMs);
}

const root = () => document.getElementById("root");

describe("dismissSplash", () => {
  beforeEach(() => {
    document.body.innerHTML = `<div id="${SPLASH_ID}"></div><div id="root" inert></div>`;
  });

  it("bez uvoda ne radi ništa", () => {
    document.body.innerHTML = `<div id="root"></div>`;
    const { env: e, jobs, saved } = env({});
    dismissSplash(e);
    expect(jobs).toHaveLength(0);
    expect(saved).toEqual({});
  });

  it("čeka cijelu priču mjereno od prvog crtanja uvoda, pa uvod izblijedi i nestane", () => {
    paintedAt(1000);
    const { env: e, jobs } = env({ now: 1500 });
    dismissSplash(e);
    const splash = document.getElementById(SPLASH_ID);

    expect(runNext(jobs)).toBe(SPLASH_MIN_MS - 500);
    expect(splash?.classList.contains("is-leaving")).toBe(true);
    expect(document.getElementById(SPLASH_ID)).not.toBeNull();

    expect(runNext(jobs)).toBe(SPLASH_FADE_MS);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
  });

  it("aplikacija spremna prije prvog crtanja uvoda: čeka cijelu priču", () => {
    const { env: e, jobs } = env({ now: 5000 });
    dismissSplash(e);
    expect(runNext(jobs)).toBe(SPLASH_MIN_MS);
  });

  it("aplikacija sporija od uvoda: izlazi odmah", () => {
    paintedAt(300);
    const { env: e, jobs } = env({ now: 300 + SPLASH_MIN_MS + 700 });
    dismissSplash(e);
    expect(runNext(jobs)).toBe(0);
  });

  it("sadržaj ispod uvoda je nedostupan (inert) dok uvod ne ode", () => {
    paintedAt(0);
    const { env: e, jobs } = env({ now: 100 });
    dismissSplash(e);
    runNext(jobs);
    expect(root()?.hasAttribute("inert")).toBe(true);
    runNext(jobs);
    expect(root()?.hasAttribute("inert")).toBe(false);
  });

  it("sigurnosni rok je već pokrenuo izlaz: blijeđenje se ne ponavlja", () => {
    document.getElementById(SPLASH_ID)?.classList.add("is-leaving");
    const { env: e, jobs, saved } = env({ now: 9100 });
    dismissSplash(e);
    expect(jobs).toHaveLength(0);
    expect(saved[SPLASH_SEEN_KEY]).toBe("1");
  });

  it("sigurnosni rok krene dok izlaz čeka: blijeđenje se ne ponavlja", () => {
    paintedAt(0);
    const { env: e, jobs } = env({ now: 100 });
    dismissSplash(e);
    document.getElementById(SPLASH_ID)?.classList.add("is-leaving");
    runNext(jobs);
    expect(jobs).toHaveLength(0);
  });

  it("smanjeni pokreti: bez čekanja i bez blijeđenja", () => {
    paintedAt(0);
    const { env: e, jobs } = env({ now: 100, reducedMotion: true });
    dismissSplash(e);
    expect(runNext(jobs)).toBe(0);
    expect(runNext(jobs)).toBe(0);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
    expect(root()?.hasAttribute("inert")).toBe(false);
  });

  it("pamti da je uvod viđen u ovoj kartici", () => {
    const { env: e, saved } = env({});
    dismissSplash(e);
    expect(saved[SPLASH_SEEN_KEY]).toBe("1");
  });

  it("nedostupna pohrana ne ruši izlaz", () => {
    const throwing = {
      setItem: () => {
        throw new Error("pohrana blokirana");
      },
    };
    const { env: e, jobs } = env({ storage: throwing });
    dismissSplash(e);
    runNext(jobs);
    runNext(jobs);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
  });
});

/** Skripta uvoda iz `index.html` (bez `type="module"`): pokrene se nad istim oznakama kao u pregledniku. */
function runIndexScript() {
  const page = new DOMParser().parseFromString(indexHtml, "text/html");
  const script = [...page.querySelectorAll("script")].find((s) => !s.type);
  if (!script) throw new Error("nema skripte uvoda u index.html");
  page.querySelectorAll("script").forEach((s) => {
    s.remove();
  });
  document.body.innerHTML = page.body.innerHTML;
  new Function(script.textContent ?? "")();
}

const FAIL_TEXT = "Učitavanje traje dulje nego obično. Osvježi stranicu.";

describe("sigurnosni rok u index.html (aplikacija se ne učita)", () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.useFakeTimers({ toFake: ["setTimeout", "requestAnimationFrame"] });
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("uvod ode, zapamti se kao viđen i ostane kratka poruka umjesto prazne stranice", () => {
    runIndexScript();
    expect(root()?.hasAttribute("inert")).toBe(true);
    vi.advanceTimersByTime(16 + 9000);
    expect(document.getElementById(SPLASH_ID)?.classList.contains("is-leaving")).toBe(true);
    expect(sessionStorage.getItem(SPLASH_SEEN_KEY)).toBe("1");
    vi.advanceTimersByTime(SPLASH_FADE_MS);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
    expect(root()?.hasAttribute("inert")).toBe(false);
    expect(root()?.textContent).toBe(FAIL_TEXT);
  });

  it("aplikacija je stigla kasno, ali prije uklanjanja: poruke nema", () => {
    runIndexScript();
    vi.advanceTimersByTime(16 + 9000);
    const app = document.createElement("main");
    app.textContent = "Započni igru";
    root()?.append(app);
    vi.advanceTimersByTime(SPLASH_FADE_MS);
    expect(root()?.textContent).toBe("Započni igru");
  });

  it("izlaz već vodi splash.ts: rok ne radi ništa", () => {
    runIndexScript();
    document.getElementById(SPLASH_ID)?.classList.add("is-leaving");
    vi.advanceTimersByTime(16 + 9000 + SPLASH_FADE_MS);
    expect(sessionStorage.getItem(SPLASH_SEEN_KEY)).toBeNull();
    expect(root()?.textContent).toBe("");
  });
});

describe("vrata u index.html", () => {
  const css = indexHtml.slice(indexHtml.indexOf("<style>"), indexHtml.indexOf("</style>"));
  const commands = (d: string) => d.replace(/[^A-Za-z]/g, "");

  it("oblik vrata (CSS d) prelazi iz zatvorenog u otvoreni s istim naredbama putanje, inače se ne animira", () => {
    const paths = [...css.matchAll(/d: path\("([^"]+)"\)/g)].map((m) => m[1] ?? "");
    const door = paths.filter((p) => p !== "M0 0");
    expect(door).toHaveLength(2);
    expect(commands(door[0] ?? "")).toBe(commands(door[1] ?? ""));
  });

  it("smanjeni pokreti gase i animaciju oblika (pravilo stoji iza @supports)", () => {
    expect(css.indexOf("@supports (d:")).toBeGreaterThan(-1);
    expect(css.indexOf("@supports (d:")).toBeLessThan(css.indexOf("prefers-reduced-motion"));
  });
});
