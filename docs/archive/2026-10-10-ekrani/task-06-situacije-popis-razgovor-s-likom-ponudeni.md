### Task 6: Situacije — popis, razgovor s likom, ponuđeni odgovori

**Files:**
- Create: `apps/web/src/screens/SituationsScreen.tsx`, `apps/web/src/screens/SituationScreen.tsx`,
  `apps/web/src/screens/situation/Conversation.tsx`, `apps/web/src/screens/situation/OfferedAnswers.tsx`,
  `apps/web/src/screens/SituationScreen.test.tsx`
- Modify: `apps/web/src/App.tsx` (rute `situations`, `situation`), `apps/web/src/api/queries.ts`, `packages/i18n/src/hr.ts`

**Interfaces:**
- Consumes: `useSituations` (Task 5), `useApi`, `keys`, `unwrap`, `poll`, `problemMessage` (Task 4), `Avatar`, `lookFor`, `PLAYER_LOOK` (Task 3),
  `Card`, `Button`, `Badge`, `StatusText`, `ErrorText` (Task 2).
- Produces:
  - `useConversation(hotelId, code, enabled: boolean)` — ispituje svakih `poll.conversationMs` dok je `awaiting_reply`.
  - `useSendMessage(hotelId, code)` — mutacija `text: string`; uspjeh upisuje odgovor u `keys.conversation`.
  - `useAnswer(hotelId, code)` — mutacija `answer: number`; uspjeh upisuje `SituationsBody` u `keys.situations` i
    invalidira `keys.feed`.
  - `SituationScreen({ me, code })` sa slotom za Task 7: komponenta `<ResponseSlot situation={…} hotelId={…} open={…} />`
    se u ovom tasku NE pravi; Task 7 je dodaje. Ovdje odgovor-dio prikazuje samo `OfferedAnswers` ili izabrani odgovor.

- [ ] **Step 1: Kuke u `api/queries.ts`**

```ts
export function useConversation(hotelId: string, code: string, enabled: boolean) {
  const api = useApi();
  return useQuery<ConversationBody>({
    queryKey: keys.conversation(hotelId, code),
    enabled,
    queryFn: async () =>
      unwrap(
        await api.GET("/v1/hotels/{hotel_id}/situations/{code}/conversation", {
          params: { path: { hotel_id: hotelId, code } },
        }),
      ),
    refetchInterval: (q) => (q.state.data?.awaiting_reply ? poll.conversationMs : false),
  });
}

export function useSendMessage(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (text: string) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/messages", {
          params: { path: { hotel_id: hotelId, code } },
          body: { text },
        }),
      ),
    onSuccess: (conv) => qc.setQueryData(keys.conversation(hotelId, code), conv),
  });
}

export function useAnswer(hotelId: string, code: string) {
  const api = useApi();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (answer: number) =>
      unwrap(
        await api.POST("/v1/hotels/{hotel_id}/situations/{code}/answer", {
          params: { path: { hotel_id: hotelId, code } },
          body: { answer },
        }),
      ),
    onSuccess: (body) => {
      qc.setQueryData(keys.situations(hotelId), body);
      void qc.invalidateQueries({ queryKey: keys.feed(hotelId) });
    },
  });
}
```

- [ ] **Step 2: Tekstovi u `hr.ts`**

```ts
  "situations.title": "Situacije mjeseca",
  "situations.month": "{month}",
  "situations.closed": "Mjesec je zatvoren — situacije su samo za čitanje.",
  "situations.none": "Ovaj mjesec nema situacija.",
  "situations.talk": "Razgovor s likom",
  "situation.status.open": "Bez odgovora",
  "situation.status.answered": "Odgovoreno",
  "situation.status.pending": "AI ocjenjuje…",
  "situation.status.quality": "Ocjena {q}/5",
  "situation.back": "Sve situacije",
  "situation.missing": "Ta situacija nije u ovom mjesecu.",
  "chat.progress": "Razgovor · {used}/{total}",
  "chat.you": "Ti",
  "chat.typing": "{name} piše…",
  "chat.placeholder": "Napiši poruku…",
  "chat.send": "Pošalji",
  "chat.label": "Poruka liku",
  "chat.limit": "Potrošio si sve poruke. Sad odgovori.",
  "answers.title": "Ponuđeni odgovori",
  "answers.orTitle": "Ili izaberi jedan od ponuđenih",
  "answers.choose": "Izaberi ovaj odgovor",
  "answers.chosen": "Tvoj izbor",
  "answers.missed": "Mjesec je zatvoren bez odgovora.",
```

- [ ] **Step 3: Napiši pad testova `screens/SituationScreen.test.tsx`**

```tsx
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../test/fakeApi";
import { conversation, s1, s3, situations, standardApi } from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("popis situacija", () => {
  it("pokazuje naslove, stanje i oznaku razgovora; vodi na situaciju", async () => {
    renderApp(standardApi(), "/situacije");
    expect(await screen.findByRole("heading", { name: "Situacije mjeseca" })).toBeTruthy();
    const link = screen.getByRole("link", { name: /Klima u sobi obitelji s djecom/ });
    expect(link.getAttribute("href")).toBe("/situacije/S1");
    expect(link.textContent).toContain("Razgovor s likom");
    expect(screen.getAllByText("Bez odgovora")).toHaveLength(2);
  });
});

describe("situacija s likom (otvorena izravno, kao nakon osvježavanja)", () => {
  it("pokazuje opis iz prve rečenice lika, inicijale i brojač razgovora", async () => {
    renderApp(standardApi(), "/situacije/S1");
    expect(await screen.findByRole("heading", { name: "Klima u sobi obitelji s djecom" })).toBeTruthy();
    expect(await screen.findByText("Klima u sobi ne radi, a djeca ne mogu spavati. Vani je 34 °C!")).toBeTruthy();
    expect(screen.getByText("Petra Horvat")).toBeTruthy();
    expect(screen.getByText("Razgovor · 0/6")).toBeTruthy();
  });

  it("poruka: šalje se jednom, „piše…“ dok lik ne odgovori, zatim odgovor", async () => {
    let polls = 0;
    const api = standardApi()
      .on("POST", "/v1/hotels/:hotel/situations/:code/messages", () =>
        json(
          202,
          conversation({ lines: [{ from: "student", text: "Možemo li dobiti drugu sobu?" }], remaining: 5, awaiting_reply: true }),
        ),
      )
      .on("GET", "/v1/hotels/:hotel/situations/:code/conversation", () => {
        polls += 1;
        if (polls < 3) return json(200, conversation());
        return json(
          200,
          conversation({
            lines: [
              { from: "student", text: "Možemo li dobiti drugu sobu?" },
              { from: "character", text: "Provjerit ću slobodne sobe." },
            ],
            remaining: 5,
          }),
        );
      });
    renderApp(api, "/situacije/S1");
    const box = await screen.findByRole("textbox", { name: "Poruka liku" });
    fireEvent.change(box, { target: { value: "Možemo li dobiti drugu sobu?" } });
    const send = screen.getByRole("button", { name: "Pošalji" });
    fireEvent.click(send);
    fireEvent.click(send);
    expect(await screen.findByText("Petra Horvat piše…")).toBeTruthy();
    expect(await screen.findByText("Provjerit ću slobodne sobe.")).toBeTruthy();
    expect(screen.getByText("Razgovor · 1/6")).toBeTruthy();
    expect(api.posts("/v1/hotels/h1/situations/S1/messages")).toHaveLength(1);
    expect(api.posts("/v1/hotels/h1/situations/S1/messages")[0]?.body).toEqual({ text: "Možemo li dobiti drugu sobu?" });
  });

  it("409 awaiting_reply je hrvatska rečenica", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/messages", () =>
      problem(409, "awaiting_reply"),
    );
    renderApp(api, "/situacije/S1");
    fireEvent.change(await screen.findByRole("textbox", { name: "Poruka liku" }), { target: { value: "Halo?" } });
    fireEvent.click(screen.getByRole("button", { name: "Pošalji" }));
    expect(await screen.findByText("Pričekaj da lik odgovori.")).toBeTruthy();
  });

  it("bez preostalih poruka nema polja za poruku", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations/:code/conversation",
      conversation({ remaining: 0, lines: Array.from({ length: 6 }, () => ({ from: "student" as const, text: "x" })) }),
    );
    renderApp(api, "/situacije/S1");
    expect(await screen.findByText("Potrošio si sve poruke. Sad odgovori.")).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: "Poruka liku" })).toBeNull();
  });
});

describe("ponuđeni odgovori (S3, bez lika)", () => {
  it("izbor se šalje jednom i prikaže kao „Tvoj izbor“", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/answer", () =>
      json(200, situations([s1(), s3({ chosen: 1 })])),
    );
    renderApp(api, "/situacije/S3");
    expect(await screen.findByRole("heading", { name: /Agencija nudi grupu/ })).toBeTruthy();
    const buttons = screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" });
    expect(buttons).toHaveLength(4);
    fireEvent.click(buttons[1] as HTMLElement);
    fireEvent.click(buttons[1] as HTMLElement);
    expect(await screen.findByText("Tvoj izbor")).toBeTruthy();
    expect(screen.getByText("Prihvatiti uz 15 % popusta i depozit")).toBeTruthy();
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")).toHaveLength(1);
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")[0]?.body).toEqual({ answer: 1 });
    await waitFor(() => expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull());
  });

  it("zatvoren mjesec bez odgovora: samo poruka, bez gumba", async () => {
    const api = standardApi().on("GET", "/v1/hotels/:hotel/situations", situations([s1(), s3()], { open: false, unanswered: 0 }));
    renderApp(api, "/situacije/S3");
    expect(await screen.findByText("Mjesec je zatvoren bez odgovora.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull();
  });

  it("nepoznata šifra: poruka i povratak na popis", async () => {
    renderApp(standardApi(), "/situacije/S9");
    expect(await screen.findByText("Ta situacija nije u ovom mjesecu.")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Sve situacije" }).getAttribute("href")).toBe("/situacije");
  });
});
```

- [ ] **Step 4: Vidi pad.**

- [ ] **Step 5: `screens/SituationsScreen.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { ChevronRight, MessageCircle } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useSituations } from "../api/queries";
import type { HotelBody, SituationBody } from "../api/types";
import { Link } from "../router";
import { Badge, Card, ErrorText, StatusText } from "../ui/primitives";

export function situationStatus(s: SituationBody): { text: string; tone: "accent" | "muted" | "up" } {
  if (s.response?.status === "pending") return { text: t("situation.status.pending"), tone: "accent" };
  if (s.response?.quality != null) return { text: t("situation.status.quality", { q: s.response.quality }), tone: "up" };
  if (s.chosen !== null || s.response) return { text: t("situation.status.answered"), tone: "up" };
  return { text: t("situation.status.open"), tone: "muted" };
}

export function SituationsScreen({ me }: { me: HotelBody }) {
  const sits = useSituations(me.hotel.hotel_id);
  if (sits.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (sits.isError) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("situations.title")}</h1>
      {!sits.data.open && <p className="text-muted">{t("situations.closed")}</p>}
      {sits.data.items.length === 0 && <p className="text-muted">{t("situations.none")}</p>}
      <ul className="space-y-3">
        {sits.data.items.map((s) => {
          const st = situationStatus(s);
          return (
            <li key={s.code}>
              <Link to={`/situacije/${encodeURIComponent(s.code)}`} className="block">
                <Card className="flex items-center gap-3">
                  <span className="min-w-0 flex-1 space-y-1">
                    <span className="block font-bold">{s.title}</span>
                    {s.character && (
                      <span className="flex items-center gap-1 text-sm text-muted">
                        <MessageCircle size={14} aria-hidden="true" />
                        {t("situations.talk")}
                      </span>
                    )}
                    <Badge tone={st.tone}>{st.text}</Badge>
                  </span>
                  <ChevronRight size={18} aria-hidden="true" className="text-muted" />
                </Card>
              </Link>
            </li>
          );
        })}
      </ul>
    </>
  );
}
```

- [ ] **Step 6: `screens/situation/Conversation.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { type FormEvent, useState } from "react";
import { problemMessage } from "../../api/errors";
import { useConversation, useSendMessage } from "../../api/queries";
import { Avatar } from "../../avatar/Avatar";
import { lookFor, PLAYER_LOOK } from "../../avatar/look";
import { Button, ErrorText, StatusText } from "../../ui/primitives";

/** Razgovor s likom (do 6 poruka). Prva rečenica lika je ujedno opis situacije. */
export function Conversation({ hotelId, code, canWrite }: { hotelId: string; code: string; canWrite: boolean }) {
  const conv = useConversation(hotelId, code, true);
  const send = useSendMessage(hotelId, code);
  const [text, setText] = useState("");
  if (conv.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (conv.isError) return <ErrorText>{problemMessage(conv.error)}</ErrorText>;
  const c = conv.data;
  const used = c.lines.filter((l) => l.from === "student").length;
  const total = used + c.remaining;
  const look = lookFor(c.character_name);

  function submit(e: FormEvent) {
    e.preventDefault();
    const msg = text.trim();
    if (!msg || send.isPending) return;
    send.mutate(msg, { onSuccess: () => setText("") });
  }

  return (
    <section className="space-y-3">
      <div className="flex items-center gap-3">
        <Avatar look={look} size={56} />
        <div>
          <p className="font-bold">{c.character_name}</p>
          <p className="text-sm text-muted">{c.role}</p>
        </div>
      </div>
      <ul className="space-y-2">
        <li className="flex items-end gap-2">
          <Avatar look={look} size={36} />
          <p className="rounded-2xl rounded-bl-sm bg-surface px-3 py-2 shadow-sm">{c.opening}</p>
        </li>
        {c.lines.map((l, i) =>
          l.from === "student" ? (
            // biome-ignore lint/suspicious/noArrayIndexKey: retci razgovora nemaju id, poredak se ne mijenja
            <li key={i} className="flex items-end justify-end gap-2">
              <p className="rounded-2xl rounded-br-sm bg-accent-soft px-3 py-2">{l.text}</p>
              <Avatar look={PLAYER_LOOK} size={36} />
            </li>
          ) : (
            // biome-ignore lint/suspicious/noArrayIndexKey: retci razgovora nemaju id, poredak se ne mijenja
            <li key={i} className="flex items-end gap-2">
              <Avatar look={look} size={36} />
              <p className="rounded-2xl rounded-bl-sm bg-surface px-3 py-2 shadow-sm">{l.text}</p>
            </li>
          ),
        )}
      </ul>
      {c.awaiting_reply && <StatusText>{t("chat.typing", { name: c.character_name })}</StatusText>}
      <div>
        <p className="text-sm text-muted">{t("chat.progress", { used, total })}</p>
        <div className="mt-1 flex gap-1" aria-hidden="true">
          {Array.from({ length: total }, (_, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: crtice brojača
            <span key={i} className={`h-1.5 flex-1 rounded-full ${i < used ? "bg-accent" : "bg-line"}`} />
          ))}
        </div>
      </div>
      {send.isError && <ErrorText>{problemMessage(send.error)}</ErrorText>}
      {canWrite && c.open && c.remaining === 0 && <p className="text-sm font-semibold">{t("chat.limit")}</p>}
      {canWrite && c.open && c.remaining > 0 && (
        <form onSubmit={submit} className="flex gap-2">
          <input
            aria-label={t("chat.label")}
            placeholder={t("chat.placeholder")}
            maxLength={2000}
            value={text}
            onChange={(e) => setText(e.target.value)}
            className="min-h-12 min-w-0 flex-1 rounded-2xl border border-line bg-surface px-3"
          />
          <Button type="submit" className="w-auto" disabled={send.isPending || c.awaiting_reply || !text.trim()}>
            {t("chat.send")}
          </Button>
        </form>
      )}
    </section>
  );
}
```

Pazi: u testu „šalje se jednom“ drugi klik mora pogoditi onemogućen gumb ili `send.isPending` provjeru. Ako se zbog
`setText("")` nakon uspjeha gumb onemogući kasno, zaštita u `submit` (`send.isPending`) je ona koja vrijedi.

- [ ] **Step 7: `screens/situation/OfferedAnswers.tsx`**

```tsx
import { t } from "@hotelixsim/i18n";
import { problemMessage } from "../../api/errors";
import { useAnswer } from "../../api/queries";
import type { SituationBody } from "../../api/types";
import { Button, Card, ErrorText } from "../../ui/primitives";

export function OfferedAnswers({ hotelId, s, heading }: { hotelId: string; s: SituationBody; heading: string }) {
  const answer = useAnswer(hotelId, s.code);
  return (
    <section className="space-y-3">
      <h2 className="text-lg font-bold">{heading}</h2>
      {answer.isError && <ErrorText>{problemMessage(answer.error)}</ErrorText>}
      <ol className="space-y-2">
        {s.answers.map((label, i) => (
          <li key={label}>
            <Card className="space-y-2 p-3">
              <p>{label}</p>
              <Button variant="secondary" disabled={answer.isPending} onClick={() => answer.mutate(i)}>
                {t("answers.choose")}
              </Button>
            </Card>
          </li>
        ))}
      </ol>
    </section>
  );
}

export function ChosenAnswer({ s }: { s: SituationBody }) {
  if (s.chosen === null) return null;
  return (
    <Card className="space-y-1 border-accent">
      <p className="text-sm font-bold text-accent">{t("answers.chosen")}</p>
      <p>{s.answers[s.chosen]}</p>
    </Card>
  );
}
```

- [ ] **Step 8: `screens/SituationScreen.tsx`** (Task 7 dodaje slobodan odgovor i presudu na mjesto označeno komentarom)

```tsx
import { t } from "@hotelixsim/i18n";
import { ChevronLeft } from "lucide-react";
import { problemMessage } from "../api/errors";
import { useSituations } from "../api/queries";
import type { HotelBody } from "../api/types";
import { Link } from "../router";
import { ErrorText, StatusText } from "../ui/primitives";
import { Conversation } from "./situation/Conversation";
import { ChosenAnswer, OfferedAnswers } from "./situation/OfferedAnswers";

export function SituationScreen({ me, code }: { me: HotelBody; code: string }) {
  const hotelId = me.hotel.hotel_id;
  const sits = useSituations(hotelId);
  const back = (
    <Link to="/situacije" className="inline-flex min-h-11 items-center gap-1 font-semibold text-accent">
      <ChevronLeft size={18} aria-hidden="true" />
      {t("situation.back")}
    </Link>
  );
  if (sits.isPending) return <StatusText>{t("status.loading")}</StatusText>;
  if (sits.isError) return <ErrorText>{problemMessage(sits.error)}</ErrorText>;
  const s = sits.data.items.find((i) => i.code === code);
  if (!s)
    return (
      <>
        {back}
        <p>{t("situation.missing")}</p>
      </>
    );
  const open = sits.data.open;
  const decided = s.chosen !== null || s.response !== null;
  return (
    <>
      {back}
      <h1 className="text-3xl leading-tight font-extrabold tracking-tight">{s.title}</h1>
      {s.character && <Conversation hotelId={hotelId} code={s.code} canWrite={open && !decided} />}
      {/* Task 7: slobodan odgovor i presuda */}
      <ChosenAnswer s={s} />
      {open && !decided && (
        <OfferedAnswers hotelId={hotelId} s={s} heading={s.free_text ? t("answers.orTitle") : t("answers.title")} />
      )}
      {!open && !decided && <p className="font-semibold">{t("answers.missed")}</p>}
    </>
  );
}
```

- [ ] **Step 9: Rute u `App.tsx`** — u `Screen` dodaj:

```tsx
    case "situations":
      return <SituationsScreen me={me} />;
    case "situation":
      return <SituationScreen me={me} code={route.code} />;
```

(+ importi). Pokreni testove → PASS; provjere (sve četiri).

- [ ] **Step 10: Commit** — „Ekrani 6/13: situacije, razgovor s likom i ponuđeni odgovori“.

---

