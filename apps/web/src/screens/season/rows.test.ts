import { describe, expect, it } from "vitest";
import { roundPhrase } from "../../format";
import { HOTEL_ID, ranked, seasonBody, seasonHotel, weekInfo } from "../../test/fixtures";
import {
  allTied,
  axisTicks,
  curveSeries,
  decisionRows,
  financeRows,
  gopMargin,
  hotelLabel,
  indexRows,
  rankingRows,
  rankLine,
  seasonDigits,
  strategySentence,
  winnerLine,
} from "./rows";

function withProgress(progress: [number, number, number, number]) {
  return seasonBody({ progress });
}

describe("poredak i pobjednik", () => {
  it("retci idu redom mjesta; robot nosi oznaku „(robot)“, moj hotel je označen", () => {
    const rows = rankingRows(seasonBody());
    expect(rows.map((r) => r.label)).toEqual([
      "Brižni (robot)",
      "Tvoj hotel",
      "Štediša (robot)",
      "Lovac na popunjenost (robot)",
    ]);
    expect(rows.map((r) => r.rank)).toEqual([1, 2, 3, 4]);
    expect(rows.find((r) => r.mine)?.hotelId).toBe(HOTEL_ID);
    expect(rows.map((r) => r.winner)).toEqual([true, false, false, false]);
  });

  it("jednak napredak dijeli mjesto, i u rečenici o mjestu", () => {
    const body = withProgress([500, 500, 1200, -100]);
    expect(rankingRows(body).map((r) => r.rank)).toEqual([1, 2, 2, 4]);
    expect(rankLine(body)).toBe("Napredak od početka · dijeliš 2. mjesto od 4");
    expect(rankLine(seasonBody())).toBe("Napredak od početka · 2. mjesto od 4");
  });

  it("pobjednik je prvi po napretku, rečenica bez roda prema studentu", () => {
    expect(winnerLine(seasonBody())).toBe("Pobijedio je Brižni (robot) s napretkom od +12 bodova.");
    expect(winnerLine(withProgress([1500, 0, 1200, -100]))).toBe(
      "Tvoj hotel je pobijedio: napredovao je najviše od svih.",
    );
    expect(winnerLine(withProgress([1200, 0, 1200, -100]))).toBe(
      "Tvoj hotel je pobijedio zajedno s hotelom Brižni (robot): napredovali su jednako, najviše od svih.",
    );
    expect(winnerLine(withProgress([100, 1200, 1200, -100]))).toBe(
      "Pobijedili su Štediša (robot) i Brižni (robot) s napretkom od +12 bodova.",
    );
  });

  it("svi s istim napretkom: jedna rečenica, nitko nije posebno istaknut", () => {
    const body = withProgress([300, 300, 300, 300]);
    expect(winnerLine(body)).toBe("Svi hoteli su napredovali jednako i dijele prvo mjesto.");
    expect(allTied(body)).toBe(true);
    expect(allTied(seasonBody())).toBe(false);
    expect(rankingRows(body).every((r) => !r.winner)).toBe(true);
    expect(rankingRows(body).every((r) => r.rank === 1)).toBe(true);
  });
});

describe("preciznost bodova prati sezonskog mentora", () => {
  it("cijeli bodovi kad se prikazani brojevi razlikuju", () => {
    expect(seasonDigits(seasonBody())).toBe(0);
    expect(rankingRows(seasonBody()).map((r) => r.progressText)).toEqual([
      "+12 bodova",
      "+8 bodova",
      "−3 boda",
      "−7 bodova",
    ]);
  });

  it("dva različita mjesta nikad nisu oba „+8“: prelazi se na desetinke za sve hotele", () => {
    const body = withProgress([760, -310, 840, -700]);
    expect(seasonDigits(body)).toBe(1);
    expect(rankingRows(body).map((r) => r.progressText)).toEqual([
      "+8,4 boda",
      "+7,6 boda",
      "−3,1 boda",
      "−7,0 boda",
    ]);
    expect(winnerLine(body)).toBe("Pobijedio je Brižni (robot) s napretkom od +8,4 boda.");
    const curve = curveSeries(body);
    // Krivulja ide redom poretka, istom preciznošću (i „−7,0“ uz „−3,1“, UI KS-3 M-1).
    expect(curve.map((s) => s.valueText)).toEqual(["+8,4 boda", "+7,6 boda", "−3,1 boda", "−7,0 boda"]);
  });
});

describe("krivulja", () => {
  it("svaki hotel počinje s 0 (početak), pa svaki krug; moj hotel ima svoj stil, roboti različite po slotu", () => {
    const series = curveSeries(seasonBody());
    expect(series).toHaveLength(4);
    for (const s of series) {
      expect(s.values[0]).toBe(0);
      expect(s.values).toHaveLength(18);
    }
    // Redom poretka (kao ljestvica), a stil prati hotel (slot), ne mjesto: moj 0, roboti 1, 2, 3 po slotu.
    expect(series.map((s) => [s.label, s.style])).toEqual([
      ["Brižni (robot)", 2],
      ["Tvoj hotel", 0],
      ["Štediša (robot)", 1],
      ["Lovac na popunjenost (robot)", 3],
    ]);
    // Drugačiji poredak ne mijenja stil hotela.
    const other = curveSeries(seasonBody({ progress: [100, 900, -500, 400] }));
    expect(other.map((s) => [s.label, s.style])).toEqual([
      ["Štediša (robot)", 1],
      ["Lovac na popunjenost (robot)", 3],
      ["Tvoj hotel", 0],
      ["Brižni (robot)", 2],
    ]);
  });

  it("tjedna os: Početak i mjeseci na granicama, nikad kod tjedna", () => {
    const ticks = axisTicks(seasonBody());
    expect(ticks.map((x) => x.text)).toEqual(["Početak", "lipanj", "srpanj", "kolovoz", "rujan"]);
    expect(ticks[0]?.at).toBe(0);
    expect(ticks.at(-1)?.at).toBe(17);
    expect(ticks.some((x) => /t\d/.test(x.text))).toBe(false);
  });

  it("mjesečna os: Početak i četiri mjeseca", () => {
    const ticks = axisTicks(seasonBody({ weekly: false }));
    expect(ticks.map((x) => [x.at, x.text])).toEqual([
      [0, "Početak"],
      [1, "lipanj"],
      [2, "srpanj"],
      [3, "kolovoz"],
      [4, "rujan"],
    ]);
  });
});

describe("kartica hotela", () => {
  const body = seasonBody();
  const brizni = body.hotels.find((h) => h.name === "Brižni");
  const mine = body.hotels.find((h) => h.mine);
  if (!brizni || !mine) throw new Error("fixture");

  it("indeksi: početak, kraj i promjena u cijelim bodovima, redom indeksa", () => {
    expect(indexRows(brizni)).toEqual([
      { code: "G", start: 45, end: 61, delta: 16 },
      { code: "R", start: 50, end: 52, delta: 2 },
      { code: "Z", start: 30, end: 52, delta: 22 },
      { code: "O", start: 55, end: 54, delta: -1 },
      { code: "F", start: 50, end: 47, delta: -3 },
    ]);
  });

  it("financije sezone kroz format.ts; bez financija nema redaka (ne nule)", () => {
    expect(financeRows(brizni)?.map((r) => [r.label, r.value, r.note])).toEqual([
      ["Prihod (sobe i ostalo)", "412.300 €", null],
      ["GOP — bruto operativna dobit", "131.900 €", "32,0 % prihoda ostalo je kao dobit"],
      ["Blagajna: početak → kraj", "80.000 € → 151.200 €", null],
    ]);
    expect(financeRows({ ...brizni, finances: null })).toBeNull();
    // GOP marža u tablici uvijek s jednom decimalom, da se stupac može usporediti („38,0 %“ uz „42,3 %“).
    expect(gopMargin(brizni)).toBe("32,0 %");
    expect(gopMargin({ ...brizni, finances: null })).toBeNull();
  });

  it("odluke u brojkama: isti retci za robota i za mene", () => {
    const rows = decisionRows(mine, body);
    expect(rows?.map((r) => [r.label, r.value])).toEqual([
      ["Prosječna cijena sobe", "98 €"],
      ["Marketing u sezoni", "42.000 €"],
      ["Plaće", "100–105 % uobičajenih"],
      ["Najviše sezonskih radnika istodobno", "24 osobe"],
      ["Obuka", "6 od 17 tjedana"],
      ["Odgođeno održavanje", "0 od 17 tjedana"],
      ["Odgovaranje na recenzije", "0 od 17 tjedana"],
      ["Obnova", "nije bilo"],
      ["Situacije", "34 situacije · prosječna ocjena 3,6/5"],
      ["Bez odgovora", "2 situacije"],
    ]);
    const fixed = decisionRows(
      {
        ...brizni,
        decisions: brizni.decisions && {
          ...brizni.decisions,
          wage_pct_min: 103,
          wage_pct_max: 103,
          renovation_round: "t05",
          avg_quality_hundredths: null,
        },
      },
      body,
    );
    expect(fixed?.find((r) => r.key === "wage")?.value).toBe("103 % uobičajenih");
    expect(fixed?.find((r) => r.key === "renovation")?.value).toBe(roundPhrase(weekInfo(5)));
    expect(fixed?.find((r) => r.key === "situations")?.value).toBe("34 situacije");
    expect(decisionRows({ ...brizni, decisions: null }, body)).toBeNull();
  });

  it("mjesečna igra broji mjesece", () => {
    const monthly = seasonBody({ weekly: false });
    const h = monthly.hotels[0];
    if (!h) throw new Error("fixture");
    expect(decisionRows(h, monthly)?.find((r) => r.key === "training")?.value).toBe("4 od 4 mjeseca");
  });

  it("rečenica strategije za svakog robota; moj hotel je nema", () => {
    expect(strategySentence(brizni, true)).toMatch(
      /^Brižni je ulagao u ljude i goste: cijena sobe 5\s%\siznad/,
    );
    const stedisa = body.hotels.find((h) => h.name === "Štediša");
    const lovac = body.hotels.find((h) => h.name === "Lovac na popunjenost");
    expect(stedisa && strategySentence(stedisa, true)).toMatch(/^Štediša je cijelu sezonu čuvao novac/);
    expect(lovac && strategySentence(lovac, true)).toMatch(/^Lovac na popunjenost htio je puni hotel/);
    expect(strategySentence(mine, true)).toBeNull();
    // Mjesečna igra: Brižni bez rečenice o tjednom zapošljavanju (UI KS-3 M-8).
    expect(strategySentence(brizni, true)).toContain("u tjednoj igri");
    expect(strategySentence(brizni, false)).not.toContain("tjedn");
    expect(strategySentence(brizni, false)).toMatch(/plaće 103\s% uobičajenih/);
  });

  it("tuđi ljudski hotel ima oznaku drugog studenta", () => {
    const other = ranked([seasonHotel("h9", 2, "Hotel Ana", null, 100)])[0];
    if (!other) throw new Error("fixture");
    expect(hotelLabel({ ...other, mine: false })).toBe("Hotel Ana (drugi student)");
  });
});
