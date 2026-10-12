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
  formatPercentBpFixed,
  formatPercentWhole,
  formatPointsCount,
  formatPointsPrecise,
  formatQuality,
  formatRange,
  formatRating,
  formatRoundsOf,
  formatWhole,
  formatWholeSigned,
  keepRoundTogether,
  monthName,
  pointsAt,
  progressDigits,
  progressTone,
  roundDates,
  roundPhrase,
  roundTitle,
  roundWithoutGame,
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
    expect(formatEuros(9_800_000)).toBe("98.000\u00a0€");
    expect(formatEuros(4_200_049)).toBe("42.000\u00a0€");
    expect(formatEuros(13_200)).toBe("132\u00a0€");
  });
  it("formatEuros uvijek grupira tisuće, i za četiri znamenke", () => {
    expect(formatEuros(123_400)).toBe("1.234\u00a0€");
    expect(formatEuros(-123_400)).toBe("−1.234\u00a0€");
  });
  it("formatRating je ocjena do 10, uvijek s točno jednom decimalom", () => {
    expect(formatRating(840)).toBe("8,4/10");
    expect(formatRating(800)).toBe("8,0/10");
    expect(formatRating(898)).toBe("9,0/10");
    expect(formatRating(803)).toBe("8,0/10");
    expect(formatRating(1000)).toBe("10,0/10");
  });
  it("formatRange: raspon s grupiranjem i jedinicom, imenica po gornjoj granici", () => {
    expect(formatRange(0, 40_000, "euro")).toBe("0–40.000\u00a0€");
    expect(formatRange(81, 202, "euro")).toBe("81–202\u00a0€");
    expect(formatRange(90, 115, "percent")).toBe("90–115\u00a0%");
    expect(formatRange(18, 38, "person")).toBe("18–38 osoba");
    expect(formatRange(1, 2, "person")).toBe("1–2 osobe");
    expect(formatWhole(40_000)).toBe("40.000");
  });
  it("pokazatelji: postotak iz baznih bodova, euri s centima, noći; null je crtica", () => {
    expect([6800, 3173, 12_121, -450].map(formatPercentBp)).toEqual([
      "68\u00a0%",
      "31,7\u00a0%",
      "121,2\u00a0%",
      "−4,5\u00a0%",
    ]);
    expect(formatPercentBp(null)).toBe("—");
    expect([1300, 1590, -450].map(formatPercentBpFixed)).toEqual([
      "13,0\u00a0%",
      "15,9\u00a0%",
      "−4,5\u00a0%",
    ]);
    expect(formatPercentBpFixed(null)).toBe("—");
    expect([4003, 123_456, -5500, 0].map(formatEurosCents)).toEqual([
      "40,03\u00a0€",
      "1.234,56\u00a0€",
      "−55,00\u00a0€",
      "0,00\u00a0€",
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
    expect(roundDates(t05)).toBe("29.\u00a06.\u00a0–\u2060\u00a05.\u00a07.");
    expect(roundTitle(t05)).toBe("Tjedan\u00a05 (29.\u00a06.\u00a0–\u2060\u00a05.\u00a07.)");
    expect(roundPhrase(t05)).toBe("tjedan\u00a05 (29.\u00a06.\u00a0–\u2060\u00a05.\u00a07.)");
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
    expect(formatEurosSigned(789_600)).toBe("+7.896\u00a0€");
    expect(formatEurosSigned(-80_000)).toBe("−800\u00a0€");
    expect(formatEurosSigned(0)).toBe("0\u00a0€");
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

describe("plan odjela", () => {
  it("sezonski radnici s množinom (i u akuzativu), razina cijene kao cijeli postotak", () => {
    expect([1, 3, 5].map((n) => formatCount(n, "seasonalWorker"))).toEqual([
      "1 sezonski radnik",
      "3 sezonska radnika",
      "5 sezonskih radnika",
    ]);
    expect([1, 3, 5, 21].map((n) => formatCount(n, "seasonalWorkerAcc"))).toEqual([
      "1 sezonskog radnika",
      "3 sezonska radnika",
      "5 sezonskih radnika",
      "21 sezonskog radnika",
    ]);
    expect(formatPercentWhole(110)).toBe("110\u00a0%");
  });
});

describe("roundWithoutGame", () => {
  it("tjedan bez igre: „tjedan 5“ s nerazdvojnim razmakom, mjesec ostaje ime mjeseca", () => {
    expect(roundWithoutGame("t05", "jun")).toBe("tjedan\u00a05");
    expect(roundWithoutGame("jul", "jul")).toBe("srpanj");
  });
});

describe("keepRoundTogether", () => {
  it("broj tjedna i datumi u zagradi ne lome se, prijelaz je moguć samo ispred zagrade", () => {
    expect(keepRoundTogether("Račun dobiti — Tjedan 4 (22. 6. – 28. 6.)")).toBe(
      "Račun dobiti — Tjedan\u00a04 (22.\u00a06.\u00a0–\u2060\u00a028.\u00a06.)",
    );
    expect(keepRoundTogether("Račun dobiti — sezona do tjedna 4 (22. 6. – 28. 6.)")).toBe(
      "Račun dobiti — sezona do tjedna\u00a04 (22.\u00a06.\u00a0–\u2060\u00a028.\u00a06.)",
    );
  });
  it("poziv je idempotentan: tekst iz roundPhrase/roundDates prolazi opet isti", () => {
    const once = keepRoundTogether("Tjedan 1 završen (1. 6. – 7. 6.)");
    expect(keepRoundTogether(once)).toBe(once);
    expect(once).toBe("Tjedan\u00a01 završen (1.\u00a06.\u00a0–\u2060\u00a07.\u00a06.)");
  });
  it("mjesečni naslov ostaje kakav je", () => {
    expect(keepRoundTogether("Račun dobiti — Lipanj")).toBe("Račun dobiti — Lipanj");
  });
});

describe("bodovi napretka s preciznošću kraja sezone", () => {
  it("pointsAt zaokružuje pola od nule na zadani broj decimala (isto kao mentor)", () => {
    expect(pointsAt(845, 0)).toBe(8);
    expect(pointsAt(845, 1)).toBe(8.5);
    expect(pointsAt(-845, 1)).toBe(-8.5);
    expect(pointsAt(-844, 1)).toBe(-8.4);
    expect(pointsAt(842, 2)).toBe(8.42);
  });
  it("formatPointsPrecise: cijeli bodovi kao prije; uz decimale uvijek isti broj decimala i „boda“", () => {
    expect(formatPointsPrecise(800, 0)).toBe("+8 bodova");
    expect(formatPointsPrecise(840, 1)).toBe("+8,4 boda");
    // Ista preciznost u istoj usporedbi: „−4,0“ uz „−3,7“, ne „−4“ (UI KS-3 M-1).
    expect(formatPointsPrecise(800, 1)).toBe("+8,0 boda");
    expect(formatPointsPrecise(-401, 1)).toBe("−4,0 boda");
    expect(formatPointsPrecise(-760, 1)).toBe("−7,6 boda");
    expect(formatPointsPrecise(842, 2)).toBe("+8,42 boda");
    expect(formatPointsPrecise(840, 2)).toBe("+8,40 boda");
    expect(formatPointsPrecise(4, 1)).toBe("0,0 boda");
    expect(formatPointsPrecise(4, 0)).toBe("0 bodova");
    expect(formatPointsPrecise(100, 0)).toBe("+1 bod");
  });
  it("progressDigits: najmanje decimala uz koje se različiti napretci i dalje razlikuju", () => {
    expect(progressDigits([1210, 800, -300, -700])).toBe(0);
    // +8,4 i +7,6 su oba „+8“: treba desetinka
    expect(progressDigits([840, 760, -300])).toBe(1);
    // +8,42 i +8,38 su oba „+8,4“: trebaju stotinke
    expect(progressDigits([842, 838])).toBe(2);
    // jednaki napredak ne traži decimale
    expect(progressDigits([800, 800, 500])).toBe(0);
    expect(progressDigits([])).toBe(0);
  });
  it("progressTone prati prikazani broj: −0,3 uz desetinke je pad, uz cijele bodove nula", () => {
    expect(progressTone(-30)).toBe("text-muted");
    expect(progressTone(-30, 1)).toBe("text-down");
  });
});

describe("kraj sezone: ocjena situacija i broj krugova", () => {
  it("formatQuality je prosjek ocjena 1–5 s jednom decimalom", () => {
    expect(formatQuality(360)).toBe("3,6/5");
    expect(formatQuality(400)).toBe("4,0/5");
  });
  it("formatRoundsOf: „6 od 17 tjedana“, „1 od 4 mjeseca“", () => {
    expect(formatRoundsOf(6, 17, true)).toBe("6 od 17 tjedana");
    expect(formatRoundsOf(17, 17, true)).toBe("17 od 17 tjedana");
    expect(formatRoundsOf(1, 4, false)).toBe("1 od 4 mjeseca");
    expect(formatRoundsOf(0, 4, false)).toBe("0 od 4 mjeseca");
  });
});
