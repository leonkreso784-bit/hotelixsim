import { describe, expect, it } from "vitest";
import css from "./styles.css?raw";

// Kontrast tokena (WCAG 2.1, I3): tekst ispod 18,66 px podebljano traži ≥ 4,5:1.
function token(name: string): string {
  const m = css.match(new RegExp(String.raw`--color-${name}:\s*(#[0-9a-fA-F]{6})`));
  if (!m?.[1]) throw new Error(`nema tokena --color-${name}`);
  return m[1];
}

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const v = Number.parseInt(hex.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0);
}

function contrast(fg: string, bg: string): number {
  const [hi, lo] = [luminance(fg), luminance(bg)].sort((a, b) => b - a);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

// Parovi teksta i podloge koji se stvarno pojavljuju (značke, kartice, poveznice, ljestvica, greške).
const TEXT_PAIRS: [string, string][] = [
  ["up", "surface"],
  ["up", "canvas"],
  ["up", "mint"],
  ["up", "accent-soft"],
  ["down", "surface"],
  ["down", "canvas"],
  ["down", "sand"],
  ["down", "accent-soft"],
  ["accent-ink", "surface"],
  ["accent-ink", "canvas"],
  ["accent-ink", "accent-soft"],
  ["accent-ink", "mint"],
  ["muted", "surface"],
  ["muted", "canvas"],
  ["muted", "line"],
  ["ink", "surface"],
];

describe("kontrast tokena (I3)", () => {
  it.each(TEXT_PAIRS)("%s na %s ≥ 4,5:1", (fg, bg) => {
    expect(contrast(token(fg), token(bg))).toBeGreaterThanOrEqual(4.5);
  });
  it("bijeli tekst na bg-accent ≥ 4,5:1", () => {
    expect(contrast("#ffffff", token("accent"))).toBeGreaterThanOrEqual(4.5);
  });
  it("izračun se slaže s poznatim omjerima", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#15924a", "#ffffff")).toBeCloseTo(4.01, 2);
  });
});
