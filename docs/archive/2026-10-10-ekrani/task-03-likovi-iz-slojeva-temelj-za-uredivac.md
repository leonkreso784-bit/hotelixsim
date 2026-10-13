### Task 3: Likovi iz slojeva (temelj za uređivač kao u BitLifeu)

Leon (10.10.): *„trebat cemo napravit razlicite likove u igrici izgledno i to na nacin da cemo kasnije moci ih customizat
isto kao sto bit life ima"* → anketa: **temelj sad, uređivač poslije demoa** ([ADR-021](../../records/DECISIONS/ADR-021-koncept-ekrana-v2-i-odluke-o.md)).
Lik se crta iz SVG slojeva (pozadina, odjeća, vrat, kosa straga, uši, lice, nos, oči, obrve, usta, brada, naočale,
kosa sprijeda, kapa). **Izgled je podatak** (`AvatarLook`): danas ga daje tablica poznatih likova ili deterministički
„slučajni“ izgled iz imena; poslije demoa isti podatak dolazi iz sadržaja (`character.look`) i profila igrača, a uređivač
samo mijenja polja. Zato ništa u ovom tasku ne smije crtati lik mimo `AvatarLook`.

**Files:**
- Create: `apps/web/src/avatar/palette.ts`, `apps/web/src/avatar/look.ts`, `apps/web/src/avatar/Avatar.tsx`,
  `apps/web/src/avatar/avatar.test.tsx`, `apps/web/src/screens/AvatarGallery.tsx`
- Modify: `packages/i18n/src/hr.ts`

**Interfaces:**
- Produces (`avatar/look.ts`): `HAIR_STYLES, EYES, MOUTHS, FACIAL_HAIR, GLASSES, HATS` (readonly nizovi opcija);
  `type AvatarLook = { skin: number; hair: HairStyle; hairColor: number; eyes: Eyes; mouth: Mouth; facialHair: FacialHair;
  glasses: Glasses; hat: Hat; clothes: number; backdrop: number }` (brojevi su indeksi u nizove iz `palette.ts`);
  `lookFromSeed(seed: string): AvatarLook`; `lookFor(name: string): AvatarLook`; `isValidLook(l: AvatarLook): boolean`;
  `PLAYER_LOOK`, `MENTOR_LOOK`, `KNOWN_LOOKS: Record<string, AvatarLook>`.
- Produces (`avatar/palette.ts`): `SKIN, HAIR, CLOTHES, BACKDROP` (readonly nizovi heks boja).
- Produces (`avatar/Avatar.tsx`): `Avatar({ look: AvatarLook; size?: number; title?: string })` — bez `title` je
  `aria-hidden` (ime stoji kao tekst uz lik), s `title` je `role="img"` s tim imenom. Svaki sloj ima `data-part="<sloj>-<opcija>"`.
- Produces (`screens/AvatarGallery.tsx`): `AvatarGallery()` — stranica `/likovi` (rutu dodaje Task 4).

- [ ] **Step 1: `avatar/palette.ts`**

```ts
// Boje LIKA su podatak (kao u BitLifeu: koža, kosa, odjeća), ne tema sučelja — zato su ovdje, a ne u styles.css.
// Indeksi u ove nizove spremaju se u AvatarLook; novi element ide NA KRAJ niza (stari izgledi se ne mijenjaju).
export const SKIN = ["#FCE3D3", "#F5CBA7", "#E0AC69", "#C68642", "#8D5524", "#5C3A21"] as const;
/** crna, tamnosmeđa, smeđa, plava, riđa, sijeda */
export const HAIR = ["#1F1A17", "#4A2F1D", "#8B5A2B", "#D9A441", "#B5552B", "#A7ADB4"] as const;
/** plava, zelena, oker, ljubičasta, crvena, tamna, bijela (kuhar) */
export const CLOTHES = ["#246BFD", "#15924A", "#E0A82E", "#7C4DFF", "#DC2F2F", "#2C3E50", "#F7F7F7"] as const;
/** pijesak, menta, svijetloplava — boje koncepta v2 */
export const BACKDROP = ["#F4E8D5", "#D8F0EE", "#EAF1FF"] as const;
```

- [ ] **Step 2: Napiši pad testova `avatar/avatar.test.tsx`**

```tsx
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Avatar } from "./Avatar";
import { HAIR_STYLES, isValidLook, KNOWN_LOOKS, lookFor, lookFromSeed, MENTOR_LOOK, PLAYER_LOOK } from "./look";

afterEach(cleanup);

describe("izgled lika", () => {
  it("isto ime uvijek daje isti izgled", () => {
    expect(lookFromSeed("Ana Kovač")).toEqual(lookFromSeed("Ana Kovač"));
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
  it("igrač, mentor i poznati likovi su valjani", () => {
    for (const l of [PLAYER_LOOK, MENTOR_LOOK, ...Object.values(KNOWN_LOOKS)]) expect(isValidLook(l)).toBe(true);
  });
  it("isValidLook odbija indeks izvan palete i nepoznatu opciju", () => {
    expect(isValidLook({ ...PLAYER_LOOK, skin: 99 })).toBe(false);
    expect(isValidLook({ ...PLAYER_LOOK, hair: "mohawk" as never })).toBe(false);
  });
});

describe("Avatar", () => {
  it("crta slojeve koje izgled traži, i samo njih", () => {
    const { container } = render(<Avatar look={{ ...PLAYER_LOOK, hair: "bun", glasses: "round", hat: "none" }} />);
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
```

- [ ] **Step 3: Vidi pad** — `cd /c/Users/leonk/Documents/hotelixsim && corepack pnpm --filter @hotelixsim/web exec vitest run src/avatar; echo EXIT=$?` → FAIL.

- [ ] **Step 4: `avatar/look.ts`**

```ts
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

export function lookFromSeed(seed: string): AvatarLook {
  const r = rng(seed);
  const hair = pick(HAIR_STYLES, r);
  // Brada rjeđe od ostalog, kapa vrlo rijetko — „slučajni“ gosti trebaju izgledati obično.
  const facialHair = r(3) === 0 ? pick(FACIAL_HAIR, r) : "none";
  return {
    skin: r(SKIN.length),
    hair,
    hairColor: r(HAIR.length - 1), // sijeda (zadnja) samo ručno
    eyes: pick(EYES, r),
    mouth: pick(MOUTHS, r),
    facialHair,
    glasses: r(4) === 0 ? pick(GLASSES, r) : "none",
    hat: r(12) === 0 ? "cap" : "none",
    clothes: r(CLOTHES.length - 1), // bijela (kuhar) samo ručno
    backdrop: r(BACKDROP.length),
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
  return KNOWN_LOOKS[name] ?? lookFromSeed(name);
}
```

- [ ] **Step 5: `avatar/Avatar.tsx`** (viewBox 64×64; glava je elipsa `cx=32 cy=27 rx=13 ry=15`, oči na `y≈28`,
  usta na `y≈35`; sve unutar kruga-izreza)

```tsx
import { useId } from "react";
import type { AvatarLook } from "./look";
import { BACKDROP, CLOTHES, HAIR, SKIN } from "./palette";

const at = <T,>(xs: readonly T[], i: number): T => xs[Math.min(Math.max(i, 0), xs.length - 1)] as T;
const LINE = "#3B2A22";

function HairBack({ look, color }: { look: AvatarLook; color: string }) {
  if (look.hair === "long")
    return <path data-part="hairback-long" d="M17 26 C17 11 47 11 47 26 L49 50 C43 54 21 54 15 50 Z" fill={color} />;
  if (look.hair === "ponytail")
    return <path data-part="hairback-ponytail" d="M44 21 C55 23 55 41 47 47 C50 37 48 30 43 27 Z" fill={color} />;
  return null;
}

function HairFront({ look, color }: { look: AvatarLook; color: string }) {
  switch (look.hair) {
    case "none":
      return null;
    case "buzz":
      return <path data-part="hair-buzz" d="M19.5 23 C20 11 44 11 44.5 23 C40 18 24 18 19.5 23 Z" fill={color} opacity={0.8} />;
    case "short":
      return <path data-part="hair-short" d="M18.5 25 C16.5 10 47.5 9 45.5 25 C43 19 38 17 32 17 C26 17 21 19 18.5 25 Z" fill={color} />;
    case "side":
      return <path data-part="hair-side" d="M18.5 26 C15.5 9 48.5 7 45.5 24 C41 15 31 22 22.5 19.5 C20.5 21.5 19.3 23.5 18.5 26 Z" fill={color} />;
    case "curly":
      return (
        <g data-part="hair-curly" fill={color}>
          {[
            [20, 21],
            [25, 15],
            [32, 13],
            [39, 15],
            [44, 21],
            [21, 27],
            [43, 27],
          ].map(([x, y]) => (
            <circle key={`${x}-${y}`} cx={x} cy={y} r={5.2} />
          ))}
        </g>
      );
    case "long":
      return <path data-part="hair-long" d="M18 31 C14.5 9 49.5 9 46 31 C44 20 38 16 32 16 C26 16 20 20 18 31 Z" fill={color} />;
    case "ponytail":
      return <path data-part="hair-ponytail" d="M18.5 25 C16.5 10 47.5 10 45.5 25 C42 18 22 18 18.5 25 Z" fill={color} />;
    case "bun":
      return (
        <g data-part="hair-bun" fill={color}>
          <circle cx={32} cy={9.5} r={6} />
          <path d="M18.5 25 C16.5 11 47.5 11 45.5 25 C42 18 22 18 18.5 25 Z" />
        </g>
      );
  }
}

function Eyes({ look }: { look: AvatarLook }) {
  if (look.eyes === "happy")
    return (
      <g data-part="eyes-happy" stroke={LINE} strokeWidth={1.6} fill="none" strokeLinecap="round">
        <path d="M24.8 28.6 Q27 25.8 29.2 28.6" />
        <path d="M34.8 28.6 Q37 25.8 39.2 28.6" />
      </g>
    );
  if (look.eyes === "wide")
    return (
      <g data-part="eyes-wide">
        <ellipse cx={27} cy={28} rx={2.6} ry={3} fill="#fff" />
        <ellipse cx={37} cy={28} rx={2.6} ry={3} fill="#fff" />
        <circle cx={27.3} cy={28.4} r={1.4} fill={LINE} />
        <circle cx={37.3} cy={28.4} r={1.4} fill={LINE} />
      </g>
    );
  return (
    <g data-part="eyes-dot" fill={LINE}>
      <circle cx={27} cy={28} r={1.7} />
      <circle cx={37} cy={28} r={1.7} />
    </g>
  );
}

function Mouth({ look }: { look: AvatarLook }) {
  const common = { stroke: LINE, strokeWidth: 1.6, fill: "none", strokeLinecap: "round" as const };
  switch (look.mouth) {
    case "smile":
      return <path data-part="mouth-smile" d="M27.5 34.5 Q32 38.5 36.5 34.5" {...common} />;
    case "neutral":
      return <path data-part="mouth-neutral" d="M28.5 35.5 L35.5 35.5" {...common} />;
    case "worried":
      return <path data-part="mouth-worried" d="M28 37 Q32 33.8 36 37" {...common} />;
    case "grin":
      return (
        <g data-part="mouth-grin">
          <path d="M27 34 Q32 40.5 37 34 Z" fill={LINE} />
          <path d="M28.3 34.6 L35.7 34.6 L35 35.9 L29 35.9 Z" fill="#fff" />
        </g>
      );
  }
}

function FacialHair({ look, color }: { look: AvatarLook; color: string }) {
  switch (look.facialHair) {
    case "none":
      return null;
    case "stubble":
      return <path data-part="facial-stubble" d="M20 31 C22 44 42 44 44 31 C40 38 24 38 20 31 Z" fill={color} opacity={0.3} />;
    case "mustache":
      return <path data-part="facial-mustache" d="M27.2 33.6 Q32 30.6 36.8 33.6 Q32 34.8 27.2 33.6 Z" fill={color} />;
    case "beard":
      return <path data-part="facial-beard" d="M19.5 30 C20 46.5 44 46.5 44.5 30 C42 38.5 38 39.5 32 39.5 C26 39.5 22 38.5 19.5 30 Z" fill={color} />;
  }
}

function Glasses({ look }: { look: AvatarLook }) {
  const frame = { stroke: "#151B2B", strokeWidth: 1.4, fill: "rgba(255,255,255,0.18)" };
  if (look.glasses === "round")
    return (
      <g data-part="glasses-round">
        <circle cx={27} cy={28} r={4.3} {...frame} />
        <circle cx={37} cy={28} r={4.3} {...frame} />
        <path d="M31.3 28 L32.7 28" stroke="#151B2B" strokeWidth={1.4} />
      </g>
    );
  if (look.glasses === "square")
    return (
      <g data-part="glasses-square">
        <rect x={22.4} y={24.6} width={9} height={7} rx={1.6} {...frame} />
        <rect x={32.6} y={24.6} width={9} height={7} rx={1.6} {...frame} />
        <path d="M31.4 27.5 L32.6 27.5" stroke="#151B2B" strokeWidth={1.4} />
      </g>
    );
  return null;
}

function HatLayer({ look, clothes }: { look: AvatarLook; clothes: string }) {
  if (look.hat === "chef")
    return (
      <g data-part="hat-chef" fill="#fff" stroke="#E2E8F3" strokeWidth={0.8}>
        <circle cx={24} cy={8.5} r={5.5} />
        <circle cx={40} cy={8.5} r={5.5} />
        <circle cx={32} cy={6} r={6.5} />
        <rect x={21} y={10} width={22} height={8.5} rx={2} />
      </g>
    );
  if (look.hat === "cap")
    return (
      <g data-part="hat-cap" fill={clothes}>
        <path d="M18.5 22.5 C18 8.5 46 8.5 45.5 22.5 Z" />
        <path d="M32 20.5 L55 22 C53 25.5 40 24.5 32 23.2 Z" />
      </g>
    );
  return null;
}

/** Lik iz slojeva. Bez `title` je ukras (ime stoji uz njega), s `title` je slika s imenom. */
export function Avatar({ look, size = 44, title }: { look: AvatarLook; size?: number; title?: string }) {
  const clip = `lik-${useId().replace(/[^a-zA-Z0-9_-]/g, "")}`;
  const skin = at(SKIN, look.skin);
  const hair = at(HAIR, look.hairColor);
  const clothes = at(CLOTHES, look.clothes);
  const a11y = title ? { role: "img", "aria-label": title } : { "aria-hidden": true as const };
  return (
    <svg viewBox="0 0 64 64" width={size} height={size} className="shrink-0" {...a11y}>
      <defs>
        <clipPath id={clip}>
          <circle cx={32} cy={32} r={32} />
        </clipPath>
      </defs>
      <g clipPath={`url(#${clip})`}>
        <rect data-part="backdrop" width={64} height={64} fill={at(BACKDROP, look.backdrop)} />
        <HairBack look={look} color={hair} />
        <path data-part="clothes" d="M6 64 C7 50 19 45 32 45 C45 45 57 50 58 64 Z" fill={clothes} />
        <path d="M27 45.5 L32 51 L37 45.5" stroke="#fff" strokeWidth={1.5} fill="none" opacity={0.8} />
        <rect x={27} y={37} width={10} height={10} rx={3} fill={skin} />
        <circle cx={19.2} cy={28.5} r={3} fill={skin} />
        <circle cx={44.8} cy={28.5} r={3} fill={skin} />
        <ellipse data-part="face" cx={32} cy={27} rx={13} ry={15} fill={skin} />
        <path d="M32 29 Q30.4 32 32.6 32.2" stroke="rgba(0,0,0,0.25)" strokeWidth={1.2} fill="none" strokeLinecap="round" />
        <Eyes look={look} />
        <g stroke={hair} strokeWidth={1.5} strokeLinecap="round">
          <path d="M24.4 23.6 L29.2 23" />
          <path d="M34.8 23 L39.6 23.6" />
        </g>
        <FacialHair look={look} color={hair} />
        <Mouth look={look} />
        <Glasses look={look} />
        <HairFront look={look} color={hair} />
        <HatLayer look={look} clothes={clothes} />
      </g>
    </svg>
  );
}
```

`exactOptionalPropertyTypes`: `title?: string` uz `{title}` destrukturiranje je u redu; ako `a11y` spread javi grešku
tipa, razdvoji na dvije grane `return`. Boje u ovoj datoteci su **dio lika** (linija, bijela očiju, okvir naočala) —
dopuštena iznimka od pravila „heks samo u tokenima“ (Global Constraints). Ako prilikom provjere u pregledniku (Task 13)
neki sloj izgleda krivo, popravak je u ovoj datoteci, ne u ekranima.

- [ ] **Step 6: Pokreni avatar testove** → PASS.

- [ ] **Step 7: Galerija `/likovi`** — za Leona (vidi raznolikost) i kao početak uređivača. Tekstovi u `hr.ts`:

```ts
  "avatars.title": "Likovi",
  "avatars.lead": "Svaki lik se slaže iz slojeva. Uređivač dolazi poslije demoa.",
  "avatars.known": "Likovi iz igre",
  "avatars.player": "Ti",
  "avatars.mentor": "Mentor",
  "avatars.random": "Gosti iz imena",
  "avatars.hair": "Frizure",
```

```tsx
// screens/AvatarGallery.tsx
import { t } from "@hotelixsim/i18n";
import { Avatar } from "../avatar/Avatar";
import { HAIR_STYLES, KNOWN_LOOKS, lookFromSeed, MENTOR_LOOK, PLAYER_LOOK } from "../avatar/look";
import { Card } from "../ui/primitives";

const GUESTS = ["Ana Kovač", "Ivan Perić", "Maja Jurić", "Luka Novak", "Iva Babić", "Tomislav Marić", "Lana Vuković", "Karlo Šimić", "Ema Knežević"];

export function AvatarGallery() {
  return (
    <>
      <h1 className="text-3xl font-extrabold tracking-tight">{t("avatars.title")}</h1>
      <p className="text-muted">{t("avatars.lead")}</p>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.known")}</h2>
        <ul className="grid grid-cols-3 gap-3 text-center text-sm">
          <li>
            <Avatar look={PLAYER_LOOK} size={72} />
            {t("avatars.player")}
          </li>
          <li>
            <Avatar look={MENTOR_LOOK} size={72} />
            {t("avatars.mentor")}
          </li>
          {Object.entries(KNOWN_LOOKS).map(([name, look]) => (
            <li key={name}>
              <Avatar look={look} size={72} />
              {name}
            </li>
          ))}
        </ul>
      </Card>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.random")}</h2>
        <ul className="grid grid-cols-3 gap-3 text-center text-xs">
          {GUESTS.map((name) => (
            <li key={name}>
              <Avatar look={lookFromSeed(name)} size={64} />
              {name}
            </li>
          ))}
        </ul>
      </Card>
      <Card>
        <h2 className="pb-2 font-bold">{t("avatars.hair")}</h2>
        <ul className="grid grid-cols-4 gap-2 text-center text-xs">
          {HAIR_STYLES.map((hair) => (
            <li key={hair}>
              <Avatar look={{ ...PLAYER_LOOK, hair }} size={56} />
              {hair}
            </li>
          ))}
        </ul>
      </Card>
    </>
  );
}
```

(`li` s `Avatar` i tekstom: dodaj `className="flex flex-col items-center gap-1"` na svaki `li`.) Dodaj u
`avatar.test.tsx` jedan test: `render(<AvatarGallery />)` → `getByText("Petra Horvat")`, `getByText("Marko Babić")` i
`container.querySelectorAll("svg").length >= 2 + 2 + 9 + 8`.

- [ ] **Step 8: Provjere** (sve četiri). Uz to ručno: otvori `apps/web` dev server nije potreban — slike likova se
  gledaju u Tasku 13 (Playwright, `/likovi`). Ovdje provjeri barem da `vite build` prolazi.

- [ ] **Step 9: Commit** — `git add apps/web/src/avatar apps/web/src/screens/AvatarGallery.tsx packages/i18n/src/hr.ts`
  → „Ekrani 3/11: likovi iz slojeva — izgled kao podatak, temelj za uređivač“.

---

