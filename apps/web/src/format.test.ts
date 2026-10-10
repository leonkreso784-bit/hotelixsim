import { describe, expect, it } from "vitest";
import {
  capitalize,
  formatCount,
  formatEuros,
  formatEurosCents,
  formatIndex,
  formatNights,
  formatPercentBp,
  formatPointsCount,
  formatRating,
  formatWholeSigned,
  monthName,
  strategyLabel,
} from "./format";

describe("formatIndex", () => {
  it("prikazuje stotinke hrvatskim zarezom i bez suvišnih nula", () => {
    expect(formatIndex(5000)).toBe("50");
    expect(formatIndex(0)).toBe("0");
    expect(formatIndex(10000)).toBe("100");
  });
  it("prikazuje cijeli broj, pola zaokružuje od nule (4850 → 49, 4849 → 48)", () => {
    expect(formatIndex(4850)).toBe("49");
    expect(formatIndex(4849)).toBe("48");
    expect(formatIndex(4876)).toBe("49");
    expect(formatIndex(5025)).toBe("50");
  });
});

describe("formatWholeSigned", () => {
  it("piše predznak, pravi minus i nulu bez predznaka", () => {
    expect(formatWholeSigned(8)).toBe("+8");
    expect(formatWholeSigned(-2)).toBe("−2");
    expect(formatWholeSigned(0)).toBe("0");
    expect(formatWholeSigned(1234)).toBe("+1.234");
  });
});

describe("formatPointsCount", () => {
  it("stotinke → cijeli bodovi s predznakom i množinom", () => {
    expect(formatPointsCount(450)).toBe("+5 bodova");
    expect(formatPointsCount(449)).toBe("+4 boda");
    expect(formatPointsCount(-100)).toBe("−1 bod");
    expect(formatPointsCount(20)).toBe("0 bodova");
  });
});

describe("novac i ocjena", () => {
  it("formatEuros zaokružuje na cijele eure s točkom za tisuće", () => {
    expect(formatEuros(9_800_000)).toBe("98.000 €");
    expect(formatEuros(4_200_049)).toBe("42.000 €");
    expect(formatEuros(13_200)).toBe("132 €");
  });
  it("formatEuros uvijek grupira tisuće, i za četiri znamenke", () => {
    expect(formatEuros(123_400)).toBe("1.234 €");
    expect(formatEuros(-123_400)).toBe("−1.234 €");
  });
  it("formatRating je ocjena do 10", () => {
    expect(formatRating(840)).toBe("8,4/10");
    expect(formatRating(845)).toBe("8,45/10");
    expect(formatRating(1000)).toBe("10/10");
  });
  it("pokazatelji: postotak iz baznih bodova, euri s centima, noći; null je crtica", () => {
    expect([6800, 3173, 12_121, -450].map(formatPercentBp)).toEqual(["68 %", "31,7 %", "121,2 %", "−4,5 %"]);
    expect(formatPercentBp(null)).toBe("—");
    expect([4003, 123_456, -5500, 0].map(formatEurosCents)).toEqual([
      "40,03 €",
      "1.234,56 €",
      "−55,00 €",
      "0,00 €",
    ]);
    expect(formatEurosCents(null)).toBe("—");
    expect([1, 3, 60, 3600].map(formatNights)).toEqual(["1 noć", "3 noći", "60 noći", "3.600 noći"]);
  });
});

describe("nazivi", () => {
  it("mjeseci i strategije su hrvatski", () => {
    expect(monthName("jun")).toBe("lipanj");
    expect(capitalize(monthName("jul"))).toBe("Srpanj");
    expect(strategyLabel("lovac")).toBe("lovi popunjenost");
  });
});

describe("formatCount", () => {
  it("množina: 1, 2–4, 5+, 11, 21, 22", () => {
    const f = (n: number) => formatCount(n, "point");
    expect([1, 2, 5, 11, 21, 22].map(f)).toEqual([
      "1 bod",
      "2 boda",
      "5 bodova",
      "11 bodova",
      "21 bod",
      "22 boda",
    ]);
  });
  it("bod: bod, boda, bodova; decimale uz „boda“", () => {
    expect([1, 3, 8].map((n) => formatCount(n, "point"))).toEqual(["1 bod", "3 boda", "8 bodova"]);
    expect(formatCount(8.25, "point")).toBe("8,25 boda");
    expect([1, 2, 4, 5, 11, 32].map((n) => formatCount(n, "person"))).toEqual([
      "1 osoba",
      "2 osobe",
      "4 osobe",
      "5 osoba",
      "11 osoba",
      "32 osobe",
    ]);
  });
});
