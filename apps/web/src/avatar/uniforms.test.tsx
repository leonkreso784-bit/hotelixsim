// IZGLED-c (kartica `task-13c-izgled-uniforme.md`, §7 IZGLED-c-b + §11 Leonove odluke 18.10.): uniforma vanjskog lika
// iz podatka `uniform` (API) — mijenja SAMO odjeću (oblik, boja, kapa, ovratnik), ne lice, kosu ni nakit.
//
// PRETPOSTAVKE O SUČELJU (jedino mjesto; graditelj ih smije uskladiti):
// - `look.ts` izvozi `UNIFORMS`, `type Uniform`, `UNIFORM_LOOK`, `applyUniform`; `LookHint.uniform?: Uniform | null`;
// - nove opcije NA KRAJ nizova: `OUTFITS` + police, firefighter, paramedic, coat; `HATS` + police, fire (kartica §11:
//   vatrogasac IMA kacigu `hat-fire`, pa tablica §4 „fire.hat: none" više ne vrijedi); `NECKWEAR` + stethoscope;
// - `CLOTHES` + dvije boje na kraju (indeks 10 tamnoplava, 11 crvena hitne).
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AvatarGallery } from "../screens/AvatarGallery";
import { Avatar } from "./Avatar";
import {
  type Age,
  type AvatarLook,
  applyUniform,
  HATS,
  isValidLook,
  KNOWN_LOOKS,
  type LookHint,
  lookFor,
  lookFromHint,
  NECKWEAR,
  normalizeLook,
  OUTFITS,
  PLAYER_LOOK,
  UNIFORM_LOOK,
  UNIFORMS,
  type Uniform,
} from "./look";
import { CLOTHES } from "./palette";

afterEach(cleanup);

const NAMES = Array.from({ length: 300 }, (_, i) => `lik-${i}`);
const AGES_HINT: (Age | null)[] = [null, "young", "adult", "senior"];

/** Sve kombinacije roda, dobi i podrijetla (2 x 4 x 2) bez uniforme. */
const HINTS: LookHint[] = (["f", "m"] as const).flatMap((gender) =>
  AGES_HINT.flatMap((age_group) =>
    (["regional", "foreign"] as const).map((origin) => ({ gender, age_group, origin })),
  ),
);

/** Kombinacija za i-to ime: pokriva sve kombinacije već u 300 imena (300 > 16). */
const hintFor = (i: number): LookHint => HINTS[i % HINTS.length] as LookHint;

/** Tablica iz kartice (§4, s kacigom iz §11). */
const EXPECTED: Record<Uniform, Pick<AvatarLook, "outfit" | "clothes" | "hat" | "neckwear">> = {
  police: { outfit: "police", clothes: 10, hat: "police", neckwear: "none" },
  fire: { outfit: "firefighter", clothes: 10, hat: "fire", neckwear: "none" },
  ems: { outfit: "paramedic", clothes: 11, hat: "none", neckwear: "none" },
  doctor: { outfit: "coat", clothes: 6, hat: "none", neckwear: "stethoscope" },
};

const FACE_FIELDS = [
  "skin",
  "hair",
  "hairColor",
  "eyes",
  "mouth",
  "facialHair",
  "glasses",
  "brows",
  "nose",
  "age",
  "earrings",
  "backdrop",
] as const satisfies readonly (keyof AvatarLook)[];

describe("uniforma lika", () => {
  it("bez uniforme isto kao prije: bez polja, null i undefined", () => {
    NAMES.forEach((name, i) => {
      const hint = hintFor(i);
      const want = lookFromHint(name, hint);
      expect(lookFor(name, hint)).toEqual(want);
      expect(lookFor(name, { ...hint, uniform: null })).toEqual(want);
      expect(lookFor(name, { ...hint, uniform: undefined })).toEqual(want);
    });
  });

  it("popis uniformi je točno četiri, redom iz kartice", () => {
    expect([...UNIFORMS]).toEqual(["police", "fire", "ems", "doctor"]);
  });

  it("uniforma iz hinta: odjeća, boja, kapa i vez iz tablice", () => {
    expect(UNIFORM_LOOK).toEqual(EXPECTED);
    NAMES.forEach((name, i) => {
      for (const uniform of UNIFORMS) {
        const l = lookFor(name, { ...hintFor(i), uniform });
        const e = EXPECTED[uniform];
        expect({ outfit: l.outfit, clothes: l.clothes, hat: l.hat, neckwear: l.neckwear }).toEqual(e);
      }
    });
  });

  it("uniforma ne mijenja lice, kosu ni nakit", () => {
    NAMES.forEach((name, i) => {
      const hint = hintFor(i);
      const plain = lookFromHint(name, hint);
      for (const uniform of UNIFORMS) {
        const l = lookFor(name, { ...hint, uniform });
        for (const f of FACE_FIELDS) expect(l[f], `${name} ${uniform} ${f}`).toEqual(plain[f]);
      }
    });
  });

  it("applyUniform ne troši slučajnost: isti ulaz, isti izlaz, i bez uniforme izgled ostaje", () => {
    const base = lookFromHint("lik-7", { gender: "m", age_group: "adult", origin: "regional" });
    for (const uniform of UNIFORMS) expect(applyUniform(base, uniform)).toEqual(applyUniform(base, uniform));
    expect(applyUniform(base, null)).toEqual(base);
    expect(applyUniform(base, undefined)).toEqual(base);
  });

  it("nepoznata uniforma ne ruši", () => {
    const hint: LookHint = { gender: "f", age_group: "adult", origin: "regional" };
    const base = lookFromHint("lik-3", hint);
    expect(applyUniform(base, "pilot" as never)).toEqual(base);
    expect(() => lookFor("lik-3", { ...hint, uniform: "pilot" as never })).not.toThrow();
    expect(lookFor("lik-3", { ...hint, uniform: "pilot" as never })).toEqual(base);
  });

  it("poznati likovi pobjeđuju uniformu", () => {
    const hint: LookHint = { gender: "f", age_group: "adult", origin: "regional", uniform: "police" };
    expect(lookFor("Petra Horvat", hint)).toEqual(KNOWN_LOOKS["Petra Horvat"]);
  });

  it("palete i bazeni samo rastu: stari na istom mjestu, novo na kraju", () => {
    expect([...CLOTHES].slice(0, 10)).toEqual([
      "#246BFD",
      "#15924A",
      "#E0A82E",
      "#7C4DFF",
      "#DC2F2F",
      "#2C3E50",
      "#F7F7F7",
      "#00897B",
      "#C2185B",
      "#8D6E63",
    ]);
    expect(CLOTHES.length).toBe(12);
    const hex = (c: string) => [1, 3, 5].map((i) => Number.parseInt(c.slice(i, i + 2), 16));
    const [r10, g10, b10] = hex(CLOTHES[10] as string) as [number, number, number];
    expect(CLOTHES[10]).toMatch(/^#[0-9A-Fa-f]{6}$/);
    expect(b10).toBeGreaterThan(r10); // tamnoplava
    expect(b10).toBeGreaterThan(g10);
    expect(Math.max(r10, g10, b10)).toBeLessThan(110);
    const [r11, g11, b11] = hex(CLOTHES[11] as string) as [number, number, number];
    expect(CLOTHES[11]).toMatch(/^#[0-9A-Fa-f]{6}$/);
    expect(r11).toBeGreaterThan(150); // crvena hitne
    expect(g11).toBeLessThan(60);
    expect(b11).toBeLessThan(80);
    expect([...OUTFITS].slice(0, 5)).toEqual(["shirt", "blazer", "uniform", "apron", "tshirt"]);
    expect([...HATS].slice(0, 4)).toEqual(["none", "chef", "cap", "beanie"]);
    expect([...NECKWEAR].slice(0, 4)).toEqual(["none", "tie", "scarf", "bowtie"]);
    for (const o of ["police", "firefighter", "paramedic", "coat"]) expect(OUTFITS).toContain(o);
    for (const h of ["police", "fire"]) expect(HATS).toContain(h);
    expect(NECKWEAR).toContain("stethoscope");
  });

  it("uniforma je uvijek valjan izgled, a normalizeLook zadržava nove opcije", () => {
    NAMES.forEach((name, i) => {
      for (const uniform of UNIFORMS)
        expect(isValidLook(lookFor(name, { ...hintFor(i), uniform }))).toBe(true);
    });
    const l = normalizeLook({ ...PLAYER_LOOK, outfit: "police", hat: "police", neckwear: "stethoscope" });
    expect(l.outfit).toBe("police");
    expect(l.hat).toBe("police");
    expect(l.neckwear).toBe("stethoscope");
    expect(normalizeLook({ ...PLAYER_LOOK, hat: "fire" }).hat).toBe("fire");
  });
});

describe("slojevi službe u Avatar-u", () => {
  const parts = (uniform: Uniform) => {
    const look = lookFor("Test Lik", { gender: "m", age_group: "adult", origin: "regional", uniform });
    const { container, unmount } = render(<Avatar look={look} />);
    const found = [...container.querySelectorAll("[data-part]")].map((e) => e.getAttribute("data-part"));
    unmount();
    return found;
  };

  it("policija: odora i šapka", () => {
    const p = parts("police");
    expect(p).toContain("outfit-police");
    expect(p).toContain("hat-police");
  });
  it("vatrogasac: odora i kaciga", () => {
    const p = parts("fire");
    expect(p).toContain("outfit-firefighter");
    expect(p).toContain("hat-fire");
  });
  it("hitna: odora, bez kape", () => {
    const p = parts("ems");
    expect(p).toContain("outfit-paramedic");
    expect(p.some((x) => x?.startsWith("hat-"))).toBe(false);
  });
  it("liječnik: kuta i stetoskop, bez kape", () => {
    const p = parts("doctor");
    expect(p).toContain("outfit-coat");
    expect(p).toContain("neckwear-stethoscope");
    expect(p.some((x) => x?.startsWith("hat-"))).toBe(false);
  });
  it("lik bez uniforme ne crta nijedan sloj službe", () => {
    const look = lookFor("Test Lik", { gender: "m", age_group: "adult", origin: "regional" });
    const { container } = render(<Avatar look={look} />);
    for (const p of [
      "outfit-police",
      "outfit-firefighter",
      "outfit-paramedic",
      "outfit-coat",
      "hat-police",
      "hat-fire",
      "neckwear-stethoscope",
    ])
      expect(container.querySelector(`[data-part="${p}"]`), p).toBeNull();
  });
  it("uniforma nikad ne crta crveni križ", () => {
    // Zaštićen znak (Ženevske konvencije): nema sloja ni oznake koja ga imenuje.
    for (const u of UNIFORMS) for (const p of parts(u)) expect(p ?? "").not.toMatch(/cross|kriz/i);
  });
});

describe("galerija /likovi", () => {
  it("ima red „Službe“ s odorom policije, vatrogasaca, hitne i liječnika (muški i ženski)", () => {
    const { container } = render(<AvatarGallery />);
    expect(screen.getByText("Službe")).toBeTruthy();
    const count = (part: string) => container.querySelectorAll(`[data-part="${part}"]`).length;
    expect(count("outfit-police")).toBeGreaterThanOrEqual(2); // policajac i policajka
    expect(count("outfit-firefighter")).toBeGreaterThanOrEqual(1);
    expect(count("outfit-paramedic")).toBeGreaterThanOrEqual(1);
    expect(count("outfit-coat")).toBeGreaterThanOrEqual(2); // liječnik i liječnica
    expect(count("hat-police")).toBeGreaterThanOrEqual(2);
    expect(count("hat-fire")).toBeGreaterThanOrEqual(1);
    expect(count("neckwear-stethoscope")).toBeGreaterThanOrEqual(2);
  });
});
