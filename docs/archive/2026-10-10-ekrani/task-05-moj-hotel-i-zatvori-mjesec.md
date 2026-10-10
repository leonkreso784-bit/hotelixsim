### Task 5: Moj hotel i „Zatvori mjesec“

**Files:**
- Create: `apps/web/src/screens/CloseMonth.tsx`, `apps/web/src/screens/HotelScreen.test.tsx`
- Modify: `apps/web/src/screens/HotelScreen.tsx` (cijela zamjena), `apps/web/src/api/queries.ts`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `useGame`, `keys`, `unwrap`, `problemMessage` (Task 4); `IndexBars`, `Card`, `Button`, `Badge` (Task 2);
  `latestIndices`, `indexDeltas`, `monthPosition` (Task 1); `formatEuros`, `formatOccupancy`, `formatRating`, `monthName`,
  `capitalize` (Task 1).
- Produces: `useSituations(hotelId)` (ispituje svakih `poll.verdictMs` dok je ijedan `response.status === "pending"`),
  `useCloseMonth(gameId)` (mutacija `month: MonthCode`; nakon uspjeha invalidira sve upite) — u `api/queries.ts`.

- [ ] **Step 1: Kuke u `api/queries.ts`**

```ts
export function useSituations(hotelId: string) {
  const api = useApi();
  return useQuery<SituationsBody>({
    queryKey: keys.situations(hotelId),
    queryFn: async () =>
      unwrap(await api.GET("/v1/hotels/{hotel_id}/situations", { params: { path: { hotel_id: hotelId } } })),
    refetchInterval: (q) =>
      q.state.data?.items.some((i) => i.response?.status === "pending") ? poll.verdictMs : false,
  });
}

export function useCloseMonth(gameId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (month: MonthCode) =>
      unwrap(
        await api.POST("/v1/games/{game_id}/rounds/{month}/close", { params: { path: { game_id: gameId, month } } }),
      ),
    onSuccess: () => qc.invalidateQueries(),
  });
}
```

(dodaj `MonthCode`, `SituationsBody` u import iz `./types`)

- [ ] **Step 2: Tekstovi u `hr.ts`**

```ts
  "hotel.title": "Moj hotel",
  "hotel.rooms": "{rooms} soba",
  "hotel.fromStart": "od početka",
  "hotel.occupancy": "Popunjenost",
  "hotel.revenue": "Prihod od soba",
  "hotel.cash": "Blagajna",
  "hotel.staff": "Zaposleni",
  "hotel.rating": "Ocjena na portalima",
  "hotel.lastMonth": "zadnji mjesec: {month}",
  "hotel.noMonthYet": "nakon prvog mjeseca",
  "hotel.situations": "Situacije mjeseca",
  "hotel.plan": "Plan mjeseca",
  "hotel.result": "Rezultat: {month}",
  "hotel.closing": "Obračun mjeseca „{month}“ je u tijeku.",
  "hotel.closingLink": "Pogledaj obračun",
  "hotel.seasonOver": "Sezona je završena.",
  "hotel.newGame": "Nova igra",
  "close.button": "Zatvori mjesec",
  "close.confirmTitle": "Zatvaraš {month}.",
  "close.confirmText": "Mjesec se obračunava za sve hotele odjednom i nakon toga se više ne mijenja.",
  "close.unanswered": "Bez odgovora: {n}. Takve situacije dobivaju najnižu ocjenu (1).",
  "close.confirm": "Zatvori {month}",
  "close.cancel": "Odustani",
```

- [ ] **Step 3: Napiši pad testova `screens/HotelScreen.test.tsx`**

```tsx
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { json } from "../test/fakeApi";
import { game, ix, myHotel, standardApi } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const junScore = { month: "jun" as const, indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 };

describe("Moj hotel", () => {
  it("prije prvog obračuna: indeksi bez promjena, stvarne brojke, ocjena do 10, broj situacija", async () => {
    renderApp(standardApi());
    expect(await screen.findByRole("heading", { name: "Moj hotel" })).toBeTruthy();
    expect(screen.getByText("120 soba")).toBeTruthy();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
    expect(screen.getAllByRole("img", { name: /od 100$/ })).toHaveLength(5);
    expect(screen.queryByText(/▲|▼/)).toBeNull();
    expect(screen.getByText("42.000 €")).toBeTruthy();
    expect(screen.getByText("58")).toBeTruthy();
    expect(screen.getByText("8,4/10")).toBeTruthy();
    const sit = screen.getByRole("link", { name: /Situacije mjeseca/ });
    expect(await within(sit).findByText("2")).toBeTruthy();
    expect(screen.getByRole("link", { name: /Plan mjeseca/ }).getAttribute("href")).toBe("/plan");
  });

  it("nakon lipnja: promjene od početka, popunjenost i prihod lipnja, poveznica na rezultat", async () => {
    const api = standardApi()
      .on(
        "GET",
        "/v1/me/hotel",
        myHotel({
          months: [junScore],
          planMonth: "jul",
          lastMonth: {
            month: "jun",
            sold: 2448,
            available: 3600,
            rooms_revenue_cents: 9_800_000,
            gop_cents: 2_000_000,
            goppar_cents: 55_500,
            mentor: "Usluga je bolja. Prati troškove.",
          },
        }),
      )
      .on("GET", "/v1/games/:game", game({ current: { month: "jul", status: "open", unanswered: 1 } }));
    renderApp(api);
    expect(await screen.findByText("SRPANJ · 2/2")).toBeTruthy();
    expect(await screen.findByText("▲ +3")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(screen.getByText("od početka")).toBeTruthy();
    expect(screen.getByText("68 %")).toBeTruthy();
    expect(screen.getByText("98.000 €")).toBeTruthy();
    expect(screen.getByRole("link", { name: /Rezultat: lipanj/ }).getAttribute("href")).toBe("/mjesec/jun");
  });

  it("„Zatvori mjesec“ traži potvrdu s upozorenjem, šalje jednom i vodi na obračun", async () => {
    const api = standardApi().on("POST", "/v1/games/:game/rounds/:month/close", () =>
      json(202, { month: "jun", status: "closing", unanswered: 0 }),
    );
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    expect(screen.getByText("Zatvaraš lipanj.")).toBeTruthy();
    expect(await screen.findByText("Bez odgovora: 2. Takve situacije dobivaju najnižu ocjenu (1).")).toBeTruthy();
    const confirm = screen.getByRole("button", { name: "Zatvori lipanj" });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    await waitFor(() => expect(window.location.pathname).toBe("/mjesec/jun"));
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(1);
  });

  it("„Odustani“ zatvara potvrdu bez zahtjeva", async () => {
    const api = standardApi();
    renderApp(api);
    fireEvent.click(await screen.findByRole("button", { name: "Zatvori mjesec" }));
    fireEvent.click(screen.getByRole("button", { name: "Odustani" }));
    expect(screen.queryByText("Zatvaraš lipanj.")).toBeNull();
    expect(api.posts("/v1/games/g1/rounds/jun/close")).toHaveLength(0);
  });

  it("dok se mjesec obračunava: nema gumba, ima poveznica na obračun", async () => {
    const api = standardApi().on("GET", "/v1/games/:game", game({ current: { month: "jun", status: "closing", unanswered: 0 } }));
    renderApp(api);
    expect(await screen.findByText("Obračun mjeseca „lipanj“ je u tijeku.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori mjesec" })).toBeNull();
    expect(screen.getByRole("link", { name: "Pogledaj obračun" }).getAttribute("href")).toBe("/mjesec/jun");
  });

  it("kraj sezone: nema zatvaranja ni plana, ima rezultat srpnja i „Nova igra“", async () => {
    const julScore = { month: "jul" as const, indices: ix(5600, 5200, 5000, 5400, 5000), progress_hundredths: 1100 };
    const api = standardApi()
      .on("GET", "/v1/me/hotel", myHotel({ months: [junScore, julScore], planMonth: null }))
      .on("GET", "/v1/games/:game", game({ current: null }))
      .on("POST", "/v1/demo", () => json(201, { game_id: "g2", hotel_id: "h9" }));
    renderApp(api);
    expect(await screen.findByText("Sezona je završena.")).toBeTruthy();
    expect(screen.getByText("SEZONA ZAVRŠENA")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Zatvori mjesec" })).toBeNull();
    expect(screen.queryByRole("link", { name: /Plan mjeseca/ })).toBeNull();
    expect(screen.getByRole("link", { name: /Rezultat: srpanj/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Nova igra" }));
    await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  });
});
```

Napomena: „Rezultat: srpanj“ bez `lastMonth` — poveznica na rezultat uzima **zadnji mjesec iz `me.hotel.months`**, ne
`last_month` (to vrijedi i za test 2).

- [ ] **Step 4: Vidi pad** (`… vitest run src/screens/HotelScreen.test.tsx`).

- [ ] **Step 5: `screens/CloseMonth.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { useState } from "react";
import { problemMessage } from "../api/errors";
import { useCloseMonth } from "../api/queries";
import type { MonthCode } from "../api/types";
import { monthName } from "../format";
import { navigate } from "../router";
import { Button, Card, ErrorText } from "../ui/primitives";

/** Gumb s potvrdom; upozorava na neodgovorene PRIJE klika (BACKLOG [dnevnik] M4). */
export function CloseMonth({ gameId, month, unanswered }: { gameId: string; month: MonthCode; unanswered: number }) {
  const [asking, setAsking] = useState(false);
  const close = useCloseMonth(gameId);
  const name = monthName(month);
  if (!asking) return <Button onClick={() => setAsking(true)}>{t("close.button")}</Button>;
  return (
    <Card className="space-y-3 border-accent">
      <p className="text-lg font-bold">{t("close.confirmTitle", { month: name })}</p>
      <p className="text-sm text-muted">{t("close.confirmText")}</p>
      {unanswered > 0 && <ErrorText>{t("close.unanswered", { n: unanswered })}</ErrorText>}
      {close.isError && <ErrorText>{problemMessage(close.error)}</ErrorText>}
      <Button
        disabled={close.isPending}
        onClick={() => close.mutate(month, { onSuccess: () => navigate(`/mjesec/${month}`) })}
      >
        {t("close.confirm", { month: name })}
      </Button>
      <Button variant="secondary" disabled={close.isPending} onClick={() => setAsking(false)}>
        {t("close.cancel")}
      </Button>
    </Card>
  );
}
```

- [ ] **Step 6: `screens/HotelScreen.tsx`** (cijela zamjena; ikone `lucide-react`)

```tsx
import { t } from "@hotelixsim/i18n";
import { BedDouble, CalendarDays, ChartColumn, ChevronRight, FileText, Star, Users, Wallet } from "lucide-react";
import type { ReactNode } from "react";
import { problemMessage } from "../api/errors";
import { useGame, useSituations, useStartDemo } from "../api/queries";
import type { HotelBody } from "../api/types";
import { formatEuros, formatOccupancy, formatRating, monthName } from "../format";
import { indexDeltas, latestIndices } from "../game";
import { Link, navigate } from "../router";
import { IndexBars } from "../ui/IndexBars";
import { Badge, Button, Card, ErrorText, StatusText } from "../ui/primitives";
import { CloseMonth } from "./CloseMonth";

function Stat({ icon, label, value, note }: { icon: ReactNode; label: string; value: string; note?: string }) {
  return (
    <Card className="flex items-center gap-3 p-3">
      <span className="grid size-10 place-items-center rounded-xl bg-accent-soft text-accent">{icon}</span>
      <span className="min-w-0">
        <span className="block text-xs text-muted">{label}</span>
        <strong className="block text-lg tabular-nums">{value}</strong>
        {note && <span className="block truncate text-xs text-muted">{note}</span>}
      </span>
    </Card>
  );
}

function Row({ to, icon, label, end }: { to: string; icon: ReactNode; label: string; end?: ReactNode }) {
  return (
    <Link to={to} className="flex min-h-12 items-center gap-3 py-2">
      <span className="text-accent">{icon}</span>
      <span className="flex-1 font-semibold">{label}</span>
      {end}
      <ChevronRight size={18} aria-hidden="true" className="text-muted" />
    </Link>
  );
}

export function HotelScreen({ me }: { me: HotelBody }) {
  const game = useGame(me.game_id);
  const sits = useSituations(me.hotel.hotel_id);
  const start = useStartDemo();
  const priv = me.private;
  if (game.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (game.isError) return <ErrorText>{problemMessage(game.error)}</ErrorText>;
  if (!priv) return <ErrorText>{t("error.generic")}</ErrorText>;

  const h = me.hotel;
  const current = game.data.current;
  const lastScored = h.months.at(-1);
  const deltas = lastScored ? indexDeltas(h.start, lastScored.indices) : null;
  const lm = priv.last_month;
  const ic = (n: ReactNode) => n;

  return (
    <>
      <div>
        <h1 className="text-3xl font-extrabold tracking-tight">{t("hotel.title")}</h1>
        <p className="text-muted">{t("hotel.rooms", { rooms: priv.now.rooms })}</p>
      </div>

      <Card>
        <IndexBars
          indices={latestIndices(h)}
          deltas={deltas}
          deltaLabel={deltas ? t("hotel.fromStart") : null}
        />
      </Card>

      <div className="grid grid-cols-2 gap-3">
        <Stat
          icon={ic(<BedDouble size={20} aria-hidden="true" />)}
          label={t("hotel.occupancy")}
          value={lm ? formatOccupancy(lm.sold, lm.available) : "—"}
          note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
        />
        <Stat
          icon={<ChartColumn size={20} aria-hidden="true" />}
          label={t("hotel.revenue")}
          value={lm ? formatEuros(lm.rooms_revenue_cents) : "—"}
          note={lm ? t("hotel.lastMonth", { month: monthName(lm.month) }) : t("hotel.noMonthYet")}
        />
        <Stat icon={<Wallet size={20} aria-hidden="true" />} label={t("hotel.cash")} value={formatEuros(priv.now.cash_cents)} />
        <Stat
          icon={<Users size={20} aria-hidden="true" />}
          label={t("hotel.staff")}
          value={String(priv.now.permanent + priv.now.seasonal)}
        />
      </div>

      <Card className="flex items-center gap-3 py-3">
        <Star size={20} aria-hidden="true" className="text-accent" />
        <span className="flex-1 font-semibold">{t("hotel.rating")}</span>
        <strong className="text-lg tabular-nums">{formatRating(priv.now.rating)}</strong>
      </Card>

      <Card className="divide-y divide-line py-1">
        {current && (
          <Row
            to="/situacije"
            icon={<FileText size={20} aria-hidden="true" />}
            label={t("hotel.situations")}
            end={sits.data && sits.data.unanswered > 0 ? <Badge>{sits.data.unanswered}</Badge> : undefined}
          />
        )}
        {current?.status === "open" && (
          <Row to="/plan" icon={<CalendarDays size={20} aria-hidden="true" />} label={t("hotel.plan")} />
        )}
        {lastScored && (
          <Row
            to={`/mjesec/${lastScored.month}`}
            icon={<ChartColumn size={20} aria-hidden="true" />}
            label={t("hotel.result", { month: monthName(lastScored.month) })}
          />
        )}
      </Card>

      {current?.status === "open" && (
        <CloseMonth gameId={me.game_id} month={current.month} unanswered={sits.data?.unanswered ?? current.unanswered} />
      )}
      {current?.status === "closing" && (
        <Card className="space-y-2">
          <p role="status">{t("hotel.closing", { month: monthName(current.month) })}</p>
          <Link to={`/mjesec/${current.month}`} className="font-semibold text-accent">
            {t("hotel.closingLink")}
          </Link>
        </Card>
      )}
      {!current && (
        <Card className="space-y-3">
          <p className="text-lg font-bold">{t("hotel.seasonOver")}</p>
          {start.isError && <ErrorText>{problemMessage(start.error)}</ErrorText>}
          <Button disabled={start.isPending} onClick={() => start.mutate(undefined, { onSuccess: () => navigate("/") })}>
            {t("hotel.newGame")}
          </Button>
        </Card>
      )}
    </>
  );
}
```

(`ic` pomoćnik nije potreban — izvođač ga smije maknuti; `end={… : undefined}` uz `exactOptionalPropertyTypes` zahtijeva
`end?: ReactNode | undefined` u `Row` — prilagodi potpis ako typecheck pada.)

Poveznica „Situacije mjeseca“ ima tekst „Situacije mjeseca 2“ (oznaka je unutra) — test je traži s `/Situacije mjeseca/`.

- [ ] **Step 7: Pokreni testove ekrana** → PASS; zatim provjere (sve četiri).

- [ ] **Step 8: Commit** — „Ekrani 5/13: Moj hotel i Zatvori mjesec s upozorenjem“.

---

