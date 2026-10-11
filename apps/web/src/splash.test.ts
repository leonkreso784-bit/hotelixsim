import { beforeEach, describe, expect, it } from "vitest";
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
