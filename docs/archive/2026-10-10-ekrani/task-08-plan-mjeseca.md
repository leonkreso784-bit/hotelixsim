### Task 8: Plan mjeseca

**Files:**
- Create: `apps/web/src/screens/PlanScreen.tsx`, `apps/web/src/screens/PlanScreen.test.tsx`
- Modify: `apps/web/src/App.tsx` (ruta `plan`), `apps/web/src/api/queries.ts`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `HotelBody.private.plan: PlanStateBody | null` (`{ month, plan: PlanBody, limits: PlanLimitsBody }`), `keys`, `unwrap`.
- Produces: `useSavePlan(hotelId)` — mutacija `{ month: MonthCode; plan: PlanBody }`; uspjeh invalidira `keys.me` i `keys.feed`.

- [ ] **Step 1: Kuka**

```ts
export function useSavePlan(hotelId: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ month, plan }: { month: MonthCode; plan: PlanBody }) =>
      unwrap(
        await api.PUT("/v1/hotels/{hotel_id}/plans/{month}", {
          params: { path: { hotel_id: hotelId, month } },
          body: plan,
        }),
      ),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: keys.me });
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}
```

- [ ] **Step 2: Tekstovi**

```ts
  "plan.title": "Plan mjeseca",
  "plan.for": "Za {month}. Vrijedi od zatvaranja mjeseca; zadnji spremljeni plan pobjeđuje.",
  "plan.closed": "Plan se mijenja dok je mjesec otvoren.",
  "plan.price": "Cijena sobe po noći (€)",
  "plan.seasonal": "Sezonski radnici",
  "plan.maintenance": "Održavanje",
  "plan.more": "Više postavki",
  "plan.marketing": "Marketing (€)",
  "plan.wage": "Plaće (% od tržišne)",
  "plan.training": "Obuka",
  "plan.renovation": "Obnova (jednom u sezoni)",
  "plan.renovationUsed": "Obnova je već iskorištena.",
  "plan.replyReviews": "Odgovaramo na recenzije",
  "plan.range": "{min}–{max}",
  "plan.save": "Spremi plan",
  "plan.saved": "Plan za {month} je spremljen.",
  "plan.invalid": "Upiši brojeve u dopuštenom rasponu.",
  "maintenance.deferred": "odgođeno",
  "maintenance.regular": "redovno",
  "maintenance.intensive": "pojačano",
  "training.none": "bez obuke",
  "training.basic": "osnovna",
  "training.full": "puna",
```

- [ ] **Step 3: Napiši pad testova `screens/PlanScreen.test.tsx`**

```tsx
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../test/fakeApi";
import { myHotel, plan, standardApi } from "../test/fixtures";
import { renderApp } from "../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("plan mjeseca", () => {
  it("glavne tri postavke odmah, ostalih pet pod „Više postavki“, s granicama", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("heading", { name: "Plan mjeseca" })).toBeTruthy();
    const price = screen.getByLabelText("Cijena sobe po noći (€)") as HTMLInputElement;
    expect(price.value).toBe("132");
    expect(price.min).toBe("79");
    expect(price.max).toBe("198");
    expect((screen.getByLabelText("Sezonski radnici") as HTMLInputElement).value).toBe("28");
    expect(screen.getByRole("radio", { name: "redovno" })).toBeTruthy();
    const more = screen.getByText("Više postavki").closest("details") as HTMLDetailsElement;
    expect(more.open).toBe(false);
    expect(screen.getByLabelText("Marketing (€)")).toBeTruthy();
    expect(screen.getByLabelText("Plaće (% od tržišne)")).toBeTruthy();
    expect(screen.getByRole("radio", { name: "puna" })).toBeTruthy();
    expect(screen.getByLabelText("Obnova (jednom u sezoni)")).toBeTruthy();
    expect(screen.getByLabelText("Odgovaramo na recenzije")).toBeTruthy();
  });

  it("spremanje šalje cijeli plan u centima i potvrdi", async () => {
    const api = standardApi().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "140" } });
    fireEvent.click(screen.getByRole("radio", { name: "pojačano" }));
    fireEvent.change(screen.getByLabelText("Marketing (€)"), { target: { value: "5000" } });
    fireEvent.click(screen.getByLabelText("Odgovaramo na recenzije"));
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    expect(await screen.findByText("Plan za lipanj je spremljen.")).toBeTruthy();
    const put = api.calls.filter((c) => c.method === "PUT");
    expect(put).toHaveLength(1);
    expect(put[0]?.path).toBe("/v1/hotels/h1/plans/jun");
    expect(put[0]?.body).toEqual({
      ...plan(),
      price_cents: 14_000,
      maintenance: "intensive",
      marketing_cents: 500_000,
      reply_reviews: true,
    });
  });

  it("422 od poslužitelja je hrvatska rečenica", async () => {
    const api = standardApi().on("PUT", "/v1/hotels/:hotel/plans/:month", () => problem(422, "seasonal_step_too_big"));
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Sezonski radnici"), { target: { value: "38" } });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    expect(await screen.findByText("Sezonce možeš mijenjati najviše za 10 mjesečno.")).toBeTruthy();
  });

  it("prazno ili decimalno polje ne šalje zahtjev nego traži ispravak", async () => {
    const api = standardApi();
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    expect(await screen.findByText("Upiši brojeve u dopuštenom rasponu.")).toBeTruthy();
    await waitFor(() => expect(api.calls.filter((c) => c.method === "PUT")).toHaveLength(0));
  });

  it("obnova je onemogućena kad je iskorištena", async () => {
    const me = myHotel();
    if (me.private?.plan) me.private.plan.limits.renovation_available = false;
    renderApp(standardApi().on("GET", "/v1/me/hotel", me), "/plan");
    expect(((await screen.findByLabelText("Obnova (jednom u sezoni)")) as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText("Obnova je već iskorištena.")).toBeTruthy();
  });

  it("zatvoren mjesec: nema obrasca", async () => {
    renderApp(standardApi().on("GET", "/v1/me/hotel", myHotel({ planMonth: null })), "/plan");
    expect(await screen.findByText("Plan se mijenja dok je mjesec otvoren.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Spremi plan" })).toBeNull();
  });
});
```

- [ ] **Step 4: Vidi pad.**

- [ ] **Step 5: `screens/PlanScreen.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { type FormEvent, type ReactNode, useState } from "react";
import { problemMessage } from "../api/errors";
import { useSavePlan } from "../api/queries";
import type { HotelBody, PlanBody, PlanLimitsBody } from "../api/types";
import { monthName } from "../format";
import { Button, Card, ErrorText, StatusText } from "../ui/primitives";

type Draft = {
  price: string;
  marketing: string;
  seasonal: string;
  wage: string;
  training: PlanBody["training"];
  maintenance: PlanBody["maintenance"];
  renovation: boolean;
  reply: boolean;
};

const toDraft = (p: PlanBody): Draft => ({
  price: String(p.price_cents / 100),
  marketing: String(Math.round(p.marketing_cents / 100)),
  seasonal: String(p.seasonal_target),
  wage: String(p.wage_pct),
  training: p.training,
  maintenance: p.maintenance,
  renovation: p.renovation,
  reply: p.reply_reviews,
});

/** Cijeli broj u granicama ili null (prazno, decimalno, izvan raspona). */
function int(s: string, min: number, max: number): number | null {
  if (!/^\d+$/.test(s.trim())) return null;
  const n = Number(s.trim());
  return n >= min && n <= max ? n : null;
}

function toPlan(d: Draft, l: PlanLimitsBody): PlanBody | null {
  const price = int(d.price, l.price_min_cents / 100, l.price_max_cents / 100);
  const marketing = int(d.marketing, 0, l.marketing_max_cents / 100);
  const seasonal = int(d.seasonal, l.seasonal_min, l.seasonal_max);
  const wage = int(d.wage, l.wage_min_pct, l.wage_max_pct);
  if (price === null || marketing === null || seasonal === null || wage === null) return null;
  return {
    price_cents: price * 100,
    marketing_cents: marketing * 100,
    seasonal_target: seasonal,
    wage_pct: wage,
    training: d.training,
    maintenance: d.maintenance,
    renovation: d.renovation,
    reply_reviews: d.reply,
  };
}

function NumberField(props: { id: string; label: string; value: string; min: number; max: number; onChange: (v: string) => void }) {
  return (
    <div className="space-y-1">
      <label htmlFor={props.id} className="block font-semibold">
        {props.label}
      </label>
      <input
        id={props.id}
        type="number"
        inputMode="numeric"
        step={1}
        min={props.min}
        max={props.max}
        value={props.value}
        onChange={(e) => props.onChange(e.target.value)}
        className="min-h-12 w-full rounded-2xl border border-line bg-surface px-3 text-lg tabular-nums"
      />
      <p className="text-xs text-muted">{t("plan.range", { min: props.min, max: props.max })}</p>
    </div>
  );
}

function Choice<T extends string>(props: { legend: string; name: string; value: T; options: { v: T; label: string }[]; onChange: (v: T) => void }) {
  return (
    <fieldset className="space-y-1">
      <legend className="font-semibold">{props.legend}</legend>
      <div className="grid grid-cols-3 gap-2">
        {props.options.map((o) => (
          <label
            key={o.v}
            className={`flex min-h-11 cursor-pointer items-center justify-center rounded-xl border text-sm font-semibold ${
              props.value === o.v ? "border-accent bg-accent-soft text-accent" : "border-line bg-surface"
            }`}
          >
            <input
              type="radio"
              name={props.name}
              value={o.v}
              checked={props.value === o.v}
              onChange={() => props.onChange(o.v)}
              className="sr-only"
            />
            {o.label}
          </label>
        ))}
      </div>
    </fieldset>
  );
}

function Toggle(props: { id: string; label: string; checked: boolean; disabled?: boolean; note?: ReactNode; onChange: (v: boolean) => void }) {
  return (
    <div>
      <label htmlFor={props.id} className="flex min-h-11 items-center gap-3 font-semibold">
        <input
          id={props.id}
          type="checkbox"
          checked={props.checked}
          disabled={props.disabled}
          onChange={(e) => props.onChange(e.target.checked)}
          className="size-5 accent-accent"
        />
        {props.label}
      </label>
      {props.note}
    </div>
  );
}

export function PlanScreen({ me }: { me: HotelBody }) {
  const state = me.private?.plan ?? null;
  const save = useSavePlan(me.hotel.hotel_id);
  const [draft, setDraft] = useState<Draft | null>(state ? toDraft(state.plan) : null);
  const [invalid, setInvalid] = useState(false);
  if (!state || !draft)
    return (
      <>
        <h1 className="text-3xl font-extrabold tracking-tight">{t("plan.title")}</h1>
        <p className="text-muted">{t("plan.closed")}</p>
      </>
    );
  const l = state.limits;
  const set = (patch: Partial<Draft>) => {
    setDraft({ ...draft, ...patch });
    setInvalid(false);
    save.reset();
  };

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!state || !draft || save.isPending) return;
    const plan = toPlan(draft, l);
    if (!plan) {
      setInvalid(true);
      return;
    }
    save.mutate({ month: state.month, plan });
  }

  const month = monthName(state.month);
  return (
    <form onSubmit={onSubmit} className="space-y-3">
      <h1 className="text-3xl font-extrabold tracking-tight">{t("plan.title")}</h1>
      <p className="text-sm text-muted">{t("plan.for", { month })}</p>
      <Card className="space-y-4">
        <NumberField id="plan-price" label={t("plan.price")} value={draft.price} min={l.price_min_cents / 100} max={l.price_max_cents / 100} onChange={(v) => set({ price: v })} />
        <NumberField id="plan-seasonal" label={t("plan.seasonal")} value={draft.seasonal} min={l.seasonal_min} max={l.seasonal_max} onChange={(v) => set({ seasonal: v })} />
        <Choice
          legend={t("plan.maintenance")}
          name="maintenance"
          value={draft.maintenance}
          options={[
            { v: "deferred", label: t("maintenance.deferred") },
            { v: "regular", label: t("maintenance.regular") },
            { v: "intensive", label: t("maintenance.intensive") },
          ]}
          onChange={(v) => set({ maintenance: v })}
        />
      </Card>
      <details className="rounded-card border border-line bg-surface p-4">
        <summary className="min-h-11 cursor-pointer font-bold">{t("plan.more")}</summary>
        <div className="mt-3 space-y-4">
          <NumberField id="plan-marketing" label={t("plan.marketing")} value={draft.marketing} min={0} max={l.marketing_max_cents / 100} onChange={(v) => set({ marketing: v })} />
          <NumberField id="plan-wage" label={t("plan.wage")} value={draft.wage} min={l.wage_min_pct} max={l.wage_max_pct} onChange={(v) => set({ wage: v })} />
          <Choice
            legend={t("plan.training")}
            name="training"
            value={draft.training}
            options={[
              { v: "none", label: t("training.none") },
              { v: "basic", label: t("training.basic") },
              { v: "full", label: t("training.full") },
            ]}
            onChange={(v) => set({ training: v })}
          />
          <Toggle
            id="plan-renovation"
            label={t("plan.renovation")}
            checked={draft.renovation}
            disabled={!l.renovation_available}
            note={!l.renovation_available && <p className="text-xs text-muted">{t("plan.renovationUsed")}</p>}
            onChange={(v) => set({ renovation: v })}
          />
          <Toggle id="plan-reply" label={t("plan.replyReviews")} checked={draft.reply} onChange={(v) => set({ reply: v })} />
        </div>
      </details>
      {invalid && <ErrorText>{t("plan.invalid")}</ErrorText>}
      {save.isError && <ErrorText>{problemMessage(save.error)}</ErrorText>}
      {save.isSuccess && <StatusText>{t("plan.saved", { month })}</StatusText>}
      <Button type="submit" disabled={save.isPending}>
        {t("plan.save")}
      </Button>
    </form>
  );
}
```

Napomene: (1) `exactOptionalPropertyTypes` — `disabled?: boolean` i `note?: ReactNode` u `Toggle` mogu tražiti
`| undefined`; prilagodi potpis. (2) `invalidateQueries(keys.me)` nakon spremanja ponovno dohvaća `me`, a `draft` je
lokalno stanje, pa se obrazac ne resetira — to je namjerno. (3) Tekst `{min}–{max}` za cijenu je u eurima. Linije dulje od
110 znakova Biome formatter prelama (`corepack pnpm exec biome format --write apps/web/src`).

- [ ] **Step 6: Ruta** — u `App.tsx` `case "plan": return <PlanScreen me={me} />;`. Testovi → PASS; provjere (sve četiri).

- [ ] **Step 7: Commit** — „Ekrani 8/13: plan mjeseca sa svih osam postavki“.

---

