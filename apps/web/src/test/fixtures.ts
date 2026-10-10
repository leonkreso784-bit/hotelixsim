// test/fixtures.ts — tipizirani primjerci odgovora API-ja (oblik iz openapi.json).
import type {
  ConversationBody,
  GameBody,
  HotelBody,
  IndexEntry,
  LastMonthBody,
  MonthScoreBody,
  PlanBody,
  PlanLimitsBody,
  PublicHotelBody,
  RoundBody,
  SituationBody,
  SituationsBody,
  StrategyCode,
} from "../api/types";
import { FakeApi } from "./fakeApi";

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
    hotel_id: "h1",
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
    game_id: "g1",
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
      plan: planMonth === null ? null : { month: planMonth, plan: plan(), limits: limits() },
    },
  };
}

export function game(opts: { current?: RoundBody | null; hotels?: PublicHotelBody[] } = {}): GameBody {
  return {
    game_id: "g1",
    months: ["jun", "jul"],
    current: opts.current === undefined ? { month: "jun", status: "open", unanswered: 2 } : opts.current,
    my_hotel_id: "h1",
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
    title: "Agencija nudi grupu od 25 soba za 7 noći (nacrt)",
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

/** Igra u otvorenom lipnju: moj hotel, igra, situacije, razgovor S1, prazan tijek. */
export function standardApi(): FakeApi {
  return new FakeApi()
    .on("GET", "/v1/me/hotel", myHotel())
    .on("GET", "/v1/games/:game", game())
    .on("GET", "/v1/hotels/:hotel/situations", situations())
    .on("GET", "/v1/hotels/:hotel/situations/:code/conversation", conversation())
    .on("GET", "/v1/hotels/:hotel/feed", { items: [] });
}
