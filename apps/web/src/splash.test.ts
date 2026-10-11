import { beforeEach, describe, expect, it } from "vitest";
import { dismissSplash, SPLASH_FADE_MS, SPLASH_ID, SPLASH_MIN_MS, SPLASH_SEEN_KEY } from "./splash";

type Job = { fn: () => void; ms: number };

function env(over: {
  elapsedMs?: number;
  reducedMotion?: boolean;
  storage?: Pick<Storage, "setItem"> | null;
}) {
  const jobs: Job[] = [];
  const saved: Record<string, string> = {};
  return {
    jobs,
    saved,
    env: {
      doc: document,
      elapsedMs: over.elapsedMs ?? 0,
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

describe("dismissSplash", () => {
  beforeEach(() => {
    document.body.innerHTML = `<div id="${SPLASH_ID}"></div><div id="root"></div>`;
  });

  it("bez uvoda ne radi ništa", () => {
    document.body.innerHTML = `<div id="root"></div>`;
    const { env: e, jobs, saved } = env({});
    dismissSplash(e);
    expect(jobs).toHaveLength(0);
    expect(saved).toEqual({});
  });

  it("čeka da se vrata otvore, pa uvod izblijedi i nestane", () => {
    const { env: e, jobs } = env({ elapsedMs: 500 });
    dismissSplash(e);
    const splash = document.getElementById(SPLASH_ID);

    expect(runNext(jobs)).toBe(SPLASH_MIN_MS - 500);
    expect(splash?.classList.contains("is-leaving")).toBe(true);
    expect(document.getElementById(SPLASH_ID)).not.toBeNull();

    expect(runNext(jobs)).toBe(SPLASH_FADE_MS);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
  });

  it("aplikacija sporija od uvoda: izlazi odmah", () => {
    const { env: e, jobs } = env({ elapsedMs: SPLASH_MIN_MS + 700 });
    dismissSplash(e);
    expect(runNext(jobs)).toBe(0);
  });

  it("smanjeni pokreti: bez čekanja i bez blijeđenja", () => {
    const { env: e, jobs } = env({ elapsedMs: 100, reducedMotion: true });
    dismissSplash(e);
    expect(runNext(jobs)).toBe(0);
    expect(runNext(jobs)).toBe(0);
    expect(document.getElementById(SPLASH_ID)).toBeNull();
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
