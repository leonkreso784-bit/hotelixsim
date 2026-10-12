import { describe, expect, it } from "vitest";
import type { PlanStateBody, RenewalLimitsBody, RenewalOptionBody } from "../../api/types";
import {
  BUILDING,
  depreciationText,
  gainText,
  lockedPhrase,
  payText,
  planPayNow,
  stepAmount,
} from "./renewal";

const NBSP = / /g;
const plain = (s: string) => s.replace(NBSP, " ");

function option(amount: number, installment: number, last = installment): RenewalOptionBody {
  return {
    amount_cents: amount,
    installment_cents: installment,
    last_installment_cents: last,
    gain_pts: 2100,
    gain_capped_pts: 2100,
    depreciation_monthly_cents: Math.round(amount / 84),
  };
}

function limits(over: Partial<RenewalLimitsBody> = {}): RenewalLimitsBody {
  return {
    month: "jul",
    rounds_left: 4,
    max_monthly_cents: 1_000_000,
    step_cents: 500_000,
    life_months: 84,
    condition_pts: 6400,
    locked: null,
    options: [option(0, 0), option(500_000, 125_000), option(1_000_000, 250_000)],
    ...over,
  };
}

describe("obnova: tekstovi uz izbor", () => {
  it("rata: tjedno s brojem tjedana, ostatak u zadnjem tjednu, odjednom u jednom krugu", () => {
    expect(plain(payText(option(4_000_000, 1_000_000), 4))).toBe("Plaćaš 10.000 € tjedno, 4 tjedna.");
    expect(plain(payText(option(4_000_000, 1_333_333, 1_333_334), 3))).toBe(
      "Plaćaš 13.333 € tjedno, 3 tjedna; zadnji tjedan 13.333 €.",
    );
    expect(plain(payText(option(4_000_000, 4_000_000), 1))).toBe("Plaćaš 40.000 € odjednom.");
  });

  it("učinak: razlika prikazanih cijelih bodova, do granice 100", () => {
    expect(gainText("dept", 6400, 2100)).toBe("Stanje prostora +21 (s 64 na 85/100).");
    // 63,6 → 64 i 63,6 + 21,2 = 84,8 → 85: razlika +21, ne +21,2 zaokruženo odvojeno.
    expect(gainText("building", 6360, 2120)).toBe("Stanje zgrade +21 (s 64 na 85/100).");
  });

  it("amortizacija: godine kad je vijek djeljiv s 12, inače mjeseci; od idućeg mjeseca", () => {
    expect(plain(depreciationText(option(4_000_000, 1_000_000), limits()))).toBe(
      "Amortizacija 476 € mjesečno kroz 7 godina, od kolovoza.",
    );
    expect(
      plain(depreciationText(option(4_000_000, 1_000_000), limits({ life_months: 30, month: "sep" }))),
    ).toBe("Amortizacija 476 € mjesečno kroz 30 mjeseci, od listopada.");
  });

  it("zaključano: tjedno s brojem preostalih tjedana; zadnji tjedan ostatak; mjesečno „plaćeno“", () => {
    const s = {
      month: "jul" as const,
      total_cents: 4_000_000,
      installment_cents: 1_000_000,
      paid_cents: 2_000_000,
      rounds_left: 2,
      gain_pts: 2100,
      gained_pts: 1050,
    };
    expect(plain(lockedPhrase(s, true))).toBe("40.000 € u srpnju, 10.000 € tjedno, još 2 tjedna");
    expect(plain(lockedPhrase({ ...s, paid_cents: 3_000_000, rounds_left: 1 }, true))).toBe(
      "40.000 € u srpnju, ovaj tjedan zadnja rata 10.000 €",
    );
    expect(plain(lockedPhrase(s, false))).toBe("40.000 € u srpnju, plaćeno");
  });

  it("koraci: susjedni iznos ili ništa na rubu", () => {
    const l = limits();
    expect(stepAmount(l, 0, 1)).toBe(500_000);
    expect(stepAmount(l, 0, -1)).toBeNull();
    expect(stepAmount(l, 1_000_000, 1)).toBeNull();
  });
});

describe("obnova: rate ovog kruga za cijeli plan", () => {
  function state(): PlanStateBody {
    const dept = { open: true, seasonal_target: 0, price_level_pct: 100, maintenance: "regular" as const };
    const deptLimits = {
      seasonal_min: 0,
      seasonal_max: 0,
      price_level_min_pct: 100,
      price_level_max_pct: 100,
      can_close: false,
      maintenance: { deferred_cents: 0, regular_cents: 0, intensive_cents: 0 },
      seasonal_worker_cents: 0,
    };
    return {
      month: "jul",
      round: "t06",
      plan: {
        price_cents: 10_000,
        marketing_cents: 0,
        seasonal_target: 0,
        wage_pct: 100,
        training: "none",
        maintenance: "regular",
        renovation: false,
        reply_reviews: false,
        building_renewal_cents: 1_000_000,
        departments: {
          restaurant: { ...dept, renewal_cents: 500_000 },
          bar: { ...dept, renewal_cents: 0 },
          shop: dept,
        },
      },
      limits: {
        price_min_cents: 0,
        price_max_cents: 20_000,
        marketing_max_cents: 0,
        seasonal_min: 0,
        seasonal_max: 0,
        wage_min_pct: 90,
        wage_max_pct: 115,
        renovation_available: false,
        costs: {
          renovation_cents: 0,
          renovation_equipment_pts: 0,
          renovation_satisfaction_pts: 0,
          reply_cents: 0,
          training_basic: { cost_cents: 0, experience_pts: 0, morale_pts: 0 },
          training_full: { cost_cents: 0, experience_pts: 0, morale_pts: 0 },
          maintenance: { deferred_cents: 0, regular_cents: 0, intensive_cents: 0 },
          payroll_week_cents: 0,
          payroll_round_cents: 0,
          building_renewal: limits(),
        },
        departments: {
          restaurant: { ...deptLimits, renewal: limits() },
          bar: {
            ...deptLimits,
            renewal: limits({
              locked: {
                month: "jul",
                total_cents: 1_000_000,
                installment_cents: 250_000,
                paid_cents: 750_000,
                rounds_left: 1,
                gain_pts: 0,
                gained_pts: 0,
              },
            }),
          },
          shop: deptLimits,
        },
      },
    } as PlanStateBody;
  }

  it("zbraja odjele i zgradu; zaključana obnova plaća svoju ratu (zadnji tjedan ostatak)", () => {
    // Restoran 1.250 € + bar (zaključan, ostatak) 2.500 € + zgrada 2.500 €.
    expect(planPayNow(state())).toBe(125_000 + 250_000 + 250_000);
  });

  it("nespremljeni iznos odjela ili zgrade zamjenjuje spremljeni", () => {
    expect(planPayNow(state(), { code: "restaurant", amount: 0 })).toBe(250_000 + 250_000);
    expect(planPayNow(state(), { code: BUILDING, amount: 500_000 })).toBe(125_000 + 250_000 + 125_000);
  });
});
