### Task 7: Slobodan odgovor i AI obrazloženje

**Files:**
- Create: `apps/web/src/screens/situation/FreeResponse.tsx`, `apps/web/src/screens/situation/Verdict.tsx`,
  `apps/web/src/screens/situation/Response.test.tsx`
- Modify: `apps/web/src/screens/SituationScreen.tsx` (mjesto „Task 7“), `apps/web/src/api/queries.ts`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `useSituations` (ispituje dok je `pending`, Task 5), `keys`, `unwrap` (Task 4), `SituationBody`, `ResponseBody` (Task 1).
- Produces: `useSubmitResponse(hotelId, code)` — mutacija `text: string`; 202 `SituationsBody` upisuje u `keys.situations`
  (time kreće ispitivanje presude) i invalidira `keys.feed`. `FreeResponse({ hotelId, s })`, `Verdict({ response })`.

- [ ] **Step 1: Kuka**

```ts
export function useSubmitResponse(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (text: string) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/response", {
          params: { path: { hotel_id: hotelId, code } },
          body: { text },
        }),
      ),
    onSuccess: (body) => {
      qc.setQueryData(keys.situations(hotelId), body);
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}
```

- [ ] **Step 2: Tekstovi**

```ts
  "response.title": "Tvoj odgovor",
  "response.label": "Tvoj odgovor svojim riječima",
  "response.placeholder": "Napiši što radiš i zašto…",
  "response.hint": "Odgovor se predaje jednom. AI ga ocjenjuje po pet kriterija.",
  "response.submit": "Predaj odgovor",
  "response.count": "{n}/2000",
  "verdict.title": "AI obrazloženje",
  "verdict.pending": "AI ocjenjuje tvoj odgovor…",
  "verdict.total": "Ukupno {q}/5",
  "verdict.met": "ispunjeno",
  "verdict.notMet": "nije ispunjeno",
  "verdict.neutral": "Presuda nije stigla na vrijeme: neutralna ocjena 3, odgovor ide profesoru na pregled.",
  "verdict.note": "AI ocjenjuje odgovor. Pravila računaju brojke.",
```

- [ ] **Step 3: Napiši pad testova `screens/situation/Response.test.tsx`**

```tsx
import { cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ResponseBody } from "../../api/types";
import { json, problem } from "../../test/fakeApi";
import { s1, s3, situations, standardApi } from "../../test/fixtures";
import { fastPolling, renderApp } from "../../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const judged: ResponseBody = {
  text: "Ponudit ću zamjensku sobu i pozvati servis.",
  status: "judged",
  quality: 4,
  rationale: "Brza i konkretna reakcija.",
  criteria: [
    { n: 1, text: "Ispriča se i pokaže razumijevanje.", met: true, reason: "Uvažava potrebe obitelji." },
    { n: 2, text: "Ponudi konkretno rješenje odmah.", met: true, reason: "Nudi zamjensku sobu." },
    { n: 3, text: "Pošalje tehničara.", met: true, reason: "Poziva servis." },
    { n: 4, text: "Objasni sljedeći korak.", met: true, reason: "Jasno kaže što slijedi." },
    { n: 5, text: "Javi se gošći sutradan.", met: false, reason: "Nema poziva sutradan." },
  ],
};

describe("slobodan odgovor", () => {
  it("predaja jednom → „AI ocjenjuje…“ → pet redaka ✓/✗ s rečenicom i razlogom, ukupno 4/5", async () => {
    let polls = 0;
    const pending = situations([s1({ response: { ...judged, status: "pending", quality: null, criteria: [], rationale: null } }), s3()]);
    const api = standardApi()
      .on("POST", "/v1/hotels/:hotel/situations/:code/response", () => json(202, pending))
      .on("GET", "/v1/hotels/:hotel/situations", () => {
        if (api.posts("/v1/hotels/h1/situations/S1/response").length === 0) return json(200, situations());
        polls += 1;
        return json(200, polls < 3 ? pending : situations([s1({ response: judged }), s3()]));
      });
    renderApp(api, "/situacije/S1");
    const area = await screen.findByRole("textbox", { name: "Tvoj odgovor svojim riječima" });
    fireEvent.change(area, { target: { value: "Ponudit ću zamjensku sobu i pozvati servis." } });
    const submit = screen.getByRole("button", { name: "Predaj odgovor" });
    fireEvent.click(submit);
    fireEvent.click(submit);
    expect(await screen.findByText("AI ocjenjuje tvoj odgovor…")).toBeTruthy();
    expect(await screen.findByText("Ukupno 4/5")).toBeTruthy();
    expect(screen.getByText("Ispriča se i pokaže razumijevanje.")).toBeTruthy();
    expect(screen.getByText("Nema poziva sutradan.")).toBeTruthy();
    expect(screen.getAllByText("ispunjeno")).toHaveLength(4);
    expect(screen.getAllByText("nije ispunjeno")).toHaveLength(1);
    expect(screen.getByText("AI ocjenjuje odgovor. Pravila računaju brojke.")).toBeTruthy();
    expect(screen.getByText("Ponudit ću zamjensku sobu i pozvati servis.")).toBeTruthy();
    expect(api.posts("/v1/hotels/h1/situations/S1/response")).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull();
  });

  it("prazan tekst se ne može predati", async () => {
    renderApp(standardApi(), "/situacije/S1");
    const submit = await screen.findByRole("button", { name: "Predaj odgovor" });
    expect((submit as HTMLButtonElement).disabled).toBe(true);
  });

  it("422 bad_text je hrvatska rečenica", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/response", () => problem(422, "bad_text"));
    renderApp(api, "/situacije/S1");
    fireEvent.change(await screen.findByRole("textbox", { name: "Tvoj odgovor svojim riječima" }), {
      target: { value: "x" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Predaj odgovor" }));
    expect(await screen.findByText("Tekst je prazan ili predug (najviše 2000 znakova).")).toBeTruthy();
  });

  it("neutralna presuda kaže da ide profesoru", async () => {
    const neutral: ResponseBody = { ...judged, status: "neutral", quality: 3, criteria: [], rationale: null };
    const api = standardApi().on("GET", "/v1/hotels/:hotel/situations", situations([s1({ response: neutral }), s3()]));
    renderApp(api, "/situacije/S1");
    expect(
      await screen.findByText("Presuda nije stigla na vrijeme: neutralna ocjena 3, odgovor ide profesoru na pregled."),
    ).toBeTruthy();
    expect(screen.getByText("Ukupno 3/5")).toBeTruthy();
  });

  it("S3 nema polja za slobodan odgovor", async () => {
    renderApp(standardApi(), "/situacije/S3");
    await screen.findByRole("heading", { name: /Agencija nudi grupu/ });
    expect(screen.queryByRole("textbox", { name: "Tvoj odgovor svojim riječima" })).toBeNull();
  });
});
```

- [ ] **Step 4: Vidi pad.**

- [ ] **Step 5: `screens/situation/FreeResponse.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { type FormEvent, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useSubmitResponse } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, ErrorText } from "../../ui/primitives";

export function FreeResponse({ hotelId, s }: { hotelId: string; s: SituationBody }) {
  const submit = useSubmitResponse(hotelId, s.code);
  const [text, setText] = useState("");
  function onSubmit(e: FormEvent) {
    e.preventDefault();
    const answer = text.trim();
    if (!answer || submit.isPending) return;
    submit.mutate(answer);
  }
  return (
    <form onSubmit={onSubmit} className="space-y-2">
      <h2 className="text-lg font-bold">{t("response.title")}</h2>
      <textarea
        aria-label={t("response.label")}
        placeholder={t("response.placeholder")}
        maxLength={2000}
        rows={5}
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="w-full rounded-2xl border border-line bg-surface p-3"
      />
      <div className="flex justify-between text-xs text-muted">
        <span>{t("response.hint")}</span>
        <span>{t("response.count", { n: text.length })}</span>
      </div>
      {submit.isError && <ErrorText>{problemMessage(submit.error)}</ErrorText>}
      <Button type="submit" disabled={submit.isPending || !text.trim()}>
        {t("response.submit")}
      </Button>
    </form>
  );
}
```

- [ ] **Step 6: `screens/situation/Verdict.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { Check, Info, Sparkles, X } from "lucide-react";
import type { ResponseBody } from "../../api/types";
import { Card, StatusText } from "../../ui/primitives";

export function Verdict({ response }: { response: ResponseBody }) {
  return (
    <section className="space-y-3">
      <Card className="space-y-1">
        <p className="text-sm font-bold text-accent">{t("response.title")}</p>
        <p>{response.text}</p>
      </Card>
      {response.status === "pending" ? (
        <StatusText>{t("verdict.pending")}</StatusText>
      ) : (
        <Card className="space-y-3 bg-accent-soft">
          <h2 className="flex items-center gap-2 text-lg font-bold">
            <Sparkles size={18} aria-hidden="true" className="text-accent" />
            {t("verdict.title")}
          </h2>
          {response.status === "neutral" && <p className="text-sm">{t("verdict.neutral")}</p>}
          {response.quality !== null && <p className="text-2xl font-extrabold">{t("verdict.total", { q: response.quality })}</p>}
          <ul className="divide-y divide-line rounded-xl bg-surface">
            {response.criteria.map((c) => (
              <li key={c.n} className="flex gap-3 p-3">
                <span className={c.met ? "text-up" : "text-down"}>
                  {c.met ? <Check size={20} aria-hidden="true" /> : <X size={20} aria-hidden="true" />}
                </span>
                <span className="min-w-0 space-y-0.5">
                  <span className="block font-semibold">{c.text}</span>
                  <span className={`block text-xs font-bold ${c.met ? "text-up" : "text-down"}`}>
                    {c.met ? t("verdict.met") : t("verdict.notMet")}
                  </span>
                  <span className="block text-sm text-muted">{c.reason}</span>
                </span>
              </li>
            ))}
          </ul>
          {response.rationale && <p className="text-sm">{response.rationale}</p>}
        </Card>
      )}
      <p className="flex items-center gap-2 text-xs text-muted">
        <Info size={14} aria-hidden="true" />
        {t("verdict.note")}
      </p>
    </section>
  );
}
```

- [ ] **Step 7: Uvezi u `SituationScreen.tsx`** — komentar `{/* Task 7: … */}` zamijeni s:

```tsx
      {s.response && <Verdict response={s.response} />}
      {open && !decided && s.free_text && <FreeResponse hotelId={hotelId} s={s} />}
```

(+ importi). Testovi Taska 5 i 6 → PASS; provjere (sve četiri).

- [ ] **Step 8: Commit** — „Ekrani 7/13: slobodan odgovor i AI obrazloženje po kriterijima“.

---

