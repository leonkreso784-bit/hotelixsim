import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { IndexEntry } from "../api/types";
import { Delta, IndexBars } from "./IndexBars";
import { Button, Card } from "./primitives";

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
  it("vrijednost i opis trake su cijeli brojevi (4876 → 49)", () => {
    render(
      <IndexBars
        indices={[{ code: "G", hundredths: 4876 }, ...ix.slice(1)]}
        deltas={null}
        deltaLabel={null}
      />,
    );
    expect(screen.getByText("49")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Zadovoljstvo gostiju: 49 od 100" })).toBeTruthy();
  });
  it("naziv indeksa nije rezan (bez truncate) i traka je na svojem retku", () => {
    render(<IndexBars indices={ix} deltas={null} deltaLabel={null} />);
    const name = screen.getByText("Zadovoljstvo gostiju");
    expect(name.className).not.toContain("truncate");
    const bar = screen.getByRole("img", { name: /Zadovoljstvo gostiju/ });
    expect(bar.className).toContain("col-span-full");
  });
  it("uz promjene pokazuje ▲/▼ s predznakom", () => {
    render(
      <IndexBars
        indices={ix}
        deltas={
          new Map([
            ["G", 3],
            ["F", -2],
            ["R", 0],
            ["Z", 1],
            ["O", 4],
          ])
        }
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
  it('Button je zadano pune širine, a width="auto" ga ne širi', () => {
    render(
      <>
        <Button>Puni</Button>
        <Button width="auto">Uski</Button>
      </>,
    );
    expect(screen.getByRole("button", { name: "Puni" }).className).toContain("w-full");
    const narrow = screen.getByRole("button", { name: "Uski" }).className;
    expect(narrow).not.toContain("w-full");
    expect(narrow).toContain("w-auto");
  });
});

describe("Card", () => {
  it("zadano: bijela, sivi obrub, p-4", () => {
    render(<Card>x</Card>);
    const c = screen.getByText("x").closest("section") as HTMLElement;
    expect(c.className).toContain("bg-surface");
    expect(c.className).toContain("border-line");
    expect(c.className).toContain("p-4");
  });
  it("tone, emphasis i pad zamjenjuju zadane klase umjesto da se lijepe uz njih (I1)", () => {
    render(
      <Card tone="mint" emphasis pad="sm">
        y
      </Card>,
    );
    const c = (screen.getByText("y").closest("section") as HTMLElement).className.split(/\s+/);
    expect(c).toContain("bg-mint");
    expect(c).not.toContain("bg-surface");
    expect(c).toContain("border-accent");
    expect(c).not.toContain("border-line");
    expect(c).toContain("p-3");
    expect(c).not.toContain("p-4");
  });
});

describe("Card u ekranima", () => {
  // Boja, obrub i padding u `className` kartice tiho gube od zadanih klasa (I1) — idu samo kroz propove.
  const sources = import.meta.glob("../**/*.tsx", { query: "?raw", import: "default", eager: true });
  it("nijedna <Card> ne dobiva bg-*, border-* ni p-* kroz className", () => {
    expect(Object.keys(sources).some((f) => f.endsWith("MonthScreen.tsx"))).toBe(true);
    const bad: string[] = [];
    for (const [file, text] of Object.entries(sources)) {
      if (file.includes(".test.")) continue;
      for (const m of String(text).matchAll(/<Card\b[^>]*?className="([^"]*)"/g)) {
        const cls = (m[1] ?? "").split(/\s+/);
        if (cls.some((c) => /^(?:[a-z]+:)*(?:bg-|border-(?!0)|p[xytblr]?-\d)/.test(c)))
          bad.push(`${file}: ${m[1]}`);
      }
    }
    expect(bad).toEqual([]);
  });
});
