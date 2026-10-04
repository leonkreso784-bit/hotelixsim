import { createApiClient } from "@hotelixsim/api-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { formatIndex } from "./format";

function jsonResponse(status: number, body: unknown) {
  return new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });
}

function renderWith(fetchImpl: (req: Request) => Promise<Response>) {
  vi.stubGlobal("fetch", vi.fn(fetchImpl));
  const client = createApiClient("http://localhost");
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <App client={client} />
    </QueryClientProvider>,
  );
}

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("formatIndex", () => {
  it("prikazuje stotinke hrvatskim zarezom i bez suvišnih nula", () => {
    expect(formatIndex(5000)).toBe("50");
    expect(formatIndex(5025)).toBe("50,25");
    expect(formatIndex(0)).toBe("0");
    expect(formatIndex(10000)).toBe("100");
  });
});

describe("App", () => {
  it("prikazuje spojenu bazu i pet indeksa profila C", async () => {
    renderWith(async (req) => {
      const url = new URL(req.url);
      if (url.pathname === "/v1/health") return jsonResponse(200, { database: "ok", events: 0 });
      if (url.pathname === "/v1/demo/profiles/C/indices")
        return jsonResponse(200, {
          profile: "C",
          indices: ["G", "R", "Z", "O", "F"].map((code) => ({ code, hundredths: 5000 })),
        });
      return jsonResponse(404, {});
    });
    expect(await screen.findByText("Baza: spojena")).toBeTruthy();
    expect(await screen.findByText("Zadovoljstvo gostiju")).toBeTruthy();
    expect(screen.getByText("Financije")).toBeTruthy();
    expect(screen.getAllByText("50")).toHaveLength(5);
  });

  it("javlja nedostupnu bazu kad zdravlje vrati 503", async () => {
    renderWith(async (req) => {
      const url = new URL(req.url);
      if (url.pathname === "/v1/health") return jsonResponse(503, { database: "error", events: null });
      return jsonResponse(200, { profile: "C", indices: [] });
    });
    expect(await screen.findByText("Baza: nije dostupna")).toBeTruthy();
  });

  it("javlja nedostupan poslužitelj kad mreža padne", async () => {
    renderWith(async () => {
      throw new TypeError("Failed to fetch");
    });
    expect(await screen.findByText("Poslužitelj nije dostupan")).toBeTruthy();
  });
});
