### Task 1: Tekstovi, formatiranje i logika igre (čisto, bez Reacta)

**Files:**
- Create: `apps/web/src/api/types.ts`, `apps/web/src/game.ts`, `apps/web/src/game.test.ts`, `apps/web/src/format.test.ts`
- Modify: `apps/web/src/format.ts`, `apps/web/src/App.test.tsx` (makni `describe("formatIndex")` — seli u `format.test.ts`),
  `packages/i18n/src/hr.ts`

**Interfaces:**
- Produces (`api/types.ts`): `type S = components["schemas"]` i aliasi `HotelBody, PublicHotelBody, PrivateHotelBody,
  LastMonthBody, MonthScoreBody, GameBody, RoundBody, SituationsBody, SituationBody, ConversationBody, ResponseBody,
  CriterionBody, PlanBody, PlanLimitsBody, FeedItemBody, IndexEntry, IndexCode, MonthCode, CauseCode, VarCode, UnitCode,
  StrategyCode` (svaki `= S["<Ime>"]`).
- Produces (`format.ts`): `formatIndex(h)`, `formatSigned(h)`, `formatEuros(cents)`, `formatSignedEuros(cents)`,
  `formatRating(h)`, `formatOccupancy(sold, available)`, `formatChange(amount, unit)`, `monthName(code)`, `capitalize(s)`,
  `causeLabel(code)`, `varLabel(code)`, `strategyLabel(code)` — svi vraćaju `string`.
- Produces (`game.ts`): `INDEX_ORDER`, `sortIndices`, `indicesAt`, `latestIndices`, `indicesBefore`, `indexDeltas`,
  `progressAt`, `leaderboard` + `LeaderRow`, `monthPosition`, `nextMonth` (potpisi niže).

- [ ] **Step 1: Napiši `api/types.ts`**

```ts
// Aliasi generiranih tipova API-ja; ručno se ništa ne opisuje.
import type { components } from "@hotelixsim/api-client";

type S = components["schemas"];

export type HotelBody = S["HotelBody"];
export type PublicHotelBody = S["PublicHotelBody"];
export type PrivateHotelBody = S["PrivateHotelBody"];
export type LastMonthBody = S["LastMonthBody"];
export type MonthScoreBody = S["MonthScoreBody"];
export type GameBody = S["GameBody"];
export type RoundBody = S["RoundBody"];
export type SituationsBody = S["SituationsBody"];
export type SituationBody = S["SituationBody"];
export type ConversationBody = S["ConversationBody"];
export type ResponseBody = S["ResponseBody"];
export type CriterionBody = S["CriterionBody"];
export type PlanBody = S["PlanBody"];
export type PlanLimitsBody = S["PlanLimitsBody"];
export type FeedItemBody = S["FeedItemBody"];
export type IndexEntry = S["IndexEntry"];
export type IndexCode = S["IndexCode"];
export type MonthCode = S["MonthCode"];
export type CauseCode = S["CauseCode"];
export type VarCode = S["VarCode"];
export type UnitCode = S["UnitCode"];
export type StrategyCode = S["StrategyCode"];
```

- [ ] **Step 2: Dodaj ključeve u `packages/i18n/src/hr.ts`** (ispod postojećih, prije `} as const;`)

```ts
  "month.jun": "lipanj",
  "month.jul": "srpanj",
  "month.aug": "kolovoz",
  "month.sep": "rujan",
  "var.sold": "Prodane noći",
  "var.revenue": "Prihod",
  "var.costs": "Troškovi",
  "var.cash": "Blagajna",
  "var.staff": "Zaposleni",
  "var.morale": "Moral",
  "var.experience": "Iskustvo",
  "var.equipment": "Oprema",
  "var.satisfaction": "Zadovoljstvo gostiju",
  "var.rating": "Ocjena na portalima",
  "cause.baseline": "osnova",
  "cause.price": "cijena",
  "cause.reputation": "ugled",
  "cause.marketing": "marketing",
  "cause.groups": "grupe",
  "cause.capacity": "nema slobodnih soba",
  "cause.other_revenue": "ostali prihodi",
  "cause.wage": "plaće",
  "cause.load": "opterećenje osoblja",
  "cause.training": "obuka",
  "cause.hires": "zapošljavanje",
  "cause.release": "otpuštanje",
  "cause.turnover": "odlasci zaposlenih",
  "cause.maintenance": "održavanje",
  "cause.renovation": "obnova",
  "cause.reply_reviews": "odgovori na recenzije",
  "cause.fixed_costs": "fiksni troškovi",
  "cause.interest": "kamate",
  "cause.situation": "situacija",
  "unit.room_nights": "noćenja",
  "unit.people": "ljudi",
  "unit.points": "bodova",
  "unit.rating_hundredths": "ocjene",
  "strategy.stedisa": "štedi",
  "strategy.brizni": "brine o ljudima",
  "strategy.lovac": "lovi popunjenost",
  "strategy.nasumicni": "igra nasumično",
```

- [ ] **Step 3: Napiši pad testova `apps/web/src/format.test.ts`**

```ts
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
```

- [ ] **Step 4: Pokreni i vidi pad**

Run: `cd /c/Users/leonk/Documents/hotelixsim && corepack pnpm --filter @hotelixsim/web exec vitest run src/format.test.ts; echo EXIT=$?`
Expected: FAIL — `formatSigned` (i ostali) nisu izvezeni.

- [ ] **Step 5: Napiši `apps/web/src/format.ts`**

```ts
import { t } from "@hotelixsim/i18n";
import type { CauseCode, MonthCode, StrategyCode, UnitCode, VarCode } from "./api/types";

const nf2 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });
const nf0 = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 0 });
const MINUS = "−";

/** Indeks u stotinkama (0–10000) → tekst za prikaz, npr. 5025 → "50,25". */
export function formatIndex(hundredths: number): string {
  return nf2.format(hundredths / 100);
}

function signed(value: number, abs: string): string {
  if (value === 0) return abs;
  return value > 0 ? `+${abs}` : `${MINUS}${abs}`;
}

/** Stotinke s predznakom: 825 → "+8,25", -200 → "−2", 0 → "0". */
export function formatSigned(hundredths: number): string {
  return signed(hundredths, nf2.format(Math.abs(hundredths) / 100));
}

/** Centi → cijeli euri: 9 800 000 → "98.000 €". */
export function formatEuros(cents: number): string {
  const euros = Math.round(cents / 100);
  return `${euros < 0 ? MINUS : ""}${nf0.format(Math.abs(euros))} €`;
}

export function formatSignedEuros(cents: number): string {
  return signed(cents, formatEuros(Math.abs(cents)));
}

/** Ocjena u stotinkama na ljestvici do 10: 840 → "8,4/10". */
export function formatRating(hundredths: number): string {
  return `${nf2.format(hundredths / 100)}/10`;
}

export function formatOccupancy(sold: number, available: number): string {
  if (available <= 0) return "—";
  return `${Math.round((sold * 100) / available)} %`;
}

/** Iznos promjene s jedinicom iz obračuna (feed `change`). */
export function formatChange(amount: number, unit: UnitCode): string {
  switch (unit) {
    case "cents":
      return formatSignedEuros(amount);
    case "room_nights":
      return `${signed(amount, nf0.format(Math.abs(amount)))} ${t("unit.room_nights")}`;
    case "people":
      return `${signed(amount, nf0.format(Math.abs(amount)))} ${t("unit.people")}`;
    case "points":
      return `${formatSigned(amount)} ${t("unit.points")}`;
    case "rating_hundredths":
      return `${formatSigned(amount)} ${t("unit.rating_hundredths")}`;
    case "bp":
      return `${formatSigned(amount)} %`;
  }
}

export function monthName(code: MonthCode): string {
  return t(`month.${code}` as const);
}

export function capitalize(s: string): string {
  return s.charAt(0).toLocaleUpperCase("hr-HR") + s.slice(1);
}

export function causeLabel(code: CauseCode): string {
  return t(`cause.${code}` as const);
}

export function varLabel(code: VarCode): string {
  return t(`var.${code}` as const);
}

export function strategyLabel(code: StrategyCode): string {
  return t(`strategy.${code}` as const);
}
```

Napomena: `hr-HR` u Node/ICU grupira tisuće točkom tek od 5 znamenki ovisno o ICU-u; testovi zato provjeravaju 5+
znamenki i „132 €“. Ako `formatEuros(9_800_000)` u tvojem okruženju ne da `98.000 €`, NE mijenjaj test nego javi.

- [ ] **Step 6: Pokreni format testove** — `… vitest run src/format.test.ts; echo EXIT=$?` → PASS.

- [ ] **Step 7: Napiši pad testova `apps/web/src/game.test.ts`**

```ts
import { describe, expect, it } from "vitest";
import type { GameBody, IndexEntry, PublicHotelBody } from "./api/types";
import {
  indexDeltas,
  indicesBefore,
  latestIndices,
  leaderboard,
  monthPosition,
  nextMonth,
  progressAt,
  sortIndices,
} from "./game";

const ix = (g: number, r: number, z: number, o: number, f: number): IndexEntry[] => [
  { code: "G", hundredths: g },
  { code: "R", hundredths: r },
  { code: "Z", hundredths: z },
  { code: "O", hundredths: o },
  { code: "F", hundredths: f },
];

function hotel(id: string, slot: number, months: PublicHotelBody["months"]): PublicHotelBody {
  return {
    hotel_id: id,
    slot,
    name: id === "h1" ? "Tvoj hotel" : `Robot ${slot}`,
    seat: id === "h1" ? { kind: "human", strategy: null } : { kind: "robot", strategy: "stedisa" },
    start: ix(5000, 5000, 5000, 5000, 5000),
    months,
  };
}

const jun = { month: "jun" as const, indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 };
const jul = { month: "jul" as const, indices: ix(5600, 5200, 5000, 5400, 5000), progress_hundredths: 1100 };

describe("indeksi", () => {
  it("sortIndices slaže G, R, Z, O, F", () => {
    const shuffled: IndexEntry[] = [
      { code: "F", hundredths: 1 },
      { code: "G", hundredths: 2 },
      { code: "O", hundredths: 3 },
      { code: "R", hundredths: 4 },
      { code: "Z", hundredths: 5 },
    ];
    expect(sortIndices(shuffled).map((i) => i.code)).toEqual(["G", "R", "Z", "O", "F"]);
  });
  it("latestIndices je zadnji obračunati mjesec ili početak", () => {
    expect(latestIndices(hotel("h1", 1, []))).toEqual(ix(5000, 5000, 5000, 5000, 5000));
    expect(latestIndices(hotel("h1", 1, [jun, jul]))).toEqual(jul.indices);
  });
  it("indicesBefore je prethodni mjesec, a za prvi mjesec početak", () => {
    const h = hotel("h1", 1, [jun, jul]);
    expect(indicesBefore(h, ["jun", "jul"], "jun")).toEqual(h.start);
    expect(indicesBefore(h, ["jun", "jul"], "jul")).toEqual(jun.indices);
  });
  it("indexDeltas daje razliku po kodu", () => {
    const d = indexDeltas(ix(5000, 5000, 5000, 5000, 5000), jun.indices);
    expect(d.get("G")).toBe(300);
    expect(d.get("F")).toBe(-200);
  });
});

describe("napredak i ljestvica", () => {
  const game: GameBody = {
    game_id: "g1",
    months: ["jun", "jul"],
    current: { month: "jul", status: "open", unanswered: 1 },
    my_hotel_id: "h1",
    hotels: [
      hotel("h1", 1, [jun]),
      hotel("h2", 2, [{ ...jun, progress_hundredths: 600 }]),
      hotel("h3", 3, [{ ...jun, progress_hundredths: 800 }]),
      hotel("h4", 4, []),
    ],
  };
  it("progressAt vraća null za neobračunat mjesec", () => {
    expect(progressAt(game.hotels[0] as PublicHotelBody, "jun")).toBe(800);
    expect(progressAt(game.hotels[3] as PublicHotelBody, "jun")).toBeNull();
  });
  it("leaderboard slaže po napretku, jednake po mjestu, i označi moj hotel", () => {
    const rows = leaderboard(game, "jun");
    expect(rows.map((r) => r.hotelId)).toEqual(["h1", "h3", "h2"]);
    expect(rows[0]?.mine).toBe(true);
    expect(rows[1]?.strategy).toBe("stedisa");
  });
  it("monthPosition daje mjesec, redni broj i ukupno; null na kraju sezone", () => {
    expect(monthPosition(game)).toEqual({ month: "jul", index: 1, total: 2 });
    expect(monthPosition({ ...game, current: null })).toBeNull();
  });
  it("nextMonth daje sljedeći mjesec igre ili null", () => {
    expect(nextMonth(["jun", "jul"], "jun")).toBe("jul");
    expect(nextMonth(["jun", "jul"], "jul")).toBeNull();
  });
});
```

- [ ] **Step 8: Pokreni i vidi pad** — `… vitest run src/game.test.ts; echo EXIT=$?` → FAIL (nema `./game`).

- [ ] **Step 9: Napiši `apps/web/src/game.ts`**

```ts
// Čista logika igre za ekrane: indeksi, promjene, ljestvica. Bez Reacta i mreže.
import type { GameBody, IndexCode, IndexEntry, MonthCode, PublicHotelBody, StrategyCode } from "./api/types";

export const INDEX_ORDER: readonly IndexCode[] = ["G", "R", "Z", "O", "F"];

export function sortIndices(xs: IndexEntry[]): IndexEntry[] {
  return [...xs].sort((a, b) => INDEX_ORDER.indexOf(a.code) - INDEX_ORDER.indexOf(b.code));
}

export function indicesAt(h: PublicHotelBody, month: MonthCode): IndexEntry[] | null {
  return h.months.find((m) => m.month === month)?.indices ?? null;
}

/** Zadnji obračunati mjesec, a prije prvog obračuna početno stanje. */
export function latestIndices(h: PublicHotelBody): IndexEntry[] {
  return h.months.at(-1)?.indices ?? h.start;
}

/** Indeksi prije zadanog mjeseca: prethodni mjesec igre ili početak. */
export function indicesBefore(h: PublicHotelBody, months: MonthCode[], month: MonthCode): IndexEntry[] {
  const i = months.indexOf(month);
  const prev = i > 0 ? months[i - 1] : undefined;
  return (prev && indicesAt(h, prev)) || h.start;
}

export function indexDeltas(from: IndexEntry[], to: IndexEntry[]): Map<IndexCode, number> {
  const before = new Map(from.map((e) => [e.code, e.hundredths]));
  return new Map(to.map((e) => [e.code, e.hundredths - (before.get(e.code) ?? e.hundredths)]));
}

export function progressAt(h: PublicHotelBody, month: MonthCode): number | null {
  return h.months.find((m) => m.month === month)?.progress_hundredths ?? null;
}

export type LeaderRow = {
  hotelId: string;
  name: string;
  mine: boolean;
  strategy: StrategyCode | null;
  progress: number;
};

/** Ljestvica napretka za mjesec: samo obračunati hoteli, najbolji prvi, jednaki po mjestu. */
export function leaderboard(game: GameBody, month: MonthCode): LeaderRow[] {
  const rows: (LeaderRow & { slot: number })[] = [];
  for (const h of game.hotels) {
    const progress = progressAt(h, month);
    if (progress === null) continue;
    rows.push({
      hotelId: h.hotel_id,
      name: h.name,
      mine: h.hotel_id === game.my_hotel_id,
      strategy: h.seat.strategy,
      progress,
      slot: h.slot,
    });
  }
  rows.sort((a, b) => b.progress - a.progress || a.slot - b.slot);
  return rows.map(({ slot: _slot, ...row }) => row);
}

export function monthPosition(game: GameBody): { month: MonthCode; index: number; total: number } | null {
  if (!game.current) return null;
  return {
    month: game.current.month,
    index: game.months.indexOf(game.current.month),
    total: game.months.length,
  };
}

export function nextMonth(months: MonthCode[], month: MonthCode): MonthCode | null {
  return months[months.indexOf(month) + 1] ?? null;
}
```

- [ ] **Step 10: Makni `describe("formatIndex")` i njegov import iz `App.test.tsx`** (preselio je u `format.test.ts`).

- [ ] **Step 11: Provjere** — sve četiri iz Global Constraints, svaka s `echo EXIT=$?`. Expected: EXIT=0, novi testovi prolaze.

- [ ] **Step 12: Commit** — `git add apps/web/src/api/types.ts apps/web/src/format.ts apps/web/src/format.test.ts
  apps/web/src/game.ts apps/web/src/game.test.ts apps/web/src/App.test.tsx packages/i18n/src/hr.ts` → „Ekrani 1/11:
  tekstovi, formatiranje i logika indeksa i ljestvice".

---

