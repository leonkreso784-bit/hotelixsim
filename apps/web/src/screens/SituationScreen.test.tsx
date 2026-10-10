import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../test/fakeApi";
import { conversation, game, s1, s3, situations, standardApi } from "../test/fixtures";
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
    expect(link.querySelector("svg[aria-hidden='true']:not(.lucide)")).not.toBeNull();
    const s3link = screen.getByRole("link", { name: /Agencija nudi/ });
    expect(s3link.querySelector("svg:not(.lucide)")).toBeNull();
    expect(screen.getAllByText("Bez odgovora")).toHaveLength(2);
  });
});

describe("popis situacija nakon kraja sezone", () => {
  it("jedna rečenica s poveznicom na Tijek, bez proturječnih rečenica (I11)", async () => {
    const api = standardApi()
      .on("GET", "/v1/games/:game", game({ current: null }))
      .on("GET", "/v1/hotels/:hotel/situations", situations([], { open: false, unanswered: 0 }));
    renderApp(api, "/situacije");
    expect(await screen.findByText("Sezona je završena. Sve odluke su u Tijeku.")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Otvori Tijek" }).getAttribute("href")).toBe("/tijek");
    expect(screen.queryByText("Ovaj mjesec nema situacija.")).toBeNull();
    expect(screen.queryByText("Mjesec je zatvoren — situacije su samo za čitanje.")).toBeNull();
    // m4: bez situacija nema ni praznog popisa.
    expect(document.querySelector("main ul")).toBeNull();
  });

  it("zatvoren mjesec usred sezone: samo rečenica „samo za čitanje“", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1(), s3()], { open: false, unanswered: 0 }),
    );
    renderApp(api, "/situacije");
    expect(await screen.findByText("Mjesec je zatvoren — situacije su samo za čitanje.")).toBeTruthy();
    expect(screen.queryByText("Ovaj mjesec nema situacija.")).toBeNull();
    expect(screen.queryByText("Sezona je završena. Sve odluke su u Tijeku.")).toBeNull();
  });
});

describe("situacija s likom (otvorena izravno, kao nakon osvježavanja)", () => {
  it("pokazuje opis iz prve rečenice lika, inicijale i brojač razgovora", async () => {
    renderApp(standardApi(), "/situacije/S1");
    expect(await screen.findByRole("heading", { name: "Klima u sobi obitelji s djecom" })).toBeTruthy();
    expect(
      await screen.findByText("Klima u sobi ne radi, a djeca ne mogu spavati. Vani je 34 °C!"),
    ).toBeTruthy();
    expect(screen.getByText("Petra Horvat")).toBeTruthy();
    expect(screen.getByText("Razgovor · 0/6")).toBeTruthy();
  });

  it("poruka: šalje se jednom, „piše…“ dok lik ne odgovori, zatim odgovor", async () => {
    let polls = 0;
    let posted = false;
    const waiting = conversation({
      lines: [{ from: "student", text: "Možemo li dobiti drugu sobu?" }],
      remaining: 5,
      awaiting_reply: true,
    });
    const api = standardApi()
      .on("POST", "/v1/hotels/:hotel/situations/:code/messages", () => {
        posted = true;
        return json(
          202,
          conversation({
            lines: [{ from: "student", text: "Možemo li dobiti drugu sobu?" }],
            remaining: 5,
            awaiting_reply: true,
          }),
        );
      })
      .on("GET", "/v1/hotels/:hotel/situations/:code/conversation", () => {
        if (!posted) return json(200, conversation());
        polls += 1;
        if (polls < 3) return json(200, waiting);
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
    expect(api.posts("/v1/hotels/h1/situations/S1/messages")[0]?.body).toEqual({
      text: "Možemo li dobiti drugu sobu?",
    });
  });

  it("gumb Pošalji ne gazi polje poruke: nije pune širine, polje se širi", async () => {
    renderApp(standardApi(), "/situacije/S1");
    const box = await screen.findByRole("textbox", { name: "Poruka liku" });
    expect(box.className).toContain("flex-1");
    expect(box.className).toContain("min-w-0");
    const send = screen.getByRole("button", { name: "Pošalji" });
    expect(send.className).not.toContain("w-full");
  });

  it("409 awaiting_reply je hrvatska rečenica", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/messages", () =>
      problem(409, "awaiting_reply"),
    );
    renderApp(api, "/situacije/S1");
    fireEvent.change(await screen.findByRole("textbox", { name: "Poruka liku" }), {
      target: { value: "Halo?" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Pošalji" }));
    expect(await screen.findByText("Pričekaj da lik odgovori.")).toBeTruthy();
  });

  it("bez preostalih poruka nema polja za poruku", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations/:code/conversation",
      conversation({
        remaining: 0,
        lines: Array.from({ length: 6 }, () => ({ from: "student" as const, text: "x" })),
      }),
    );
    renderApp(api, "/situacije/S1");
    expect(await screen.findByText("Poruke za ovaj razgovor su potrošene. Sad odgovori.")).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: "Poruka liku" })).toBeNull();
  });
});

describe("ponuđeni odgovori (S3, bez lika)", () => {
  it("izbor u dva koraka: oznaka, pa „Potvrdi izbor“ — šalje se jednom i prikaže kao „Tvoj izbor“", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/answer", () =>
      json(200, situations([s1(), s3({ chosen: 1 })])),
    );
    renderApp(api, "/situacije/S3");
    expect(await screen.findByRole("heading", { name: /Agencija nudi grupu/ })).toBeTruthy();
    const buttons = screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" });
    expect(buttons).toHaveLength(4);
    // Svaki od četiri ista gumba čitač zaslona razlikuje po opisu: tekstu svog odgovora (I8).
    const described = buttons.map((b) => {
      const id = b.getAttribute("aria-describedby") ?? "";
      return document.getElementById(id)?.textContent;
    });
    expect(described).toEqual(s3().answers);
    fireEvent.click(buttons[1] as HTMLElement);
    // Prvi klik samo označi odgovor: nema zahtjeva, fokus je na potvrdi.
    const title = await screen.findByRole("heading", { name: "Izabrani odgovor" });
    expect(document.activeElement).toBe(title);
    expect(buttons[1]?.getAttribute("aria-pressed")).toBe("true");
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")).toHaveLength(0);
    const confirm = screen.getByRole("button", { name: "Potvrdi izbor" });
    fireEvent.click(confirm);
    fireEvent.click(confirm);
    expect(await screen.findByText("Tvoj izbor")).toBeTruthy();
    expect(screen.getByText("Prihvatiti uz 15 % popusta i depozit")).toBeTruthy();
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")).toHaveLength(1);
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")[0]?.body).toEqual({ answer: 1 });
    await waitFor(() => expect(screen.queryByRole("button", { name: "Izaberi ovaj odgovor" })).toBeNull());
  });

  it("m1: nakon „Potvrdi izbor“ fokus ide na „Tvoj izbor“, ne na <body>", async () => {
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/answer", () =>
      json(200, situations([s1(), s3({ chosen: 1 })])),
    );
    renderApp(api, "/situacije/S3");
    const buttons = await screen.findAllByRole("button", { name: "Izaberi ovaj odgovor" });
    fireEvent.click(buttons[1] as HTMLElement);
    fireEvent.click(await screen.findByRole("button", { name: "Potvrdi izbor" }));
    const chosen = await screen.findByText("Tvoj izbor");
    await waitFor(() => expect(document.activeElement).toBe(chosen));
  });

  it("m1: već izabran odgovor otvoren izravno ne otima fokus", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1(), s3({ chosen: 1 })]),
    );
    renderApp(api, "/situacije/S3");
    const chosen = await screen.findByText("Tvoj izbor");
    expect(document.activeElement).not.toBe(chosen);
  });

  it("m3: izabrana kartica ima svoju oznaku „Označeno“, ostale ostaju „Izaberi ovaj odgovor“", async () => {
    renderApp(standardApi(), "/situacije/S3");
    const buttons = await screen.findAllByRole("button", { name: "Izaberi ovaj odgovor" });
    fireEvent.click(buttons[2] as HTMLElement);
    await screen.findByRole("heading", { name: "Izabrani odgovor" });
    expect(buttons[2]?.textContent).toBe("Označeno");
    expect(screen.getAllByRole("button", { name: "Izaberi ovaj odgovor" })).toHaveLength(3);
  });

  it("„Odustani“ u potvrdi izbora: bez zahtjeva, fokus natrag na isti gumb", async () => {
    const api = standardApi();
    renderApp(api, "/situacije/S3");
    const buttons = await screen.findAllByRole("button", { name: "Izaberi ovaj odgovor" });
    fireEvent.click(buttons[2] as HTMLElement);
    fireEvent.click(await screen.findByRole("button", { name: "Odustani" }));
    expect(screen.queryByRole("heading", { name: "Izabrani odgovor" })).toBeNull();
    expect(document.activeElement).toBe(buttons[2]);
    expect(api.posts("/v1/hotels/h1/situations/S3/answer")).toHaveLength(0);
  });

  it("bez lika su povratak, naslov i odgovori u istom stupcu za čitanje (I5)", async () => {
    renderApp(standardApi(), "/situacije/S3");
    const h1 = await screen.findByRole("heading", { level: 1, name: /Agencija nudi grupu/ });
    const column = h1.closest("[data-layout='reading']") as HTMLElement;
    expect(column.className).toContain("max-w-2xl");
    expect(column.contains(screen.getByRole("link", { name: "Sve situacije" }))).toBe(true);
    expect(column.contains(screen.getByRole("heading", { name: "Ponuđeni odgovori" }))).toBe(true);
  });

  it("zatvoren mjesec bez odgovora: samo poruka, bez gumba", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1(), s3()], { open: false, unanswered: 0 }),
    );
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
