### Task 6: `openapi.json` i generirani TypeScript klijent

**Files:**
- Create: `openapi.json` (generirano), `package.json`, `pnpm-workspace.yaml`, `biome.json`, `tsconfig.base.json`
- Create: `packages/api-client/package.json`, `packages/api-client/src/index.ts`, `packages/api-client/src/schema.d.ts` (generirano)
- Modify: `.gitignore` (dodaj `target/`, `dist/`)

**Interfaces:**
- Consumes: `cargo run -q -p server -- openapi`
- Produces: paket `@hotelixsim/api-client` s izvozom `createApiClient(baseUrl: string)` (openapi-fetch klijent tipiziran s `paths`) i tipovima `paths`, `components`; skripte `corepack pnpm gen:api` i `corepack pnpm check:api`.

- [ ] **Step 1: Korijenski `package.json`**

```json
{
  "name": "hotelixsim",
  "private": true,
  "packageManager": "pnpm@10.34.6",
  "engines": { "node": ">=24" },
  "scripts": {
    "gen:openapi": "node scripts/gen-openapi.mjs",
    "gen:api": "pnpm gen:openapi && pnpm --filter @hotelixsim/api-client gen",
    "check:api": "pnpm gen:api && git diff --exit-code -- openapi.json packages/api-client/src/schema.d.ts",
    "lint": "biome check .",
    "typecheck": "pnpm -r typecheck",
    "test": "pnpm -r test"
  },
  "devDependencies": {
    "@biomejs/biome": "2.5.15",
    "typescript": "5.9.3"
  }
}
```

`scripts/gen-openapi.mjs` (zapisuje točno izlaz servera, bez pretvorbe završetaka redaka):

```js
// Generira openapi.json iz Rust opisa API-ja. ARHITEKTURA §11.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const json = execFileSync("cargo", ["run", "-q", "-p", "server", "--", "openapi"], { encoding: "utf8" });
if (!json.startsWith("{") || !json.includes('"openapi": "3.1.0"')) {
  console.error("Neočekivan izlaz servera (nije OpenAPI 3.1):", json.slice(0, 200));
  process.exit(1);
}
writeFileSync("openapi.json", json);
console.log(`openapi.json: ${json.length} znakova`);
```

`pnpm-workspace.yaml`:

```yaml
packages:
  - "apps/*"
  - "packages/*"
```

`tsconfig.base.json`:

```json
{
  "compilerOptions": {
    "target": "ES2023",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "verbatimModuleSyntax": true,
    "noEmit": true
  }
}
```

`biome.json`:

```json
{
  "$schema": "https://biomejs.dev/schemas/2.5.15/schema.json",
  "files": { "includes": ["**", "!**/schema.d.ts", "!**/dist", "!**/target", "!openapi.json"] },
  "formatter": { "indentStyle": "space", "indentWidth": 2, "lineWidth": 110 },
  "linter": { "enabled": true, "rules": { "recommended": true } }
}
```

Ako Biome 2.5 javi grešku u obliku `files.includes`, pokreni `corepack pnpm exec biome migrate --write` i zadrži isto značenje (isključene: `schema.d.ts`, `dist`, `target`, `openapi.json`).

- [ ] **Step 2: Paket `packages/api-client`**

`packages/api-client/package.json`:

```json
{
  "name": "@hotelixsim/api-client",
  "private": true,
  "type": "module",
  "main": "src/index.ts",
  "types": "src/index.ts",
  "scripts": {
    "gen": "openapi-typescript ../../openapi.json -o src/schema.d.ts",
    "typecheck": "tsc -p tsconfig.json"
  },
  "dependencies": { "openapi-fetch": "0.17.0" },
  "devDependencies": { "openapi-typescript": "7.13.0", "typescript": "5.9.3" }
}
```

`packages/api-client/tsconfig.json`:

```json
{ "extends": "../../tsconfig.base.json", "include": ["src"] }
```

`packages/api-client/src/index.ts`:

```ts
// Jedini put do API-ja iz sučelja. Tipovi su GENERIRANI iz openapi.json; ručno se ne pišu.
import createClient from "openapi-fetch";
import type { components, paths } from "./schema";

export type { components, paths };

export function createApiClient(baseUrl: string) {
  return createClient<paths>({ baseUrl });
}

export type ApiClient = ReturnType<typeof createApiClient>;
```

- [ ] **Step 3: Instalacija i generiranje**

Dodaj u `.gitignore` retke `target/` i `dist/`.
Run: `COREPACK_ENABLE_DOWNLOAD_PROMPT=0 corepack pnpm install; echo EXIT=$?` → `EXIT=0`; `ls node_modules | wc -l` > 0.
Run: `corepack pnpm gen:api; echo EXIT=$?` → `openapi.json: N znakova`, `EXIT=0`; nastaje `packages/api-client/src/schema.d.ts` s `"/v1/health"` i `"/v1/demo/profiles/{profile}/indices"`.
Run: `corepack pnpm --filter @hotelixsim/api-client typecheck; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 4: Stabilnost generiranja (Review Focus 5)**

Commitaj generirane datoteke (Step 5), pa:
Run: `corepack pnpm check:api; echo EXIT=$?` → `EXIT=0` i **prazan** `git diff`. Ako `git diff` pokaže samo završetke redaka, popravak je u `.gitattributes`/pisanju datoteke, ne u isključivanju provjere.

- [ ] **Step 5: Commit** — `git add package.json pnpm-lock.yaml pnpm-workspace.yaml biome.json tsconfig.base.json .gitignore scripts/gen-openapi.mjs openapi.json packages/api-client` · poruka: „Kostur 6/8: openapi.json i generirani TS klijent"

---

