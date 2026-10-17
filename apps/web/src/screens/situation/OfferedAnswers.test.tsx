import { cleanup, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { s1, s3, situations, standardApi } from "../../test/fixtures";
import { fastPolling, renderApp } from "../../test/renderApp";

beforeEach(fastPolling);
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

// PRAVILO-c (ADR-047, ADR-056 A5): kad je klik ograničen, student to vidi PRIJE nego klikne.
describe("napomena o pravilu klika", () => {
  it("uz click_capped: true piše da klik daje najviše 4 od 5, a puni 5 su za vlastite riječi", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1({ click_capped: true }), s3()]),
    );
    renderApp(api, "/situacije/S1");
    expect(await screen.findByText(/Klikom dobivaš najviše 4 od 5/)).toBeTruthy();
    expect(screen.getByText(/Za punih 5 odgovori gostu svojim riječima/)).toBeTruthy();
  });

  it("uz click_capped: false napomene nema", async () => {
    const api = standardApi().on(
      "GET",
      "/v1/hotels/:hotel/situations",
      situations([s1({ click_capped: false }), s3()]),
    );
    renderApp(api, "/situacije/S1");
    await screen.findByRole("heading", { name: "Ili izaberi jedan od ponuđenih" });
    expect(screen.queryByText(/najviše 4 od 5/)).toBeNull();
  });
});
