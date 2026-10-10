import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { json, problem } from "../test/fakeApi";
import { HOTEL_ID, myHotel, plan, standardApi, v2Api } from "../test/fixtures";
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
    expect(screen.getByLabelText("Plaće (% uobičajene plaće)")).toBeTruthy();
    expect(screen.getByRole("radio", { name: "puna" })).toBeTruthy();
    expect(screen.getByLabelText("Obnova (jednom u sezoni)")).toBeTruthy();
    expect(screen.getByLabelText("Odgovaramo na recenzije")).toBeTruthy();
  });

  it("rasponi imaju jedinicu i grupiranje tisuća; izbor ima vidljiv fokus na oznaci", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByText("Dopušteno: 79–198 €")).toBeTruthy();
    expect(screen.getByText("Dopušteno: 18–38 osoba")).toBeTruthy();
    expect(screen.getByText("Dopušteno: 0–40.000 €")).toBeTruthy();
    expect(screen.getByText("Dopušteno: 90–115 %")).toBeTruthy();
    const label = screen.getByRole("radio", { name: "redovno" }).closest("label") as HTMLLabelElement;
    expect(label.className).toContain("has-[:focus-visible]:outline-2");
    expect(label.className).toContain("has-[:focus-visible]:outline-accent");
  });

  it("spremanje šalje cijeli plan u centima i potvrdi", async () => {
    const api = standardApi().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) =>
      json(200, body as object),
    );
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

  it("dvostruki klik na „Spremi plan“ šalje jedan zahtjev", async () => {
    const api = standardApi().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) =>
      json(200, body as object),
    );
    renderApp(api, "/plan");
    const button = await screen.findByRole("button", { name: "Spremi plan" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(await screen.findByText("Plan za lipanj je spremljen.")).toBeTruthy();
    expect(api.calls.filter((c) => c.method === "PUT")).toHaveLength(1);
  });

  it("422 od poslužitelja je hrvatska rečenica", async () => {
    const api = standardApi().on("PUT", "/v1/hotels/:hotel/plans/:month", () =>
      problem(422, "seasonal_step_too_big"),
    );
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
    expect(((await screen.findByLabelText("Obnova (jednom u sezoni)")) as HTMLInputElement).disabled).toBe(
      true,
    );
    expect(screen.getByText("Obnova je već iskorištena.")).toBeTruthy();
  });

  it("zatvoren mjesec: nema obrasca", async () => {
    renderApp(standardApi().on("GET", "/v1/me/hotel", myHotel({ planMonth: null })), "/plan");
    expect(await screen.findByText("Plan se mijenja dok je mjesec otvoren.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Spremi plan" })).toBeNull();
  });
});

describe("plan tjedna", () => {
  it("naslov i opis govore o tjednu, a plan se sprema za kod kruga t01 jednim zahtjevom", async () => {
    const api = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) =>
      json(200, body as object),
    );
    renderApp(api, "/plan");
    expect(await screen.findByRole("heading", { level: 1, name: "Plan tjedna" })).toBeTruthy();
    expect(
      screen.getByText(
        "Za tjedan 1 (1. 6. – 7. 6.). Vrijedi od zatvaranja tjedna; zadnji spremljeni plan pobjeđuje.",
      ),
    ).toBeTruthy();
    const button = screen.getByRole("button", { name: "Spremi plan" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(await screen.findByText("Plan za tjedan 1 (1. 6. – 7. 6.) je spremljen.")).toBeTruthy();
    expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1);
  });
});

describe("plan tjedna: greške", () => {
  it("krug nije otvoren i prevelik pomak sezonaca govore o tjednu, a v1 ostaje kao prije", async () => {
    const api = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () => problem(409, "round_not_open"));
    renderApp(api, "/plan");
    fireEvent.click(await screen.findByRole("button", { name: "Spremi plan" }));
    expect(await screen.findByText("Tjedan nije otvoren — sad se ne može mijenjati.")).toBeTruthy();
    cleanup();
    const api2 = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () =>
      problem(422, "seasonal_step_too_big"),
    );
    renderApp(api2, "/plan");
    fireEvent.click(await screen.findByRole("button", { name: "Spremi plan" }));
    expect(
      await screen.findByText("Sezonce možeš mijenjati samo malo odjednom; smanji promjenu."),
    ).toBeTruthy();
  });
});
