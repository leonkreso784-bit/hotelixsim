### Task 7: Jedan ekran (`apps/web`) i prijevodi (`packages/i18n`)

**Files:**
- Create: `packages/i18n/package.json`, `packages/i18n/tsconfig.json`, `packages/i18n/src/hr.ts`, `packages/i18n/src/index.ts`
- Create: `apps/web/package.json`, `apps/web/tsconfig.json`, `apps/web/vite.config.ts`, `apps/web/index.html`, `apps/web/src/main.tsx`, `apps/web/src/App.tsx`, `apps/web/src/format.ts`, `apps/web/src/App.test.tsx`

**Interfaces:**
- Consumes: `@hotelixsim/api-client` (`createApiClient`, `ApiClient`)
- Produces: `@hotelixsim/i18n` s `t(key: MessageKey): string` i tipom `MessageKey`; `apps/web` komponenta `App({ client }: { client: ApiClient })`; `formatIndex(hundredths: number): string` (npr. `5000` → `"50"`, `5025` → `"50,25"`).

Mali vlastiti rječnik umjesto i18next-a je namjeran za kostur (YAGNI): ispunjava K9 (tekst odvojen od komponenti), a i18next dolazi kad zatreba množina ili drugi jezik.

- [ ] **Step 1: `packages/i18n`**

`packages/i18n/package.json`:

```json
{
  "name": "@hotelixsim/i18n",
  "private": true,
  "type": "module",
  "main": "src/index.ts",
  "types": "src/index.ts",
  "scripts": { "typecheck": "tsc -p tsconfig.json" },
  "devDependencies": { "typescript": "5.9.3" }
}
```

`packages/i18n/tsconfig.json`: `{ "extends": "../../tsconfig.base.json", "include": ["src"] }`

`packages/i18n/src/hr.ts`:

```ts
export const hr = {
  "app.title": "HotelixSim",
  "status.loading": "Učitavam…",
  "status.db.ok": "Baza: spojena",
  "status.db.error": "Baza: nije dostupna",
  "status.api.error": "Poslužitelj nije dostupan",
  "status.events": "Događaja u dnevniku",
  "profile.heading": "Početni indeksi — profil",
  "index.G": "Zadovoljstvo gostiju",
  "index.R": "Recenzije",
  "index.Z": "Zaposlenici",
  "index.O": "Operativa",
  "index.F": "Financije",
} as const;
```

`packages/i18n/src/index.ts`:

```ts
import { hr } from "./hr";

export type MessageKey = keyof typeof hr;

export function t(key: MessageKey): string {
  return hr[key];
}
```

- [ ] **Step 2: Test koji pada — `apps/web/src/App.test.tsx`**

```tsx
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
```

- [ ] **Step 3: `apps/web` konfiguracija**

`apps/web/package.json`:

```json
{
  "name": "@hotelixsim/web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -p tsconfig.json && vite build",
    "typecheck": "tsc -p tsconfig.json",
    "test": "vitest run"
  },
  "dependencies": {
    "@hotelixsim/api-client": "workspace:*",
    "@hotelixsim/i18n": "workspace:*",
    "@tanstack/react-query": "5.104.1",
    "react": "19.3.0",
    "react-dom": "19.3.0"
  },
  "devDependencies": {
    "@testing-library/dom": "10.4.2",
    "@testing-library/react": "16.3.3",
    "@types/react": "19.3.0",
    "@types/react-dom": "19.3.0",
    "@vitejs/plugin-react": "6.1.1",
    "jsdom": "30.1.1",
    "typescript": "5.9.3",
    "vite": "8.3.2",
    "vitest": "5.0.3"
  }
}
```

`apps/web/tsconfig.json`:

```json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": { "jsx": "react-jsx", "lib": ["ES2023", "DOM", "DOM.Iterable"], "types": ["vite/client"] },
  "include": ["src", "vite.config.ts"]
}
```

`apps/web/vite.config.ts`:

```ts
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: { "/v1": "http://127.0.0.1:8080" },
  },
  test: { environment: "jsdom" },
});
```

`apps/web/index.html`:

```html
<!doctype html>
<html lang="hr">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>HotelixSim</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

Run: `corepack pnpm install; echo EXIT=$?` → `EXIT=0`
Run: `corepack pnpm --filter @hotelixsim/web test; echo EXIT=$?` → pada jer `./App` i `./format` ne postoje, `EXIT≠0`.

Napomena: openapi-fetch 0.17 uzima `globalThis.fetch` pri `createClient` i zove ga s jednim `Request` objektom, pa test zamijeni `fetch` **prije** `createApiClient`. Ako izvedba ipak zove `fetch(url, init)`, prilagodi samo lažni `fetch` u testu (čitaj URL iz prvog argumenta bilo kojeg oblika), ne tvrdnje.

- [ ] **Step 4: Izvedba**

`apps/web/src/format.ts`:

```ts
const nf = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });

/** Indeks u stotinkama (0–10000) → tekst za prikaz, npr. 5025 → "50,25". */
export function formatIndex(hundredths: number): string {
  return nf.format(hundredths / 100);
}
```

`apps/web/src/App.tsx`:

```tsx
import type { ApiClient } from "@hotelixsim/api-client";
import { type MessageKey, t } from "@hotelixsim/i18n";
import { useQuery } from "@tanstack/react-query";
import { formatIndex } from "./format";

const PROFILE = "C";

export function App({ client }: { client: ApiClient }) {
  const health = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const { data, error, response } = await client.GET("/v1/health");
      if (data) return data;
      if (response.status === 503 && error) return error;
      throw new Error(`health ${response.status}`);
    },
  });
  const indices = useQuery({
    queryKey: ["indices", PROFILE],
    queryFn: async () => {
      const { data, response } = await client.GET("/v1/demo/profiles/{profile}/indices", {
        params: { path: { profile: PROFILE } },
      });
      if (!data) throw new Error(`indices ${response.status}`);
      return data;
    },
  });

  return (
    <main>
      <h1>{t("app.title")}</h1>
      <p>
        {health.isPending
          ? t("status.loading")
          : health.isError
            ? t("status.api.error")
            : health.data.database === "ok"
              ? t("status.db.ok")
              : t("status.db.error")}
      </p>
      {health.data?.events != null && (
        <p>
          {t("status.events")}: {health.data.events}
        </p>
      )}
      <h2>
        {t("profile.heading")} {PROFILE}
      </h2>
      <ul>
        {indices.data?.indices.map((i) => (
          <li key={i.code}>
            <span>{t(`index.${i.code}` as MessageKey)}</span> <strong>{formatIndex(i.hundredths)}</strong>
          </li>
        ))}
      </ul>
    </main>
  );
}
```

Napomena za izvođača: ako generirani tip za 503 odgovor ne daje `error` istog oblika kao `data`, zadrži ponašanje iz testova (503 → „Baza: nije dostupna") i prilagodi samo tipiziranje; razlog zapiši u izvještaj.

`apps/web/src/main.tsx`:

```tsx
import { createApiClient } from "@hotelixsim/api-client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("nema #root elementa");

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={new QueryClient()}>
      <App client={createApiClient(window.location.origin)} />
    </QueryClientProvider>
  </StrictMode>,
);
```

- [ ] **Step 5: Pokreni — mora proći**

Run: `corepack pnpm --filter @hotelixsim/web test; echo EXIT=$?` → završni redak `Tests  4 passed (4)`, `EXIT=0`.
Run: `corepack pnpm typecheck; echo EXIT=$?` → `EXIT=0`.
Run: `corepack pnpm lint; echo EXIT=$?` → `EXIT=0` (ako Biome traži samo formatiranje: `corepack pnpm exec biome check --write .`, pa ponovi).

- [ ] **Step 6: Commit** — `git add packages/i18n apps/web pnpm-lock.yaml` · poruka: „Kostur 7/8: jedan ekran (zdravlje i indeksi) i prijevodi"

---

