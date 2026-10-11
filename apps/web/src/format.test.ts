import { describe, expect, it } from "vitest";
import {
  capitalize,
  formatCount,
  formatEuros,
  formatEurosCents,
  formatEurosSigned,
  formatIndex,
  formatLoad,
  formatNights,
  formatPercentBp,
  formatPointsCount,
  formatRange,
  formatRating,
  formatWhole,
  formatWholeSigned,
  monthName,
  roundDates,
  roundTitle,
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
  it("formatRating je ocjena do 10, uvijek s točno jednom decimalom", () => {
    expect(formatRating(840)).toBe("8,4/10");
    expect(formatRating(800)).toBe("8,0/10");
    expect(formatRating(898)).toBe("9,0/10");
    expect(formatRating(803)).toBe("8,0/10");
    expect(formatRating(1000)).toBe("10,0/10");
  });
  it("formatRange: raspon s grupiranjem i jedinicom, imenica po gornjoj granici", () => {
    expect(formatRange(0, 40_000, "euro")).toBe("0–40.000 €");
    expect(formatRange(81, 202, "euro")).toBe("81–202 €");
    expect(formatRange(90, 115, "percent")).toBe("90–115 %");
    expect(formatRange(18, 38, "person")).toBe("18–38 osoba");
    expect(formatRange(1, 2, "person")).toBe("1–2 osobe");
    expect(formatWhole(40_000)).toBe("40.000");
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

describe("oznaka kruga", () => {
  it("oznaka tjedna i mjeseca", () => {
    const t05 = {
      code: "t05",
      kind: "week",
      index: 4,
      days: 7,
      first_day: 29,
      first_month: "jun",
      last_day: 5,
      last_month: "jul",
      month_ends: true,
    } as const;
    expect(roundDates(t05)).toBe("29. 6. – 5. 7.");
    expect(roundTitle(t05)).toBe("Tjedan 5 (29. 6. – 5. 7.)");
    const jun = {
      code: "jun",
      kind: "month",
      index: 0,
      days: 30,
      first_day: 1,
      first_month: "jun",
      last_day: 30,
      last_month: "jun",
      month_ends: true,
    } as const;
    expect(roundDates(jun)).toBe("lipanj");
    expect(roundTitle(jun)).toBe("Lipanj");
  });
});

describe("formatEurosSigned", () => {
  it("predznak i cijeli euri, nula bez predznaka", () => {
    expect(formatEurosSigned(789_600)).toBe("+7.896 €");
    expect(formatEurosSigned(-80_000)).toBe("−800 €");
    expect(formatEurosSigned(0)).toBe("0 €");
  });
});

describe("formatLoad i množina ljudi", () => {
  it("gosti po zaposlenom: zarez i oblik uz decimale", () => {
    expect(formatLoad(190)).toBe("1,9 gosta");
    expect(formatLoad(100)).toBe("1 gost");
    expect(formatLoad(500)).toBe("5 gostiju");
  });
  it("stalni i sezonski: 1, 2–4, 5+", () => {
    expect([1, 2, 5].map((n) => formatCount(n, "permanent"))).toEqual(["1 stalni", "2 stalna", "5 stalnih"]);
    expect([1, 2, 10].map((n) => formatCount(n, "seasonal"))).toEqual([
      "1 sezonski",
      "2 sezonska",
      "10 sezonskih",
    ]);
  });
});
