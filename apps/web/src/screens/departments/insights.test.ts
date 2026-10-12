import { describe, expect, it } from "vitest";
import type { DeptRoundBody } from "../../api/types";
import { departmentsBody } from "../../test/fixtures";
import { type DeptKind, INSIGHT_LIMITS, insights, staffUsePct } from "./insights";

/** Restoran iz t01, s neutralnim vrijednostima: nijedno pravilo ne pali dok ga test ne pomakne. */
function round(over: Partial<DeptRoundBody> = {}): DeptRoundBody {
  const last = departmentsBody().departments.find((d) => d.code === "restaurant")?.last;
  if (!last) throw new Error("restoran bez kruga");
  return {
    ...last,
    load_hundredths: 144, // 80 % granice za dobru uslugu
    served: 2000,
    demand: 2000,
    condition_after_pts: 6000,
    satisfaction_before_pts: 7000,
    satisfaction_after_pts: 7000,
    profit_cents: 0,
    payroll_cents: 116_666,
    ...over,
  };
}

const RESTAURANT: DeptKind = { class: "food_beverage", open: true };
const ROOMS: DeptKind = { class: "rooms", open: true };

const texts = (r: DeptRoundBody, weekly = true, dept: DeptKind = RESTAURANT) => {
  const x = insights(dept, r, weekly);
  return { weak: x.weak.map((i) => i.text), strong: x.strong.map((i) => i.text) };
};

describe("pragovi u jednoj tablici", () => {
  it("vrijednosti pragova", () => {
    expect(INSIGHT_LIMITS).toEqual({
      fullLoadHundredths: 180,
      worstLoadHundredths: 240,
      overloadedPct: 100,
      idlePct: 60,
      conditionWorn: 50,
      conditionGood: 70,
      satisfactionStep: 2,
    });
  });
  it("granica za dobru uslugu: opterećenje 180 (iznad nje jezgra kažnjava kvalitetu) je 100 %", () => {
    expect(staffUsePct(180)).toBe(100);
    expect(staffUsePct(190)).toBe(106);
    expect(staffUsePct(108)).toBe(60);
    expect(staffUsePct(0)).toBe(0);
  });
});

describe("opterećenje (ista mjera kao kartica Ljudi i ocjena)", () => {
  it("iznad 100 % granice: osoblje je preopterećeno, s prijedlogom", () => {
    const x = insights(RESTAURANT, round({ load_hundredths: 181 }), true);
    expect(x.weak[0]).toEqual({
      text: "Osoblje je preopterećeno: 101 % granice za dobru uslugu, kvaliteta usluge pada",
      suggestion: "Prijedlog: zaposli više sezonskih radnika u ovom odjelu.",
    });
  });
  it("točno 100 % i točno 60 %: broj ljudi odgovara poslu", () => {
    expect(texts(round({ load_hundredths: 180 })).strong).toContain(
      "Broj ljudi odgovara poslu: osoblje radi na 100 % granice za dobru uslugu",
    );
    expect(texts(round({ load_hundredths: 108 })).strong).toContain(
      "Broj ljudi odgovara poslu: osoblje radi na 60 % granice za dobru uslugu",
    );
  });
  it("ispod 60 %: previše ljudi", () => {
    const x = texts(round({ load_hundredths: 107 }));
    expect(x.weak).toContain(
      "Previše ljudi za ovoliko gostiju: osoblje radi na 59 % granice za dobru uslugu",
    );
    expect(x.strong.some((s) => s.startsWith("Broj ljudi"))).toBe(false);
  });
  it("odjel bez gostiju po korištenju (najam): bez pravila o ljudima i kapacitetu", () => {
    const x = texts(round({ load_hundredths: 0, served: 0, demand: 0 }), true, {
      class: "rentals",
      open: true,
    });
    expect(
      [...x.weak, ...x.strong].filter((s) => /ljudi|osoblje|gost(?!iju palo)|posluženi/i.test(s)),
    ).toEqual([]);
  });
});

describe("kapacitet: korištenja, za Smještaj noćenja", () => {
  it("posluženo manje od potražnje: odbijena korištenja", () => {
    expect(texts(round({ served: 2000, demand: 2100 })).weak).toContain(
      "Odbijeno 100 korištenja jer nije bilo kapaciteta",
    );
    expect(texts(round({ served: 2099, demand: 2100 })).weak).toContain(
      "Odbijeno 1 korištenje jer nije bilo kapaciteta",
    );
    expect(texts(round({ served: 2098, demand: 2100 })).weak).toContain(
      "Odbijena 2 korištenja jer nije bilo kapaciteta",
    );
  });
  it("Smještaj broji noćenja", () => {
    expect(texts(round({ served: 600, demand: 605 }), true, ROOMS).weak).toContain(
      "Odbijeno 5 noćenja jer nije bilo kapaciteta",
    );
    expect(texts(round({ served: 605, demand: 605 }), true, ROOMS).strong).toContain(
      "Posluženi su svi gosti koji su htjeli doći (605 noćenja)",
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
    const x = insights(RESTAURANT, round({ condition_after_pts: 4949 }), true);
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

describe("zadovoljstvo gostiju (promjena od 2 prikazana boda naviše)", () => {
  it("pad i rast za 2 boda", () => {
    expect(texts(round({ satisfaction_before_pts: 7000, satisfaction_after_pts: 6849 })).weak).toContain(
      "Zadovoljstvo gostiju palo s 70/100 na 68/100",
    );
    expect(texts(round({ satisfaction_before_pts: 6849, satisfaction_after_pts: 7000 })).strong).toContain(
      "Zadovoljstvo gostiju poraslo s 68/100 na 70/100",
    );
  });
  it("promjena za 1 prikazani bod ili manje: ništa", () => {
    for (const [before, after] of [
      [7000, 6949],
      [6949, 7000],
      [7000, 6960],
    ] as const) {
      const x = texts(round({ satisfaction_before_pts: before, satisfaction_after_pts: after }));
      expect([...x.weak, ...x.strong].filter((s) => s.startsWith("Zadovoljstvo"))).toEqual([]);
    }
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

describe("zatvoren odjel (M2)", () => {
  const closed = round({ served: 0, demand: 0, load_hundredths: 0, profit_cents: -116_666 });
  it("razlog gubitka: zatvoren je, a stalni radnici se i dalje plaćaju", () => {
    const x = texts(closed, true, { class: "other_operated", open: false });
    expect(x.weak).toEqual([
      "Odjel je zatvoren: ne prima goste, a stalni radnici se i dalje plaćaju (plaće 1.167 € ovaj tjedan)",
    ]);
    expect(x.strong).toEqual([]);
  });
  it("prostor se i dalje prati", () => {
    const x = texts({ ...closed, condition_after_pts: 4000 }, false, {
      class: "other_operated",
      open: false,
    });
    expect(x.weak).toEqual([
      "Odjel je zatvoren: ne prima goste, a stalni radnici se i dalje plaćaju (plaće 1.167 € ovaj mjesec)",
      "Prostor je dotrajao (40/100)",
    ]);
  });
});
