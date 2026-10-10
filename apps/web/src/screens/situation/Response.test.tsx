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
  it("predaja jednom → „AI ocjenjuje…“ → pet redaka ✓/✗ s rečenicom i razlogom, ocjena 4/5", async () => {
    let polls = 0;
    const pending = situations([
      s1({ response: { ...judged, status: "pending", quality: null, criteria: [], rationale: null } }),
      s3(),
    ]);
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
    expect(await screen.findByText("Ocjena 4/5")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "AI obrazloženje" })).toBeTruthy();
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
    const api = standardApi().on("POST", "/v1/hotels/:hotel/situations/:code/response", () =>
      problem(422, "bad_text"),
    );
    renderApp(api, "/situacije/S1");
    fireEvent.change(await screen.findByRole("textbox", { name: "Tvoj odgovor svojim riječima" }), {
      target: { value: "x" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Predaj odgovor" }));
    expect(await screen.findByText("Tekst je prazan ili predug (najviše 2000 znakova).")).toBeTruthy();
  });

  it("neutralna presuda: naslov „Ocjena 3/5“ i jedna rečenica, bez tehničke rečenice poslužitelja", async () => {
    const neutral: ResponseBody = {
      ...judged,
      status: "neutral",
      quality: 3,
      criteria: [],
      rationale:
        "AI presuda nije uspjela ni nakon ponovljenih pokušaja; neutralna presuda po pravilu, na pregled profesoru.",
    };
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1({ response: neutral }), s3()]),
    );
    renderApp(api, "/situacije/S1");
    expect(await screen.findByRole("heading", { name: "Ocjena 3/5" })).toBeTruthy();
    expect(
      screen.getByText(
        "AI ocjena nije stigla na vrijeme: vrijedi neutralna ocjena, a odgovor pregledava profesor.",
      ),
    ).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "AI obrazloženje" })).toBeNull();
    expect(screen.queryByText(/AI presuda nije uspjela/)).toBeNull();
    // Bez kriterija nema ni praznog okvira popisa.
    expect(document.querySelector("ul.divide-y")).toBeNull();
  });

  it("S3 nema polja za slobodan odgovor", async () => {
    renderApp(standardApi(), "/situacije/S3");
    await screen.findByRole("heading", { name: /Agencija nudi grupu/ });
    expect(screen.queryByRole("textbox", { name: "Tvoj odgovor svojim riječima" })).toBeNull();
  });
});
