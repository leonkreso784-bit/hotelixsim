// test/fixtures.ts — tipizirani primjerci odgovora API-ja (oblik iz openapi.json).
import type {
  ConversationBody,
  DepartmentBody,
  DepartmentDetailBody,
  DepartmentsBody,
  DeptRoundBody,
  ExplanationBody,
  GameBody,
  HotelBody,
  IndexEntry,
  KpiMonthBody,
  LastMonthBody,
  MonthScoreBody,
  PlanBody,
  PlanLimitsBody,
  PublicHotelBody,
  RoundBody,
  RoundInfoBody,
  ServiceRoundBody,
  SituationBody,
  SituationsBody,
  StrategyCode,
} from "../api/types";
import { FakeApi, json, problem } from "./fakeApi";

export const HOTEL_ID = "h1";
export const GAME_ID = "g1";

const MONTH_DAYS = { jun: 30, jul: 31, aug: 31, sep: 30 } as const;
const MONTH_INDEX = { jun: 0, jul: 1, aug: 2, sep: 3 } as const;

/** Kalendar mjesečnog kruga (v1): cijeli kalendarski mjesec. */
export function roundInfo(code: keyof typeof MONTH_DAYS): RoundBody["round"] {
  return {
    code,
    kind: "month",
    index: MONTH_INDEX[code],
    days: MONTH_DAYS[code],
    first_day: 1,
    first_month: code,
    last_day: MONTH_DAYS[code],
    last_month: code,
    month_ends: true,
  };
}

export function ix(g: number, r: number, z: number, o: number, f: number): IndexEntry[] {
  return [
    { code: "G", hundredths: g },
    { code: "R", hundredths: r },
    { code: "Z", hundredths: z },
    { code: "O", hundredths: o },
    { code: "F", hundredths: f },
  ];
}

export function plan(): PlanBody {
  return {
    price_cents: 13_200,
    marketing_cents: 0,
    seasonal_target: 28,
    wage_pct: 100,
    training: "none",
    maintenance: "regular",
    renovation: false,
    reply_reviews: false,
  };
}

export function limits(): PlanLimitsBody {
  return {
    price_min_cents: 7_900,
    price_max_cents: 19_800,
    marketing_max_cents: 4_000_000,
    seasonal_min: 18,
    seasonal_max: 38,
    wage_min_pct: 90,
    wage_max_pct: 115,
    renovation_available: true,
  };
}

export function myPublic(months: MonthScoreBody[] = []): PublicHotelBody {
  return {
    hotel_id: HOTEL_ID,
    slot: 1,
    name: "Tvoj hotel",
    seat: { kind: "human", strategy: null },
    start: ix(5000, 5000, 5000, 5000, 5000),
    months,
  };
}

export function robot(
  id: string,
  slot: number,
  name: string,
  strategy: StrategyCode,
  months: MonthScoreBody[] = [],
) {
  const h: PublicHotelBody = {
    hotel_id: id,
    slot,
    name,
    seat: { kind: "robot", strategy },
    start: ix(5000, 5000, 5000, 5000, 5000),
    months,
  };
  return h;
}

export function myHotel(
  opts: {
    months?: MonthScoreBody[];
    lastMonth?: LastMonthBody | null;
    planMonth?: "jun" | "jul" | null;
  } = {},
): HotelBody {
  const planMonth = opts.planMonth === undefined ? "jun" : opts.planMonth;
  return {
    game_id: GAME_ID,
    hotel: myPublic(opts.months ?? []),
    private: {
      profile: "C",
      now: {
        rooms: 120,
        permanent: 30,
        seasonal: 28,
        wage_pct: 100,
        morale_pts: 5500,
        experience_pts: 6000,
        equipment_pts: 6000,
        satisfaction_pts: 7000,
        rating: 840,
        cash_cents: 4_200_000,
        load_hundredths: 180,
      },
      last_month: opts.lastMonth ?? null,
      plan:
        planMonth === null ? null : { month: planMonth, round: planMonth, plan: plan(), limits: limits() },
    },
  };
}

export function game(opts: { current?: RoundBody | null; hotels?: PublicHotelBody[] } = {}): GameBody {
  return {
    game_id: GAME_ID,
    model: "v1",
    round_kind: "month",
    rounds: [roundInfo("jun"), roundInfo("jul")],
    months: ["jun", "jul"],
    current:
      opts.current === undefined
        ? { month: "jun", round: roundInfo("jun"), status: "open", unanswered: 2 }
        : opts.current,
    my_hotel_id: HOTEL_ID,
    hotels: opts.hotels ?? [
      myPublic(),
      robot("h2", 2, "Štediša", "stedisa"),
      robot("h3", 3, "Brižni", "brizni"),
      robot("h4", 4, "Lovac na popunjenost", "lovac"),
    ],
  };
}

export function s1(over: Partial<SituationBody> = {}): SituationBody {
  return {
    code: "S1",
    title: "Klima u sobi obitelji s djecom",
    answers: [
      "Ispričati se, odmah premjestiti obitelj, ponuditi večeru, poslati tehničara, sutra nazvati gošću",
      "Ispričati se, premjestiti obitelj i poslati tehničara",
      "Vratiti novac za cijeli boravak",
      "Objasniti da su zbog toplinskog vala svi uređaji opterećeni i poslati tehničara kad stigne",
    ],
    chosen: null,
    free_text: true,
    character: { name: "Petra Horvat", role: "gošća u sobi 214, s dvoje male djece" },
    response: null,
    ...over,
  };
}

export function s2(over: Partial<SituationBody> = {}): SituationBody {
  return {
    ...s1(),
    code: "S2",
    title: "Šef kuhinje traži povišicu ili odlazi",
    character: { name: "Marko Babić", role: "šef kuhinje, u hotelu sedam sezona" },
    ...over,
  };
}

export function s3(over: Partial<SituationBody> = {}): SituationBody {
  return {
    code: "S3",
    title: "Agencija nudi grupu: 25 soba, 7 noći u kolovozu, uz 25 % popusta",
    answers: [
      "Protuponuda: grupa u rujnu, 20 % popusta, depozit i rok otkaza",
      "Prihvatiti uz 15 % popusta i depozit",
      "Prihvatiti ponudu kako je",
      "Odbiti",
    ],
    chosen: null,
    free_text: false,
    character: null,
    response: null,
    ...over,
  };
}

export function situations(
  items: SituationBody[] = [s1(), s3()],
  over: Partial<SituationsBody> = {},
): SituationsBody {
  return {
    month: "jun",
    round: "jun",
    open: true,
    unanswered: items.filter((i) => i.chosen === null && i.response === null).length,
    items,
    ...over,
  };
}

export function conversation(over: Partial<ConversationBody> = {}): ConversationBody {
  return {
    situation: "S1",
    character_name: "Petra Horvat",
    role: "gošća u sobi 214, s dvoje male djece",
    opening: "Klima u sobi ne radi, a djeca ne mogu spavati. Vani je 34 °C!",
    lines: [],
    remaining: 6,
    awaiting_reply: false,
    open: true,
    ...over,
  };
}

/** „Zašto se indeksi pomaknuli“ za lipanj; indeksi kao `ix(5600, 5200, 5000, 5400, 4800)`, najviše G (+6). */
export function explanation(over: Partial<ExplanationBody> = {}): ExplanationBody {
  return {
    month: "jun",
    round: "jun",
    progress_text: "+8 bodova",
    indices: [
      {
        code: "G",
        name: "Zadovoljstvo gostiju",
        before: 50,
        after: 56,
        change: 6,
        drivers: [
          {
            var: "satisfaction",
            name: "Zadovoljstvo gostiju",
            meaning: "Koliko su gosti zadovoljni boravkom, na ljestvici do 100 bodova.",
            lines: [
              { cause: "usluga, oprema i cijena zajedno", amount: "+1 bod", effect: "up" },
              {
                cause: "situacija „Klima u sobi obitelji s djecom“",
                amount: "+5 bodova",
                effect: "up",
              },
            ],
          },
        ],
      },
      {
        code: "R",
        name: "Recenzije",
        before: 50,
        after: 52,
        change: 2,
        drivers: [
          {
            var: "rating",
            name: "Ocjena na portalima",
            meaning: "Prosječna ocjena hotela na portalima za rezervacije, na ljestvici do deset.",
            lines: [
              { cause: "ocjene gostiju prema njihovu zadovoljstvu", amount: "+0,12 ocjene", effect: "up" },
            ],
          },
        ],
      },
      {
        code: "Z",
        name: "Zaposlenici",
        before: 50,
        after: 50,
        change: 0,
        drivers: [
          {
            var: "morale",
            name: "Moral",
            meaning: "Raspoloženje i volja tima, na ljestvici do 100 bodova.",
            lines: [
              { cause: "ukupna promjena morala u mjesecu", amount: "−1 bod", effect: "neutral" },
              { cause: "plaće", amount: "−10 bodova", effect: "down" },
            ],
          },
          {
            var: "staff",
            name: "Zaposleni",
            meaning: "Tko je došao i tko je otišao.",
            lines: [
              {
                cause: "odlasci zaposlenih (zamijenjeni novima)",
                amount: "otišle 2 osobe",
                effect: "neutral",
              },
            ],
          },
        ],
      },
      {
        code: "O",
        name: "Operativa",
        before: 50,
        after: 54,
        change: 4,
        drivers: [
          {
            var: "staff_load",
            name: "Opterećenje osoblja",
            meaning: "Koliko posla pada na jednog zaposlenog.",
            lines: [
              {
                cause: "opterećenje u ovom mjesecu",
                amount: "1,6 noćenja po zaposlenom na dan, u redu",
                effect: "up",
              },
            ],
          },
        ],
      },
      {
        code: "F",
        name: "Financije",
        before: 50,
        after: 48,
        change: -2,
        drivers: [
          {
            var: "costs",
            name: "Troškovi",
            meaning: "Što je hotel platio ovaj mjesec, po vrsti troška.",
            lines: [{ cause: "plaće", amount: "117.040 €", effect: "neutral" }],
          },
        ],
      },
    ],
    ...over,
  };
}

/** Pokazatelji lipnja: 120 soba × 30 dana, 2 sobe izvan funkcije, ADR 132,45 €, točka pokrića 52,4 %. */
export function kpiMonth(over: Partial<KpiMonthBody> = {}): KpiMonthBody {
  return {
    month: "jun",
    round: "jun",
    days: 30,
    room_nights: 3600,
    sold_nights: 2448,
    lost_nights: 60,
    occupancy_bp: 6800,
    sellable_occupancy_bp: 9833,
    adr_cents: 13_245,
    revpar_cents: 9007,
    rooms_revenue_cents: 32_423_760,
    total_revenue_cents: 42_150_888,
    trevpar_cents: 11_709,
    gop_cents: 11_650_000,
    gop_margin_bp: 2764,
    goppar_cents: 3236,
    labor_cents: 11_300_000,
    labor_share_bp: 2681,
    cpor_cents: 2200,
    break_even_occupancy_bp: 5240,
    ...over,
  };
}

/** Igra u otvorenom lipnju: moj hotel, igra, situacije, razgovor S1, prazan tijek, još bez pokazatelja. */
export function standardApi(): FakeApi {
  return new FakeApi()
    .on("GET", "/v1/me/hotel", myHotel())
    .on("GET", "/v1/games/:game", game())
    .on("GET", "/v1/hotels/:hotel/situations", situations())
    .on("GET", "/v1/hotels/:hotel/situations/:code/conversation", conversation())
    .on("GET", "/v1/hotels/:hotel/feed", { items: [] })
    .on("GET", "/v1/hotels/:hotel/months/:month/explanation", explanation())
    .on("GET", "/v1/hotels/:hotel/kpi", { months: [] });
}

const SEASON_MONTHS = ["jun", "jul", "aug", "sep"] as const;

/** Dan sezone (0 = 1. 6.) → kalendarski dan i mjesec. */
function seasonDay(offset: number): { day: number; month: (typeof SEASON_MONTHS)[number] } {
  let left = offset;
  for (const m of SEASON_MONTHS) {
    if (left < MONTH_DAYS[m]) return { day: left + 1, month: m };
    left -= MONTH_DAYS[m];
  }
  throw new Error("izvan sezone");
}

/** Kalendar tjednog kruga tNN (n = 1…17): 16 tjedana po 7 dana, zadnji ima 10 dana (do 30. 9.). */
export function weekInfo(n: number): RoundInfoBody {
  const first = (n - 1) * 7;
  const days = n === 17 ? 10 : 7;
  const a = seasonDay(first);
  const b = seasonDay(first + days - 1);
  return {
    code: `t${String(n).padStart(2, "0")}`,
    kind: "week",
    index: n - 1,
    days,
    first_day: a.day,
    first_month: a.month,
    last_day: b.day,
    last_month: b.month,
    month_ends: a.month !== b.month || b.day === MONTH_DAYS[b.month],
  };
}

/** Tjedna v2 igra: 17 krugova, trenutni je tjedan `week` (zadano t01, otvoren). */
export function weeklyGame(
  opts: { week?: number; status?: RoundBody["status"]; hotels?: PublicHotelBody[] } = {},
): GameBody {
  const info = weekInfo(opts.week ?? 1);
  return {
    ...game(opts.hotels ? { hotels: opts.hotels } : {}),
    model: "v2",
    round_kind: "week",
    rounds: Array.from({ length: 17 }, (_v, i) => weekInfo(i + 1)),
    months: [...SEASON_MONTHS],
    current: { month: info.first_month, round: info, status: opts.status ?? "open", unanswered: 2 },
  };
}

/** Moj hotel u tjednoj igri: plan za krug `week`. */
export function weeklyHotel(
  opts: { week?: number; months?: MonthScoreBody[]; lastMonth?: LastMonthBody | null } = {},
): HotelBody {
  const code = weekInfo(opts.week ?? 1).code;
  const base = myHotel({ months: opts.months ?? [], lastMonth: opts.lastMonth ?? null });
  return {
    ...base,
    private: base.private
      ? {
          ...base.private,
          plan: { month: weekInfo(opts.week ?? 1).first_month, round: code, plan: plan(), limits: limits() },
        }
      : base.private,
  };
}

/** Tjedna igra u kojoj je t01 obračunat (napredak +8), a trenutni je t02. */
export function resolvedWeekApi(): FakeApi {
  const scored: MonthScoreBody[] = [
    { month: "jun", round: "t01", indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 },
  ];
  return v2Api()
    .on("GET", "/v1/me/hotel", weeklyHotel({ week: 2, months: scored }))
    .on(
      "GET",
      "/v1/games/:game",
      weeklyGame({
        week: 2,
        hotels: [
          myPublic(scored),
          robot("h2", 2, "Štediša", "stedisa", scored),
          robot("h3", 3, "Brižni", "brizni", scored),
          robot("h4", 4, "Lovac na popunjenost", "lovac", scored),
        ],
      }),
    );
}

/** Kostur v2 igre: kao `standardApi()`, ali tjedna igra u krugu t01 (Taskovi 19–21 dodaju rute odjela i računa). */
export function v2Api(): FakeApi {
  return standardApi()
    .on("GET", "/v1/games/:game", weeklyGame())
    .on("GET", "/v1/me/hotel", weeklyHotel())
    .on("GET", "/v1/hotels/:hotel/situations", situations(undefined, { month: "jun", round: "t01" }))
    .on("GET", "/v1/hotels/:hotel/departments", departmentsBody())
    .on("GET", "/v1/hotels/:hotel/departments/:code", (_req, params) => {
      const body = departmentDetailBody(params.code ?? "");
      return body ? json(200, body) : problem(404, "unknown_department");
    });
}

/** Rezultat jednog odjela u t01; brojke restorana iz Taska 9 (prihod 67.200 €, plaće 37.800 €, namirnice 21.504 €). */
function deptRound(over: Partial<DeptRoundBody> = {}): DeptRoundBody {
  return {
    round: "t01",
    revenue_cents: 6_720_000,
    payroll_cents: 3_780_000,
    cost_of_sales_cents: 2_150_400,
    other_cents: 0,
    profit_cents: 789_600,
    margin_bp: 1175,
    served: 2100,
    capacity: 2400,
    load_hundredths: 190,
    usage_bp: 4500,
    quality_pts: 7000,
    price_level_pct: 100,
    condition_before_pts: 7000,
    wear_pts: -300,
    load_wear_pts: -100,
    maintenance_pts: 300,
    condition_after_pts: 6900,
    maintenance_cents: 400_000,
    value_pts: 7000,
    satisfaction_before_pts: 7000,
    satisfaction_after_pts: 6949,
    ...over,
  };
}

function serviceRound(total: number): ServiceRoundBody {
  return {
    round: "t01",
    payroll_cents: total - 700_000,
    fixed_cents: 500_000,
    per_room_cents: 0,
    marketing_cents: 0,
    reply_cents: 0,
    maintenance_cents: 200_000,
    training_cents: 0,
    hiring_cents: 0,
    situation_cents: 0,
    total_cents: total,
  };
}

function dept(
  over: Partial<DepartmentBody> & Pick<DepartmentBody, "code" | "name" | "class">,
): DepartmentBody {
  return {
    open: true,
    can_close: false,
    permanent: 4,
    seasonal: 2,
    condition_pts: 6900,
    guest_satisfaction_pts: 6949,
    last: deptRound(),
    service_last: null,
    ...over,
  };
}

/** Hotel bez Rooftop bara: smještaj, restoran, wellness, parking (zatvoren) i četiri službe bez prihoda. */
export function departmentsBody(): DepartmentsBody {
  const service = (code: string, name: string, cls: DepartmentBody["class"], total: number) =>
    dept({
      code,
      name,
      class: cls,
      condition_pts: 8000,
      guest_satisfaction_pts: null,
      last: null,
      service_last: serviceRound(total),
    });
  return {
    round: "t01",
    departments: [
      dept({
        code: "rooms",
        name: "Smještaj",
        class: "rooms",
        permanent: 8,
        seasonal: 10,
        condition_pts: 7200,
        guest_satisfaction_pts: 7400,
        last: deptRound({ revenue_cents: 9_800_000, profit_cents: 3_100_000 }),
      }),
      dept({ code: "restaurant", name: "Restoran", class: "food_beverage" }),
      dept({
        code: "spa",
        name: "Mali wellness",
        class: "other_operated",
        permanent: 2,
        seasonal: 1,
        condition_pts: 6500,
        guest_satisfaction_pts: 7400,
        last: deptRound({ revenue_cents: 1_800_000, profit_cents: 400_000, margin_bp: 2222 }),
      }),
      dept({
        code: "parking",
        name: "Parking",
        class: "rentals",
        open: false,
        can_close: true,
        permanent: 0,
        seasonal: 0,
        last: null,
      }),
      service("admin", "Uprava", "admin_general", 5_905_000),
      service("sales", "Prodaja i marketing", "sales_marketing", 2_400_000),
      service("maintenance", "Održavanje", "maintenance", 1_900_000),
      service("energy", "Energija i voda", "utilities", 3_100_000),
    ],
  };
}

/** Odjel s rezultatima po obračunatim krugovima (jedan krug: t01); nepoznat kod vraća null. */
export function departmentDetailBody(code: string): DepartmentDetailBody | null {
  const department = departmentsBody().departments.find((d) => d.code === code);
  if (!department) return null;
  return { department, history: department.last ? [department.last] : [] };
}

/** Gost bez igre: `/v1/me/hotel` je 404 `not_in_game` dok `POST /v1/demo` ne stvori igru, zatim moj hotel. */
export function startApi(): FakeApi {
  let started = false;
  return new FakeApi()
    .on("GET", "/v1/me/hotel", () => (started ? json(200, myHotel()) : problem(404, "not_in_game")))
    .on("GET", "/v1/games/:game", game())
    .on("POST", "/v1/demo", () => {
      started = true;
      return json(201, { game_id: GAME_ID, hotel_id: HOTEL_ID });
    });
}
