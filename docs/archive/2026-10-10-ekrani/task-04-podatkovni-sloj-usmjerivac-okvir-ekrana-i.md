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

**Nastavak:** Step 8–15 (okvir ekrana, gost, testni pomoćnici, provjere i commit) su u
[Task 4, dio 2](task-04b-okvir-ekrana-gost-testni-pomocnici-i-provjere.md).
