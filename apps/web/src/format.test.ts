import { describe, expect, it } from "vitest";
import {
  capitalize,
  causeLabel,
  formatChange,
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
    expect(formatIndex(5025)).toBe("50,25");
    expect(formatIndex(0)).toBe("0");
    expect(formatIndex(10000)).toBe("100");
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
    expect(formatChange(450, "points")).toBe("+4,5 bodova");
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
