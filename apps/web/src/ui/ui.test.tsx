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
        deltas={
          new Map([
            ["G", 300],
            ["F", -200],
            ["R", 0],
            ["Z", 100],
            ["O", 400],
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
});
