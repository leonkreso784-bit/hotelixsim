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

/** Ton boje u stupnjevima (HSL), 0 = crvena, 60 = žuta. */
function hue(hex: string): number {
  const [r = 0, g = 0, b = 0] = [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16) / 255);
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  if (d === 0) return 0;
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return (h * 60 + 360) % 360;
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
  // Jantarna „srednje“ (ocjena odjela): traka, a ako ikad bude tekst, i on mora biti čitljiv.
  ["warn", "surface"],
  ["warn", "canvas"],
];

describe("kontrast tokena (I3)", () => {
  it.each(TEXT_PAIRS)("%s na %s ≥ 4,5:1", (fg, bg) => {
    expect(contrast(token(fg), token(bg))).toBeGreaterThanOrEqual(4.5);
  });
  it("trake razina ≥ 3:1 na podlozi trake (bg-line)", () => {
    for (const bar of ["up", "warn", "down", "muted"])
      expect(contrast(token(bar), token("line")), bar).toBeGreaterThanOrEqual(3);
  });
  it("jantarna (srednje) se tonom razlikuje od crvene (slabo) bar za 25°", () => {
    expect(hue(token("warn")) - hue(token("down"))).toBeGreaterThanOrEqual(25);
  });
  it("bijeli tekst na bg-accent ≥ 4,5:1", () => {
    expect(contrast("#ffffff", token("accent"))).toBeGreaterThanOrEqual(4.5);
  });
  it("izračun se slaže s poznatim omjerima", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#15924a", "#ffffff")).toBeCloseTo(4.01, 2);
  });
});
