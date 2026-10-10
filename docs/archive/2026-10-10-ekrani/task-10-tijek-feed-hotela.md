### Task 10: Tijek (feed hotela)

**Files:**
- Create: `apps/web/src/screens/FeedScreen.tsx`, `apps/web/src/screens/FeedScreen.test.tsx`
- Modify: `apps/web/src/App.tsx` (ruta `feed`), `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `useFeed(hotelId)` (Task 9), `FeedChange`, `Changes` (Task 9), formatiranje (Task 1).
- Produces: `FeedScreen({ me })`; čista funkcija `feedEntries(items: FeedItemBody[]): FeedEntry[]` (izvezena radi testa) —
  `FeedEntry = { key: string; month: MonthCode | null; kind: "item"; item: Exclude<FeedItemBody, FeedChange> } |
  { key: string; month: MonthCode; kind: "changes"; changes: FeedChange[] }`; uzastopne `change` stavke istog mjeseca su
  jedna stavka; poredak **najnovije prvo**.

- [ ] **Step 1: Tekstovi**

```ts
  "feed.title": "Tijek",
  "feed.empty": "Još se ništa nije dogodilo.",
  "feed.start": "Početak",
  "feed.hotel_received": "Dobio si hotel „{name}“.",
  "feed.plan_changed": "Plan za {month} je promijenjen.",
  "feed.plan_changed.robot": "Robot je postavio plan za {month}.",
  "feed.answer_chosen": "{title}: izabrano „{label}“.",
  "feed.round_closed": "{month} je zatvoren.",
  "feed.situation_ignored": "{title}: bez odgovora (ocjena 1).",
  "feed.month_resolved": "{month} je obračunat: napredak {progress}.",
  "feed.response": "{title}: predan odgovor svojim riječima.",
  "feed.verdict": "{title}: AI ocjena {q}/5.",
  "feed.verdict.neutral": "{title}: neutralna ocjena {q}/5, ide profesoru na pregled.",
  "feed.verdict.late": "{title}: zakašnjela AI ocjena {q}/5 — ne mijenja obračun.",
  "feed.mentor": "Mentor: {text}",
  "feed.changes": "Obračun: {n} promjena s uzrokom",
```

- [ ] **Step 2: Napiši pad testova `screens/FeedScreen.test.tsx`**

```tsx
import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FeedItemBody } from "../api/types";
import { ix, plan, standardApi } from "../test/fixtures";
import { renderApp } from "../test/renderApp";
import { feedEntries } from "./FeedScreen";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const items: FeedItemBody[] = [
  { kind: "hotel_received", seq: 1, name: "Tvoj hotel", profile: "C" },
  { kind: "plan_changed", seq: 2, month: "jun", plan: plan(), by: "guest" },
  { kind: "answer_chosen", seq: 3, month: "jun", situation: "S3", title: "Agencija", answer: 1, label: "Prihvatiti uz 15 % popusta i depozit", by: "guest" },
  { kind: "response", seq: 4, month: "jun", situation: "S1", title: "Klima" },
  { kind: "round_closed", seq: 5, month: "jun", by: "guest" },
  { kind: "verdict", seq: 6, month: "jun", situation: "S1", title: "Klima", quality: 4, neutral: false, late: false },
  { kind: "change", seq: 7, month: "jun", var: "sold", cause: "price", amount: -1200, unit: "bp", situation: null },
  { kind: "change", seq: 8, month: "jun", var: "cash", cause: "interest", amount: 10_000, unit: "cents", situation: null },
  { kind: "month_resolved", seq: 9, month: "jun", indices: ix(5300, 5200, 5100, 5400, 4800), progress_hundredths: 800 },
  { kind: "mentor", seq: 10, month: "jun", text: "Usluga je bolja." },
];

describe("feedEntries", () => {
  it("najnovije prvo, uzastopne promjene istog mjeseca u jednoj stavci", () => {
    const e = feedEntries(items);
    expect(e[0]?.kind === "item" && e[0].item.kind).toBe("mentor");
    const changes = e.find((x) => x.kind === "changes");
    expect(changes?.kind === "changes" && changes.changes).toHaveLength(2);
    expect(e).toHaveLength(9);
    expect(e.at(-1)?.month).toBeNull();
  });
});

describe("Tijek", () => {
  it("rečenice po vrsti, mjeseci kao naslovi, promjene se otvaraju", async () => {
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/feed", { items }), "/tijek");
    expect(await screen.findByRole("heading", { name: "Tijek" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Lipanj" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Početak" })).toBeTruthy();
    expect(screen.getByText("Mentor: Usluga je bolja.")).toBeTruthy();
    expect(screen.getByText("Lipanj je obračunat: napredak +8.")).toBeTruthy();
    expect(screen.getByText("Klima: AI ocjena 4/5.")).toBeTruthy();
    expect(screen.getByText("Agencija: izabrano „Prihvatiti uz 15 % popusta i depozit“.")).toBeTruthy();
    expect(screen.getByText("Dobio si hotel „Tvoj hotel“.")).toBeTruthy();
    const sum = screen.getByText("Obračun: 2 promjena s uzrokom");
    fireEvent.click(sum);
    const box = sum.closest("details") as HTMLElement;
    expect(within(box).getByText("−12 %")).toBeTruthy();
  });

  it("prazan tijek", async () => {
    renderApp(standardApi(), "/tijek");
    expect(await screen.findByText("Još se ništa nije dogodilo.")).toBeTruthy();
  });
});
```

- [ ] **Step 3: Vidi pad.**

- [ ] **Step 4: `screens/FeedScreen.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../api/errors";
import { useFeed } from "../api/queries";
import type { FeedItemBody, HotelBody, MonthCode } from "../api/types";
import { capitalize, causeLabel, formatChange, formatSigned, monthName, varLabel } from "../format";
import { Card, ErrorText, StatusText } from "../ui/primitives";
import type { FeedChange } from "./month/Changes";

type Plain = Exclude<FeedItemBody, FeedChange>;
export type FeedEntry =
  | { key: string; month: MonthCode | null; kind: "item"; item: Plain }
  | { key: string; month: MonthCode; kind: "changes"; changes: FeedChange[] };

export function feedEntries(items: FeedItemBody[]): FeedEntry[] {
  const out: FeedEntry[] = [];
  for (const i of items) {
    if (i.kind === "change") {
      const last = out.at(-1);
      if (last?.kind === "changes" && last.month === i.month) last.changes.push(i);
      else out.push({ key: `c${i.seq}`, month: i.month, kind: "changes", changes: [i] });
    } else {
      out.push({ key: `i${i.seq}`, month: "month" in i ? i.month : null, kind: "item", item: i });
    }
  }
  return out.reverse();
}

function sentence(i: Plain): string {
  switch (i.kind) {
    case "hotel_received":
      return t("feed.hotel_received", { name: i.name });
    case "plan_changed":
      return t(i.by === "robot" ? "feed.plan_changed.robot" : "feed.plan_changed", { month: monthName(i.month) });
    case "answer_chosen":
      return t("feed.answer_chosen", { title: i.title, label: i.label });
    case "round_closed":
      return t("feed.round_closed", { month: capitalize(monthName(i.month)) });
    case "situation_ignored":
      return t("feed.situation_ignored", { title: i.title });
    case "month_resolved":
      return t("feed.month_resolved", { month: capitalize(monthName(i.month)), progress: formatSigned(i.progress_hundredths) });
    case "response":
      return t("feed.response", { title: i.title });
    case "verdict":
      return t(i.late ? "feed.verdict.late" : i.neutral ? "feed.verdict.neutral" : "feed.verdict", {
        title: i.title,
        q: i.quality,
      });
    case "mentor":
      return t("feed.mentor", { text: i.text });
  }
}

export function FeedScreen({ me }: { me: HotelBody }) {
  const feed = useFeed(me.hotel.hotel_id);
  if (feed.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (feed.isError) return <ErrorText>{problemMessage(feed.error)}</ErrorText>;
  const entries = feedEntries(feed.data);
  const groups: { month: MonthCode | null; entries: FeedEntry[] }[] = [];
  for (const e of entries) {
    const g = groups.at(-1);
    if (g && g.month === e.month) g.entries.push(e);
    else groups.push({ month: e.month, entries: [e] });
  }
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("feed.title")}</h1>
      {entries.length === 0 && <p className="text-muted">{t("feed.empty")}</p>}
      {groups.map((g) => (
        <section key={`${g.month ?? "start"}-${g.entries[0]?.key}`} className="space-y-2">
          <h2 className="pt-2 text-sm font-bold tracking-wide text-muted uppercase">
            {g.month ? capitalize(monthName(g.month)) : t("feed.start")}
          </h2>
          <Card className="divide-y divide-line py-1">
            <ol>
              {g.entries.map((e) =>
                e.kind === "item" ? (
                  <li key={e.key} className="py-2">
                    {sentence(e.item)}
                  </li>
                ) : (
                  <li key={e.key} className="py-2">
                    <details>
                      <summary className="min-h-11 cursor-pointer font-semibold">
                        {t("feed.changes", { n: e.changes.length })}
                      </summary>
                      <ul className="text-sm">
                        {e.changes.map((c) => (
                          <li key={c.seq} className="flex justify-between gap-3 py-0.5">
                            <span className="text-muted">
                              {varLabel(c.var)} · {causeLabel(c.cause)}
                            </span>
                            <span className="tabular-nums">{formatChange(c.amount, c.unit)}</span>
                          </li>
                        ))}
                      </ul>
                    </details>
                  </li>
                ),
              )}
            </ol>
          </Card>
        </section>
      ))}
    </>
  );
}
```

Napomena: test očekuje naslov „Lipanj“ (veliko početno slovo kroz CSS `uppercase` ne mijenja tekst u DOM-u — `capitalize`
daje „Lipanj“); ako `getByRole("heading", { name: "Lipanj" })` ne nađe zbog `uppercase` — ne nađe se jer jsdom ne
primjenjuje CSS, pa je u redu. „Obračun: 2 promjena“ je gramatički nespretno za 2–4 — prihvaćeno za demo, zapiši u izvještaj.

- [ ] **Step 5: Ruta** — `case "feed": return <FeedScreen me={me} />;`. Testovi → PASS; provjere (sve četiri).

- [ ] **Step 6: Commit** — „Ekrani 10/13: tijek hotela“.

---

