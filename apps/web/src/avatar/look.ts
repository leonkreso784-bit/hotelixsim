// Izgled lika kao podatak (temelj za uređivač kao u BitLifeu, ADR-021/022). Lik se NIKAD ne crta mimo AvatarLook.
import { BACKDROP, CLOTHES, HAIR, SKIN } from "./palette";

export const HAIR_STYLES = [
  "none",
  "buzz",
  "short",
  "side",
  "curly",
  "long",
  "ponytail",
  "bun",
  "afro",
  "wavy",
  "bob",
  "slick",
] as const;
export const EYES = ["dot", "happy", "wide"] as const;
export const MOUTHS = ["smile", "neutral", "worried", "grin"] as const;
export const FACIAL_HAIR = ["none", "stubble", "mustache", "beard", "goatee", "full"] as const;
export const GLASSES = ["none", "round", "square"] as const;
export const HATS = ["none", "chef", "cap", "beanie"] as const;
export const BROWS = ["straight", "arched", "thick", "worried"] as const;
export const NOSES = ["button", "long", "wide"] as const;
export const AGES = ["young", "adult", "senior"] as const;
export const EARRINGS = ["none", "studs", "hoops"] as const;
export const NECKWEAR = ["none", "tie", "scarf", "bowtie"] as const;
export const OUTFITS = ["shirt", "blazer", "uniform", "apron", "tshirt"] as const;

export type HairStyle = (typeof HAIR_STYLES)[number];
export type Eyes = (typeof EYES)[number];
export type Mouth = (typeof MOUTHS)[number];
export type FacialHair = (typeof FACIAL_HAIR)[number];
export type Glasses = (typeof GLASSES)[number];
export type Hat = (typeof HATS)[number];
export type Brows = (typeof BROWS)[number];
export type Nose = (typeof NOSES)[number];
export type Age = (typeof AGES)[number];
export type Earrings = (typeof EARRINGS)[number];
export type Neckwear = (typeof NECKWEAR)[number];
export type Outfit = (typeof OUTFITS)[number];

export type AvatarLook = {
  skin: number;
  hair: HairStyle;
  hairColor: number;
  eyes: Eyes;
  mouth: Mouth;
  facialHair: FacialHair;
  glasses: Glasses;
  hat: Hat;
  clothes: number;
  backdrop: number;
  brows: Brows;
  nose: Nose;
  age: Age;
  earrings: Earrings;
  neckwear: Neckwear;
  outfit: Outfit;
};

/** Redoslijed polja za budući uređivač (kao u BitLifeu: koža, lice, kosa, dodaci, odjeća, pozadina). */
export type LookField =
  | { key: keyof AvatarLook; kind: "option"; values: readonly string[] }
  | { key: keyof AvatarLook; kind: "palette"; size: number };

export const LOOK_SCHEMA: readonly LookField[] = [
  { key: "skin", kind: "palette", size: SKIN.length },
  { key: "age", kind: "option", values: AGES },
  { key: "brows", kind: "option", values: BROWS },
  { key: "eyes", kind: "option", values: EYES },
  { key: "nose", kind: "option", values: NOSES },
  { key: "mouth", kind: "option", values: MOUTHS },
  { key: "hair", kind: "option", values: HAIR_STYLES },
  { key: "hairColor", kind: "palette", size: HAIR.length },
  { key: "facialHair", kind: "option", values: FACIAL_HAIR },
  { key: "glasses", kind: "option", values: GLASSES },
  { key: "earrings", kind: "option", values: EARRINGS },
  { key: "hat", kind: "option", values: HATS },
  { key: "outfit", kind: "option", values: OUTFITS },
  { key: "neckwear", kind: "option", values: NECKWEAR },
  { key: "clothes", kind: "palette", size: CLOTHES.length },
  { key: "backdrop", kind: "palette", size: BACKDROP.length },
];

const inRange = (i: number, n: number) => Number.isInteger(i) && i >= 0 && i < n;
const oneOf = <T extends string>(xs: readonly T[], v: string) => (xs as readonly string[]).includes(v);

export function isValidLook(l: AvatarLook): boolean {
  return (
    inRange(l.skin, SKIN.length) &&
    inRange(l.hairColor, HAIR.length) &&
    inRange(l.clothes, CLOTHES.length) &&
    inRange(l.backdrop, BACKDROP.length) &&
    oneOf(HAIR_STYLES, l.hair) &&
    oneOf(EYES, l.eyes) &&
    oneOf(MOUTHS, l.mouth) &&
    oneOf(FACIAL_HAIR, l.facialHair) &&
    oneOf(GLASSES, l.glasses) &&
    oneOf(HATS, l.hat) &&
    oneOf(BROWS, l.brows) &&
    oneOf(NOSES, l.nose) &&
    oneOf(AGES, l.age) &&
    oneOf(EARRINGS, l.earrings) &&
    oneOf(NECKWEAR, l.neckwear) &&
    oneOf(OUTFITS, l.outfit)
  );
}

/** Izgled po zadanim vrijednostima — popuna za sve što izvana nedostaje ili je neispravno. */
export const DEFAULT_LOOK: AvatarLook = {
  skin: 1,
  hair: "short",
  hairColor: 1,
  eyes: "dot",
  mouth: "smile",
  facialHair: "none",
  glasses: "none",
  hat: "none",
  clothes: 0,
  backdrop: 0,
  brows: "straight",
  nose: "button",
  age: "adult",
  earrings: "none",
  neckwear: "none",
  outfit: "shirt",
};

/** Izgled koji dolazi izvana (poslužitelj, uređivač): svako neispravno polje dobiva zadanu vrijednost, višak se odbacuje. */
export function normalizeLook(raw: unknown): AvatarLook {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return { ...DEFAULT_LOOK };
  const src = raw as Record<string, unknown>;
  const out: Record<string, unknown> = { ...DEFAULT_LOOK };
  for (const f of LOOK_SCHEMA) {
    const v = src[f.key];
    if (f.kind === "palette") {
      if (typeof v === "number" && inRange(v, f.size)) out[f.key] = v;
    } else if (typeof v === "string" && (f.values as readonly string[]).includes(v)) out[f.key] = v;
  }
  return out as AvatarLook;
}

/** FNV-1a (32 bita) pa xorshift: isto ime → isti niz brojeva, na svakom računalu. */
function rng(seed: string): (n: number) => number {
  let h = 0x811c9dc5;
  for (let i = 0; i < seed.length; i++) h = Math.imul(h ^ seed.charCodeAt(i), 0x01000193) >>> 0;
  return (n: number) => {
    h ^= h << 13;
    h ^= h >>> 17;
    h ^= h << 5;
    h >>>= 0;
    return h % n;
  };
}

function pick<T>(xs: readonly T[], r: (n: number) => number): T {
  return xs[r(xs.length)] as T;
}

// Zamrznuti bazeni za izgled iz imena (nakon ove jednokratne prilagodbe pravilo je strogo: samo dodavanje na kraj). Nove boje/opcije dodaju se u palete i nizove opcija, ali NE ovamo —
// tako svako ime zadržava isti izgled i kad se palete prošire. Redoslijed izvlačenja je fiksan.
const SEED_SKIN = [0, 1, 2, 3, 4, 5] as const;
const SEED_HAIR_STYLES = ["none", "buzz", "short", "side", "curly", "long", "ponytail", "bun"] as const;
const SEED_HAIR_COLORS = [0, 1, 2, 3, 4] as const; // bez sijede
const SEED_EYES = ["dot", "happy", "wide"] as const;
// Jednokratno (prije prvog spremanja): dobrodošao izraz — gosti su većinom ljubazni, bez „worried“.
const SEED_MOUTHS = ["smile", "grin", "smile", "neutral"] as const;
const SEED_FACIAL_HAIR = ["none", "stubble", "mustache", "beard"] as const;
const SEED_GLASSES = ["none", "round", "square"] as const;
const SEED_CLOTHES = [0, 1, 2, 3, 4, 5] as const; // bez bijele (kuhar)
const SEED_BACKDROPS = [0, 1, 2] as const;
// Nove osobine (izvlače se NAKON svih starih, pa stari izgledi ostaju isti).
const SEED_BROWS = ["straight", "arched", "thick"] as const; // bez „worried“ (to je Petra)
const SEED_NOSES = ["button", "long", "wide"] as const;
const SEED_AGES = ["young", "adult"] as const; // bez „senior“ — to je mentor
const SEED_EARRINGS = ["none", "studs", "hoops"] as const;
const SEED_NECKWEAR = ["none", "tie", "scarf", "bowtie"] as const;
const SEED_OUTFITS = ["shirt", "blazer", "tshirt"] as const; // bez pregače i uniforme (uloge osoblja)

export function lookFromSeed(seed: string): AvatarLook {
  const r = rng(seed);
  const skin = pick(SEED_SKIN, r);
  const hair = pick(SEED_HAIR_STYLES, r);
  const hairColor = pick(SEED_HAIR_COLORS, r);
  const eyes = pick(SEED_EYES, r);
  const mouth = pick(SEED_MOUTHS, r);
  // Brada rjeđe od ostalog, kapa vrlo rijetko — „slučajni“ gosti trebaju izgledati obično.
  const facialHair = r(3) === 0 ? pick(SEED_FACIAL_HAIR, r) : "none";
  const glasses = r(4) === 0 ? pick(SEED_GLASSES, r) : "none";
  const hat = r(12) === 0 ? "cap" : "none";
  const clothes = pick(SEED_CLOTHES, r);
  const backdrop = pick(SEED_BACKDROPS, r);
  const brows = pick(SEED_BROWS, r);
  const nose = pick(SEED_NOSES, r);
  const age = pick(SEED_AGES, r);
  // Nakit i vezovi rjeđe — slučajni gosti trebaju izgledati obično.
  const earrings = r(4) === 0 ? pick(SEED_EARRINGS, r) : "none";
  const neckwear = r(5) === 0 ? pick(SEED_NECKWEAR, r) : "none";
  const outfit = pick(SEED_OUTFITS, r);
  return {
    skin,
    hair,
    hairColor,
    eyes,
    mouth,
    facialHair,
    glasses,
    hat,
    clothes,
    backdrop,
    brows,
    nose,
    age,
    earrings,
    neckwear,
    outfit,
  };
}

/** Ti — upravitelj hotela. Uređivač (poslije demoa) mijenja upravo ovo. */
export const PLAYER_LOOK: AvatarLook = {
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
};

export const MENTOR_LOOK: AvatarLook = {
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
};

/** Likovi iz sadržaja (content/hotel/situations.json). Poslije demoa ovo seli u sadržaj kao `character.look`. */
export const KNOWN_LOOKS: Record<string, AvatarLook> = {
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
  // Roboti-upravitelji: ključ je ime hotela iz igre (crates/game/src/command.rs).
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

export function lookFor(name: string): AvatarLook {
  return Object.hasOwn(KNOWN_LOOKS, name) ? (KNOWN_LOOKS[name] as AvatarLook) : lookFromSeed(name);
}
