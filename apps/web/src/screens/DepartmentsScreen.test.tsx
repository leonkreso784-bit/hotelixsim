import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { problem } from "../test/fakeApi";
import {
  departmentDetailBody,
  departmentsBody,
  game,
  standardApi,
  v2Api,
  v2PlanHotel,
} from "../test/fixtures";
import { fastPolling, renderApp } from "../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("Odjeli", () => {
  it("popis: samo odjeli hotela, brojke s imenom i jedinicom", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    const spa = (await screen.findByRole("link", { name: /Mali wellness/ })).closest(
      "article",
    ) as HTMLElement;
    // Jedna traka „Ocjena odjela“: prosjek prostora 65, zadovoljstva 74 i usklađenosti ljudi 83 (opterećenje 190)
    // = 74, uz broj i riječ.
    expect(within(spa).getByText("Ocjena odjela")).toBeTruthy();
    expect(within(spa).getByText("74/100 · srednje")).toBeTruthy();
    expect(within(spa).getAllByRole("img")).toHaveLength(1);
    expect(within(spa).getByRole("img").getAttribute("aria-label")).toBe("Ocjena odjela: 74 od 100, srednje");
    expect(within(spa).getByText("Ljudi: 2 stalna, 1 sezonski")).toBeTruthy();
    expect(screen.queryByText("Rooftop bar")).toBeNull();
    expect(screen.getByRole("heading", { name: "Službe bez prihoda" })).toBeTruthy();
  });

  it("kartice i službe imaju ikonu odjela uz ime; službe nemaju traku ocjene", async () => {
    renderApp(v2Api(), "/odjeli");
    const rest = (await screen.findByRole("link", { name: "Restoran" })).closest("article") as HTMLElement;
    const icon = rest.querySelector("[data-dept-icon='restaurant']");
    expect(icon?.getAttribute("aria-hidden")).toBe("true");
    expect(rest.querySelector("[data-dept-icon='rooms']")).toBeNull();
    const admin = screen.getByRole("link", { name: /Uprava/ });
    expect(admin.querySelector("[data-dept-icon='admin']")?.getAttribute("aria-hidden")).toBe("true");
    expect(within(admin).queryByRole("img")).toBeNull();
  });

  it("detalj: ocjena i tri sastavnice, što mu fali i u čemu je dobar iz zadnjeg tjedna", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const h1 = await screen.findByRole("heading", { level: 1, name: "Restoran" });
    // Stupac detalja (popis lijevo ima svoju karticu Restorana s istom ocjenom).
    const detail = within(h1.closest("[data-layout] > div") as HTMLElement);
    expect(detail.getByRole("img", { name: "Ocjena odjela: 74 od 100, srednje" })).toBeTruthy();
    expect(detail.getByText("74/100 · srednje")).toBeTruthy();
    // Kvaliteta usluge hotela više ne ulazi (Leon 12.10.); usklađenost ljudi je ista mjera kao rečenica ispod.
    expect(detail.queryByRole("img", { name: /Kvaliteta usluge/ })).toBeNull();
    expect(detail.getByRole("img", { name: "Stanje prostora: 69 od 100" })).toBeTruthy();
    expect(detail.getByRole("img", { name: "Zadovoljstvo gostiju: 69 od 100" })).toBeTruthy();
    expect(detail.getByRole("img", { name: "Usklađenost ljudi s poslom: 83 od 100" })).toBeTruthy();
    expect(
      detail.getByText("Prosjek stanja prostora, zadovoljstva gostiju i usklađenosti ljudi s poslom."),
    ).toBeTruthy();
    const weak = screen
      .getByRole("heading", { level: 2, name: "Što mu fali" })
      .closest("section") as HTMLElement;
    expect(
      within(weak).getByText(
        "Osoblje je preopterećeno: 106 % granice za dobru uslugu, kvaliteta usluge pada",
      ),
    ).toBeTruthy();
    expect(within(weak).getByText("Prijedlog: zaposli više sezonskih radnika u ovom odjelu.")).toBeTruthy();
    // Pad za 1 prikazani bod (70 → 69) nije slaba strana (M7).
    expect(within(weak).queryByText(/Zadovoljstvo gostiju palo/)).toBeNull();
    const strong = screen
      .getByRole("heading", { level: 2, name: "U čemu je dobar" })
      .closest("section") as HTMLElement;
    expect(
      within(strong).getByText("Posluženi su svi gosti koji su htjeli doći (2.100 korištenja)"),
    ).toBeTruthy();
    expect(within(strong).getByText("Odjel je ovaj tjedan u dobiti (+7.896 €)")).toBeTruthy();
  });

  it("detalj bez obračunatog kruga: slabe i jake strane vide se nakon prvog tjedna", async () => {
    renderApp(v2Api(), "/odjeli/parking");
    expect(
      await screen.findByText("Slabe i jake strane odjela vide se nakon prvog obračunatog tjedna."),
    ).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Što mu fali" })).toBeNull();
  });

  it("detalj odjela izravno na putanji: prostor, gosti, novac, ljudi", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    for (const h of ["Prostor", "Gosti", "Novac", "Ljudi"])
      expect(screen.getByRole("heading", { level: 2, name: h })).toBeTruthy();
    expect(screen.getByText("67.200 €")).toBeTruthy();
    expect(screen.getByText("trošenje −3 boda")).toBeTruthy();
    expect(screen.getByText("preopterećenje −1 bod")).toBeTruthy();
    expect(screen.getByText("održavanje +3 boda")).toBeTruthy();
    // Poslužiteljeve promjene već nose predznak: prije + promjene = poslije.
    const r = departmentDetailBody("restaurant")?.department.last;
    expect(r && r.condition_before_pts + r.wear_pts + r.load_wear_pts + r.maintenance_pts).toBe(
      r?.condition_after_pts,
    );
    expect(screen.getByRole("heading", { level: 2, name: "Po tjednima" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 3, name: "Restoran" })).toBeTruthy();
    const guests = screen.getByRole("heading", { level: 2, name: "Gosti" }).closest("section") as HTMLElement;
    expect(within(guests).getByText("Gosti hotela koji dnevno dođu u odjel")).toBeTruthy();
    expect(within(guests).getByText("45 %")).toBeTruthy();
    expect(within(guests).getByText("2.100 korištenja")).toBeTruthy();
    // Jedna mjera iskorištenosti (I4): opterećenje u postotku granice, bez kapaciteta i gostiju po zaposlenom.
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("Opterećenje osoblja: 106 % granice za dobru uslugu.")).toBeTruthy();
    expect(within(people).queryByText(/Kapacitet|po zaposlenom/)).toBeNull();
  });

  it("Smještaj broji noćenja i pokazuje popunjenost soba (M3)", async () => {
    renderApp(v2Api(), "/odjeli/rooms");
    expect(await screen.findByRole("heading", { level: 1, name: "Smještaj" })).toBeTruthy();
    const guests = screen.getByRole("heading", { level: 2, name: "Gosti" }).closest("section") as HTMLElement;
    expect(within(guests).getByText("Popunjenost soba")).toBeTruthy();
    expect(within(guests).getByText("2.100 noćenja")).toBeTruthy();
    expect(screen.getByText("Posluženi su svi gosti koji su htjeli doći (2.100 noćenja)")).toBeTruthy();
    expect(screen.queryByText(/korištenja/)).toBeNull();
  });

  it("srednja ocjena je jantarna traka, uz broj i riječ (I2)", async () => {
    const body = departmentsBody();
    const spa = body.departments.find((d) => d.code === "spa");
    if (spa) Object.assign(spa, { condition_pts: 4000, guest_satisfaction_pts: 6000 });
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/departments", body), "/odjeli");
    const card = (await screen.findByRole("link", { name: /Mali wellness/ })).closest(
      "article",
    ) as HTMLElement;
    // (40 + 60 + 83) / 3 = 61
    expect(within(card).getByText("61/100 · srednje")).toBeTruthy();
    expect(within(card).getByRole("img").querySelector(".bg-warn")).toBeTruthy();
  });

  it("zatvoren odjel: bez ocjene, razlog gubitka su plaće stalnih radnika (M2)", async () => {
    const body = departmentsBody();
    const spa = body.departments.find((d) => d.code === "spa");
    if (!spa?.last) throw new Error("wellness bez kruga");
    Object.assign(spa, {
      open: false,
      seasonal: 0,
      last: {
        ...spa.last,
        served: 0,
        demand: 0,
        load_hundredths: 0,
        payroll_cents: 116_666,
        profit_cents: -116_666,
      },
    });
    const api = v2Api()
      .on("GET", "/v1/hotels/:hotel/departments", body)
      .on("GET", "/v1/hotels/:hotel/departments/:code", { department: spa, history: [spa.last] });
    renderApp(api, "/odjeli/spa");
    const h1 = await screen.findByRole("heading", { level: 1, name: "Mali wellness" });
    const detail = within(h1.closest("[data-layout] > div") as HTMLElement);
    expect(detail.queryByRole("img", { name: /Ocjena odjela/ })).toBeNull();
    const weak = screen
      .getByRole("heading", { level: 2, name: "Što mu fali" })
      .closest("section") as HTMLElement;
    expect(
      within(weak).getByText(
        "Odjel je zatvoren: ne prima goste, a stalni radnici se i dalje plaćaju (plaće 1.167 € ovaj tjedan)",
      ),
    ).toBeTruthy();
    expect(within(weak).queryByText(/u gubitku/)).toBeNull();
    // Zatvoren odjel nema jakih strana ni gostiju u krugu (M-11).
    expect(screen.queryByRole("heading", { level: 2, name: "U čemu je dobar" })).toBeNull();
    expect(screen.queryByRole("heading", { level: 2, name: "Gosti" })).toBeNull();
    const card = screen.getAllByRole("link", { name: /Mali wellness/ })[0]?.closest("article") as HTMLElement;
    expect(within(card).queryByRole("img")).toBeNull();
    expect(within(card).getByText("Zatvoren")).toBeTruthy();
  });

  it("najam bez ljudi (parking): ocjena je samo stanje prostora, bez redaka o ljudima i gostima", async () => {
    const body = departmentsBody();
    const parking = body.departments.find((d) => d.code === "parking");
    const restaurant = body.departments.find((d) => d.code === "restaurant");
    if (!parking || !restaurant?.last) throw new Error("nema parkinga");
    Object.assign(parking, {
      open: true,
      condition_pts: 4200,
      guest_satisfaction_pts: 7000,
      last: { ...restaurant.last, served: 0, demand: 0, load_hundredths: 0, payroll_cents: 0 },
    });
    // Plan kruga: parking otvoren, bez sezonskih radnika (granice 0–0) i ne može se zatvoriti, kao u katalogu.
    const hotel = v2PlanHotel();
    const plan = hotel.private?.plan;
    if (!plan?.plan.departments || !plan.limits.departments?.parking) throw new Error("nema plana");
    plan.plan.departments.parking = {
      open: true,
      seasonal_target: 0,
      price_level_pct: 100,
      maintenance: "regular",
    };
    Object.assign(plan.limits.departments.parking, { seasonal_max: 0, can_close: false });
    const api = v2Api()
      .on("GET", "/v1/me/hotel", hotel)
      .on("GET", "/v1/hotels/:hotel/departments", body)
      .on("GET", "/v1/hotels/:hotel/departments/:code", { department: parking, history: [parking.last] });
    renderApp(api, "/odjeli/parking");
    const h1 = await screen.findByRole("heading", { level: 1, name: "Parking" });
    const detail = within(h1.closest("[data-layout] > div") as HTMLElement);
    expect(detail.getByRole("img", { name: "Ocjena odjela: 42 od 100, slabo" })).toBeTruthy();
    expect(detail.getByText("Ocjena je stanje prostora.")).toBeTruthy();
    // Jedna sastavnica se ne crta dvaput (M-10); zadovoljstvo se nigdje ne spominje, ni u povijesti (N-I1).
    expect(detail.queryByRole("img", { name: /Stanje prostora|Zadovoljstvo|Usklađenost/ })).toBeNull();
    expect(detail.queryByText(/Zadovoljstvo gostiju|zadovoljstvo/)).toBeNull();
    expect(detail.queryByRole("columnheader", { name: "Zadovoljstvo gostiju" })).toBeNull();
    // Odluke parkinga: bez sezonskih radnika; cijena kao u restoranu (O-P, granice 90–120 %) i održavanje.
    const decisions = await screen.findByRole("group", { name: "Parking" });
    expect(within(decisions).queryByText("Sezonski radnici u odjelu")).toBeNull();
    expect(within(decisions).getByText("Cijene u odjelu")).toBeTruthy();
    expect(
      within(decisions).getByText(
        "Viša cijena donosi više novca od svakog korištenja, ali prostor koristi manje gostiju.",
      ),
    ).toBeTruthy();
    expect(within(decisions).getByRole("radio", { name: "redovno" })).toBeTruthy();
    expect(screen.queryByText(/Jedan sezonski radnik|možeš još promijeniti/)).toBeNull();
    expect(screen.getByText("Odredi razinu cijena i održavanje.")).toBeTruthy();
    for (const h of ["Gosti", "Ljudi"])
      expect(detail.queryByRole("heading", { level: 2, name: h })).toBeNull();
    const card = screen.getAllByRole("link", { name: "Parking" })[0]?.closest("article") as HTMLElement;
    expect(within(card).queryByText(/Ljudi:/)).toBeNull();
  });

  it("računalo: popis i detalj u dva stupca", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const grid = (await screen.findByRole("heading", { level: 1, name: "Restoran" })).closest(
      "[data-layout='list-detail']",
    );
    expect(grid?.className).toMatch(/lg:grid/);
    // Stupac detalja se smije suziti i tablica se pomiče u svojoj regiji, nikad ne gura stranicu.
    expect(
      screen.getByRole("heading", { level: 1, name: "Restoran" }).closest("[data-layout] > div")?.className,
    ).toMatch(/min-w-0/);
    const region = screen.getByRole("region", { name: "Rezultat odjela po tjednima" });
    expect(region.className).toMatch(/overflow-x-auto/);
    expect(region.getAttribute("tabindex")).toBe("0");
  });

  it("v1 igra: rečenica umjesto odjela i nema stavke izbornika", async () => {
    renderApp(
      standardApi().on("GET", "/v1/hotels/:hotel/departments", () => problem(409, "model_v1")),
      "/odjeli",
    );
    expect(await screen.findByText("Ova igra je na starom modelu i nema ovaj prikaz.")).toBeTruthy();
    expect(await screen.findByText(/LIPANJ/)).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });

  it("nepoznat odjel: rečenica, ne prazno", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => problem(404, "unknown_department")),
      "/odjeli/casino",
    );
    expect(await screen.findByText("Taj odjel ne postoji u ovom hotelu.")).toBeTruthy();
    expect(screen.getByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.getAllByRole("link", { name: "Odjeli" }).length).toBeGreaterThan(1);
  });

  it("v2 igra: izbornik ima Odjeli, zatvoren odjel nosi oznaku, kartica je jedna poveznica", async () => {
    renderApp(v2Api(), "/odjeli");
    const parking = (await screen.findByRole("link", { name: "Parking" })).closest("article") as HTMLElement;
    expect(within(parking).getByText("Zatvoren")).toBeTruthy();
    expect(within(parking).getAllByRole("link")).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Odjeli" }).getAttribute("href")).toBe("/odjeli");
    expect(screen.getByRole("link", { name: /Uprava/ }).textContent).toContain("59.050\u00a0€");
  });

  it("služba bez prihoda: trošak po stavkama, bez kartica Prostor i Gosti", async () => {
    renderApp(v2Api(), "/odjeli/admin");
    expect(await screen.findByRole("heading", { level: 1, name: "Uprava" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Trošak po stavkama" })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Prostor" })).toBeNull();
    expect(screen.getByRole("heading", { level: 2, name: "Ljudi" })).toBeTruthy();
  });

  it("odjel bez obračunatog kruga: rečenica što će se pojaviti, ne nule", async () => {
    renderApp(v2Api(), "/odjeli/parking");
    expect(await screen.findByText("Rezultat se pojavi nakon prvog obračunatog tjedna.")).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Novac" })).toBeNull();
  });

  it("prije prvog obračuna: rečenica kad se pojavljuju rezultati", async () => {
    const empty = {
      round: null,
      departments: departmentsBody().departments.map((d) => ({ ...d, last: null, service_last: null })),
    };
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/departments", empty), "/odjeli");
    expect(
      await screen.findByText("Rezultati odjela pojave se nakon prvog obračunatog tjedna."),
    ).toBeTruthy();
  });

  it("detalj dok se učitava ima povratak i naslov prve razine", async () => {
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => new Promise<Response>(() => {})),
      "/odjeli/restaurant",
    );
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    await waitFor(() => expect(screen.getAllByRole("link", { name: "Odjeli" }).length).toBeGreaterThan(1));
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("ljudi: odlasci odjela u zadnjem krugu, i kad nitko nije otišao", async () => {
    renderApp(v2Api(), "/odjeli/rooms");
    expect(await screen.findByRole("heading", { level: 1, name: "Smještaj" })).toBeTruthy();
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("Otišlo u tjednu")).toBeTruthy();
    expect(within(people).getByText("2 osobe")).toBeTruthy();
    expect(
      within(people).getByText("Tko ode, zamijeni se novim zaposlenim; trošak zamjene je u Upravi."),
    ).toBeTruthy();
    cleanup();
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    const quiet = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(quiet).getByText("Otišlo u tjednu")).toBeTruthy();
    expect(within(quiet).getByText("nitko")).toBeTruthy();
    expect(within(quiet).queryByText(/trošak zamjene/)).toBeNull();
  });

  it("ljudi u mjesečnoj v2 igri: odlasci u mjesecu", async () => {
    renderApp(v2Api().on("GET", "/v1/games/:game", { ...game(), model: "v2" }), "/odjeli/rooms");
    expect(await screen.findByRole("heading", { level: 1, name: "Smještaj" })).toBeTruthy();
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("Otišlo u mjesecu")).toBeTruthy();
    expect(within(people).queryByText("Otišlo u tjednu")).toBeNull();
  });

  it("služba: odlasci iz zadnjeg troška službe", async () => {
    renderApp(v2Api(), "/odjeli/admin");
    expect(await screen.findByRole("heading", { level: 1, name: "Uprava" })).toBeTruthy();
    const people = screen.getByRole("heading", { level: 2, name: "Ljudi" }).closest("section") as HTMLElement;
    expect(within(people).getByText("1 osoba")).toBeTruthy();
  });

  it("prostor: prikazane promjene se zbrajaju u prikazanu razliku (78 → 77)", async () => {
    const body = departmentDetailBody("restaurant");
    if (!body?.department.last) throw new Error("restoran bez rezultata");
    // Svaka promjena zasebno zaokružena bila bi 0, 0, 0 uz „78 → 77“.
    const last = {
      ...body.department.last,
      condition_before_pts: 7760,
      wear_pts: -33,
      load_wear_pts: -17,
      maintenance_pts: 0,
      condition_after_pts: 7710,
    };
    renderApp(
      v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", {
        ...body,
        department: { ...body.department, last },
      }),
      "/odjeli/restaurant",
    );
    expect(await screen.findByText("78/100 → 77/100")).toBeTruthy();
    expect(screen.getByText("trošenje −1 bod")).toBeTruthy();
    expect(screen.getByText("preopterećenje 0 bodova")).toBeTruthy();
    expect(screen.getByText("održavanje 0 bodova")).toBeTruthy();
  });

  it("povijest: ne lomi se samo naziv kruga, zaglavlja stupaca se smiju prelomiti", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const head = await screen.findByRole("columnheader", { name: "Zadovoljstvo gostiju" });
    expect(head.className).not.toMatch(/whitespace-nowrap/);
    expect(screen.getByRole("rowheader").className).toMatch(/whitespace-nowrap/);
  });

  it("lokal u najmu bez troška održavanja: nema odluka, samo rečenica (N-I3)", async () => {
    const hotel = v2PlanHotel();
    const plan = hotel.private?.plan;
    if (!plan?.plan.departments || !plan.limits.departments?.parking) throw new Error("nema plana");
    plan.plan.departments.parking = {
      open: true,
      seasonal_target: 0,
      price_level_pct: 100,
      maintenance: "regular",
    };
    // Lokal nema cijenu: poslužitelj šalje granice razine 100–100 (O-P).
    Object.assign(plan.limits.departments.parking, {
      seasonal_max: 0,
      can_close: false,
      price_level_min_pct: 100,
      price_level_max_pct: 100,
      maintenance: { deferred_cents: 0, regular_cents: 0, intensive_cents: 0 },
    });
    renderApp(v2Api().on("GET", "/v1/me/hotel", hotel), "/odjeli/parking");
    expect(
      await screen.findByText("Ovdje nema odluka: prihod je ugovoren, a o prostoru brine zakupac."),
    ).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Spremi odluke odjela" })).toBeNull();
    expect(screen.queryByRole("radio")).toBeNull();
  });

  it("desno okno od 1024 px: uputa i kako čitati ocjenu; služba s kratkim natpisom troška", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(
      await screen.findByText(
        "Ocjena odjela je dobra od 80/100, srednja od 60/100 do 79/100, a slaba ispod 60/100; uz boju uvijek piše i riječ.",
      ),
    ).toBeTruthy();
    expect(
      await screen.findByText(
        "Ove službe nemaju prihod, ali hotel na njih troši novac. Uz svaku je trošak zadnjeg tjedna.",
      ),
    ).toBeTruthy();
    expect(screen.getByRole("link", { name: /Uprava/ }).textContent).toContain("Trošak");
    expect(screen.queryByText("Trošak zadnjeg tjedna")).toBeNull();
  });

  it("Restoran (ne zatvara se): uvod ne spominje otvaranje; gumb spremanja je u stupcu odluka", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(
      await screen.findByText(/^Odredi koliko sezonskih radnika ima, razinu cijena i održavanje\./),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Spremi odluke odjela" }).className).toMatch(/md:max-w-md/);
  });
});
