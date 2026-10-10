import { useId } from "react";
import type { AvatarLook } from "./look";
import { BACKDROP, CLOTHES, HAIR, SKIN } from "./palette";

const at = <T,>(xs: readonly T[], i: number): T => xs[Math.min(Math.max(i, 0), xs.length - 1)] as T;
const LINE = "#3B2A22";

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
        <path d="M27 45.5 L32 51 L37 45.5" stroke="#fff" strokeWidth={1.5} fill="none" opacity={0.8} />
        <rect x={27} y={37} width={10} height={10} rx={3} fill={skin} />
        <circle cx={19.2} cy={28.5} r={3} fill={skin} />
        <circle cx={44.8} cy={28.5} r={3} fill={skin} />
        <ellipse data-part="face" cx={32} cy={27} rx={13} ry={15} fill={skin} />
        <path
          d="M32 29 Q30.4 32 32.6 32.2"
          stroke="rgba(0,0,0,0.25)"
          strokeWidth={1.2}
          fill="none"
          strokeLinecap="round"
        />
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
