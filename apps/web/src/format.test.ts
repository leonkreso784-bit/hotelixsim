import { describe, expect, it } from "vitest";
import {
  capitalize,
  causeLabel,
  formatChange,
  formatCount,
  formatEuros,
  formatIndex,
  formatOccupancy,
  formatRating,
  formatSigned,
  formatSignedEuros,
  monthName,
  strategyLabel,
  varLabel,
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

describe("formatSigned", () => {
  it("piše predznak, pravi minus i nulu bez predznaka", () => {
    expect(formatSigned(800)).toBe("+8");
    expect(formatSigned(825)).toBe("+8,25");
    expect(formatSigned(-200)).toBe("−2");
    expect(formatSigned(0)).toBe("0");
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
  it("formatSignedEuros piše predznak", () => {
    expect(formatSignedEuros(1_250_000)).toBe("+12.500 €");
    expect(formatSignedEuros(-1_250_000)).toBe("−12.500 €");
  });
  it("formatRating je ocjena do 10", () => {
    expect(formatRating(840)).toBe("8,4/10");
    expect(formatRating(845)).toBe("8,45/10");
    expect(formatRating(1000)).toBe("10/10");
  });
  it("formatOccupancy je postotak prodanih, a bez kapaciteta crtica", () => {
    expect(formatOccupancy(2448, 3600)).toBe("68 %");
    expect(formatOccupancy(0, 0)).toBe("—");
  });
});

describe("formatChange", () => {
  it("svaka jedinica ima svoj oblik", () => {
    expect(formatChange(312, "room_nights")).toBe("+312 noćenja");
    expect(formatChange(-1_250_000, "cents")).toBe("−12.500 €");
    expect(formatChange(-3, "people")).toBe("−3 ljudi");
    expect(formatChange(450, "points")).toBe("+5 bodova");
    expect(formatChange(449, "points")).toBe("+4 boda");
    expect(formatChange(-100, "points")).toBe("−1 bod");
    expect(formatChange(-1200, "points")).toBe("−12 bodova");
    expect(formatChange(20, "points")).toBe("0 bodova");
    expect(formatChange(12, "rating_hundredths")).toBe("+0,12 ocjene");
    expect(formatChange(-1200, "bp")).toBe("−12 %");
  });
});

describe("nazivi", () => {
  it("mjeseci, uzroci, varijable i strategije su hrvatski", () => {
    expect(monthName("jun")).toBe("lipanj");
    expect(capitalize(monthName("jul"))).toBe("Srpanj");
    expect(causeLabel("reply_reviews")).toBe("odgovori na recenzije");
    expect(varLabel("sold")).toBe("Prodane noći");
    expect(strategyLabel("lovac")).toBe("lovi popunjenost");
  });
});

describe("formatCount", () => {
  it("promjena: 1, 2–4, 5+, 11, 21, 22", () => {
    const f = (n: number) => formatCount(n, "change");
    expect([1, 2, 5, 11, 21, 22].map(f)).toEqual([
      "1 promjena",
      "2 promjene",
      "5 promjena",
      "11 promjena",
      "21 promjena",
      "22 promjene",
    ]);
  });
  it("bod: bod, boda, bodova; decimale uz „boda“", () => {
    expect([1, 3, 8].map((n) => formatCount(n, "point"))).toEqual(["1 bod", "3 boda", "8 bodova"]);
    expect(formatCount(8.25, "point")).toBe("8,25 boda");
  });
});
