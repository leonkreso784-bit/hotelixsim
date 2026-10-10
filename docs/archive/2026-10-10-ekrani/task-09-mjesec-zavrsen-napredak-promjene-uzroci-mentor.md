### Task 9: Mjesec završen — napredak, promjene, uzroci, mentor, ljestvica

**Files:**
- Create: `apps/web/src/screens/MonthScreen.tsx`, `apps/web/src/screens/month/Leaderboard.tsx`,
  `apps/web/src/screens/month/Changes.tsx`, `apps/web/src/screens/MonthScreen.test.tsx`
- Modify: `apps/web/src/App.tsx` (ruta `month`), `apps/web/src/api/queries.ts`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `useApi`, `keys`, `unwrap`, `poll` (Task 4); `progressAt`, `indicesAt`, `indicesBefore`, `indexDeltas`,
  `leaderboard`, `nextMonth` (Task 1); `formatSigned`, `formatChange`, `causeLabel`, `varLabel`, `strategyLabel`,
  `monthName`, `capitalize` (Task 1); `Delta`, `Card`, `Button` (Task 2); `Avatar`, `MENTOR_LOOK` (Task 3).
- Produces:
  - `useGameUntilResolved(gameId, hotelId, month)` — kao `useGame`, ali ispituje svakih `poll.roundMs` dok moj hotel nema
    `months` zapis za `month` (koristi isti `keys.game`).
  - `useFeed(hotelId, opts?: { untilMentorFor?: MonthCode })` — ispituje svakih `poll.feedMs` dok nema `mentor` stavke za
    taj mjesec, najviše `poll.maxFeedPolls` puta (`q.state.dataUpdateCount`).
  - `type FeedChange = Extract<FeedItemBody, { kind: "change" }>`; `Changes({ items: FeedChange[] })`.

- [ ] **Step 1: Kuke**

```ts
export function useGameUntilResolved(gameId: string, hotelId: string, month: MonthCode) {
  const api = useApi();
  return useQuery<GameBody>({
    queryKey: keys.game(gameId),
    queryFn: async () => unwrap(await api.GET("/v1/games/{game_id}", { params: { path: { game_id: gameId } } })),
    // Ispituje samo dok je baš taj mjesec trenutni a moj hotel ga još nema obračunatog (zatvaranje ga prebaci u closing).
    refetchInterval: (q) => {
      const g = q.state.data;
      const mine = g?.hotels.find((h) => h.hotel_id === hotelId);
      if (mine?.months.some((m) => m.month === month)) return false;
      return g?.current?.month === month ? poll.roundMs : false;
    },
  });
}

export function useFeed(hotelId: string, opts: { untilMentorFor?: MonthCode } = {}) {
  const api = useApi();
  return useQuery<FeedItemBody[]>({
    queryKey: keys.feed(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/feed", { params: { path: { hotel_id: hotelId } } })).items,
    refetchInterval: (q) => {
      const m = opts.untilMentorFor;
      if (!m) return false;
      const has = q.state.data?.some((i) => i.kind === "mentor" && i.month === m);
      return has || q.state.dataUpdateCount >= poll.maxFeedPolls ? false : poll.feedMs;
    },
  });
}
```

- [ ] **Step 2: Tekstovi**

```ts
  "month.title": "{month} završen",
  "month.closingTitle": "Obračunavam {month}…",
  "month.closingText": "Svi hoteli se obračunavaju zajedno. AI još može ocjenjivati odgovore — najviše dvije minute.",
  "month.slow": "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi Leonu.",
  "month.notYet": "Ovaj mjesec još nije zatvoren.",
  "month.progress": "Napredak od početka",
  "month.indices": "Promjene indeksa u ovom mjesecu",
  "month.changes": "Zašto (svaka promjena s uzrokom)",
  "month.changesNone": "Za ovaj mjesec nema zapisanih promjena.",
  "month.mentor": "Mentor",
  "month.mentorWaiting": "Mentor piše…",
  "month.mentorMissing": "Mentor se ovaj put nije javio.",
  "month.leaderboard": "Ljestvica napretka",
  "month.continue": "Nastavi u {month}",
  "month.back": "Natrag na hotel",
  "month.seasonOver": "To je bio zadnji mjesec sezone.",
  "month.situationCause": "situacija {code}",
```

- [ ] **Step 3: Napiši pad testova `screens/MonthScreen.test.tsx`**

```tsx
import { cleanup, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { json } from "../test/fakeApi";
import { game, ix, myHotel, myPublic, robot, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const jun = (p: number) => ({ month: "jun" as const, indices: ix(5600, 5200, 5000, 5400, 4800), progress_hundredths: p });

const resolvedGame = game({
  current: { month: "jul", status: "open", unanswered: 1 },
  hotels: [
    myPublic([jun(800)]),
    robot("h2", 2, "Štediša", "stedisa", [jun(400)]),
    robot("h3", 3, "Brižni", "brizni", [jun(600)]),
    robot("h4", 4, "Lovac na popunjenost", "lovac", [jun(300)]),
  ],
});

const feed: FeedItemBody[] = [
  { kind: "round_closed", seq: 1, month: "jun", by: "guest" },
  { kind: "change", seq: 2, month: "jun", var: "sold", cause: "price", amount: -1200, unit: "bp", situation: null },
  { kind: "change", seq: 3, month: "jun", var: "satisfaction", cause: "situation", amount: 450, unit: "points", situation: "S1" },
  { kind: "change", seq: 4, month: "jun", var: "cash", cause: "fixed_costs", amount: -1_250_000, unit: "cents", situation: null },
  { kind: "month_resolved", seq: 5, month: "jun", indices: ix(5600, 5200, 5000, 5400, 4800), progress_hundredths: 800 },
  { kind: "mentor", seq: 6, month: "jun", text: "Usluga je bolja. Prati troškove." },
];

describe("Mjesec završen", () => {
  it("obračunat lipanj: +8, promjene indeksa, uzroci, mentor, ljestvica, Nastavi u srpanj", async () => {
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByRole("heading", { name: "Lipanj završen" })).toBeTruthy();
    const hero = screen.getByText("Napredak od početka").parentElement as HTMLElement;
    expect(within(hero).getByText("+8")).toBeTruthy();
    expect(screen.getByText("▲ +6")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
    const changes = screen.getByText("Zašto (svaka promjena s uzrokom)").closest("section") as HTMLElement;
    expect(within(changes).getByText("Prodane noći")).toBeTruthy();
    expect(within(changes).getByText("cijena")).toBeTruthy();
    expect(within(changes).getByText("−12 %")).toBeTruthy();
    expect(within(changes).getByText("situacija S1")).toBeTruthy();
    expect(within(changes).getByText("−12.500 €")).toBeTruthy();
    const board = screen.getByRole("list", { name: "Ljestvica napretka" });
    const rows = within(board).getAllByRole("listitem");
    expect(rows.map((r) => r.textContent)).toEqual([
      expect.stringContaining("Tvoj hotel"),
      expect.stringContaining("Brižni"),
      expect.stringContaining("Štediša"),
      expect.stringContaining("Lovac na popunjenost"),
    ]);
    expect(rows[0]?.getAttribute("aria-current")).toBe("true");
    expect(screen.getByRole("link", { name: "Nastavi u srpanj" }).getAttribute("href")).toBe("/");
  });

  it("dok se obračunava: čeka i ispituje, pa pokaže rezultat (otvoreno izravno, kao nakon osvježavanja)", async () => {
    let calls = 0;
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", () => {
        calls += 1;
        return json(200, calls < 3 ? closing : resolvedGame);
      })
      .on("GET", "/v1/hotels/:hotel/feed", { items: feed });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Obračunavam lipanj…")).toBeTruthy();
    expect(screen.queryByText("+0")).toBeNull();
    expect(await screen.findByRole("heading", { name: "Lipanj završen" })).toBeTruthy();
  });

  it("predugo obračunavanje: poruka da traje dulje", async () => {
    const closing = game({ current: { month: "jun", status: "closing", unanswered: 0 } });
    const api = standardApi().on("GET", "/v1/games/:game", closing);
    renderApp(api, "/mjesec/jun");
    expect(
      await screen.findByText(
        "Obračun traje dulje nego inače. Osvježi stranicu za minutu; ako i dalje stoji, javi Leonu.",
        {},
        { timeout: 2000 },
      ),
    ).toBeTruthy();
  });

  it("mentor još nije stigao: „Mentor piše…“, zatim tekst", async () => {
    let n = 0;
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [jun(800)], planMonth: "jul" }))
      .on("GET", "/v1/games/:game", resolvedGame)
      .on("GET", "/v1/hotels/:hotel/feed", () => {
        n += 1;
        return json(200, { items: n < 3 ? feed.slice(0, 5) : feed });
      });
    renderApp(api, "/mjesec/jun");
    expect(await screen.findByText("Mentor piše…")).toBeTruthy();
    expect(await screen.findByText("Usluga je bolja. Prati troškove.")).toBeTruthy();
  });

  it("zadnji mjesec: nema „Nastavi“, ima „Natrag na hotel“", async () => {
    const jul = { month: "jul" as const, indices: ix(5600, 5200, 5000, 5400, 5000), progress_hundredths: 1100 };
    const over = game({
      current: null,
      hotels: [myPublic([jun(800), jul]), robot("h2", 2, "Štediša", "stedisa", [jun(400), { ...jul, progress_hundredths: 500 }])],
    });
    const api = standardApi().on("GET", "/v1/games/:game", over).on("GET", "/v1/hotels/:hotel/feed", {
      items: [{ kind: "mentor", seq: 9, month: "jul", text: "Dobra sezona." }],
    });
    renderApp(api, "/mjesec/jul");
    expect(await screen.findByRole("heading", { name: "Srpanj završen" })).toBeTruthy();
    expect(screen.getByText("To je bio zadnji mjesec sezone.")).toBeTruthy();
    expect(screen.queryByRole("link", { name: /Nastavi u/ })).toBeNull();
    expect(screen.getByRole("link", { name: "Natrag na hotel" })).toBeTruthy();
  });

  it("mjesec koji još nije ni zatvoren", async () => {
    renderApp(standardApi(), "/mjesec/jul");
    expect(await screen.findByText("Ovaj mjesec još nije zatvoren.")).toBeTruthy();
  });
});
```

- [ ] **Step 4: Vidi pad.**

- [ ] **Step 5: `screens/month/Changes.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import type { FeedItemBody, VarCode } from "../../api/types";
import { causeLabel, formatChange, varLabel } from "../../format";
import { Card } from "../../ui/primitives";

export type FeedChange = Extract<FeedItemBody, { kind: "change" }>;

/** Promjene obračuna grupirane po varijabli, redoslijedom kojim ih je jezgra zapisala. */
export function Changes({ items }: { items: FeedChange[] }) {
  const groups = new Map<VarCode, FeedChange[]>();
  for (const c of items) groups.set(c.var, [...(groups.get(c.var) ?? []), c]);
  return (
    <Card>
      <section className="space-y-3">
        <h2 className="text-lg font-bold">{t("month.changes")}</h2>
        {groups.size === 0 && <p className="text-muted">{t("month.changesNone")}</p>}
        {[...groups].map(([v, rows]) => (
          <div key={v}>
            <h3 className="font-semibold">{varLabel(v)}</h3>
            <ul className="text-sm">
              {rows.map((c) => (
                <li key={c.seq} className="flex justify-between gap-3 border-b border-line py-1 last:border-0">
                  <span className="text-muted">
                    {c.cause === "situation" && c.situation
                      ? t("month.situationCause", { code: c.situation })
                      : causeLabel(c.cause)}
                  </span>
                  <span className={`tabular-nums ${c.amount < 0 ? "text-down" : "text-up"}`}>
                    {formatChange(c.amount, c.unit)}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>
    </Card>
  );
}
```

Popis je dug (desetak redaka po varijabli). Ako ekran na 390 px postane predug, u Tasku 13 se odlučuje o sklapanju
(`<details>`); ovdje ne.

- [ ] **Step 6: `screens/month/Leaderboard.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { ChartColumn } from "lucide-react";
import type { LeaderRow } from "../../game";
import { formatSigned, strategyLabel } from "../../format";
import { Card } from "../../ui/primitives";

export function Leaderboard({ rows }: { rows: LeaderRow[] }) {
  return (
    <Card className="p-3">
      <h2 id="ljestvica" className="flex items-center gap-2 px-1 pb-2 text-lg font-bold">
        <ChartColumn size={18} aria-hidden="true" className="text-accent" />
        {t("month.leaderboard")}
      </h2>
      <ol aria-labelledby="ljestvica" className="space-y-1">
        {rows.map((r, i) => (
          <li
            key={r.hotelId}
            aria-current={r.mine ? "true" : undefined}
            className={`flex items-center gap-3 rounded-xl px-2 py-2 ${r.mine ? "bg-accent-soft font-bold text-accent" : ""}`}
          >
            <span className="w-5 tabular-nums">{i + 1}.</span>
            <span className="min-w-0 flex-1">
              <span className="block truncate">{r.name}</span>
              {r.strategy && <span className="block text-xs font-normal text-muted">{strategyLabel(r.strategy)}</span>}
            </span>
            <span className={`tabular-nums font-extrabold ${r.progress < 0 ? "text-down" : "text-up"}`}>
              {formatSigned(r.progress)}
            </span>
          </li>
        ))}
      </ol>
    </Card>
  );
}
```

(Test traži listu po imenu „Ljestvica napretka“ — `aria-labelledby` to daje. Ako se `id` sudara jer je ekran jedan, u redu je;
inače `useId()`.)

- [ ] **Step 7: `screens/MonthScreen.tsx`**

```tsx
import { indexLabel, t } from "@hotelixsim/i18n";
import { useState } from "react";
import { problemMessage } from "../api/errors";
import { poll } from "../api/poll";
import { useFeed, useGameUntilResolved } from "../api/queries";
import type { HotelBody, MonthCode } from "../api/types";
import { capitalize, formatSigned, monthName } from "../format";
import { indexDeltas, indicesAt, indicesBefore, leaderboard, nextMonth, progressAt, sortIndices } from "../game";
import { Link } from "../router";
import { Delta } from "../ui/IndexBars";
import { Avatar } from "../avatar/Avatar";
import { MENTOR_LOOK } from "../avatar/look";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import { Changes, type FeedChange } from "./month/Changes";
import { Leaderboard } from "./month/Leaderboard";

export function MonthScreen({ me, month }: { me: HotelBody; month: MonthCode }) {
  const hotelId = me.hotel.hotel_id;
  const game = useGameUntilResolved(me.game_id, hotelId, month);
  const feed = useFeed(hotelId, { untilMentorFor: month });
  const [since] = useState(() => Date.now());
  const name = monthName(month);

  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  const g = game.data;
  const mine = g.hotels.find((h) => h.hotel_id === hotelId);
  const progress = mine ? progressAt(mine, month) : null;

  if (!mine || progress === null) {
    const closing = g.current?.month === month && g.current.status === "closing";
    if (!closing) return <p className="text-muted">{t("month.notYet")}</p>;
    return (
      <Card className="space-y-2">
        <h1 role="status" className="text-2xl font-extrabold">
          {t("month.closingTitle", { month: name })}
        </h1>
        <p className="text-muted">{t("month.closingText")}</p>
        {Date.now() - since > poll.slowRoundMs && <ErrorText>{t("month.slow")}</ErrorText>}
      </Card>
    );
  }

  const now = indicesAt(mine, month) ?? [];
  const deltas = indexDeltas(indicesBefore(mine, g.months, month), now);
  const items = feed.data ?? [];
  const changes = items.filter((i): i is FeedChange => i.kind === "change" && i.month === month);
  const mentor = items.find((i) => i.kind === "mentor" && i.month === month);
  const next = nextMonth(g.months, month);

  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("month.title", { month: capitalize(name) })}</h1>
      <Card className="bg-accent-soft py-6 text-center">
        <p className="text-6xl font-extrabold text-accent tabular-nums">{formatSigned(progress)}</p>
        <p className="mt-1">{t("month.progress")}</p>
      </Card>

      <Card>
        <h2 className="pb-2 font-bold">{t("month.indices")}</h2>
        <ul className="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {sortIndices(now).map((e) => (
            <li key={e.code} className="rounded-xl border border-line p-2">
              <span className="block text-sm font-semibold">{indexLabel(e.code)}</span>
              <Delta value={deltas.get(e.code) ?? 0} size="lg" />
            </li>
          ))}
        </ul>
      </Card>

      <Card className="flex items-center gap-3 bg-mint">
        <Avatar look={MENTOR_LOOK} size={56} />
        <div>
          <p className="font-bold text-accent">{t("month.mentor")}</p>
          {mentor?.kind === "mentor" ? (
            <p>{mentor.text}</p>
          ) : (
            <p role="status">
              {feed.dataUpdateCount >= poll.maxFeedPolls ? t("month.mentorMissing") : t("month.mentorWaiting")}
            </p>
          )}
        </div>
      </Card>

      <Leaderboard rows={leaderboard(g, month)} />
      <Changes items={changes} />

      {next ? (
        <Link to="/" className="flex min-h-12 items-center justify-center rounded-2xl bg-accent font-bold text-white">
          {t("month.continue", { month: monthName(next) })}
        </Link>
      ) : (
        <>
          <p className="font-semibold">{t("month.seasonOver")}</p>
          <Link to="/" className="flex min-h-12 items-center justify-center rounded-2xl bg-accent-soft font-bold text-accent">
            {t("month.back")}
          </Link>
        </>
      )}
    </>
  );
}
```

„Obračun traje dulje“ ovisi o ponovnom renderu —
`useGameUntilResolved` svakih `poll.roundMs` osvježava ekran, pa se uvjet `Date.now() - since` provjerava.

- [ ] **Step 8: Ruta** — `case "month": return <MonthScreen me={me} month={route.month} />;`. Testovi → PASS; provjere.

- [ ] **Step 9: Commit** — „Ekrani 9/13: Mjesec završen s mentorom, uzrocima i ljestvicom“.

---

