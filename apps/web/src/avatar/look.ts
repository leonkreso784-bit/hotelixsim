// Izgled lika kao podatak (temelj za uređivač kao u BitLifeu, ADR-021/022). Lik se NIKAD ne crta mimo AvatarLook.
import { BACKDROP, CLOTHES, HAIR, SKIN } from "./palette";

export const HAIR_STYLES = ["none", "buzz", "short", "side", "curly", "long", "ponytail", "bun"] as const;
export const EYES = ["dot", "happy", "wide"] as const;
export const MOUTHS = ["smile", "neutral", "worried", "grin"] as const;
export const FACIAL_HAIR = ["none", "stubble", "mustache", "beard"] as const;
export const GLASSES = ["none", "round", "square"] as const;
export const HATS = ["none", "chef", "cap"] as const;

export type HairStyle = (typeof HAIR_STYLES)[number];
export type Eyes = (typeof EYES)[number];
export type Mouth = (typeof MOUTHS)[number];
export type FacialHair = (typeof FACIAL_HAIR)[number];
export type Glasses = (typeof GLASSES)[number];
export type Hat = (typeof HATS)[number];

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
};

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
    oneOf(HATS, l.hat)
  );
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

// Zamrznuti bazeni za izgled iz imena. Nove boje/opcije dodaju se u palete i nizove opcija, ali NE ovamo —
// tako svako ime zadržava isti izgled i kad se palete prošire. Redoslijed izvlačenja je fiksan.
const SEED_SKIN = [0, 1, 2, 3, 4, 5] as const;
const SEED_HAIR_STYLES = ["none", "buzz", "short", "side", "curly", "long", "ponytail", "bun"] as const;
const SEED_HAIR_COLORS = [0, 1, 2, 3, 4] as const; // bez sijede
const SEED_EYES = ["dot", "happy", "wide"] as const;
const SEED_MOUTHS = ["smile", "neutral", "worried", "grin"] as const;
const SEED_FACIAL_HAIR = ["none", "stubble", "mustache", "beard"] as const;
const SEED_GLASSES = ["none", "round", "square"] as const;
const SEED_CLOTHES = [0, 1, 2, 3, 4, 5] as const; // bez bijele (kuhar)
const SEED_BACKDROPS = [0, 1, 2] as const;

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
  return { skin, hair, hairColor, eyes, mouth, facialHair, glasses, hat, clothes, backdrop };
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
  },
};

export function lookFor(name: string): AvatarLook {
  return Object.hasOwn(KNOWN_LOOKS, name) ? (KNOWN_LOOKS[name] as AvatarLook) : lookFromSeed(name);
}
