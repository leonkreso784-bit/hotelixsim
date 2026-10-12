import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HotelBody, PlanBody } from "../../api/types";
import { draftKey, readDraft, writeDraft } from "../../drafts";
import { navigate } from "../../router";
import { json, problem } from "../../test/fakeApi";
import { departmentsBody, HOTEL_ID, plan, v2Api, v2PlanHotel } from "../../test/fixtures";
import { renderApp } from "../../test/renderApp";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const PUT = `/v1/hotels/${HOTEL_ID}/plans/t01`;

/** v2 igra u kojoj PUT plana vraća poslano tijelo (poslužitelj vraća spremljeni plan). */
const echoApi = () =>
  v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => json(200, body as object));

/** Moj hotel s izmijenjenim planom kruga (npr. odluke drugog odjela spremljene u međuvremenu). */
function hotelWith(change: (p: PlanBody) => PlanBody, limits?: (h: HotelBody) => void): HotelBody {
  const h = v2PlanHotel();
  if (h.private?.plan) h.private.plan.plan = change(h.private.plan.plan);
  limits?.(h);
  return h;
}

/** Plan s drugim ciljem sezonskih radnika u jednom odjelu. */
function withTarget(p: PlanBody, code: string, seasonal_target: number): PlanBody {
  const d = p.departments?.[code];
  if (!d) throw new Error(`nema odjela ${code}`);
  return { ...p, departments: { ...p.departments, [code]: { ...d, seasonal_target } } };
}

/**
 * Poslužitelj koji pamti plan: PUT ga sprema (kroz `change`, kao da poslužitelj nešto prilagodi), a moj hotel ga
 * nakon toga vraća, kao pravi poslužitelj.
 */
function storingApi(change: (sent: PlanBody) => PlanBody = (x) => x) {
  let stored: PlanBody | null = null;
  return v2Api()
    .on("GET", "/v1/me/hotel", () => json(200, stored ? hotelWith(() => stored as PlanBody) : v2PlanHotel()))
    .on("PUT", "/v1/hotels/:hotel/plans/:month", (_r, _p, body) => {
      stored = change(body as PlanBody);
      return json(200, stored);
    });
}

/** Moj hotel kad krug nije otvoren: poslužitelj ne šalje plan. */
function closedHotel(): HotelBody {
  const h = v2PlanHotel();
  if (h.private) h.private.plan = null;
  return h;
}

async function decisions(name: string) {
  return await screen.findByRole("group", { name });
}

describe("Odluke odjela na ekranu odjela", () => {
  it("odjel s gostima ima odluke za ovaj tjedan; služba ih nema", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    expect(
      await screen.findByRole("heading", { level: 1, name: "Restoran" }, { timeout: 3000 }),
    ).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Odluke za ovaj tjedan" })).toBeTruthy();
    expect(await decisions("Restoran")).toBeTruthy();
    cleanup();
    renderApp(echoApi(), "/odjeli/admin");
    expect(await screen.findByRole("heading", { level: 1, name: "Uprava" })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Odluke za ovaj tjedan" })).toBeNull();
  });

  it("svaki gumb, potvrda i izbor imaju opis s imenom odjela; gumbi sezonskih radnika imaju vidljivo ime", async () => {
    renderApp(echoApi(), "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    const controls = [
      ...within(bar).getAllByRole("button"),
      ...within(bar).getAllByRole("checkbox"),
      ...within(bar).getAllByRole("radio"),
    ];
    expect(controls.length).toBeGreaterThanOrEqual(8);
    for (const c of controls) {
      const id = c.getAttribute("aria-describedby");
      expect(id, c.outerHTML).toBeTruthy();
      // Prvi opis je ime odjela; izbor održavanja uz njega ima i cijenu.
      const first = (id ?? "").split(" ")[0] ?? "";
      expect(document.getElementById(first)?.textContent).toBe("Lobby bar");
    }
    const hire = within(bar).getByRole("button", { name: "Zaposli sezonskog radnika" });
    expect(hire.textContent).toBe("Zaposli sezonskog radnika");
    expect(within(bar).getByRole("button", { name: "Otpusti sezonskog radnika" }).textContent).toBe(
      "Otpusti sezonskog radnika",
    );
    expect(within(bar).getByText("3 sezonska radnika")).toBeTruthy();
  });

  it("zatvoren odjel: sve osim „Otvoren“ je onemogućeno; ponovno otvaranje vraća sezonske radnike i oslobađa korak", async () => {
    renderApp(echoApi(), "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    expect(
      screen.getByText("Ovaj tjedan možeš još promijeniti 5 sezonskih radnika u cijelom hotelu"),
    ).toBeTruthy();
    const open = within(bar).getByRole("checkbox", { name: "Otvoren" });
    fireEvent.click(open);
    expect(
      screen.getByText("Ovaj tjedan možeš još promijeniti 2 sezonska radnika u cijelom hotelu"),
    ).toBeTruthy();
    for (const b of within(bar).getAllByRole("button")) expect(b.getAttribute("aria-disabled")).toBe("true");
    for (const r of within(bar).getAllByRole("radio")) expect((r as HTMLInputElement).disabled).toBe(true);
    fireEvent.click(open);
    expect(
      screen.getByText("Ovaj tjedan možeš još promijeniti 5 sezonskih radnika u cijelom hotelu"),
    ).toBeTruthy();
    expect(within(bar).getByText("3 sezonska radnika")).toBeTruthy();
  });

  it("spremanje: jedan PUT s cijelim svježim planom; ostali odjeli i odluke hotela netaknuti", async () => {
    // Prvi dohvat (ekran) vidi plan iz fixturea; dohvat prije spremanja vidi novije odluke drugog ekrana.
    let reads = 0;
    const fresh = hotelWith((p) => ({ ...withTarget(p, "restaurant", 3), price_cents: 15_000 }));
    const api = echoApi().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : fresh);
    });
    renderApp(api, "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    fireEvent.click(within(bar).getByRole("checkbox", { name: "Otvoren" }));
    const save = screen.getByRole("button", { name: "Spremi odluke odjela" });
    fireEvent.click(save);
    fireEvent.click(save);
    await waitFor(() => expect(api.posts(PUT)).toHaveLength(1));
    const body = api.posts(PUT)[0]?.body as PlanBody;
    const base = fresh.private?.plan?.plan as PlanBody;
    expect(body.departments?.lobby_bar).toMatchObject({ open: false, seasonal_target: 0 });
    expect(body.departments?.restaurant).toEqual(base.departments?.restaurant);
    for (const c of ["rooms", "parking", "admin", "sales", "maintenance", "energy"])
      expect(body.departments?.[c], c).toEqual(base.departments?.[c]);
    expect(body).toMatchObject({
      ...plan(),
      price_cents: 15_000,
      seasonal_target: 21,
      maintenance: "regular",
    });
    expect(await screen.findByText("Odluke odjela su spremljene.")).toBeTruthy();
  });

  it("odgovor poslužitelja (spremljeni plan) osvježava prikaz", async () => {
    const api = storingApi((sent) => ({
      ...sent,
      departments: {
        ...sent.departments,
        lobby_bar: { open: true, seasonal_target: 1, price_level_pct: 90, maintenance: "deferred" },
      },
    }));
    renderApp(api, "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect(await within(bar).findByText("1 sezonski radnik")).toBeTruthy();
    expect(within(bar).getByText("90 %")).toBeTruthy();
    expect((within(bar).getByRole("radio", { name: "odgođeno" }) as HTMLInputElement).checked).toBe(true);
  });

  it("restoran se ne može zatvoriti", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    expect(within(rest).queryByRole("checkbox", { name: "Otvoren" })).toBeNull();
  });

  it("korak potrošen: zapošljavanje onemogućeno i rečenica", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    const plus = within(rest).getByRole("button", { name: "Zaposli sezonskog radnika" });
    for (let i = 0; i < 5; i++) fireEvent.click(plus);
    expect(screen.getByText("Ovaj tjedan više ne možeš mijenjati broj sezonskih radnika.")).toBeTruthy();
    expect(plus.getAttribute("aria-disabled")).toBe("true");
    expect(within(rest).getByText("7 sezonskih radnika")).toBeTruthy();
  });

  it("gumb koji dosegne granicu zadržava fokus (nije disabled), klik na njega ne radi ništa", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    const plus = within(rest).getByRole("button", { name: "Zaposli sezonskog radnika" });
    plus.focus();
    for (let i = 0; i < 5; i++) fireEvent.click(plus);
    expect(plus.getAttribute("aria-disabled")).toBe("true");
    expect(plus.hasAttribute("disabled")).toBe(false);
    expect(document.activeElement).toBe(plus);
    const shown = rest.querySelector("output")?.textContent;
    fireEvent.click(plus);
    expect(rest.querySelector("output")?.textContent).toBe(shown);
  });

  it("korak koji su potrošili drugi odjeli: vidi se ostatak, zatvaranje koje ne stane je onemogućeno", async () => {
    const h = hotelWith((p) => withTarget(p, "restaurant", 6));
    renderApp(echoApi().on("GET", "/v1/me/hotel", h), "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    expect(
      screen.getByText("Ovaj tjedan možeš još promijeniti 1 sezonskog radnika u cijelom hotelu"),
    ).toBeTruthy();
    // Kvačica ostaje u redu tipkovnice (aria-disabled, kao gumbi brojača; M5), a klik je ne mijenja.
    const open = within(bar).getByRole("checkbox", { name: "Otvoren" }) as HTMLInputElement;
    expect(open.getAttribute("aria-disabled")).toBe("true");
    expect(open.disabled).toBe(false);
    fireEvent.click(open);
    expect(open.checked).toBe(true);
    expect(within(bar).getByText("3 sezonska radnika")).toBeTruthy();
    expect(
      screen.getByText(
        "Zatvaranje bi otpustilo više sezonskih radnika nego što se ovaj tjedan još smije promijeniti.",
      ),
    ).toBeTruthy();
  });

  it("zbroj iznad najvećeg broja hotela: jasna rečenica, bez zahtjeva", async () => {
    const h = hotelWith(
      (p) => p,
      (x) => {
        if (x.private?.plan) x.private.plan.limits.seasonal_max = 24;
      },
    );
    const api = echoApi().on("GET", "/v1/me/hotel", h);
    renderApp(api, "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    const plus = within(rest).getByRole("button", { name: "Zaposli sezonskog radnika" });
    fireEvent.click(plus);
    fireEvent.click(plus);
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect(
      await screen.findByText(
        "Odjeli zajedno imaju previše sezonskih radnika: hotel smije imati najviše 24 sezonska radnika. Smanji ih u nekom odjelu.",
      ),
    ).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(0);
    fireEvent.click(within(rest).getByRole("button", { name: "Otpusti sezonskog radnika" }));
    expect(screen.queryByText(/previše sezonskih/)).toBeNull();
  });

  it("razina cijene u koracima od 5 do granice; održavanje se šalje po odjelu", async () => {
    const api = echoApi();
    renderApp(api, "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    expect(within(rest).getByText("Cijene u odjelu")).toBeTruthy();
    expect(within(rest).getByText("100 %")).toBeTruthy();
    const up = within(rest).getByRole("button", { name: "Više cijene" });
    for (let i = 0; i < 6; i++) fireEvent.click(up);
    expect(within(rest).getByText("120 %")).toBeTruthy();
    expect(up.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(within(rest).getByRole("radio", { name: "pojačano" }));
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    await waitFor(() => expect(api.posts(PUT)).toHaveLength(1));
    const body = api.posts(PUT)[0]?.body as PlanBody;
    expect(body.departments?.restaurant).toMatchObject({ price_level_pct: 120, maintenance: "intensive" });
  });

  it("422 department_plan_invalid je rečenica", async () => {
    renderApp(
      v2Api().on("PUT", "/v1/hotels/:hotel/plans/:month", () => problem(422, "department_plan_invalid")),
      "/odjeli/restaurant",
    );
    await decisions("Restoran");
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect((await screen.findByRole("alert")).textContent).toContain("Postavke odjela nisu dopuštene");
  });

  it("krug zatvoren u međuvremenu: rečenica, bez PUT-a", async () => {
    let reads = 0;
    const closed = closedHotel();
    const api = echoApi().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : closed);
    });
    renderApp(api, "/odjeli/restaurant");
    await decisions("Restoran");
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect(await screen.findByText("Tjedan nije otvoren — sad se ne može mijenjati.")).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(0);
  });

  it("krug nije otvoren: odluke samo za čitanje", async () => {
    const closed = closedHotel();
    renderApp(echoApi().on("GET", "/v1/me/hotel", closed), "/odjeli/lobby_bar");
    expect(await screen.findByText("Odluke odjela mijenjaju se dok je tjedan otvoren.")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Spremi odluke odjela" })).toBeNull();
    expect(screen.queryByRole("checkbox")).toBeNull();
    expect(screen.getByText("3 sezonska radnika")).toBeTruthy();
    // Razina cijena zadnjeg obračunatog tjedna (M6).
    const price = screen.getByText("Cijene u zadnjem tjednu").parentElement as HTMLElement;
    expect(within(price).getByText("100 %")).toBeTruthy();
  });

  it("„Spremi odluke odjela“ čeka popis odjela (sadašnji broj ljudi ostalih odjela)", async () => {
    renderApp(
      echoApi().on("GET", "/v1/hotels/:hotel/departments", () => new Promise<Response>(() => {})),
      "/odjeli/restaurant",
    );
    await decisions("Restoran");
    // aria-disabled, ne disabled: fokus ostaje na gumbu (M-6); klik ne šalje ništa.
    const save = screen.getByRole("button", { name: "Spremi odluke odjela" });
    expect(save.getAttribute("aria-disabled")).toBe("true");
    expect(save.hasAttribute("disabled")).toBe(false);
  });

  it("jasan jezik: nigdje „sezonci“; uz ljude objašnjenje stalnih i sezonskih radnika", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    await decisions("Restoran");
    expect(document.body.textContent).not.toMatch(/sezonc/i);
    expect(
      screen.getByText(
        "Stalni rade cijelu godinu. Sezonski su zaposleni samo za ljeto (lipanj–rujan) i trošak su samo dok rade.",
      ),
    ).toBeTruthy();
    for (const b of screen.getAllByRole("button"))
      expect(b.getAttribute("aria-label") ?? "").not.toMatch(/sezonc/i);
  });
});

describe("nespremljene izmjene i svjež plan", () => {
  it("izmjena odjela preživi odlazak na drugi odjel i povratak (I1)", async () => {
    const api = echoApi();
    renderApp(api, "/odjeli/lobby_bar");
    let bar = await decisions("Lobby bar");
    fireEvent.click(within(bar).getByRole("button", { name: "Zaposli sezonskog radnika" }));
    expect(within(bar).getByText("4 sezonska radnika")).toBeTruthy();
    expect(screen.getByText("Izmjene još nisu spremljene.")).toBeTruthy();
    act(() => navigate("/odjeli/restaurant"));
    await decisions("Restoran");
    expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull();
    act(() => navigate("/odjeli/lobby_bar"));
    bar = await decisions("Lobby bar");
    expect(within(bar).getByText("4 sezonska radnika")).toBeTruthy();
    expect(screen.getByText("Izmjene još nisu spremljene.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect(await screen.findByText("Odluke odjela su spremljene.")).toBeTruthy();
    expect(screen.queryByText("Izmjene još nisu spremljene.")).toBeNull();
  });

  it("šalju se samo dirana polja: ono što je u međuvremenu spremljeno drugdje ostaje (minor 1)", async () => {
    let reads = 0;
    // Dok je ekran otvoren, netko je spremio višu razinu cijena Lobby bara (npr. druga kartica preglednika).
    const fresh = hotelWith((p) => {
      const d = p.departments?.lobby_bar;
      if (!d) throw new Error("nema lobby bara");
      return { ...p, departments: { ...p.departments, lobby_bar: { ...d, price_level_pct: 110 } } };
    });
    const api = echoApi().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : fresh);
    });
    renderApp(api, "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    fireEvent.click(within(bar).getByRole("button", { name: "Zaposli sezonskog radnika" }));
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    await waitFor(() => expect(api.posts(PUT)).toHaveLength(1));
    const body = api.posts(PUT)[0]?.body as PlanBody;
    expect(body.departments?.lobby_bar).toEqual({
      open: true,
      seasonal_target: 4,
      price_level_pct: 110,
      maintenance: "regular",
    });
  });

  it("u međuvremenu otvoren novi tjedan: bez PUT-a, rečenica da odluke vrijede za novi tjedan (minor 2)", async () => {
    let reads = 0;
    const next = v2PlanHotel();
    if (next.private?.plan) next.private.plan.round = "t02";
    const api = echoApi().on("GET", "/v1/me/hotel", () => {
      reads += 1;
      return json(200, reads === 1 ? v2PlanHotel() : next);
    });
    renderApp(api, "/odjeli/lobby_bar");
    const bar = await decisions("Lobby bar");
    fireEvent.click(within(bar).getByRole("button", { name: "Zaposli sezonskog radnika" }));
    fireEvent.click(screen.getByRole("button", { name: "Spremi odluke odjela" }));
    expect(
      await screen.findByText(
        "Otvoren je novi tjedan. Odluke sada vrijede za njega; nespremljene izmjene prošlog tjedna nisu prenesene.",
      ),
    ).toBeTruthy();
    expect(api.posts(PUT)).toHaveLength(0);
    expect(within(await decisions("Lobby bar")).getByText("3 sezonska radnika")).toBeTruthy();
  });

  it("popis odjela se ne učita: u odlukama je ponovni pokušaj (mobitel, minor 3)", async () => {
    let failing = true;
    const api = echoApi().on("GET", "/v1/hotels/:hotel/departments", () =>
      failing ? problem(500, "internal") : json(200, departmentsBody()),
    );
    renderApp(api, "/odjeli/restaurant");
    const rest = await decisions("Restoran");
    const card = rest.closest("section") as HTMLElement;
    const retry = await within(card).findByRole("button", { name: "Ponovno učitaj odjele" });
    expect(retry.className).toMatch(/lg:hidden/);
    failing = false;
    fireEvent.click(retry);
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "Spremi odluke odjela" }) as HTMLButtonElement).disabled,
      ).toBe(false),
    );
    expect(within(card).queryByRole("button", { name: "Ponovno učitaj odjele" })).toBeNull();
  });

  it("nacrt odjela iz prošlog tjedna: rečenica pri otvaranju i nacrt se briše (minor 3)", async () => {
    renderApp(echoApi(), "/odjeli/restaurant");
    writeDraft(draftKey("dept", HOTEL_ID, "t00", "lobby_bar"), { seasonal_target: 1 });
    await decisions("Restoran");
    expect(
      await screen.findByText(
        "Otvoren je novi tjedan. Odluke sada vrijede za njega; nespremljene izmjene prošlog tjedna nisu prenesene.",
      ),
    ).toBeTruthy();
    await waitFor(() => expect(readDraft(draftKey("dept", HOTEL_ID, "t00", "lobby_bar"))).toEqual({}));
  });
});
