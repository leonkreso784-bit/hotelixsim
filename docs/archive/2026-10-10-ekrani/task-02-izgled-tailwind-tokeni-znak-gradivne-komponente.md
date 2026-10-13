### Task 2: Izgled — Tailwind, tokeni, znak, gradivne komponente, build u CI-ju

**Files:**
- Modify: `apps/web/package.json`, `pnpm-lock.yaml`, `apps/web/vite.config.ts`, `apps/web/index.html`, `biome.json`,
  `.github/workflows/ci.yml`
- Create: `apps/web/src/styles.css`, `apps/web/public/logo-znak.png`, `apps/web/src/ui/primitives.tsx`,
  `apps/web/src/ui/IndexBars.tsx`, `apps/web/src/ui/ui.test.tsx`

**Interfaces:**
- Consumes: `formatIndex`, `formatSigned` (Task 1); `IndexEntry`, `IndexCode` (Task 1); `indexLabel` iz i18n.
- Produces:
  - `Card({ children, className? })`, `Button({ variant?: "primary" | "secondary", ...button props })`,
    `Badge({ children, tone?: "accent" | "muted" | "up" | "down" })`, `StatusText({ children })` (`role="status"`),
    `ErrorText({ children })` (`role="alert"`) — iz `ui/primitives.tsx`.
  - `IndexBars({ indices: IndexEntry[]; deltas: Map<IndexCode, number> | null; deltaLabel: string | null })`,
    `Delta({ value: number | null; size?: "sm" | "lg" })` — `ui/IndexBars.tsx`.
  - Tailwind tokeni: `accent, accent-soft, ink, muted, line, canvas, surface, sand, mint, up, down` (boje), `rounded-card`,
    `font-sans` = Plus Jakarta Sans.

- [ ] **Step 1: Dodaj pakete** (verzije po pravilu „starije od 14 dana“ iz Global Constraints; zapiši izabrane u izvještaj)

```bash
cd /c/Users/leonk/Documents/hotelixsim && corepack pnpm view tailwindcss time --json > /dev/null; corepack pnpm --filter @hotelixsim/web add -E tailwindcss@<v> @tailwindcss/vite@<v> lucide-react@<v> @fontsource-variable/plus-jakarta-sans@<v>; echo EXIT=$?
```

`tailwindcss` i `@tailwindcss/vite` idu u `dependencies` web paketa kao i ostali (build ih treba), iste verzije.

- [ ] **Step 2: `apps/web/vite.config.ts`**

```ts
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: { "/v1": "http://127.0.0.1:8080" },
  },
  test: { environment: "jsdom" },
});
```

- [ ] **Step 3: `apps/web/src/styles.css`** — jedini izvor izgleda (DIZAJN.md §1)

```css
@import "tailwindcss";
@import "@fontsource-variable/plus-jakarta-sans";

/* Izgled HotelixSima: mijenja se OVDJE, ne u ekranima (DIZAJN.md §1, ADR-022 t. 2). */
@theme {
  --font-sans: "Plus Jakarta Sans Variable", ui-sans-serif, system-ui, "Segoe UI", Roboto, sans-serif;
  --color-accent: #246bfd;
  --color-accent-soft: #eaf1ff;
  --color-ink: #151b2b;
  --color-muted: #5d6679;
  --color-line: #e2e8f3;
  --color-canvas: #f5f8fd;
  --color-surface: #ffffff;
  --color-sand: #f4e8d5;
  --color-mint: #d8f0ee;
  --color-up: #15924a;
  --color-down: #dc2f2f;
  --radius-card: 1.25rem;
}

html {
  background-color: var(--color-canvas);
  color: var(--color-ink);
  font-family: var(--font-sans);
  -webkit-text-size-adjust: 100%;
}
```

- [ ] **Step 4: `biome.json`** — Biome mora razumjeti Tailwind direktive. Dodaj na vrh objekta:

```json
  "css": { "parser": { "tailwindDirectives": true } },
```

Provjeri `corepack pnpm lint; echo EXIT=$?`. Ako Biome 2.5.15 ne zna tu opciju (greška konfiguracije), umjesto nje dodaj
`"!apps/web/src/styles.css"` u `files.includes` i zapiši to u izvještaj.

- [ ] **Step 5: Znak loga** — skripta Write alatom u scratchpad (npr. `<scratchpad>/znak.py`), pa
  `cd /c/Users/leonk/Documents/hotelixsim && PYTHONUTF8=1 python <scratchpad>/znak.py; echo EXIT=$?`

```python
from PIL import Image
im = Image.open("docs/product/dizajn/logo.png").convert("RGBA")
znak = im.crop((300, 205, 953, 848)).resize((128, 128), Image.LANCZOS)
znak.save("apps/web/public/logo-znak.png", optimize=True)
print(znak.size)
```

Otvori `apps/web/public/logo-znak.png` Read alatom i potvrdi da je cijeli plavi kvadrat s „H“ (bez teksta „HotelixSim“).

- [ ] **Step 6: `apps/web/index.html`** — u `<head>` dodaj:

```html
    <link rel="icon" type="image/png" href="/logo-znak.png" />
    <meta name="theme-color" content="#246BFD" />
```

- [ ] **Step 7: Napiši pad testova `apps/web/src/ui/ui.test.tsx`**

```tsx
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { IndexEntry } from "../api/types";
import { Delta, IndexBars } from "./IndexBars";
import { Button } from "./primitives";

afterEach(cleanup);

const ix: IndexEntry[] = [
  { code: "G", hundredths: 6200 },
  { code: "R", hundredths: 5800 },
  { code: "Z", hundredths: 6500 },
  { code: "O", hundredths: 5400 },
  { code: "F", hundredths: 6000 },
];

describe("IndexBars", () => {
  it("pokazuje pet indeksa s nazivom, vrijednošću i opisom trake", () => {
    render(<IndexBars indices={ix} deltas={null} deltaLabel={null} />);
    expect(screen.getByText("Zadovoljstvo gostiju")).toBeTruthy();
    expect(screen.getByText("62")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Recenzije: 58 od 100" })).toBeTruthy();
    expect(screen.getByText("50 = prosjek usporedivog hotela")).toBeTruthy();
  });
  it("uz promjene pokazuje ▲/▼ s predznakom", () => {
    render(
      <IndexBars
        indices={ix}
        deltas={new Map([["G", 300], ["F", -200], ["R", 0], ["Z", 100], ["O", 400]])}
        deltaLabel="od početka"
      />,
    );
    expect(screen.getByText("▲ +3")).toBeTruthy();
    expect(screen.getByText("▼ −2")).toBeTruthy();
    expect(screen.getByText("od početka")).toBeTruthy();
  });
});

describe("Delta i Button", () => {
  it("Delta bez vrijednosti je prazna", () => {
    const { container } = render(<Delta value={null} />);
    expect(container.textContent).toBe("");
  });
  it("Button je type=button i poštuje disabled", () => {
    render(<Button disabled>Spremi</Button>);
    const b = screen.getByRole("button", { name: "Spremi" }) as HTMLButtonElement;
    expect(b.type).toBe("button");
    expect(b.disabled).toBe(true);
  });
});
```

Dodaj u `hr.ts`: `"index.scale": "50 = prosjek usporedivog hotela"`, `"index.bar": "{label}: {value} od 100"`.

- [ ] **Step 8: Pokreni i vidi pad** — `… vitest run src/ui; echo EXIT=$?` → FAIL (nema modula).

- [ ] **Step 9: `apps/web/src/ui/primitives.tsx`**

```tsx
import type { ButtonHTMLAttributes, ReactNode } from "react";

export function Card({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <section className={`rounded-card border border-line bg-surface p-4 ${className}`}>{children}</section>;
}

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "secondary" };

export function Button({ variant = "primary", className = "", type = "button", ...rest }: ButtonProps) {
  const look = variant === "primary" ? "bg-accent text-white" : "bg-accent-soft text-accent";
  return (
    <button
      type={type}
      className={`min-h-12 w-full rounded-2xl px-4 py-3 text-base font-bold disabled:opacity-50 ${look} ${className}`}
      {...rest}
    />
  );
}

const BADGE = {
  accent: "bg-accent-soft text-accent",
  muted: "bg-line text-muted",
  up: "bg-mint text-up",
  down: "bg-sand text-down",
} as const;

export function Badge({ children, tone = "accent" }: { children: ReactNode; tone?: keyof typeof BADGE }) {
  return (
    <span className={`inline-flex min-w-7 justify-center rounded-full px-2 py-0.5 text-sm font-bold ${BADGE[tone]}`}>
      {children}
    </span>
  );
}

export function StatusText({ children }: { children: ReactNode }) {
  return (
    <p role="status" className="py-2 text-muted">
      {children}
    </p>
  );
}

export function ErrorText({ children }: { children: ReactNode }) {
  return (
    <p role="alert" className="rounded-xl bg-sand px-3 py-2 text-sm text-down">
      {children}
    </p>
  );
}
```

- [ ] **Step 10: `apps/web/src/ui/IndexBars.tsx`** (ikone iz `lucide-react`; ako neko ime ne postoji u izabranoj verziji,
  typecheck pada — uzmi najbliže s lucide.dev i zapiši)

```tsx
import { indexLabel, t } from "@hotelixsim/i18n";
import { Coins, MessageSquare, Settings, User, Users } from "lucide-react";
import type { ReactNode } from "react";
import type { IndexCode, IndexEntry } from "../api/types";
import { formatIndex, formatSigned } from "../format";
import { sortIndices } from "../game";

const ICON: Record<IndexCode, ReactNode> = {
  G: <User size={18} aria-hidden="true" />,
  R: <MessageSquare size={18} aria-hidden="true" />,
  Z: <Users size={18} aria-hidden="true" />,
  O: <Settings size={18} aria-hidden="true" />,
  F: <Coins size={18} aria-hidden="true" />,
};

export function Delta({ value, size = "sm" }: { value: number | null; size?: "sm" | "lg" }) {
  if (value === null) return <span />;
  const tone = value > 0 ? "text-up" : value < 0 ? "text-down" : "text-muted";
  const arrow = value > 0 ? "▲ " : value < 0 ? "▼ " : "";
  const text = size === "lg" ? "text-3xl font-extrabold" : "text-sm font-bold";
  return <span className={`whitespace-nowrap tabular-nums ${tone} ${text}`}>{`${arrow}${formatSigned(value)}`}</span>;
}

export function IndexBars({
  indices,
  deltas,
  deltaLabel,
}: {
  indices: IndexEntry[];
  deltas: Map<IndexCode, number> | null;
  deltaLabel: string | null;
}) {
  return (
    <div>
      <div className="flex items-baseline justify-between pb-1 text-xs text-muted">
        <span>{t("index.scale")}</span>
        {deltaLabel && <span>{deltaLabel}</span>}
      </div>
      <ul>
        {sortIndices(indices).map((e) => {
          const label = indexLabel(e.code);
          const value = formatIndex(e.hundredths);
          const pct = Math.max(0, Math.min(100, e.hundredths / 100));
          return (
            <li key={e.code} className="grid grid-cols-[1.25rem_7.5rem_1fr_2.25rem_3.5rem] items-center gap-2 py-1.5">
              <span className="text-accent">{ICON[e.code]}</span>
              <span className="truncate text-sm">{label}</span>
              <span
                role="img"
                aria-label={t("index.bar", { label, value })}
                className="relative h-2 rounded-full bg-line"
              >
                <span className="absolute inset-y-0 left-0 rounded-full bg-accent" style={{ width: `${pct}%` }} />
                <span className="absolute -top-1 left-1/2 h-4 w-0.5 rounded bg-ink" />
              </span>
              <strong className="text-right tabular-nums">{value}</strong>
              <span className="text-right">
                <Delta value={deltas?.get(e.code) ?? null} />
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
```

Na 360 px stupac naziva od 7.5rem može biti tijesan za „Zadovoljstvo gostiju“ — `truncate` to reže; provjera izgleda je
u Tasku 13, ovdje ne mijenjaj.

- [ ] **Step 11: Pokreni ui testove** → PASS.

- [ ] **Step 12: CI gradi web** — u `.github/workflows/ci.yml`, poslu weba, iza `- run: pnpm test` dodaj:

```yaml
      - run: pnpm --filter @hotelixsim/web build
```

- [ ] **Step 13: Provjere** (sve četiri). Uz to potvrdi da build uključuje tokene:
  `cd /c/Users/leonk/Documents/hotelixsim && grep -c "246bfd" apps/web/dist/assets/*.css; echo EXIT=$?` → broj ≥ 1.
  `apps/web/dist` je izlaz builda — provjeri da je u `.gitignore` (`git status --short` ga ne smije pokazati); ako nije, dodaj `dist` u `.gitignore`.

- [ ] **Step 14: Commit** — izričite putanje svih datoteka iz „Files“ → „Ekrani 2/11: Tailwind, tokeni izgleda, znak i
  gradivne komponente".

---

