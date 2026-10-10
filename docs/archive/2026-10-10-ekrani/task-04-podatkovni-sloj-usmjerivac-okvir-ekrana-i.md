### Task 4: Podatkovni sloj, usmjerivač, okvir ekrana i gost („Započni igru“)

**Files:**
- Create: `apps/web/src/router.tsx`, `apps/web/src/router.test.tsx`, `apps/web/src/api/errors.ts`,
  `apps/web/src/api/errors.test.ts`, `apps/web/src/api/ApiContext.tsx`, `apps/web/src/api/queryClient.ts`,
  `apps/web/src/api/poll.ts`, `apps/web/src/api/queries.ts`, `apps/web/src/ui/Layout.tsx`,
  `apps/web/src/screens/StartScreen.tsx`, `apps/web/src/screens/NotFound.tsx`, `apps/web/src/screens/HotelScreen.tsx`
  (privremeno samo naslov; Task 5 ga puni), `apps/web/src/test/fakeApi.ts`, `apps/web/src/test/fixtures.ts`,
  `apps/web/src/test/renderApp.tsx`
- Modify: `apps/web/src/App.tsx` (zamjena cijele datoteke), `apps/web/src/App.test.tsx` (zamjena cijele datoteke),
  `apps/web/src/main.tsx`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: Task 1 (`types`, `format`, `game`), Task 2 (`ui/*`), Task 3 (`AvatarGallery` za rutu `/likovi`).
- Produces:
  - `router.tsx`: `type Route = { name: "hotel" } | { name: "situations" } | { name: "situation"; code: string } |
    { name: "plan" } | { name: "month"; month: MonthCode } | { name: "feed" } | { name: "avatars" } | { name: "notFound" }`;
    `matchRoute(pathname: string): Route`; `useRoute(): Route`; `navigate(to: string): void`;
    `Link({ to, className?, children, ...anchor props })`.
  - `api/errors.ts`: `class ApiError extends Error { status: number; code: string | null }`;
    `unwrap<T>(r: { data?: T; error?: unknown; response: Response }): T`; `shouldRetry(failureCount: number, error: unknown): boolean`;
    `problemMessage(error: unknown): string`; `isNoGame(error: unknown): boolean`.
  - `api/ApiContext.tsx`: `ApiProvider({ client, children })`, `useApi(): ApiClient`.
  - `api/queryClient.ts`: `createQueryClient(): QueryClient`.
  - `api/poll.ts`: `poll = { conversationMs: 1500, verdictMs: 2000, roundMs: 2000, feedMs: 2000, slowRoundMs: 150_000,
    maxFeedPolls: 60 }` (mutabilno; testovi postavljaju male vrijednosti).
  - `api/queries.ts`: `keys`, `useMe()`, `useGame(gameId)`, `useStartDemo()`; ostali taskovi ovdje dodaju svoje kuke.
  - `ui/Layout.tsx`: `Layout({ chip: string | null; children })`; `chipText(game: GameBody): string`.
  - `test/fakeApi.ts`: `json(status, body)`, `problem(status, code)`, `class FakeApi { calls; on(method, path, handlerOrBody); fetch }`.
  - `test/fixtures.ts`: `ix(...)`, `plan()`, `limits()`, `myHotel(opts?)`, `robot(...)`, `game(opts?)`, `s1(over?)`,
    `s2(over?)`, `s3(over?)`, `situations(items?, over?)`, `conversation(over?)`, `standardApi(): FakeApi`.
  - `test/renderApp.tsx`: `renderApp(api: FakeApi, path?: string)`, `fastPolling()`.

- [ ] **Step 1: Napiši pad testova `apps/web/src/router.test.tsx`**

```tsx
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Link, matchRoute, useRoute } from "./router";

afterEach(cleanup);

describe("matchRoute", () => {
  it("prepoznaje sve rute igrača", () => {
    expect(matchRoute("/")).toEqual({ name: "hotel" });
    expect(matchRoute("/demo")).toEqual({ name: "hotel" });
    expect(matchRoute("/situacije")).toEqual({ name: "situations" });
    expect(matchRoute("/situacije/S1")).toEqual({ name: "situation", code: "S1" });
    expect(matchRoute("/plan")).toEqual({ name: "plan" });
    expect(matchRoute("/mjesec/jun")).toEqual({ name: "month", month: "jun" });
    expect(matchRoute("/tijek/")).toEqual({ name: "feed" });
    expect(matchRoute("/likovi")).toEqual({ name: "avatars" });
  });
  it("sve ostalo je notFound", () => {
    expect(matchRoute("/mjesec/xyz")).toEqual({ name: "notFound" });
    expect(matchRoute("/situacije/S1/x")).toEqual({ name: "notFound" });
    expect(matchRoute("/nesto")).toEqual({ name: "notFound" });
  });
});

function Where() {
  const r = useRoute();
  return (
    <div>
      <p>ruta:{r.name}</p>
      <Link to="/tijek">tijek</Link>
    </div>
  );
}

describe("Link i useRoute", () => {
  it("klik na Link mijenja putanju bez punog učitavanja i obavijesti useRoute", () => {
    window.history.replaceState(null, "", "/");
    render(<Where />);
    expect(screen.getByText("ruta:hotel")).toBeTruthy();
    fireEvent.click(screen.getByRole("link", { name: "tijek" }));
    expect(window.location.pathname).toBe("/tijek");
    expect(screen.getByText("ruta:feed")).toBeTruthy();
  });
  it("povratak (popstate) također osvježi rutu", () => {
    window.history.replaceState(null, "", "/plan");
    render(<Where />);
    window.history.replaceState(null, "", "/situacije");
    fireEvent(window, new PopStateEvent("popstate"));
    expect(screen.getByText("ruta:situations")).toBeTruthy();
  });
});
```

- [ ] **Step 2: Vidi pad**, pa napiši `apps/web/src/router.tsx`

```tsx
// Mali usmjerivač nad History API-jem (ADR-022 t. 1): sedam ruta, bez učitavača.
import { type AnchorHTMLAttributes, type MouseEvent, useSyncExternalStore } from "react";
import type { MonthCode } from "./api/types";

export type Route =
  | { name: "hotel" }
  | { name: "situations" }
  | { name: "situation"; code: string }
  | { name: "plan" }
  | { name: "month"; month: MonthCode }
  | { name: "feed" }
  | { name: "avatars" }
  | { name: "notFound" };

const MONTHS: readonly MonthCode[] = ["jun", "jul", "aug", "sep"];

function isMonth(s: string): s is MonthCode {
  return (MONTHS as readonly string[]).includes(s);
}

export function matchRoute(pathname: string): Route {
  const [a, b, ...rest] = pathname.split("/").filter(Boolean);
  if (rest.length > 0) return { name: "notFound" };
  if (a === undefined || (a === "demo" && b === undefined)) return { name: "hotel" };
  if (a === "situacije") return b === undefined ? { name: "situations" } : { name: "situation", code: decodeURIComponent(b) };
  if (a === "plan" && b === undefined) return { name: "plan" };
  if (a === "mjesec" && b !== undefined && isMonth(b)) return { name: "month", month: b };
  if (a === "tijek" && b === undefined) return { name: "feed" };
  if (a === "likovi" && b === undefined) return { name: "avatars" };
  return { name: "notFound" };
}

const listeners = new Set<() => void>();

function subscribe(onChange: () => void) {
  listeners.add(onChange);
  window.addEventListener("popstate", onChange);
  return () => {
    listeners.delete(onChange);
    window.removeEventListener("popstate", onChange);
  };
}

export function navigate(to: string): void {
  if (to !== window.location.pathname) window.history.pushState(null, "", to);
  for (const l of listeners) l();
  document.documentElement.scrollTop = 0;
}

export function useRoute(): Route {
  const path = useSyncExternalStore(subscribe, () => window.location.pathname);
  return matchRoute(path);
}

type LinkProps = AnchorHTMLAttributes<HTMLAnchorElement> & { to: string };

export function Link({ to, onClick, ...rest }: LinkProps) {
  function handle(e: MouseEvent<HTMLAnchorElement>) {
    onClick?.(e);
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    navigate(to);
  }
  return <a href={to} onClick={handle} {...rest} />;
}
```

Pokreni `… vitest run src/router.test.tsx` → PASS.

- [ ] **Step 3: Dodaj tekstove grešaka i okvira u `hr.ts`**

```ts
  "error.server": "Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.",
  "error.generic": "Nešto nije u redu. Pokušaj ponovno.",
  "error.retry": "Pokušaj ponovno",
  "problem.guest_required": "Prvo započni igru.",
  "problem.not_in_game": "Nisi u ovoj igri.",
  "problem.not_your_hotel": "To nije tvoj hotel.",
  "problem.unknown_game": "Ta igra ne postoji.",
  "problem.unknown_hotel": "Taj hotel ne postoji.",
  "problem.unknown_month": "Taj mjesec ne postoji u igri.",
  "problem.unknown_situation": "Ta situacija ne postoji.",
  "problem.unknown_answer": "Taj odgovor ne postoji.",
  "problem.unknown_profile": "Taj profil hotela ne postoji.",
  "problem.round_not_open": "Mjesec nije otvoren — sad se ne može mijenjati.",
  "problem.already_answered": "Na ovu situaciju je već odgovoreno.",
  "problem.not_free_text": "Ova situacija nema odgovor svojim riječima.",
  "problem.chat_limit": "Potrošio si sve poruke za ovaj razgovor.",
  "problem.awaiting_reply": "Pričekaj da lik odgovori.",
  "problem.bad_text": "Tekst je prazan ili predug (najviše 2000 znakova).",
  "problem.price_not_whole_euro": "Cijena mora biti u cijelim eurima.",
  "problem.price_out_of_range": "Cijena je izvan dopuštenog raspona.",
  "problem.marketing_out_of_range": "Marketing je izvan dopuštenog raspona.",
  "problem.seasonal_out_of_range": "Broj sezonaca je izvan dopuštenog raspona.",
  "problem.seasonal_step_too_big": "Sezonce možeš mijenjati najviše za 10 mjesečno.",
  "problem.wage_out_of_range": "Plaće su izvan dopuštenog raspona.",
  "problem.renovation_twice": "Obnova je već iskorištena.",
  "problem.storage_unavailable": "Baza trenutno nije dostupna. Pokušaj ponovno.",
  "problem.internal": "Greška na poslužitelju. Pokušaj ponovno.",
  "problem.unknown_route": "Ta adresa ne postoji na poslužitelju.",
  "problem.invalid_body": "Poslužitelj nije razumio zahtjev.",
  "nav.hotel": "Hotel",
  "nav.situations": "Situacije",
  "nav.feed": "Tijek",
  "nav.label": "Glavni izbornik",
  "chip.position": "{month} · {n}/{total}",
  "chip.over": "SEZONA ZAVRŠENA",
  "start.title1": "Tvoj hotel.",
  "start.title2": "Svaka odluka se računa.",
  "start.lead": "Simulacija vođenja hotela za studente.",
  "start.measure": "Mjeri se napredak — u odnosu na početno stanje tvog hotela.",
  "start.button": "Započni igru",
  "start.busy": "Pripremam tvoj hotel…",
  "notFound.title": "Ova stranica ne postoji.",
  "notFound.back": "Natrag na hotel",
```

- [ ] **Step 4: Napiši pad testova `apps/web/src/api/errors.test.ts`**

```ts
import { describe, expect, it } from "vitest";
import { ApiError, isNoGame, problemMessage, shouldRetry, unwrap } from "./errors";

const res = (status: number) => new Response(null, { status });

describe("unwrap", () => {
  it("vraća podatke", () => {
    expect(unwrap({ data: { a: 1 }, response: res(200) })).toEqual({ a: 1 });
  });
  it("baca ApiError sa statusom i kodom problema", () => {
    try {
      unwrap({ error: { type: "x", title: "x", status: 409, code: "awaiting_reply" }, response: res(409) });
      expect.unreachable();
    } catch (e) {
      expect(e).toBeInstanceOf(ApiError);
      expect((e as ApiError).status).toBe(409);
      expect((e as ApiError).code).toBe("awaiting_reply");
    }
  });
  it("tijelo bez koda daje code null", () => {
    expect(() => unwrap({ error: "Bad Gateway", response: res(502) })).toThrowError(ApiError);
  });
});

describe("shouldRetry", () => {
  it("4xx se ne ponavlja, 5xx i mreža jednom", () => {
    expect(shouldRetry(0, new ApiError(404, null))).toBe(false);
    expect(shouldRetry(0, new ApiError(503, "storage_unavailable"))).toBe(true);
    expect(shouldRetry(1, new ApiError(503, null))).toBe(false);
    expect(shouldRetry(0, new TypeError("Failed to fetch"))).toBe(true);
  });
});

describe("problemMessage i isNoGame", () => {
  it("poznati kod daje hrvatsku rečenicu", () => {
    expect(problemMessage(new ApiError(422, "price_out_of_range"))).toBe("Cijena je izvan dopuštenog raspona.");
  });
  it("nepoznat kod: 5xx i mreža → poslužitelj, ostalo → općenito", () => {
    expect(problemMessage(new ApiError(500, "novi_kod"))).toBe(
      "Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.",
    );
    expect(problemMessage(new TypeError("Failed to fetch"))).toBe(
      "Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.",
    );
    expect(problemMessage(new ApiError(400, null))).toBe("Nešto nije u redu. Pokušaj ponovno.");
  });
  it("401 i 404 na /me/hotel znače: gost još nema igru", () => {
    expect(isNoGame(new ApiError(401, "guest_required"))).toBe(true);
    expect(isNoGame(new ApiError(404, "not_in_game"))).toBe(true);
    expect(isNoGame(new ApiError(500, null))).toBe(false);
  });
});
```

- [ ] **Step 5: Vidi pad**, pa napiši `apps/web/src/api/errors.ts`

```ts
import { isMessageKey, t } from "@hotelixsim/i18n";

/** Greška API-ja: HTTP status i stabilan kod problema (`Problem.code`) ako ga tijelo ima. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string | null,
  ) {
    super(`API ${status}${code ? ` ${code}` : ""}`);
    this.name = "ApiError";
  }
}

function problemCode(body: unknown): string | null {
  if (typeof body === "object" && body !== null && "code" in body && typeof body.code === "string") return body.code;
  return null;
}

/** Rezultat openapi-fetch poziva → podaci ili ApiError. */
export function unwrap<T>(r: { data?: T; error?: unknown; response: Response }): T {
  if (r.data !== undefined) return r.data;
  throw new ApiError(r.response.status, problemCode(r.error));
}

/** 4xx je odluka poslužitelja i ne ponavlja se; 5xx i mreža jednom. */
export function shouldRetry(failureCount: number, error: unknown): boolean {
  if (error instanceof ApiError && error.status < 500) return false;
  return failureCount < 1;
}

export function problemMessage(error: unknown): string {
  if (error instanceof ApiError) {
    const key = `problem.${error.code ?? ""}`;
    if (error.code && isMessageKey(key)) return t(key);
    return error.status >= 500 ? t("error.server") : t("error.generic");
  }
  return t("error.server");
}

export function isNoGame(error: unknown): boolean {
  return error instanceof ApiError && (error.status === 401 || error.status === 404);
}
```

Pokreni → PASS.

- [ ] **Step 6: `api/ApiContext.tsx`, `api/queryClient.ts`, `api/poll.ts`**

```tsx
// api/ApiContext.tsx
import type { ApiClient } from "@hotelixsim/api-client";
import { createContext, type ReactNode, useContext } from "react";

const ApiContext = createContext<ApiClient | null>(null);

export function ApiProvider({ client, children }: { client: ApiClient; children: ReactNode }) {
  return <ApiContext value={client}>{children}</ApiContext>;
}

export function useApi(): ApiClient {
  const client = useContext(ApiContext);
  if (!client) throw new Error("ApiProvider nedostaje");
  return client;
}
```

```ts
// api/queryClient.ts
import { QueryClient } from "@tanstack/react-query";
import { shouldRetry } from "./errors";

export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: shouldRetry, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });
}
```

```ts
// api/poll.ts
/** Intervali ispitivanja dok AI ili obračun rade (ADR-022 t. 8). Testovi ih skraćuju. */
export const poll = {
  conversationMs: 1500,
  verdictMs: 2000,
  roundMs: 2000,
  feedMs: 2000,
  slowRoundMs: 150_000,
  maxFeedPolls: 60,
};
```

- [ ] **Step 7: `api/queries.ts` (početak; taskovi 5–10 dodaju svoje kuke ovdje)**

```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { GameBody, HotelBody } from "./types";
import { useApi } from "./ApiContext";
import { unwrap } from "./errors";
import { poll } from "./poll";

export const keys = {
  me: ["me"] as const,
  game: (gameId: string) => ["game", gameId] as const,
  situations: (hotelId: string) => ["situations", hotelId] as const,
  conversation: (hotelId: string, code: string) => ["conversation", hotelId, code] as const,
  feed: (hotelId: string) => ["feed", hotelId] as const,
};

export function useMe() {
  const api = useApi();
  return useQuery<HotelBody>({
    queryKey: keys.me,
    queryFn: async () => unwrap(await api.GET("/v1/me/hotel")),
  });
}

/** Igra; dok se mjesec obračunava, ispituje svakih `poll.roundMs`. */
export function useGame(gameId: string) {
  const api = useApi();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: async () => unwrap(await api.GET("/v1/games/{game_id}", { params: { path: { game_id: gameId } } })),
    refetchInterval: (q) => (q.state.data?.current?.status === "closing" ? poll.roundMs : false),
  });
}

/** Nova demo igra (i gost, ako ga nema). Nakon nje sve se čita ispočetka. */
export function useStartDemo() {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => unwrap(await api.POST("/v1/demo")),
    // resetQueries vrati sve upite u početno stanje i ponovno dohvati aktivne (stara igra ne smije ostati na ekranu).
    onSuccess: () => qc.resetQueries(),
  });
}
```

Napomena za `useGame`: ekran „Mjesec završen“ (Task 9) treba ispitivati i dok runda još nije prešla u `closing`
(neposredno nakon 202); on to radi vlastitim `refetchInterval` u kuki `useGameUntilResolved` (Task 9). Ovdje ne mijenjaj.

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

