import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { AvatarGallery } from "../screens/AvatarGallery";
import { Avatar } from "./Avatar";
import {
  AGES,
  type Age,
  type AvatarLook,
  BROWS,
  DEFAULT_LOOK,
  EARRINGS,
  EYES,
  FACIAL_HAIR,
  GLASSES,
  HAIR_STYLES,
  HATS,
  HINT_HAIR_F,
  HINT_HAIR_M,
  HINT_SKIN_FOREIGN,
  HINT_SKIN_REGIONAL,
  isValidLook,
  KNOWN_LOOKS,
  LOOK_SCHEMA,
  type LookHint,
  lookFor,
  lookFromHint,
  lookFromSeed,
  MENTOR_LOOK,
  MOUTHS,
  NECKWEAR,
  NOSES,
  normalizeLook,
  OUTFITS,
  PLAYER_LOOK,
} from "./look";

afterEach(cleanup);

describe("izgled lika", () => {
  it("isto ime uvijek daje isti izgled", () => {
    expect(lookFromSeed("Ana Kovač")).toEqual(lookFromSeed("Ana Kovač"));
  });
  it("izgled iz imena je zamrznut: promjena palete ga ne smije pomaknuti", () => {
    expect(lookFromSeed("Ana Kovač")).toEqual({
      skin: 1,
      hair: "buzz",
      hairColor: 2,
      eyes: "happy",
      mouth: "smile",
      facialHair: "none",
      glasses: "none",
      hat: "none",
      clothes: 2,
      backdrop: 2,
      brows: "arched",
      nose: "long",
      age: "young",
      earrings: "none",
      neckwear: "none",
      outfit: "tshirt",
    });
  });
  it("ime koje je svojstvo Objecta pada na izgled iz imena", () => {
    expect(lookFor("constructor")).toEqual(lookFromSeed("constructor"));
  });
  it("izgled iz imena je uvijek valjan", () => {
    for (let i = 0; i < 300; i++) expect(isValidLook(lookFromSeed(`lik-${i}`))).toBe(true);
  });
  it("različita imena daju raznolike likove", () => {
    const names = ["Ana", "Ivan", "Maja", "Luka", "Petar", "Iva", "Tomislav", "Lana", "Karlo", "Ema"];
    const distinct = new Set(names.map((n) => JSON.stringify(lookFromSeed(n))));
    expect(distinct.size).toBeGreaterThanOrEqual(8);
    const hairs = new Set(names.map((n) => lookFromSeed(n).hair));
    expect(hairs.size).toBeGreaterThanOrEqual(4);
  });
  it("poznati likovi imaju ručno izabran izgled, nepoznati izgled iz imena", () => {
    expect(lookFor("Petra Horvat")).toEqual(KNOWN_LOOKS["Petra Horvat"]);
    expect(lookFor("Marko Babić").hat).toBe("chef");
    expect(lookFor("Netko Novi")).toEqual(lookFromSeed("Netko Novi"));
  });
  it("roboti-upravitelji imaju lik pod imenom hotela iz igre", () => {
    expect(lookFor("Štediša")).toEqual(KNOWN_LOOKS.Štediša);
    expect(lookFor("Štediša").glasses).toBe("square");
    expect(lookFor("Brižni").earrings).toBe("hoops");
    expect(lookFor("Lovac na popunjenost").hat).toBe("cap");
  });
  it("igrač, mentor i poznati likovi su valjani", () => {
    for (const l of [PLAYER_LOOK, MENTOR_LOOK, ...Object.values(KNOWN_LOOKS)])
      expect(isValidLook(l)).toBe(true);
  });
  it("isValidLook odbija indeks izvan palete i nepoznatu opciju", () => {
    expect(isValidLook({ ...PLAYER_LOOK, skin: 99 })).toBe(false);
    expect(isValidLook({ ...PLAYER_LOOK, hair: "mohawk" as never })).toBe(false);
    expect(isValidLook({ ...PLAYER_LOOK, nose: "pig" as never })).toBe(false);
    expect(isValidLook({ ...PLAYER_LOOK, outfit: "toga" as never })).toBe(false);
  });
  it("LOOK_SCHEMA sadrži svaki ključ izgleda točno jednom", () => {
    const keys = LOOK_SCHEMA.map((f) => f.key).sort();
    expect(keys).toEqual(Object.keys(PLAYER_LOOK).sort());
    expect(new Set(keys).size).toBe(keys.length);
  });
  it("gosti iz imena su ljubazni: nikad zabrinuta usta ni obrve", () => {
    for (let i = 0; i < 300; i++) {
      const l = lookFromSeed(`lik-${i}`);
      expect(l.mouth).not.toBe("worried");
      expect(l.brows).not.toBe("worried");
    }
  });
  it("novi izgledi iz imena nikad ne koriste starost „senior“, pregaču ni uniformu", () => {
    for (let i = 0; i < 300; i++) {
      const l = lookFromSeed(`lik-${i}`);
      expect(l.age).not.toBe("senior");
      expect(["apron", "uniform"]).not.toContain(l.outfit);
    }
  });
});

describe("normalizeLook", () => {
  it("DEFAULT_LOOK je valjan", () => {
    expect(isValidLook(DEFAULT_LOOK)).toBe(true);
  });
  it("izgled starog oblika zadržava stara polja, nova dobiva iz DEFAULT_LOOK", () => {
    const { brows, nose, age, earrings, neckwear, outfit, ...old } = PLAYER_LOOK;
    void [brows, nose, age, earrings, neckwear, outfit];
    expect(normalizeLook(old)).toEqual({
      ...old,
      brows: DEFAULT_LOOK.brows,
      nose: DEFAULT_LOOK.nose,
      age: DEFAULT_LOOK.age,
      earrings: DEFAULT_LOOK.earrings,
      neckwear: DEFAULT_LOOK.neckwear,
      outfit: DEFAULT_LOOK.outfit,
    });
  });
  it("neispravno polje dobiva vrijednost iz DEFAULT_LOOK", () => {
    const n = normalizeLook({ skin: 999, hair: "x" });
    expect(n.skin).toBe(DEFAULT_LOOK.skin);
    expect(n.hair).toBe(DEFAULT_LOOK.hair);
  });
  it("ne-objekt daje DEFAULT_LOOK", () => {
    for (const raw of [null, 42, [], "lik", undefined]) expect(normalizeLook(raw)).toEqual(DEFAULT_LOOK);
  });
  it("višak polja se odbacuje", () => {
    const n = normalizeLook({ ...PLAYER_LOOK, evil: 1 });
    expect(Object.hasOwn(n, "evil")).toBe(false);
    expect(n).toEqual(PLAYER_LOOK);
  });
});

describe("Avatar", () => {
  it("crta slojeve koje izgled traži, i samo njih", () => {
    const { container } = render(
      <Avatar look={{ ...PLAYER_LOOK, hair: "bun", glasses: "round", hat: "none" }} />,
    );
    expect(container.querySelector('[data-part="hair-bun"]')).not.toBeNull();
    expect(container.querySelector('[data-part="glasses-round"]')).not.toBeNull();
    expect(container.querySelector('[data-part^="hat-"]')).toBeNull();
  });
  it("svaka frizura ima svoj sloj (osim „bez kose“)", () => {
    for (const hair of HAIR_STYLES) {
      const { container, unmount } = render(<Avatar look={{ ...PLAYER_LOOK, hair }} />);
      const part = container.querySelector(`[data-part="hair-${hair}"]`);
      expect(hair === "none" ? part === null : part !== null).toBe(true);
      unmount();
    }
  });
  it("svaka opcija svake osobine ima svoj sloj (osim praznih)", () => {
    const traits: [keyof AvatarLook, readonly string[]][] = [
      ["eyes", EYES],
      ["mouth", MOUTHS],
      ["facialHair", FACIAL_HAIR],
      ["glasses", GLASSES],
      ["hat", HATS],
      ["brows", BROWS],
      ["nose", NOSES],
      ["age", AGES],
      ["earrings", EARRINGS],
      ["neckwear", NECKWEAR],
      ["outfit", OUTFITS],
    ];
    const part = (key: string) => (key === "facialHair" ? "facial" : key);
    for (const [key, options] of traits) {
      for (const option of options) {
        const { container, unmount } = render(<Avatar look={{ ...PLAYER_LOOK, [key]: option }} />);
        const found = container.querySelector(`[data-part="${part(key)}-${option}"]`);
        const empty = option === "none" || (key === "age" && option !== "senior");
        expect(empty ? found === null : found !== null).toBe(true);
        unmount();
      }
    }
  });
  it("bez naslova je skriven čitaču, s naslovom je slika s imenom", () => {
    const { container } = render(<Avatar look={PLAYER_LOOK} />);
    expect(container.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
    render(<Avatar look={MENTOR_LOOK} title="Mentor" />);
    expect(screen.getByRole("img", { name: "Mentor" })).toBeTruthy();
  });
  it("dva lika na istom ekranu ne dijele id izreza", () => {
    const { container } = render(
      <>
        <Avatar look={PLAYER_LOOK} />
        <Avatar look={MENTOR_LOOK} />
      </>,
    );
    const ids = [...container.querySelectorAll("clipPath")].map((c) => c.id);
    expect(new Set(ids).size).toBe(2);
  });
});

// Zamrznut snimak ručnih izgleda prije IZGLED-b (lookFor(ime, hint) ih ne smije dirati).
const KNOWN_LOOKS_SNAPSHOT = {
  "Petra Horvat": {
    skin: 1,
    hair: "long",
    hairColor: 2,
    eyes: "wide",
    mouth: "worried",
    facialHair: "none",
    glasses: "none",
    hat: "none",
    clothes: 2,
    backdrop: 0,
    brows: "worried",
    nose: "button",
    age: "adult",
    earrings: "studs",
    neckwear: "none",
    outfit: "tshirt",
  },
  "Marko Babić": {
    skin: 2,
    hair: "short",
    hairColor: 0,
    eyes: "dot",
    mouth: "neutral",
    facialHair: "mustache",
    glasses: "none",
    hat: "chef",
    clothes: 6,
    backdrop: 1,
    brows: "thick",
    nose: "wide",
    age: "adult",
    earrings: "none",
    neckwear: "none",
    outfit: "apron",
  },
  Štediša: {
    skin: 0,
    hair: "slick",
    hairColor: 5,
    eyes: "dot",
    mouth: "neutral",
    facialHair: "none",
    glasses: "square",
    hat: "none",
    clothes: 4,
    backdrop: 1,
    brows: "straight",
    nose: "long",
    age: "senior",
    earrings: "none",
    neckwear: "tie",
    outfit: "blazer",
  },
  Brižni: {
    skin: 2,
    hair: "wavy",
    hairColor: 1,
    eyes: "happy",
    mouth: "smile",
    facialHair: "none",
    glasses: "none",
    hat: "none",
    clothes: 1,
    backdrop: 0,
    brows: "arched",
    nose: "button",
    age: "adult",
    earrings: "hoops",
    neckwear: "none",
    outfit: "shirt",
  },
  "Lovac na popunjenost": {
    skin: 3,
    hair: "short",
    hairColor: 0,
    eyes: "wide",
    mouth: "grin",
    facialHair: "stubble",
    glasses: "none",
    hat: "cap",
    clothes: 3,
    backdrop: 2,
    brows: "thick",
    nose: "wide",
    age: "young",
    earrings: "none",
    neckwear: "none",
    outfit: "tshirt",
  },
};

const REGIONAL_SKIN = [0, 1, 6];
const FOREIGN_SKIN = [0, 1, 2, 6];
const HINT_AGES: (Age | null)[] = ["young", "adult", "senior", null];
const HINT_ORIGINS = ["regional", "foreign"] as const;
const HINT_NAMES = Array.from({ length: 300 }, (_, i) => `lik-${i}`);
// GRADITELJ: upiši vrijednost iz prvog zelenog prolaza (lookFor("Anna Müller", foreign/f/null)); od tada je pravilo kao za SEED_*.
const ANNA_MULLER_SNAPSHOT: AvatarLook | null = null;

function allHints(gender: "f" | "m"): LookHint[] {
  return HINT_AGES.flatMap((age_group) => HINT_ORIGINS.map((origin) => ({ gender, age_group, origin })));
}

describe("izgled iz podataka lika", () => {
  it("bez hinta isto kao prije", () => {
    for (const ime of HINT_NAMES) {
      expect(lookFor(ime)).toEqual(lookFromSeed(ime));
      expect(lookFor(ime, null)).toEqual(lookFromSeed(ime));
    }
  });

  it("poznati likovi pobjeđuju hint, a ručni izgledi su nepromijenjeni", () => {
    const hint: LookHint = { gender: "m", age_group: "senior", origin: "foreign" };
    expect(lookFor("Petra Horvat", hint)).toEqual(KNOWN_LOOKS["Petra Horvat"]);
    expect(lookFor("Marko Babić", { ...hint, gender: "f" })).toEqual(KNOWN_LOOKS["Marko Babić"]);
    expect(KNOWN_LOOKS).toEqual(KNOWN_LOOKS_SNAPSHOT);
    expect(PLAYER_LOOK).toEqual({
      skin: 1,
      hair: "side",
      hairColor: 1,
      eyes: "happy",
      mouth: "smile",
      facialHair: "none",
      glasses: "none",
      hat: "none",
      clothes: 0,
      backdrop: 2,
      brows: "straight",
      nose: "button",
      age: "adult",
      earrings: "none",
      neckwear: "tie",
      outfit: "blazer",
    });
    expect(MENTOR_LOOK).toEqual({
      skin: 0,
      hair: "bun",
      hairColor: 5,
      eyes: "dot",
      mouth: "smile",
      facialHair: "none",
      glasses: "round",
      hat: "none",
      clothes: 5,
      backdrop: 1,
      brows: "arched",
      nose: "long",
      age: "senior",
      earrings: "studs",
      neckwear: "scarf",
      outfit: "blazer",
    });
  });

  it("bazeni su zamrznuti kako kartica kaže", () => {
    expect(HINT_SKIN_REGIONAL).toEqual(REGIONAL_SKIN);
    expect(HINT_SKIN_FOREIGN).toEqual(FOREIGN_SKIN);
    expect(HINT_HAIR_F).toEqual(["long", "ponytail", "bun", "bob", "wavy", "curly", "side", "short"]);
    expect(HINT_HAIR_M).toEqual(["short", "buzz", "side", "slick", "curly", "wavy", "none"]);
  });

  it("žensko ime nikad nema bradu, a frizura je iz ženskog bazena", () => {
    for (const hint of allHints("f"))
      for (const ime of HINT_NAMES) {
        const l = lookFromHint(ime, hint);
        expect(l.facialHair).toBe("none");
        expect(HINT_HAIR_F as readonly string[]).toContain(l.hair);
      }
  });

  it("muško ime nikad nema naušnice ni žensku frizuru, a brada se ipak pojavljuje", () => {
    let brade = 0;
    for (const hint of allHints("m"))
      for (const ime of HINT_NAMES) {
        const l = lookFromHint(ime, hint);
        expect(l.earrings).toBe("none");
        expect(HINT_HAIR_M as readonly string[]).toContain(l.hair);
        if (l.facialHair !== "none") brade++;
      }
    expect(brade).toBeGreaterThan(0);
  });

  it("dob iz hinta: zadana dob vrijedi, senior ima sijedu kosu, null bira mlad ili odrasla", () => {
    for (const gender of ["f", "m"] as const)
      for (const origin of HINT_ORIGINS)
        for (const ime of HINT_NAMES) {
          for (const age_group of ["young", "adult", "senior"] as const) {
            const l = lookFromHint(ime, { gender, age_group, origin });
            expect(l.age).toBe(age_group);
            if (age_group === "senior") expect(l.hairColor).toBe(5);
          }
          expect(["young", "adult"]).toContain(lookFromHint(ime, { gender, age_group: null, origin }).age);
        }
  });

  it("podrijetlo bira raspon kože: nikad 3, 4, 5 ni 7", () => {
    for (const gender of ["f", "m"] as const)
      for (const age_group of HINT_AGES)
        for (const ime of HINT_NAMES) {
          expect(REGIONAL_SKIN).toContain(lookFromHint(ime, { gender, age_group, origin: "regional" }).skin);
          expect(FOREIGN_SKIN).toContain(lookFromHint(ime, { gender, age_group, origin: "foreign" }).skin);
        }
  });

  it("isti ulaz uvijek isti izgled, uvijek valjan, a lookFor s hintom je lookFromHint", () => {
    for (const hint of [...allHints("f"), ...allHints("m")])
      for (const ime of HINT_NAMES) {
        const l = lookFromHint(ime, hint);
        expect(lookFromHint(ime, hint)).toEqual(l);
        expect(lookFor(ime, hint)).toEqual(l);
        expect(isValidLook(l)).toBe(true);
      }
  });

  it("deset ženskih imena daje barem četiri različite frizure", () => {
    const hint: LookHint = { gender: "f", age_group: null, origin: "regional" };
    const frizure = new Set(HINT_NAMES.slice(0, 10).map((ime) => lookFromHint(ime, hint).hair));
    expect(frizure.size).toBeGreaterThanOrEqual(4);
  });

  it("zamrznut izgled iz hinta za Anna Müller", () => {
    const l = lookFor("Anna Müller", { gender: "f", age_group: null, origin: "foreign" });
    expect(ANNA_MULLER_SNAPSHOT, "GRADITELJ: upiši zamrznuti izgled u ANNA_MULLER_SNAPSHOT").not.toBeNull();
    expect(l).toEqual(ANNA_MULLER_SNAPSHOT);
  });
});

describe("AvatarGallery", () => {
  it("pokazuje poznate likove i sve osobine", () => {
    const { container } = render(<AvatarGallery />);
    expect(screen.getByText("Petra Horvat")).toBeTruthy();
    expect(screen.getByText("Marko Babić")).toBeTruthy();
    expect(container.querySelectorAll("svg").length).toBeGreaterThanOrEqual(
      2 + 2 + 9 + 12 + 4 + 3 + 3 + 3 + 4 + 5,
    );
  });
});
