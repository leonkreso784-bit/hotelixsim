import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  BuildingBody,
  DepartmentsBody,
  HotelBody,
  PlBody,
  RenewalLimitsBody,
  RenewalOptionBody,
  RenewalStateBody,
} from "../../api/types";
import { json } from "../../test/fakeApi";
import {
  departmentsBody,
  game,
  HOTEL_ID,
  plBody,
  standardApi,
  v2Api,
  v2PlanHotel,
} from "../../test/fixtures";
import { renderApp } from "../../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/**
 * Izbori obnove 0, korak … najviše (kao poslužitelj): rata = iznos / broj krugova mjeseca (ostatak u zadnjem), učinak
 * 21 bod za 40.000 €, amortizacija kroz `life` mjeseci. Blagajna fixturea je 42.000 €, tjedne plaće 30.000 €.
 */
function renewal(
  opts: { max?: number; roundsLeft?: number; condition?: number; locked?: RenewalStateBody | null } = {},
): RenewalLimitsBody {
  const max = opts.max ?? 5_000_000;
  const step = 500_000;
  const n = opts.roundsLeft ?? 4;
  const condition = opts.condition ?? 6400;
  const options: RenewalOptionBody[] = [];
  for (let a = 0; a <= max; a += step) {
    const installment = Math.floor(a / n);
    const gain = Math.round((a * 2100) / 4_000_000);
    options.push({
      amount_cents: a,
      installment_cents: installment,
      last_installment_cents: a - installment * (n - 1),
      gain_pts: gain,
      gain_capped_pts: Math.min(gain, 10_000 - condition),
      depreciation_monthly_cents: Math.round(a / 84),
    });
  }
  return {
    month: "jun",
    rounds_left: n,
    max_monthly_cents: max,
    step_cents: step,
    life_months: 84,
    condition_pts: condition,
    locked: opts.locked ?? null,
    options,
  };
}

const LOCKED: RenewalStateBody = {
  month: "jun",
  total_cents: 4_000_000,
  installment_cents: 1_000_000,
  paid_cents: 2_000_000,
  rounds_left: 2,
  gain_pts: 2100,
  gained_pts: 1050,
};

const BUILDING: BuildingBody = {
  condition_pts: 7200,
  start_pts: 7400,
  wear_pts_per_month: 50,
  locked: null,
  invested_cents: 4_000_000,
};

/** v2 hotel s obnovom: restoran i zgrada imaju izbor, stara kvačica nije dostupna. */
function renewalHotel(
  opts: { building?: number; restaurant?: RenewalLimitsBody; monthly?: boolean } = {},
): HotelBody {
  const h = v2PlanHotel();
  const p = h.private;
  const s = p?.plan;
  if (!p || !s?.limits.departments?.restaurant || !s.plan.departments) throw new Error("nema plana");
  p.building = BUILDING;
  s.limits.renovation_available = false;
  s.limits.departments.restaurant.renewal = opts.restaurant ?? renewal({ roundsLeft: opts.monthly ? 1 : 4 });
  s.limits.costs.building_renewal = renewal({
    max: 10_000_000,
    condition: 7200,
    roundsLeft: opts.monthly ? 1 : 4,
  });
  s.limits.costs.renewal_guide = {
    reserve_bp: 400,
    season_revenue_budget_cents: 250_000_000,
    guide_cents: 10_000_000,
    invested_cents: 4_000_000,
  };
  s.plan.building_renewal_cents = opts.building ?? 0;
  for (const d of Object.values(s.plan.departments)) d.renewal_cents = 0;
  if (opts.monthly) {
    s.round = "jun";
    s.month = "jun";
  }
  return h;
}

/** v2 igra po mjesecima (lipanj otvoren). */
const monthlyV2 = () => ({ ...game(), model: "v2" as const });

function api(hotel: HotelBody, opts: { monthly?: boolean; departments?: DepartmentsBody } = {}) {
  let a = v2Api()
    .on("GET", "/v1/me/hotel", hotel)
    .on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));
  if (opts.monthly) a = a.on("GET", "/v1/games/:game", monthlyV2());
  if (opts.departments) {
    const body = opts.departments;
    a = a
      .on("GET", "/v1/hotels/:hotel/departments", body)
      .on("GET", "/v1/hotels/:hotel/departments/:code", (_r, params) => {
        const department = body.departments.find((d) => d.code === params.code);
        return json(200, { department, history: department?.last ? [department.last] : [] });
      });
  }
  return a;
}

async function restaurant() {
  return await screen.findByRole("group", { name: "Restoran" }, { timeout: 3000 });
}

function more(times: number) {
  const b = screen.getByRole("button", { name: "Više za obnovu" });
  for (let i = 0; i < times; i++) fireEvent.click(b);
}

describe("Obnova opreme u odjelu", () => {
  it("izbor iznosa: rata tjedno, učinak „s 64 na 85/100“, blagajna nakon i amortizacija", async () => {
    renderApp(api(renewalHotel()), "/odjeli/restaurant");
    const r = await restaurant();
    expect(
      within(r).getByText("Bez obnove ovaj mjesec. Najviše 50.000 € mjesečno, u koracima od 5.000 €."),
    ).toBeTruthy();
    more(8);
    expect(within(r).getByText("40.000 €")).toBeTruthy();
    expect(within(r).getByText("Plaćaš 10.000 € tjedno, 4 tjedna.")).toBeTruthy();
    expect(within(r).getByText("Stanje prostora +21 (s 64 na 85/100).")).toBeTruthy();
    expect(within(r).getByText("Blagajna sada 42.000 €, nakon ovog plaćanja 32.000 €.")).toBeTruthy();
    expect(within(r).getByText("Amortizacija 476 € mjesečno kroz 7 godina, od srpnja.")).toBeTruthy();
    expect(screen.queryByText(/^Pozor/)).toBeNull();
    // Najviše 50.000 €: dalje ne ide.
    more(2);
    expect(screen.getByRole("button", { name: "Više za obnovu" }).getAttribute("aria-disabled")).toBe("true");
  });

  it("upozorenje kad bi blagajna pala ispod tjedna plaća; zbraja i obnovu zgrade iz plana", async () => {
    // Zgrada 20.000 € (5.000 € tjedno) + restoran 40.000 € (10.000 € tjedno) = 15.000 € ovaj tjedan.
    renderApp(api(renewalHotel({ building: 2_000_000 })), "/odjeli/restaurant");
    const r = await restaurant();
    more(8);
    expect(within(r).getByText("Blagajna sada 42.000 €, nakon ovog plaćanja 27.000 €.")).toBeTruthy();
    const warning = within(r).getByText(
      "Pozor: nakon obnove u blagajni ostaje manje od jednog tjedna plaća (30.000 €).",
    );
    expect(warning.closest("[role=status]")).toBeTruthy();
  });

  it("mjesečna igra: „odjednom“; minus u blagajni je upozorenje", async () => {
    renderApp(api(renewalHotel({ monthly: true }), { monthly: true }), "/odjeli/restaurant");
    const r = await restaurant();
    more(8);
    expect(within(r).getByText("Plaćaš 40.000 € odjednom.")).toBeTruthy();
    expect(within(r).getByText("Blagajna sada 42.000 €, nakon ovog plaćanja 2.000 €.")).toBeTruthy();
    more(2);
    expect(within(r).getByText("Blagajna sada 42.000 €, nakon ovog plaćanja −8.000 €.")).toBeTruthy();
    expect(within(r).getByText("Pozor: nakon obnove blagajna bi bila u minusu.")).toBeTruthy();
  });

  it("spremanje: jedan PUT po kliku, s iznosom obnove restorana", async () => {
    const a = api(renewalHotel());
    renderApp(a, "/odjeli/restaurant");
    await restaurant();
    more(8);
    const save = screen.getByRole("button", { name: "Spremi odluke odjela" });
    fireEvent.click(save);
    fireEvent.click(save);
    expect(await screen.findByText("Odluke odjela su spremljene.")).toBeTruthy();
    const puts = a.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`);
    expect(puts).toHaveLength(1);
    const body = puts[0]?.body as { departments: Record<string, { renewal_cents: number }> };
    expect(body.departments.restaurant?.renewal_cents).toBe(4_000_000);
    expect(body.departments.lobby_bar?.renewal_cents).toBe(0);
  });

  it("zaključano: točan tekst u role=status i nema izbora; isti redak na kartici i u planu", async () => {
    const hotel = renewalHotel({ restaurant: renewal({ roundsLeft: 2, locked: LOCKED }) });
    const plan = hotel.private?.plan?.plan.departments?.restaurant;
    if (plan) plan.renewal_cents = 4_000_000;
    const body = departmentsBody();
    const rest = body.departments.find((d) => d.code === "restaurant");
    if (rest) rest.renewal = LOCKED;
    renderApp(api(hotel, { departments: body }), "/odjeli/restaurant");
    const r = await restaurant();
    const text =
      "Obnova je odlučena: 40.000 € u lipnju, 10.000 € tjedno, još 2 tjedna. Do kraja mjeseca se ne mijenja.";
    expect(within(r).getByText(text).getAttribute("role")).toBe("status");
    expect(screen.queryByRole("button", { name: /za obnovu/ })).toBeNull();
    const card = screen.getAllByRole("link", { name: "Restoran" })[0]?.closest("article") as HTMLElement;
    expect(within(card).getByText("Obnova: 40.000 € u lipnju, 10.000 € tjedno, još 2 tjedna")).toBeTruthy();
    cleanup();
    renderApp(api(hotel, { departments: body }), "/plan");
    expect(
      await screen.findByText(/^obnova 40\.000 € u lipnju, 10\.000 € tjedno, još 2 tjedna$/, undefined, {
        timeout: 3000,
      }),
    ).toBeTruthy();
  });

  it("odgođeno održavanje: rečenica da prostor propada brže", async () => {
    renderApp(api(renewalHotel()), "/odjeli/restaurant");
    const r = await restaurant();
    expect(screen.queryByText("Uz odgođeno održavanje prostor se troši brže nego inače.")).toBeNull();
    fireEvent.click(within(r).getByRole("radio", { name: "odgođeno" }));
    expect(within(r).getByText("Uz odgođeno održavanje prostor se troši brže nego inače.")).toBeTruthy();
  });

  it("lokal: bez cijene (100–100) i bez obnove, ali s pravim izborom održavanja", async () => {
    const hotel = renewalHotel();
    const s = hotel.private?.plan;
    if (!s?.limits.departments?.parking || !s.plan.departments) throw new Error("nema plana");
    s.plan.departments.parking = {
      open: true,
      seasonal_target: 0,
      price_level_pct: 100,
      maintenance: "regular",
      renewal_cents: 0,
    };
    Object.assign(s.limits.departments.parking, {
      seasonal_max: 0,
      can_close: false,
      price_level_min_pct: 100,
      price_level_max_pct: 100,
      maintenance: { deferred_cents: 1_000, regular_cents: 20_000, intensive_cents: 40_000 },
    });
    renderApp(api(hotel), "/odjeli/parking");
    const g = await screen.findByRole("group", { name: "Parking" }, { timeout: 3000 });
    expect(within(g).queryByText("Cijene u odjelu")).toBeNull();
    expect(within(g).queryByText("Obnova opreme")).toBeNull();
    expect(within(g).getAllByRole("radio")).toHaveLength(3);
    expect(screen.getByText(/^Najam ima ugovoreni prihod/)).toBeTruthy();
  });

  it("mobitel: dugi retci obnove se lome (nisu nowrap), stupac odluka smije se suziti", async () => {
    renderApp(api(renewalHotel({ building: 2_000_000 })), "/odjeli/restaurant");
    const r = await restaurant();
    more(10);
    expect(r.className).toContain("min-w-0");
    for (const text of [
      "Plaćaš 12.500 € tjedno, 4 tjedna.",
      "Pozor: nakon obnove u blagajni ostaje manje od jednog tjedna plaća (30.000 €).",
      "Amortizacija 595 € mjesečno kroz 7 godina, od srpnja.",
    ]) {
      let el: HTMLElement | null = within(r).getByText(text);
      while (el && el !== r) {
        expect(el.className, text).not.toMatch(/whitespace-nowrap|truncate/);
        el = el.parentElement;
      }
    }
  });
});

describe("Zgrada u planu", () => {
  it("traka zgrade, smjernica FF&E i izbor iznosa; stara kvačica skrivena", async () => {
    const a = api(renewalHotel());
    renderApp(a, "/plan");
    const card = (
      await screen.findByRole("heading", { level: 2, name: "Zgrada" }, { timeout: 3000 })
    ).closest("section") as HTMLElement;
    const z = within(card);
    expect(z.getByRole("img", { name: "Stanje zgrade: 72 od 100" })).toBeTruthy();
    expect(z.getByText("Troši se oko 0,5 boda mjesečno.")).toBeTruthy();
    expect(
      z.getByText("Preporučeno ulaganje u sezoni: oko 4 % prihoda ≈ 100.000 €. Do sada uloženo 40.000 €."),
    ).toBeTruthy();
    expect(z.getByText(/\(FF&E\)/)).toBeTruthy();
    expect(screen.queryByRole("checkbox", { name: "Obnova (jednom u sezoni)" })).toBeNull();
    fireEvent.click(z.getByRole("button", { name: "Više za obnovu" }));
    fireEvent.click(z.getByRole("button", { name: "Više za obnovu" }));
    expect(z.getByText("Plaćaš 2.500 € tjedno, 4 tjedna.")).toBeTruthy();
    expect(z.getByText("Stanje zgrade +5 (s 72 na 77/100).")).toBeTruthy();
    const save = screen.getByRole("button", { name: "Spremi plan" });
    fireEvent.click(save);
    fireEvent.click(save);
    await waitFor(() => expect(a.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const body = a.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as { building_renewal_cents: number };
    expect(body.building_renewal_cents).toBe(1_000_000);
  });

  it("v1 igra: stara kvačica obnove je tu, zgrade nema", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("checkbox", { name: "Obnova (jednom u sezoni)" })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Zgrada" })).toBeNull();
  });
});

describe("Zgrada na Mom hotelu", () => {
  it("redak „Stanje zgrade 72/100“; zgrada koja se ne troši i bez ulaganja se ne prikazuje", async () => {
    renderApp(api(renewalHotel()), "/");
    const row = await screen.findByRole("link", { name: /Stanje zgrade/ }, { timeout: 3000 });
    expect(row.textContent).toContain("72/100");
    cleanup();
    const h = renewalHotel();
    if (h.private) h.private.building = { ...BUILDING, wear_pts_per_month: 0, invested_cents: 0 };
    renderApp(api(h), "/");
    expect(await screen.findByRole("link", { name: /Ocjena na portalima/ }, { timeout: 3000 })).toBeTruthy();
    expect(screen.queryByRole("link", { name: /Stanje zgrade/ })).toBeNull();
  });
});

describe("Račun dobiti s obnovom", () => {
  function pl(): PlBody {
    const base = plBody();
    const line = (code: string, name: string, kind: "expense" | "result", a: number) => ({
      code,
      name,
      kind,
      actual_cents: a,
      budget_cents: 0,
      variance_cents: kind === "expense" ? a : a,
      favorable: kind !== "expense",
    });
    return {
      ...base,
      sections: [
        ...base.sections,
        {
          section: "depreciation",
          lines: [
            line("equipment", "Amortizacija opreme", "expense", 47_619),
            line("building", "Amortizacija zgrade", "expense", 0),
          ],
          total: line("total", "Ukupna amortizacija", "expense", 47_619),
        },
        {
          section: "ebit",
          lines: [],
          total: line("ebit", "EBIT (dobit iz poslovanja, prije kamata i poreza)", "result", 2_981_481),
        },
      ],
      ebit_margin_bp: { actual: 2535, budget: 2602 },
      capex_cents: { actual: 4_000_000, budget: 0 },
    };
  }

  it("retci amortizacije i EBIT ispod EBITDA, ulaganja ispod računa, EBIT marža s objašnjenjem", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", pl()), "/racun-dobiti");
    const table = await screen.findByRole("table", undefined, { timeout: 3000 });
    const groups = within(table)
      .getAllByRole("rowheader")
      .filter((h) => h.getAttribute("scope") === "rowgroup")
      .map((h) => h.textContent);
    expect(groups.slice(-3)).toEqual([
      "EBITDA",
      "Amortizacija (uloženo u obnovu, raspoređeno kroz godine)",
      "EBIT (dobit iz poslovanja, prije kamata i poreza)",
    ]);
    expect(within(table).getByRole("rowheader", { name: "Amortizacija opreme" })).toBeTruthy();
    expect(
      screen.getByText("Ulaganja u obnovu (plaćeno iz blagajne, nije trošak ove sezone): 40.000 €"),
    ).toBeTruthy();
    expect(screen.getByText("EBIT marža")).toBeTruthy();
    expect(
      screen.getByText(/^Koliki dio prihoda ostaje kao EBIT, a to je dobit iz poslovanja nakon amortizacije/),
    ).toBeTruthy();
  });

  it("v1/stari račun bez polja obnove: nema retka ulaganja ni EBIT marže", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", plBody()), "/racun-dobiti");
    expect(await screen.findByRole("table", undefined, { timeout: 3000 })).toBeTruthy();
    expect(screen.queryByText(/Ulaganja u obnovu/)).toBeNull();
    expect(screen.queryByText("EBIT marža")).toBeNull();
  });
});

describe("Financije (F) u v2", () => {
  it("opis kaže „dobit iz poslovanja nakon ulaganja“; v1 ostaje kakav je", async () => {
    renderApp(api(renewalHotel()), "/pokazatelj/F");
    expect(
      await screen.findByText(/^Kako stoje novac i dobit iz poslovanja nakon ulaganja/, undefined, {
        timeout: 3000,
      }),
    ).toBeTruthy();
    cleanup();
    renderApp(standardApi(), "/pokazatelj/F");
    expect(await screen.findByText("Kako stoje novac, prihodi i troškovi hotela.")).toBeTruthy();
  });
});
