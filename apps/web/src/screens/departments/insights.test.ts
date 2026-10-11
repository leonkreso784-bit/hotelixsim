import { describe, expect, it } from "vitest";
import type { DeptRoundBody } from "../../api/types";
import { departmentsBody } from "../../test/fixtures";
import { INSIGHT_LIMITS, insights, staffUsePct } from "./insights";

/** Restoran iz t01, s neutralnim vrijednostima: nijedno pravilo ne pali dok ga test ne pomakne. */
function round(over: Partial<DeptRoundBody> = {}): DeptRoundBody {
  const last = departmentsBody().departments.find((d) => d.code === "restaurant")?.last;
  if (!last) throw new Error("restoran bez kruga");
  return {
    ...last,
    load_hundredths: 144, // 80 % mogućnosti
    served: 2000,
    demand: 2000,
    condition_after_pts: 6000,
    satisfaction_before_pts: 7000,
    satisfaction_after_pts: 7000,
    profit_cents: 0,
    ...over,
  };
}

const texts = (r: DeptRoundBody, weekly = true) => {
  const x = insights(r, weekly);
  return { weak: x.weak.map((i) => i.text), strong: x.strong.map((i) => i.text) };
};

describe("pragovi u jednoj tablici", () => {
  it("vrijednosti pragova", () => {
    expect(INSIGHT_LIMITS).toEqual({
      fullLoadHundredths: 180,
      overloadedPct: 100,
      idlePct: 60,
      conditionWorn: 50,
      conditionGood: 70,
    });
  });
  it("mogućnosti odjela: opterećenje 180 (granica iznad koje kvaliteta pada) je 100 %", () => {
    expect(staffUsePct(180)).toBe(100);
    expect(staffUsePct(190)).toBe(106);
    expect(staffUsePct(108)).toBe(60);
    expect(staffUsePct(0)).toBe(0);
  });
});

describe("opterećenje", () => {
  it("iznad 100 % mogućnosti: premalo ljudi, s prijedlogom", () => {
    const x = insights(round({ load_hundredths: 181 }), true);
    expect(x.weak[0]).toEqual({
      text: "Premalo ljudi: odjel radi na 101 % svojih mogućnosti",
      suggestion: "Prijedlog: zaposli više sezonskih radnika u ovom odjelu.",
    });
  });
  it("točno 100 % i točno 60 %: broj ljudi odgovara poslu", () => {
    expect(texts(round({ load_hundredths: 180 })).strong).toContain(
      "Broj ljudi odgovara poslu: odjel radi na 100 % svojih mogućnosti",
    );
    expect(texts(round({ load_hundredths: 108 })).strong).toContain(
      "Broj ljudi odgovara poslu: odjel radi na 60 % svojih mogućnosti",
    );
  });
  it("ispod 60 %: previše ljudi", () => {
    const x = texts(round({ load_hundredths: 107 }));
    expect(x.weak).toContain("Previše ljudi za ovoliko gostiju (59 %)");
    expect(x.strong.some((s) => s.startsWith("Broj ljudi"))).toBe(false);
  });
  it("odjel bez gostiju po korištenju (najam, zatvoren): bez pravila o ljudima i kapacitetu", () => {
    const x = texts(round({ load_hundredths: 0, served: 0, demand: 0 }));
    expect([...x.weak, ...x.strong].filter((s) => /ljudi|gost(?!iju palo)|posluženi/i.test(s))).toEqual([]);
  });
});

describe("kapacitet", () => {
  it("posluženo manje od potražnje: odbijeni gosti", () => {
    expect(texts(round({ served: 2000, demand: 2100 })).weak).toContain(
      "Odbijeno 100 gostiju jer nije bilo kapaciteta",
    );
    expect(texts(round({ served: 2099, demand: 2100 })).weak).toContain(
      "Odbijen 1 gost jer nije bilo kapaciteta",
    );
  });
  it("posluženi svi: dobra strana s brojem", () => {
    expect(texts(round({ served: 2100, demand: 2100 })).strong).toContain(
      "Posluženi su svi gosti koji su htjeli doći (2.100 korištenja)",
    );
  });
});

describe("stanje prostora", () => {
  it("ispod 50/100: dotrajao, s prijedlogom održavanja", () => {
    const x = insights(round({ condition_after_pts: 4949 }), true);
    expect(x.weak).toContainEqual({
      text: "Prostor je dotrajao (49/100)",
      suggestion: "Prijedlog: pojačano održavanje odjela.",
    });
  });
  it("50–69/100: ni slaba ni jaka strana", () => {
    for (const pts of [4950, 6949]) {
      const x = texts(round({ condition_after_pts: pts }));
      expect([...x.weak, ...x.strong].filter((s) => s.startsWith("Prostor"))).toEqual([]);
    }
  });
  it("od 70/100: dobro stanje", () => {
    expect(texts(round({ condition_after_pts: 6950 })).strong).toContain(
      "Prostor je u dobrom stanju (70/100)",
    );
  });
});

describe("zadovoljstvo gostiju", () => {
  it("pad i rast po prikazanim bodovima", () => {
    expect(texts(round({ satisfaction_before_pts: 7000, satisfaction_after_pts: 6949 })).weak).toContain(
      "Zadovoljstvo gostiju palo s 70/100 na 69/100",
    );
    expect(texts(round({ satisfaction_before_pts: 6949, satisfaction_after_pts: 7000 })).strong).toContain(
      "Zadovoljstvo gostiju poraslo s 69/100 na 70/100",
    );
  });
  it("isti prikazani broj: ništa", () => {
    const x = texts(round({ satisfaction_before_pts: 7000, satisfaction_after_pts: 6960 }));
    expect([...x.weak, ...x.strong].filter((s) => s.startsWith("Zadovoljstvo"))).toEqual([]);
  });
});

describe("rezultat", () => {
  it("gubitak i dobit s iznosom; tjedan i mjesec", () => {
    expect(texts(round({ profit_cents: -80_000 })).weak).toContain("Odjel je ovaj tjedan u gubitku (−800 €)");
    expect(texts(round({ profit_cents: -80_000 }), false).weak).toContain(
      "Odjel je ovaj mjesec u gubitku (−800 €)",
    );
    expect(texts(round({ profit_cents: 789_600 })).strong).toContain(
      "Odjel je ovaj tjedan u dobiti (+7.896 €)",
    );
  });
  it("ispod pola eura: ništa", () => {
    const x = texts(round({ profit_cents: 40 }));
    expect([...x.weak, ...x.strong].filter((s) => s.startsWith("Odjel je"))).toEqual([]);
  });
});
