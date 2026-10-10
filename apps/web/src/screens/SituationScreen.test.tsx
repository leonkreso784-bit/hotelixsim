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
    expect(link.querySelector("svg[aria-hidden='true']:not(.lucide)")).not.toBeNull();
    const s3link = screen.getByRole("link", { name: /Agencija nudi/ });
    expect(s3link.querySelector("svg:not(.lucide)")).toBeNull();
    expect(screen.getAllByText("Bez odgovora")).toHaveLength(2);
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
