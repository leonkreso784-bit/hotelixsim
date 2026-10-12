import { act, cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PlanBody } from "../api/types";
import { draftKey, readDraft, writeDraft } from "../drafts";
import { navigate } from "../router";
import { json, problem } from "../test/fakeApi";
import { HOTEL_ID, myHotel, plan, standardApi, v2Api, v2PlanHotel } from "../test/fixtures";
import { joined } from "../test/joined";
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
    expect(await screen.findByText("Sezonske radnike možeš mijenjati najviše za 10 mjesečno.")).toBeTruthy();
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
        joined(
          "Za tjedan 1 (1. 6. – 7. 6.). Vrijedi od zatvaranja tjedna; zadnji spremljeni plan pobjeđuje.",
        ),
      ),
    ).toBeTruthy();
    const button = screen.getByRole("button", { name: "Spremi plan" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(await screen.findByText(joined("Plan za tjedan 1 (1. 6. – 7. 6.) je spremljen."))).toBeTruthy();
    expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1);
  });
});

describe("plan tjedna: odjeli", () => {
  const echo = () =>
    v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));

  it("plan nema kontrole odjela ni sezonske radnike, nego sažetak s poveznicom na odjele", async () => {
    renderApp(echo(), "/plan");
    expect(await screen.findByRole("heading", { level: 2, name: "Odjeli" })).toBeTruthy();
    expect(await screen.findByText("Odjeli ovaj tjedan rade kao dosad.")).toBeTruthy();
    expect(screen.getByRole("link", { name: "Svi odjeli" }).getAttribute("href")).toBe("/odjeli");
    expect(screen.queryByRole("group", { name: "Restoran" })).toBeNull();
    expect(screen.queryByRole("button", { name: /sezonskog radnika/ })).toBeNull();
    expect(screen.queryByLabelText("Sezonski radnici")).toBeNull();
    expect(screen.queryByRole("radio", { name: "redovno" })).toBeNull();
    expect(document.body.textContent).not.toMatch(/sezonc/i);
  });

  it("sažetak promjena po odjelu vodi na detalj odjela", async () => {
    const hotel = v2PlanHotel();
    const p = hotel.private?.plan?.plan;
    if (p?.departments?.restaurant)
      p.departments.restaurant = { ...p.departments.restaurant, seasonal_target: 4, price_level_pct: 105 };
    renderApp(echo().on("GET", "/v1/me/hotel", hotel), "/plan");
    const link = await screen.findByRole("link", {
      name: "Restoran: +2 sezonska radnika, cijene 105\u00a0%",
    });
    expect(link.getAttribute("href")).toBe("/odjeli/restaurant");
  });

  it("spremanje plana hotela šalje svježi plan kruga s nepromijenjenim odjelima", async () => {
    let reads = 0;
    const fresh = v2PlanHotel();
    const fp = fresh.private?.plan?.plan;
    if (fp?.departments?.restaurant)
      fp.departments.restaurant = { ...fp.departments.restaurant, seasonal_target: 4 };
    const api = echo().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : fresh);
    });
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "140" } });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body).toEqual({ ...fp, price_cents: 14_000 });
  });

  it("v1 igra nema dio Odjeli", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Odjeli" })).toBeNull();
  });
});

describe("plan tjedna: greške", () => {
  it("krug nije otvoren i prevelik pomak sezonskih radnika govore o tjednu, a v1 ostaje kao prije", async () => {
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
      await screen.findByText("Broj sezonskih radnika možeš mijenjati samo malo odjednom; smanji promjenu."),
    ).toBeTruthy();
  });
});

describe("plan: nespremljene izmjene (I1)", () => {
  it("upisana cijena preživi odlazak u odjele i povratak; nakon spremanja nema napomene", async () => {
    const api = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) =>
      json(200, body as object),
    );
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "150" } });
    expect(screen.getByText("Izmjene još nisu spremljene.")).toBeTruthy();
    fireEvent.click(screen.getByRole("link", { name: "Svi odjeli" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    act(() => navigate("/plan"));
    const price = (await screen.findByLabelText("Cijena sobe po noći (€)")) as HTMLInputElement;
    expect(price.value).toBe("150");
    expect(screen.getByText("Izmjene još nisu spremljene.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const sent = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as PlanBody;
    expect(sent.price_cents).toBe(15_000);
    await waitFor(() => expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull());
  });

  it("vraćena ista vrijednost nije nespremljena izmjena", async () => {
    renderApp(v2Api(), "/plan");
    const price = (await screen.findByLabelText("Cijena sobe po noći (€)")) as HTMLInputElement;
    const was = price.value;
    fireEvent.change(price, { target: { value: "150" } });
    fireEvent.change(price, { target: { value: was } });
    expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull();
  });

  it("u međuvremenu otvoren novi tjedan: bez PUT-a, jedna rečenica o novom tjednu (minor 2)", async () => {
    let reads = 0;
    const next = v2PlanHotel();
    if (next.private?.plan) next.private.plan.round = "t02";
    const api = v2Api().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : next);
    });
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "150" } });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    expect(
      await screen.findByText(
        "Otvoren je novi tjedan. Odluke sada vrijede za njega; nespremljene izmjene prošlog tjedna nisu prenesene.",
      ),
    ).toBeTruthy();
    expect(screen.queryByText(/pa odluke nisu spremljene/)).toBeNull();
    expect(api.calls.filter((c) => c.method === "PUT")).toHaveLength(0);
    expect((screen.getByLabelText("Cijena sobe po noći (€)") as HTMLInputElement).value).not.toBe("150");
    // Gumb je daleko ispod: fokus ide na rečenicu (M-7).
    expect(document.activeElement?.textContent).toMatch(/^Otvoren je novi tjedan/);
  });

  it("nacrt prošlog tjedna pri otvaranju plana: rečenica i nacrt se briše (minor 3)", async () => {
    renderApp(v2Api(), "/plan");
    writeDraft(draftKey("plan", HOTEL_ID, "t00"), { price: "999" });
    expect(
      await screen.findByText(
        "Otvoren je novi tjedan. Odluke sada vrijede za njega; nespremljene izmjene prošlog tjedna nisu prenesene.",
      ),
    ).toBeTruthy();
    expect((screen.getByLabelText("Cijena sobe po noći (€)") as HTMLInputElement).value).not.toBe("999");
    await waitFor(() => expect(readDraft(draftKey("plan", HOTEL_ID, "t00"))).toEqual({}));
  });

  it("šalju se samo dirana polja hotela: ono što je u međuvremenu spremljeno drugdje ostaje (minor 2)", async () => {
    let reads = 0;
    const fresh = v2PlanHotel();
    if (fresh.private?.plan) fresh.private.plan.plan.marketing_cents = 70_000;
    const api = v2Api()
      .on("GET", "/v1/me/hotel", () => {
        reads += 1;
        return json(200, reads === 1 ? v2PlanHotel() : fresh);
      })
      .on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));
    renderApp(api, "/plan");
    fireEvent.change(await screen.findByLabelText("Cijena sobe po noći (€)"), { target: { value: "150" } });
    fireEvent.click(screen.getByRole("button", { name: "Spremi plan" }));
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const sent = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as PlanBody;
    expect(sent.price_cents).toBe(15_000);
    expect(sent.marketing_cents).toBe(70_000);
  });

  it("dok se sprema, gumb ostaje u redu tipkovnice (aria-disabled, M-6)", async () => {
    const api = v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () => new Promise<Response>(() => {}));
    renderApp(api, "/plan");
    const button = await screen.findByRole("button", { name: "Spremi plan" });
    button.focus();
    fireEvent.click(button);
    await waitFor(() => expect(button.getAttribute("aria-disabled")).toBe("true"));
    expect(button.hasAttribute("disabled")).toBe(false);
    expect(document.activeElement).toBe(button);
  });

  it("sažetak odjela kaže kad odjel ima nespremljene izmjene (M-8)", async () => {
    renderApp(v2Api(), "/plan");
    writeDraft(draftKey("dept", HOTEL_ID, "t01", "lobby_bar"), { seasonal_target: 4 });
    expect(await screen.findByRole("link", { name: "Lobby bar: izmjene još nisu spremljene" })).toBeTruthy();
  });
});
