import { useId } from "react";
import type { AvatarLook } from "./look";
import { BACKDROP, CLOTHES, HAIR, SKIN } from "./palette";

const at = <T,>(xs: readonly T[], i: number): T => xs[Math.min(Math.max(i, 0), xs.length - 1)] as T;
const LINE = "#3B2A22";
const GOLD = "#E0A82E";
const RED = "#B71C1C";

function HairBack({ look, color }: { look: AvatarLook; color: string }) {
  if (look.hair === "long")
    return (
      <path
        data-part="hairback-long"
        d="M17 26 C17 11 47 11 47 26 L49 50 C43 54 21 54 15 50 Z"
        fill={color}
      />
    );
  if (look.hair === "ponytail")
    return (
      <path data-part="hairback-ponytail" d="M44 21 C55 23 55 41 47 47 C50 37 48 30 43 27 Z" fill={color} />
    );
  if (look.hair === "afro") return <circle data-part="hairback-afro" cx={32} cy={20} r={17.5} fill={color} />;
  if (look.hair === "wavy")
    return (
      <path
        data-part="hairback-wavy"
        d="M17 26 C17 11 47 11 47 26 L48.5 41 Q45 44 42 41 Q39 38 36 41 L28 41 Q25 38 22 41 Q19 44 15.5 41 Z"
        fill={color}
      />
    );
  if (look.hair === "bob")
    return (
      <path
        data-part="hairback-bob"
        d="M16.5 27 C16.5 10 47.5 10 47.5 27 L47 40 C43 43 21 43 17 40 Z"
        fill={color}
      />
    );
  return null;
}

function HairFront({ look, color }: { look: AvatarLook; color: string }) {
  switch (look.hair) {
    case "none":
      return null;
    case "buzz":
      return (
        <path
          data-part="hair-buzz"
          d="M19.5 23 C20 11 44 11 44.5 23 C40 18 24 18 19.5 23 Z"
          fill={color}
          opacity={0.8}
        />
      );
    case "short":
      return (
        <path
          data-part="hair-short"
          d="M18.5 25 C16.5 10 47.5 9 45.5 25 C43 19 38 17 32 17 C26 17 21 19 18.5 25 Z"
          fill={color}
        />
      );
    case "side":
      return (
        <path
          data-part="hair-side"
          d="M18.5 26 C15.5 9 48.5 7 45.5 24 C41 15 31 22 22.5 19.5 C20.5 21.5 19.3 23.5 18.5 26 Z"
          fill={color}
        />
      );
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
      return (
        <path
          data-part="hair-long"
          d="M18 31 C14.5 9 49.5 9 46 31 C44 20 38 16 32 16 C26 16 20 20 18 31 Z"
          fill={color}
        />
      );
    case "ponytail":
      return (
        <path
          data-part="hair-ponytail"
          d="M18.5 25 C16.5 10 47.5 10 45.5 25 C42 18 22 18 18.5 25 Z"
          fill={color}
        />
      );
    case "bun":
      return (
        <g data-part="hair-bun" fill={color}>
          <circle cx={32} cy={9.5} r={6} />
          <path d="M18.5 25 C16.5 11 47.5 11 45.5 25 C42 18 22 18 18.5 25 Z" />
        </g>
      );
    case "afro":
      return (
        <path
          data-part="hair-afro"
          d="M18.5 26 C15 3 49 3 45.5 26 C43 22 37 18.5 29 19.5 C24 20 20.5 22.5 18.5 26 Z"
          fill={color}
        />
      );
    case "wavy":
      return (
        <path
          data-part="hair-wavy"
          d="M18.5 28 C15 4 49 4 45.5 28 C44.5 23 40 19 32 19.5 C26 20 21.5 22 20 25 C19.5 26 19 27 18.5 28 Z"
          fill={color}
        />
      );
    case "bob":
      return (
        <path
          data-part="hair-bob"
          d="M18 30 C15 4 49 4 46 30 C45 24 40 19.5 31 19.5 C25 20.5 21 22.5 19.5 25.5 C18.8 27 18.3 28.5 18 30 Z"
          fill={color}
        />
      );
    case "slick":
      return (
        <g data-part="hair-slick">
          <path d="M18.5 25 C16 4 48 3 45.5 25 C43 18 38 15 31 15 C25 15 21 18 18.5 25 Z" fill={color} />
          <path
            d="M24 14 C30 11 37 11.5 41 16"
            stroke="rgba(255,255,255,0.3)"
            strokeWidth={1}
            fill="none"
            strokeLinecap="round"
          />
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
        <circle cx={27.3} cy={28.4} r={1.5} fill={LINE} />
        <circle cx={37.3} cy={28.4} r={1.5} fill={LINE} />
        <circle cx={27.9} cy={27.7} r={0.5} fill="#fff" />
        <circle cx={37.9} cy={27.7} r={0.5} fill="#fff" />
      </g>
    );
  return (
    <g data-part="eyes-dot" fill={LINE}>
      <circle cx={27} cy={28} r={1.9} />
      <circle cx={37} cy={28} r={1.9} />
      <circle cx={27.6} cy={27.3} r={0.6} fill="#fff" />
      <circle cx={37.6} cy={27.3} r={0.6} fill="#fff" />
    </g>
  );
}

function Brows({ look, color }: { look: AvatarLook; color: string }) {
  const common = { stroke: color, strokeLinecap: "round" as const, fill: "none" };
  switch (look.brows) {
    case "straight":
      return (
        <g data-part="brows-straight" {...common} strokeWidth={1.5}>
          <path d="M24.4 23.6 Q26.8 22.2 29.2 23.4" />
          <path d="M34.8 23.4 Q37.2 22.2 39.6 23.6" />
        </g>
      );
    case "arched":
      return (
        <g data-part="brows-arched" {...common} strokeWidth={1.4}>
          <path d="M24.2 24 Q26.8 20.6 29.4 23.4" />
          <path d="M34.6 23.4 Q37.2 20.6 39.8 24" />
        </g>
      );
    case "thick":
      return (
        <g data-part="brows-thick" {...common} strokeWidth={2.4}>
          <path d="M24.6 23.8 Q26.8 22.4 29 23.4" />
          <path d="M35 23.4 Q37.2 22.4 39.4 23.8" />
        </g>
      );
    case "worried":
      return (
        <g data-part="brows-worried" {...common} strokeWidth={1.5}>
          <path d="M24.4 24.4 L29.2 22" />
          <path d="M34.8 22 L39.6 24.4" />
        </g>
      );
  }
}

function Nose({ look }: { look: AvatarLook }) {
  const common = {
    stroke: "rgba(0,0,0,0.25)",
    strokeWidth: 1.2,
    fill: "none",
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
  };
  switch (look.nose) {
    case "button":
      return <path data-part="nose-button" d="M32 29 Q30.4 32 32.6 32.2" {...common} />;
    case "long":
      return <path data-part="nose-long" d="M32.2 26.5 L31 32.4 Q32.4 33.6 33.8 32.4" {...common} />;
    case "wide":
      return (
        <path
          data-part="nose-wide"
          d="M32 29.5 L31 31.6 Q29.8 32.6 31.2 33 Q32 33.4 32.8 33 Q34.2 32.6 33 31.6"
          {...common}
        />
      );
  }
}

/** Starost: „young“ i „adult“ su bez dodatnog sloja, „senior“ ima bore na čelu i uz oči. */
function Age({ look }: { look: AvatarLook }) {
  if (look.age !== "senior") return null;
  return (
    <g data-part="age-senior" stroke="rgba(0,0,0,0.18)" strokeWidth={0.8} fill="none" strokeLinecap="round">
      <path d="M26 19.6 Q32 18.2 38 19.6" />
      <path d="M27 21.4 Q32 20.2 37 21.4" />
      <path d="M23.4 28.6 L21.6 27.6" />
      <path d="M23.4 30 L21.6 31" />
      <path d="M40.6 28.6 L42.4 27.6" />
      <path d="M40.6 30 L42.4 31" />
    </g>
  );
}

function EarringsLayer({ look }: { look: AvatarLook }) {
  if (look.earrings === "studs")
    return (
      <g data-part="earrings-studs" fill={GOLD}>
        <circle cx={19.2} cy={31.4} r={1} />
        <circle cx={44.8} cy={31.4} r={1} />
      </g>
    );
  if (look.earrings === "hoops")
    return (
      <g data-part="earrings-hoops" stroke={GOLD} strokeWidth={0.9} fill="none">
        <circle cx={19.2} cy={33} r={1.8} />
        <circle cx={44.8} cy={33} r={1.8} />
      </g>
    );
  return null;
}

/** Oblik odjeće preko boje `clothes` (sama boja se crta zasebno kao sloj „clothes“). */
function Outfit({ look, skin }: { look: AvatarLook; skin: string }) {
  switch (look.outfit) {
    case "shirt":
      return (
        <path
          data-part="outfit-shirt"
          d="M27 45.5 L32 51 L37 45.5"
          stroke="#fff"
          strokeWidth={1.5}
          fill="none"
          opacity={0.8}
        />
      );
    case "blazer":
      return (
        <g data-part="outfit-blazer">
          <path d="M27.3 45.2 L32 56 L36.7 45.2 Z" fill="#fff" />
          <path d="M23.5 46.8 L31.2 57.5 L27.3 45.2 Z" fill="rgba(0,0,0,0.25)" />
          <path d="M40.5 46.8 L32.8 57.5 L36.7 45.2 Z" fill="rgba(0,0,0,0.25)" />
        </g>
      );
    case "uniform":
      return (
        <g data-part="outfit-uniform">
          <path
            d="M26.5 45.5 L29.5 43.8 L32 48.5 L34.5 43.8 L37.5 45.5 L36 49.5 L32 51 L28 49.5 Z"
            fill="rgba(255,255,255,0.4)"
            stroke="rgba(0,0,0,0.25)"
            strokeWidth={0.6}
          />
          <circle cx={32} cy={55} r={0.9} fill="rgba(255,255,255,0.85)" />
          <circle cx={32} cy={59.5} r={0.9} fill="rgba(255,255,255,0.85)" />
          <rect x={38.5} y={52.5} width={6} height={3} rx={0.6} fill="#fff" />
        </g>
      );
    case "apron":
      return (
        <g data-part="outfit-apron">
          <path d="M25 64 L26 54 L28.5 50.5 L35.5 50.5 L38 54 L39 64 Z" fill="#fff" />
          <path
            d="M28.5 50.5 L27 43.5 M35.5 50.5 L37 43.5"
            stroke="#fff"
            strokeWidth={1.3}
            strokeLinecap="round"
          />
        </g>
      );
    case "tshirt":
      return (
        <path
          data-part="outfit-tshirt"
          d="M26.5 45.2 Q32 54 37.5 45.2 Z"
          fill={skin}
          stroke="rgba(0,0,0,0.15)"
          strokeWidth={0.6}
        />
      );
  }
}

function Neckwear({ look }: { look: AvatarLook }) {
  switch (look.neckwear) {
    case "none":
      return null;
    case "tie":
      return (
        <g data-part="neckwear-tie" fill={RED}>
          <path d="M30.4 46.4 L33.6 46.4 L34.2 49.2 L29.8 49.2 Z" />
          <path d="M29.8 49.2 L34.2 49.2 L35.4 60 L32 62.5 L28.6 60 Z" />
        </g>
      );
    case "scarf":
      return (
        <g data-part="neckwear-scarf" fill={GOLD}>
          <path d="M24.5 44.8 Q32 52 39.5 44.8 L39.2 49 Q32 56.5 24.8 49 Z" />
          <path d="M35.5 51.5 L39 61 L34.5 59.5 Z" />
        </g>
      );
    case "bowtie":
      return (
        <g data-part="neckwear-bowtie" fill={RED}>
          <path d="M32 47.4 L25.8 44.2 L25.8 50.6 Z" />
          <path d="M32 47.4 L38.2 44.2 L38.2 50.6 Z" />
          <circle cx={32} cy={47.4} r={1.4} />
        </g>
      );
  }
}

function Mouth({ look }: { look: AvatarLook }) {
  const common = { stroke: LINE, strokeWidth: 1.6, fill: "none", strokeLinecap: "round" as const };
  switch (look.mouth) {
    case "smile":
      return <path data-part="mouth-smile" d="M27 34.2 Q32 39.2 37 34.2" {...common} />;
    case "neutral":
      return <path data-part="mouth-neutral" d="M28.5 35 Q32 36.8 35.5 35" {...common} />;
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
      return (
        <path
          data-part="facial-stubble"
          d="M20 31 C22 44 42 44 44 31 C40 38 24 38 20 31 Z"
          fill={color}
          opacity={0.3}
        />
      );
    case "mustache":
      return (
        <path
          data-part="facial-mustache"
          d="M27.2 33.6 Q32 30.6 36.8 33.6 Q32 34.8 27.2 33.6 Z"
          fill={color}
        />
      );
    case "beard":
      return (
        <path
          data-part="facial-beard"
          d="M19.5 30 C20 46.5 44 46.5 44.5 30 C42 38.5 38 39.5 32 39.5 C26 39.5 22 38.5 19.5 30 Z"
          fill={color}
        />
      );
    case "goatee":
      return (
        <path
          data-part="facial-goatee"
          d="M28.4 37.2 Q32 35.4 35.6 37.2 Q35.4 42.4 32 43 Q28.6 42.4 28.4 37.2 Z"
          fill={color}
        />
      );
    case "full":
      return (
        <g data-part="facial-full" fill={color}>
          <path d="M19.5 30 C20 46.5 44 46.5 44.5 30 C42 38.5 38 39.5 32 39.5 C26 39.5 22 38.5 19.5 30 Z" />
          <path d="M27.2 33.6 Q32 30.6 36.8 33.6 Q32 34.8 27.2 33.6 Z" />
        </g>
      );
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
        <path d="M18 23 C17 6.5 47 6.5 46 23 Z" />
        <path d="M30 20.2 L51 21.6 C50 25.2 40 25.4 30 23.6 Z" />
      </g>
    );
  if (look.hat === "beanie")
    return (
      <g data-part="hat-beanie" fill={clothes}>
        <path d="M18 22 C17 6.5 47 6.5 46 22 Z" />
        <rect x={17.4} y={19.6} width={29.2} height={5} rx={2.2} />
        <rect x={17.4} y={19.6} width={29.2} height={5} rx={2.2} fill="rgba(0,0,0,0.2)" />
        <circle cx={32} cy={7.2} r={3} />
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
  const layers = (
    <>
      <defs>
        <clipPath id={clip}>
          <circle cx={32} cy={32} r={32} />
        </clipPath>
      </defs>
      <g clipPath={`url(#${clip})`}>
        <rect data-part="backdrop" width={64} height={64} fill={at(BACKDROP, look.backdrop)} />
        <HairBack look={look} color={hair} />
        <path data-part="clothes" d="M6 64 C7 50 19 45 32 45 C45 45 57 50 58 64 Z" fill={clothes} />
        <rect x={27} y={37} width={10} height={10} rx={3} fill={skin} />
        <Outfit look={look} skin={skin} />
        <Neckwear look={look} />
        <circle cx={19.2} cy={28.5} r={3} fill={skin} />
        <circle cx={44.8} cy={28.5} r={3} fill={skin} />
        <ellipse data-part="face" cx={32} cy={27} rx={13} ry={15} fill={skin} />
        <EarringsLayer look={look} />
        <Age look={look} />
        <g data-part="cheeks" fill="rgba(255,110,110,0.16)">
          <circle cx={24} cy={33} r={2.4} />
          <circle cx={40} cy={33} r={2.4} />
        </g>
        <Nose look={look} />
        <Eyes look={look} />
        <Brows look={look} color={hair} />
        <FacialHair look={look} color={hair} />
        <Mouth look={look} />
        <Glasses look={look} />
        <HairFront look={look} color={hair} />
        <HatLayer look={look} clothes={clothes} />
      </g>
    </>
  );
  if (title)
    return (
      <svg viewBox="0 0 64 64" width={size} height={size} className="shrink-0" role="img" aria-label={title}>
        {layers}
      </svg>
    );
  return (
    <svg viewBox="0 0 64 64" width={size} height={size} className="shrink-0" aria-hidden="true">
      {layers}
    </svg>
  );
}
