### Task 4, dio 2: Okvir ekrana, gost, testni pomoćnici i provjere (Step 8–15)

Početak taska (popis datoteka, Step 1–7: usmjerivač i podatkovni sloj) je u [Task 4](task-04-podatkovni-sloj-usmjerivac-okvir-ekrana-i.md).

- [ ] **Step 8: `ui/Layout.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { ChartNoAxesColumn, House, MessageCircle } from "lucide-react";
import type { ReactNode } from "react";
import type { GameBody } from "../api/types";
import { monthName } from "../format";
import { monthPosition } from "../game";
import { Link, type Route, useRoute } from "../router";

export function chipText(game: GameBody): string {
  const pos = monthPosition(game);
  if (!pos) return t("chip.over");
  return t("chip.position", {
    month: monthName(pos.month).toLocaleUpperCase("hr-HR"),
    n: pos.index + 1,
    total: pos.total,
  });
}

function Logo() {
  return (
    <Link to="/" className="flex items-center gap-2">
      <img src="/logo-znak.png" alt="" className="size-9 rounded-xl" />
      <span className="text-xl font-extrabold tracking-tight">
        Hotelix<span className="text-accent">Sim</span>
      </span>
    </Link>
  );
}

const TABS = [
  { to: "/", label: "nav.hotel", icon: House, match: ["hotel", "plan", "month"] },
  { to: "/situacije", label: "nav.situations", icon: MessageCircle, match: ["situations", "situation"] },
  { to: "/tijek", label: "nav.feed", icon: ChartNoAxesColumn, match: ["feed"] },
] as const;

function BottomNav({ route }: { route: Route }) {
  return (
    <nav
      aria-label={t("nav.label")}
      className="fixed inset-x-0 bottom-0 border-t border-line bg-surface pb-[env(safe-area-inset-bottom)]"
    >
      <ul className="mx-auto grid max-w-md grid-cols-3">
        {TABS.map((tab) => {
          const active = (tab.match as readonly string[]).includes(route.name);
          const Icon = tab.icon;
          return (
            <li key={tab.to}>
              <Link
                to={tab.to}
                aria-current={active ? "page" : undefined}
                className={`flex min-h-14 flex-col items-center justify-center gap-0.5 text-xs font-semibold ${
                  active ? "text-accent" : "text-muted"
                }`}
              >
                <Icon size={22} aria-hidden="true" />
                {t(tab.label)}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

export function Layout({ chip, children }: { chip: string | null; children: ReactNode }) {
  const route = useRoute();
  return (
    <div className="mx-auto min-h-dvh max-w-md">
      <header className="flex items-center justify-between gap-2 px-4 pt-4 pb-2">
        <Logo />
        {chip && (
          <span className="rounded-full bg-accent px-3 py-1 text-xs font-bold tracking-wide text-white">{chip}</span>
        )}
      </header>
      <main className="space-y-3 px-4 pb-28">{children}</main>
      <BottomNav route={route} />
    </div>
  );
}
```

`aria-current={active ? "page" : undefined}` uz `exactOptionalPropertyTypes` može javiti grešku tipa; tada razdvoji:
`{...(active ? { "aria-current": "page" as const } : {})}`.

- [ ] **Step 9: `screens/StartScreen.tsx`, `screens/NotFound.tsx`, privremeni `screens/HotelScreen.tsx`**

```tsx
// screens/StartScreen.tsx — gost bez igre (DEMO §1.4, ADR-022 t. 3)
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { useStartDemo } from "../api/queries";
import { navigate } from "../router";
import { Button, ErrorText, StatusText } from "../ui/primitives";

export function StartScreen() {
  const start = useStartDemo();
  return (
    <main className="mx-auto flex min-h-dvh max-w-md flex-col justify-center gap-6 px-6 py-10">
      <img src="/logo-znak.png" alt="" className="size-16 rounded-2xl" />
      <h1 className="text-4xl leading-tight font-extrabold tracking-tight">
        {t("start.title1")} <span className="text-accent">{t("start.title2")}</span>
      </h1>
      <p className="text-lg text-muted">{t("start.lead")}</p>
      <p className="font-semibold">{t("start.measure")}</p>
      {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
      {start.isPending && <StatusText>{t("start.busy")}</StatusText>}
      <Button disabled={start.isPending} onClick={() => start.mutate(undefined, { onSuccess: () => navigate("/") })}>
        {t("start.button")}
      </Button>
    </main>
  );
}
```

```tsx
// screens/NotFound.tsx
import { t } from "@hotelixsim/i18n";
import { Link } from "../router";
import { Card } from "../ui/primitives";

export function NotFound() {
  return (
    <Card>
      <h1 className="text-xl font-bold">{t("notFound.title")}</h1>
      <Link to="/" className="mt-3 inline-block font-semibold text-accent">
        {t("notFound.back")}
      </Link>
    </Card>
  );
}
```

```tsx
// screens/HotelScreen.tsx — privremeno; Task 5 ga zamjenjuje cijelog
import type { HotelBody } from "../api/types";

export function HotelScreen({ me }: { me: HotelBody }) {
  return <h1 className="text-3xl font-extrabold">Moj hotel · {me.hotel.name}</h1>;
}
```

- [ ] **Step 10: `App.tsx` i `main.tsx`**

```tsx
// App.tsx — ulaz: gost bez igre vidi početak, inače okvir s ekranom po ruti.
import { t } from "@hotelixsim/i18n";
import { isNoGame, problemMessage } from "./api/errors";
import { useGame, useMe } from "./api/queries";
import type { HotelBody } from "./api/types";
import { type Route, useRoute } from "./router";
import { AvatarGallery } from "./screens/AvatarGallery";
import { HotelScreen } from "./screens/HotelScreen";
import { NotFound } from "./screens/NotFound";
import { StartScreen } from "./screens/StartScreen";
import { chipText, Layout } from "./ui/Layout";
import { Button, ErrorText, StatusText } from "./ui/primitives";

function Screen({ route, me }: { route: Route; me: HotelBody }) {
  switch (route.name) {
    case "hotel":
      return <HotelScreen me={me} />;
    case "avatars":
      return <AvatarGallery />;
    default:
      return <NotFound />;
  }
}

function GameShell({ me }: { me: HotelBody }) {
  const route = useRoute();
  const game = useGame(me.game_id);
  return (
    <Layout chip={game.data ? chipText(game.data) : null}>
      <Screen route={route} me={me} />
    </Layout>
  );
}

export function App() {
  const me = useMe();
  if (me.isPending)
    return (
      <div className="mx-auto max-w-md p-6">
        <StatusText>{t("status.loading")}</StatusText>
      </div>
    );
  if (me.isError) {
    if (isNoGame(me.error)) return <StartScreen />;
    return (
      <div className="mx-auto max-w-md space-y-3 p-6">
        <ErrorText>{problemMessage(me.error)}</ErrorText>
        <Button variant="secondary" onClick={() => void me.refetch()}>
          {t("error.retry")}
        </Button>
      </div>
    );
  }
  return <GameShell me={me.data} />;
}
```

```tsx
// main.tsx
import { createApiClient } from "@hotelixsim/api-client";
import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { ApiProvider } from "./api/ApiContext";
import { createQueryClient } from "./api/queryClient";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("nema #root elementa");

createRoot(root).render(
  <StrictMode>
    <ApiProvider client={createApiClient(window.location.origin)}>
      <QueryClientProvider client={createQueryClient()}>
        <App />
      </QueryClientProvider>
    </ApiProvider>
  </StrictMode>,
);
```

Ključeve `status.db.*`, `status.events`, `status.indices.*`, `profile.heading` iz `hr.ts` makni ako ih više ništa ne koristi
(`app.title`, `status.loading`, `status.api.error`, `index.*` ostaju). Provjeri grepom prije brisanja.

- [ ] **Step 11: Testni pomoćnici**

```ts
// test/fakeApi.ts — lažni poslužitelj za testove: rute po metodi i putanji, zapis svih poziva.
export function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });
}

export function problem(status: number, code: string): Response {
  return new Response(JSON.stringify({ type: "about:blank", title: code, status, code }), {
    status,
    headers: { "content-type": "application/problem+json" },
  });
}

type Params = Record<string, string>;
export type Handler = (req: Request, params: Params, body: unknown) => Response | Promise<Response>;

type RouteEntry = { method: string; pattern: RegExp; names: string[]; handler: Handler };

export class FakeApi {
  readonly calls: { method: string; path: string; body: unknown }[] = [];
  private routes: RouteEntry[] = [];

  /** `path` kao "/v1/hotels/:hotel/situations"; tijelo umjesto funkcije = 200 JSON. Kasnija ruta pobjeđuje. */
  on(method: string, path: string, handlerOrBody: Handler | object): this {
    const names: string[] = [];
    const pattern = new RegExp(
      `^${path.replace(/:(\w+)/g, (_m, name: string) => {
        names.push(name);
        return "([^/]+)";
      })}$`,
    );
    const handler: Handler =
      typeof handlerOrBody === "function" ? (handlerOrBody as Handler) : () => json(200, handlerOrBody);
    this.routes.unshift({ method, pattern, names, handler });
    return this;
  }

  posts(path: string) {
    return this.calls.filter((c) => c.method !== "GET" && c.path === path);
  }

  fetch = async (req: Request): Promise<Response> => {
    const url = new URL(req.url);
    const text = req.method === "GET" ? "" : await req.clone().text();
    const body: unknown = text ? JSON.parse(text) : undefined;
    this.calls.push({ method: req.method, path: url.pathname, body });
    for (const r of this.routes) {
      if (r.method !== req.method) continue;
      const m = r.pattern.exec(url.pathname);
      if (!m) continue;
      const params: Params = {};
      r.names.forEach((n, i) => {
        params[n] = decodeURIComponent(m[i + 1] ?? "");
      });
      return r.handler(req, params, body);
    }
    return problem(404, "unknown_route");
  };
}
```

```ts
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

export function robot(id: string, slot: number, name: string, strategy: StrategyCode, months: MonthScoreBody[] = []) {
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
  opts: { months?: MonthScoreBody[]; lastMonth?: LastMonthBody | null; planMonth?: "jun" | "jul" | null } = {},
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

export function situations(items: SituationBody[] = [s1(), s3()], over: Partial<SituationsBody> = {}): SituationsBody {
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
```

```tsx
// test/renderApp.tsx
import { createApiClient } from "@hotelixsim/api-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { vi } from "vitest";
import { App } from "../App";
import { ApiProvider } from "../api/ApiContext";
import { poll } from "../api/poll";
import type { FakeApi } from "./fakeApi";

/** Cijela aplikacija na zadanoj putanji, nad lažnim API-jem. `fetch` se lažira PRIJE stvaranja klijenta. */
export function renderApp(api: FakeApi, path = "/") {
  window.history.replaceState(null, "", path);
  vi.stubGlobal("fetch", vi.fn(api.fetch));
  const client = createApiClient("http://localhost");
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(
    <ApiProvider client={client}>
      <QueryClientProvider client={qc}>
        <App />
      </QueryClientProvider>
    </ApiProvider>,
  );
}

/** Ispitivanje u testu: svakih 20 ms, „sporo“ nakon 200 ms. */
export function fastPolling() {
  Object.assign(poll, { conversationMs: 20, verdictMs: 20, roundMs: 20, feedMs: 20, slowRoundMs: 200, maxFeedPolls: 5 });
}
```

- [ ] **Step 12: Zamijeni `App.test.tsx`** (stari testovi zdravlja i profila C odlaze zajedno sa starim ekranom)

```tsx
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FakeApi, json, problem } from "./test/fakeApi";
import { game, myHotel } from "./test/fixtures";
import { renderApp } from "./test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("gost", () => {
  it("bez igre vidi početak, a „Započni igru“ stvori igru jednom i otvori hotel", async () => {
    let started = false;
    let release: () => void = () => {};
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const api = new FakeApi()
      .on("GET", "/v1/me/hotel", () => (started ? json(200, myHotel()) : problem(401, "guest_required")))
      .on("GET", "/v1/games/:game", game())
      .on("POST", "/v1/demo", async () => {
        await gate;
        started = true;
        return json(201, { game_id: "g1", hotel_id: "h1" });
      });
    renderApp(api, "/demo");
    const button = await screen.findByRole("button", { name: "Započni igru" });
    fireEvent.click(button);
    fireEvent.click(button);
    await waitFor(() => expect((button as HTMLButtonElement).disabled).toBe(true));
    release();
    expect(await screen.findByRole("heading", { name: /Moj hotel/ })).toBeTruthy();
    expect(api.posts("/v1/demo")).toHaveLength(1);
    expect(window.location.pathname).toBe("/");
  });

  it("greška pri stvaranju igre (503) je hrvatska rečenica", async () => {
    const api = new FakeApi()
      .on("GET", "/v1/me/hotel", () => problem(404, "not_in_game"))
      .on("POST", "/v1/demo", () => problem(503, "storage_unavailable"));
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Započni igru" }));
    expect(await screen.findByText("Baza trenutno nije dostupna. Pokušaj ponovno.")).toBeTruthy();
  });

  it("poslužitelj nedostupan → poruka i „Pokušaj ponovno“", async () => {
    const api = new FakeApi();
    api.fetch = async () => {
      throw new TypeError("Failed to fetch");
    };
    renderApp(api);
    expect(await screen.findByText("Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Pokušaj ponovno" })).toBeTruthy();
  });
});

describe("okvir", () => {
  it("zaglavlje pokazuje oznaku mjeseca, a donja traka tri kartice", async () => {
    const api = new FakeApi().on("GET", "/v1/me/hotel", myHotel()).on("GET", "/v1/games/:game", game());
    renderApp(api);
    expect(await screen.findByText("LIPANJ · 1/2")).toBeTruthy();
    const nav = screen.getByRole("navigation", { name: "Glavni izbornik" });
    expect(nav.textContent).toContain("Hotel");
    expect(nav.textContent).toContain("Situacije");
    expect(nav.textContent).toContain("Tijek");
  });

  it("nepoznata putanja daje „Ova stranica ne postoji.“", async () => {
    const api = new FakeApi().on("GET", "/v1/me/hotel", myHotel()).on("GET", "/v1/games/:game", game());
    renderApp(api, "/nesto");
    expect(await screen.findByText("Ova stranica ne postoji.")).toBeTruthy();
  });
});
```

(Treći test mijenja `api.fetch` PRIJE `renderApp`, koji ga stuba kao globalni `fetch`. `me` upit se uz `shouldRetry`
ne koristi — test-klijent ima `retry: false`.)

- [ ] **Step 13: Pokreni sve web testove** → PASS. Ako `openapi-fetch` ne parsira `application/problem+json` tijelo (kod
  ostane `null`, test 503 pada), u `createApiClient` se NE dira; umjesto toga u `unwrap` pročitaj tijelo iz
  `r.response.clone().json()` nije moguće (već pročitano) — javi nalaz kao BLOCKED s ispisom `r.error`. (BACKLOG `[ekrani]`
  „openapi-fetch i problem+json“.)

- [ ] **Step 14: Provjere** (sve četiri) i ručno: `corepack pnpm --filter @hotelixsim/web dev` pa otvori http://localhost:5173 —
  ne zahtijeva poslužitelj; bez njega mora pisati poruka „Poslužitelj trenutno ne odgovara…“. Ugasi dev server.

- [ ] **Step 15: Commit** — „Ekrani 4/11: podatkovni sloj, usmjerivač, okvir i početak za gosta“.

---

