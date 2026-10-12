import { describe, expect, it } from "vitest";
import { spreadLabels, yRange } from "./chartScale";

describe("yRange", () => {
  it("uvijek uključuje 0 i zaokružuje na 5 bodova", () => {
    expect(yRange([0, 1210, 825, -310, -700])).toEqual({ lo: -1000, hi: 1500, step: 500 });
    expect(yRange([0, 200, 300])).toEqual({ lo: 0, hi: 500, step: 500 });
  });
  it("ravna crta na nuli dobije raspon oko nule", () => {
    expect(yRange([0, 0])).toEqual({ lo: -500, hi: 500, step: 500 });
  });
  it("velik raspon ide po 10 bodova", () => {
    expect(yRange([0, 3200, -1500])).toEqual({ lo: -2000, hi: 4000, step: 1000 });
  });
});

describe("spreadLabels", () => {
  it("razmakne oznake koje se preklapaju, redoslijed ulaza ostaje", () => {
    const out = spreadLabels([100, 102, 50], 15, 0, 200);
    expect(out[2]).toBe(50);
    expect(out[0]).toBe(100);
    expect(out[1]).toBe(115);
  });
  it("ne izlazi ispod dna: gura gore", () => {
    expect(spreadLabels([200, 200], 15, 0, 200)).toEqual([185, 200]);
  });
});
