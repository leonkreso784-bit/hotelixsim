// test/renderApp.tsx
import { createApiClient } from "@hotelixsim/api-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { vi } from "vitest";
import { App } from "../App";
import { ApiProvider } from "../api/ApiContext";
import { poll } from "../api/poll";
import { clearDrafts } from "../drafts";
import type { FakeApi } from "./fakeApi";

/** Cijela aplikacija na zadanoj putanji, nad lažnim API-jem. `fetch` se lažira PRIJE stvaranja klijenta. */
export function renderApp(api: FakeApi, path = "/") {
  // Nespremljene izmjene žive u sessionStorage-u preglednika; svaki test počinje bez njih.
  clearDrafts();
  window.history.replaceState(null, "", path);
  vi.stubGlobal("fetch", vi.fn(api.fetch));
  const client = createApiClient("http://localhost");
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(
    <ApiProvider client={client}>
      <QueryClientProvider client={qc}>
        <App />
      </QueryClientProvider>
    </ApiProvider>,
  );
}

/** Ispitivanje u testu: svakih 20 ms, „sporo“ nakon 200 ms. */
export function fastPolling() {
  Object.assign(poll, {
    conversationMs: 20,
    verdictMs: 20,
    roundMs: 20,
    feedMs: 20,
    slowRoundMs: 200,
    maxFeedPolls: 5,
  });
}
